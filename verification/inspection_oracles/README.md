# T18.01 synthetic inspection oracle

Status: independent fixture development in progress. This directory is a
test-only analytical reference, not a production solver, runtime fallback,
public contract or scientific/consumer qualification receipt.

## Scope and ownership

Dylan authorised this bounded lane on 16 November 2026. Its
[pre-implementation claim](https://github.com/edithatogo/paignia/pull/14#issuecomment-6896113657)
reserves only these four new files: `README.md`, `inspection_2x2_v1.json`,
`reference.py` and `test_inspection.py`. Coordinator
`6af93fe8-b14c-83ec-8de0-728c856bcbb3` retains shared integration.

The reference implements the independent analytical part of
[T18.01](../../conductor/tracks/t18_illicit_enforcement_inspection_games_20261110/plan.md).
The live base is `fca2b2f8811c8cd56d27be4c9407f573d54ad1e9`.
The T18 specification and plan blobs are respectively
`b1805340504c5601c3cbbe5c51be1853d7eae630` and
`5e6f95b0274ca36da6ec7e0f3d9363245629fbec`.
All 13 published branch trees, current open PR file lists/discussions, T18
registration PR5 and the coordinator ownership record were checked. No
published intersecting fixture claim was found. This observation does not
assert direct acknowledgement from other live sessions.

PR15 at `af7b9783e63943cce3988873160166ca0adfc73f` retains its numerical-oracle
namespace and separate independent reviewer. No source or review is duplicated
there. Only this directory changes; shared Conductor, production Rust, APIs,
manifests, lockfiles, CI, registries and consumer pins stay with their owners.

## Local increment

- [~] T18.01.oracle: specify exact synthetic 2x2 inspection payoffs, pure/mixed
  best responses and commitment values; test ties, effort constraints,
  supremum attainment and distinguishing invalid shortcuts.
- [ ] Independent specialist numerical review and coordinator acceptance.

These are local fixture milestones, not a second parent-track status ledger.
T18.01 production integration remains open. T18.02 interval/credal bounds and
residual guarantees, and T18.03 consumer integration, are outside this slice.

## Prerequisites retained

Full T17.01 is incomplete. Its result/identity/algorithm/status/residual,
information-structure, selection and catalogue guarantees, plus appropriate
production tolerances, must be accepted and pinned before native comparisons.
The candidate input contract at PR14
`0c88607a142508d1fe4f1153b963fd1411b49395` admits a finite perfect-information
extensive-form graph; it does not admit simultaneous play or commitment with
an unobserved random action. This reference does not encode either game as that
profile. T17.13 remains held. Other T17 scientific/package and real-consumer
obligations remain open. All runtime game execution stays Rust-owned.

No real records or empirical calibration are used. This is one synthetic
channel with binary effort and arbitrary signed utility units. Mathematical
fixture agreement cannot establish an operational or scientific claim.
