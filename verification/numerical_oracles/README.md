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
| T17.04.oracle | Verify four-stage/logistic replicator references and distinguishing controls | README, harness, replicator fixture | [~] |
| T17.05.oracle | Verify exact finite-game Nash witnesses and deviation residuals | README, harness, Nash fixture | [ ] |
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

Mathematical provenance, tolerances, fixture hashes, executed receipts and the
precise coordinator integration request follow with each implemented slice.
