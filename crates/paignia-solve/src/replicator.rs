//! Explicit, nonprojecting steps for continuous replicator population dynamics.
//!
//! These numerical steps are independent of finite graph solving, simulation
//! time, event scheduling and equilibrium claims.

use thiserror::Error;

pub const RK4_VERSION: &str = "1.0.0";
pub const EULER_VERSION: &str = "1.0.0";

/// Explicit numerical admission budget, not a local truncation-error bound.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Config {
    /// Maximum absolute difference between population mass and one.
    /// Must be finite, nonnegative and less than one. No normalization occurs.
    pub mass_tolerance: f64,
}

/// The actual numerical method used; Euler is never reported as RK4.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Method {
    ClassicalRk4,
    ForwardEuler,
}

impl Method {
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::ClassicalRk4 => "paignia/replicator-classical-rk4",
            Self::ForwardEuler => "paignia/replicator-forward-euler",
        }
    }
}

/// Location of an input, intermediate state, callback or arithmetic failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Stage {
    Initial,
    First,
    Second,
    Third,
    Fourth,
    Output,
}

/// One accepted population update with method-specific provenance.
#[derive(Clone, Debug, PartialEq)]
pub struct Step {
    pub method: Method,
    pub method_version: &'static str,
    pub start_time: f64,
    pub end_time: f64,
    pub step_size: f64,
    pub mass_tolerance: f64,
    pub population: Vec<f64>,
    /// Actual absolute mass error of the returned population.
    pub mass_error: f64,
    pub fitness_evaluations: u8,
}

/// A rejected step leaves the caller's input population untouched.
#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub enum StepError {
    #[error("mass tolerance must be finite and in [0, 1)")]
    InvalidMassTolerance,
    #[error("population needs at least two components, received {dimension}")]
    InvalidDimension { dimension: usize },
    #[error("start time must be finite")]
    InvalidTime,
    #[error("step size must be finite and nonnegative")]
    InvalidStep,
    #[error("positive step does not have finite, strictly ordered stage times")]
    UnrepresentableStageTime,
    #[error("nonfinite population component {index} at {stage:?}")]
    NonFinitePopulation { stage: Stage, index: usize },
    #[error("population component {index} leaves [0, 1] at {stage:?}")]
    PopulationOutOfBounds { stage: Stage, index: usize },
    #[error("population mass is outside the declared tolerance at {stage:?}")]
    PopulationMass { stage: Stage },
    #[error("fitness dimension at {stage:?}: expected {expected}, received {actual}")]
    FitnessDimension {
        stage: Stage,
        expected: usize,
        actual: usize,
    },
    #[error("nonfinite fitness component {index} at {stage:?}")]
    NonFiniteFitness { stage: Stage, index: usize },
    #[error("fitness callback failed at {stage:?}: {message}")]
    FitnessCallback { stage: Stage, message: String },
    #[error("nonfinite arithmetic for {operation} at {stage:?}")]
    NonFiniteArithmetic {
        stage: Stage,
        operation: &'static str,
    },
}

/// Advance one step using the classical four-stage Runge–Kutta tableau.
///
/// The callback supplies fitness at each actual stage time and population.
/// It must describe one smooth segment, including the segment's one-sided
/// endpoint extension. Split discontinuities externally; this is not a
/// scheduler. Zero steps preserve input bits and make no callback calls.
///
/// # Errors
///
/// Returns [`StepError`] for invalid parameters, callback failures, nonfinite
/// arithmetic or any intermediate/final population outside the admitted
/// simplex. No state is clipped, normalized or silently retried.
pub fn rk4_step<F>(
    population: &[f64],
    time: f64,
    step: f64,
    config: Config,
    _fitness: F,
) -> Result<Step, StepError>
where
    F: FnMut(f64, &[f64]) -> Result<Vec<f64>, String>,
{
    // Explicit tests-first identity prototype; no numerical implementation yet.
    Ok(Step {
        method: Method::ClassicalRk4,
        method_version: RK4_VERSION,
        start_time: time,
        end_time: time + step,
        step_size: step,
        mass_tolerance: config.mass_tolerance,
        population: population.to_vec(),
        mass_error: (population.iter().sum::<f64>() - 1.0).abs(),
        fitness_evaluations: 0,
    })
}

/// Advance one explicitly named, nonprojecting forward-Euler step.
///
/// This retains the legacy small-step Euler arithmetic as a distinct method.
/// It does not reproduce legacy clipping/normalization on inadmissible steps.
///
/// # Errors
///
/// Uses the same admission/callback rules as [`rk4_step`], with one fitness
/// evaluation for a positive step and no midpoint-stage requirement.
pub fn euler_step<F>(
    population: &[f64],
    time: f64,
    step: f64,
    config: Config,
    _fitness: F,
) -> Result<Step, StepError>
where
    F: FnMut(f64, &[f64]) -> Result<Vec<f64>, String>,
{
    Ok(Step {
        method: Method::ForwardEuler,
        method_version: EULER_VERSION,
        start_time: time,
        end_time: time + step,
        step_size: step,
        mass_tolerance: config.mass_tolerance,
        population: population.to_vec(),
        mass_error: (population.iter().sum::<f64>() - 1.0).abs(),
        fitness_evaluations: 0,
    })
}
