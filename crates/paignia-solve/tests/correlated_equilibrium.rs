//! Independent analytic cases and rejection controls for the bounded CE API.
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

fn gap_game() -> (Vec<Vec<f64>>, Vec<Vec<f64>>, Vec<Vec<f64>>) {
    (
        vec![vec![0., 7.], vec![2., 6.]],
        vec![vec![0., 2.], vec![7., 6.]],
        vec![vec![0., 9.], vec![9., 12.]],
    )
}

fn exact_dual() -> Dual {
    Dual {
        mass: 10.5,
        nonnegativity: vec![10.5, 0., 0., 0.],
        incentives: vec![0., 0.75, 0., 0.75],
    }
}

fn encloses(interval: Interval, value: f64) {
    assert!(interval.lower <= value, "{interval:?} excludes {value}");
    assert!(interval.upper >= value, "{interval:?} excludes {value}");
}

fn optimum(row: &[Vec<f64>], column: &[Vec<f64>], objective: &[Vec<f64>], value: f64) {
    let config = Config::default();
    let record = solve(row, column, objective, config).unwrap();
    assert_eq!(record.status(), Status::ApproximateOptimum);
    assert_eq!(record.termination(), Termination::Solved);
    assert_eq!(record.method(), "paignia/clarabel-correlated-equilibrium");
    assert_eq!(record.method_version(), "1.0.0");
    assert_eq!(record.backend_version(), "0.11.1");
    assert_eq!(record.dimensions(), (row.len(), row[0].len()));
    assert_eq!(record.config(), config);
    let candidate = record.accepted_candidate().unwrap();
    assert!(candidate.primal.within_tolerance);
    assert!(candidate.certificate.within_tolerance);
    assert!(candidate.primal.mass_error_upper <= config.probability_tolerance);
    assert!(candidate.primal.maximum_negative_weight <= config.probability_tolerance);
    assert!((candidate.primal.objective.lower - value).abs() < 1e-8);
    assert!((candidate.primal.objective.upper - value).abs() < 1e-8);
    assert!(candidate.certificate.objective_gap_abs_upper <= config.objective_tolerance);
    assert!(candidate.raw_joint_weights.iter().all(|value| value.is_finite()));
    let recomputed = evaluate(row, column, objective, &candidate.raw_joint_weights, config).unwrap();
    assert_eq!(recomputed, candidate.primal);
    let checked = certificate(
        row, column, objective, &candidate.raw_joint_weights, &candidate.dual, config,
    ).unwrap();
    assert_eq!(checked, candidate.certificate);
}

#[test]
fn correlated_optimum_exceeds_every_nash_product_in_the_frozen_example() {
    let (row, column, objective) = gap_game();
    optimum(&row, &column, &objective, 10.5);
}

#[test]
fn dominated_actions_and_unused_recommendations_have_a_certified_optimum() {
    optimum(
        &[vec![2., 2.], vec![1., 1.]],
        &[vec![2., 1.], vec![2., 1.]],
        &[vec![4., 3.], vec![3., 2.]], 4.,
    );
}

#[test]
fn rectangular_two_by_three_game_uses_column_player_incentives() {
    optimum(
        &[vec![1., -1., 0.], vec![-1., 1., 0.]],
        &[vec![-1., 1., -2.], vec![1., -1., -2.]],
        &[vec![0., 0., -2.], vec![0., 0., -2.]], 0.,
    );
}

#[test]
fn single_action_negative_objective_is_retained() {
    optimum(&[vec![-3.]], &[vec![-4.]], &[vec![-7.]], -7.);
}

#[test]
fn objective_is_supplied_by_the_caller() {
    optimum(
        &[vec![0., 0.], vec![0., 0.]], &[vec![0., 0.], vec![0., 0.]],
        &[vec![0., 2.], vec![1., 0.]], 2.,
    );
}

#[test]
fn feasible_nash_product_does_not_pass_the_optimality_certificate() {
    let (row, column, objective) = gap_game();
    let joint = [1. / 9., 2. / 9., 2. / 9., 4. / 9.];
    let primal = evaluate(&row, &column, &objective, &joint, Config::default()).unwrap();
    assert!(primal.within_tolerance);
    encloses(primal.objective, 28. / 3.);
    let checked = certificate(&row, &column, &objective, &joint, &exact_dual(), Config::default()).unwrap();
    assert!(checked.dual_inequalities_nonnegative);
    assert!(checked.stationarity_error_upper < 1e-8);
    assert!(checked.objective_gap.lower > 1.);
    assert!(!checked.within_tolerance);
}

#[test]
fn product_of_ce_marginals_breaks_incentives() {
    let (row, column, objective) = gap_game();
    let primal = evaluate(&row, &column, &objective, &[1. / 16., 3. / 16., 3. / 16., 9. / 16.], Config::default()).unwrap();
    assert!(!primal.within_tolerance);
    assert!(primal.maximum_weighted_gain_upper[0] >= 3. / 16.);
    assert!(primal.maximum_weighted_gain_upper[1] >= 3. / 16.);
    for incentive in primal.incentives.iter().filter(|entry| entry.recommendation == 1) {
        encloses(incentive.conditional_gain.unwrap(), 0.25);
    }
}

#[test]
fn a_higher_objective_does_not_override_failed_incentives() {
    let (row, column, objective) = gap_game();
    let primal = evaluate(&row, &column, &objective, &[0., 0., 0., 1.], Config::default()).unwrap();
    encloses(primal.objective, 12.);
    assert!(primal.maximum_weighted_gain_upper.iter().all(|gain| *gain >= 1.));
    assert!(!primal.within_tolerance);
}

#[test]
fn asymmetric_column_deviations_use_their_own_payoff_matrix() {
    let primal = evaluate(
        &[vec![3., 0.], vec![1., 2.]], &[vec![1., 2.], vec![3., 0.]],
        &[vec![4., 2.], vec![4., 2.]], &[0.5, 0., 0., 0.5], Config::default(),
    ).unwrap();
    assert_eq!(primal.maximum_weighted_gain_upper[0], 0.);
    assert!(primal.maximum_weighted_gain_upper[1] >= 1.5);
    let gain = primal.incentives.iter().find(|entry| entry.player == Player::Column && entry.recommendation == 1).unwrap();
    encloses(gain.conditional_gain.unwrap(), 3.);
    assert!(!primal.within_tolerance);
}

#[test]
fn zero_mass_recommendations_have_no_conditional_ratio() {
    let primal = evaluate(
        &[vec![2., 2.], vec![1., 1.]], &[vec![2., 1.], vec![2., 1.]],
        &[vec![4., 3.], vec![3., 2.]], &[1., 0., 0., 0.], Config::default(),
    ).unwrap();
    for entry in primal.incentives.iter().filter(|entry| entry.recommendation == 1) {
        assert_eq!(entry.recommendation_mass, Interval { lower: 0., upper: 0. });
        assert!(entry.conditional_gain.is_none());
    }
}

#[test]
fn tiny_positive_recommendations_keep_large_conditional_gains_visible() {
    let primal = evaluate(
        &[vec![0., 0.], vec![2., 2.]], &[vec![0., 0.], vec![0., 0.]],
        &[vec![0., 0.], vec![0., 0.]], &[1e-20, 0., 0., 1.], Config::default(),
    ).unwrap();
    assert!(primal.within_tolerance);
    let gain = primal.incentives.iter().find(|entry| entry.player == Player::Row && entry.recommendation == 0).unwrap();
    assert!(gain.weighted_gain.upper < 1e-8);
    encloses(gain.conditional_gain.unwrap(), 2.);
}

#[test]
fn raw_negative_approximation_is_reported_without_projection() {
    let (row, column, objective) = gap_game();
    let joint = [-1e-12, 0.25, 0.25, 0.500_000_000_001];
    let bits = joint.map(f64::to_bits);
    let primal = evaluate(&row, &column, &objective, &joint, Config::default()).unwrap();
    assert_eq!(joint.map(f64::to_bits), bits);
    assert_eq!(primal.maximum_negative_weight, 1e-12);
    assert!(primal.within_tolerance);
}

#[test]
fn invalid_probability_mass_and_negative_weight_are_not_normalized() {
    let (row, column, objective) = gap_game();
    for joint in [[-1., 0., 0., 2.], [1., 0., 0., 1.], [0.; 4]] {
        let original = joint;
        let primal = evaluate(&row, &column, &objective, &joint, Config::default()).unwrap();
        assert!(!primal.within_tolerance);
        assert_eq!(joint, original);
    }
}

#[test]
fn exact_analytic_primal_dual_certificate_is_accepted_numerically() {
    let (row, column, objective) = gap_game();
    let checked = certificate(&row, &column, &objective, &[0., 0.25, 0.25, 0.5], &exact_dual(), Config::default()).unwrap();
    assert!(checked.within_tolerance);
    for residual in checked.stationarity { encloses(residual, 0.); }
    encloses(checked.objective_gap, 0.);
}

#[test]
fn negative_incentive_multiplier_rejects_even_with_correct_stationarity_and_gap() {
    let (row, column, objective) = gap_game();
    let dual = Dual { mass: 10.5, nonnegativity: vec![8.5, 1., 0., 0.], incentives: vec![-1., 0.75, 0., 0.75] };
    let checked = certificate(&row, &column, &objective, &[0., 0.25, 0.25, 0.5], &dual, Config::default()).unwrap();
    assert!(checked.stationarity_error_upper < 1e-8);
    assert!(checked.objective_gap_abs_upper < 1e-8);
    assert!(!checked.dual_inequalities_nonnegative);
    assert!(!checked.within_tolerance);
}

#[test]
fn incorrect_stationarity_is_not_a_valid_certificate() {
    let (row, column, objective) = gap_game();
    let mut dual = exact_dual();
    dual.nonnegativity[0] = 9.5;
    let checked = certificate(&row, &column, &objective, &[0., 0.25, 0.25, 0.5], &dual, Config::default()).unwrap();
    assert!(checked.stationarity_error_upper >= 1.);
    assert!(!checked.within_tolerance);
}

#[test]
fn feasible_dual_with_a_loose_objective_bound_fails_the_gap_check() {
    let (row, column, objective) = gap_game();
    let dual = Dual { mass: 11.5, nonnegativity: vec![11.5, 1., 1., 1.], incentives: vec![0., 0.75, 0., 0.75] };
    let checked = certificate(&row, &column, &objective, &[0., 0.25, 0.25, 0.5], &dual, Config::default()).unwrap();
    assert!(checked.dual_inequalities_nonnegative);
    assert!(checked.stationarity_error_upper < 1e-8);
    assert!(checked.objective_gap.lower > 0.99);
    assert!(!checked.within_tolerance);
}

#[test]
fn catastrophic_cancellation_cannot_certify_a_positive_gain_as_zero() {
    let large = 2_f64.powi(54);
    let primal = evaluate(
        &[vec![0., 0., 0.], vec![large, 1., -large]],
        &[vec![0.; 3], vec![0.; 3]], &[vec![0.; 3], vec![0.; 3]],
        &[0.25, 0.5, 0.25, 0., 0., 0.], Config::default(),
    ).unwrap();
    let gain = &primal.incentives[0];
    encloses(gain.weighted_gain, 0.5);
    assert!(gain.weighted_gain.upper >= 0.5);
    assert!(!primal.within_tolerance);
}

#[test]
fn underflowed_positive_gain_is_not_an_exact_zero_certificate() {
    let config = Config { incentive_tolerance: [0.; 2], ..Config::default() };
    let primal = evaluate(
        &[vec![0., 0.], vec![f64::from_bits(1), 0.]],
        &[vec![0.; 2], vec![0.; 2]], &[vec![0.; 2], vec![0.; 2]],
        &[0.5, 0.5, 0., 0.], config,
    ).unwrap();
    assert!(primal.maximum_weighted_gain_upper[0] > 0.);
    assert!(!primal.within_tolerance);
}

#[test]
fn finite_payoffs_with_unrepresentable_differences_fail_explicitly() {
    assert!(matches!(solve(
        &[vec![f64::MAX], vec![-f64::MAX]], &[vec![0.], vec![0.]],
        &[vec![0.], vec![0.]], Config::default(),
    ), Err(Error::Arithmetic { .. })));
}

#[test]
fn zero_and_one_iteration_budgets_do_not_return_accepted_candidates() {
    let (row, column, objective) = gap_game();
    for budget in [0, 1] {
        let record = solve(&row, &column, &objective, Config { max_iterations: budget, ..Config::default() }).unwrap();
        assert_eq!(record.termination(), Termination::MaxIterations);
        assert_eq!(record.status(), Status::BackendTerminated);
        assert!(record.accepted_candidate().is_none());
        assert!(record.candidate().is_none());
        assert!(record.backend().iterations <= budget);
    }
}

#[test]
fn solved_status_cannot_override_zero_admission_tolerances() {
    let (row, column, objective) = gap_game();
    let config = Config { probability_tolerance: 0., incentive_tolerance: [0.; 2], dual_tolerance: 0., objective_tolerance: 0., ..Config::default() };
    let record = solve(&row, &column, &objective, config).unwrap();
    assert_eq!(record.termination(), Termination::Solved);
    assert_eq!(record.status(), Status::ResidualRejected);
    assert!(record.accepted_candidate().is_none());
    assert!(record.candidate().is_some());
}

#[test]
fn empty_ragged_mismatched_and_excessive_games_are_rejected() {
    let config = Config::default();
    assert!(solve(&[], &[], &[], config).is_err());
    assert!(solve(&[vec![]], &[vec![]], &[vec![]], config).is_err());
    assert!(solve(&[vec![0.], vec![0., 1.]], &[vec![0.], vec![0.]], &[vec![0.], vec![0.]], config).is_err());
    assert!(solve(&[vec![0.]], &[vec![0.], vec![1.]], &[vec![0.]], config).is_err());
    assert!(solve(&[vec![0.]], &[vec![0.]], &[vec![0., 1.]], config).is_err());
    let oversized = vec![vec![0.; 9]; 1];
    assert!(solve(&oversized, &oversized, &oversized, config).is_err());
}

#[test]
fn every_nonfinite_input_role_is_rejected() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let finite = [vec![0.]];
        let bad = [vec![value]];
        assert!(solve(&bad, &finite, &finite, Config::default()).is_err());
        assert!(solve(&finite, &bad, &finite, Config::default()).is_err());
        assert!(solve(&finite, &finite, &bad, Config::default()).is_err());
        assert!(evaluate(&finite, &finite, &finite, &[value], Config::default()).is_err());
    }
}

#[test]
fn candidate_and_certificate_dimensions_are_checked() {
    let (row, column, objective) = gap_game();
    assert!(evaluate(&row, &column, &objective, &[1.], Config::default()).is_err());
    let mut dual = exact_dual();
    dual.incentives.pop();
    assert!(certificate(&row, &column, &objective, &[0., 0.25, 0.25, 0.5], &dual, Config::default()).is_err());
    dual = exact_dual();
    dual.nonnegativity[0] = f64::NAN;
    assert!(certificate(&row, &column, &objective, &[0., 0.25, 0.25, 0.5], &dual, Config::default()).is_err());
}

#[test]
fn invalid_tolerances_and_excessive_iteration_budget_are_rejected() {
    let (row, column, objective) = gap_game();
    for bad in [-1., f64::NAN, f64::INFINITY] {
        for config in [
            Config { probability_tolerance: bad, ..Config::default() },
            Config { incentive_tolerance: [bad, 0.], ..Config::default() },
            Config { incentive_tolerance: [0., bad], ..Config::default() },
            Config { dual_tolerance: bad, ..Config::default() },
            Config { objective_tolerance: bad, ..Config::default() },
        ] { assert!(solve(&row, &column, &objective, config).is_err()); }
    }
    assert!(solve(&row, &column, &objective, Config { probability_tolerance: 1., ..Config::default() }).is_err());
    assert!(solve(&row, &column, &objective, Config { max_iterations: u32::MAX, ..Config::default() }).is_err());
}

#[test]
fn signed_zero_configuration_and_raw_joint_bits_are_preserved() {
    let config = Config { incentive_tolerance: [-0., -0.], ..Config::default() };
    let row = [vec![0., 0.]];
    let joint = [-0., 1.];
    let original = joint.map(f64::to_bits);
    let primal = evaluate(&row, &row, &row, &joint, config).unwrap();
    assert!(primal.within_tolerance);
    assert_eq!(joint.map(f64::to_bits), original);
    let record = solve(&[vec![0.]], &[vec![0.]], &[vec![0.]], config).unwrap();
    assert_eq!(record.config().incentive_tolerance[0].to_bits(), (-0_f64).to_bits());
}
