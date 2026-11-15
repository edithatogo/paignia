# T17.04–T17.06 independent numerical fixtures

Status: scoped fixture work in progress; **no production qualification**.

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
| T17.05.oracle | Verify exact finite-game Nash witnesses and deviation residuals | README, harness, Nash fixture | [~] |
| T17.06.oracle | Verify CE incentive and objective certificates | README, harness, CE fixture | [ ] |

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
at intake. No other work, cache or build was deleted. The planned harness uses
only Python's standard library, exact rational arithmetic where possible, and
explicit high-precision decimal arithmetic for analytic exponentials; no network,
package installation, optimizer dependency or native build is required.

## Run the reference checks

From the repository root, using Python 3.12 or later:

```bash
python3 -B verification/numerical_oracles/test_oracles.py
python3 -B verification/numerical_oracles/test_oracles.py ReplicatorOracleTests
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
