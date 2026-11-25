//! Certified stationary policy evaluation and two-by-two Shapley backups.

use paignia_core::discounted::{self, Game, StationaryPolicy};
use paignia_core::NodeId;
use thiserror::Error;

use crate::interval::{ArithmeticError, Interval};

pub const MAX_ITERATIONS: u32 = 10_000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Config {
    pub max_iterations: u32,
    pub value_tolerance: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Status {
    ToleranceReached,
    IterationLimit,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Evaluation {
    /// Player zero's values. Player one's values have the opposite sign.
    pub state_values: Vec<(NodeId, f64)>,
    pub initial_value: f64,
    pub iterations: u32,
    pub residual_bound: f64,
    pub value_error_bound: f64,
    pub initial_error_bound: f64,
    pub status: Status,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Equilibrium {
    pub policy: StationaryPolicy,
    pub game_values: Evaluation,
    pub policy_values: Evaluation,
    pub row_best_response: Evaluation,
    /// These values use the same payoff sign as `policy_values`.
    pub column_best_response: Evaluation,
    /// Certified unilateral gain upper bounds over all admitted states.
    pub deviation_bounds: [f64; 2],
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Input(#[from] discounted::Error),
    #[error("iteration cap must be in [1, {MAX_ITERATIONS}] and tolerance finite and nonnegative")]
    Config,
    #[error("could not certify the arithmetic: {0}")]
    Arithmetic(&'static str),
}

impl From<ArithmeticError> for Error {
    fn from(error: ArithmeticError) -> Self {
        Self::Arithmetic(error.0)
    }
}

type Matrix = [[Interval; 2]; 2];

struct Move {
    reward: Interval,
    transition: Vec<Interval>,
}

struct Model<'a> {
    game: &'a Game,
    moves: Vec<[[Move; 2]; 2]>,
    initial: Vec<Interval>,
    discount: Interval,
}

#[derive(Clone, Copy)]
enum Backup<'a> {
    Fixed(&'a [Matrix]),
    Minimax,
    RowResponse(&'a [Matrix]),
    ColumnResponse(&'a [Matrix]),
}

/// Evaluate a stationary policy pair from every state and the initial mixture.
/// A submitted probability row denotes its weights divided by their exact sum.
///
/// # Errors
///
/// Returns an admission, budget-configuration or arithmetic error.
pub fn evaluate(
    game: &Game,
    policy: &StationaryPolicy,
    config: Config,
) -> Result<Evaluation, Error> {
    let model = Model::admit(game, config)?;
    policy.validate(game)?;
    model.iterate(Backup::Fixed(&policy_weights(policy)?), config)
}

/// Compute bounded Shapley values and independently evaluate both best responses
/// to the returned stationary policies. Inspect both deviation bounds and the
/// reported statuses when using the result.
///
/// # Errors
///
/// Returns an admission, budget-configuration or arithmetic error.
pub fn solve_zero_sum(game: &Game, config: Config) -> Result<Equilibrium, Error> {
    let model = Model::admit(game, config)?;
    let game_values = model.iterate(Backup::Minimax, config)?;
    let points: Vec<_> = game_values
        .state_values
        .iter()
        .map(|(_, value)| Interval::point(*value))
        .collect();
    let weights = model
        .moves
        .iter()
        .map(|moves| Ok(saddle_weights(model.matrix(moves, &points)?)))
        .collect::<Result<_, Error>>()?;
    let policy = StationaryPolicy { weights };
    let weights = policy_weights(&policy)?;
    let policy_values = model.iterate(Backup::Fixed(&weights), config)?;
    let row_best_response = model.iterate(Backup::RowResponse(&weights), config)?;
    let column_best_response = model.iterate(Backup::ColumnResponse(&weights), config)?;
    let deviation_bounds = [
        difference_bound(&row_best_response, &policy_values)?,
        difference_bound(&policy_values, &column_best_response)?,
    ];
    Ok(Equilibrium {
        policy,
        game_values,
        policy_values,
        row_best_response,
        column_best_response,
        deviation_bounds,
    })
}

fn difference_bound(upper: &Evaluation, lower: &Evaluation) -> Result<f64, Error> {
    let error =
        Interval::point(upper.value_error_bound).add(Interval::point(lower.value_error_bound))?;
    upper
        .state_values
        .iter()
        .zip(&lower.state_values)
        .try_fold(0.0_f64, |maximum, (a, b)| {
            let gap = Interval::point(a.1).sub(Interval::point(b.1))?.add(error)?;
            Ok(maximum.max(gap.upper))
        })
}

fn total(weights: &[f64]) -> Result<Interval, ArithmeticError> {
    weights
        .iter()
        .try_fold(Interval::point(0.0), |sum, &value| {
            sum.add(Interval::point(value))
        })
}

fn normalize(weights: &[f64]) -> Result<Vec<Interval>, ArithmeticError> {
    let denominator = total(weights)?;
    weights
        .iter()
        .map(|&value| Interval::point(value).div_positive(denominator))
        .collect()
}

fn pair(weights: [f64; 2]) -> Result<[Interval; 2], ArithmeticError> {
    let denominator = total(&weights)?;
    Ok([
        Interval::point(weights[0]).div_positive(denominator)?,
        Interval::point(weights[1]).div_positive(denominator)?,
    ])
}

fn policy_weights(policy: &StationaryPolicy) -> Result<Vec<Matrix>, ArithmeticError> {
    policy
        .weights
        .iter()
        .map(|&[row, column]| Ok([pair(row)?, pair(column)?]))
        .collect()
}

fn weighted(weights: &[Interval], values: &[Interval]) -> Result<Interval, ArithmeticError> {
    weights
        .iter()
        .zip(values)
        .try_fold(Interval::point(0.0), |sum, (&weight, &value)| {
            sum.add(weight.mul(value)?)
        })
}

fn midpoint(range: Interval) -> f64 {
    range.lower + 0.5 * (range.upper - range.lower)
}

fn choose_probability([[a, b], [c, d]]: [[f64; 2]; 2]) -> f64 {
    let score = |p: f64| (p * a + (1.0 - p) * c).min(p * b + (1.0 - p) * d);
    let intersection = (d - c) / ((a - c) - (b - d));
    [0.0, intersection].into_iter().fold(1.0, |best, p| {
        if (0.0..=1.0).contains(&p) && score(p) > score(best) {
            p
        } else {
            best
        }
    })
}

fn saddle_weights(matrix: Matrix) -> [[f64; 2]; 2] {
    let [[a, b], [c, d]] = matrix.map(|row| row.map(midpoint));
    let row = choose_probability([[a, b], [c, d]]);
    let column = choose_probability([[-a, -c], [-b, -d]]);
    [[row, 1.0 - row], [column, 1.0 - column]]
}

fn transpose([[a, b], [c, d]]: Matrix) -> Matrix {
    [[a, c], [b, d]]
}

fn action_values(matrix: Matrix, other: [Interval; 2]) -> Result<[Interval; 2], ArithmeticError> {
    Ok([weighted(&other, &matrix[0])?, weighted(&other, &matrix[1])?])
}

fn bound([a, b]: [Interval; 2], choose: fn(f64, f64) -> f64) -> Interval {
    Interval {
        lower: choose(a.lower, b.lower),
        upper: choose(a.upper, b.upper),
    }
}

impl Backup<'_> {
    fn apply(self, matrix: Matrix, state: usize) -> Result<Interval, ArithmeticError> {
        match self {
            Self::Fixed(weights) => {
                let [row, column] = weights[state];
                weighted(&row, &action_values(matrix, column)?)
            }
            Self::RowResponse(weights) => {
                Ok(bound(action_values(matrix, weights[state][1])?, f64::max))
            }
            Self::ColumnResponse(weights) => Ok(bound(
                action_values(transpose(matrix), weights[state][0])?,
                f64::min,
            )),
            Self::Minimax => {
                // Every feasible row/column strategy supplies one side of the
                // matrix-game value, even when rounding perturbs the intersection.
                let [row, column] = saddle_weights(matrix);
                let lower = bound(action_values(transpose(matrix), pair(row)?)?, f64::min).lower;
                let upper = bound(action_values(matrix, pair(column)?)?, f64::max).upper;
                Ok(Interval { lower, upper })
            }
        }
    }
}

impl<'a> Model<'a> {
    fn admit(game: &'a Game, config: Config) -> Result<Self, Error> {
        game.validate()?;
        if !(1..=MAX_ITERATIONS).contains(&config.max_iterations)
            || !config.value_tolerance.is_finite()
            || config.value_tolerance < 0.0
        {
            return Err(Error::Config);
        }
        let project = |outcome: &discounted::Outcome| -> Result<Move, ArithmeticError> {
            Ok(Move {
                reward: Interval::point(outcome.reward),
                transition: normalize(&outcome.transition)?,
            })
        };
        let moves = game
            .states
            .iter()
            .map(|state| {
                let [[a, b], [c, d]] = &state.outcomes;
                Ok([[project(a)?, project(b)?], [project(c)?, project(d)?]])
            })
            .collect::<Result<_, ArithmeticError>>()?;
        Ok(Self {
            game,
            moves,
            initial: normalize(&game.initial)?,
            discount: Interval::point(game.discount),
        })
    }

    fn matrix(
        &self,
        [[a, b], [c, d]]: &[[Move; 2]; 2],
        values: &[Interval],
    ) -> Result<Matrix, ArithmeticError> {
        let backup = |outcome: &Move| {
            outcome
                .reward
                .add(self.discount.mul(weighted(&outcome.transition, values)?)?)
        };
        Ok([[backup(a)?, backup(b)?], [backup(c)?, backup(d)?]])
    }

    fn iterate(&self, operator: Backup<'_>, config: Config) -> Result<Evaluation, Error> {
        let contraction_gap = Interval::point(1.0).sub(self.discount)?;
        let mut values = vec![Interval::point(0.0); self.moves.len()];
        for iterations in 0..=config.max_iterations {
            let updates = self
                .moves
                .iter()
                .enumerate()
                .map(|(state, moves)| operator.apply(self.matrix(moves, &values)?, state))
                .collect::<Result<Vec<_>, ArithmeticError>>()?;
            let residual = updates.iter().zip(&values).try_fold(
                0.0_f64,
                |maximum, (&updated, &current)| {
                    Ok::<_, ArithmeticError>(maximum.max(updated.sub(current)?.abs_upper()))
                },
            )?;
            let error = Interval::point(residual)
                .div_positive(contraction_gap)?
                .upper;
            let converged = error <= config.value_tolerance;
            if converged || iterations == config.max_iterations {
                let initial = weighted(&self.initial, &values)?;
                let estimate = midpoint(initial);
                let mixing_error = initial.sub(Interval::point(estimate))?.abs_upper();
                return Ok(Evaluation {
                    state_values: self
                        .game
                        .states
                        .iter()
                        .zip(&values)
                        .map(|(state, value)| (state.id.clone(), value.lower))
                        .collect(),
                    initial_value: estimate,
                    iterations,
                    residual_bound: residual,
                    value_error_bound: error,
                    initial_error_bound: Interval::point(error)
                        .add(Interval::point(mixing_error))?
                        .upper,
                    status: if converged {
                        Status::ToleranceReached
                    } else {
                        Status::IterationLimit
                    },
                });
            }
            for (value, update) in values.iter_mut().zip(updates) {
                *value = Interval::point(midpoint(update));
            }
        }
        unreachable!("the inclusive loop reports its final allowed iteration")
    }
}
