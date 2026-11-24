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

## Adaptive enclosure for two payoff vertices

Pass exactly two paired 2×2 games to the segment solver
(`paignia_solve::enclose_credal_nash_segment`). Vertex zero has weight
`1 − t` and vertex one has weight `t`. The same rational weight applies to
both payoff matrices. Depth zero visits the full strategy square. The maximum
depth is seven, with at most 21,845 examined boxes. The uniform method above
continues to accept arbitrary finite vertex hulls.

For each closed strategy box, consider three support cases per player. The
first action can be pure at share one, the second can be pure at share zero,
and either can be used when the player is indifferent. Restrict the player's
share to its support case before testing the other player's best response.

For a fixed weight, a player's first-action advantage varies affinely in the
opponent's share. A pure best response requires the appropriate sign at either
endpoint of that opponent's restricted interval. Indifference requires the
two endpoint values to straddle zero, including equalities. Each endpoint
advantage is affine in the common weight. Its sign therefore defines a closed
weight interval, an empty interval, or the whole unit interval. Intersect the
two players' allowed weight intervals for each support pair. An excluded box
has no nonempty intersection. The returned box coordinates and the two input
games suffice to replay these finite rational tests.

Each retained leaf carries one shared weight and strategy profile. At the
chosen weight the two best-response conditions constrain different shares,
so endpoint selection or an exact zero crossing gives a joint witness. Closed
boundaries, singleton intervals and all-tie games remain covered. The tests
recompute both players' expected and best pure payoffs from the original
matrices and the witness.

The method reports coordinate-wise payoff and share bounds, maximum deviation
gain over the retained boxes and the whole segment, and the examined tree
size. A retained box contains at least one equilibrium across the segment.
Its other points and the payoff-bound endpoints can fall outside equilibrium.
Checked rational arithmetic propagates an error when an intermediate is
unrepresentable.

### Reproduce the comparison

```sh
cargo test --locked -p paignia-solve --test credal_nash_segment
cargo run --locked --release -p paignia-solve --example credal_nash_segment_benchmark
```

The three synthetic cases use the same 64×64 finest grid. In the shared-weight
conflict, positive first-action shares require `t ≤ 1/4` for the row player
and `t ≥ 3/4` for the column player. The fixed grid retains all 4,096 cells.
The segment solver examines 481 boxes and retains 127 leaf cells along the
two axes. Both methods retain the same four cells in matching pennies. The
interior all-tie case retains all 4,096 leaf cells and takes more work with
adaptive subdivision. The example reports median runtime over 21 runs after
warm-up. Compare results on the same toolchain and machine.

The next benchmark needs higher-dimensional uncertainty sets and an
independent feasibility oracle. Exact support feasibility for a line segment
does not establish a speed or coverage claim for general payoff hulls.
