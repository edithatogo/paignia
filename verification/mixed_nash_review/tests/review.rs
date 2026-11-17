//! Independent review cases; expected values are analytic, not solver outputs.
use paignia_solve::{
    evaluate_mixed_nash_candidate, solve_mixed_nash, MixedNashAlgorithm as Algorithm,
    MixedNashCatalogueCompleteness as Catalogue, MixedNashConfig as Config,
    MixedNashError as Error, MixedNashResult as ResultRecord,
    MixedNashSearchCompleteness as Search, MixedNashStatus as Status,
};

type Matrix = Vec<Vec<f64>>;

fn solve(a: &Matrix, b: &Matrix, config: Config) -> ResultRecord {
    solve_mixed_nash(a, b, Algorithm::SupportEnumeration, config).unwrap()
}

fn near(actual: &[f64], expected: &[f64], error: f64) {
    assert_eq!(actual.len(), expected.len());
    for (a, e) in actual.iter().zip(expected) {
        assert!(
            a.is_finite() && (a - e).abs() <= error,
            "{actual:?} vs {expected:?}"
        );
    }
}

fn accounted(result: &ResultRecord) {
    assert_eq!(
        result.visited_support_pairs,
        result.candidates.len()
            + result.unresolved_support_pairs
            + result.non_positive_support_pairs
            + result.rejected_best_response_support_pairs
    );
}

#[test]
fn finite_cancellation_cannot_erase_a_profitable_deviation() {
    // p chooses row 1. At q=(1/4,1/2,1/4), deviating to row 0 pays
    // (1/4)*2^54 + (1/2)*1 - (1/4)*2^54 = 1/2 exactly.
    let large = 2.0_f64.powi(54);
    let a = vec![vec![large, 1.0, -large], vec![0.0; 3]];
    let b = vec![vec![0.0; 3]; 2];
    for order in [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ] {
        let permuted: Matrix = a
            .iter()
            .map(|r| order.iter().map(|&j| r[j]).collect())
            .collect();
        let q: Vec<_> = order.iter().map(|&j| [0.25, 0.5, 0.25][j]).collect();
        let candidate =
            evaluate_mixed_nash_candidate(&permuted, &b, &[0.0, 1.0], &q, Config::default())
                .unwrap();
        assert_eq!(
            candidate.diagnostics.best_response_gains[0].to_bits(),
            0.5_f64.to_bits(),
            "{order:?}"
        );
        assert!(!candidate.diagnostics.within_tolerance);
    }
}

#[test]
fn finite_cancellation_does_not_erase_an_expected_payoff() {
    let large = 2.0_f64.powi(54);
    let a = vec![vec![large, 1.0, -large]];
    let b = vec![vec![0.0; 3]];
    let candidate =
        evaluate_mixed_nash_candidate(&a, &b, &[1.0], &[0.25, 0.5, 0.25], Config::default())
            .unwrap();
    assert_eq!(
        candidate.diagnostics.expected_payoffs[0].to_bits(),
        0.5_f64.to_bits()
    );
    assert!(candidate.diagnostics.within_tolerance);
}

#[test]
fn rounded_payoff_differences_cannot_erase_a_profitable_deviation() {
    // Exact improvement is (H+1)/2-H/2=1/2, but H+1 is not a binary64 value.
    // A robust final sum alone cannot recover a subtraction rounded earlier.
    let large = 2.0_f64.powi(54);
    let a = vec![vec![large, -large], vec![-1.0, 0.0]];
    let b = vec![vec![0.0; 2]; 2];
    let candidate =
        evaluate_mixed_nash_candidate(&a, &b, &[0.0, 1.0], &[0.5, 0.5], Config::default()).unwrap();
    assert_eq!(
        candidate.diagnostics.best_response_gains[0].to_bits(),
        0.5_f64.to_bits()
    );
    assert!(!candidate.diagnostics.within_tolerance);
}

#[test]
fn rounded_products_cannot_erase_a_profitable_deviation() {
    // The exact weighted deviation is
    // -(1-e)*(1+e)*s + e*(s/e) = s*e^2 = 1/2.
    // Both individually rounded products would be -s and s.
    let epsilon = 2.0_f64.powi(-27);
    let scale = 2.0_f64.powi(53);
    let a = vec![
        vec![-(1.0 + epsilon) * scale, scale / epsilon],
        vec![0.0; 2],
    ];
    let b = vec![vec![0.0; 2]; 2];
    let candidate = evaluate_mixed_nash_candidate(
        &a,
        &b,
        &[0.0, 1.0],
        &[1.0 - epsilon, epsilon],
        Config::default(),
    )
    .unwrap();
    assert_eq!(
        candidate.diagnostics.best_response_gains[0].to_bits(),
        0.5_f64.to_bits()
    );
    assert!(!candidate.diagnostics.within_tolerance);
}

#[test]
fn an_unequal_support_equilibrium_is_not_a_no_equilibrium_conclusion() {
    // Every equilibrium uses p=(1,0,0) and q=(u,1-u), u in [1/3,2/3].
    // At u=1/2 row 0 yields 2 while alternatives yield 3/2. Columns tie.
    // No pure cell is an equilibrium. Every equal-size mixed support either
    // has a zero coordinate or permits the profitable deviation to row 0.
    let a = vec![vec![2.0, 2.0], vec![0.0, 3.0], vec![3.0, 0.0]];
    let b = vec![vec![0.0, 0.0], vec![1.0, 0.0], vec![0.0, 1.0]];
    let witness =
        evaluate_mixed_nash_candidate(&a, &b, &[1.0, 0.0, 0.0], &[0.5, 0.5], Config::default())
            .unwrap();
    near(&witness.diagnostics.expected_payoffs, &[2.0, 0.0], 0.0);
    near(&witness.diagnostics.best_response_gains, &[0.0, 0.0], 0.0);
    let result = solve(&a, &b, Config::default());
    assert_eq!(result.status, Status::NoVerifiedWitness);
    assert_eq!(
        result.support_search,
        Search::AllDeclaredSupportPairsVisited
    );
    assert_eq!(
        result.catalogue_completeness,
        Catalogue::FiniteVerifiedWitnessesOnly
    );
    assert_eq!(result.total_support_pairs, 9);
    assert!(result.candidates.is_empty());
    assert_eq!(result.unresolved_support_pairs, 0);
    assert_eq!(result.non_positive_support_pairs, 2);
    assert_eq!(result.rejected_best_response_support_pairs, 7);
    accounted(&result);
}

fn rectangular_rps() -> (Matrix, Matrix) {
    // The first 3x3 block is skew-symmetric weighted RPS with kernel (3,2,1).
    // Extra row is strictly dominated by row 0; extra columns are dominated.
    (
        vec![
            vec![0.0, -1.0, 2.0, 7.0, -4.0],
            vec![1.0, 0.0, -3.0, -2.0, 8.0],
            vec![-2.0, 3.0, 0.0, 5.0, 6.0],
            vec![-10.0; 5],
        ],
        vec![
            vec![0.0, 1.0, -2.0, -10.0, -10.0],
            vec![-1.0, 0.0, 3.0, -10.0, -10.0],
            vec![2.0, -3.0, 0.0, -10.0, -10.0],
            vec![0.0, 0.0, 0.0, -10.0, -10.0],
        ],
    )
}

#[test]
fn rectangular_three_action_interior_survives_independent_permutations() {
    let (a, b) = rectangular_rps();
    let expected_p = [0.5, 1.0 / 3.0, 1.0 / 6.0, 0.0];
    let expected_q = [0.5, 1.0 / 3.0, 1.0 / 6.0, 0.0, 0.0];
    for (rows, columns) in [
        ([0, 1, 2, 3], [0, 1, 2, 3, 4]),
        ([3, 1, 0, 2], [4, 2, 0, 3, 1]),
    ] {
        let permute = |matrix: &Matrix| -> Matrix {
            rows.iter()
                .map(|&i| columns.iter().map(|&j| matrix[i][j]).collect())
                .collect()
        };
        let result = solve(&permute(&a), &permute(&b), Config::default());
        assert_eq!(result.total_support_pairs, 125);
        assert_eq!(result.visited_support_pairs, 125);
        assert_eq!(result.candidates.len(), 1);
        let witness = &result.candidates[0];
        near(&witness.row_strategy, &rows.map(|i| expected_p[i]), 1e-12);
        near(
            &witness.column_strategy,
            &columns.map(|i| expected_q[i]),
            1e-12,
        );
        assert_eq!(witness.row_support.len(), 3);
        assert_eq!(witness.column_support.len(), 3);
        assert!(witness.diagnostics.within_tolerance);
        accounted(&result);
    }
}

#[test]
fn duplicated_actions_report_unresolved_continua_and_retain_finite_witnesses() {
    let a = vec![
        vec![1.0, -1.0, 1.0],
        vec![-1.0, 1.0, -1.0],
        vec![1.0, -1.0, 1.0],
    ];
    let b: Matrix = a.iter().map(|r| r.iter().map(|x| -x).collect()).collect();
    let witness = evaluate_mixed_nash_candidate(
        &a,
        &b,
        &[0.25, 0.5, 0.25],
        &[0.125, 0.5, 0.375],
        Config::default(),
    )
    .unwrap();
    near(&witness.diagnostics.best_response_gains, &[0.0, 0.0], 0.0);
    let result = solve(&a, &b, Config::default());
    assert_eq!(result.status, Status::UnresolvedSupports);
    assert_eq!(result.candidates.len(), 4);
    assert!(result.unresolved_support_pairs > 0);
    assert_eq!(
        result.catalogue_completeness,
        Catalogue::FiniteVerifiedWitnessesOnly
    );
    assert!(result
        .candidates
        .iter()
        .all(|c| c.row_support.len() == 2 && c.column_support.len() == 2));
    accounted(&result);
}

#[test]
fn the_full_eight_action_scope_is_visited_and_accounted_without_false_completeness() {
    let zero = vec![vec![0.0; 8]; 8];
    let result = solve(&zero, &zero, Config::default());
    assert_eq!(result.total_support_pairs, 12869);
    assert_eq!(result.visited_support_pairs, 12869);
    assert_eq!(result.candidates.len(), 64);
    assert_eq!(result.unresolved_support_pairs, 12805);
    assert_eq!(result.status, Status::UnresolvedSupports);
    assert_eq!(
        result.support_search,
        Search::AllDeclaredSupportPairsVisited
    );
    assert_eq!(
        result.catalogue_completeness,
        Catalogue::FiniteVerifiedWitnessesOnly
    );
    accounted(&result);
}

#[test]
fn tiny_positive_full_support_coordinates_are_not_tolerance_cutoffs() {
    let delta = 2.0_f64.powi(-40);
    let a = vec![
        vec![0.0, -1.0, 1.0],
        vec![1.0, 0.0, -delta],
        vec![-1.0, delta, 0.0],
    ];
    let b: Matrix = a.iter().map(|r| r.iter().map(|x| -x).collect()).collect();
    let config = Config {
        equilibrium_tolerance: 1e-14,
        ..Config::default()
    };
    let result = solve(&a, &b, config);
    let candidate = result
        .candidates
        .iter()
        .find(|c| c.row_support.len() == 3 && c.column_support.len() == 3)
        .unwrap();
    let expected = [
        delta / (2.0 + delta),
        1.0 / (2.0 + delta),
        1.0 / (2.0 + delta),
    ];
    near(&candidate.row_strategy, &expected, 2e-15);
    near(&candidate.column_strategy, &expected, 2e-15);
    assert!(
        candidate.row_strategy[0] > 0.0 && candidate.row_strategy[0] < config.probability_tolerance
    );
    assert!(
        candidate.column_strategy[0] > 0.0
            && candidate.column_strategy[0] < config.probability_tolerance
    );
}

#[test]
fn row_scaled_near_rank_loss_is_reported_as_unresolved() {
    for (delta, expect_full_support) in [(2.0_f64.powi(-20), true), (2.0_f64.powi(-45), false)] {
        // A is a weighted path Laplacian with kernel (1,1,1). For delta>0
        // the zero-sum game has the unique completely mixed uniform saddle.
        let a = vec![
            vec![1.0, -1.0, 0.0],
            vec![-1.0, 1.0 + delta, -delta],
            vec![0.0, -delta, delta],
        ];
        let b: Matrix = a.iter().map(|r| r.iter().map(|x| -x).collect()).collect();
        let result = solve(&a, &b, Config::default());
        let candidate = result
            .candidates
            .iter()
            .find(|c| c.row_support.len() == 3 && c.column_support.len() == 3);
        assert_eq!(candidate.is_some(), expect_full_support);
        if let Some(candidate) = candidate {
            near(&candidate.row_strategy, &[1.0 / 3.0; 3], 1e-8);
            near(&candidate.column_strategy, &[1.0 / 3.0; 3], 1e-8);
        } else {
            assert!(result.unresolved_support_pairs > 0);
            assert_eq!(result.status, Status::UnresolvedSupports);
        }
        assert_eq!(
            result.catalogue_completeness,
            Catalogue::FiniteVerifiedWitnessesOnly
        );
        accounted(&result);
    }
}

#[test]
fn a_positive_dyadic_gain_is_retained_below_admission_tolerance() {
    let delta = 2.0_f64.powi(-100);
    let a = vec![vec![0.0, 0.0], vec![delta, delta]];
    let b = vec![vec![0.0; 2]; 2];
    for tolerance in [0.0, 1e-10] {
        let candidate = evaluate_mixed_nash_candidate(
            &a,
            &b,
            &[0.5, 0.5],
            &[0.25, 0.75],
            Config {
                equilibrium_tolerance: tolerance,
                ..Config::default()
            },
        )
        .unwrap();
        assert_eq!(
            candidate.diagnostics.best_response_gains[0].to_bits(),
            (delta / 2.0).to_bits()
        );
        assert_eq!(candidate.diagnostics.within_tolerance, tolerance > 0.0);
    }
}

#[test]
fn unsupported_algorithm_and_invalid_config_have_no_silent_search_fallback() {
    let (a, b) = rectangular_rps();
    let result = solve_mixed_nash(&a, &b, Algorithm::LemkeHowson, Config::default()).unwrap();
    assert_eq!(result.status, Status::UnsupportedAlgorithm);
    assert_eq!(result.support_search, Search::NotRun);
    assert_eq!(result.catalogue_completeness, Catalogue::NotClaimed);
    assert_eq!(result.algorithm_version, None);
    assert_eq!(result.scope, None);
    assert_eq!(result.visited_support_pairs, 0);
    assert!(result.candidates.is_empty());
    let nan = f64::from_bits(0xfff8_0000_0000_0123);
    let error = solve_mixed_nash(
        &a,
        &b,
        Algorithm::LemkeHowson,
        Config {
            probability_tolerance: nan,
            ..Config::default()
        },
    )
    .unwrap_err();
    assert!(
        matches!(error, Error::InvalidTolerance { field:"probability",value } if value.to_bits()==nan.to_bits())
    );
}

#[test]
fn exact_product_range_guard_distinguishes_the_subnormal_boundary() {
    // Multiplication by 1/2 is exactly representable for even subnormal bit
    // counts. Odd counts have an exact low bit below 2^-1074 and must reject,
    // including those whose main rounded product is nonzero.
    for bits in 1..=6_u64 {
        for sign in [-1.0, 1.0] {
            let payoff = sign * f64::from_bits(bits);
            let result = evaluate_mixed_nash_candidate(
                &[vec![payoff, 0.0]],
                &[vec![0.0, 0.0]],
                &[1.0],
                &[0.5, 0.5],
                Config::default(),
            );
            if bits % 2 == 0 {
                let candidate = result.unwrap();
                assert_eq!(
                    candidate.diagnostics.expected_payoffs[0].to_bits(),
                    (sign * f64::from_bits(bits / 2)).to_bits()
                );
            } else {
                assert!(matches!(result, Err(Error::ArithmeticFailure { .. })));
            }
        }
    }
}

#[test]
fn mass_residual_is_measured_before_rounding_the_total_to_one() {
    let delta = 2.0_f64.powi(-56);
    let a = vec![vec![0.0], vec![0.0]];
    let candidate =
        evaluate_mixed_nash_candidate(&a, &a, &[1.0, delta], &[1.0], Config::default()).unwrap();
    assert_eq!(
        candidate.diagnostics.probability_mass_errors[0].to_bits(),
        delta.to_bits()
    );
    assert_eq!(candidate.row_strategy[1].to_bits(), delta.to_bits());
    let error = evaluate_mixed_nash_candidate(
        &a,
        &a,
        &[1.0, delta],
        &[1.0],
        Config {
            probability_tolerance: 0.0,
            ..Config::default()
        },
    )
    .unwrap_err();
    assert!(
        matches!(error, Error::InvalidProbabilityMass { mass_error, .. } if mass_error.to_bits()==delta.to_bits())
    );
}
