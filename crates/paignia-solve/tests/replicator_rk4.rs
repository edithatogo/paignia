//! Production checks against independently derived rational/logistic witnesses.
//!
//! Frozen source: `verification/numerical_oracles/replicator_rk4_v1.json`,
//! SHA-256 e793f8f4d2121e58d4195ee2967bfb61fbe533377863a18b3926a08b3210d5e3.

use paignia_solve::{
    replicator_euler_step, replicator_rk4_step, ReplicatorConfig, ReplicatorError,
    ReplicatorMethod, ReplicatorStage, REPLICATOR_EULER_VERSION, REPLICATOR_RK4_VERSION,
};

const MASS_TOLERANCE: f64 = 1e-12;

fn config() -> ReplicatorConfig {
    ReplicatorConfig {
        mass_tolerance: MASS_TOLERANCE,
    }
}

type Fitness = fn(f64, &[f64]) -> Result<Vec<f64>, String>;
const CONSTANT: Fitness = |_, _| Ok(vec![1.0, 0.0]);

fn integrate<F>(
    mut population: Vec<f64>,
    start: f64,
    end: f64,
    count: u32,
    mut fitness: F,
    euler: bool,
) -> Vec<f64>
where
    F: FnMut(f64, &[f64]) -> Result<Vec<f64>, String>,
{
    let step = (end - start) / f64::from(count);
    for index in 0..count {
        let time = start + f64::from(index) * step;
        let result = if euler {
            replicator_euler_step(&population, time, step, config(), &mut fitness)
        } else {
            replicator_rk4_step(&population, time, step, config(), &mut fitness)
        }
        .unwrap();
        assert!(result.mass_error <= MASS_TOLERANCE);
        assert!(result.population.iter().all(|x| (0.0..=1.0).contains(x)));
        population = result.population;
    }
    population
}

fn rational(numerator: &str, denominator: &str) -> f64 {
    numerator.parse::<f64>().unwrap() / denominator.parse::<f64>().unwrap()
}

#[test]
fn frozen_four_stage_witness_and_method_identity_are_preserved() {
    let mut calls = Vec::new();
    let result = replicator_rk4_step(&[0.25, 0.75], 0.0, 0.5, config(), |time, state| {
        calls.push((time, state.to_vec()));
        Ok(vec![1.0, 0.0])
    })
    .unwrap();
    assert_eq!(REPLICATOR_RK4_VERSION, "1.0.0");
    assert_eq!(result.method, ReplicatorMethod::ClassicalRk4);
    assert_eq!(result.method.id(), "paignia/replicator-classical-rk4");
    assert_eq!(result.method_version, "1.0.0");
    assert_eq!(result.fitness_evaluations, 4);
    assert_eq!(calls.len(), 4);
    let times = [0.0_f64, 0.25, 0.25, 0.5];
    let first_states = [
        0.25,
        rational("19", "64"),
        rational("4951", "16384"),
        rational("190822511", "536870912"),
    ];
    for ((time, state), (expected_time, first)) in calls.iter().zip(times.iter().zip(first_states))
    {
        assert_eq!(time.to_bits(), expected_time.to_bits());
        assert!((state[0] - first).abs() <= 4.0 * f64::EPSILON);
        assert!((state.iter().sum::<f64>() - 1.0).abs() <= MASS_TOLERANCE);
    }
    let expected = rational("408885515741609973", "1152921504606846976");
    assert!((result.population[0] - expected).abs() <= 4.0 * f64::EPSILON);
    assert_eq!(result.start_time.to_bits(), 0.0_f64.to_bits());
    assert_eq!(result.end_time.to_bits(), 0.5_f64.to_bits());
    assert_eq!(result.step_size.to_bits(), 0.5_f64.to_bits());
    assert_eq!(result.mass_tolerance.to_bits(), MASS_TOLERANCE.to_bits());
}

#[test]
fn separately_named_euler_is_not_rk4() {
    let euler = replicator_euler_step(&[0.25, 0.75], 0.0, 0.5, config(), CONSTANT).unwrap();
    let rk4 = replicator_rk4_step(&[0.25, 0.75], 0.0, 0.5, config(), CONSTANT).unwrap();
    assert_eq!(REPLICATOR_EULER_VERSION, "1.0.0");
    assert_eq!(euler.method, ReplicatorMethod::ForwardEuler);
    assert_eq!(euler.method.id(), "paignia/replicator-forward-euler");
    assert_eq!(euler.method_version, "1.0.0");
    assert_eq!(euler.fitness_evaluations, 1);
    assert_eq!(euler.population[0].to_bits(), (11.0_f64 / 32.0).to_bits());
    assert!((euler.population[0] - rk4.population[0]).abs() > 1e-3);
}

#[test]
fn constant_positive_logistic_has_fourth_order_refinement() {
    check_refinement(1.0, 0.0, [8, 16, 32], "0.475366886418671691097653482610462410143978975060974436435118025053784674434418651672751763");
}

#[test]
fn constant_negative_logistic_has_fourth_order_refinement() {
    check_refinement(-1.0, 0.0, [8, 16, 32], "0.109231772573035928014267726759222873947887519142982896767210838808287709293363589847379970");
}

#[test]
fn affine_time_fitness_has_fourth_order_refinement() {
    check_refinement(1.0, 2.0, [32, 64, 128], "0.711234594227593859942056149222187642798586337981888539215935164643753568815025604211938211");
}

fn check_refinement(a: f64, b: f64, counts: [u32; 3], analytic: &str) {
    let expected = analytic.parse::<f64>().unwrap();
    let mut errors = Vec::new();
    for count in counts {
        let result = integrate(
            vec![0.25, 0.75],
            0.0,
            1.0,
            count,
            |time, _| Ok(vec![a + b * time, 0.0]),
            false,
        );
        errors.push((result[0] - expected).abs());
    }
    eprintln!("a={a}, b={b}, errors={errors:?}");
    assert!(errors[2] > 1e-14);
    assert!(errors[2] < 1e-9);
    for pair in errors.windows(2) {
        let ratio = pair[0] / pair[1];
        assert!(ratio > 14.0 && ratio < 18.0, "refinement ratio {ratio}");
    }
}

#[test]
fn euler_negative_control_fails_the_finest_rk4_accuracy_bound() {
    let target = "0.475366886418671691097653482610462410143978975060974436435118025053784674434418651672751763".parse::<f64>().unwrap();
    let result = integrate(vec![0.25, 0.75], 0.0, 1.0, 32, CONSTANT, true);
    assert!((result[0] - target).abs() > 1e-4);
}

#[test]
fn callback_receives_actual_nonzero_stage_times() {
    let mut times = Vec::new();
    replicator_rk4_step(&[0.25, 0.75], 2.0, 0.125, config(), |time, _| {
        times.push(time);
        Ok(vec![1.0 + 2.0 * time, 0.0])
    })
    .unwrap();
    let expected = [2.0_f64, 33.0 / 16.0, 33.0 / 16.0, 17.0 / 8.0];
    assert_eq!(
        times.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
        expected.iter().map(|x| x.to_bits()).collect::<Vec<_>>()
    );
}

#[test]
fn frozen_time_negative_control_fails_the_analytic_trajectory() {
    let mut population = vec![0.25, 0.75];
    let step = 1.0 / 64.0;
    for index in 0..64_u32 {
        let time = f64::from(index) * step;
        population = replicator_rk4_step(&population, time, step, config(), |_, _| {
            Ok(vec![1.0 + 2.0 * time, 0.0])
        })
        .unwrap()
        .population;
    }
    let target = "0.711234594227593859942056149222187642798586337981888539215935164643753568815025604211938211".parse::<f64>().unwrap();
    assert!((population[0] - target).abs() > 1e-4);
}

#[test]
fn state_dependent_fitness_is_recomputed_at_each_stage() {
    // f1=1/(x*(1-x)), f2=0 gives x'=1 on the interior: x(t)=x0+t.
    let mut states = Vec::new();
    let result = replicator_rk4_step(&[0.25, 0.75], 0.0, 0.125, config(), |_, state| {
        states.push(state[0]);
        Ok(vec![1.0 / (state[0] * state[1]), 0.0])
    })
    .unwrap();
    assert_eq!(states.len(), 4);
    for (actual, expected) in states.iter().zip([0.25, 0.3125, 0.3125, 0.375]) {
        assert!((actual - expected).abs() < 64.0 * f64::EPSILON);
    }
    assert!((result.population[0] - 0.375).abs() < 64.0 * f64::EPSILON);
}

#[test]
fn segment_specific_callbacks_handle_a_discontinuity() {
    let left = integrate(
        vec![0.25, 0.75],
        0.0,
        0.5,
        16,
        |_, _| Ok(vec![1.0, 0.0]),
        false,
    );
    let right = integrate(left, 0.5, 1.0, 16, |_, _| Ok(vec![-1.0, 0.0]), false);
    assert!((right[0] - 0.25).abs() < 1e-9);
    let unsplit = integrate(
        vec![0.25, 0.75],
        0.0,
        1.0,
        32,
        |time, _| Ok(vec![if time < 0.5 { 1.0 } else { -1.0 }, 0.0]),
        false,
    );
    assert!((unsplit[0] - 0.25).abs() > 1e-5);
}

#[test]
fn simplex_faces_and_equal_fitness_are_stationary() {
    for population in [[0.0, 1.0], [1.0, 0.0]] {
        let result =
            replicator_rk4_step(&population, 0.0, 1.0, config(), |_, _| Ok(vec![-2.0, 3.0]))
                .unwrap();
        assert_eq!(result.population, population);
    }
    let population = [0.25, 0.75];
    let result =
        replicator_rk4_step(&population, 0.0, 1.0, config(), |_, _| Ok(vec![-7.0, -7.0])).unwrap();
    assert_eq!(result.population, population);
}

#[test]
fn zero_frequency_does_not_form_zero_times_overflowed_fitness_difference() {
    let result = replicator_rk4_step(&[1.0, 0.0], 0.0, 1.0, config(), |_, _| {
        Ok(vec![f64::MAX, -f64::MAX])
    })
    .unwrap();
    assert_eq!(result.population, [1.0, 0.0]);
}

#[test]
fn three_strategy_constant_fitness_matches_its_independent_closed_form() {
    let initial = [0.2_f64, 0.3, 0.5];
    let fitness = [-1.0_f64, 0.5, 2.0];
    let numerators = initial
        .iter()
        .zip(fitness)
        .map(|(x, f)| x * (f * 0.5).exp())
        .collect::<Vec<_>>();
    let denominator: f64 = numerators.iter().sum();
    let result = integrate(
        initial.to_vec(),
        0.0,
        0.5,
        64,
        |_, _| Ok(fitness.to_vec()),
        false,
    );
    for (actual, numerator) in result.iter().zip(numerators) {
        assert!((actual - numerator / denominator).abs() < 1e-9);
    }
}

#[test]
fn zero_step_preserves_all_bits_without_evaluating_fitness() {
    let population = [-0.0_f64, 1.0];
    for step in [0.0, -0.0] {
        let result = replicator_rk4_step(&population, -0.0, step, config(), |_, _| {
            panic!("zero step must not call fitness")
        })
        .unwrap();
        assert_eq!(result.fitness_evaluations, 0);
        assert_eq!(result.start_time.to_bits(), (-0.0_f64).to_bits());
        assert_eq!(result.end_time.to_bits(), (-0.0_f64).to_bits());
        assert_eq!(result.step_size.to_bits(), step.to_bits());
        for (actual, expected) in result.population.iter().zip(population) {
            assert_eq!(actual.to_bits(), expected.to_bits());
        }
    }
}

#[test]
fn bad_population_dimensions_fail_before_callback() {
    for population in [vec![], vec![1.0]] {
        assert_eq!(
            replicator_rk4_step(&population, 0.0, 0.1, config(), |_, _| panic!(
                "invalid input called callback"
            )),
            Err(ReplicatorError::InvalidDimension {
                dimension: population.len()
            })
        );
    }
}

#[test]
fn nonfinite_or_out_of_range_population_is_not_repaired() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(matches!(
            replicator_rk4_step(&[value, 0.75], 0.0, 0.1, config(), CONSTANT),
            Err(ReplicatorError::NonFinitePopulation {
                stage: ReplicatorStage::Initial,
                index: 0
            })
        ));
    }
    for value in [-0.1, -f64::from_bits(1), 1.1] {
        assert!(matches!(
            replicator_rk4_step(&[value, 0.75], 0.0, 0.1, config(), CONSTANT),
            Err(ReplicatorError::PopulationOutOfBounds {
                stage: ReplicatorStage::Initial,
                index: 0
            })
        ));
    }
}

#[test]
fn wrong_mass_and_zero_mass_are_not_normalized() {
    for population in [[0.25, 0.5], [0.0, 0.0], [1.0, 1.0]] {
        assert_eq!(
            replicator_rk4_step(&population, 0.0, 0.1, config(), CONSTANT),
            Err(ReplicatorError::PopulationMass {
                stage: ReplicatorStage::Initial
            })
        );
    }
}

#[test]
fn invalid_mass_tolerances_reject() {
    for mass_tolerance in [f64::NAN, f64::INFINITY, -1.0, -f64::from_bits(1), 1.0, 2.0] {
        assert_eq!(
            replicator_rk4_step(
                &[0.25, 0.75],
                0.0,
                0.1,
                ReplicatorConfig { mass_tolerance },
                CONSTANT
            ),
            Err(ReplicatorError::InvalidMassTolerance)
        );
    }
}

#[test]
fn mass_tolerance_is_an_admission_budget_not_a_normalization_command() {
    let population = [0.25, 0.75 - 2e-13];
    let result = replicator_rk4_step(&population, 0.0, 0.0, config(), CONSTANT).unwrap();
    assert_eq!(result.population, population);
    assert!(result.mass_error > 0.0 && result.mass_error < MASS_TOLERANCE);
    assert!(replicator_rk4_step(
        &population,
        0.0,
        0.0,
        ReplicatorConfig {
            mass_tolerance: 0.0
        },
        CONSTANT
    )
    .is_err());
}

#[test]
fn nonfinite_time_and_invalid_steps_reject() {
    for time in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            replicator_rk4_step(&[0.25, 0.75], time, 0.1, config(), CONSTANT),
            Err(ReplicatorError::InvalidTime)
        );
    }
    for step in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        -0.1,
        -f64::from_bits(1),
    ] {
        assert_eq!(
            replicator_rk4_step(&[0.25, 0.75], 0.0, step, config(), CONSTANT),
            Err(ReplicatorError::InvalidStep)
        );
    }
}

#[test]
fn positive_steps_require_distinct_representable_stage_times() {
    for (time, step) in [
        (f64::MAX, f64::MAX),
        (1.0, f64::from_bits(1)),
        (0.0, f64::from_bits(1)),
    ] {
        assert_eq!(
            replicator_rk4_step(&[0.25, 0.75], time, step, config(), |_, _| panic!(
                "invalid time grid called callback"
            )),
            Err(ReplicatorError::UnrepresentableStageTime)
        );
    }
}

#[test]
fn fitness_dimension_is_checked_at_each_stage() {
    for actual in [0, 1, 3] {
        let mut calls = 0;
        let error = replicator_rk4_step(&[0.25, 0.75], 0.0, 0.1, config(), |_, _| {
            calls += 1;
            Ok(if calls == 2 {
                vec![1.0; actual]
            } else {
                vec![1.0, 0.0]
            })
        })
        .unwrap_err();
        assert_eq!(
            error,
            ReplicatorError::FitnessDimension {
                stage: ReplicatorStage::Second,
                expected: 2,
                actual
            }
        );
        assert_eq!(calls, 2);
    }
}

#[test]
fn nonfinite_fitness_is_not_an_accepted_population_result() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let error = replicator_rk4_step(&[0.25, 0.75], 0.0, 0.1, config(), |_, _| {
            Ok(vec![value, 0.0])
        })
        .unwrap_err();
        assert_eq!(
            error,
            ReplicatorError::NonFiniteFitness {
                stage: ReplicatorStage::First,
                index: 0
            }
        );
    }
}

#[test]
fn callback_failure_is_typed_and_keeps_its_stage() {
    let mut calls = 0;
    let result = replicator_rk4_step(&[0.25, 0.75], 0.0, 0.1, config(), |_, _| {
        calls += 1;
        if calls == 3 {
            Err("declared domain unavailable".into())
        } else {
            Ok(vec![1.0, 0.0])
        }
    });
    assert_eq!(
        result,
        Err(ReplicatorError::FitnessCallback {
            stage: ReplicatorStage::Third,
            message: "declared domain unavailable".into()
        })
    );
    assert_eq!(calls, 3);
}

#[test]
fn oversized_steps_fail_without_clipping_or_normalizing() {
    let input = [0.25_f64, 0.75];
    let result = replicator_rk4_step(&input, 0.0, 16.0, config(), CONSTANT);
    assert!(matches!(
        result,
        Err(ReplicatorError::PopulationOutOfBounds {
            stage: ReplicatorStage::Second,
            ..
        })
    ));
    assert_eq!(input.map(f64::to_bits), [0.25, 0.75].map(f64::to_bits));
    assert!(replicator_euler_step(&input, 0.0, 16.0, config(), CONSTANT).is_err());
}

#[test]
fn intermediate_states_are_rejected_before_calling_fitness_outside_its_domain() {
    let mut calls = 0;
    let error = replicator_rk4_step(&[0.25, 0.75], 0.0, 16.0, config(), |_, state| {
        calls += 1;
        assert!(state.iter().all(|x| (0.0..=1.0).contains(x)));
        Ok(vec![1.0, 0.0])
    })
    .unwrap_err();
    assert_eq!(calls, 1);
    assert!(matches!(
        error,
        ReplicatorError::PopulationOutOfBounds {
            stage: ReplicatorStage::Second,
            ..
        }
    ));
}

#[test]
fn nonfinite_intermediate_arithmetic_is_an_explicit_failure() {
    let result = replicator_rk4_step(&[0.5 + f64::EPSILON, 0.5], 0.0, 0.1, config(), |_, _| {
        Ok(vec![f64::MAX, f64::MAX])
    });
    assert!(matches!(
        result,
        Err(ReplicatorError::NonFiniteArithmetic {
            stage: ReplicatorStage::First,
            ..
        })
    ));
}

#[test]
fn legacy_euler_keeps_the_same_input_and_callback_failure_rules() {
    assert_eq!(
        replicator_euler_step(&[0.25, 0.75], 0.0, -1.0, config(), CONSTANT),
        Err(ReplicatorError::InvalidStep)
    );
    assert_eq!(
        replicator_euler_step(&[0.25, 0.75], 0.0, 0.1, config(), |_, _| Err(
            "failure".into()
        )),
        Err(ReplicatorError::FitnessCallback {
            stage: ReplicatorStage::First,
            message: "failure".into()
        })
    );
    let identity = replicator_euler_step(&[-0.0, 1.0], -0.0, -0.0, config(), |_, _| {
        panic!("zero Euler step called fitness")
    })
    .unwrap();
    assert_eq!(identity.population[0].to_bits(), (-0.0_f64).to_bits());
    assert_eq!(identity.end_time.to_bits(), (-0.0_f64).to_bits());
}
