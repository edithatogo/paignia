//! Independent public-API CE controls. No backend is used to produce an oracle.
#![cfg(feature = "correlated-equilibrium")]
#![allow(clippy::float_cmp)]

use paignia_solve::{
    evaluate_correlated_equilibrium_candidate as evaluate,
    evaluate_correlated_equilibrium_certificate as certificate,
    solve_correlated_equilibrium as solve, CorrelatedEquilibriumConfig as Config,
    CorrelatedEquilibriumDualVariables as Dual, CorrelatedEquilibriumError as Error,
    CorrelatedEquilibriumInterval as Interval, CorrelatedEquilibriumPlayer as Player,
    CorrelatedEquilibriumStatus as Status, CorrelatedEquilibriumTermination as Termination,
};

type Matrix = Vec<Vec<f64>>;

fn encloses(bound: Interval, exact: f64) {
    assert!(bound.lower.is_finite() && bound.upper.is_finite());
    assert!(
        bound.lower <= exact && exact <= bound.upper,
        "{bound:?} excludes {exact}"
    );
}

fn zeros(rows: usize, columns: usize) -> Matrix {
    vec![vec![0.; columns]; rows]
}

fn gap() -> (Matrix, Matrix, Matrix) {
    (
        vec![vec![0., 7.], vec![2., 6.]],
        vec![vec![0., 2.], vec![7., 6.]],
        vec![vec![0., 9.], vec![9., 12.]],
    )
}

fn permute(matrix: &[Vec<f64>], rows: &[usize], columns: &[usize]) -> Matrix {
    rows.iter()
        .map(|&i| columns.iter().map(|&j| matrix[i][j]).collect())
        .collect()
}

fn pair_index(actions: usize, recommendation: usize, deviation: usize) -> usize {
    recommendation * (actions - 1)
        + if deviation < recommendation {
            deviation
        } else {
            deviation - 1
        }
}

fn permute_dual(dual: &Dual, rows: &[usize], columns: &[usize]) -> Dual {
    let r = rows.len();
    let s = columns.len();
    let mut incentives = Vec::new();
    for (offset, permutation) in [(0, rows), (r * (r - 1), columns)] {
        for &recommendation in permutation {
            for &deviation in permutation {
                if recommendation != deviation {
                    incentives.push(
                        dual.incentives
                            [offset + pair_index(permutation.len(), recommendation, deviation)],
                    );
                }
            }
        }
    }
    Dual {
        mass: dual.mass,
        nonnegativity: rows
            .iter()
            .flat_map(|&i| columns.iter().map(move |&j| dual.nonnegativity[i * s + j]))
            .collect(),
        incentives,
    }
}

#[test]
fn a_dual_feasible_zero_gap_tuple_does_not_admit_an_infeasible_primal() {
    let (a, b, c) = gap();
    // alpha=12 >= every objective coefficient: this is an exact feasible dual.
    // z11=1 achieves its bound but violates BOTH players' obedience constraints.
    let dual = Dual {
        mass: 12.,
        nonnegativity: vec![12., 3., 3., 0.],
        incentives: vec![0.; 4],
    };
    let result = certificate(&a, &b, &c, &[0., 0., 0., 1.], &dual, Config::default()).unwrap();
    assert!(result.dual_inequalities_nonnegative);
    assert_eq!(result.objective_gap_abs_upper, 0.);
    assert!(result.stationarity_error_upper < 1e-8);
    assert!(!result.within_tolerance);
}

#[test]
fn signed_nonnegativity_multipliers_cannot_hide_behind_zero_residuals() {
    let a = zeros(1, 2);
    let dual = Dual {
        mass: 0.,
        nonnegativity: vec![0., -1.],
        incentives: vec![0.; 2],
    };
    let result = certificate(&a, &a, &[vec![0., 1.]], &[1., 0.], &dual, Config::default()).unwrap();
    assert_eq!(result.stationarity_error_upper, 0.);
    assert_eq!(result.objective_gap_abs_upper, 0.);
    assert!(!result.dual_inequalities_nonnegative);
    assert!(!result.within_tolerance);
}

#[test]
fn every_nonzero_three_action_dual_coordinate_survives_independent_permutations() {
    // All twelve recommendation/deviation constraints bind at the uniform joint.
    // c = 5*1 + D^T*[1,2,...,12] is computed analytically, not by the backend.
    let a = vec![vec![0., -1., 1.], vec![1., 0., -1.], vec![-1., 1., 0.]];
    let b: Matrix = a
        .iter()
        .map(|row| row.iter().map(|value| -value).collect())
        .collect();
    let c = vec![vec![3., -19., 36.], vec![17., 7., -19.], vec![0., 17., 3.]];
    let dual = Dual {
        mass: 5.,
        nonnegativity: vec![0.; 9],
        incentives: (1..=12).map(f64::from).collect(),
    };
    let permutations = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    for rows in permutations {
        for columns in permutations {
            let (a, b, c) = (
                permute(&a, &rows, &columns),
                permute(&b, &rows, &columns),
                permute(&c, &rows, &columns),
            );
            let dual = permute_dual(&dual, &rows, &columns);
            let result = certificate(&a, &b, &c, &[1. / 9.; 9], &dual, Config::default()).unwrap();
            assert!(
                result.within_tolerance,
                "rows={rows:?}, columns={columns:?}: {result:?}"
            );
            for residual in result.stationarity {
                encloses(residual, 0.);
            }
            encloses(result.objective_gap, 0.);
        }
    }
    let record = solve(&a, &b, &c, Config::default()).unwrap();
    assert_eq!(record.status(), Status::ApproximateOptimum, "{record:?}");
    let raw = record.accepted_candidate().unwrap();
    assert!((raw.primal.objective.lower - 5.).abs() < 1e-8);
    assert!((raw.primal.objective.upper - 5.).abs() < 1e-8);
}

#[test]
fn rectangular_certificate_and_transposed_game_preserve_player_roles() {
    let a = vec![vec![1., -1., 0.], vec![-1., 1., 0.]];
    let b = vec![vec![-1., 1., -2.], vec![1., -1., -2.]];
    let c = vec![vec![0., 0., -2.], vec![0., 0., -2.]];
    let joint = vec![vec![0.25, 0.25, 0.], vec![0.25, 0.25, 0.]];
    let dual = Dual {
        mass: 0.,
        nonnegativity: vec![0., 0., 2., 0., 0., 2.],
        incentives: vec![0.; 8],
    };
    for rows in [[0, 1], [1, 0]] {
        for columns in [[0, 1, 2], [2, 0, 1], [1, 2, 0]] {
            let a = permute(&a, &rows, &columns);
            let b = permute(&b, &rows, &columns);
            let c = permute(&c, &rows, &columns);
            let joint: Vec<f64> = permute(&joint, &rows, &columns)
                .into_iter()
                .flatten()
                .collect();
            assert!(
                certificate(
                    &a,
                    &b,
                    &c,
                    &joint,
                    &permute_dual(&dual, &rows, &columns),
                    Config::default()
                )
                .unwrap()
                .within_tolerance
            );
        }
    }
    let transpose =
        |m: &[Vec<f64>]| -> Matrix { (0..3).map(|j| (0..2).map(|i| m[i][j]).collect()).collect() };
    let record = solve(
        &transpose(&b),
        &transpose(&a),
        &transpose(&c),
        Config::default(),
    )
    .unwrap();
    assert_eq!(record.dimensions(), (3, 2));
    assert_eq!(record.status(), Status::ApproximateOptimum, "{record:?}");
    assert!(
        record
            .accepted_candidate()
            .unwrap()
            .primal
            .objective
            .abs_upper_for_test()
            < 1e-8
    );
}

// Keep interval comparisons explicit without relying on private implementation.
trait AbsoluteInterval {
    fn abs_upper_for_test(self) -> f64;
}
impl AbsoluteInterval for Interval {
    fn abs_upper_for_test(self) -> f64 {
        self.lower.abs().max(self.upper.abs())
    }
}

#[test]
fn incentive_budgets_follow_each_players_original_units() {
    let (a, b, c) = gap();
    let scales = [2_f64.powi(-40), 2_f64.powi(20)];
    let a: Matrix = a
        .iter()
        .map(|row| row.iter().map(|value| value * scales[0]).collect())
        .collect();
    let b: Matrix = b
        .iter()
        .map(|row| row.iter().map(|value| value * scales[1]).collect())
        .collect();
    let joint = [1. / 16., 3. / 16., 3. / 16., 9. / 16.];
    let config = Config {
        incentive_tolerance: [0.25 * scales[0], 0.125 * scales[1]],
        ..Config::default()
    };
    let p = evaluate(&a, &b, &c, &joint, config).unwrap();
    assert!(p.maximum_weighted_gain_upper[0] < config.incentive_tolerance[0]);
    assert!(p.maximum_weighted_gain_upper[1] > config.incentive_tolerance[1]);
    assert!(!p.within_tolerance);
    let config = Config {
        incentive_tolerance: [0.125 * scales[0], 0.25 * scales[1]],
        ..config
    };
    let p = evaluate(&a, &b, &c, &joint, config).unwrap();
    assert!(p.maximum_weighted_gain_upper[0] > config.incentive_tolerance[0]);
    assert!(p.maximum_weighted_gain_upper[1] < config.incentive_tolerance[1]);
    assert!(!p.within_tolerance);
    for entry in p
        .incentives
        .iter()
        .filter(|entry| entry.recommendation == 1)
    {
        let scale = scales[usize::from(entry.player == Player::Column)];
        encloses(entry.weighted_gain, 3. / 16. * scale);
        encloses(entry.conditional_gain.unwrap(), scale / 4.);
    }
}

#[test]
fn affine_objective_and_independent_payoff_scales_transform_dual_units() {
    let (a, b, c) = gap();
    let row_scale = 2_f64.powi(16);
    let column_scale = 2_f64.powi(-16);
    let a: Matrix = a
        .iter()
        .map(|row| row.iter().map(|value| value * row_scale).collect())
        .collect();
    let b: Matrix = b
        .iter()
        .map(|row| row.iter().map(|value| value * column_scale).collect())
        .collect();
    let c: Matrix = c
        .iter()
        .map(|row| row.iter().map(|value| value * 8. - 3.).collect())
        .collect();
    let mut dual = Dual {
        mass: 81.,
        nonnegativity: vec![84., 0., 0., 0.],
        incentives: vec![0., 6. / row_scale, 0., 6. / column_scale],
    };
    let result = certificate(&a, &b, &c, &[0., 0.25, 0.25, 0.5], &dual, Config::default()).unwrap();
    assert!(result.within_tolerance, "{result:?}");
    encloses(result.objective_gap, 0.);
    dual.incentives[1] = 6.;
    dual.incentives[3] = 6.;
    let bad = certificate(&a, &b, &c, &[0., 0.25, 0.25, 0.5], &dual, Config::default()).unwrap();
    assert!(!bad.within_tolerance);
    assert!(bad.stationarity_error_upper > 1.);
}

#[test]
fn large_cancellation_bounds_contain_the_exact_gain_in_all_column_orders() {
    let h = 2_f64.powi(54);
    let a = vec![vec![h, 1., -h], vec![0.; 3]];
    let joint = vec![vec![0.; 3], vec![0.25, 0.5, 0.25]];
    for columns in [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ] {
        let a = permute(&a, &[0, 1], &columns);
        let joint: Vec<f64> = permute(&joint, &[0, 1], &columns)
            .into_iter()
            .flatten()
            .collect();
        let p = evaluate(&a, &zeros(2, 3), &zeros(2, 3), &joint, Config::default()).unwrap();
        let gain = p
            .incentives
            .iter()
            .find(|g| g.player == Player::Row && g.recommendation == 1)
            .unwrap();
        encloses(gain.weighted_gain, 0.5);
        assert!(!p.within_tolerance);
    }
}

#[test]
fn subtraction_and_product_rounding_cannot_erase_half_a_unit_of_gain() {
    let large = 2_f64.powi(54);
    let a = vec![vec![large, -large], vec![-1., 0.]];
    let p = evaluate(
        &a,
        &zeros(2, 2),
        &zeros(2, 2),
        &[0., 0., 0.5, 0.5],
        Config::default(),
    )
    .unwrap();
    encloses(p.incentives[1].weighted_gain, 0.5);
    assert!(!p.within_tolerance);
    let epsilon = 2_f64.powi(-27);
    let scale = 2_f64.powi(53);
    let a = vec![vec![-(1. + epsilon) * scale, scale / epsilon], vec![0., 0.]];
    let p = evaluate(
        &a,
        &zeros(2, 2),
        &zeros(2, 2),
        &[0., 0., 1. - epsilon, epsilon],
        Config::default(),
    )
    .unwrap();
    encloses(p.incentives[1].weighted_gain, 0.5);
    assert!(!p.within_tolerance);
}

#[test]
fn cancelled_payoff_and_objective_moments_retain_the_exact_half_unit() {
    let h = 2_f64.powi(54);
    let a = vec![vec![h, 1., -h]];
    let p = evaluate(&a, &zeros(1, 3), &a, &[0.25, 0.5, 0.25], Config::default()).unwrap();
    encloses(p.payoffs[0], 0.5);
    encloses(p.objective, 0.5);
}

fn subnormal_ticks(value: f64) -> i64 {
    assert!(value.abs() < f64::MIN_POSITIVE);
    let ticks = i64::try_from(value.abs().to_bits()).unwrap();
    if value.is_sign_negative() {
        -ticks
    } else {
        ticks
    }
}

#[test]
fn both_signs_and_half_subnormal_boundaries_are_enclosed() {
    for bits in 1_u64..=8 {
        for sign in [-1_i64, 1] {
            let value = f64::from_bits(bits) * if sign < 0 { -1. } else { 1. };
            let p = evaluate(
                &zeros(1, 2),
                &zeros(1, 2),
                &[vec![value, 0.]],
                &[0.5, 0.5],
                Config::default(),
            )
            .unwrap();
            let twice_exact_ticks = i64::try_from(bits).unwrap() * sign;
            assert!(2 * subnormal_ticks(p.objective.lower) <= twice_exact_ticks);
            assert!(2 * subnormal_ticks(p.objective.upper) >= twice_exact_ticks);
            assert!(p.objective.lower < p.objective.upper);
        }
    }
}

#[test]
fn exact_maximum_identities_succeed_and_unbounded_enclosures_fail_explicitly() {
    for value in [f64::MAX, -f64::MAX] {
        let table = [vec![value]];
        let primal = evaluate(&table, &table, &table, &[1.], Config::default()).unwrap();
        assert_eq!(
            primal.objective,
            Interval {
                lower: value,
                upper: value
            }
        );
        let dual = Dual {
            mass: value,
            nonnegativity: vec![0.],
            incentives: vec![],
        };
        assert!(
            certificate(&table, &table, &table, &[1.], &dual, Config::default())
                .unwrap()
                .within_tolerance
        );
    }
    let result = evaluate(
        &zeros(1, 2),
        &zeros(1, 2),
        &[vec![f64::MAX, f64::MAX]],
        &[0.5, 0.5],
        Config::default(),
    );
    assert!(matches!(result, Err(Error::Arithmetic { .. })));
}

#[test]
fn a_lost_mass_bit_is_visible_and_zero_raw_recommendation_mass_is_not_a_ratio() {
    let tiny = 2_f64.powi(-56);
    let strict = Config {
        probability_tolerance: 0.,
        ..Config::default()
    };
    let p = evaluate(
        &zeros(2, 1),
        &zeros(2, 1),
        &zeros(2, 1),
        &[1., tiny],
        strict,
    )
    .unwrap();
    assert!(!p.within_tolerance);
    assert!(p.mass_error_upper >= tiny);
    let subnormal = f64::from_bits(1);
    let p = evaluate(
        &[vec![0., 0.], vec![0., 1.]],
        &zeros(2, 2),
        &zeros(2, 2),
        &[-subnormal, subnormal, 1., 0.],
        Config {
            incentive_tolerance: [0., 0.],
            ..Config::default()
        },
    )
    .unwrap();
    let entry = &p.incentives[0];
    assert_eq!(
        entry.recommendation_mass,
        Interval {
            lower: 0.,
            upper: 0.
        }
    );
    assert!(entry.conditional_gain.is_none());
    encloses(entry.weighted_gain, subnormal);
    assert!(!p.within_tolerance);
}

#[test]
fn extreme_finite_backend_inputs_never_forge_an_accepted_status() {
    let config = Config {
        max_iterations: 10,
        ..Config::default()
    };
    for scale in [
        f64::from_bits(1),
        f64::MIN_POSITIVE,
        1e-300,
        1e300,
        f64::MAX,
    ] {
        let a = vec![vec![0., scale], vec![scale, 0.]];
        let b = vec![vec![scale, 0.], vec![0., scale]];
        let c = vec![vec![scale, 0.], vec![0., -scale]];
        let outcome = std::panic::catch_unwind(|| solve(&a, &b, &c, config));
        let outcome = outcome.expect("finite bounded input must not panic");
        if let Ok(record) = outcome {
            if let Some(candidate) = record.accepted_candidate() {
                assert_eq!(record.termination(), Termination::Solved);
                assert_eq!(record.status(), Status::ApproximateOptimum);
                assert!(
                    evaluate(&a, &b, &c, &candidate.raw_joint_weights, config)
                        .unwrap()
                        .within_tolerance
                );
                assert!(
                    certificate(
                        &a,
                        &b,
                        &c,
                        &candidate.raw_joint_weights,
                        &candidate.dual,
                        config
                    )
                    .unwrap()
                    .within_tolerance
                );
            } else {
                assert_ne!(record.status(), Status::ApproximateOptimum);
            }
        }
    }
}
