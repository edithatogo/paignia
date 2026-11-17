//! Bounded two-player CE linear programs with independent interval admission.
//!
//! Raw binary64 joint weights are retained. Admission concerns the explicitly
//! approximate LP point and its residual bounds, never an exact probability
//! distribution or an exact-arithmetic equilibrium certificate.
#![allow(clippy::float_cmp)] // Interval point identities require exact equality.

use clarabel::algebra::CscMatrix;
use clarabel::solver::{
    DefaultSettingsBuilder, DefaultSolution, DefaultSolver, IPSolver, NonnegativeConeT,
    SolverStatus, ZeroConeT,
};
use thiserror::Error;

pub const MAX_ACTIONS: usize = 8;
pub const MAX_ITERATIONS: u32 = 10_000;
pub const METHOD: &str = "paignia/clarabel-correlated-equilibrium";
pub const VERSION: &str = "1.0.0";
pub const BACKEND_VERSION: &str = "0.11.1";

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Config {
    /// Absolute mass-error and negative-coordinate budgets; finite in [0, 1).
    pub probability_tolerance: f64,
    /// Recommendation-weighted gain budgets in each player's original units.
    pub incentive_tolerance: [f64; 2],
    /// Stationarity residual budget in the caller's objective units.
    pub dual_tolerance: f64,
    /// Absolute primal/dual gap budget in the caller's objective units.
    pub objective_tolerance: f64,
    /// Finite iteration budget, from zero to `MAX_ITERATIONS` inclusive.
    pub max_iterations: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            probability_tolerance: 1e-8,
            incentive_tolerance: [1e-8; 2],
            dual_tolerance: 1e-8,
            objective_tolerance: 1e-8,
            max_iterations: 200,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Player {
    Row,
    Column,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Status {
    /// Backend Solved and all independent interval bounds meet the config.
    /// Raw weights can still be negative within the probability tolerance.
    ApproximateOptimum,
    /// Backend Solved, but at least one residual bound fails admission.
    ResidualRejected,
    /// Independent finite arithmetic or backend diagnostics could not verify.
    NumericalFailure,
    /// Any backend termination other than Solved; never an accepted CE.
    BackendTerminated,
}

/// Paignia-owned termination identity; Clarabel types do not cross the API.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Termination {
    Unsolved,
    Solved,
    PrimalInfeasible,
    DualInfeasible,
    AlmostSolved,
    AlmostPrimalInfeasible,
    AlmostDualInfeasible,
    MaxIterations,
    MaxTime,
    NumericalError,
    InsufficientProgress,
    CallbackTerminated,
}

/// Finite bounds enclosing real arithmetic on the supplied binary64 inputs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Interval {
    pub lower: f64,
    pub upper: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Incentive {
    pub player: Player,
    pub recommendation: usize,
    pub deviation: usize,
    pub recommendation_mass: Interval,
    pub weighted_gain: Interval,
    /// Defined only when the entire raw recommendation-mass interval is > 0.
    /// No probability tolerance is used as a recommendation cutoff.
    pub conditional_gain: Option<Interval>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PrimalDiagnostics {
    pub mass: Interval,
    pub mass_error_upper: f64,
    pub maximum_negative_weight: f64,
    /// Linear moments of the unchanged raw joint weights, not renormalised.
    pub payoffs: [Interval; 2],
    pub objective: Interval,
    pub maximum_weighted_gain_upper: [f64; 2],
    /// Row recommendations/deviations first, then columns; self pairs omitted.
    pub incentives: Vec<Incentive>,
    /// Numerical admission of mass/nonnegativity and weighted incentives only.
    /// This boolean does not establish objective optimality.
    pub within_tolerance: bool,
}

/// Public native certificate coordinates, independent of any backend type.
#[derive(Clone, Debug, PartialEq)]
pub struct DualVariables {
    /// Free multiplier of the unit-mass equality.
    pub mass: f64,
    /// Nonnegative multipliers of -I x <= 0, in row-major coordinate order.
    pub nonnegativity: Vec<f64>,
    /// Nonnegative multipliers of deviation-gain <= 0, in incentive order.
    pub incentives: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CertificateDiagnostics {
    /// Outward bounds on each independent original-unit stationarity equation.
    pub stationarity: Vec<Interval>,
    pub stationarity_error_upper: f64,
    /// Exact sign test on all finite inequality multipliers, without clipping.
    pub dual_inequalities_nonnegative: bool,
    /// Raw equality multiplier, an approximate dual objective.
    pub objective: Interval,
    /// Dual minus primal objective, retaining its sign and uncertainty.
    pub objective_gap: Interval,
    pub objective_gap_abs_upper: f64,
    /// Includes primal admission, multiplier signs, stationarity and gap.
    pub within_tolerance: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Candidate {
    pub raw_joint_weights: Vec<f64>,
    pub dual: DualVariables,
    pub primal: PrimalDiagnostics,
    pub certificate: CertificateDiagnostics,
    pub backend_objective_consistency_upper: f64,
}

/// Finite backend observations. None explicitly means unavailable/non-finite.
#[derive(Clone, Debug, PartialEq)]
pub struct BackendDiagnostics {
    pub iterations: u32,
    pub solve_seconds: Option<f64>,
    pub primal_residual: Option<f64>,
    pub dual_residual: Option<f64>,
    /// Maximisation objective sign convention, also for the dual observation.
    pub objective: Option<f64>,
    pub dual_objective: Option<f64>,
}

/// Immutable result observation: inspecting a candidate cannot relabel admission.
#[derive(Clone, Debug, PartialEq)]
pub struct ResultRecord {
    dimensions: (usize, usize),
    config: Config,
    termination: Termination,
    status: Status,
    backend: BackendDiagnostics,
    candidate: Option<Candidate>,
    numerical_error: Option<Error>,
}

impl ResultRecord {
    #[must_use]
    pub const fn method(&self) -> &'static str {
        METHOD
    }
    #[must_use]
    pub const fn method_version(&self) -> &'static str {
        VERSION
    }
    #[must_use]
    pub const fn backend_version(&self) -> &'static str {
        BACKEND_VERSION
    }
    #[must_use]
    pub const fn game_class(&self) -> &'static str {
        "two-player-finite-complete-information-bimatrix"
    }
    #[must_use]
    pub const fn dimensions(&self) -> (usize, usize) {
        self.dimensions
    }
    #[must_use]
    pub const fn config(&self) -> Config {
        self.config
    }
    #[must_use]
    pub const fn termination(&self) -> Termination {
        self.termination
    }
    #[must_use]
    pub const fn status(&self) -> Status {
        self.status
    }
    #[must_use]
    pub const fn backend(&self) -> &BackendDiagnostics {
        &self.backend
    }
    #[must_use]
    pub fn candidate(&self) -> Option<&Candidate> {
        self.candidate.as_ref()
    }
    #[must_use]
    pub fn numerical_error(&self) -> Option<&Error> {
        self.numerical_error.as_ref()
    }
    #[must_use]
    pub fn accepted_candidate(&self) -> Option<&Candidate> {
        if self.status == Status::ApproximateOptimum {
            self.candidate.as_ref()
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, PartialEq, Error)]
pub enum Error {
    #[error("empty payoff matrix")]
    EmptyGame,
    #[error("game has {rows} by {columns} actions; limit is {limit} per player")]
    ActionLimit {
        rows: usize,
        columns: usize,
        limit: usize,
    },
    #[error("{input} shape mismatch at row {row:?}: expected {expected}, found {actual}")]
    Shape {
        input: &'static str,
        row: Option<usize>,
        expected: usize,
        actual: usize,
    },
    #[error("non-finite {input} value at flattened coordinate {index}")]
    NonFinite { input: &'static str, index: usize },
    #[error("invalid {name} tolerance {value}")]
    InvalidTolerance { name: &'static str, value: f64 },
    #[error("iteration budget {value} exceeds {maximum}")]
    IterationLimit { value: u32, maximum: u32 },
    #[error("finite interval arithmetic unavailable for {context}")]
    Arithmetic { context: &'static str },
    #[error("backend setup failed: {message}")]
    BackendSetup { message: String },
    #[error("backend reported Solved with missing or invalid diagnostics")]
    BackendDiagnostics,
}

struct Game<'a> {
    row: &'a [Vec<f64>],
    column: &'a [Vec<f64>],
    objective: &'a [Vec<f64>],
    rows: usize,
    columns: usize,
}

impl<'a> Game<'a> {
    fn new(
        row: &'a [Vec<f64>],
        column: &'a [Vec<f64>],
        objective: &'a [Vec<f64>],
    ) -> Result<Self, Error> {
        let rows = row.len();
        let columns = row.first().map_or(0, Vec::len);
        if rows == 0 || columns == 0 {
            return Err(Error::EmptyGame);
        }
        if rows > MAX_ACTIONS || columns > MAX_ACTIONS {
            return Err(Error::ActionLimit {
                rows,
                columns,
                limit: MAX_ACTIONS,
            });
        }
        for (input, table) in [
            ("row payoffs", row),
            ("column payoffs", column),
            ("objective", objective),
        ] {
            if table.len() != rows {
                return Err(Error::Shape {
                    input,
                    row: None,
                    expected: rows,
                    actual: table.len(),
                });
            }
            for (i, values) in table.iter().enumerate() {
                if values.len() != columns {
                    return Err(Error::Shape {
                        input,
                        row: Some(i),
                        expected: columns,
                        actual: values.len(),
                    });
                }
                for (j, value) in values.iter().enumerate() {
                    if !value.is_finite() {
                        return Err(Error::NonFinite {
                            input,
                            index: i * columns + j,
                        });
                    }
                }
            }
        }
        Ok(Self {
            row,
            column,
            objective,
            rows,
            columns,
        })
    }
    fn size(&self) -> usize {
        self.rows * self.columns
    }
    fn incentive_count(&self) -> usize {
        self.rows * (self.rows - 1) + self.columns * (self.columns - 1)
    }
}

fn validate_config(config: Config) -> Result<(), Error> {
    for (name, value) in [
        ("probability", config.probability_tolerance),
        ("row incentive", config.incentive_tolerance[0]),
        ("column incentive", config.incentive_tolerance[1]),
        ("dual", config.dual_tolerance),
        ("objective", config.objective_tolerance),
    ] {
        if !value.is_finite() || value < 0. || (name == "probability" && value >= 1.) {
            return Err(Error::InvalidTolerance { name, value });
        }
    }
    if config.max_iterations > MAX_ITERATIONS {
        return Err(Error::IterationLimit {
            value: config.max_iterations,
            maximum: MAX_ITERATIONS,
        });
    }
    Ok(())
}

fn validate_vector(input: &'static str, values: &[f64], expected: usize) -> Result<(), Error> {
    if values.len() != expected {
        return Err(Error::Shape {
            input,
            row: None,
            expected,
            actual: values.len(),
        });
    }
    for (index, value) in values.iter().enumerate() {
        if !value.is_finite() {
            return Err(Error::NonFinite { input, index });
        }
    }
    Ok(())
}

// One representable neighbour outwards encloses a correctly rounded operation,
// including gradual underflow. An unbounded endpoint causes an explicit error.
fn neighbour(value: f64, upward: bool) -> Result<f64, Error> {
    if !value.is_finite() {
        return Err(Error::Arithmetic {
            context: "non-finite operation",
        });
    }
    let result = if value == 0. {
        if upward {
            f64::from_bits(1)
        } else {
            -f64::from_bits(1)
        }
    } else if (value > 0.) == upward {
        f64::from_bits(value.to_bits() + 1)
    } else {
        f64::from_bits(value.to_bits() - 1)
    };
    if result.is_finite() {
        Ok(result)
    } else {
        Err(Error::Arithmetic {
            context: "outward rounding overflow",
        })
    }
}

impl Interval {
    fn point(value: f64) -> Self {
        Self {
            lower: value,
            upper: value,
        }
    }
    fn is_point(self, value: f64) -> bool {
        self.lower == value && self.upper == value
    }
    fn neg(self) -> Self {
        Self {
            lower: -self.upper,
            upper: -self.lower,
        }
    }
    fn add(self, other: Self) -> Result<Self, Error> {
        if self.is_point(0.) {
            return Ok(other);
        }
        if other.is_point(0.) {
            return Ok(self);
        }
        if self.lower == self.upper && other.is_point(-self.lower) {
            return Ok(Self::point(0.));
        }
        Ok(Self {
            lower: neighbour(self.lower + other.lower, false)?,
            upper: neighbour(self.upper + other.upper, true)?,
        })
    }
    fn sub(self, other: Self) -> Result<Self, Error> {
        self.add(other.neg())
    }
    fn mul(self, other: Self) -> Result<Self, Error> {
        if self.is_point(0.) || other.is_point(0.) {
            return Ok(Self::point(0.));
        }
        if self.is_point(1.) {
            return Ok(other);
        }
        if other.is_point(1.) {
            return Ok(self);
        }
        if self.is_point(-1.) {
            return Ok(other.neg());
        }
        if other.is_point(-1.) {
            return Ok(self.neg());
        }
        Self::enclose_corners([
            self.lower * other.lower,
            self.lower * other.upper,
            self.upper * other.lower,
            self.upper * other.upper,
        ])
    }
    fn div_positive(self, other: Self) -> Result<Self, Error> {
        if other.lower <= 0. {
            return Err(Error::Arithmetic {
                context: "uncertain recommendation mass",
            });
        }
        if self.is_point(0.) {
            return Ok(Self::point(0.));
        }
        if other.is_point(1.) {
            return Ok(self);
        }
        Self::enclose_corners([
            self.lower / other.lower,
            self.lower / other.upper,
            self.upper / other.lower,
            self.upper / other.upper,
        ])
    }
    fn enclose_corners(values: [f64; 4]) -> Result<Self, Error> {
        if values.iter().any(|value| !value.is_finite()) {
            return Err(Error::Arithmetic {
                context: "interval product or quotient",
            });
        }
        let lower = values.iter().copied().fold(values[0], f64::min);
        let upper = values.iter().copied().fold(values[0], f64::max);
        Ok(Self {
            lower: neighbour(lower, false)?,
            upper: neighbour(upper, true)?,
        })
    }
    fn abs_upper(self) -> f64 {
        self.lower.abs().max(self.upper.abs())
    }
}

fn sum(values: impl IntoIterator<Item = f64>) -> Result<Interval, Error> {
    values
        .into_iter()
        .try_fold(Interval::point(0.), |total, value| {
            total.add(Interval::point(value))
        })
}

fn moment(table: &[Vec<f64>], joint: &[f64]) -> Result<Interval, Error> {
    table
        .iter()
        .flatten()
        .zip(joint)
        .try_fold(Interval::point(0.), |total, (&payoff, &weight)| {
            total.add(Interval::point(payoff).mul(Interval::point(weight))?)
        })
}

fn incentive(
    game: &Game<'_>,
    joint: &[f64],
    player: Player,
    recommendation: usize,
    deviation: usize,
) -> Result<Incentive, Error> {
    let mut mass = Interval::point(0.);
    let mut gain = Interval::point(0.);
    let count = if player == Player::Row {
        game.columns
    } else {
        game.rows
    };
    for other in 0..count {
        let (index, followed, changed) = match player {
            Player::Row => (
                recommendation * game.columns + other,
                game.row[recommendation][other],
                game.row[deviation][other],
            ),
            Player::Column => (
                other * game.columns + recommendation,
                game.column[other][recommendation],
                game.column[other][deviation],
            ),
        };
        let weight = Interval::point(joint[index]);
        mass = mass.add(weight)?;
        if !weight.is_point(0.) {
            let difference = Interval::point(changed).sub(Interval::point(followed))?;
            gain = gain.add(difference.mul(weight)?)?;
        }
    }
    let conditional_gain = if mass.lower > 0. {
        Some(gain.div_positive(mass)?)
    } else {
        None
    };
    Ok(Incentive {
        player,
        recommendation,
        deviation,
        recommendation_mass: mass,
        weighted_gain: gain,
        conditional_gain,
    })
}

fn primal(game: &Game<'_>, joint: &[f64], config: Config) -> Result<PrimalDiagnostics, Error> {
    validate_vector("joint weight", joint, game.size())?;
    let mass = sum(joint.iter().copied())?;
    let mass_error_upper = mass.sub(Interval::point(1.))?.abs_upper();
    let maximum_negative_weight = joint.iter().map(|value| -*value).fold(0., f64::max);
    let payoffs = [moment(game.row, joint)?, moment(game.column, joint)?];
    let objective = moment(game.objective, joint)?;
    let mut incentives = Vec::with_capacity(game.incentive_count());
    let mut maximum_weighted_gain_upper = [0_f64; 2];
    for (player_index, player, actions) in [
        (0, Player::Row, game.rows),
        (1, Player::Column, game.columns),
    ] {
        for recommendation in 0..actions {
            for deviation in 0..actions {
                if recommendation == deviation {
                    continue;
                }
                let entry = incentive(game, joint, player, recommendation, deviation)?;
                maximum_weighted_gain_upper[player_index] =
                    maximum_weighted_gain_upper[player_index].max(entry.weighted_gain.upper);
                incentives.push(entry);
            }
        }
    }
    let within_tolerance = mass_error_upper <= config.probability_tolerance
        && maximum_negative_weight <= config.probability_tolerance
        && maximum_weighted_gain_upper
            .iter()
            .zip(config.incentive_tolerance)
            .all(|(gain, tolerance)| *gain <= tolerance);
    Ok(PrimalDiagnostics {
        mass,
        mass_error_upper,
        maximum_negative_weight,
        payoffs,
        objective,
        maximum_weighted_gain_upper,
        incentives,
        within_tolerance,
    })
}

fn stationarity(
    game: &Game<'_>,
    dual: &DualVariables,
    row: usize,
    column: usize,
) -> Result<Interval, Error> {
    let mut residual = Interval::point(dual.mass)
        .sub(Interval::point(game.objective[row][column]))?
        .sub(Interval::point(
            dual.nonnegativity[row * game.columns + column],
        ))?;
    let mut index = 0;
    for recommendation in 0..game.rows {
        for deviation in 0..game.rows {
            if recommendation == deviation {
                continue;
            }
            if recommendation == row && dual.incentives[index] != 0. {
                let gain = Interval::point(game.row[deviation][column])
                    .sub(Interval::point(game.row[row][column]))?;
                residual = residual.add(gain.mul(Interval::point(dual.incentives[index]))?)?;
            }
            index += 1;
        }
    }
    for recommendation in 0..game.columns {
        for deviation in 0..game.columns {
            if recommendation == deviation {
                continue;
            }
            if recommendation == column && dual.incentives[index] != 0. {
                let gain = Interval::point(game.column[row][deviation])
                    .sub(Interval::point(game.column[row][column]))?;
                residual = residual.add(gain.mul(Interval::point(dual.incentives[index]))?)?;
            }
            index += 1;
        }
    }
    Ok(residual)
}

fn dual_diagnostics(
    game: &Game<'_>,
    primal: &PrimalDiagnostics,
    dual: &DualVariables,
    config: Config,
) -> Result<CertificateDiagnostics, Error> {
    validate_vector("dual mass", &[dual.mass], 1)?;
    validate_vector("dual nonnegativity", &dual.nonnegativity, game.size())?;
    validate_vector("dual incentive", &dual.incentives, game.incentive_count())?;
    let dual_inequalities_nonnegative = dual
        .nonnegativity
        .iter()
        .chain(&dual.incentives)
        .all(|value| *value >= 0.);
    let mut stationarity_values = Vec::with_capacity(game.size());
    let mut stationarity_error_upper = 0_f64;
    for row in 0..game.rows {
        for column in 0..game.columns {
            let residual = stationarity(game, dual, row, column)?;
            stationarity_error_upper = stationarity_error_upper.max(residual.abs_upper());
            stationarity_values.push(residual);
        }
    }
    let objective = Interval::point(dual.mass);
    let objective_gap = objective.sub(primal.objective)?;
    let objective_gap_abs_upper = objective_gap.abs_upper();
    let within_tolerance = primal.within_tolerance
        && dual_inequalities_nonnegative
        && stationarity_error_upper <= config.dual_tolerance
        && objective_gap_abs_upper <= config.objective_tolerance;
    Ok(CertificateDiagnostics {
        stationarity: stationarity_values,
        stationarity_error_upper,
        dual_inequalities_nonnegative,
        objective,
        objective_gap,
        objective_gap_abs_upper,
        within_tolerance,
    })
}

/// Evaluate unchanged raw joint weights against the original payoff tables.
/// This reports approximate primal feasibility, not objective optimality.
///
/// # Errors
///
/// Rejects invalid configuration, shape/non-finite inputs and arithmetic whose
/// finite outward enclosure cannot be represented.
pub fn evaluate_candidate(
    row: &[Vec<f64>],
    column: &[Vec<f64>],
    objective: &[Vec<f64>],
    joint: &[f64],
    config: Config,
) -> Result<PrimalDiagnostics, Error> {
    validate_config(config)?;
    primal(&Game::new(row, column, objective)?, joint, config)
}

/// Independently evaluate a native primal/dual certificate in original units.
/// A successful return contains explicit admission booleans; it is not itself
/// evidence that the supplied certificate meets the configured tolerances.
///
/// # Errors
///
/// Returns the candidate evaluator's errors, and rejects malformed/non-finite
/// dual vectors or unrepresentable certificate arithmetic.
pub fn evaluate_certificate(
    row: &[Vec<f64>],
    column: &[Vec<f64>],
    objective: &[Vec<f64>],
    joint: &[f64],
    dual: &DualVariables,
    config: Config,
) -> Result<CertificateDiagnostics, Error> {
    validate_config(config)?;
    let game = Game::new(row, column, objective)?;
    let primal = primal(&game, joint, config)?;
    dual_diagnostics(&game, &primal, dual, config)
}

fn constraint_matrix(game: &Game<'_>) -> Result<CscMatrix<f64>, Error> {
    let size = game.size();
    let mut rows = vec![vec![1.; size]];
    for coordinate in 0..size {
        let mut row = vec![0.; size];
        row[coordinate] = -1.;
        rows.push(row);
    }
    for recommendation in 0..game.rows {
        for deviation in 0..game.rows {
            if recommendation == deviation {
                continue;
            }
            let mut row = vec![0.; size];
            for column in 0..game.columns {
                row[recommendation * game.columns + column] =
                    game.row[deviation][column] - game.row[recommendation][column];
            }
            rows.push(row);
        }
    }
    for recommendation in 0..game.columns {
        for deviation in 0..game.columns {
            if recommendation == deviation {
                continue;
            }
            let mut row = vec![0.; size];
            for player_row in 0..game.rows {
                row[player_row * game.columns + recommendation] =
                    game.column[player_row][deviation] - game.column[player_row][recommendation];
            }
            rows.push(row);
        }
    }
    if rows.iter().flatten().any(|value| !value.is_finite()) {
        return Err(Error::Arithmetic {
            context: "backend deviation coefficient",
        });
    }
    let mut pointers = vec![0];
    let mut indices = Vec::new();
    let mut values = Vec::new();
    for column in 0..size {
        for (index, row) in rows.iter().enumerate() {
            if row[column] != 0. {
                indices.push(index);
                values.push(row[column]);
            }
        }
        pointers.push(values.len());
    }
    let matrix = CscMatrix::new(rows.len(), size, pointers, indices, values);
    matrix.check_format().map_err(|error| Error::BackendSetup {
        message: error.to_string(),
    })?;
    Ok(matrix)
}

fn termination(status: SolverStatus) -> Termination {
    match status {
        SolverStatus::Unsolved => Termination::Unsolved,
        SolverStatus::Solved => Termination::Solved,
        SolverStatus::PrimalInfeasible => Termination::PrimalInfeasible,
        SolverStatus::DualInfeasible => Termination::DualInfeasible,
        SolverStatus::AlmostSolved => Termination::AlmostSolved,
        SolverStatus::AlmostPrimalInfeasible => Termination::AlmostPrimalInfeasible,
        SolverStatus::AlmostDualInfeasible => Termination::AlmostDualInfeasible,
        SolverStatus::MaxIterations => Termination::MaxIterations,
        SolverStatus::MaxTime => Termination::MaxTime,
        SolverStatus::NumericalError => Termination::NumericalError,
        SolverStatus::InsufficientProgress => Termination::InsufficientProgress,
        SolverStatus::CallbackTerminated => Termination::CallbackTerminated,
    }
}

fn backend_diagnostics(solution: &DefaultSolution<f64>) -> BackendDiagnostics {
    let finite = |value: f64| value.is_finite().then_some(value);
    let nonnegative = |value: f64| (value.is_finite() && value >= 0.).then_some(value);
    BackendDiagnostics {
        iterations: solution.iterations,
        solve_seconds: nonnegative(solution.solve_time),
        primal_residual: nonnegative(solution.r_prim),
        dual_residual: nonnegative(solution.r_dual),
        objective: finite(-solution.obj_val),
        dual_objective: finite(-solution.obj_val_dual),
    }
}

fn candidate(
    game: &Game<'_>,
    solution: &DefaultSolution<f64>,
    config: Config,
) -> Result<Candidate, Error> {
    let backend = backend_diagnostics(solution);
    if [
        backend.solve_seconds,
        backend.primal_residual,
        backend.dual_residual,
        backend.objective,
        backend.dual_objective,
    ]
    .iter()
    .any(Option::is_none)
    {
        return Err(Error::BackendDiagnostics);
    }
    let size = game.size();
    validate_vector(
        "backend dual",
        &solution.z,
        1 + size + game.incentive_count(),
    )?;
    let dual = DualVariables {
        mass: solution.z[0],
        nonnegativity: solution.z[1..=size].to_vec(),
        incentives: solution.z[size + 1..].to_vec(),
    };
    let primal = primal(game, &solution.x, config)?;
    let certificate = dual_diagnostics(game, &primal, &dual, config)?;
    let backend_objective_consistency_upper = Interval::point(-solution.obj_val)
        .sub(primal.objective)?
        .abs_upper()
        .max(
            Interval::point(-solution.obj_val_dual)
                .sub(Interval::point(dual.mass))?
                .abs_upper(),
        );
    Ok(Candidate {
        raw_joint_weights: solution.x.clone(),
        dual,
        primal,
        certificate,
        backend_objective_consistency_upper,
    })
}

/// Maximise the explicit linear objective over the two-player CE constraints.
///
/// The supported input is a complete finite bimatrix with one to eight actions
/// per player. Return values preserve approximate raw weights and distinguish
/// backend termination, failed independent admission and an admitted approximate
/// optimum. No failure status proves absence of a CE.
///
/// # Errors
///
/// Rejects invalid inputs/configuration, unrepresentable LP coefficients and
/// backend setup errors. Post-solve numerical failures remain in the result
/// record alongside the backend's original termination.
pub fn solve(
    row: &[Vec<f64>],
    column: &[Vec<f64>],
    objective: &[Vec<f64>],
    config: Config,
) -> Result<ResultRecord, Error> {
    validate_config(config)?;
    let game = Game::new(row, column, objective)?;
    let matrix = constraint_matrix(&game)?;
    let size = game.size();
    let constraints = 1 + size + game.incentive_count();
    let quadratic = CscMatrix::<f64>::zeros((size, size));
    let linear: Vec<f64> = objective.iter().flatten().map(|value| -*value).collect();
    let mut rhs = vec![0.; constraints];
    rhs[0] = 1.;
    let cones = [ZeroConeT(1), NonnegativeConeT(constraints - 1)];
    let settings = DefaultSettingsBuilder::default()
        .verbose(false)
        .max_iter(config.max_iterations)
        .tol_feas(1e-10)
        .tol_gap_abs(1e-10)
        .tol_gap_rel(1e-10)
        .direct_solve_method("qdldl".to_string())
        .build()
        .map_err(|error| Error::BackendSetup {
            message: error.to_string(),
        })?;
    let mut solver = DefaultSolver::new(&quadratic, &linear, &matrix, &rhs, &cones, settings)
        .map_err(|error| Error::BackendSetup {
            message: error.to_string(),
        })?;
    solver.solve();
    let solution = solver.solution;
    let mut result = ResultRecord {
        dimensions: (game.rows, game.columns),
        config,
        termination: termination(solution.status),
        status: Status::BackendTerminated,
        backend: backend_diagnostics(&solution),
        candidate: None,
        numerical_error: None,
    };
    if solution.status == SolverStatus::Solved {
        match candidate(&game, &solution, config) {
            Ok(candidate) => {
                result.status = if candidate.certificate.within_tolerance
                    && candidate.backend_objective_consistency_upper <= config.objective_tolerance
                {
                    Status::ApproximateOptimum
                } else {
                    Status::ResidualRejected
                };
                result.candidate = Some(candidate);
            }
            Err(error) => {
                result.status = Status::NumericalFailure;
                result.numerical_error = Some(error);
            }
        }
    }
    Ok(result)
}
