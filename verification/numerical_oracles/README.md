# T17.04–T17.06 independent numerical fixtures

Status: three local fixture slices implemented; independent numerical review
pending; **no production qualification**.

This directory contains synthetic mathematical references and certificate checks,
not a Paignia solver, public interchange schema, or host-language runtime fallback.
All supported game execution stays in Rust. These reference checks intentionally
do not import either production crate. Passing them establishes properties of
these fixtures only; it does not execute or qualify a production method.

## Ownership and bounded increments

Dylan authorised this separate numerical-fixture lane on 15 November 2026.
The [claim was published before implementation](https://github.com/edithatogo/paignia/pull/14#issuecomment-6890831487).
The inspected main was `fca2b2f8811c8cd56d27be4c9407f573d54ad1e9`; coordinator
PR14 was the draft input-profile slice at
`0c88607a142508d1fe4f1153b963fd1411b49395`. Open PRs #7, #11 and #14,
their changed-file lists/comments, the 12 published branches, main's complete
tree, and the coordinator's current ownership map had no intersecting claim.
This is evidence of the inspected published state, not private-owner acknowledgement.

This lane owns exactly five new paths in this directory: `README.md`,
`test_oracles.py`, `replicator_rk4_v1.json`, `mixed_nash_v1.json`, and
`correlated_equilibrium_v1.json`. No existing path is claimed. The coordinator
`6af93fe8-b14c-83ec-8de0-728c856bcbb3` retains all shared integration and
production paths. Reviewer `6af956e9-2d90-83ec-9bd9-5697990c506f` retains
PR14's specialist review. New intersecting claims stop writes until reconciled.

The parent [T17 plan](../../conductor/tracks/t17_game_runtime_ecosystem_20261107/plan.md)
and [specification](../../conductor/tracks/t17_game_runtime_ecosystem_20261107/spec.md)
remain authoritative; their shared status files are coordinator-owned.

| Local fixture slice | One reference behavior | Changed files | Status |
| --- | --- | --- | --- |
| T17.04.oracle | Verify four-stage/logistic replicator references and distinguishing controls | README, harness, replicator fixture | [x] local reference checks; specialist review pending |
| T17.05.oracle | Verify exact finite-game Nash witnesses and deviation residuals | README, harness, Nash fixture | [x] local reference checks; specialist review pending |
| T17.06.oracle | Verify CE incentive and objective certificates | README, harness, CE fixture | [x] local reference checks; specialist review pending |

Tests precede each reference implementation. Record actual distinguishing failures,
nonzero executed cases, baseline/head and fixture hashes; an absent target is not
a passing baseline. Each increment changes at most five files and one behavior.

## Qualification boundary

Full T17.01 result, identity, algorithm, status, tolerance/residual, selection and
catalogue semantics are not accepted/pinned. Its input-profile PR alone cannot
unblock production numerical acceptance. All three parent tasks remain open.
Independent specialist mathematical review is required by the Conductor workflow;
author review is separate and no new worker is authorised in this lane.

Static and dynamic acceptance remain separate. No simulation adapter, T17.13
conversion, consumer/provider pin, upstream source, manifest, lock, root CI,
registry or shared Conductor record is changed. The fixture's mathematical
probability/domain rules are reference assumptions, not invented public errors or
production tolerance semantics. An integration adapter must wait for T17.01.

## Environment

The shared filesystem had approximately 6.1 MB free. This lane uses an isolated
checkout on the separate `/dev/shm` device, with approximately 4.8 GiB available
at intake. No other work, cache or build was deleted. The harness uses
only Python's standard library, exact rational arithmetic where possible, and
explicit high-precision decimal arithmetic for analytic exponentials; no network,
package installation, optimizer dependency or native build is required.

## Run the reference checks

From the repository root, using Python 3.12 (executed here on 3.12.14):

```bash
python3 -B verification/numerical_oracles/test_oracles.py
python3 -B verification/numerical_oracles/test_oracles.py ReplicatorOracleTests
python3 -B verification/numerical_oracles/test_oracles.py NashOracleTests
python3 -B verification/numerical_oracles/test_oracles.py CorrelatedOracleTests
```

These are development/reference commands. The existing root CI does not run this
new harness; adding an explicit invocation is a coordinator integration request.
The repository's native stable/MSRV checks remain separate required merge gates.

## T17.04.oracle: replicator references

For two proportions `(x, 1-x)` and fitness difference `s(t)`, the replicator
equation reduces to `x' = s(t) x (1-x)`. Separation gives
`log(x/(1-x)) - log(x0/(1-x0)) = integral(s(t), t0, t)`, hence the closed form
recorded in the JSON. This independent solution, not a second integration
implementation, supplies the trajectory targets. The boundary states 0 and 1
and equal-fitness states are stationary. Negative fitness is allowed.

The [classical four-stage tableau](https://www.math.auckland.ac.nz/~butcher/CONFERENCES/seminar-21-10-04.pdf#page=22)
is independently expanded at `x0=1/4`, `h=1/2`, `s=1` into exact rational stage
states and slopes. The result is
`408885515741609973/1152921504606846976`; the named Euler control gives `11/32`.
The harness advances both components and checks their sum, with no projection,
clipping, or renormalization. An oversized step that leaves the simplex fails
explicitly in the reference checker; positivity is only asserted on the stated
small-step grids, never unconditionally for RK4.

Smooth references cover constant positive, constant negative and affine
time-varying fitness. At 80-digit decimal precision, successive error ratios
must lie strictly between 14 and 18 (the fourth-order asymptote is 16), and the
finest absolute error must be below `1e-9`. The analytic literals have a separate
`1e-70` check and population mass a `1e-65` check. Exact rational witnesses use
zero tolerance. These limits describe the fixture harness, not a future binary64
production acceptance policy.

The initial affine-fitness grid of 64 steps missed the unchanged `1e-9` error
budget: observed error `1.0539302169784621e-9`. The reference grid was refined to
32/64/128 steps, retaining both the error budget and order interval. Its final
error is approximately `6.5723e-11`; the last error ratio is approximately 16.036.
Constant positive/negative finest errors are approximately `2.5503e-10` and
`3.1339e-10`. This was a reference-fixture refinement, not a production result.

The event fixture switches the fitness difference from +1 to -1 halfway through
a unit interval, so the integrated difference is exactly zero and the final
proportion is exactly the initial 1/4. Split smooth segments use one-sided
endpoint stages and meet the error budget. An unsplit right-continuous callback
misses it. Additional controls detect Euler relabelled as RK4, frozen stage
times, invalid populations/time/steps and invalid fitness callbacks.

Upstream characterization is pinned to
[Kairos `cad43b5`, `normal_form.rs`](https://github.com/edithatogo/kairos/blob/cad43b592893579ccbea40343548fda7acd83fd0/crates/kairo-ecs-game-theory/src/normal_form.rs):
`ReplicatorDynamics::step_continuous_rk4` is the inherited clipped/normalized
Euler expression, and `run_trajectory` calls it with a fixed fitness vector.
The tiny admitted single-step fixture needs no clipping, so its named Euler
value is analytically comparable. This lane does not execute, repair or retire
that API and does not complete the feature-aware upstream caller inventory.

Tests-first commit `c052ec32708b0022b133d482826488bd1f039f7f` has the frozen
fixture and 14 executed tests: exit 1, with 27 errors (including subtests) from
explicitly unimplemented reference helpers. After implementation and grid
refinement, all 14 tests pass on CPython 3.12.14. The final PR receipt pins the
implementation SHA and fixture hashes. This local completion marker does not
close T17.04/S01 or satisfy its independent specialist review.

## T17.05.oracle: exact Nash witnesses

For complete two-player payoff tables `A[i][j]`, `B[i][j]`, let `p` and `q`
be independent probability vectors over rows and columns. The harness computes
each pure action's payoff against the opponent, each player's mixed payoff, and
the difference between the best pure payoff and the mixed payoff. A mixed
deviation is a convex combination of pure deviations, so nonpositive pure gains
are sufficient. Nonnegative exact gains must both be zero at a witness. We
record each gain, their sum and their maximum separately, without assigning
unsettled production residual names or tolerances.

Eight original rational examples contain 12 equilibrium witnesses and five
false candidates with pinned positive gains:

| Example | Independent justification |
| --- | --- |
| Strict dominance | Row 0 and column 0 each beat the other action by exactly 1 |
| Matching pennies | Two indifference equations force both probabilities to 1/2 |
| Coordination | Two diagonal pure equilibria and the unique interior profile `(2/3,1/3)` / `(1/3,2/3)` |
| Asymmetric interior | Indifference gives row probability 3/4 and column probability 1/2 |
| Rectangular 2x3 and 3x2 | Strictly dominated third action has zero probability; remaining game is matching pennies |
| Degenerate zero game | All mixed profiles are equilibria; the finite list is explicitly samples of a continuum |
| Signed single-action game | No possible deviation; the negative payoff is retained |

The strict-dominance example is derived from the inspected upstream formulas,
not from their output: `A=[[2,2],[1,1]]`, `B=[[2,1],[2,1]]` make both legacy
indifference denominators zero. `MixedNashSolver::solve_2x2_and_support_enumeration`
therefore selects half/half by its literal fallback; `LemkeHowsonSolver::solve`
delegates to it. That profile gives utility 3/2 to each player but a unilateral
gain of 1/2 each. The unique equilibrium is pure row 0/column 0. Source reading
and algebra are the evidence; no upstream execution or full caller retirement
receipt is claimed. The original source-side obligations remain with Kairos.

The independent references here are mathematical identities, not an external
software comparator. [Gambit 16.6.0's algorithm documentation](https://gambitproject.readthedocs.io/en/v16.6.0/algorithms.html)
distinguishes support/extreme-point enumeration from Lemke–Howson's accessible
equilibria. Neither algorithm is run by this harness; neither can borrow its
correctness or catalogue guarantees. Production algorithm identity, unsupported
dimensions, nonconvergence and completeness reporting still require T17.01 and
the actual future implementation. No Gambit/OpenSpiel package was installed.

All payoff, probability and residual checks use `fractions.Fraction`, with zero
tolerance. Matrix cells must be present; zero is explicit. Probabilities must
have exact unit mass and are never normalized. Negative payoffs are valid.
Domain controls cover malformed/ragged/mismatched shapes, non-finite values,
wrong dimensions, negative/wrong-mass distributions, booleans and floating-point
input. An exact positive gain of `2e-30` is retained and rejected. Reversing
action order preserves certificates, and positive affine payoff transformations
scale gains as expected. Array positions here are local mathematical coordinates,
not a substitute for the pending public identity contract.

Tests-first commit `0d512b26472f05bda793de855eeef1e4266c09f7` runs 12 tests,
exiting 1 with 43 errors (including subtests) from the missing certificate
helpers. After implementation, these 12 and the earlier 14 tests pass (26 total).
The parent T17.05/S02 remains open, with independent numerical review pending.

## T17.06.oracle: correlated-equilibrium certificates

For each recommended action and each alternative, the checker constructs the
[standard obedience inequality](https://people.duke.edu/~rnau/Nau_et_al_2004_Geometry_of_Nash_and_Correlated_Equilibria.pdf#page=4):
sum the recommendation-weighted payoff advantage of obeying over deviating.
All those inequalities, probability nonnegativity and unit mass must hold.
Rows precede columns and every ordered action pair is included, including the
rectangular 2x3 case. Conditional ratios use positive recommendation masses only;
zero-mass recommendations are recorded as `None`, without division by zero.
Weighted and conditional deviation gains are deliberately distinct diagnostics.

Let the incentive matrix be `C`, with `Cz >= 0`, and the explicit maximization
objective be `c^T z`. A supplied rational multiplier vector `lambda >= 0` and
upper bound `U` are a certificate if `d = U*1 - c - C^T lambda >= 0`.
For any feasible distribution, `U - c^T z = lambda^T C z + d^T z >= 0`.
A feasible witness attaining `U` is therefore optimal. The checker verifies
this algebra exactly; it does not run or implement an optimizer.

The distinguishing game is `A=[[0,7],[2,6]]`, `B=[[0,2],[7,6]]`, with the
explicit synthetic objective `c=A+B`. For row-major coordinates, its four
incentive slacks are:

1. `-2*z00 + z01`;
2. `2*z10 - z11`;
3. `-2*z00 + z10`;
4. `2*z01 - z11`.

The witness `z=[[0,1/4],[1/4,1/2]]` achieves **21/2**. Multipliers
`[0,3/4,0,3/4]` yield the exact dual slack `[21/2,0,0,0]`, proving the bound.
Its two off-diagonal pure Nash profiles yield objective 9. Full-support
indifference gives each player's first-action probability 1/3 and objective
**28/3**. No player can mix alone against a pure action because every pure-action
best response is strict; these are the only three Nash profiles. Any independent
CE is a Nash product, so this CE optimum strictly improves on every feasible
independent product. The harness also checks those three Nash witnesses.

Three controls separate failure modes. The interior Nash product is obedient
but falls short of the optimum by **7/6**. Multiplying the optimum's own
marginals breaks obedience, with weighted gain **3/16** and conditional gain
**1/4**. The bottom-right point mass has objective 12 but violates two
incentive constraints, so a higher objective cannot conceal infeasibility.
An asymmetric control has positive row slacks and negative column slacks,
exposing row/column indexing or sign mistakes. Other exact certificates cover
zero-mass recommendations, a rectangular game, and a negative single-action
objective. Invalid probabilities, objective shapes and dual certificates reject.

The [pinned upstream source](https://github.com/edithatogo/kairos/blob/cad43b592893579ccbea40343548fda7acd83fd0/crates/kairo-ecs-game-theory/src/normal_form.rs)
contains both the product-based `CorrelatedEquilibriumSolver::verify_and_solve`
and `ClarabelConvexSolver::solve_correlated_equilibrium_social_welfare`.
Neither is executed or qualified here. The existing Clarabel candidate remains
for the coordinator's qualified backend decision; no new optimizer dependency
or competing production implementation is introduced.

Tests-first commit `7a2cd6c7ea9a173f697acdf77e7199ca417bac0c` runs 13 tests,
exiting 1 with 34 errors (including subtests) from missing CE helpers. After
implementation all 13 pass, and all three groups run **39 tests**. These results
establish only the stated reference identities and certificate checks.

## Frozen fixture provenance and receipt

All three fixtures are original synthetic data and derivations under the
repository's Apache-2.0 licence. No external dataset, executable solver, ontology
asset or source implementation was copied. The fixed references are Butcher's
21 October 2004 classical tableau, Nau/Canovas/Hansen's 2004 CE inequalities,
Gambit documentation version 16.6.0 for algorithm-scope distinctions, and the
immutable Kairos source/obligation revision linked above. These are definitions
and source characterization, not external software validation runs.

| Fixture | SHA-256 |
| --- | --- |
| `replicator_rk4_v1.json` | `e793f8f4d2121e58d4195ee2967bfb61fbe533377863a18b3926a08b3210d5e3` |
| `mixed_nash_v1.json` | `9545116fc6aa4b1026bf6b2a6a9b509af92d8ed6ca2204e68bcdccf8e2e126e3` |
| `correlated_equilibrium_v1.json` | `1271809d3bf6a64c04d48456d42b87448f8c884cc159d0125ec68fc177a4f9ac` |

The [PR15 receipt](https://github.com/edithatogo/paignia/pull/15) records final
immutable commit/tree, harness hash, exact command results, author review and
hosted-check provenance. Earlier exact implementation heads are
`6216c480f59e0f89e33af279d31cc21a690fcb48` (14 RK4 tests) and
`a8f329c1e8b3207ffa393b8fe9ec012ba1205895` (12 Nash tests; 26 combined).
Tests-first failures are harness development evidence, not failed production
qualification. The baseline main has no such fixture target or advanced method.

## Coordinator handoff and remaining gates

At the later ownership refresh, [PR14's independent interface review](https://github.com/edithatogo/paignia/pull/14#pullrequestreview-5909550432)
had arrived for its unchanged input-profile head, finding no blocker within that
slice. It explicitly leaves the full result/identity/algorithm/status/residual
contract incomplete. This lane has not duplicated or accepted that review on
the coordinator's behalf.

For coordinator `6af93fe8-b14c-83ec-8de0-728c856bcbb3`:

1. Record this scoped receipt in the existing ownership map/Conductor integration
   only when accepted. Do not close the parent T17.04–06 tasks from fixture work.
2. Add the full `python3 -B verification/numerical_oracles/test_oracles.py`
   invocation to coordinator-owned CI, with an explicitly selected Python
   interpreter. It currently executes 39 tests. No Cargo dependency/lock/export
   change is required to run these independent checks.
3. Obtain independent specialist numerical review of the derivations, precision,
   residual definitions, coverage limits and primal/dual signs. Author review
   and an automated CI pass are not that review. No worker was created here.
4. Accept and pin the complete T17.01 semantics before an owned Rust adapter can
   map these mathematical inputs to actual production APIs, statuses, identities,
   algorithm/version, selection and catalogue claims. Freeze a separate binary64
   tolerance policy before production comparison; these zero/exact and decimal
   reference thresholds are not automatically suitable for production.
5. Keep the upstream feature-aware API/caller inventory and retain/replace/retire
   decisions open. Execute required Gambit/OpenSpiel or other approved comparator
   work and qualify the existing Clarabel candidate before backend acceptance.
   No external solver comparator, algorithm-completeness, dynamic-adapter or
   real-consumer acceptance receipt is supplied by this lane.

The authored diff is confined to the five claimed new paths. A normal revert of
these additive fixture commits is the rollback; no runtime or consumer pin moved.
Shared source/manifests/locks/CI/registry/Conductor and all upstream scopes remain
with their existing owners. PR15 stays draft/open; no merge or release is authorised.
