//! Independent review of e194e90612d14e99da53b8843dc2431d2a4510f4.
//! Oracles below are derived from polynomial quadrature and separable ODEs,
//! without calling the production method to obtain expected values.

#![deny(warnings)]

use paignia_solve::{
    replicator_euler_step, replicator_rk4_step, ReplicatorConfig, ReplicatorError,
    ReplicatorMethod, ReplicatorStage,
};

fn config() -> ReplicatorConfig {
    ReplicatorConfig {
        mass_tolerance: 1e-12,
    }
}

#[test]
fn cubic_forcing_has_an_exact_independent_quadrature_witness() {
    // Fitness f1=t^3/(x1*x2), f2=0 reduces to x1'=t^3 on unit mass.
    // Integral from 1/2 to 3/4 is 65/1024. Classical RK4 quadrature is
    // exact for a cubic in time; all expected stage states below are rational.
    let mut observed = Vec::new();
    let step = replicator_rk4_step(&[0.25, 0.75], 0.5, 0.25, config(), |t, x| {
        observed.push((t, x.to_vec()));
        Ok(vec![t.powi(3) / (x[0] * x[1]), 0.0])
    })
    .unwrap();
    let times = [0.5_f64, 0.625, 0.625, 0.75];
    let states = [0.25, 17.0 / 64.0, 1149.0 / 4096.0, 637.0 / 2048.0];
    assert_eq!(observed.len(), 4);
    for ((t, x), (expected_t, expected_x)) in observed.iter().zip(times.into_iter().zip(states)) {
        assert_eq!(t.to_bits(), expected_t.to_bits());
        assert!((x[0] - expected_x).abs() < 2e-15);
        assert!((x[0] + x[1] - 1.0).abs() < 2e-15);
    }
    assert!((step.population[0] - 321.0 / 1024.0).abs() < 2e-15);
    assert_eq!(step.fitness_evaluations, 4);
}

#[test]
fn nonlinear_state_dependent_flow_matches_a_separable_rational_solution() {
    // f1=x1/x2, f2=0 gives x1'=x1^2, hence x1(1)=1/3 for x1(0)=1/4.
    // This fitness changes with the intermediate state, unlike constant
    // fitness logistic controls. The expectation is an analytic solution.
    let mut rk4 = vec![0.25, 0.75];
    let mut euler = rk4.clone();
    for i in 0..32_u32 {
        let t = f64::from(i) / 32.0;
        rk4 = replicator_rk4_step(&rk4, t, 1.0 / 32.0, config(), |_, x| {
            Ok(vec![x[0] / x[1], 0.0])
        })
        .unwrap()
        .population;
        euler = replicator_euler_step(&euler, t, 1.0 / 32.0, config(), |_, x| {
            Ok(vec![x[0] / x[1], 0.0])
        })
        .unwrap()
        .population;
    }
    assert!((rk4[0] - 1.0 / 3.0).abs() < 1e-9);
    assert!((rk4.iter().sum::<f64>() - 1.0).abs() < 1e-12);
    assert!((euler[0] - 1.0 / 3.0).abs() > 1e-4);
}

#[test]
fn strategy_permutation_and_common_fitness_shift_preserve_the_flow() {
    let initial = [0.125, 0.25, 0.625];
    let baseline = replicator_rk4_step(&initial, -1.0, 1.0 / 16.0, config(), |_, _| {
        Ok(vec![-2.0, 1.0, 4.0])
    })
    .unwrap();
    let reversed =
        replicator_rk4_step(&[0.625, 0.25, 0.125], -1.0, 1.0 / 16.0, config(), |_, _| {
            Ok(vec![4.0, 1.0, -2.0])
        })
        .unwrap();
    let shifted = replicator_rk4_step(&initial, -1.0, 1.0 / 16.0, config(), |_, _| {
        Ok(vec![6.0, 9.0, 12.0])
    })
    .unwrap();
    for (i, actual) in baseline.population.iter().enumerate() {
        assert!((actual - reversed.population[2 - i]).abs() < 32.0 * f64::EPSILON);
        assert!((actual - shifted.population[i]).abs() < 32.0 * f64::EPSILON);
    }
    assert_eq!(
        baseline.mass_error.to_bits(),
        (baseline.population.iter().sum::<f64>() - 1.0)
            .abs()
            .to_bits()
    );
}

#[test]
fn an_inactive_extreme_fitness_strategy_stays_on_the_simplex_face() {
    let step = replicator_rk4_step(&[0.25, 0.75, -0.0], 0.0, 0.125, config(), |_, _| {
        Ok(vec![1.0, -1.0, f64::MAX])
    })
    .unwrap();
    assert_eq!(step.population[2].to_bits(), (-0.0_f64).to_bits());
    assert!(step
        .population
        .iter()
        .all(|x| x.is_finite() && (0.0..=1.0).contains(x)));
    assert!(step.mass_error <= 1e-12);
}

#[test]
fn each_callback_failure_stage_stops_immediately_and_retains_the_message() {
    let stages = [
        ReplicatorStage::First,
        ReplicatorStage::Second,
        ReplicatorStage::Third,
        ReplicatorStage::Fourth,
    ];
    for (index, stage) in stages.into_iter().enumerate() {
        for mode in 0..3 {
            let input = [0.25_f64, 0.75];
            let mut calls = 0;
            let error = replicator_rk4_step(&input, 0.0, 0.125, config(), |_, _| {
                calls += 1;
                if calls != index + 1 {
                    return Ok(vec![1.0, 0.0]);
                }
                match mode {
                    0 => Err("namespace/error\0 full message ".into()),
                    1 => Ok(vec![1.0]),
                    _ => Ok(vec![1.0, f64::NAN]),
                }
            })
            .unwrap_err();
            let expected = match mode {
                0 => ReplicatorError::FitnessCallback {
                    stage,
                    message: "namespace/error\0 full message ".into(),
                },
                1 => ReplicatorError::FitnessDimension {
                    stage,
                    expected: 2,
                    actual: 1,
                },
                _ => ReplicatorError::NonFiniteFitness { stage, index: 1 },
            };
            assert_eq!(error, expected);
            assert_eq!(calls, index + 1);
            assert_eq!(input.map(f64::to_bits), [0.25, 0.75].map(f64::to_bits));
        }
    }
}

#[test]
fn every_intermediate_and_final_domain_failure_is_exposed() {
    let cases = [
        (1, ReplicatorStage::Second, 8.0),
        (2, ReplicatorStage::Third, 8.0),
        (3, ReplicatorStage::Fourth, 8.0),
        (4, ReplicatorStage::Output, 24.0),
    ];
    for (failure_after, expected_stage, spike) in cases {
        let mut calls = 0;
        let error = replicator_rk4_step(&[0.5, 0.5], 0.0, 1.0, config(), |_, x| {
            calls += 1;
            assert!(x.iter().all(|x| (0.0..=1.0).contains(x)));
            Ok(vec![if calls == failure_after { spike } else { 0.0 }, 0.0])
        })
        .unwrap_err();
        assert_eq!(
            error,
            ReplicatorError::PopulationOutOfBounds {
                stage: expected_stage,
                index: 0
            }
        );
        assert_eq!(calls, failure_after);
    }
}

#[test]
fn intermediate_mass_failure_is_not_hidden_by_in_range_components() {
    // Initial mass 0.75 is within the deliberately loose admission budget.
    // Equal negative fitness drives this off-simplex input farther away; all
    // components remain in range, so the independent mass gate must reject.
    let mut calls = 0;
    let result = replicator_rk4_step(
        &[0.25, 0.5],
        0.0,
        1.0,
        ReplicatorConfig {
            mass_tolerance: 0.3,
        },
        |_, _| {
            calls += 1;
            Ok(vec![-4.0, -4.0])
        },
    );
    assert_eq!(calls, 1);
    assert_eq!(
        result,
        Err(ReplicatorError::PopulationMass {
            stage: ReplicatorStage::Second
        })
    );
}

#[test]
fn midpoint_representability_distinguishes_rk4_from_euler() {
    let eps = f64::EPSILON;
    let result = replicator_rk4_step(&[0.5, 0.5], 1.0, eps, config(), |_, _| {
        panic!("unrepresentable midpoint reached callback")
    });
    assert_eq!(result, Err(ReplicatorError::UnrepresentableStageTime));
    let euler =
        replicator_euler_step(&[0.5, 0.5], 1.0, eps, config(), |_, _| Ok(vec![0.0, 0.0])).unwrap();
    assert_eq!(euler.method, ReplicatorMethod::ForwardEuler);
    assert_eq!(euler.fitness_evaluations, 1);
    assert_eq!(euler.end_time.to_bits(), (1.0 + eps).to_bits());
    for (start, h) in [(1.0, 2.0 * eps), (-1.0, eps), (0.0, f64::from_bits(2))] {
        let mut observed = Vec::new();
        let result = replicator_rk4_step(&[0.5, 0.5], start, h, config(), |t, _| {
            observed.push(t);
            Ok(vec![0.0, 0.0])
        })
        .unwrap();
        assert_eq!(
            observed,
            [start, start + h / 2.0, start + h / 2.0, start + h]
        );
        assert_eq!(result.population, [0.5, 0.5]);
        assert_eq!(result.method, ReplicatorMethod::ClassicalRk4);
        assert_eq!(result.method_version, "1.0.0");
    }
}

#[test]
fn zero_step_still_rejects_invalid_input_without_a_callback() {
    let invalid_dimension =
        replicator_rk4_step(&[1.0], 0.0, 0.0, config(), |_, _| panic!("invalid input"));
    assert_eq!(
        invalid_dimension,
        Err(ReplicatorError::InvalidDimension { dimension: 1 })
    );
    let invalid_mass = replicator_rk4_step(&[0.1, 0.2], 0.0, -0.0, config(), |_, _| {
        panic!("invalid input")
    });
    assert_eq!(
        invalid_mass,
        Err(ReplicatorError::PopulationMass {
            stage: ReplicatorStage::Initial
        })
    );
    let invalid_time = replicator_euler_step(&[0.25, 0.75], f64::NAN, -0.0, config(), |_, _| {
        panic!("invalid input")
    });
    assert_eq!(invalid_time, Err(ReplicatorError::InvalidTime));
}

#[test]
fn asymmetric_segment_integrals_cancel_only_with_the_correct_endpoint_extension() {
    // Integral s(t)dt is 3*(1/4) - 1*(3/4)=0, so the exact final population
    // equals the initial population for the two-strategy logistic equation.
    let mut split = vec![0.5, 0.5];
    let mut global = split.clone();
    for i in 0..64_u32 {
        let t = 2.0 + f64::from(i) / 64.0;
        let segment_value = if i < 16 { 3.0 } else { -1.0 };
        split = replicator_rk4_step(&split, t, 1.0 / 64.0, config(), |_, _| {
            Ok(vec![segment_value, 0.0])
        })
        .unwrap()
        .population;
        global = replicator_rk4_step(&global, t, 1.0 / 64.0, config(), |time, _| {
            Ok(vec![if time < 2.25 { 3.0 } else { -1.0 }, 0.0])
        })
        .unwrap()
        .population;
    }
    assert!((split[0] - 0.5).abs() < 1e-8);
    assert!((global[0] - 0.5).abs() > 1e-4);
}
