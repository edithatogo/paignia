//! Bounded two-player support enumeration. The first commit is a failing test target.

use thiserror::Error;

pub const MAX_ACTIONS: usize = 8;
pub const SUPPORT_ENUMERATION_VERSION: &str = "1.0.0";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Algorithm {
    SupportEnumeration,
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
    pub probability_tolerance: f64,
    pub equilibrium_tolerance: f64,
    pub pivot_tolerance: f64,
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
    CompletedWithinScope,
    UnresolvedSupports,
    BudgetExhausted,
    NoVerifiedWitness,
    UnsupportedAlgorithm,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SearchCompleteness {
    AllDeclaredSupportPairsVisited,
    BudgetLimited,
    NotRun,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CatalogueCompleteness {
    FiniteVerifiedWitnessesOnly,
    NotClaimed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Scope {
    pub row_actions: usize,
    pub column_actions: usize,
    pub maximum_support_cardinality: usize,
    pub equal_cardinality_only: bool,
    pub singular_systems_supported: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Diagnostics {
    pub expected_payoffs: [f64; 2],
    pub best_response_gains: [f64; 2],
    pub max_gain: f64,
    pub sum_gain: f64,
    pub probability_mass_errors: [f64; 2],
    pub within_tolerance: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Candidate {
    pub row_strategy: Vec<f64>,
    pub column_strategy: Vec<f64>,
    pub row_support: Vec<usize>,
    pub column_support: Vec<usize>,
    pub diagnostics: Diagnostics,
    pub support_equation_residual: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResultRecord {
    pub algorithm: Algorithm,
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
    pub unresolved_support_pairs: usize,
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
    #[error("{player:?} probability mass {mass} exceeds the configured unit-mass tolerance")]
    InvalidProbabilityMass { player: Player, mass: f64 },
    #[error("unrepresentable arithmetic while computing {operation}")]
    ArithmeticFailure { operation: &'static str },
    #[error("test-first implementation is absent")]
    NotImplemented,
}

/// # Errors
/// Returns an explicit test-first failure until the numerical implementation exists.
pub fn solve(
    _row_payoffs: &[Vec<f64>],
    _column_payoffs: &[Vec<f64>],
    _algorithm: Algorithm,
    _config: Config,
) -> Result<ResultRecord, Error> {
    Err(Error::NotImplemented)
}

/// # Errors
/// Returns an explicit test-first failure until the numerical implementation exists.
pub fn evaluate_candidate(
    _row_payoffs: &[Vec<f64>],
    _column_payoffs: &[Vec<f64>],
    _row_strategy: &[f64],
    _column_strategy: &[f64],
    _config: Config,
) -> Result<Candidate, Error> {
    Err(Error::NotImplemented)
}
