//! Test-only public betting tree. Hidden opponent cards carry reach weights.
//! The only input shared with the solver is the named behavioral policy.

use paignia_core::InformationSetId;
use paignia_solve::CfrStrategy;

#[derive(Clone, Copy, Default, Debug)]
pub(super) struct Returns {
    pub(super) on_policy: f64,
    pub(super) best_response: f64,
}

impl Returns {
    fn terminal(value: f64) -> Self {
        Self {
            on_policy: value,
            best_response: value,
        }
    }

    fn add(&mut self, other: Self) {
        self.on_policy += other.on_policy;
        self.best_response += other.best_response;
    }
}

#[derive(Clone)]
struct Betting {
    history: [Vec<&'static str>; 2],
    board: Option<usize>,
    // Equal investment per seat when this round began, including the ante.
    starting_stake: i32,
}

impl Betting {
    fn round(&self) -> usize {
        usize::from(self.board.is_some())
    }

    fn size(&self) -> i32 {
        if self.board.is_some() {
            4
        } else {
            2
        }
    }

    fn raises(&self) -> i32 {
        self.history[self.round()]
            .iter()
            .map(|&play| i32::from(play == "r"))
            .sum()
    }

    fn actions(&self) -> &'static [&'static str] {
        match self.raises() {
            0 => &["c", "r"],
            1 => &["f", "c", "r"],
            2 => &["f", "c"],
            _ => unreachable!("two-raise cap"),
        }
    }

    fn actor(&self) -> usize {
        self.history[self.round()].len() % 2
    }

    fn probability(&self, policy: &CfrStrategy, card: usize, play: &str) -> f64 {
        let board = self.board.map_or(-1, |b| i32::try_from(b).unwrap());
        let id = InformationSetId(format!(
            "urn:paignia:cfr:set:leduc-p{}-c{card}-b{board}-{}|{}",
            self.actor(),
            self.history[0].join(","),
            self.history[1].join(",")
        ));
        let name = match play {
            "f" => "urn:paignia:cfr:action:fold",
            "c" => "urn:paignia:cfr:action:call",
            "r" => "urn:paignia:cfr:action:raise",
            _ => unreachable!("unknown play"),
        };
        let index = policy.action_order[&id]
            .iter()
            .position(|action| action.0 == name)
            .expect("fixture policy contains the legal action");
        policy.probabilities[&id][index]
    }
}

struct Responder<'a> {
    seat: usize,
    private: usize,
    policy: &'a CfrStrategy,
}

impl Responder<'_> {
    fn evaluate(&self, state: &Betting, weights: [f64; 6]) -> Returns {
        if weights.iter().all(|&weight| weight == 0.0) {
            return Returns::default();
        }
        let mut result = Returns::default();
        if state.actor() == self.seat {
            // All compatible opponent cards share this information set. Maximize
            // only after summing the continuation over the whole distribution.
            result.best_response = f64::NEG_INFINITY;
            for &play in state.actions() {
                let child = self.after(state, weights, play);
                result.on_policy +=
                    state.probability(self.policy, self.private, play) * child.on_policy;
                result.best_response = result.best_response.max(child.best_response);
            }
        } else {
            for &play in state.actions() {
                let mut weighted = weights;
                for (card, weight) in weighted.iter_mut().enumerate() {
                    if *weight > 0.0 {
                        *weight *= state.probability(self.policy, card, play);
                    }
                }
                result.add(self.after(state, weighted, play));
            }
        }
        result
    }

    fn after(&self, state: &Betting, weights: [f64; 6], play: &'static str) -> Returns {
        let raises = state.raises();
        if play == "f" {
            let lost = state.starting_stake + (raises - 1) * state.size();
            let signed = if state.actor() == self.seat {
                -lost
            } else {
                lost
            };
            return Returns::terminal(weights.iter().sum::<f64>() * f64::from(signed));
        }
        let round = state.round();
        let mut child = state.clone();
        child.history[round].push(play);
        if play == "r" || state.history[round].is_empty() {
            return self.evaluate(&child, weights);
        }

        let matched_chips = state.starting_stake + raises * state.size();
        if let Some(board) = state.board {
            let rank = |card: usize| (card / 2 == board / 2, card / 2);
            let mut value = 0.0;
            for (other, weight) in weights.into_iter().enumerate() {
                let sign = match rank(self.private).cmp(&rank(other)) {
                    std::cmp::Ordering::Greater => 1,
                    std::cmp::Ordering::Equal => 0,
                    std::cmp::Ordering::Less => -1,
                };
                value += weight * f64::from(matched_chips * sign);
            }
            return Returns::terminal(value);
        }

        // Four boards remain for any fixed private pair. Split the belief only
        // when the public card becomes visible to both players.
        let mut result = Returns::default();
        child.starting_stake = matched_chips;
        for board in (0..6).filter(|&card| card != self.private) {
            child.board = Some(board);
            let mut dealt = weights.map(|weight| weight / 4.0);
            dealt[board] = 0.0;
            result.add(self.evaluate(&child, dealt));
        }
        result
    }
}

pub(super) fn returns(policy: &CfrStrategy, seat: usize) -> Returns {
    let root = Betting {
        history: [vec![], vec![]],
        board: None,
        starting_stake: 1,
    };
    let mut total = Returns::default();
    for private in 0..6 {
        // Each ordered pair of private cards has unconditional probability 1/30.
        let mut weights = [1.0 / 30.0; 6];
        weights[private] = 0.0;
        total.add(
            Responder {
                seat,
                private,
                policy,
            }
            .evaluate(&root, weights),
        );
    }
    total
}
