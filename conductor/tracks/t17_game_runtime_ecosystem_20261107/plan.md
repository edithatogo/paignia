# T17 implementation plan — static profile qualified, full track active

Choose a coherent, reviewable leaf and split it when behavior, ownership or
reviewability requires it. A nonexistent target or a filtered Cargo run with zero
matching tests is not a pass. Paths are repository-relative unless prefixed with
a consumer name. The [original plan and completed-leaf evidence](plan-history-20261122.md)
are preserved verbatim as the 22 November historical snapshot.

`local` means routine implementation, diff review, and focused checks.
`specialist` adds independent agent review at the named numerical or public
contract boundary. Neither label adds a repeated human approval gate. Before a
phase closes, record exact command results and relevant CI once; rerun broader
checks only for an identified risk or required repository gate. Provider pin
changes require their own qualified migration leaf, not a hidden extra edit.

The historical receiving handoff is [issue 100](https://github.com/edithatogo/new-drug-reimbursement-game/issues/100),
coordinated with [Innovate PR 570, G01–G04](https://github.com/edithatogo/innovate/pull/570).
Kairos `K01.games` supplies `conductor/contracts/game-extraction-map.md`;
`K01.port` supplies `conductor/contracts/game-execution-port.md`. Their contracts
are recorded in merged [Kairos PR 382](https://github.com/edithatogo/kairos/pull/382);
they are merged contract inputs, not scientific or adapter acceptance evidence.

Dependency lines control execution; phase headings group related work. The
bounded static path is T17.01–T17.03, two selected real consumer cases in
T17.09–T17.10, then T17.12. It does not require the optional native adapter or
all future numerical methods. Additional capabilities require their own named
receipts. Leaf status below records bounded acceptance; partial qualification
does not close T17.
The [naming supplement](paignia-identity-and-promotion.md) records the bounded
seed rename and source-level consumer slices without adding competing leaves.

## Static provider acceptance — 17 November 2026

[Accepted source and Conductor review](../../../docs/validation/t17-review-20261117.md):
`bd880dd756d2ba56cc5f479bc078cea31b53c1d1`, hosted native run
[44060831778](https://github.com/edithatogo/paignia/actions/runs/44060831778).
Both Rust 1.75 and stable 1.100 pass 232 default / 267 all-feature native tests,
five doctests, 58 retained independent native controls, 54 Python guards and
seven artifact profiles (40 executions of 12 distinct cases). Frozen numerical
39 / inspection 28 controls and privacy pass. Offline security remains queued.

**Completed bounded provider leaves:** T17.01, .02, .03, .04, .05, .06, .13, .14.
Their independent specialist reviews and final integrated gates are accepted.
The unchanged specification assigns corrected reusable methods to Paignia;
Kairos Track 110 separately retains source regression, migration and retirement.
Native wire serialization, a general normal-form envelope and a mandatory Gambit
run were not added as unsupported extra prerequisites.

## Current acceptance — 22 November 2026

T17.09 records the first application artifact receipt and T17.17 has the
accepted executable profile. The merged Kairos `K01.port` receipt qualifies
T17.08, T17.15 and T17.16. T17.07 is complete after the independent Leduc
comparison and the legacy Kairos CFR retirement. T17.11 accepts the
application's valued-payoff port and synthetic comparison. T17.10 and T17.12
remain open. The bounded release check still requires a second matching
application receipt.
[Kairos port acceptance](https://github.com/edithatogo/kairos/blob/f7484751c4a058581f2425ca493faec8e85e5637/conductor/contracts/game-port-acceptance.md)

Kairos withdrew the legacy S01 RK4 export at `ae9ae116` and the S03
correlated-equilibrium exports at `01156707`. Paignia retained the two S03
oracles and their weighted incentive threshold before the Kairos retirement.
Kairos retired the legacy mixed-strategy exports (S02) at `9f51775` after
removing the product-equilibrium caller. The exact-base native coverage gate
passed at 33,121/38,353 against 33,170/38,412. [Kairos #409](https://github.com/edithatogo/kairos/pull/409) · [Coverage result](https://github.com/edithatogo/kairos/actions/runs/44813475869/job/136563716910)
[Kairos #407](https://github.com/edithatogo/kairos/pull/407) ·
[Paignia #64](https://github.com/edithatogo/paignia/pull/64) ·
[Kairos #408](https://github.com/edithatogo/kairos/pull/408)

Earlier dated paragraphs retain historical source, failure and review evidence,
including then-future paths and then-pending PR/merge wording. The current
checkboxes and linked receipt control status. No old result is relabelled as
new execution, and no remaining requirement is withdrawn to close the track.

## Phase A — package boundary

### [x] T17.integration-20261117 — receive merged work and resume maturity

- [Accepted implementation history and evidence](plan-history-20261122.md).

### [x] T17.01 — expose a versioned generic game/result contract

- [Accepted implementation history and evidence](plan-history-20261122.md).

#### [x] T17.01.profile — admit the existing static input profile explicitly

- [Historical evidence](plan-history-20261122.md).

#### [x] T17.01.profile-controls — retain accepted admission regressions

- [Historical evidence](plan-history-20261122.md).

#### [x] T17.01.identities — admit a versioned result identity record

- [Historical evidence](plan-history-20261122.md).

#### [x] T17.01.reviewed-integration — qualify the reviewed static components on main

- [Historical evidence](plan-history-20261122.md).

#### [x] T17.01.doctest-ci — execute existing role-separation doctests in CI

- [Historical evidence](plan-history-20261122.md).

#### [x] T17.01.diagnostics — admit finite declared diagnostic magnitudes

- [Historical evidence](plan-history-20261122.md).

#### [x] T17.01.envelope/emission — admit and emit the selected native result

- [Historical evidence](plan-history-20261122.md).

### [x] T17.02 — reject Kairos in the static package dependency closure

- [Accepted implementation history and evidence](plan-history-20261122.md).

### [x] T17.03 — consume the packaged crates outside this workspace

- [Accepted implementation history and evidence](plan-history-20261122.md).

## Phase B — inherited numerical methods

These leaves own the corrected generic implementations, coordinated with Kairos
Track 110's source regressions and transfer/retirement decisions. Do not maintain
two corrected solver implementations. A bounded release may leave S01–S04
pending and explicitly unsupported; any advertised method must complete its own
oracle and result contract. Exclusion is not scientific completion.

#### T17.04–06.oracle — separate synthetic fixture and CI receipts

- [PR15](https://github.com/edithatogo/paignia/pull/15) retains five fixture paths
  at `af7b9783e63943cce3988873160166ca0adfc73f`; author ownership is unchanged.
  [Independent numerical review accepted](https://github.com/edithatogo/paignia/pull/15#issuecomment-6896316061):
  39 unchanged-harness tests, 10 reviewer tests, seven detected mutations, no
  corrective edit. Reviewer `6af9c0e1-b6a0-83ec-ad3b-62b228818763` has no residual
  assignment and may archive. This is bounded synthetic agent-review acceptance.
- Coordinator-owned [PR17](https://github.com/edithatogo/paignia/pull/17) changes
  only `.github/workflows/ci.yml`, stacked on that fixed PR15 head. Exact head
  `89da0a0261cd5908aa7c1830a536283a03bcec1c` has actual hosted CPython 3.12.14
  evidence: all 39 oracle tests pass; zero/partial/skipped/failed reports reject.
  Existing native stable/MSRV/privacy gates pass. Retain the equal head/merge
  tree provenance in its [receipt](https://github.com/edithatogo/paignia/pull/17#issuecomment-6896316260).
- Both PRs are unmerged. New PR18 action-pin overlap is serialized by the sole
  integrator; combined-tree evidence is required before integration.
- These receipts do not complete T17.04–06/S01–S03. Full accepted/pinned T17.01
  result semantics, justified production tolerance/API mapping, upstream
  caller/retirement, comparator/backend and package/consumer obligations remain.
  T17.13 remains held; no T18/native simulation or consumer activation follows.

### [x] T17.04 — implement the declared four-stage replicator step (S01)

- [Accepted implementation history and evidence](plan-history-20261122.md).

### [x] T17.05 — return a qualified mixed-Nash result (S02)

- [Accepted implementation history and evidence](plan-history-20261122.md).

### [x] T17.06 — solve the declared correlated-equilibrium program (S03)

- [Accepted implementation history and evidence](plan-history-20261122.md).

### [x] T17.07 — train CFR/CFR+ on a supported information-set graph (S04)

- [Accepted implementation history and evidence](plan-history-20261122.md).

## Phase C — optional execution and representation gaps

### [x] T17.08 — execute a supported game trace through the Rust Kairos adapter

- [Accepted implementation history and evidence](plan-history-20261122.md).

### [x] T17.13 — reject semantically lossy game-profile conversions

- [Accepted implementation history and evidence](plan-history-20261122.md).

### [x] T17.14 — expose only the declared player observation to a live policy

- [Accepted implementation history and evidence](plan-history-20261122.md).

### [x] T17.15 — isolate solver exploration from simulation random streams

- [Accepted implementation history and evidence](plan-history-20261122.md).

### [x] T17.16 — restore qualified policy and native state for causal replay

- [Accepted implementation history and evidence](plan-history-20261122.md).

### [x] T17.17 — validate an ECS-independent information-set representation

- [Accepted implementation history and evidence](plan-history-20261122.md).

## Phase D — real application consumers

### [x] T17.09 — external reimbursement consumer qualification

- [Accepted implementation history and evidence](plan-history-20261122.md).

### [ ] T17.10 — external second-application qualification

- Owner: the selected distinct application, currently
  [Innovate G01–G04](https://github.com/edithatogo/innovate/pull/570).
- Behavior: a second actual model consumes the same qualified Paignia artifacts
  and profile and retains its games/payoffs locally. Another wrapper or toy in
  this repository does not count.
- Depends on: T17.03 and only the algorithm/native capabilities actually used.
- Files: none in this provider leaf; the consumer owns its implementation and
  receipt. It executes its real model, records independent expected results and
  exact versions/hashes, and supplies the receipt for T17.12.
- Five pricing controls passed in the Innovate candidate using the same
  qualified `bd880dd` archives and semantic profile as the reimbursement case.
  The receipt awaits application acceptance. The draft's RustSec and
  release-readiness checks are failing.
  [Innovate candidate](https://github.com/edithatogo/innovate/pull/696)
- Static reimbursement plus Innovate is an eligible pair. CareOps can supply a
  different real case; an optional Kairos adapter is not a blanket prerequisite.

### [x] T17.11 — external reimbursement valued-payoff port

- [Accepted implementation history and evidence](plan-history-20261122.md).

## Phase E — bounded evidence and release eligibility

### [ ] T17.12 — require two real consumers for bounded release eligibility

#### [x] T17.12.guard — validate received evidence without inventing acceptance

- Implement a deterministic fail-closed validator for actual package receipts,
  two distinct external application receipts with matching provider artifacts
  and semantic profile, and explicit S01–S04 dispositions. Receipt files are
  content-hashed; publication and full-track completion are separate decisions.
- Scope: `scripts/validate_game_consumer_evidence.py`, its focused test file,
  `docs/validation/game-consumers.json`, and this plan. The checked-in report
  remains ineligible until both application receipts are accepted.
- Tests distinguish two valid synthetic receipt records from duplicate, toy,
  mismatched, missing, corrupted, failed or unsupported-method evidence. These
  test records exercise admission only and never count as actual consumers.
- Tests-first `76b3168` executes 12 tests with 20 errors from the explicit
  unimplemented validator. The initial implementation executes 13 passing tests,
  including malformed-data rejection. The initial manifest reported
  ineligible (exit 1) before the immutable artifacts, profile and application
  receipts arrived. Owner review of the content-bound receipts remains required.
  Independent review subsequently found identity canonicalization, profile
  binding, scientific-method evidence and feature-coverage gaps. Corrected
  admission now passes 30 controls; the real manifest remains ineligible.
- Current evidence: `docs/validation/receipts/t17-bd880dd-handoff.json` binds a
  clean immutable provider, exact profile and both qualified archive bytes,
  independently matched to the downloaded hosted artifact. The current guard
  admits the first reimbursement receipt and still returns exit 1 because
  eligibility requires a second distinct application receipt.
  Its selected release capability is backward induction through the graph
  profile; separate bimatrix/ODE claims retain their method/input requirements.

- Behavior: report bounded release as ineligible until two distinct real consumer
  receipts identify the same qualified static artifacts and passing profile.
- Depends on: T17.03, T17.09 and T17.10. Numerical/native receipts apply only to
  the supported capabilities claimed by the artifacts and consumer cases.
- Execution: local evidence validation; reuse the numerical/public-contract
  reviews already recorded instead of requesting repeated human approval.
- Release evidence is tracked in `docs/validation/game-consumers.json`
  and `extraction-manifest.json`. The guard and its controls are in
  `scripts/validate_game_consumer_evidence.py` and
  `tests/test_game_consumer_evidence.py`.
- Run `target/dev-python/bin/python -m unittest discover -s tests -p
  'test_game_consumer_evidence.py'`,
  `target/dev-python/bin/python scripts/validate_game_consumer_evidence.py` and
  `cargo test --locked -p paignia-core -p paignia-solve`.
  Reject duplicated application identities, mismatched artifacts/profiles,
  toy-only cases, absent/failed receipts and advertised capabilities without
  their method evidence. Accept two real static consumers, including the
  reimbursement application plus Innovate, without requiring optional Kairos
  integration, T17.11 or every future algorithm. Record S01–S04 individually as
  supported with evidence, explicitly unsupported/pending, or withdrawn with a
  replacement decision; omission is not allowed. Pending excluded methods keep
  their owners and completion tasks open. This bounded eligibility report does
  not close the whole track, publish packages or change consumer pins. The
  selected repository already exists; its creation is not this gate. Full track closure still requires its remaining obligations and quality
  gates; the first useful static package does not wait for that broader closure.

## Evidence and rollback

Each completed leaf records exact provider/consumer revisions, changed files,
commands, test counts, and review findings in its local evidence receipt. Failed
scientific controls remain visible. Mark a leaf complete only after its behavior
and acceptance evidence exist. A numerical implementation failure preserves an
explicit unsupported status and requires its supported replacement decision;
moving files or changing labels never supplies validation evidence.

Rollback uses a normal revert or the last qualified package/adapter revision,
retaining original algorithm/output versions for reproducibility. The provider transfer can be reverted normally, and consumers can restore a
previous qualified immutable pin. Historical receipts retain their original
repository/package names. The application test and its acceptance remain with
its owner; no second local solver is introduced as a rollback fallback.
