# Exact Nash feasibility on a paired payoff segment

## Scope

Compute an adaptive outer cover of all simultaneous Nash profiles for two
paired rational 2×2 payoff vertices. Use one mixture weight for both players.
Record a witness for feasible leaves and retain excluded boxes for replay.
Preserve the general fixed-grid method and its existing error contract.

## Implementation and verification

- [x] Derive closed weight-interval checks for the nine support pairs.
- [x] Return rational witnesses and conservative payoff and deviation bounds.
- [x] Check incompatible weights, boundary coverage, a nonvertex all-tie game,
  and arithmetic failures against independent conditions.
- [x] Provide a release-mode benchmark that compares both methods on the same
  synthetic games and finest grid.

The [method contract and benchmark](../../docs/validation/credal-nash.md#adaptive-enclosure-for-two-payoff-vertices)
record the bounds and their limitations. Larger hulls and independently
verified application cases remain separate research work.
