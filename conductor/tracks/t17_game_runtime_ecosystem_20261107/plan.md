# T17 implementation plan — static profile qualified, full track active

T17.10 and T17.12 are open. Obtain an accepted second application receipt for
the [same qualified artifacts and semantic profile](../../../docs/validation/receipts/t17-bd880dd-handoff.json),
then run the existing bounded release eligibility check. Paignia owns the
provider leaves. The applications own their model, valuation and acceptance.

The [specification](spec.md) defines the capability and evidence boundaries.
The [22 November plan archive](plan-history-20261122.md)
preserves the dated dependencies, implementation, failed controls and reviews
for completed work. Reuse the [static package review](../../../docs/validation/t17-review-20261117.md)
and [native provider review](../../../docs/validation/provider-review-20261120.md)
for their recorded revisions and bounded claims.

## Phase A — package boundary

### [x] T17.integration-20261117 — receive merged work and resume maturity

Integrated the reviewed static provider on Rust 1.75 and stable. [Hosted run](https://github.com/edithatogo/paignia/actions/runs/44060831778)

### [x] T17.01 — expose a versioned generic game/result contract

[Game and result contract](../../../docs/architecture/game-result-contract-v1.md)

#### [x] T17.01.profile — admit the existing static input profile explicitly

#### [x] T17.01.profile-controls — retain accepted admission regressions

#### [x] T17.01.identities — admit a versioned result identity record

#### [x] T17.01.reviewed-integration — qualify the reviewed static components on main

#### [x] T17.01.doctest-ci — execute existing role-separation doctests in CI

#### [x] T17.01.diagnostics — admit finite declared diagnostic magnitudes

#### [x] T17.01.envelope/emission — admit and emit the selected native result

### [x] T17.02 — reject Kairos in the static package dependency closure

The declared static profiles have a checked, resolved dependency boundary.
[Boundary evidence](../../../docs/validation/t17-review-20261117.md#final-integrated-execution)

### [x] T17.03 — consume the packaged crates outside this workspace

Both compiler profiles qualified the unpacked `.crate` archives with the
provider checkout unavailable. [Package handoff](../../../docs/validation/receipts/t17-bd880dd-handoff.json)

## Phase B — inherited numerical methods

#### T17.04–06.oracle — separate synthetic fixture and CI receipts

The [numerical corpus](../../../verification/numerical_oracles/README.md) and
its hosted execution supplement the independent reviews below.

### [x] T17.04 — implement the declared four-stage replicator step (S01)

[RK4/Euler validation](../../../docs/validation/replicator-rk4.md) · [Kairos retirement](https://github.com/edithatogo/kairos/pull/407)

### [x] T17.05 — return a qualified mixed-Nash result (S02)

[Verified support witnesses](../../../docs/validation/mixed-nash.md) · [Kairos retirement](https://github.com/edithatogo/kairos/pull/409)

### [x] T17.06 — solve the declared correlated-equilibrium program (S03)

[Optional solver and admission](../../../docs/validation/correlated-equilibrium.md) · [Kairos retirement](https://github.com/edithatogo/kairos/pull/408)

### [x] T17.07 — train CFR/CFR+ on a supported information-set graph (S04)

[Kuhn and Leduc comparisons](../../../docs/validation/cfr.md) · [Kairos retirement](https://github.com/edithatogo/kairos/pull/406)

## Phase C — optional execution and representation gaps

### [x] T17.08 — execute a supported game trace through the Rust Kairos adapter

[Native execution contract](../../../docs/architecture/game-native-execution-v1.md) · [Kairos port acceptance](https://github.com/edithatogo/kairos/blob/f7484751c4a058581f2425ca493faec8e85e5637/conductor/contracts/game-port-acceptance.md)

### [x] T17.13 — reject semantically lossy game-profile conversions

[Bounded static conversion](../../../docs/architecture/game-semantic-conversion-v1.md)

### [x] T17.14 — expose only the declared player observation to a live policy

[Owned policy observation](../../../docs/architecture/policy-observation-v1.md)

### [x] T17.15 — isolate solver exploration from simulation random streams

[Stream isolation evidence](../../../docs/validation/game-rng-isolation.md)

### [x] T17.16 — restore qualified policy and native state for causal replay

[In-process checkpoint evidence](../../../docs/validation/game-policy-resume.md)

### [x] T17.17 — validate an ECS-independent information-set representation

[Finite perfect-recall graph](../../../docs/architecture/game-information-contract-v1.md) · [Executable UOGTO profile](https://github.com/edithatogo/UOGTO/blob/3000c231b2c102e18bf7c3294bec498f00f1976c/docs/contracts/executable-information-set-profile-v1.md)

## Phase D — real application consumers

### [x] T17.09 — external reimbursement consumer qualification

The reimbursement application qualified its Chapter 8 game against the
independent equation 8.2 benchmark. [Application #114](https://github.com/edithatogo/new-drug-reimbursement-game/pull/114) · [Receipt](../../../docs/validation/receipts/t17-reimbursement-chapter8.json) · [Execution log](../../../docs/validation/receipts/t17-reimbursement-chapter8.log)

### [ ] T17.10 — external second-application qualification

The selected second application owns this gate. The
[Innovate candidate](https://github.com/edithatogo/innovate/pull/696) ran five
pricing controls on the same `bd880dd` archives and profile as T17.09 and awaits
application acceptance. Its RustSec and release-readiness checks still fail.
Another distinct real application, such as CareOps, can supply a matching receipt.

Use the [application handoff](../../../docs/validation/game-consumer-handoff.md).
The owner runs its real model with independent expected results and records
nonzero tests, exact revisions and artifact/profile hashes. T17.03 and the
capabilities exercised by that case determine its prerequisites.

### [x] T17.11 — external reimbursement valued-payoff port

The reimbursement owner accepted valued inputs and a synthetic comparison.
[Application #118](https://github.com/edithatogo/new-drug-reimbursement-game/pull/118) · [Voiage #1386](https://github.com/edithatogo/voiage/pull/1386)

## Phase E — bounded evidence and release eligibility

### [ ] T17.12 — require two real consumers for bounded release eligibility

Paignia owns the eligibility check. After T17.03, T17.09 and T17.10, record both
accepted receipts in [the consumer manifest](../../../docs/validation/game-consumers.json)
and run `target/dev-python/bin/python scripts/validate_game_consumer_evidence.py`.
The guard currently admits the first receipt and reports ineligible.

Both applications must use the same qualified bytes and semantic profile.
The guard requires the evidence for advertised methods and explicit S01–S04
dispositions. Its current scope is graph backward induction. Optional native
execution, separate numerical claims and T17.11 have their own acceptance.
Bounded eligibility and registry publication are separate decisions.

#### [x] T17.12.guard — validate received evidence without inventing acceptance

[Guard implementation](../../../scripts/validate_game_consumer_evidence.py) · [Admission controls](../../../tests/test_game_consumer_evidence.py)

## Evidence and rollback

Retain exact revisions, hashes, commands, nonzero test counts, review findings
and scientific limits with the relevant owner receipt. Roll back with a normal
revert or the last qualified package or adapter pin. Preserve historical
algorithm and output versions.
