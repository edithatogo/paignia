//! Labelled tabular discounted, fully observed, two-player zero-sum games.

use std::collections::BTreeSet;
use thiserror::Error;

use crate::{ActionId, NodeId, PlayerId};

pub const MAX_STATES: usize = 32;
pub const MAX_ABS_REWARD: f64 = 1.0e12;
pub const MASS_TOLERANCE: f64 = 1.0e-12;

/// The probability axis follows `Game::states`. The immediate reward belongs
/// to player zero. Player one's reward is its negation.
#[derive(Clone, Debug, PartialEq)]
pub struct Outcome {
    pub reward: f64,
    pub transition: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct State {
    pub id: NodeId,
    /// Outcomes declared in row-player then column-player action order.
    pub outcomes: [[Outcome; 2]; 2],
}

/// Both players observe the current state and choose simultaneously. State
/// and action order define the axes of transitions and policy weights.
#[derive(Clone, Debug, PartialEq)]
pub struct Game {
    pub players: [PlayerId; 2],
    pub actions: [[ActionId; 2]; 2],
    pub states: Vec<State>,
    pub initial: Vec<f64>,
    pub discount: f64,
}

/// One action weight pair per player per state, in the game's declared order.
#[derive(Clone, Debug, PartialEq)]
pub struct StationaryPolicy {
    pub weights: Vec<[[f64; 2]; 2]>,
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum Error {
    #[error("expected one to {MAX_STATES} states")]
    StateCount,
    #[error("players, states and each player's actions require unique nonblank identifiers")]
    Identity,
    #[error("discount must be in [0, 1)")]
    Discount,
    #[error("invalid initial state distribution")]
    Initial,
    #[error("invalid transition at state {state}, row {row}, column {column}")]
    Transition {
        state: usize,
        row: usize,
        column: usize,
    },
    #[error("invalid or out-of-bounds reward at state {state}, row {row}, column {column}")]
    Reward {
        state: usize,
        row: usize,
        column: usize,
    },
    #[error("expected one policy record per state")]
    PolicyCount,
    #[error("invalid distribution at state {state} for player {player}")]
    Policy { state: usize, player: usize },
}

impl Game {
    /// Validate identifiers, axes, the discount and every joint outcome.
    ///
    /// # Errors
    ///
    /// Returns the offending position or identifier category on invalid input.
    pub fn validate(&self) -> Result<(), Error> {
        let count = self.states.len();
        if !(1..=MAX_STATES).contains(&count) {
            return Err(Error::StateCount);
        }
        if !identities(self.players.iter().map(|id| id.0.as_str()))
            || !self
                .actions
                .iter()
                .all(|seat| identities(seat.iter().map(|id| id.0.as_str())))
            || !identities(self.states.iter().map(|state| state.id.0.as_str()))
        {
            return Err(Error::Identity);
        }
        if !(0.0..1.0).contains(&self.discount) {
            return Err(Error::Discount);
        }
        if self.initial.len() != count || !distribution(&self.initial) {
            return Err(Error::Initial);
        }
        for (state, declaration) in self.states.iter().enumerate() {
            for (row, outcomes) in declaration.outcomes.iter().enumerate() {
                for (column, outcome) in outcomes.iter().enumerate() {
                    if !outcome.reward.is_finite() || outcome.reward.abs() > MAX_ABS_REWARD {
                        return Err(Error::Reward { state, row, column });
                    }
                    if outcome.transition.len() != count || !distribution(&outcome.transition) {
                        return Err(Error::Transition { state, row, column });
                    }
                }
            }
        }
        Ok(())
    }
}

impl StationaryPolicy {
    /// Validate coverage and policy mass for each state and player.
    ///
    /// # Errors
    ///
    /// Returns a policy error if the state inventory or any distribution differs
    /// from the model contract.
    pub fn validate(&self, game: &Game) -> Result<(), Error> {
        if self.weights.len() != game.states.len() {
            return Err(Error::PolicyCount);
        }
        for (state, seats) in self.weights.iter().enumerate() {
            for (player, weights) in seats.iter().enumerate() {
                if !distribution(weights) {
                    return Err(Error::Policy { state, player });
                }
            }
        }
        Ok(())
    }
}

fn identities<'a>(mut ids: impl Iterator<Item = &'a str>) -> bool {
    let mut seen = BTreeSet::new();
    ids.all(|id| !id.trim().is_empty() && seen.insert(id))
}

fn distribution(weights: &[f64]) -> bool {
    weights.iter().all(|weight| (0.0..=1.0).contains(weight))
        && leading_sum_part(weights.iter().copied().chain([-1.0, -MASS_TOLERANCE])) <= 0.0
        && leading_sum_part(weights.iter().copied().chain([-1.0, MASS_TOLERANCE])) >= 0.0
}

fn leading_sum_part(values: impl Iterator<Item = f64>) -> f64 {
    // Keep each rounded addition and its exact remainder. Non-overlapping
    // parts are ordered by magnitude, so the final part determines the sign.
    let mut parts = Vec::<f64>::new();
    for mut high in values {
        let mut retained = 0;
        for index in 0..parts.len() {
            let mut part = parts[index];
            if high.abs() < part.abs() {
                std::mem::swap(&mut high, &mut part);
            }
            let sum = high + part;
            let remainder = part - (sum - high);
            if remainder != 0.0 {
                parts[retained] = remainder;
                retained += 1;
            }
            high = sum;
        }
        parts.truncate(retained);
        if high != 0.0 {
            parts.push(high);
        }
    }
    parts.last().copied().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::distribution;

    #[test]
    fn admits_exact_probability_mass_independently_of_order() {
        let epsilon = f64::EPSILON;
        let mut excess = [1.0, 4503.0 * epsilon, 0.375 * epsilon, 0.375 * epsilon];
        assert!(!distribution(&excess));
        excess.reverse();
        assert!(!distribution(&excess));

        let mut boundary = [1.0, super::MASS_TOLERANCE, 0.0];
        assert!(distribution(&boundary));
        boundary.reverse();
        assert!(distribution(&boundary));
        assert!(!distribution(&[
            1.0,
            super::MASS_TOLERANCE,
            f64::from_bits(1)
        ]));
    }
}
