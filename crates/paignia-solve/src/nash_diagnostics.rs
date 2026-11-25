//! Payoff and deviation diagnostics shared by the admitted Nash methods.

use paignia_core::PlayerId;
use thiserror::Error;

use crate::{NormalFormGame, SolveError, StrategyProfile};

/// Original-unit expected payoff and gain from a best unilateral deviation.
/// Algorithms retain their admission and weighting rules for candidate strategies
/// whose probability mass is within tolerance of one.
#[derive(Clone, Debug, PartialEq)]
pub struct PlayerDeviation<P> {
    pub player: P,
    pub expected_payoff: f64,
    pub gain: f64,
}

/// Candidate diagnostics. Check gains against a tolerance chosen for the game.
///
/// When players' payoff units differ, use the individual gains to interpret
/// deviations. The sum uses a rounding correction; a legacy diagnostic can
/// differ slightly in how it rounds the same aggregate.
#[derive(Clone, Debug, PartialEq)]
pub struct NashDeviationReport<P> {
    pub players: Vec<PlayerDeviation<P>>,
    pub max_gain: f64,
    pub nash_conv: f64,
}

impl<P> NashDeviationReport<P> {
    pub(crate) fn from_players(players: Vec<PlayerDeviation<P>>) -> Option<Self> {
        let (sum, correction) = players.iter().fold((0.0, 0.0), |(sum, correction), row| {
            let (next, error) = crate::two_sum(sum, row.gain);
            (next, correction + error)
        });
        let nash_conv = sum + correction;
        nash_conv.is_finite().then(|| Self {
            max_gain: players.iter().map(|row| row.gain).fold(0.0, f64::max),
            nash_conv,
            players,
        })
    }
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum PureNashEvaluationError {
    #[error(transparent)]
    InvalidGame(#[from] SolveError),
    #[error("candidate strategies differ from the game's declared strategy domain")]
    InvalidProfile,
    #[error("unilateral deviation or NashConv is outside finite binary64 range")]
    NonFiniteGain,
}

/// Evaluate a deterministic strategy profile on a complete finite payoff table.
/// Gains use payoff subtraction. Pure enumeration compares the best payoff
/// with the current payoff plus a tolerance, which can round differently.
///
/// # Errors
/// Rejects an invalid game, a profile outside its strategy domain, or an
/// unrepresentable gain. Validates the full payoff table and evaluates only
/// the supplied candidate.
pub fn evaluate_pure_nash_candidate(
    game: &NormalFormGame,
    profile: &StrategyProfile,
) -> Result<NashDeviationReport<PlayerId>, PureNashEvaluationError> {
    game.validate()?;
    if profile.len() != game.players.len()
        || game.players.iter().any(|player| {
            !profile
                .get(player)
                .is_some_and(|action| game.strategies[player].contains(action))
        })
    {
        return Err(PureNashEvaluationError::InvalidProfile);
    }
    let current = &game.payoffs[profile];
    let mut players = Vec::with_capacity(game.players.len());
    let mut deviation = profile.clone();
    for player in &game.players {
        let expected_payoff = current[player];
        let mut best_payoff = expected_payoff;
        for action in &game.strategies[player] {
            deviation.insert(player.clone(), action.clone());
            best_payoff = best_payoff.max(game.payoffs[&deviation][player]);
        }
        deviation.insert(player.clone(), profile[player].clone());
        players.push(PlayerDeviation {
            player: player.clone(),
            expected_payoff,
            gain: best_payoff - expected_payoff,
        });
    }
    NashDeviationReport::from_players(players).ok_or(PureNashEvaluationError::NonFiniteGain)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(gains: &[f64]) -> Option<NashDeviationReport<usize>> {
        NashDeviationReport::from_players(
            gains
                .iter()
                .enumerate()
                .map(|(player, &gain)| PlayerDeviation {
                    player,
                    expected_payoff: 0.0,
                    gain,
                })
                .collect(),
        )
    }

    #[test]
    fn retains_representable_small_gains_and_rejects_aggregate_overflow() {
        let scale = 2_f64.powi(53);
        let actual = report(&[scale, 1.0, 1.0]).unwrap();
        assert_eq!(actual.nash_conv.to_bits(), (scale + 2.0).to_bits());
        let small = 2_f64.powi(969);
        assert!(report(&[f64::MAX, small, small, small]).is_none());
    }
}
