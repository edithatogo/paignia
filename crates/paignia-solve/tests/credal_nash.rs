use paignia_core::Bimatrix2;
use paignia_solve::{
    enclose_credal_nash, CredalNashError, InspectionError, InspectionRatio as Ratio, RatioBounds,
    MAX_CREDAL_NASH_GRID_DIVISIONS,
};

fn r(n: i64, d: i64) -> Ratio {
    Ratio::new(n, d).unwrap()
}

fn bounds(lower: Ratio, upper: Ratio) -> RatioBounds {
    RatioBounds { lower, upper }
}

fn game(row: [[i64; 2]; 2], column: [[i64; 2]; 2]) -> Bimatrix2<Ratio> {
    Bimatrix2 {
        row_payoffs: row.map(|pair| pair.map(|x| r(x, 1))),
        column_payoffs: column.map(|pair| pair.map(|x| r(x, 1))),
    }
}

const MATCH: [[i64; 2]; 2] = [[1, -1], [-1, 1]];
const MISMATCH: [[i64; 2]; 2] = [[-1, 1], [1, -1]];

#[test]
fn matching_pennies_keeps_every_cell_touching_the_dyadic_equilibrium() {
    // U=(2p-1)(2q-1). Both response differences change sign at 1/2.
    // For example, grid 8 has maximum row gain 2*(5/8)*(1/4).
    // Grid 3 puts the equilibrium strictly inside one cell.
    for (grid, cells, lo, hi, payoff, gain) in [
        (3, 1, r(1, 3), r(2, 3), bounds(r(-1, 9), r(1, 9)), r(4, 9)),
        (4, 4, r(1, 4), r(3, 4), bounds(r(-1, 4), r(1, 4)), r(3, 4)),
        (
            8,
            4,
            r(3, 8),
            r(5, 8),
            bounds(r(-1, 16), r(1, 16)),
            r(5, 16),
        ),
    ] {
        let enclosure = enclose_credal_nash(&[game(MATCH, MISMATCH)], grid).unwrap();
        assert_eq!(enclosure.first_action_shares, [bounds(lo, hi); 2]);
        assert_eq!(enclosure.payoffs, [payoff; 2]);
        assert_eq!(enclosure.max_deviation_gain, gain);
        assert_eq!(enclosure.grid_divisions, grid);
        assert_eq!(enclosure.total_cells, u32::from(grid).pow(2));
        assert_eq!(enclosure.retained_cells, cells);
    }
}

#[test]
fn identity_game_residual_is_the_maximum_over_retained_cells() {
    let identity = [[1, 0], [0, 1]];
    let coordination = game(identity, identity);
    let enclosure = enclose_credal_nash(&[coordination], 4).unwrap();
    assert_eq!((enclosure.total_cells, enclosure.retained_cells), (16, 6));
    assert_eq!(enclosure.max_deviation_gain, r(3, 8));

    // Shares (1, 0) select row 0 and column 1.
    for (range, share) in enclosure
        .first_action_shares
        .into_iter()
        .zip([r(1, 1), r(0, 1)])
    {
        assert!(range.lower <= share && share <= range.upper);
    }
    let row = coordination.row_payoffs;
    let column = coordination.column_payoffs;
    let row_gain = row[1][1].checked_sub(row[0][1]).unwrap();
    let column_gain = column[0][0].checked_sub(column[0][1]).unwrap();
    assert_eq!(row_gain.max(column_gain), r(1, 1));
}

#[test]
fn strict_dominance_retains_the_cell_at_the_correct_pure_corner() {
    // First row wins by 2 and second column wins by 4 for every opponent mix.
    let dominant = game([[3, 3], [1, 1]], [[0, 4], [0, 4]]);
    let enclosure = enclose_credal_nash(&[dominant], 4).unwrap();
    assert_eq!(
        enclosure.first_action_shares,
        [bounds(r(3, 4), r(1, 1)), bounds(r(0, 1), r(1, 4))]
    );
    assert_eq!(
        enclosure.payoffs,
        [bounds(r(5, 2), r(3, 1)), bounds(r(3, 1), r(4, 1))]
    );
    assert_eq!(enclosure.max_deviation_gain, r(1, 1));
    assert_eq!((enclosure.total_cells, enclosure.retained_cells), (16, 1));
}

#[test]
fn common_weights_preserve_all_ties_even_with_different_vertex_payoffs() {
    let low = Bimatrix2 {
        row_payoffs: [[r(-2, 1); 2]; 2],
        column_payoffs: [[r(5, 2); 2]; 2],
    };
    let high = game([[4; 2]; 2], [[-1; 2]; 2]);
    let enclosure = enclose_credal_nash(&[low, high], 3).unwrap();
    assert_eq!(enclosure.first_action_shares, [bounds(r(0, 1), r(1, 1)); 2]);
    assert_eq!(
        enclosure.payoffs,
        [bounds(r(-2, 1), r(4, 1)), bounds(r(-1, 1), r(5, 2))]
    );
    assert_eq!(enclosure.max_deviation_gain, r(0, 1));
    assert_eq!(enclosure.retained_cells, 9);
    assert_eq!(
        enclosure,
        enclose_credal_nash(&[high, low, high], 3).unwrap()
    );
}

#[test]
fn a_nonvertex_tie_has_equilibria_outside_both_vertex_share_bounds() {
    // Each signed matching-pennies vertex has unique shares (1/2, 1/2).
    // Their half mixture is the zero game: even (1/4, 3/4) and pure pairs are Nash.
    let enclosure =
        enclose_credal_nash(&[game(MATCH, MISMATCH), game(MISMATCH, MATCH)], 8).unwrap();
    assert_eq!(enclosure.first_action_shares, [bounds(r(0, 1), r(1, 1)); 2]);
    assert_eq!(enclosure.payoffs, [bounds(r(-1, 1), r(1, 1)); 2]);
    assert_eq!(enclosure.max_deviation_gain, r(2, 1));
    assert_eq!(enclosure.retained_cells, 64);
}

#[test]
fn an_interior_game_has_a_nash_payoff_below_both_vertex_nash_payoffs() {
    // Column 0 strictly dominates throughout. Each vertex gives its best row 1.
    // Their half mixture gives every row 0, including at mixed Nash (1/2, 1).
    let column = [[1, 0], [1, 0]];
    let top = game([[1; 2], [-1; 2]], column);
    let bottom = game([[-1; 2], [1; 2]], column);
    let enclosure = enclose_credal_nash(&[top, bottom], 4).unwrap();
    assert_eq!(
        enclosure.first_action_shares,
        [bounds(r(0, 1), r(1, 1)), bounds(r(3, 4), r(1, 1))]
    );
    assert_eq!(
        enclosure.payoffs,
        [bounds(r(-1, 1), r(1, 1)), bounds(r(3, 4), r(1, 1))]
    );
    assert_eq!(enclosure.retained_cells, 4);
    assert_eq!(enclosure.max_deviation_gain, r(2, 1));
}

#[test]
fn a_detection_mixture_encloses_pure_mixed_and_boundary_witnesses() {
    // At detection d the inspect/evade cell is (-1-4(1-d), 1-2d).
    // Mixing the vertices d=1/8 and d=1 uses the same weight for both players.
    let low = Bimatrix2 {
        row_payoffs: [[r(-1, 1), r(-9, 2)], [r(0, 1), r(-4, 1)]],
        column_payoffs: [[r(0, 1), r(3, 4)], [r(0, 1), r(1, 1)]],
    };
    let high = game([[-1, -1], [0, -4]], [[0, -1], [0, 1]]);
    let enclosure = enclose_credal_nash(&[low, high], 8).unwrap();
    for (p, q, row, column) in [
        (r(0, 1), r(0, 1), r(-4, 1), r(1, 1)), // d=1/8
        (r(1, 1), r(0, 1), r(-7, 2), r(1, 4)), // d=3/8
        (r(1, 1), r(1, 4), r(-5, 2), r(0, 1)), // d=1/2 (follower tie)
        (r(2, 3), r(2, 3), r(-4, 3), r(0, 1)), // d=3/4
        (r(1, 2), r(3, 4), r(-1, 1), r(0, 1)), // d=1
    ] {
        for (range, value) in enclosure
            .first_action_shares
            .into_iter()
            .zip([p, q])
            .chain(enclosure.payoffs.into_iter().zip([row, column]))
        {
            assert!(
                range.lower <= value && value <= range.upper,
                "{range:?} excludes {value:?}"
            );
        }
    }
}

#[test]
fn invalid_inputs_and_unrepresentable_deviation_return_errors() {
    let pennies = game(MATCH, MISMATCH);
    assert_eq!(enclose_credal_nash(&[], 1), Err(CredalNashError::EmptyHull));
    for grid in [0, MAX_CREDAL_NASH_GRID_DIVISIONS + 1] {
        assert_eq!(
            enclose_credal_nash(&[pennies], grid),
            Err(CredalNashError::InvalidGrid(grid))
        );
    }
    let indifferent = game([[0; 2]; 2], [[0; 2]; 2]);
    for grid in [1, MAX_CREDAL_NASH_GRID_DIVISIONS] {
        let result = enclose_credal_nash(&[indifferent], grid).unwrap();
        assert_eq!(result.retained_cells, u32::from(grid).pow(2));
    }
    let extreme = game([[i64::MAX; 2], [i64::MIN; 2]], [[0; 2]; 2]);
    assert_eq!(
        enclose_credal_nash(&[extreme], 1),
        Err(CredalNashError::Arithmetic(InspectionError::Overflow))
    );
}
