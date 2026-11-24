//! Adaptive Nash enclosure for two jointly varying payoff vertices.

use super::{cover, Extrema, Mix, Ratio, RatioBounds};
use crate::InspectionError;
use paignia_core::Bimatrix2;

/// Maximum subdivision depth. A complete depth-seven tree has 21,845 cells.
pub const MAX_SEGMENT_DEPTH: u8 = 7;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Witness {
    pub first_action_shares: [Ratio; 2],
    /// Weight of vertex one. Vertex zero has the complementary weight.
    pub second_vertex_weight: Ratio,
    pub payoffs: [Ratio; 2],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Cell {
    pub first_action_shares: [RatioBounds; 2],
    /// None means the closed cell contains no equilibrium for any shared weight.
    pub witness: Option<Witness>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Enclosure {
    pub first_action_shares: [RatioBounds; 2],
    /// Coordinate-wise outer bounds. A bound need not be attained at Nash.
    pub payoffs: [RatioBounds; 2],
    /// Largest pure-deviation gain over the retained cells and the full segment.
    pub max_deviation_gain: Ratio,
    pub examined_cells: usize,
    /// Together these closed boxes cover both share axes, with shared boundaries.
    pub leaves: Vec<Cell>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("subdivision depth {0} exceeds {MAX_SEGMENT_DEPTH}")]
    InvalidDepth(u8),
    #[error(transparent)]
    Arithmetic(#[from] InspectionError),
    #[error("all cells were excluded from a nonempty finite game segment")]
    NoRetainedCells,
}

const UNIT: RatioBounds = RatioBounds {
    lower: Ratio::zero(),
    upper: Ratio::one(),
};

/// Enclose the union of Nash equilibria of all games along a 2x2 payoff segment.
///
/// Both players' matrices use the same weight. A cell is excluded exactly when
/// all support combinations have incompatible weight intervals. A retained leaf
/// includes one exact equilibrium witness. Coordinate bounds can span excluded
/// regions. See the credal Nash validation contract for the exclusion proof.
///
/// # Errors
/// Returns an error when the depth exceeds [`MAX_SEGMENT_DEPTH`], an intermediate
/// rational is not representable, or no leaf is retained.
pub fn enclose(vertices: &[Bimatrix2<Ratio>; 2], depth: u8) -> Result<Enclosure, Error> {
    if depth > MAX_SEGMENT_DEPTH {
        return Err(Error::InvalidDepth(depth));
    }
    let advantages = [advantages(&vertices[0])?, advantages(&vertices[1])?];
    let mut pending = vec![([UNIT; 2], 0)];
    let mut leaves = Vec::new();
    let mut first_action_shares = None;
    let mut extrema = None;
    let mut examined_cells = 0;
    while let Some((cell, level)) = pending.pop() {
        examined_cells += 1;
        let witness = if let Some((shares, weight)) = find_witness(&advantages, cell)? {
            if level < depth {
                let [p, q] = cell;
                let [p_lo, p_hi] = split(p)?;
                let [q_lo, q_hi] = split(q)?;
                pending.extend(
                    [[p_hi, q_hi], [p_hi, q_lo], [p_lo, q_hi], [p_lo, q_lo]]
                        .map(|child| (child, level + 1)),
                );
                continue;
            }
            let bounds = cell_extrema(vertices, cell)?;
            first_action_shares =
                Some(first_action_shares.map_or(cell, |total| cover(total, cell)));
            extrema = Some(extrema.map_or(bounds, |total: Extrema| total.union(bounds)));
            let [left, right] = vertex_payoffs(vertices, shares)?;
            Some(Witness {
                first_action_shares: shares,
                second_vertex_weight: weight,
                payoffs: [
                    blend([left[0], right[0]], weight)?,
                    blend([left[1], right[1]], weight)?,
                ],
            })
        } else {
            None
        };
        leaves.push(Cell {
            first_action_shares: cell,
            witness,
        });
    }
    let extrema = extrema.ok_or(Error::NoRetainedCells)?;
    Ok(Enclosure {
        first_action_shares: first_action_shares.ok_or(Error::NoRetainedCells)?,
        payoffs: extrema.payoffs,
        max_deviation_gain: extrema.max_gain,
        examined_cells,
        leaves,
    })
}

fn split(interval: RatioBounds) -> Result<[RatioBounds; 2], InspectionError> {
    let midpoint = blend([interval.lower, interval.upper], Ratio::new(1, 2)?)?;
    Ok([
        RatioBounds {
            lower: interval.lower,
            upper: midpoint,
        },
        RatioBounds {
            lower: midpoint,
            upper: interval.upper,
        },
    ])
}

fn mix(first: Ratio) -> Result<Mix, InspectionError> {
    Ok(Mix {
        first,
        second: Ratio::one().checked_sub(first)?,
    })
}

fn blend([first, second]: [Ratio; 2], weight: Ratio) -> Result<Ratio, InspectionError> {
    mix(weight)?.expectation([second, first])
}

fn vertex_payoffs(
    vertices: &[Bimatrix2<Ratio>; 2],
    [p, q]: [Ratio; 2],
) -> Result<[[Ratio; 2]; 2], InspectionError> {
    let p = mix(p)?;
    let q = mix(q)?;
    Ok([
        Extrema::at(&vertices[0], p, q)?
            .payoffs
            .map(|bounds| bounds.lower),
        Extrema::at(&vertices[1], p, q)?
            .payoffs
            .map(|bounds| bounds.lower),
    ])
}

fn cell_extrema(
    vertices: &[Bimatrix2<Ratio>; 2],
    [p, q]: [RatioBounds; 2],
) -> Result<Extrema, InspectionError> {
    let mut extrema = None;
    for row in [mix(p.lower)?, mix(p.upper)?] {
        for column in [mix(q.lower)?, mix(q.upper)?] {
            for vertex in vertices {
                let value = Extrema::at(vertex, row, column)?;
                extrema = Some(extrema.map_or(value, |total: Extrema| total.union(value)));
            }
        }
    }
    Ok(extrema.expect("both vertices and all four corners are evaluated"))
}

// Each player's first-action advantage when the opponent plays the second
// action or the first action, in that order.
fn advantages(game: &Bimatrix2<Ratio>) -> Result<[[Ratio; 2]; 2], InspectionError> {
    let [row_first, row_second] = game.row_payoffs;
    let [column_top, column_bottom] = game.column_payoffs;
    Ok([
        [
            row_first[1].checked_sub(row_second[1])?,
            row_first[0].checked_sub(row_second[0])?,
        ],
        [
            column_bottom[0].checked_sub(column_bottom[1])?,
            column_top[0].checked_sub(column_top[1])?,
        ],
    ])
}

fn endpoint_advantages(
    vertices: &[[[Ratio; 2]; 2]; 2],
    player: usize,
    opponent: RatioBounds,
) -> Result<[[Ratio; 2]; 2], InspectionError> {
    let at = |share| {
        Ok([
            blend(vertices[0][player], share)?,
            blend(vertices[1][player], share)?,
        ])
    };
    Ok([at(opponent.lower)?, at(opponent.upper)?])
}

#[derive(Clone, Copy)]
enum Support {
    First,
    Second,
    Indifferent,
}

impl Support {
    const ALL: [Self; 3] = [Self::First, Self::Second, Self::Indifferent];

    fn restrict(self, interval: RatioBounds) -> Option<RatioBounds> {
        match self {
            Self::First if interval.upper == Ratio::one() => Some(point(Ratio::one())),
            Self::Second if interval.lower == Ratio::zero() => Some(point(Ratio::zero())),
            Self::Indifferent => Some(interval),
            _ => None,
        }
    }

    fn weights(
        self,
        [lower, upper]: [[Ratio; 2]; 2],
    ) -> Result<[Option<RatioBounds>; 2], InspectionError> {
        match self {
            Self::First => Ok([sign_interval(lower, true)?, sign_interval(upper, true)?]),
            Self::Second => Ok([sign_interval(lower, false)?, sign_interval(upper, false)?]),
            Self::Indifferent => Ok([
                intersection(sign_interval(lower, true)?, sign_interval(upper, false)?),
                intersection(sign_interval(lower, false)?, sign_interval(upper, true)?),
            ]),
        }
    }

    fn opponent_share(
        self,
        interval: RatioBounds,
        [lower, upper]: [[Ratio; 2]; 2],
        weight: Ratio,
    ) -> Result<Ratio, InspectionError> {
        let low = blend(lower, weight)?;
        match self {
            Self::First if low >= Ratio::zero() => Ok(interval.lower),
            Self::Second if low <= Ratio::zero() => Ok(interval.lower),
            Self::Indifferent if low == Ratio::zero() => Ok(interval.lower),
            Self::Indifferent => {
                let high = blend(upper, weight)?;
                let crossing = low.div(low.checked_sub(high)?)?;
                blend([interval.lower, interval.upper], crossing)
            }
            _ => Ok(interval.upper),
        }
    }
}

fn point(value: Ratio) -> RatioBounds {
    RatioBounds {
        lower: value,
        upper: value,
    }
}

fn sign_interval(
    [first, second]: [Ratio; 2],
    nonnegative: bool,
) -> Result<Option<RatioBounds>, InspectionError> {
    let valid = |value| {
        if nonnegative {
            value >= Ratio::zero()
        } else {
            value <= Ratio::zero()
        }
    };
    match (valid(first), valid(second)) {
        (true, true) => Ok(Some(UNIT)),
        (false, false) => Ok(None),
        (true, false) => Ok(Some(RatioBounds {
            lower: UNIT.lower,
            upper: first.div(first.checked_sub(second)?)?,
        })),
        (false, true) => Ok(Some(RatioBounds {
            lower: first.div(first.checked_sub(second)?)?,
            upper: UNIT.upper,
        })),
    }
}

fn intersection(left: Option<RatioBounds>, right: Option<RatioBounds>) -> Option<RatioBounds> {
    let (left, right) = (left?, right?);
    let lower = left.lower.max(right.lower);
    let upper = left.upper.min(right.upper);
    (lower <= upper).then_some(RatioBounds { lower, upper })
}

fn find_witness(
    vertices: &[[[Ratio; 2]; 2]; 2],
    [p, q]: [RatioBounds; 2],
) -> Result<Option<([Ratio; 2], Ratio)>, InspectionError> {
    for row in Support::ALL {
        let Some(p) = row.restrict(p) else { continue };
        for column in Support::ALL {
            let Some(q) = column.restrict(q) else {
                continue;
            };
            let row_advantages = endpoint_advantages(vertices, 0, q)?;
            let column_advantages = endpoint_advantages(vertices, 1, p)?;
            let column_weights = column.weights(column_advantages)?;
            for row_weights in row.weights(row_advantages)?.into_iter().flatten() {
                for column_weights in column_weights.into_iter().flatten() {
                    if let Some(shared) = intersection(Some(row_weights), Some(column_weights)) {
                        let weight = shared.lower;
                        // Once the shared weight and supports are fixed, the two
                        // best-response conditions constrain different shares.
                        let x = column.opponent_share(p, column_advantages, weight)?;
                        let y = row.opponent_share(q, row_advantages, weight)?;
                        return Ok(Some(([x, y], weight)));
                    }
                }
            }
        }
    }
    Ok(None)
}
