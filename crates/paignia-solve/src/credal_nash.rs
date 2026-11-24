//! Exact outer enclosure of simultaneous Nash equilibria on a convex hull of binary games.

use crate::{InspectionError, InspectionRatio as Ratio, RatioBounds};
use paignia_core::Bimatrix2;

mod segment;

pub use segment::{
    enclose as enclose_segment, Cell as SegmentCell, Enclosure as SegmentEnclosure,
    Error as SegmentError, Witness as SegmentWitness, MAX_SEGMENT_DEPTH,
};

/// Maximum number of equal cells on each strategy axis.
pub const MAX_GRID_DIVISIONS: u16 = 128;

/// Coordinate bounds over the retained closed cells, in row/column player order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Enclosure {
    /// Probabilities of the first row and first column, respectively.
    pub first_action_shares: [RatioBounds; 2],
    /// Expected utility bounds for the two players. Endpoints can occur away from equilibrium.
    pub payoffs: [RatioBounds; 2],
    /// Largest unilateral pure-deviation gain over the retained cells and the whole hull.
    pub max_deviation_gain: Ratio,
    pub grid_divisions: u16,
    pub total_cells: u32,
    pub retained_cells: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("the payoff hull needs at least one vertex")]
    EmptyHull,
    #[error("grid divisions {0} must be in 1..={MAX_GRID_DIVISIONS}")]
    InvalidGrid(u16),
    #[error(transparent)]
    Arithmetic(#[from] InspectionError),
    #[error("all cells were excluded from a nonempty finite game hull")]
    NoRetainedCells,
}

#[derive(Clone, Copy)]
struct Mix {
    first: Ratio,
    second: Ratio,
}

impl Mix {
    fn expectation(self, [a, b]: [Ratio; 2]) -> Result<Ratio, InspectionError> {
        if a == b {
            return Ok(a);
        }
        self.first
            .checked_mul(a)?
            .checked_add(self.second.checked_mul(b)?)
    }
}

fn cover(left: [RatioBounds; 2], right: [RatioBounds; 2]) -> [RatioBounds; 2] {
    std::array::from_fn(|i| RatioBounds {
        lower: left[i].lower.min(right[i].lower),
        upper: left[i].upper.max(right[i].upper),
    })
}

#[derive(Clone, Copy)]
struct Extrema {
    min_gains: [Ratio; 4],
    max_gain: Ratio,
    payoffs: [RatioBounds; 2],
}

impl Extrema {
    fn at(game: &Bimatrix2<Ratio>, p: Mix, q: Mix) -> Result<Self, InspectionError> {
        let row = [
            q.expectation(game.row_payoffs[0])?,
            q.expectation(game.row_payoffs[1])?,
        ];
        let [top, bottom] = game.column_payoffs;
        let column = [
            p.expectation([top[0], bottom[0]])?,
            p.expectation([top[1], bottom[1]])?,
        ];
        let u = p.expectation(row)?;
        let v = q.expectation(column)?;
        let gains = [
            row[0].checked_sub(u)?,
            row[1].checked_sub(u)?,
            column[0].checked_sub(v)?,
            column[1].checked_sub(v)?,
        ];
        Ok(Self {
            min_gains: gains,
            max_gain: gains[0].max(gains[1]).max(gains[2]).max(gains[3]),
            payoffs: [u, v].map(|value| RatioBounds {
                lower: value,
                upper: value,
            }),
        })
    }

    fn union(self, other: Self) -> Self {
        Self {
            min_gains: std::array::from_fn(|i| self.min_gains[i].min(other.min_gains[i])),
            max_gain: self.max_gain.max(other.max_gain),
            payoffs: cover(self.payoffs, other.payoffs),
        }
    }
}

/// Enclose all Nash shares and payoffs of all games in the convex hull of `vertices`.
/// One vector of convex weights mixes every payoff entry for both players.
/// Each player's actions are independently randomized within the resulting game.
///
/// Expected payoffs and the four individual deviation gains are multi-affine in
/// the strategy shares and affine in the common weights. Their extrema on a
/// closed grid cell occur at grid corners and hull vertices. A cell is excluded
/// only if one fixed pure deviation improves strictly throughout that product.
/// Retained cells can include profiles with positive regret. Their worst regret
/// is reported in the original payoff units.
///
/// # Errors
///
/// Rejects an empty hull, a grid outside `1..=MAX_GRID_DIVISIONS`, or an
/// unrepresentable intermediate rational. An empty enclosure is an invariant error.
pub fn enclose(vertices: &[Bimatrix2<Ratio>], grid_divisions: u16) -> Result<Enclosure, Error> {
    let (first, rest) = vertices.split_first().ok_or(Error::EmptyHull)?;
    if !(1..=MAX_GRID_DIVISIONS).contains(&grid_divisions) {
        return Err(Error::InvalidGrid(grid_divisions));
    }
    let divisions = i64::from(grid_divisions);
    let grid = (0..=divisions)
        .map(|i| {
            Ok(Mix {
                first: Ratio::new(i, divisions)?,
                second: Ratio::new(divisions - i, divisions)?,
            })
        })
        .collect::<Result<Vec<_>, InspectionError>>()?;

    let sample_row = |p| {
        grid.iter()
            .map(|&q| {
                let mut bounds = Extrema::at(first, p, q)?;
                for vertex in rest {
                    bounds = bounds.union(Extrema::at(vertex, p, q)?);
                }
                Ok(bounds)
            })
            .collect::<Result<Vec<_>, InspectionError>>()
    };

    let mut previous = sample_row(grid[0])?;
    let mut result = None;
    for rows in grid.windows(2) {
        let current = sample_row(rows[1])?;
        for (j, columns) in grid.windows(2).enumerate() {
            let cell = previous[j]
                .union(previous[j + 1])
                .union(current[j])
                .union(current[j + 1]);
            if cell.min_gains.iter().any(|gain| gain.numerator() > 0) {
                continue;
            }
            let shares = [rows, columns].map(|axis| RatioBounds {
                lower: axis[0].first,
                upper: axis[1].first,
            });
            let outer = result.get_or_insert_with(|| Enclosure {
                first_action_shares: shares,
                payoffs: cell.payoffs,
                max_deviation_gain: cell.max_gain,
                grid_divisions,
                total_cells: u32::from(grid_divisions).pow(2),
                retained_cells: 0,
            });
            outer.first_action_shares = cover(outer.first_action_shares, shares);
            outer.payoffs = cover(outer.payoffs, cell.payoffs);
            outer.max_deviation_gain = outer.max_deviation_gain.max(cell.max_gain);
            outer.retained_cells += 1;
        }
        previous = current;
    }
    result.ok_or(Error::NoRetainedCells)
}
