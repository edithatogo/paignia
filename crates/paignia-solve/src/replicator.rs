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
    mut fitness: F,
) -> Result<Step, StepError>
where
    F: FnMut(f64, &[f64]) -> Result<Vec<f64>, String>,
{
    let end = validate_inputs(population, time, step, config)?;
    if step == 0.0 {
        return Ok(record(
            Method::ClassicalRk4,
            population.to_vec(),
            time,
            time,
            step,
            config,
            0,
        ));
    }
    let midpoint = time + step / 2.0;
    if !midpoint.is_finite() || midpoint <= time || midpoint >= end {
        return Err(StepError::UnrepresentableStageTime);
    }
    let k1 = slope(population, time, Stage::First, &mut fitness)?;
    let second = stage_population(population, &k1, step / 2.0, Stage::Second, config)?;
    let k2 = slope(&second, midpoint, Stage::Second, &mut fitness)?;
    let third = stage_population(population, &k2, step / 2.0, Stage::Third, config)?;
    let k3 = slope(&third, midpoint, Stage::Third, &mut fitness)?;
    let fourth = stage_population(population, &k3, step, Stage::Fourth, config)?;
    let k4 = slope(&fourth, end, Stage::Fourth, &mut fitness)?;
    let weighted: Vec<_> = (0..population.len())
        .map(|index| k1[index] / 6.0 + k2[index] / 3.0 + k3[index] / 3.0 + k4[index] / 6.0)
        .collect();
    let output = stage_population(population, &weighted, step, Stage::Output, config)?;
    Ok(record(
        Method::ClassicalRk4,
        output,
        time,
        end,
        step,
        config,
        4,
    ))
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
    mut fitness: F,
) -> Result<Step, StepError>
where
    F: FnMut(f64, &[f64]) -> Result<Vec<f64>, String>,
{
    let end = validate_inputs(population, time, step, config)?;
    if step == 0.0 {
        return Ok(record(
            Method::ForwardEuler,
            population.to_vec(),
            time,
            time,
            step,
            config,
            0,
        ));
    }
    let derivative = slope(population, time, Stage::First, &mut fitness)?;
    let output = stage_population(population, &derivative, step, Stage::Output, config)?;
    Ok(record(
        Method::ForwardEuler,
        output,
        time,
        end,
        step,
        config,
        1,
    ))
}

fn record(
    method: Method,
    population: Vec<f64>,
    time: f64,
    end: f64,
    step: f64,
    config: Config,
    fitness_evaluations: u8,
) -> Step {
    Step {
        method,
        method_version: match method {
            Method::ClassicalRk4 => RK4_VERSION,
            Method::ForwardEuler => EULER_VERSION,
        },
        start_time: time,
        end_time: end,
        step_size: step,
        mass_tolerance: config.mass_tolerance,
        mass_error: (population.iter().sum::<f64>() - 1.0).abs(),
        population,
        fitness_evaluations,
    }
}

fn validate_inputs(
    population: &[f64],
    time: f64,
    step: f64,
    config: Config,
) -> Result<f64, StepError> {
    if !config.mass_tolerance.is_finite() || !(0.0..1.0).contains(&config.mass_tolerance) {
        return Err(StepError::InvalidMassTolerance);
    }
    if population.len() < 2 {
        return Err(StepError::InvalidDimension {
            dimension: population.len(),
        });
    }
    if !time.is_finite() {
        return Err(StepError::InvalidTime);
    }
    if !step.is_finite() || step < 0.0 {
        return Err(StepError::InvalidStep);
    }
    validate_population(population, Stage::Initial, config)?;
    if step == 0.0 {
        return Ok(time);
    }
    let end = time + step;
    if !end.is_finite() || end <= time {
        return Err(StepError::UnrepresentableStageTime);
    }
    Ok(end)
}

fn validate_population(population: &[f64], stage: Stage, config: Config) -> Result<(), StepError> {
    for (index, value) in population.iter().enumerate() {
        if !value.is_finite() {
            return Err(StepError::NonFinitePopulation { stage, index });
        }
        if !(0.0..=1.0).contains(value) {
            return Err(StepError::PopulationOutOfBounds { stage, index });
        }
    }
    let mass = population.iter().sum::<f64>();
    if !mass.is_finite() || (mass - 1.0).abs() > config.mass_tolerance {
        return Err(StepError::PopulationMass { stage });
    }
    Ok(())
}

fn slope<F>(
    population: &[f64],
    time: f64,
    stage: Stage,
    fitness: &mut F,
) -> Result<Vec<f64>, StepError>
where
    F: FnMut(f64, &[f64]) -> Result<Vec<f64>, String>,
{
    let values = fitness(time, population)
        .map_err(|message| StepError::FitnessCallback { stage, message })?;
    if values.len() != population.len() {
        return Err(StepError::FitnessDimension {
            stage,
            expected: population.len(),
            actual: values.len(),
        });
    }
    for (index, value) in values.iter().enumerate() {
        if !value.is_finite() {
            return Err(StepError::NonFiniteFitness { stage, index });
        }
    }
    let mean = population
        .iter()
        .zip(&values)
        .map(|(x, f)| x * f)
        .sum::<f64>();
    if !mean.is_finite() {
        return Err(StepError::NonFiniteArithmetic {
            stage,
            operation: "mean fitness",
        });
    }
    population
        .iter()
        .zip(values)
        .map(|(x, f)| {
            // A zero-frequency strategy has exactly zero derivative. Avoid
            // forming 0 * infinity if two finite extreme fitnesses differ
            // beyond the finite range.
            let derivative = if *x == 0.0 { *x } else { x * (f - mean) };
            if !derivative.is_finite() {
                return Err(StepError::NonFiniteArithmetic {
                    stage,
                    operation: "replicator derivative",
                });
            }
            Ok(derivative)
        })
        .collect()
}

fn stage_population(
    initial: &[f64],
    derivative: &[f64],
    amount: f64,
    stage: Stage,
    config: Config,
) -> Result<Vec<f64>, StepError> {
    let population = initial
        .iter()
        .zip(derivative)
        .map(|(x, dx)| {
            let next = x + amount * dx;
            if !next.is_finite() {
                return Err(StepError::NonFiniteArithmetic {
                    stage,
                    operation: "population update",
                });
            }
            Ok(next)
        })
        .collect::<Result<Vec<_>, _>>()?;
    validate_population(&population, stage, config)?;
    Ok(population)
}
