//! Bounded binary64 support enumeration for two-player bimatrix games.
//!
//! Equal-cardinality supports are searched in deterministic order. Singular or
//! numerically unresolved systems are reported, not replaced by uniform play.
//! Every returned witness is checked against every pure deviation. Completing
//! this bounded search never certifies a complete equilibrium catalogue.

use thiserror::Error;

/// Per-player bound for the native bimatrix API, including candidate evaluation.
pub const MAX_ACTIONS: usize = 8;
/// Version of the numerical equations, ordering, admission and status policy.
pub const SUPPORT_ENUMERATION_VERSION: &str = "1.0.0";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Algorithm {
    /// Enumerate nonsingular equal-cardinality supports with finite budgets.
    SupportEnumeration,
    /// Reserved request identity. This algorithm is explicitly unsupported.
    LemkeHowson,
}

impl Algorithm {
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::SupportEnumeration => "paignia/equal-support-enumeration",
            Self::LemkeHowson => "lemke-howson",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Config {
    /// Absolute unit-mass error allowed for each unchanged probability vector.
    /// Finite, nonnegative and strictly less than one; never a clipping cutoff.
    pub probability_tolerance: f64,
    /// Maximum permitted unilateral gain for each player, in original payoff
    /// units. Finite and nonnegative. Positive admitted gains remain visible.
    pub equilibrium_tolerance: f64,
    /// Absolute pivot threshold and maximum equation residual after each
    /// indifference row is scaled to maximum absolute coefficient one.
    /// Finite, nonnegative and strictly less than one.
    pub pivot_tolerance: f64,
    /// Positive count of support pairs that may be visited. The default covers
    /// all 12,869 equal-cardinality pairs at the eight-by-eight action limit.
    pub max_support_pairs: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            probability_tolerance: 1e-12,
            equilibrium_tolerance: 1e-10,
            pivot_tolerance: 1e-12,
            max_support_pairs: 12_869,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Player {
    Row,
    Column,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GameClass {
    TwoPlayerFiniteCompleteInformationBimatrix,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SolutionConcept {
    MixedNash,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Selection {
    AllVerifiedInCardinalityThenLexicographicSupportOrder,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Status {
    /// All declared support pairs were processed without unresolved systems,
    /// and at least one tolerance-admitted witness was returned.
    CompletedWithinScope,
    /// At least one singular, near-singular or residual-failing support remains
    /// unresolved. Any returned witnesses still passed their own gain checks.
    UnresolvedSupports,
    /// Search stopped at the explicit support-pair budget.
    BudgetExhausted,
    /// No candidate passed the declared numerical tests. This is not a proof
    /// that an equilibrium does not exist.
    NoVerifiedWitness,
    UnsupportedAlgorithm,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SearchCompleteness {
    /// Accounting of visited pairs only, including unresolved pairs. This
    /// variant does not assert mathematical equilibrium completeness.
    AllDeclaredSupportPairsVisited,
    BudgetLimited,
    NotRun,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CatalogueCompleteness {
    /// Finite binary64, tolerance-admitted witnesses only. Neither all isolated
    /// equilibria nor a continuum is certified as exhaustively represented.
    FiniteVerifiedWitnessesOnly,
    NotClaimed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Scope {
    pub row_actions: usize,
    pub column_actions: usize,
    pub maximum_support_cardinality: usize,
    /// This implementation does not solve unequal-cardinality support pairs.
    pub equal_cardinality_only: bool,
    /// Always false in this algorithm version; no singular polytope solver.
    pub singular_systems_supported: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Diagnostics {
    /// Unmodified bilinear values p^T A q and p^T B q, in row/column order.
    pub expected_payoffs: [f64; 2],
    /// Positive parts of the maximum probability-weighted pure deviation
    /// differences, in row/column order. For unit-mass vectors these equal
    /// best-response payoff minus current payoff. No epsilon is subtracted.
    pub best_response_gains: [f64; 2],
    /// Maximum of the two reported unilateral gains, not their average.
    pub max_gain: f64,
    /// Sum of the two reported unilateral gains (often called NashConv).
    pub sum_gain: f64,
    /// Absolute distance of each supplied probability mass from one.
    pub probability_mass_errors: [f64; 2],
    /// Both gains are at most the declared equilibrium tolerance. A true value
    /// is numerical admission, not an exact-arithmetic equilibrium certificate.
    pub within_tolerance: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Candidate {
    pub row_strategy: Vec<f64>,
    pub column_strategy: Vec<f64>,
    /// Strictly positive coordinates, with zero-probability actions omitted.
    pub row_support: Vec<usize>,
    pub column_support: Vec<usize>,
    pub diagnostics: Diagnostics,
    /// Maximum residual of the two normalized indifference systems; absent
    /// when an externally supplied candidate is evaluated without solving.
    pub support_equation_residual: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResultRecord {
    pub algorithm: Algorithm,
    /// The implemented version is present only for support enumeration.
    pub algorithm_version: Option<&'static str>,
    pub game_class: GameClass,
    pub solution_concept: SolutionConcept,
    pub status: Status,
    pub config: Config,
    pub selection: Selection,
    pub scope: Option<Scope>,
    pub support_search: SearchCompleteness,
    pub catalogue_completeness: CatalogueCompleteness,
    pub visited_support_pairs: usize,
    pub total_support_pairs: usize,
    /// Includes singular, ill-conditioned and failed residual/mass checks.
    /// A rank decision at a floating-point threshold is not a degeneracy proof.
    pub unresolved_support_pairs: usize,
    /// Unique support-system solutions with a nonpositive coordinate. Boundary
    /// points belong to smaller supports and are not duplicated here.
    pub non_positive_support_pairs: usize,
    pub rejected_best_response_support_pairs: usize,
    pub candidates: Vec<Candidate>,
}

#[derive(Clone, Debug, PartialEq, Error)]
pub enum Error {
    #[error("invalid {field} tolerance {value}")]
    InvalidTolerance { field: &'static str, value: f64 },
    #[error("support-pair budget must be positive")]
    ZeroSupportBudget,
    #[error("both payoff matrices must have at least one row and column")]
    EmptyGame,
    #[error("{player:?} payoff row {row} has {actual} columns; expected {expected}")]
    InvalidShape {
        player: Player,
        row: usize,
        expected: usize,
        actual: usize,
    },
    #[error("payoff matrices have different row counts: {row_rows} and {column_rows}")]
    RowCountMismatch { row_rows: usize, column_rows: usize },
    #[error("{rows} by {columns} actions exceed the supported limit {limit}")]
    ActionLimit {
        rows: usize,
        columns: usize,
        limit: usize,
    },
    #[error("non-finite {player:?} payoff at ({row}, {column})")]
    NonFinitePayoff {
        player: Player,
        row: usize,
        column: usize,
    },
    #[error("{player:?} strategy has {actual} probabilities; expected {expected}")]
    StrategyDimension {
        player: Player,
        expected: usize,
        actual: usize,
    },
    #[error("invalid {player:?} probability at {index}: {value}")]
    InvalidProbability {
        player: Player,
        index: usize,
        value: f64,
    },
    #[error("{player:?} probability mass {mass}, with absolute error {mass_error}, exceeds the configured unit-mass tolerance")]
    InvalidProbabilityMass {
        player: Player,
        mass: f64,
        mass_error: f64,
    },
    #[error("unrepresentable arithmetic while computing {operation}")]
    ArithmeticFailure { operation: &'static str },
}

/// Search finite two-player complete-information bimatrix games with one to
/// eight actions per player. `A[i][j]` is the row player's payoff and `B[i][j]`
/// is the column player's payoff; every cell must be finite and explicit.
/// Array coordinates are local action indices, not cross-system semantic IDs.
///
/// This method solves two indifference systems for every equal-size support
/// pair, then checks positive support probabilities, mass, linear residuals
/// and every pure best response on the original payoffs. No strategy is
/// clipped or normalized. Singular systems and unequal support sizes are not
/// solved. Returned candidates are approximate witnesses at the configured
/// tolerance; no result status certifies a complete Nash catalogue.
///
/// `LemkeHowson` returns `UnsupportedAlgorithm` after input validation, without
/// calling support enumeration or substituting another algorithm.
///
/// # Errors
/// Rejects invalid configuration, malformed/non-finite payoff tables, excessive
/// dimensions, and unrepresentable arithmetic (including nonzero products or
/// quotients rounded to zero). An arithmetic error aborts the call; it is not a
/// claim that a game has no equilibrium.
///
/// # Examples
///
/// ```
/// use paignia_solve::{solve_mixed_nash, MixedNashAlgorithm, MixedNashConfig};
/// let a = vec![vec![1.0, -1.0], vec![-1.0, 1.0]];
/// let b = vec![vec![-1.0, 1.0], vec![1.0, -1.0]];
/// let result = solve_mixed_nash(&a, &b, MixedNashAlgorithm::SupportEnumeration,
///     MixedNashConfig::default()).unwrap();
/// assert_eq!(result.candidates[0].row_strategy, vec![0.5, 0.5]);
/// assert_eq!(result.candidates[0].diagnostics.max_gain, 0.0);
/// ```
pub fn solve(
    row_payoffs: &[Vec<f64>],
    column_payoffs: &[Vec<f64>],
    algorithm: Algorithm,
    config: Config,
) -> Result<ResultRecord, Error> {
    validate_config(config)?;
    let game = ValidatedGame::new(row_payoffs, column_payoffs)?;
    let mut result = ResultRecord {
        algorithm,
        algorithm_version: None,
        game_class: GameClass::TwoPlayerFiniteCompleteInformationBimatrix,
        solution_concept: SolutionConcept::MixedNash,
        status: Status::UnsupportedAlgorithm,
        config,
        selection: Selection::AllVerifiedInCardinalityThenLexicographicSupportOrder,
        scope: None,
        support_search: SearchCompleteness::NotRun,
        catalogue_completeness: CatalogueCompleteness::NotClaimed,
        visited_support_pairs: 0,
        total_support_pairs: 0,
        unresolved_support_pairs: 0,
        non_positive_support_pairs: 0,
        rejected_best_response_support_pairs: 0,
        candidates: Vec::new(),
    };
    if algorithm == Algorithm::LemkeHowson {
        return Ok(result);
    }
    let maximum_support = game.rows.min(game.columns);
    let supports: Vec<_> = (1..=maximum_support)
        .map(|size| {
            (
                combinations(game.rows, size),
                combinations(game.columns, size),
            )
        })
        .collect();
    result.algorithm_version = Some(SUPPORT_ENUMERATION_VERSION);
    result.scope = Some(Scope {
        row_actions: game.rows,
        column_actions: game.columns,
        maximum_support_cardinality: maximum_support,
        equal_cardinality_only: true,
        singular_systems_supported: false,
    });
    result.catalogue_completeness = CatalogueCompleteness::FiniteVerifiedWitnessesOnly;
    result.total_support_pairs = supports
        .iter()
        .map(|(rows, columns)| rows.len() * columns.len())
        .sum();
    for (row_supports, column_supports) in supports {
        for rows in &row_supports {
            for columns in &column_supports {
                if result.visited_support_pairs == config.max_support_pairs {
                    result.status = Status::BudgetExhausted;
                    result.support_search = SearchCompleteness::BudgetLimited;
                    return Ok(result);
                }
                result.visited_support_pairs += 1;
                match solve_support(&game, rows, columns, config)? {
                    SupportOutcome::Verified(candidate) => result.candidates.push(candidate),
                    SupportOutcome::Unresolved => result.unresolved_support_pairs += 1,
                    SupportOutcome::NonPositive => result.non_positive_support_pairs += 1,
                    SupportOutcome::Rejected => result.rejected_best_response_support_pairs += 1,
                }
            }
        }
    }
    result.support_search = SearchCompleteness::AllDeclaredSupportPairsVisited;
    result.status = if result.unresolved_support_pairs > 0 {
        Status::UnresolvedSupports
    } else if result.candidates.is_empty() {
        Status::NoVerifiedWitness
    } else {
        Status::CompletedWithinScope
    };
    Ok(result)
}

/// Evaluate a supplied independent mixed-strategy profile using every pure
/// unilateral deviation. This does not enumerate or select an equilibrium.
/// The probabilities are preserved exactly as supplied; admitted mass error
/// and positive gains remain visible in the returned diagnostics.
///
/// # Errors
/// Returns the same configuration, payoff-domain and arithmetic errors as
/// [`solve`], plus errors for invalid probability values, dimensions or mass.
pub fn evaluate_candidate(
    row_payoffs: &[Vec<f64>],
    column_payoffs: &[Vec<f64>],
    row_strategy: &[f64],
    column_strategy: &[f64],
    config: Config,
) -> Result<Candidate, Error> {
    validate_config(config)?;
    let game = ValidatedGame::new(row_payoffs, column_payoffs)?;
    evaluate_validated(&game, row_strategy, column_strategy, config)
}

struct ValidatedGame<'a> {
    a: &'a [Vec<f64>],
    b: &'a [Vec<f64>],
    rows: usize,
    columns: usize,
}

impl<'a> ValidatedGame<'a> {
    fn new(a: &'a [Vec<f64>], b: &'a [Vec<f64>]) -> Result<Self, Error> {
        if a.is_empty() || b.is_empty() || a[0].is_empty() || b[0].is_empty() {
            return Err(Error::EmptyGame);
        }
        let rows = a.len();
        let columns = a[0].len();
        if b.len() != rows {
            return Err(Error::RowCountMismatch {
                row_rows: rows,
                column_rows: b.len(),
            });
        }
        if rows > MAX_ACTIONS || columns > MAX_ACTIONS {
            return Err(Error::ActionLimit {
                rows,
                columns,
                limit: MAX_ACTIONS,
            });
        }
        for (player, matrix) in [(Player::Row, a), (Player::Column, b)] {
            for (row, values) in matrix.iter().enumerate() {
                if values.len() != columns {
                    return Err(Error::InvalidShape {
                        player,
                        row,
                        expected: columns,
                        actual: values.len(),
                    });
                }
                for (column, value) in values.iter().enumerate() {
                    if !value.is_finite() {
                        return Err(Error::NonFinitePayoff {
                            player,
                            row,
                            column,
                        });
                    }
                }
            }
        }
        Ok(Self {
            a,
            b,
            rows,
            columns,
        })
    }
}

fn validate_config(config: Config) -> Result<(), Error> {
    for (field, value, bounded) in [
        ("probability", config.probability_tolerance, true),
        ("equilibrium", config.equilibrium_tolerance, false),
        ("pivot", config.pivot_tolerance, true),
    ] {
        if !value.is_finite() || value < 0.0 || (bounded && value >= 1.0) {
            return Err(Error::InvalidTolerance { field, value });
        }
    }
    if config.max_support_pairs == 0 {
        return Err(Error::ZeroSupportBudget);
    }
    Ok(())
}

fn validate_strategy(
    p: &[f64],
    expected: usize,
    player: Player,
    tolerance: f64,
) -> Result<f64, Error> {
    if p.len() != expected {
        return Err(Error::StrategyDimension {
            player,
            expected,
            actual: p.len(),
        });
    }
    let mut sum = ExpansionSum::default();
    for (index, &value) in p.iter().enumerate() {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err(Error::InvalidProbability {
                player,
                index,
                value,
            });
        }
        sum.add(value, "probability mass")?;
    }
    let mass = sum.total("probability mass")?;
    // Form the residual before rounding the total to binary64. Otherwise a
    // genuine mass error smaller than half an ulp at one would disappear.
    sum.add(-1.0, "probability mass residual")?;
    let error = sum.total("probability mass residual")?.abs();
    if error > tolerance {
        return Err(Error::InvalidProbabilityMass {
            player,
            mass,
            mass_error: error,
        });
    }
    Ok(error)
}

fn finite(value: f64, operation: &'static str) -> Result<f64, Error> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(Error::ArithmeticFailure { operation })
    }
}

// Retain low addition parts until later cancellation has been resolved. Each
// magnitude-ordered pair uses Fast-Two-Sum: high = x+y, low = y-(high-x),
// with |x| >= |y|. See Shewchuk (1997), sections 2.3–2.4, cited in the
// validation document. Checked fused-product expansions also preserve the
// multiplication remainders used in payoffs, gains and dot-product residuals.
// Gaussian systems and final diagnostic rounding remain binary64.
#[derive(Default)]
struct ExpansionSum {
    partials: Vec<f64>,
}

impl ExpansionSum {
    fn add_product(&mut self, a: f64, b: f64, operation: &'static str) -> Result<(), Error> {
        for term in exact_product(a, b, operation)? {
            self.add(term, operation)?;
        }
        Ok(())
    }

    fn add(&mut self, value: f64, operation: &'static str) -> Result<(), Error> {
        let mut x = finite(value, operation)?;
        let mut retained = 0;
        for read in 0..self.partials.len() {
            let mut y = self.partials[read];
            if x.abs() < y.abs() {
                std::mem::swap(&mut x, &mut y);
            }
            let high = finite(x + y, operation)?;
            let low = finite(y - finite(high - x, operation)?, operation)?;
            if low != 0.0 {
                self.partials[retained] = low;
                retained += 1;
            }
            x = high;
        }
        self.partials.truncate(retained);
        if x != 0.0 {
            self.partials.push(x);
        }
        Ok(())
    }

    fn total(&self, operation: &'static str) -> Result<f64, Error> {
        self.partials
            .iter()
            .rev()
            .try_fold(0.0, |sum, &part| finite(sum + part, operation))
    }
}

// For a finite nonzero binary64 value, return the exponent of its least
// significant set bit, including subnormals. The checked product below can
// therefore reject an exact low part below2^-1074 before FMA rounds it away.
fn least_bit_exponent(value: f64) -> i32 {
    let bits = value.to_bits();
    let encoded = i32::try_from((bits >> 52) & 0x7ff).expect("eleven-bit exponent fits i32");
    let fraction = bits & ((1_u64 << 52) - 1);
    let (significand, exponent) = if encoded == 0 {
        (fraction, -1074)
    } else {
        (fraction | (1_u64 << 52), encoded - 1075)
    };
    exponent + i32::try_from(significand.trailing_zeros()).expect("bit count fits i32")
}

fn exact_product(a: f64, b: f64, operation: &'static str) -> Result<[f64; 2], Error> {
    if a == 0.0 || b == 0.0 {
        return Ok([0.0, 0.0]);
    }
    if least_bit_exponent(a) + least_bit_exponent(b) < -1074 {
        return Err(Error::ArithmeticFailure { operation });
    }
    let high = product(a, b, operation)?;
    let low = finite(a.mul_add(b, -high), operation)?;
    Ok([high, low])
}

fn product(a: f64, b: f64, operation: &'static str) -> Result<f64, Error> {
    let result = finite(a * b, operation)?;
    if result == 0.0 && a != 0.0 && b != 0.0 {
        return Err(Error::ArithmeticFailure { operation });
    }
    Ok(result)
}

fn quotient(a: f64, b: f64, operation: &'static str) -> Result<f64, Error> {
    let result = finite(a / b, operation)?;
    if result == 0.0 && a != 0.0 {
        return Err(Error::ArithmeticFailure { operation });
    }
    Ok(result)
}

fn evaluate_validated(
    game: &ValidatedGame<'_>,
    p: &[f64],
    q: &[f64],
    config: Config,
) -> Result<Candidate, Error> {
    let masses = [
        validate_strategy(p, game.rows, Player::Row, config.probability_tolerance)?,
        validate_strategy(
            q,
            game.columns,
            Player::Column,
            config.probability_tolerance,
        )?,
    ];
    let mut weights = vec![vec![[0.0, 0.0]; game.columns]; game.rows];
    let mut utility_sums = [ExpansionSum::default(), ExpansionSum::default()];
    for (i, &row_probability) in p.iter().enumerate() {
        for (j, &column_probability) in q.iter().enumerate() {
            let weight = exact_product(row_probability, column_probability, "joint probability")?;
            weights[i][j] = weight;
            for part in weight {
                for (sum, payoff) in utility_sums.iter_mut().zip([game.a[i][j], game.b[i][j]]) {
                    sum.add_product(part, payoff, "expected payoff sum")?;
                }
            }
        }
    }
    let gains = [
        maximum_gain(game, &weights, Player::Row)?,
        maximum_gain(game, &weights, Player::Column)?,
    ];
    Ok(Candidate {
        row_strategy: p.to_vec(),
        column_strategy: q.to_vec(),
        row_support: positive_indices(p),
        column_support: positive_indices(q),
        diagnostics: Diagnostics {
            expected_payoffs: [
                utility_sums[0].total("expected payoff sum")?,
                utility_sums[1].total("expected payoff sum")?,
            ],
            best_response_gains: gains,
            max_gain: gains[0].max(gains[1]),
            sum_gain: finite(gains[0] + gains[1], "sum of unilateral gains")?,
            probability_mass_errors: masses,
            within_tolerance: gains
                .iter()
                .all(|&gain| gain <= config.equilibrium_tolerance),
        },
        support_equation_residual: None,
    })
}

fn maximum_gain(
    game: &ValidatedGame<'_>,
    weights: &[Vec<[f64; 2]>],
    player: Player,
) -> Result<f64, Error> {
    let actions = match player {
        Player::Row => game.rows,
        Player::Column => game.columns,
    };
    let mut maximum: f64 = 0.0;
    for action in 0..actions {
        let mut gain = ExpansionSum::default();
        for (i, row) in weights.iter().enumerate() {
            for (j, weight) in row.iter().enumerate() {
                let (deviation, current) = match player {
                    Player::Row => (game.a[action][j], game.a[i][j]),
                    Player::Column => (game.b[i][action], game.b[i][j]),
                };
                // Do not round deviation-current before multiplying: H-(-1)
                // can lose1 even when the final gain is exactly one half.
                for &part in weight {
                    gain.add_product(part, deviation, "pure deviation sum")?;
                    gain.add_product(-part, current, "pure deviation sum")?;
                }
            }
        }
        maximum = maximum.max(gain.total("pure deviation sum")?);
    }
    Ok(maximum)
}

fn positive_indices(p: &[f64]) -> Vec<usize> {
    p.iter()
        .enumerate()
        .filter_map(|(i, &value)| (value > 0.0).then_some(i))
        .collect()
}

fn combinations(actions: usize, size: usize) -> Vec<Vec<usize>> {
    fn append(
        actions: usize,
        remaining: usize,
        next: usize,
        current: &mut Vec<usize>,
        result: &mut Vec<Vec<usize>>,
    ) {
        if remaining == 0 {
            result.push(current.clone());
            return;
        }
        for action in next..=actions - remaining {
            current.push(action);
            append(actions, remaining - 1, action + 1, current, result);
            current.pop();
        }
    }
    let mut result = Vec::new();
    append(actions, size, 0, &mut Vec::new(), &mut result);
    result
}

enum SupportOutcome {
    Verified(Candidate),
    Unresolved,
    NonPositive,
    Rejected,
}

fn solve_support(
    game: &ValidatedGame<'_>,
    rows: &[usize],
    columns: &[usize],
    config: Config,
) -> Result<SupportOutcome, Error> {
    let row_system = indifference_system(game, rows, columns, Player::Row)?;
    let column_system = indifference_system(game, rows, columns, Player::Column)?;
    let Some((p, row_residual)) = linear_solution(&row_system, config.pivot_tolerance)? else {
        return Ok(SupportOutcome::Unresolved);
    };
    let Some((q, column_residual)) = linear_solution(&column_system, config.pivot_tolerance)?
    else {
        return Ok(SupportOutcome::Unresolved);
    };
    if p.iter().chain(&q).any(|&value| value <= 0.0) {
        return Ok(SupportOutcome::NonPositive);
    }
    let mut full_p = vec![0.0; game.rows];
    let mut full_q = vec![0.0; game.columns];
    for (&index, probability) in rows.iter().zip(p) {
        full_p[index] = probability;
    }
    for (&index, probability) in columns.iter().zip(q) {
        full_q[index] = probability;
    }
    let mut candidate = match evaluate_validated(game, &full_p, &full_q, config) {
        Ok(candidate) => candidate,
        Err(Error::InvalidProbability { .. } | Error::InvalidProbabilityMass { .. }) => {
            return Ok(SupportOutcome::Unresolved)
        }
        Err(error) => return Err(error),
    };
    candidate.support_equation_residual = Some(row_residual.max(column_residual));
    if candidate.diagnostics.within_tolerance {
        Ok(SupportOutcome::Verified(candidate))
    } else {
        Ok(SupportOutcome::Rejected)
    }
}

// A p strategy makes supported B columns indifferent; a q strategy makes
// supported A rows indifferent. The final equation is always unit mass.
fn indifference_system(
    game: &ValidatedGame<'_>,
    rows: &[usize],
    columns: &[usize],
    player: Player,
) -> Result<Vec<Vec<f64>>, Error> {
    let size = rows.len();
    let mut system = Vec::with_capacity(size);
    for other in 1..size {
        let mut equation = Vec::with_capacity(size);
        for index in 0..size {
            let difference = match player {
                Player::Row => {
                    game.b[rows[index]][columns[other]] - game.b[rows[index]][columns[0]]
                }
                Player::Column => {
                    game.a[rows[other]][columns[index]] - game.a[rows[0]][columns[index]]
                }
            };
            equation.push(finite(difference, "support indifference difference")?);
        }
        let scale = equation.iter().map(|v| v.abs()).fold(0.0, f64::max);
        if scale > 0.0 {
            for coefficient in &mut equation {
                *coefficient = quotient(*coefficient, scale, "indifference row scaling")?;
            }
        }
        system.push(equation);
    }
    system.push(vec![1.0; size]);
    Ok(system)
}

// At most eight variables, no general linear-algebra backend or external solve.
// Partial pivoting is applied to already row-scaled coefficients. A failed
// pivot or original-system residual is unresolved, never a fabricated solution.
fn linear_solution(system: &[Vec<f64>], tolerance: f64) -> Result<Option<(Vec<f64>, f64)>, Error> {
    let size = system.len();
    let mut a = system.to_vec();
    let mut rhs = vec![0.0; size];
    rhs[size - 1] = 1.0;
    for column in 0..size {
        let mut pivot = column;
        for row in column + 1..size {
            if a[row][column].abs() > a[pivot][column].abs() {
                pivot = row;
            }
        }
        if a[pivot][column].abs() <= tolerance {
            return Ok(None);
        }
        a.swap(column, pivot);
        rhs.swap(column, pivot);
        for row in column + 1..size {
            let factor = quotient(a[row][column], a[column][column], "elimination multiplier")?;
            a[row][column] = 0.0;
            for j in column + 1..size {
                a[row][j] = finite(
                    a[row][j] - product(factor, a[column][j], "elimination product")?,
                    "elimination difference",
                )?;
            }
            rhs[row] = finite(
                rhs[row] - product(factor, rhs[column], "right-hand elimination product")?,
                "right-hand elimination difference",
            )?;
        }
    }
    let mut solution = vec![0.0; size];
    for row in (0..size).rev() {
        let mut sum = ExpansionSum::default();
        sum.add(rhs[row], "back substitution sum")?;
        for (j, &known) in solution.iter().enumerate().skip(row + 1) {
            sum.add_product(-a[row][j], known, "back substitution sum")?;
        }
        solution[row] = quotient(
            sum.total("back substitution sum")?,
            a[row][row],
            "back substitution quotient",
        )?;
    }
    let mut residual: f64 = 0.0;
    for (row, equation) in system.iter().enumerate() {
        let mut sum = ExpansionSum::default();
        sum.add(
            if row + 1 == size { -1.0 } else { 0.0 },
            "support residual sum",
        )?;
        for (&coefficient, &probability) in equation.iter().zip(&solution) {
            sum.add_product(coefficient, probability, "support residual sum")?;
        }
        residual = residual.max(sum.total("support residual sum")?.abs());
    }
    if residual > tolerance {
        return Ok(None);
    }
    Ok(Some((solution, residual)))
}
