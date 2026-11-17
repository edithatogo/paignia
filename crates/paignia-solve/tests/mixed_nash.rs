//! Original rational controls copied from the independently frozen fixture:
//! `verification/numerical_oracles/mixed_nash_v1.json`, SHA-256
//! 9545116fc6aa4b1026bf6b2a6a9b509af92d8ed6ca2204e68bcdccf8e2e126e3.
//! Constants are local so extracted Cargo packages run these tests without a
//! repository-relative fixture, Python runtime, or a second production solver.
#![allow(clippy::float_cmp)]

use paignia_solve::{
    evaluate_mixed_nash_candidate, solve_mixed_nash, MixedNashAlgorithm as Algorithm,
    MixedNashCandidate as Candidate, MixedNashCatalogueCompleteness as Catalogue,
    MixedNashConfig as Config, MixedNashError as Error, MixedNashGameClass as GameClass,
    MixedNashPlayer as Player, MixedNashResult as ResultRecord,
    MixedNashSearchCompleteness as Search, MixedNashSelection as Selection,
    MixedNashSolutionConcept as Concept, MixedNashStatus as Status, MAX_MIXED_NASH_ACTIONS,
    SUPPORT_ENUMERATION_VERSION,
};

type Matrix = Vec<Vec<f64>>;

struct Witness {
    p: Vec<f64>,
    q: Vec<f64>,
    utilities: [f64; 2],
}

struct Oracle {
    name: &'static str,
    a: Matrix,
    b: Matrix,
    witnesses: Vec<Witness>,
    finite_catalogue: bool,
}

fn witness(p: &[f64], q: &[f64], utilities: [f64; 2]) -> Witness {
    Witness {
        p: p.to_vec(),
        q: q.to_vec(),
        utilities,
    }
}

fn oracles() -> Vec<Oracle> {
    vec![
        Oracle {
            name: "strict dominance",
            a: vec![vec![2.0, 2.0], vec![1.0, 1.0]],
            b: vec![vec![2.0, 1.0], vec![2.0, 1.0]],
            witnesses: vec![witness(&[1.0, 0.0], &[1.0, 0.0], [2.0, 2.0])],
            finite_catalogue: true,
        },
        Oracle {
            name: "matching pennies",
            a: vec![vec![1.0, -1.0], vec![-1.0, 1.0]],
            b: vec![vec![-1.0, 1.0], vec![1.0, -1.0]],
            witnesses: vec![witness(&[0.5, 0.5], &[0.5, 0.5], [0.0, 0.0])],
            finite_catalogue: true,
        },
        Oracle {
            name: "coordination",
            a: vec![vec![2.0, 0.0], vec![0.0, 1.0]],
            b: vec![vec![1.0, 0.0], vec![0.0, 2.0]],
            witnesses: vec![
                witness(&[1.0, 0.0], &[1.0, 0.0], [2.0, 1.0]),
                witness(&[0.0, 1.0], &[0.0, 1.0], [1.0, 2.0]),
                witness(
                    &[2.0 / 3.0, 1.0 / 3.0],
                    &[1.0 / 3.0, 2.0 / 3.0],
                    [2.0 / 3.0; 2],
                ),
            ],
            finite_catalogue: true,
        },
        Oracle {
            name: "asymmetric",
            a: vec![vec![3.0, 0.0], vec![1.0, 2.0]],
            b: vec![vec![1.0, 2.0], vec![3.0, 0.0]],
            witnesses: vec![witness(&[0.75, 0.25], &[0.5, 0.5], [1.5, 1.5])],
            finite_catalogue: true,
        },
        Oracle {
            name: "rectangular 2x3",
            a: vec![vec![1.0, -1.0, 0.0], vec![-1.0, 1.0, 0.0]],
            b: vec![vec![-1.0, 1.0, -2.0], vec![1.0, -1.0, -2.0]],
            witnesses: vec![witness(&[0.5, 0.5], &[0.5, 0.5, 0.0], [0.0, 0.0])],
            finite_catalogue: true,
        },
        Oracle {
            name: "rectangular 3x2",
            a: vec![vec![-1.0, 1.0], vec![1.0, -1.0], vec![-2.0, -2.0]],
            b: vec![vec![1.0, -1.0], vec![-1.0, 1.0], vec![0.0, 0.0]],
            witnesses: vec![witness(&[0.5, 0.5, 0.0], &[0.5, 0.5], [0.0, 0.0])],
            finite_catalogue: true,
        },
        Oracle {
            name: "zero continuum",
            a: vec![vec![0.0; 2]; 2],
            b: vec![vec![0.0; 2]; 2],
            witnesses: vec![
                witness(&[1.0 / 3.0, 2.0 / 3.0], &[2.0 / 5.0, 3.0 / 5.0], [0.0, 0.0]),
                witness(&[1.0, 0.0], &[0.0, 1.0], [0.0, 0.0]),
                witness(&[0.0, 1.0], &[1.0, 0.0], [0.0, 0.0]),
            ],
            finite_catalogue: false,
        },
        Oracle {
            name: "signed single action",
            a: vec![vec![-7.0]],
            b: vec![vec![3.0]],
            witnesses: vec![witness(&[1.0], &[1.0], [-7.0, 3.0])],
            finite_catalogue: true,
        },
    ]
}

fn solve(a: &[Vec<f64>], b: &[Vec<f64>]) -> ResultRecord {
    solve_mixed_nash(a, b, Algorithm::SupportEnumeration, Config::default()).unwrap()
}

fn near(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        actual.is_finite() && (actual - expected).abs() <= tolerance,
        "actual {actual:.17e}, expected {expected:.17e}, tolerance {tolerance}"
    );
}

fn same_strategy(actual: &[f64], expected: &[f64]) -> bool {
    actual.len() == expected.len()
        && actual
            .iter()
            .zip(expected)
            .all(|(a, b)| (a - b).abs() < 1e-12)
}

fn find<'a>(candidates: &'a [Candidate], p: &[f64], q: &[f64]) -> &'a Candidate {
    candidates
        .iter()
        .find(|c| same_strategy(&c.row_strategy, p) && same_strategy(&c.column_strategy, q))
        .unwrap_or_else(|| panic!("missing witness p={p:?}, q={q:?}; got {candidates:?}"))
}

#[test]
fn enumerates_all_isolated_frozen_examples_without_borrowing_fixture_completeness() {
    for oracle in oracles().into_iter().filter(|o| o.finite_catalogue) {
        let result = solve(&oracle.a, &oracle.b);
        assert_eq!(
            result.candidates.len(),
            oracle.witnesses.len(),
            "{}",
            oracle.name
        );
        for w in oracle.witnesses {
            let c = find(&result.candidates, &w.p, &w.q);
            for (actual, expected) in c.diagnostics.expected_payoffs.iter().zip(w.utilities) {
                near(*actual, expected, 1e-12);
            }
            assert!(c.diagnostics.within_tolerance);
            assert!(c.diagnostics.max_gain <= result.config.equilibrium_tolerance);
            assert!(c.support_equation_residual.unwrap() <= result.config.pivot_tolerance);
        }
        assert_eq!(
            result.catalogue_completeness,
            Catalogue::FiniteVerifiedWitnessesOnly
        );
    }
}

#[test]
fn evaluates_all_twelve_independent_exact_witnesses_including_continuum_samples() {
    let mut executed = 0;
    for oracle in oracles() {
        for w in oracle.witnesses {
            let c =
                evaluate_mixed_nash_candidate(&oracle.a, &oracle.b, &w.p, &w.q, Config::default())
                    .unwrap();
            near(c.diagnostics.max_gain, 0.0, 1e-15);
            near(c.diagnostics.sum_gain, 0.0, 2e-15);
            assert!(c.diagnostics.within_tolerance);
            assert_eq!(c.support_equation_residual, None);
            executed += 1;
        }
    }
    assert_eq!(executed, 12);
}

#[test]
fn exposes_each_frozen_false_candidate_gain() {
    let fixtures = oracles();
    let negatives = [
        (0, vec![0.5, 0.5], vec![0.5, 0.5], [0.5, 0.5]),
        (1, vec![0.75, 0.25], vec![0.5, 0.5], [0.0, 0.5]),
        (2, vec![0.5, 0.5], vec![0.5, 0.5], [0.25, 0.25]),
        (3, vec![0.5, 0.5], vec![0.5, 0.5], [0.0, 0.5]),
        (4, vec![0.5, 0.5], vec![0.5, 0.0, 0.5], [0.5, 1.0]),
    ];
    for (index, p, q, gains) in negatives {
        let o = &fixtures[index];
        let c = evaluate_mixed_nash_candidate(&o.a, &o.b, &p, &q, Config::default()).unwrap();
        assert!(!c.diagnostics.within_tolerance);
        assert_eq!(c.diagnostics.best_response_gains, gains);
        assert_eq!(c.diagnostics.sum_gain, gains[0] + gains[1]);
        assert_eq!(c.diagnostics.max_gain, gains[0].max(gains[1]));
    }
}

#[test]
fn strict_dominance_counterexample_has_no_half_half_fallback() {
    let o = oracles().remove(0);
    let result = solve(&o.a, &o.b);
    assert_eq!(result.candidates.len(), 1);
    assert_eq!(result.candidates[0].row_strategy, vec![1.0, 0.0]);
    assert_eq!(result.candidates[0].column_strategy, vec![1.0, 0.0]);
    assert_eq!(
        result.candidates[0].diagnostics.best_response_gains,
        [0.0, 0.0]
    );
    assert_eq!(result.unresolved_support_pairs, 1);
}

#[test]
fn matching_pennies_reports_a_completed_bounded_search() {
    let o = oracles().remove(1);
    let result = solve(&o.a, &o.b);
    assert_eq!(result.status, Status::CompletedWithinScope);
    assert_eq!(
        result.support_search,
        Search::AllDeclaredSupportPairsVisited
    );
    assert_eq!(result.visited_support_pairs, 5);
    assert_eq!(result.total_support_pairs, 5);
    assert_eq!(result.rejected_best_response_support_pairs, 4);
    assert_eq!(result.unresolved_support_pairs, 0);
    assert_eq!(result.algorithm_version, Some(SUPPORT_ENUMERATION_VERSION));
    assert_eq!(result.algorithm.id(), "paignia/equal-support-enumeration");
    assert_eq!(
        result.game_class,
        GameClass::TwoPlayerFiniteCompleteInformationBimatrix
    );
    assert_eq!(result.solution_concept, Concept::MixedNash);
}

#[test]
fn coordination_retains_both_pure_and_interior_witnesses_in_declared_order() {
    let o = oracles().remove(2);
    let result = solve(&o.a, &o.b);
    assert_eq!(
        result.selection,
        Selection::AllVerifiedInCardinalityThenLexicographicSupportOrder
    );
    assert_eq!(result.candidates[0].row_support, vec![0]);
    assert_eq!(result.candidates[1].row_support, vec![1]);
    assert_eq!(result.candidates[2].row_support, vec![0, 1]);
    assert_eq!(result.candidates[2].column_support, vec![0, 1]);
}

#[test]
fn zero_game_does_not_turn_four_pure_witnesses_into_an_exhaustive_continuum() {
    let z = vec![vec![0.0; 2]; 2];
    let result = solve(&z, &z);
    assert_eq!(result.status, Status::UnresolvedSupports);
    assert_eq!(result.unresolved_support_pairs, 1);
    assert_eq!(result.candidates.len(), 4);
    assert_eq!(
        result.catalogue_completeness,
        Catalogue::FiniteVerifiedWitnessesOnly
    );
    assert_eq!(
        result.support_search,
        Search::AllDeclaredSupportPairsVisited
    );
    let scope = result.scope.unwrap();
    assert!(scope.equal_cardinality_only);
    assert!(!scope.singular_systems_supported);
    assert_eq!(scope.maximum_support_cardinality, 2);
}

#[test]
fn single_action_game_preserves_negative_utility() {
    let result = solve(&[vec![-7.0]], &[vec![3.0]]);
    assert_eq!(result.total_support_pairs, 1);
    assert_eq!(
        result.candidates[0].diagnostics.expected_payoffs,
        [-7.0, 3.0]
    );
    assert_eq!(result.candidates[0].diagnostics.sum_gain, 0.0);
}

#[test]
fn rectangular_games_search_all_equal_support_pairs_and_exclude_dominated_action() {
    for index in [4, 5] {
        let o = &oracles()[index];
        let result = solve(&o.a, &o.b);
        assert_eq!(result.total_support_pairs, 9);
        assert_eq!(result.visited_support_pairs, 9);
        let c = &result.candidates[0];
        assert_eq!(c.row_support, vec![0, 1]);
        assert_eq!(c.column_support, vec![0, 1]);
    }
}

#[test]
fn action_permutation_maps_witnesses_and_gains_back_to_the_same_game() {
    for index in [2, 3, 4, 5] {
        let o = &oracles()[index];
        let reverse = |m: &[Vec<f64>]| -> Matrix {
            m.iter()
                .rev()
                .map(|r| r.iter().rev().copied().collect())
                .collect()
        };
        let result = solve(&reverse(&o.a), &reverse(&o.b));
        for w in &o.witnesses {
            let p: Vec<_> = w.p.iter().rev().copied().collect();
            let q: Vec<_> = w.q.iter().rev().copied().collect();
            let c = find(&result.candidates, &p, &q);
            assert!(c.diagnostics.within_tolerance);
        }
    }
}

fn transpose(m: &[Vec<f64>]) -> Matrix {
    (0..m[0].len())
        .map(|j| m.iter().map(|r| r[j]).collect())
        .collect()
}

#[test]
fn exchanging_players_swaps_strategies_and_utilities() {
    for index in [2, 3, 4, 5] {
        let o = &oracles()[index];
        let result = solve(&transpose(&o.b), &transpose(&o.a));
        for w in &o.witnesses {
            let c = find(&result.candidates, &w.q, &w.p);
            near(c.diagnostics.expected_payoffs[0], w.utilities[1], 1e-12);
            near(c.diagnostics.expected_payoffs[1], w.utilities[0], 1e-12);
        }
    }
}

#[test]
fn positive_affine_payoff_transforms_preserve_witnesses_and_scale_gains() {
    let o = oracles().remove(2);
    let transform = |m: &[Vec<f64>], scale: f64, shift: f64| -> Matrix {
        m.iter()
            .map(|row| row.iter().map(|v| v * scale + shift).collect())
            .collect()
    };
    let a = transform(&o.a, 3.0, -11.0);
    let b = transform(&o.b, 7.0, 13.0);
    let result = solve(&a, &b);
    for w in &o.witnesses {
        find(&result.candidates, &w.p, &w.q);
    }
    let c =
        evaluate_mixed_nash_candidate(&a, &b, &[0.5, 0.5], &[0.5, 0.5], Config::default()).unwrap();
    assert_eq!(c.diagnostics.best_response_gains, [0.75, 1.75]);
}

#[test]
fn tiny_positive_gain_remains_visible_even_when_tolerance_admits_it() {
    let a = vec![vec![0.0], vec![2e-30]];
    let b = vec![vec![0.0], vec![0.0]];
    let admitted =
        evaluate_mixed_nash_candidate(&a, &b, &[1.0, 0.0], &[1.0], Config::default()).unwrap();
    assert_eq!(admitted.diagnostics.best_response_gains, [2e-30, 0.0]);
    assert!(admitted.diagnostics.within_tolerance);
    let exact = Config {
        equilibrium_tolerance: 0.0,
        ..Config::default()
    };
    let rejected = evaluate_mixed_nash_candidate(&a, &b, &[1.0, 0.0], &[1.0], exact).unwrap();
    assert_eq!(rejected.diagnostics.max_gain, 2e-30);
    assert!(!rejected.diagnostics.within_tolerance);
    let result = solve_mixed_nash(&a, &b, Algorithm::SupportEnumeration, exact).unwrap();
    assert_eq!(result.candidates.len(), 1);
    assert_eq!(result.candidates[0].row_strategy, vec![0.0, 1.0]);
}

#[test]
fn a_large_common_payoff_shift_does_not_erase_deviation_differences() {
    let a = vec![vec![1e15 + 2.0, 1e15], vec![1e15, 1e15 + 1.0]];
    let b = vec![vec![-1e15 + 1.0, -1e15], vec![-1e15, -1e15 + 2.0]];
    let c =
        evaluate_mixed_nash_candidate(&a, &b, &[0.5, 0.5], &[0.5, 0.5], Config::default()).unwrap();
    assert_eq!(c.diagnostics.best_response_gains, [0.25, 0.25]);
}

#[test]
fn exhausted_budget_retains_verified_witnesses_and_reports_partial_search() {
    let o = oracles().remove(0);
    let config = Config {
        max_support_pairs: 1,
        ..Config::default()
    };
    let result = solve_mixed_nash(&o.a, &o.b, Algorithm::SupportEnumeration, config).unwrap();
    assert_eq!(result.status, Status::BudgetExhausted);
    assert_eq!(result.support_search, Search::BudgetLimited);
    assert_eq!(result.visited_support_pairs, 1);
    assert_eq!(result.total_support_pairs, 5);
    assert_eq!(result.candidates.len(), 1);
    assert_eq!(result.config, config);
}

#[test]
fn no_witness_before_budget_is_not_no_equilibrium() {
    let o = oracles().remove(1);
    let config = Config {
        max_support_pairs: 4,
        ..Config::default()
    };
    let result = solve_mixed_nash(&o.a, &o.b, Algorithm::SupportEnumeration, config).unwrap();
    assert!(result.candidates.is_empty());
    assert_eq!(result.status, Status::BudgetExhausted);
    assert_eq!(result.total_support_pairs, 5);
}

#[test]
fn lemke_howson_is_explicitly_unsupported_without_substitution() {
    let o = oracles().remove(1);
    let result = solve_mixed_nash(&o.a, &o.b, Algorithm::LemkeHowson, Config::default()).unwrap();
    assert_eq!(result.algorithm, Algorithm::LemkeHowson);
    assert_eq!(result.algorithm_version, None);
    assert_eq!(result.status, Status::UnsupportedAlgorithm);
    assert_eq!(result.support_search, Search::NotRun);
    assert_eq!(result.catalogue_completeness, Catalogue::NotClaimed);
    assert_eq!(result.scope, None);
    assert_eq!(result.visited_support_pairs, 0);
    assert!(result.candidates.is_empty());
}

#[test]
fn maximum_declared_game_has_an_exact_finite_support_budget() {
    assert_eq!(MAX_MIXED_NASH_ACTIONS, 8);
    let z = vec![vec![0.0; 8]; 8];
    let config = Config {
        max_support_pairs: 65,
        ..Config::default()
    };
    let result = solve_mixed_nash(&z, &z, Algorithm::SupportEnumeration, config).unwrap();
    assert_eq!(result.total_support_pairs, 12_869);
    assert_eq!(result.visited_support_pairs, 65);
    assert_eq!(result.candidates.len(), 64);
    assert_eq!(result.unresolved_support_pairs, 1);
    assert_eq!(result.status, Status::BudgetExhausted);
}

#[test]
fn oversized_dimensions_fail_before_search() {
    for (rows, columns) in [(9, 1), (1, 9), (9, 9)] {
        let a = vec![vec![0.0; columns]; rows];
        assert!(matches!(
            solve_mixed_nash(&a, &a, Algorithm::SupportEnumeration, Config::default()),
            Err(Error::ActionLimit { limit: 8, .. })
        ));
    }
}

#[test]
fn malformed_matrices_are_not_reinterpreted_as_short_or_sparse_games() {
    assert!(matches!(
        solve_mixed_nash(&[], &[], Algorithm::SupportEnumeration, Config::default()),
        Err(Error::EmptyGame)
    ));
    assert!(matches!(
        solve_mixed_nash(
            &[vec![]],
            &[vec![]],
            Algorithm::SupportEnumeration,
            Config::default()
        ),
        Err(Error::EmptyGame)
    ));
    assert!(matches!(
        solve_mixed_nash(
            &[vec![1.0], vec![]],
            &[vec![1.0], vec![1.0]],
            Algorithm::SupportEnumeration,
            Config::default()
        ),
        Err(Error::InvalidShape {
            player: Player::Row,
            row: 1,
            ..
        })
    ));
    assert!(matches!(
        solve_mixed_nash(
            &[vec![1.0]],
            &[vec![1.0, 2.0]],
            Algorithm::SupportEnumeration,
            Config::default()
        ),
        Err(Error::InvalidShape {
            player: Player::Column,
            ..
        })
    ));
    assert!(matches!(
        solve_mixed_nash(
            &[vec![1.0]],
            &[vec![1.0], vec![1.0]],
            Algorithm::SupportEnumeration,
            Config::default()
        ),
        Err(Error::RowCountMismatch { .. })
    ));
}

#[test]
fn nonfinite_payoffs_fail_for_each_player_and_algorithm() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for algorithm in [Algorithm::SupportEnumeration, Algorithm::LemkeHowson] {
            assert!(matches!(
                solve_mixed_nash(&[vec![bad]], &[vec![0.0]], algorithm, Config::default()),
                Err(Error::NonFinitePayoff {
                    player: Player::Row,
                    ..
                })
            ));
            assert!(matches!(
                solve_mixed_nash(&[vec![0.0]], &[vec![bad]], algorithm, Config::default()),
                Err(Error::NonFinitePayoff {
                    player: Player::Column,
                    ..
                })
            ));
        }
    }
}

#[test]
fn invalid_tolerances_and_zero_budget_are_typed_errors() {
    for bad in [-1.0, f64::NAN, f64::INFINITY, 1.0] {
        for config in [
            Config {
                probability_tolerance: bad,
                ..Config::default()
            },
            Config {
                pivot_tolerance: bad,
                ..Config::default()
            },
        ] {
            assert!(matches!(
                solve_mixed_nash(
                    &[vec![0.0]],
                    &[vec![0.0]],
                    Algorithm::SupportEnumeration,
                    config
                ),
                Err(Error::InvalidTolerance { .. })
            ));
        }
    }
    for bad in [-1.0, f64::NAN, f64::INFINITY] {
        let config = Config {
            equilibrium_tolerance: bad,
            ..Config::default()
        };
        assert!(matches!(
            solve_mixed_nash(
                &[vec![0.0]],
                &[vec![0.0]],
                Algorithm::SupportEnumeration,
                config
            ),
            Err(Error::InvalidTolerance { .. })
        ));
    }
    let config = Config {
        max_support_pairs: 0,
        ..Config::default()
    };
    assert!(matches!(
        solve_mixed_nash(
            &[vec![0.0]],
            &[vec![0.0]],
            Algorithm::SupportEnumeration,
            config
        ),
        Err(Error::ZeroSupportBudget)
    ));
}

#[test]
fn invalid_probabilities_are_not_clipped() {
    let z = vec![vec![0.0; 2]; 2];
    for bad in [-1e-20, 1.0 + f64::EPSILON, f64::NAN, f64::INFINITY] {
        assert!(matches!(
            evaluate_mixed_nash_candidate(&z, &z, &[bad, 0.0], &[0.5, 0.5], Config::default()),
            Err(Error::InvalidProbability {
                player: Player::Row,
                index: 0,
                ..
            })
        ));
        assert!(matches!(
            evaluate_mixed_nash_candidate(&z, &z, &[0.5, 0.5], &[bad, 0.0], Config::default()),
            Err(Error::InvalidProbability {
                player: Player::Column,
                index: 0,
                ..
            })
        ));
    }
}

#[test]
fn incorrect_probability_mass_is_not_renormalized() {
    let z = vec![vec![0.0; 2]; 2];
    for p in [[0.0, 0.0], [0.2, 0.2], [0.7, 0.7]] {
        assert!(matches!(
            evaluate_mixed_nash_candidate(&z, &z, &p, &[0.5, 0.5], Config::default()),
            Err(Error::InvalidProbabilityMass {
                player: Player::Row,
                ..
            })
        ));
    }
}

#[test]
fn strategy_dimensions_must_match_the_full_payoff_table() {
    let z = vec![vec![0.0; 2]; 2];
    for p in [vec![], vec![1.0], vec![1.0, 0.0, 0.0]] {
        assert!(matches!(
            evaluate_mixed_nash_candidate(&z, &z, &p, &[0.5, 0.5], Config::default()),
            Err(Error::StrategyDimension {
                player: Player::Row,
                ..
            })
        ));
    }
}

#[test]
fn admitted_mass_roundoff_is_reported_and_probabilities_are_unchanged() {
    let z = vec![vec![0.0; 2]; 2];
    let p = [0.5, 0.5 + f64::EPSILON];
    let c = evaluate_mixed_nash_candidate(&z, &z, &p, &[0.5, 0.5], Config::default()).unwrap();
    assert_eq!(c.row_strategy, p);
    assert_eq!(c.diagnostics.probability_mass_errors[0], f64::EPSILON);
    assert_eq!(c.diagnostics.sum_gain, 0.0);
    let exact_mass = Config {
        probability_tolerance: 0.0,
        ..Config::default()
    };
    assert!(matches!(
        evaluate_mixed_nash_candidate(&z, &z, &p, &[0.5, 0.5], exact_mass),
        Err(Error::InvalidProbabilityMass { .. })
    ));
}

#[test]
fn unrepresentable_original_payoff_differences_fail_explicitly() {
    let a = vec![vec![f64::MAX], vec![-f64::MAX]];
    let b = vec![vec![0.0], vec![0.0]];
    assert!(matches!(
        evaluate_mixed_nash_candidate(&a, &b, &[1.0, 0.0], &[1.0], Config::default()),
        Err(Error::ArithmeticFailure { .. })
    ));
    assert!(matches!(
        solve_mixed_nash(&a, &b, Algorithm::SupportEnumeration, Config::default()),
        Err(Error::ArithmeticFailure { .. })
    ));
}

#[test]
fn underflow_does_not_silently_turn_a_nonzero_term_into_a_zero_certificate() {
    let a = vec![vec![0.0, 0.0], vec![0.0, 1.0]];
    let b = vec![vec![0.0; 2]; 2];
    let p = [1.0, f64::from_bits(1)];
    assert!(matches!(
        evaluate_mixed_nash_candidate(&a, &b, &p, &[0.5, 0.5], Config::default()),
        Err(Error::ArithmeticFailure { .. })
    ));
}

#[test]
fn boundary_solutions_are_not_duplicated_under_incorrect_larger_supports() {
    let a = vec![vec![0.0, 0.0], vec![-1.0, 0.0]];
    let b = vec![vec![0.0, -1.0], vec![0.0, 0.0]];
    let result = solve(&a, &b);
    assert_eq!(result.candidates.len(), 2);
    assert_eq!(result.non_positive_support_pairs, 1);
    assert!(result.candidates.iter().all(|c| c.row_support.len() == 1));
}

#[test]
fn near_rank_deficiency_is_unresolved_instead_of_claiming_an_exact_catalogue() {
    let delta = 1e-14;
    let a = vec![
        vec![0.0, 0.0, 0.0],
        vec![1.0, -1.0, 0.0],
        vec![1.0, -1.0 + delta, -delta],
    ];
    let b: Matrix = transpose(&a)
        .into_iter()
        .map(|r| r.into_iter().map(|v| -v).collect())
        .collect();
    let result = solve(&a, &b);
    assert_eq!(result.status, Status::UnresolvedSupports);
    assert!(result.unresolved_support_pairs > 0);
    assert!(!result.candidates.iter().any(|c| c.row_support.len() == 3));
    assert_eq!(
        result.catalogue_completeness,
        Catalogue::FiniteVerifiedWitnessesOnly
    );
}

#[test]
fn a_positive_support_probability_below_mass_tolerance_is_preserved() {
    let epsilon = 1e-14;
    let a = vec![vec![1.0, -1.0], vec![-1.0, 1.0]];
    let b = vec![vec![epsilon - 1.0, 0.0], vec![epsilon, 0.0]];
    let config = Config {
        equilibrium_tolerance: 1e-16,
        ..Config::default()
    };
    let result = solve_mixed_nash(&a, &b, Algorithm::SupportEnumeration, config).unwrap();
    let c = result
        .candidates
        .iter()
        .find(|c| c.row_support.len() == 2)
        .unwrap();
    assert!(c.row_strategy[0] > 0.0 && c.row_strategy[0] < config.probability_tolerance);
    assert_eq!(c.row_support, vec![0, 1]);
}

#[test]
fn repeated_calls_have_identical_candidates_and_accounted_supports() {
    for o in oracles() {
        let first = solve(&o.a, &o.b);
        assert_eq!(first, solve(&o.a, &o.b));
        assert_eq!(
            first.visited_support_pairs,
            first.unresolved_support_pairs
                + first.non_positive_support_pairs
                + first.rejected_best_response_support_pairs
                + first.candidates.len()
        );
    }
}

#[test]
fn one_action_against_many_selects_verified_pure_best_responses() {
    let result = solve(&[vec![-2.0, -7.0, -3.0]], &[vec![2.0, 3.0, 1.0]]);
    assert_eq!(result.total_support_pairs, 3);
    assert_eq!(result.candidates.len(), 1);
    assert_eq!(result.candidates[0].column_strategy, vec![0.0, 1.0, 0.0]);
    assert_eq!(
        result.candidates[0].diagnostics.expected_payoffs,
        [-7.0, 3.0]
    );
}
