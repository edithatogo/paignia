//! Exact controls for shared payoff weights and closed subdivision cells.

use paignia_core::Bimatrix2;
use paignia_solve::{
    enclose_credal_nash, enclose_credal_nash_segment, CredalNashSegmentCell,
    CredalNashSegmentError, InspectionRatio, RatioBounds, MAX_SEGMENT_DEPTH,
};

type Ratio = InspectionRatio;

#[path = "support/credal_segment_cases.rs"]
mod cases;

use cases::{dominance, pennies, r};

fn complement(share: Ratio) -> Ratio {
    r(1, 1).checked_sub(share).unwrap()
}

fn dot<const N: usize>(left: [Ratio; N], right: [Ratio; N]) -> Ratio {
    left.into_iter().zip(right).fold(r(0, 1), |sum, (x, y)| {
        sum.checked_add(x.checked_mul(y).unwrap()).unwrap()
    })
}

fn contains(interval: RatioBounds, value: Ratio) -> bool {
    (interval.lower..=interval.upper).contains(&value)
}

fn verify_witness(cell: &CredalNashSegmentCell, vertices: &[Bimatrix2<Ratio>; 2]) {
    let witness = cell
        .witness
        .expect("included cell carries an exact witness");
    let [row_share, column_share] = witness.first_action_shares;
    let weight = witness.second_vertex_weight;
    let [p_bound, q_bound] = cell.first_action_shares;
    assert!(contains(p_bound, row_share) && contains(q_bound, column_share));
    assert!((r(0, 1)..=r(1, 1)).contains(&weight));

    let weights = [complement(weight), weight];
    let row = |i: usize, j: usize| dot(weights, vertices.map(|game| game.row_payoffs[i][j]));
    let column = |i: usize, j: usize| dot(weights, vertices.map(|game| game.column_payoffs[i][j]));
    let row_strategy = [row_share, complement(row_share)];
    let column_strategy = [column_share, complement(column_share)];
    let row_payoffs = [
        dot(column_strategy, [row(0, 0), row(0, 1)]),
        dot(column_strategy, [row(1, 0), row(1, 1)]),
    ];
    let column_payoffs = [
        dot(row_strategy, [column(0, 0), column(1, 0)]),
        dot(row_strategy, [column(0, 1), column(1, 1)]),
    ];
    let [row_value, column_value] = witness.payoffs;
    assert_eq!(dot(row_strategy, row_payoffs), row_value);
    assert_eq!(dot(column_strategy, column_payoffs), column_value);
    assert_eq!(row_payoffs[0].max(row_payoffs[1]), row_value);
    assert_eq!(column_payoffs[0].max(column_payoffs[1]), column_value);
}

#[test]
fn infeasible_joint_weights_exclude_cells_even_when_both_sides_are_individually_possible() {
    let vertices = [dominance(1, -3), dominance(-3, 1)];
    let uniform = enclose_credal_nash(&vertices, 64).unwrap();
    let adaptive = enclose_credal_nash_segment(&vertices, 6).unwrap();

    // p > 0 needs t <= 1/4 and q > 0 needs t >= 3/4.
    // The exact union is the two axes, including the ties at 1/4 and 3/4.
    let feasible = adaptive
        .leaves
        .iter()
        .filter(|cell| cell.witness.is_some())
        .count();
    assert_eq!(uniform.retained_cells, 4096);
    assert_eq!(feasible, 127);
    assert_eq!(adaptive.examined_cells, 481);
    for cell in &adaptive.leaves {
        let [p, q] = cell.first_action_shares;
        if p.lower == r(0, 1) || q.lower == r(0, 1) {
            verify_witness(cell, &vertices);
        } else {
            assert!(cell.witness.is_none());
        }
    }
}

#[test]
fn a_boundary_equilibrium_is_covered_on_all_four_closed_sides() {
    let vertices = [pennies(1); 2];
    let adaptive = enclose_credal_nash_segment(&vertices, 4).unwrap();
    let retained: Vec<_> = adaptive
        .leaves
        .iter()
        .filter(|cell| cell.witness.is_some())
        .collect();
    assert_eq!(retained.len(), 4);
    assert_eq!(
        adaptive.first_action_shares,
        [RatioBounds {
            lower: r(7, 16),
            upper: r(9, 16)
        }; 2],
    );
    assert_eq!(
        adaptive.payoffs,
        [RatioBounds {
            lower: r(-1, 64),
            upper: r(1, 64)
        }; 2],
    );
    assert_eq!(adaptive.max_deviation_gain, r(9, 64));
    for cell in retained {
        verify_witness(cell, &vertices);
        let witness = cell.witness.unwrap();
        assert_eq!(witness.first_action_shares, [r(1, 2); 2]);
        assert_eq!(witness.payoffs, [r(0, 1); 2]);
    }
}

#[test]
fn a_nonvertex_all_tie_game_prevents_any_exclusion() {
    let vertices = [pennies(1), pennies(-1)];
    let adaptive = enclose_credal_nash_segment(&vertices, 3).unwrap();
    assert_eq!(adaptive.leaves.len(), 64);
    for cell in &adaptive.leaves {
        verify_witness(cell, &vertices);
        if cell
            .first_action_shares
            .iter()
            .any(|&interval| !contains(interval, r(1, 2)))
        {
            assert_eq!(cell.witness.unwrap().second_vertex_weight, r(1, 2));
        }
    }
}

#[test]
fn depth_zero_and_arithmetic_failures_have_explicit_outcomes() {
    let vertices = [pennies(1); 2];
    let adaptive = enclose_credal_nash_segment(&vertices, 0).unwrap();
    assert_eq!(adaptive.examined_cells, 1);
    assert_eq!(adaptive.leaves.len(), 1);
    verify_witness(&adaptive.leaves[0], &vertices);
    assert_eq!(
        enclose_credal_nash_segment(&vertices, MAX_SEGMENT_DEPTH + 1),
        Err(CredalNashSegmentError::InvalidDepth(MAX_SEGMENT_DEPTH + 1)),
    );
    let mut large = pennies(1);
    large.row_payoffs[0][0] = r(i64::MAX, 1);
    assert!(matches!(
        enclose_credal_nash_segment(&[large; 2], 1),
        Err(CredalNashSegmentError::Arithmetic(_))
    ));
}
