# T18 Implementation Plan — Illicit-supply border-enforcement and retail-inspection game contracts for tobac

Paignia completed T18.01 and T18.02. The provider work preserves T17 ownership
and the neutral vocabulary guard. T18.03 remains pending in the application repository.

## Phase 1 — Neutral leader-follower inspection primitives (`paignia`)

- [x] **T18.01**: Check the exact binary leader-follower and simultaneous Nash
  family against independent analytical controls. The reviewed solver reports
  best responses, budget-constrained values and attainment. PR #41 merged
  with both Rust profiles and the independent inspection oracle passing.
  [Commitment contract](../../../docs/validation/leader-follower.md)
- [x] **T18.02**: Bound equilibrium action shares and payoffs on the supported
  parameter ranges and on convex hulls of paired two-by-two payoff matrices.
  Exact projections retain the parameter restrictions in the implementation
  history below. PR #54 adds certified conservative Nash enclosures with
  retained cell counts and a maximum deviation gain. Both Rust profiles,
  independent controls and specialist review passed.
  [Credal Nash contract](../../../docs/validation/credal-nash.md)

## Pending downstream handoff

**T18.03** remains with `edithatogo/tobac`. The application owner will bind `tobac` border-seizure and retail-inspection evasion scenarios (`M2a`, `src/tobac/partial_identification.py`) to `paignia`'s neutral inspection game contract, preserving `dapane` enforcement resource cost provenance and `voiage` paired-draw / credal dominance compatibility.

## Provider acceptance and application handoff

[The 20 November review](../../../docs/validation/provider-review-20261120.md)
records the reviewed provider revisions, checks and owner boundaries.
Paignia main at `6d4eaf2` passed both native CI and Offline AI Security.
[Native CI](https://github.com/edithatogo/paignia/actions/runs/45358436354) · [Offline AI Security](https://github.com/edithatogo/paignia/actions/runs/45358437141)

The Tobac draft exercised both inspection channels with synthetic Dapane costs
and preserved the paired-draw identity through the installed Voiage comparison.
Hosted verification requires a fresh signed native-lineage capsule covering the
expanded source inventory, including `src/tobac/strategic_inspection.py`. The
application's empirical acceptance is pending.
[Tobac consumer draft](https://github.com/edithatogo/tobac/pull/178)

## Implementation history

The original parameter projections used these distinguishing controls.
one parameter of the exact family may range over a closed positive interval. The Nash projection reports attained coordinate bounds and an endpoint gap of zero. A point inside the crossing-cost box is shown to have a profitable deviation, so the box is not a joint equilibrium set. A later slice admits a detection probability in `(0, 1]`: perfect detection reproduces the original witness, half detection on the interior example has inspection share `2/3`, evasion share `1/2` and leader value `-2`, and a detection interval that changes regime is rejected. Commitment at detection `1/2` attains the optimistic value `-2/3` and leaves the pessimistic `-2/3` unattained. A two-parameter rectangle is projected from its corners only while every corner stays in one strict regime. Detection and cost together, on `[1/2, 1]` and `[1, 3/2]`, move inspection from `1/3` to `2/3` and the leader value from `-3` to `-1`. A commitment projection spans one strict piece: full-budget cost in `[1, 2]` moves the leader value from `-1/3` to `-2/3`, and a gain interval that crosses the budget is rejected. A four-parameter box is the corner span of one strict regime; cost `1` attains leader value `-1` with evasion at most `1/6`, so the separate upper bounds are not a joint witness. This is not an arbitrary payoff credal set. The convex hull of finitely many perfect-detection parameter vectors is projected from its vertices when they share one strict regime; the midpoint of `(1, 6, 1, 4)` and `(2, 8, 2, 2)` has inspection `1/3` and evasion `3/14`. Imperfect detection is excluded because mixing interior detection games can leave the interior. Commitment is not included either: both `(1, 4, 4, 1)` and `(2, 4, 2, 3)` have optimistic value `-4/5`, and their midpoint has value `-9/10`.
