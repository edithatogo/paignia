# Nash enclosure for convex hulls of 2×2 payoff matrices

`paignia_core::Bimatrix2<T>` holds the two players' payoff matrices in
`[row][column]` order. Pass a nonempty slice of `Bimatrix2<InspectionRatio>`
and a grid size to `paignia_solve::enclose_credal_nash`. The shares `p` and
`q` are the probabilities of row 0 and column 0. Both output arrays use
row-player, column-player order.

Each hull member is one complete-information simultaneous game. For vertices
`(Aᵢ, Bᵢ)`, a single vector of nonnegative real weights summing to one gives both
`A = Σ wᵢ Aᵢ` and `B = Σ wᵢ Bᵢ`. Players mix independently within each
game. The enclosure covers the union of its members' Nash equilibria,
including pure strategies and ties. The consumer supplies the payoff matrices.
Mixing parameter vectors before a nonlinear payoff calculation can produce
different games. The exact inspection-family parameter hull and the
leader–follower commitment methods retain their own contracts.

## Cell certificate

Let `x = (p, 1-p)`, `y = (q, 1-q)`, and let `eᵢ` denote a pure strategy.
The current payoffs are `u = xᵀAy` and `v = xᵀBy`. The signed gains are
`eᵢᵀAy - u` for each row and `xᵀBeⱼ - v` for each column.
At Nash all four gains are nonpositive. Their maximum is nonnegative because
each player's current payoff averages their two pure-action payoffs.

The solver divides each share axis into `n` equal, closed intervals. For
fixed hull weights, each gain and payoff is affine in each share separately.
Its value inside a cell is a convex combination of its four corner values.
Each corner value is in turn the same hull-weighted average of the vertex
values. Hence the scalar minima and maxima over the cell and hull occur among
those corner–vertex pairs, including when a gain is identically zero.

A cell is excluded only if **one fixed deviation** has strictly positive
minimum over the entire cell and hull. Every cell containing a Nash profile
therefore survives, including adjacent cells that share a boundary equilibrium.
Different deviations may take their minima at different profiles or weights.
Retention alone supplies no equilibrium witness or common weights realizing
the reported endpoints.

The result gives coordinate-wise bounds on first-action shares and payoffs
over the surviving cells, together with `n`, `total_cells = n²` and
`retained_cells`. The `max_deviation_gain` is the exact largest gain over all
four deviations, all hull members, and the **union of retained cells**. The
coordinate summaries can span excluded cells. Individual endpoints can come
from different games or from profiles away from equilibrium.

## Precision and controls

For a singleton matching-pennies game with row payoffs of +1 for a match
and −1 otherwise, the unique equilibrium is `(1/2, 1/2)` with zero payoffs.
An 8×8 grid retains four cells, bounds each share by `[3/8, 5/8]` and each
payoff by `[-1/16, 1/16]`, and reports worst gain `5/16`. A 3×3 grid
retains one cell with the equilibrium in its interior.

Strict dominance may leave an entire boundary cell. With matching pennies
and its negative as vertices, the midpoint is the zero game. All profiles
are Nash for that member, so every cell survives at every resolution. The
worst gain remains 2 because profiles evaluated at an endpoint can be far
from equilibrium. Refining the strategy grid cannot resolve uncertainty in
the common weights.

The grid admits 1 through 128 divisions per axis. The method evaluates
`(n+1)²` grid points per input vertex and stores two rows of corner extrema.
Arithmetic uses checked reduced rationals. An unrepresentable intermediate
returns an error for the whole request. The native controls also cover a
strict-dominance corner, varying all-tie games, a nonvertex equilibrium
payoff outside the vertex equilibrium payoff span, mixed detection across
regimes, invalid grids, an empty hull and arithmetic overflow.

```sh
cargo test --locked -p paignia-solve --test credal_nash
```
