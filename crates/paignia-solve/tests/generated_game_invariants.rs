//! Bounded synthetic game families with closed-form witnesses.
//!
//! These exercise the production Rust solver. They extend test coverage and do
//! not replace the independently frozen numerical controls or qualify consumers.

use paignia_solve::{
    evaluate_mixed_nash_candidate, solve_mixed_nash, MixedNashAlgorithm, MixedNashCandidate,
    MixedNashConfig, MixedNashError,
};

type Matrix = Vec<Vec<f64>>;

fn near(actual: f64, expected: f64) {
    assert!(
        actual.is_finite() && (actual - expected).abs() <= 1e-10,
        "actual={actual:.17e}, expected={expected:.17e}"
    );
}

fn unique_candidate(a: &Matrix, b: &Matrix) -> MixedNashCandidate {
    let result = solve_mixed_nash(
        a,
        b,
        MixedNashAlgorithm::SupportEnumeration,
        MixedNashConfig::default(),
    )
    .expect("bounded finite game must be admitted");
    assert_eq!(result.candidates.len(), 1, "a={a:?}, b={b:?}");
    assert_eq!(result.visited_support_pairs, result.total_support_pairs);
    result.candidates.into_iter().next().unwrap()
}

fn witness(candidate: &MixedNashCandidate, p: [f64; 2], q: [f64; 2], utility: [f64; 2]) {
    for (actual, expected) in candidate.row_strategy.iter().zip(p) {
        near(*actual, expected);
    }
    for (actual, expected) in candidate.column_strategy.iter().zip(q) {
        near(*actual, expected);
    }
    for (actual, expected) in candidate.diagnostics.expected_payoffs.iter().zip(utility) {
        near(*actual, expected);
    }
    assert_eq!(candidate.row_strategy.len(), 2);
    assert_eq!(candidate.column_strategy.len(), 2);
    assert!(candidate.diagnostics.within_tolerance);
    near(candidate.diagnostics.max_gain, 0.0);
}

#[test]
fn generated_dominance_games_recover_unique_actions_and_expose_false_mixtures() {
    let mut cases = 0;
    for row_gap in (-4..=4).filter(|gap| *gap != 0) {
        for column_gap in (-4..=4).filter(|gap| *gap != 0) {
            for externality in [-3, 0, 3] {
                for baseline in [-2, 0, 2] {
                    let (row_advantage, column_advantage, opponent_effect, base_payoff) = (
                        f64::from(row_gap),
                        f64::from(column_gap),
                        f64::from(externality),
                        f64::from(baseline),
                    );
                    let row_payoffs = vec![
                        vec![
                            base_payoff + row_advantage,
                            base_payoff + opponent_effect + row_advantage,
                        ],
                        vec![base_payoff, base_payoff + opponent_effect],
                    ];
                    let column_payoffs = vec![
                        vec![base_payoff + column_advantage, base_payoff],
                        vec![
                            base_payoff + opponent_effect + column_advantage,
                            base_payoff + opponent_effect,
                        ],
                    ];
                    let row_strategy = if row_gap > 0 { [1.0, 0.0] } else { [0.0, 1.0] };
                    let column_strategy = if column_gap > 0 {
                        [1.0, 0.0]
                    } else {
                        [0.0, 1.0]
                    };
                    witness(
                        &unique_candidate(&row_payoffs, &column_payoffs),
                        row_strategy,
                        column_strategy,
                        [
                            base_payoff
                                + row_advantage.max(0.0)
                                + column_strategy[1] * opponent_effect,
                            base_payoff
                                + column_advantage.max(0.0)
                                + row_strategy[1] * opponent_effect,
                        ],
                    );
                    let false_mixture = evaluate_mixed_nash_candidate(
                        &row_payoffs,
                        &column_payoffs,
                        &[0.5, 0.5],
                        &[0.5, 0.5],
                        MixedNashConfig::default(),
                    )
                    .unwrap();
                    assert!(!false_mixture.diagnostics.within_tolerance);
                    near(
                        false_mixture.diagnostics.best_response_gains[0],
                        row_advantage.abs() / 2.0,
                    );
                    near(
                        false_mixture.diagnostics.best_response_gains[1],
                        column_advantage.abs() / 2.0,
                    );
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(cases, 576);
}

fn affine(matrix: &Matrix, scale: f64, offset: f64, reverse: bool) -> Matrix {
    let mut result: Matrix = matrix
        .iter()
        .map(|row| row.iter().map(|value| value * scale + offset).collect())
        .collect();
    if reverse {
        result.reverse();
        for row in &mut result {
            row.reverse();
        }
    }
    result
}

#[test]
fn generated_zero_sum_witnesses_survive_affine_payoffs_and_action_relabelling() {
    let mut cases = 0;
    for left in 1..=9 {
        for right in 1..=9 {
            let (left, right) = (f64::from(left), f64::from(right));
            let a = vec![vec![left, -right], vec![-left, right]];
            let b = affine(&a, -1.0, 0.0, false);
            for row_scale in [1.0, 2.0, 4.0] {
                for column_scale in [1.0, 3.0] {
                    for reverse in [false, true] {
                        let mut q = [right / (left + right), left / (left + right)];
                        if reverse {
                            q.reverse();
                        }
                        witness(
                            &unique_candidate(
                                &affine(&a, row_scale, -3.0, reverse),
                                &affine(&b, column_scale, 7.0, reverse),
                            ),
                            [0.5, 0.5],
                            q,
                            [-3.0, 7.0],
                        );
                        cases += 1;
                    }
                }
            }
        }
    }
    assert_eq!(cases, 972);
}

#[test]
fn nonfinite_mutations_are_rejected_in_every_payoff_position() {
    let mut cases = 0;
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for row in 0..2 {
            for column in 0..2 {
                let mut mutated = vec![vec![1.0, -1.0], vec![-1.0, 1.0]];
                mutated[row][column] = bad;
                let finite = vec![vec![0.0; 2]; 2];
                for (a, b) in [(&mutated, &finite), (&finite, &mutated)] {
                    assert!(matches!(
                        solve_mixed_nash(
                            a,
                            b,
                            MixedNashAlgorithm::SupportEnumeration,
                            MixedNashConfig::default(),
                        ),
                        Err(MixedNashError::NonFinitePayoff { .. })
                    ));
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(cases, 24);
}
