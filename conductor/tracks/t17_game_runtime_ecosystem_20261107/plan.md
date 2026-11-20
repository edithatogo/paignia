# T17 implementation plan — static profile qualified, full track active

Each leaf changes one behavior and touches at most five files. Paths are
repository-relative unless prefixed with a consumer name. New paths and test
targets below are explicitly **future**: create the test first, observe the
distinguishing failure, then implement the behavior. A nonexistent target or a
filtered Cargo run with zero matching tests is not a pass. If a leaf needs more
files or another behavior, split it into a linked leaf before implementation.

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

## Current acceptance — 17 November 2026

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

**Nine leaves remain open:** .07, .08, .09, .10, .11, .12, .15, .16 and .17.
Their external contracts and real application evidence remain necessary. The
package handoff now binds actual immutable source/profile/archive evidence;
the eligibility guard's sole current error is the missing two application
receipts. Registry publication stays disabled and full-track archival is ineligible.

Earlier dated paragraphs retain historical source, failure and review evidence,
including then-future paths and then-pending PR/merge wording. The current
checkboxes and linked receipt control status. No old result is relabelled as
new execution, and no remaining requirement is withdrawn to close the track.

## Phase A — package boundary

### [x] T17.integration-20261117 — receive merged work and resume maturity

- User instruction: implement T17, address blockers, review, apply fixes and
  archive only after its actual completion criteria are met. The receiving
  baseline is `5642c5e226949c050d1600e021420cb1335cf492`; all earlier PRs are
  closed. Earlier pending-review/merge statements below are historical receipts.
- Immediate bounded correction: restore the Rust 1.75-compatible `thiserror`
  closure previously accepted in PR27. The later PR20 update introduced
  `thiserror 2.0.21`, which requires Rust 1.77; hosted job
  [133915868902](https://github.com/edithatogo/paignia/actions/runs/44047236919/job/133915868902)
  fails before native tests. Preserve the declared compiler contract and add
  an explicit Renovate constraint. Scope: two crate manifests, lockfile,
  `renovate.json` and this receipt.
- Correction verified locally on Rust 1.75.0: the unchanged baseline first
  fails with `thiserror-impl 2.0.21` requiring Rust 1.77 (exit 101). Restoring
  the exact PR27 manifests/lock runs 64 native tests, three compile-fail
  doctests and two source-consumer tests successfully; formatting, Clippy
  with warnings denied, extraction and warnings-denied documentation pass.
  The Renovate rule retains the compatible `thiserror` major until a deliberate
  compiler migration. Hosted validation remains required for the proposed head.
- Parallel package implementation owns only the two existing validation scripts
  and their new focused Python tests. It will add resolved normal/build closure
  checks and actual unpacked Cargo artifact consumption. Shared CI, native
  source, metadata and integration remain coordinated here.
- Required final checks: both declared Rust profiles, native tests/doctests,
  extraction and package checks, reference fixtures, independent review of new
  public/numerical interfaces, and a fresh acceptance/status reconciliation.
  An incomplete external obligation or unimplemented method is not archived as
  completed; registry publication remains disabled.
- CI reconciliation: the previously merged inspection-reference job from PR23
  was absent from the receiving tree, while its fixture files remained. Restore
  that exact 28-test execution/retention job with the current action pins. All
  28 unchanged reference tests and the original strict count guard pass locally
  on CPython 3.12.14. No fixture or numerical claim changes; this restores an
  existing shared gate rather than starting T18 production implementation.

### [x] T17.01 — expose a versioned generic game/result contract

- Behavior: validate the declared semantic and result schema version, rejecting
  unsupported versions explicitly while preserving UOGTO identifiers.
- Depends on: the transferred finite-game seed recorded in
  `extraction-manifest.json`. The [incubator T02 source history](https://github.com/edithatogo/new-drug-reimbursement-game/tree/f6c1c87b6b00875b33c1e19401c1de09481913ce/conductor/tracks/t02_rust_runtime_20260727)
  is provenance, not an unresolved local Paignia prerequisite.
- Execution: specialist, public contract and UOGTO semantics.
- Files (4): `crates/paignia-core/src/lib.rs`,
  `crates/paignia-core/tests/runtime_contract.rs` (future),
  `docs/architecture/game-runtime-contract-v1.md` (future),
  `fixtures/conformance/game-runtime-contract-v1.json` (future).
- Check: `cargo test --locked -p paignia-core --test runtime_contract`
  (future target); include unknown-version and unsupported-representation cases.
  Freeze the supported representation independently of ECS, plus specification,
  instance, session, policy and trace identity. Declare chance-zero/mass rules,
  complete versus intentional sparse-zero payoffs, tolerance and tie selection.
  Include the observed string/generational-ID and semantic conversion negatives
  from the source map. A stable result names algorithm/version, supported class,
  status and residuals; backend diagnostics are optional. Solver diagnostic
  traces and realized causal traces are different record types. Core references
  no Kairos state, clock or simulation RNG; unknown information structures reject.

#### [x] T17.01.profile — admit the existing static input profile explicitly

- One behavior: `GameSpecification::validate_for_profile` admits only the exact
  versioned finite, perfect-information, intentional sparse-zero input profile,
  rejects unknown profile strings with a structured error, then uses existing
  specification/graph validation. Existing `validate` and solver APIs are retained.
- Ownership and five-file scope: the [current ownership map](../../../docs/integrations/parallel-plan-reconciliation-20261108.md#scoped-ownership-update--15-november-2026).
  No competing published claim was found; recheck before each source write.
- Tests first: `cargo test --locked -p paignia-core --test runtime_contract`.
  Exercise exact profile/schema rejection, full-IRI preservation, graph-invalid
  controls and the inherited chance/sparse-payoff semantics. Record the initial
  failure honestly; zero tests is not acceptance.
- Qualification: candidate input profile only. The full result envelope,
  algorithm/termination/completeness guarantees, identity roles and conversion
  contracts remain pending in T17.01. T17.13 source work stays blocked until
  those required semantics are accepted and pinned. No dynamic adapter work.
- Merge gates: required native checks on stable and Rust 1.75, plus independent
  specialist public-contract review. Do not close the parent from this slice.
- Implemented and locally verified on 15 November 2026; merge remains held.
  The [candidate profile contract](../../../docs/architecture/game-runtime-contract-v1.md)
  freezes the input identifier and inherited rules. The separate
  `ProfileValidationError` keeps the original public error enum unchanged.
  Tests-first commit `25d00f40fed5d2225db82945ff1f1f40e825bd2c` fails to compile
  against the old core with E0432/E0599 (exit 101: missing API, not a passing or
  zero-test run). After implementation, the focused target runs 15 passing tests.
  Rust/cargo 1.99.1: workspace/all-targets 32 passed (6 core + 15 contract + 11
  solve), extracted-source consumer 2 passed, formatting, warnings-denied Clippy,
  extraction guard and warnings-denied rustdoc all passed. Dependencies were
  resolved offline against the unchanged lock. See the ownership-map receipt for
  source hashes and the exact PR handoff. These results do not close T17.01.
- Independent [review 5909550432](https://github.com/edithatogo/paignia/pull/14#pullrequestreview-5909550432)
  at `0c88607a142508d1fe4f1153b963fd1411b49395` was
  [accepted by the coordinator](https://github.com/edithatogo/paignia/pull/14#issuecomment-6891995075).
  Its 11-test synthetic harness and nonblocking supplemental controls are
  received. Hosted stable/MSRV/privacy checks cover the equal head/merge tree.
  The reviewer has no residual assignment and can be archived. This is scoped
  agent-review acceptance, not a formal GitHub approval or full T17.01 closure.

#### [x] T17.01.profile-controls — retain accepted admission regressions

- One behavior: pin existing input-profile admission using selected probes from
  the accepted independent review; no production or solver behavior changes.
- Owner: sole shared coordinator; [three-file claim](../../../docs/integrations/parallel-plan-reconciliation-20261108.md#continued-bounded-work--16-november-2026).
- Parent: exact reviewed PR14 head above. Preserve that head and its review;
  publish this increment separately. Existing and supplemental native cases
  must execute, with required native stable/MSRV checks on the new head.
- Cover legacy typed errors and paths, six blank identifier roles, immediate
  binary64 mass-boundary neighbours, shared descendants, signed zero/subnormal
  weights and invalid zero-weight branches. These are regressions for already
  passing behavior, not newly discovered production failures or qualification
  of the remaining result envelope. New interface work needs its own leaf.
- Implemented: ten supplemental tests retain nine independent probe groups;
  splitting the legacy-error case keeps each test bounded. Local Rust 1.75.0
  runs 25 contract / 42 total native tests and two source-consumer tests; format,
  Clippy, extraction guard and rustdoc pass. No production source changed.
  [PR16](https://github.com/edithatogo/paignia/pull/16) records the exact-head
  hosted handoff; it remains draft and does not close T17.01.

#### [x] T17.01.identities — admit a versioned result identity record

- One behavior: reject unsupported identity-schema versions or blank supplied
  identities without rewriting their strings. Specification, instantiated game
  and solver-run IDs are required and have distinct types. Session, policy
  artifact and solver-diagnostic trace IDs are separately typed and optional;
  static validation never invents them or a causal execution trace.
- Owner: sole shared coordinator; parent PR16 exact head
  `177d76167a8c58299be4ab2aba26da7f44c96964`; [five-file pre-implementation claim](../../../docs/integrations/parallel-plan-reconciliation-20261108.md#continued-bounded-work--16-november-2026).
- Check: `cargo test --locked -p paignia-core --test runtime_contract`, plus
  role-confusion compile-fail doctests and required native stable/MSRV checks.
  Observe distinguishing unknown-version/blank-ID failures before implementing
  admission; preserve legacy input structs, error variants and solver source.
- Execution: specialist public-contract review at the new exact head. Existing
  review acceptance covers the unchanged input profile only; no completed
  worker is reassigned and no new reviewer is created here.
- Limits: component schema only, not a complete result envelope, wire format,
  identity-to-source binding, conversion or numerical qualification. Full
  result/status/algorithm/selection/diagnostic/catalogue/serialization obligations
  remain; T17.13 and T18 native-comparison prerequisites stay held.
- Implementation candidate in [PR21](https://github.com/edithatogo/paignia/pull/21):
  `ResultIdentityRecord`, six distinct identity types and a separate structured
  admission error; exact identity component schema `1.0.0`. No existing core
  bytes, solver, manifest/lock or workflow change.
- Red evidence at `733ec013ea1b3cd7eee23a18b45f56c906143f6d`: the explicit
  no-check prototype runs all 35 contract tests, with 30 passes and five failures
  (exit 101), including an unknown-schema assertion. This is a development
  control, not an inherited production defect. The implemented validator passes
  all 35 contract tests, 52 workspace tests, three role-confusion compile-fail
  doctests and two extracted-source consumer tests on Rust 1.75.0; format,
  warnings-denied Clippy, extraction guard and warnings-denied rustdoc pass.
  The final exact-head hosted receipt is recorded on PR21. Independent primary
  review `5912344623` and supplemental review `5912337436` at
  `afacdb12600ccf37d32962387790f2ce9647a3c4` are now
  [accepted](https://github.com/edithatogo/paignia/pull/21#issuecomment-6902551304).
  No necessary source fix or residual reviewer duty remains. Their reproducible
  harnesses and nonblocking recommendations are retained without duplicating
  ownership or aggregating overlapping test counts. This accepts the bounded
  component, not full T17.01, conversion or consumer qualification.

#### [x] T17.01.reviewed-integration — qualify the reviewed static components on main

- One behavior: integrate the unchanged PR14/16/21 static-contract source and
  tests with current main `c7367b93244add88bb76f0ae34fcd244787ebab6`.
- Owner: sole shared coordinator; [pre-integration claim](https://github.com/edithatogo/paignia/pull/14#issuecomment-6902565416).
  Five paths relative to main: core lib, runtime-contract tests, contract doc,
  this plan and the existing ownership map. Only the two owning context files
  change relative to the accepted PR21 source snapshot.
- Gates: fresh exact-tree stable/MSRV/native/privacy execution, composition
  review linked to unchanged-source independent evidence, and receiving-main
  verification after any scoped development merge. No old run is relabeled.
- All old heads and histories remain. Full T17.01 and all scientific, package,
  consumer and T17.13 holds are independent of this development integration.
- [PR24](https://github.com/edithatogo/paignia/pull/24) passed exact-head
  composition review `5913350938`, hosted stable/MSRV/privacy and local gates.
  It merged as `969d2d22cc38a205c6087902a88cd1cd535aa96e`; receiving tree
  `9fa72d34cc8e86b2141298923c248ebfe34cfa65` equals the reviewed/tested tree
  and preserves PR14/16/21 ancestry. Post-merge workflow execution was queued
  at this source snapshot; its final receiving receipt belongs to PR24.

#### [x] T17.01.doctest-ci — execute existing role-separation doctests in CI

- One behavior: run `cargo test --locked -p paignia-core --doc` in both hosted
  compiler jobs; no existing command, permission, trigger or check is removed.
- Owner: sole coordinator, serially after reviewed integration. One workflow
  path only. This accepts the supplemental review's nonblocking suggestion;
  it does not reopen either completed reviewer or change the public interface.
- Implemented in [PR25](https://github.com/edithatogo/paignia/pull/25) at
  `4cb88678e7544e4ff63bc5ab669d2d6546885790`: one added workflow step, preserving
  all existing jobs/settings. Exact command passes locally; two positive/seven
  negative summary controls reject zero/partial/skipped execution. Hosted stable
  and MSRV each execute 52 native, three doctests and two source-consumer tests;
  privacy passes. Receiving-main integration is recorded separately on PR25.

#### [x] T17.01.diagnostics — admit finite declared diagnostic magnitudes

- One behavior: validate an exact versioned component containing a residual or
  exploitability magnitude and a caller-supplied tolerance. Reject non-finite
  or negative numbers in a defined field order; preserve all supplied bits.
- Owner: sole coordinator, serially after the reviewed component integration.
  At most five paths: core lib, existing runtime-contract tests, contract doc,
  this plan and the same ownership map. No solver, dependency, fixture or CI edit.
- Implementation parent: fixed PR25 head
  `4cb88678e7544e4ff63bc5ab669d2d6546885790`, which contains the reviewed
  composition in PR24. Hosted integration jobs are queued at this claim;
  development proceeds on the fixed proposed stack, with all merges held until
  their actual current-tree gates and receiving-main verification are satisfied.
- Component: `ResultDiagnosticRecord`, exact schema `1.0.0`, typed
  `ResultDiagnosticKind::{Residual, Exploitability}`, `value` and `tolerance`.
  Check schema first, then value, then tolerance; finite checks precede negative
  checks within each number. Signed zero is retained and zero tolerance is legal.
  Admission does not compare value with tolerance or decide convergence.
- Tests first: unsupported version, NaN/infinity/negative value and tolerance,
  boundary magnitudes and non-mutating admission; required native checks.
  A fresh independent public-contract review is required for the new head.
- This is structural admission only. No norm, units, algorithm result, source
  binding, convergence, equilibrium certificate or production binary64 tolerance
  policy is established. Full envelope/status/selection/catalogue/serialization
  obligations and T17.13/T18 prerequisites remain open.
- Implemented candidate in [PR26](https://github.com/edithatogo/paignia/pull/26).
  Ownership/plan commit `28b3724bbe79b4918e75f3130de59b8a7f35b448` precedes
  tests-first commit `2fda0b6a687189c58d0ecbbce362a4f79cb59b40`. The explicit
  no-check prototype actually runs 46 contract tests: 39 pass, seven fail,
  exit 101. The final validator passes all 46 contract tests, 63 workspace tests
  (6 core + 46 contract + 11 solver), three compile-fail doctests and two
  extracted-source tests on Rust 1.75. Format, warnings-denied Clippy,
  extraction guard and warnings-denied rustdoc pass. Existing source outside the
  additive diagnostic block, prior tests, solver, manifests/lock and CI remain
  byte-identical to the fixed parent.
- Final exact-head hosted results and author review belong to the PR26 receipt.
  **Fresh independent public-contract review is still required for this new
  component.** Accepted PR21 reviews do not cover it; no new reviewer is created.

#### [x] T17.01.envelope/emission — admit and emit the selected native result

The native envelope adds 31 source/instance/policy admission tests. The opt-in
emitter adds nine controls, including typed budget/output failures and overflow
in an unchosen branch. It retains the declared legacy selection behavior.
Independent review accepted the combined interface and nine external controls,
retained in `verification/result_contract_review`. It establishes a computed
native record, not an exact-equilibrium or serialization claim. Integrated
stable/MSRV evidence is pinned in the current receipt above.

### [x] T17.02 — reject Kairos in the static package dependency closure

- Behavior: fail the boundary guard if either generic crate gains a direct or
  transitive Kairos dependency; permit it only in the optional adapter package.
- Depends on: T17.01.
- Execution: local.
- Files (3): `scripts/validate_extraction.py`, `tests/test_extraction.py`
  (future), `.github/workflows/ci.yml`.
- Checks: `python -m unittest discover -s tests -p 'test_extraction.py'`
  (future suite), `python scripts/validate_extraction.py`, and
  `cargo test --locked -p paignia-core -p paignia-solve` (existing packages).
  The guard must inspect Cargo's resolved dependency graph and reject a seeded
  transitive violation, not just search top-level manifest strings. Inspect
  normal/build dependencies under the supported default, minimal and optional
  feature profiles; include a violation hidden behind an optional feature or
  build dependency. Keep optimization in solver/backend scope and avoid making
  semantic tooling, serialization or native integration mandatory in game core.

### [x] T17.03 — consume the packaged crates outside this workspace

- Current-Cargo compatibility correction (17 November): native CI at published
  `b9bcb435746c72608c36870152972252b7dc0dca` passed Rust 1.75 but stable Cargo
  1.100 could not create the solver archive: its generated library archive lock
  tried to resolve unpublished `paignia-core` through crates.io. Packaging both
  workspace libraries together reproduced the same failure. The focused new
  unpublished-sibling control first fails at the missing packaging helper.
  Use Cargo's supported `--exclude-lockfile` only when available; Cargo 1.75
  already omits library locks. Record this policy and each actual archive's
  contents. Provider and isolated-consumer lock hashes/dependency identity checks
  remain required; this qualifies library consumption, not `cargo install
  --locked` or registry publication. See the official
  [Cargo package reference](https://doc.rust-lang.org/cargo/commands/cargo-package.html).

- Behavior: build a standalone consumer from local package artifacts without
  importing application code or relying on the provider source tree.
- Depends on: T17.02.
- Execution: local; preserve the agreed public contract.
- Files (5): `crates/paignia-core/Cargo.toml`,
  `crates/paignia-solve/Cargo.toml`, `Cargo.lock`,
  `scripts/validate_game_packages.py` (extracted-source smoke exists; .crate qualification still pending),
  `tests/test_game_packages.py` (future).
- Checks: `cargo test --locked -p paignia-core -p paignia-solve`;
  `python scripts/validate_game_packages.py` (existing source smoke). The complete artifact check must
  unpack the produced artifacts into a temporary standalone consumer, resolve
  their local artifact dependency explicitly, run `cargo generate-lockfile
  --manifest-path <temporary-consumer>/Cargo.toml`, then run `cargo test --locked
  --manifest-path <temporary-consumer>/Cargo.toml`. Make the provider checkout
  unavailable and exercise every advertised feature profile with nonzero tests.
  Include transferred generated sources/fixtures inside their package, or mark
  that feature unsupported: Kairos's current external `include!` and ontology
  fixture paths must not survive accidentally. Qualify the selected compiler
  profile against the declared MSRV rather than assuming Kairos's 1.99 and this
  workspace's 1.75 declarations are interchangeable. Record artifact hashes;
  no publication or repository extraction is part of this leaf.
- Seed progress: source packages are named `paignia-core` and
  `paignia-solve`; the solver declares a versioned local core dependency. Native
  development checks pass locally and on hosted stable/MSRV profiles; clean
  artifact-consumption requirements remain pending.
  Paignia now passes isolated extracted-source consumption with two native
  oracle tests on Rust 1.75. The first smoke found a manifest inheritance
  substring collision; anchored assignments and parsed-field checks fixed it,
  and the actual native smoke passed. This does not establish published .crate
  artifacts, full future feature coverage or real
  consumer receipts. Transfer does not complete this leaf.
  [Merged-source hosted CI](https://github.com/edithatogo/paignia/actions/runs/42421109771)
  at `12b22ce7571b9c590405bf53c2c1c5862b49cd64` passed both profiles with
  15 library tests and two extracted-source consumer tests each.

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

- Acceptance: 28 author tests and ten independent native controls; actual RK4 artifact calls and both compiler profiles pass. T17.04 provider implementation is complete; Kairos Track 110/S01 source disposition remains external.

- Behavior: advance supported continuous replicator dynamics by the declared
  four-stage RK4 update, retaining the legacy Euler result under its own version.
- 17 November implementation scope: solver module/export, `replicator.rs`,
  `replicator_rk4.rs` tests and the method document. Consume the frozen independent
  oracle literals unchanged. Test four stage times, logistic fourth-order
  refinement, explicit simplex/non-finite failures and one-sided discontinuity
  segments. Upstream retirement and full-track acceptance remain separate.
- Depends on: T17.01; exact upstream API/caller inventory recorded first.
- Execution: specialist, numerical integration.
- Files (4): `crates/paignia-solve/src/lib.rs`,
  `crates/paignia-solve/src/replicator.rs`,
  `crates/paignia-solve/tests/replicator_rk4.rs`,
  `docs/validation/replicator-rk4.md`.
- Check: `cargo test --locked -p paignia-solve --test replicator_rk4`
  (native target). Use analytic logistic evolution and step-refinement order;
  include time-varying fitness, discontinuity handling, positivity/conservation,
  and a negative control that fails for Euler relabelled as RK4.

### [x] T17.05 — return a qualified mixed-Nash result (S02)

- Acceptance: corrected 39-test method, 14 independent controls, unchanged tolerances, actual artifact witness/cancellation calls and both compilers pass. T17.05 is complete for finite verified support witnesses; Kairos S02 source disposition remains external.

- Behavior: solve the explicitly supported game class with the named mixed-Nash
  algorithm, returning explicit unsupported/nonconverged statuses elsewhere.
- Depends on: T17.01; exact upstream API/caller inventory recorded first.
- Execution: specialist, equilibrium algorithms.
- Files (5): `crates/paignia-solve/src/lib.rs`,
  `crates/paignia-solve/src/mixed_nash.rs`,
  `crates/paignia-solve/tests/mixed_nash.rs`,
  `verification/numerical_oracles/mixed_nash_v1.json`,
  `docs/validation/mixed-nash.md`.
- Check: `cargo test --locked -p paignia-solve --test mixed_nash`
  (native target). Freeze independent fixtures and residual tolerances for
  dominance, matching pennies, coordination, degeneracy, multiple equilibria,
  and rectangular games. Include the upstream dominant-action counterexample.
  Account explicitly for support-enumeration and Lemke–Howson promises; an
  unimplemented algorithm stays unsupported, never silently substituted.

### [x] T17.06 — solve the declared correlated-equilibrium program (S03)

- Acceptance: qualified optional Clarabel 0.11.1, 27 method controls, eight backend probes, 13 independent controls and actual enabled-feature artifact optimum/nonconvergence calls pass on both compilers. Approximate raw-coordinate admission is explicit; Kairos S03 source disposition remains external.

- Behavior: return a distribution satisfying the actual incentive constraints
  and declared objective, with solver status and residuals.
- Depends on: T17.01; qualified solver-backend decision before dependency work.
- Execution: specialist, optimization and equilibrium constraints.
- Files (5): `crates/paignia-solve/src/lib.rs`,
  `crates/paignia-solve/src/correlated_equilibrium.rs`,
  `crates/paignia-solve/tests/correlated_equilibrium.rs`,
  `verification/numerical_oracles/correlated_equilibrium_v1.json`,
  `docs/validation/correlated-equilibrium.md`.
- Check: `cargo test --locked -p paignia-solve --test correlated_equilibrium --features correlated-equilibrium`
  (native target). Compare feasibility, incentive residuals, and objective with
  independent solutions, including a case distinguishing a correlated optimum
  from a product distribution. Evaluate existing Clarabel capability before
  custom optimization; any dependency/lock change gets a separate bounded leaf.

### [~] T17.07 — train CFR/CFR+ on a supported information-set graph (S04)

- Behavior: update a policy on general supported information sets using the
  declared CFR variant rather than a hard-coded Kuhn-game implementation.
- Depends on: T17.01 and T17.17; exact upstream API/caller inventory recorded first.
- Execution: specialist, extensive-game semantics and regret minimization.
- Files (5): `crates/paignia-solve/src/lib.rs`,
  `crates/paignia-solve/src/cfr.rs`,
  `crates/paignia-solve/tests/cfr_information_sets.rs`,
  `fixtures/conformance/cfr-reference-v1.json`,
  `docs/validation/cfr.md`.
- Check: `cargo test --locked -p paignia-solve --test cfr_information_sets`.
  Validate legal actions, chance weighting, imperfect-recall rejection, and
  Kuhn/Leduc value and exploitability against pinned independent references
  across iteration counts. Keep game-specific helpers separate.
- Implementation evidence — 20 November 2026: vanilla CFR and CFR+ train on the
  graph admitted by T17.17. Regret matching, chance weighting, reach-weighted
  averaging, pure best-response Nash conv, imperfect-recall rejection and
  declared action order are covered by six focused tests. Kuhn's first-player
  value `-1/18` is pinned in `fixtures/conformance/cfr-reference-v1.json` and
  evaluated by a separate deal enumerator. At 12,000 iterations both variants
  stay within `5e-3` of that constant, with independent exploitability below
  `0.02`, and the library Nash conv matches that enumerator within `1e-8`.
  Leduc uses the two-player OpenSpiel rules: 936 information states, an
  independent 120-deal check-call value, and CFR+ after 4,000 iterations within
  `0.02` of the published first-player value `-0.085606`, with Nash conv below
  `0.05`. Vanilla CFR and CFR+ both have a larger Leduc Nash conv at 1,000
  iterations than at 4,000. The published value is checked only for CFR+.
  On Kuhn, both variants are also checked at 100, 1,000, and 12,000
  iterations: exploitability shrinks at each step, and the independent value
  stays within `5e-3` of `-1/18`. The value error is not monotone, and there is
  still no published Leduc iteration curve. This does not close S04, T17.07, or
  T17.17.

- Review and integration: PRs #39, #40 and #51–#53 implement and check the
  bounded provider. Independent references check both players' best responses
  and NashConv at the recorded Kuhn and Leduc budgets. The hosted Rust profiles
  pass. UOGTO profile acceptance and the Kairos S04 source disposition remain
  pending. [20 November review](../../../docs/validation/provider-review-20261120.md)

## Phase C — optional execution and representation gaps

### [~] T17.08 — execute a supported game trace through the Rust Kairos adapter

- Behavior: translate the supported game execution plan into actual native
  Kairos events for one selected policy/chance path.
- Depends on: T17.01–T17.03, T17.14 and accepted `K01.port`/Kairos provider
  contracts under the deferred scientific-consumer track; pin qualification
  remains separate. If the adapter uses T17.13's mapping, its static converter
  must already be qualified. Native handle/lifetime checks stay in this .08
  leaf; .13 static completion does not wait for them. This is not a static
  package prerequisite.
- Execution: specialist, public execution contract.
- Files (5): `Cargo.toml` and the separate `crates/paignia-kairos` workspace's
  `Cargo.toml`, `Cargo.lock`, `src/lib.rs` and `tests/native_replay.rs`.
- Check: `cargo test --locked --manifest-path crates/paignia-kairos/Cargo.toml
  --test native_replay`. A tiny two-action/one-chance case dispatches only the
  chosen causal path. `Solution.trace` includes postorder hypothetical branches:
  preserving it for diagnostics must never schedule those alternatives or treat
  expected payoffs as sampled outcomes. Repeated execution with declared inputs
  gives the same native event order. Use the frozen observation/identity/time
  contract; reject unsupported simultaneous, stale, duplicate, cancellation or
  delayed-effect operations before mutation until separately implemented. Bound
  zero-time loops explicitly. This leaf does not qualify all lifecycle features,
  stochastic stream isolation or checkpoint resume. The adapter depends on game
  core and Kairos; static solving stays independent and still passes T17.02.
- Candidate evidence: the adapter binds a live entity generation and empty
  scheduler at Kairos `bac331fc`. It schedules one native event per chosen
  action or chance outcome and records the dispatched ID, tick, sequence and
  revision. The separate Rust 1.99 and stable CI matrix exercises the bounded
  adapter. `K01.port` provider acceptance remains with Kairos Track 114.
  [Execution contract](../../../docs/architecture/game-native-execution-v1.md)

### [x] T17.13 — reject semantically lossy game-profile conversions

- Acceptance: 29 author tests, six independent policy-invariance/identity controls, actual artifact round trip and both compiler profiles pass. Static .13 is complete; native handle/lifetime acceptance remains .08.

- Current bounded claim: convert explicit neutral identity records under the
  accepted static profile. Preserve role-specific identity bijections, full
  source/target strings, graph and edge order, numeric bits and sparse-zero
  semantics. Reject unknown/duplicate/colliding mappings and action renaming
  that changes lexical selection order. No native handles or ontology parser.
- Behavior: convert the supported versioned game representation without silently
  losing identity or changing the declared game semantics.
- Depends on: T17.01 for static semantics/identity; the source map supplies the
  exact legacy surface. Completion of .13 covers the declared static profile.
  Actual native-handle/lifetime/causal binding belongs to T17.08, not a reciprocal
  prerequisite for .13. Static semantic acceptance does not wait for it.
- Execution: specialist, public conversion and compatibility contract.
- Files (4): `crates/paignia-core/src/lib.rs`,
  `crates/paignia-core/src/conversion.rs`,
  `crates/paignia-core/tests/semantic_conversion.rs`,
  `docs/architecture/game-semantic-conversion-v1.md`.
- Check: `cargo test --locked -p paignia-core --test semantic_conversion`
  (native target). Preserve specification/instance/session/trace namespaces,
  full UOGTO and legacy Kairos IRIs, and versioned neutral identity records;
  reject collisions, unknown identities and unsupported mappings. Core accepts
  explicit DTOs, never Kairos types or runtime references. Round-trip the
  supported static profile and reject unmapped
  chance-zero, payoff-completeness or tie-selection semantics. Never silently
  discard information sets. A deliberate semantic change requires a new profile
  and consumer decision. Only consumers using this conversion require its receipt.
  The T17.08 adapter reuses this converter and separately tests real generational
  handles, stale native lifetimes and causal dispatch against Kairos; those checks
  must not be represented as completed by a static DTO round trip.

### [x] T17.14 — expose only the declared player observation to a live policy

- Acceptance: 32 author tests, six independent projection/isolation controls, the no-world-field compile-fail test and actual artifact action-order/binding checks pass on both compilers. Caller-declared consistency grants no native authority or freshness.

- Behavior: construct a policy input containing only the player's permitted
  information and legal actions for the selected executable profile.
- Depends on: T17.01; T17.17 additionally applies when information sets are claimed.
- Execution: specialist, game information and public policy contract.
- Files (4): `crates/paignia-core/src/lib.rs`,
  `crates/paignia-core/src/observation.rs`,
  `crates/paignia-core/tests/policy_observation.rs`,
  `docs/architecture/policy-observation-v1.md`.
- Check: `cargo test --locked -p paignia-core --test policy_observation`
  (native target). Two hidden states with the same lawful observation yield the
  same policy input/legal-action set; opponent-private values and full-world
  references cannot leak through it. Bind session, player/information-set and
  observation revision, rejecting unsupported information structures explicitly.
  Offline solvers may inspect the mathematical game without granting that access
  to an acting policy. A static solver with no live policy API need not wait here.

### [ ] T17.15 — isolate solver exploration from simulation random streams

- Behavior: keep exogenous simulation draws invariant when solver exploration
  or its iteration budget changes under the declared coupling policy.
- Depends on: T17.08 and T17.13 for stable runtime identity mapping.
- Execution: specialist, RNG and comparison semantics.
- Files (4): `crates/paignia-kairos/src/lib.rs`,
  `crates/paignia-kairos/src/randomness.rs` (future),
  `crates/paignia-kairos/tests/rng_isolation.rs` (future),
  `docs/validation/game-rng-isolation.md` (future).
- Check: `cargo test --locked -p paignia-kairos --test rng_isolation`
  (future target). Use native purpose/identity streams and record algorithm,
  stream/distribution versions and the intended strategy/draw coupling. Changed
  solver budgets, reordered construction and policy branches preserve declared
  exogenous draws; intentionally independent streams remain distinct. The
  RNG-isolation regression must consume both exploration and exogenous streams
  using an explicit exploratory or stochastic fixture, so it distinguishes
  stream isolation from a fixture that draws nothing. Static
  expectation-based solving draws nothing. A future stochastic static solver
  receives an explicit RNG/configuration without a Kairos dependency; it must
  not borrow a simulation-global stream. A shared seed alone is not this oracle.

### [ ] T17.16 — restore qualified policy and native state for causal replay

- Behavior: resume the supported game execution from one checkpoint with the
  same subsequent causal events and outcomes as uninterrupted execution.
- Depends on: T17.08 and T17.15.
- Execution: specialist, state ownership and replay.
- Files (4): `crates/paignia-kairos/src/lib.rs`,
  `crates/paignia-kairos/src/checkpoint.rs` (future),
  `crates/paignia-kairos/tests/policy_resume.rs` (future),
  `docs/validation/game-policy-resume.md` (future).
- Check: `cargo test --locked -p paignia-kairos --test policy_resume`
  (future target). Restore live policy state, identity mapping, native state and
  qualified RNG checkpoints together; a sibling branch remains isolated. Keep
  frozen offline training provenance distinct from live replay state. Reject
  incompatible versions or unsupported mutable policy state before resuming;
  matching a solver's diagnostic trace alone does not establish replay.

### [~] T17.17 — validate an ECS-independent information-set representation

- Behavior: validate the declared finite information-set graph without importing
  Kairos storage or treating ontology labels as implemented semantics.
- Depends on: T17.01 and the UOGTO executable-profile contract.
- Execution: specialist, extensive-game representation.
- Files (4): `crates/paignia-core/src/lib.rs`,
  `crates/paignia-core/src/information_sets.rs`,
  `crates/paignia-core/tests/information_sets.rs`,
  `docs/architecture/game-information-contract-v1.md`.
- Check: `cargo test --locked -p paignia-core --test information_sets`
  (native target). Check player/namespace ownership, consistent legal actions
  within each information set and the declared recall/observation rules. Reject
  unsupported imperfect recall and invalid memberships. The base `Game` retains
  its bounded perfect-information profile; this contract adds a separate declared
  partition and does not change `Game` semantics. CFR acceptance follows in T17.07.
- Implementation evidence — 20 November 2026: complete-partition admission,
  exact ordered-action agreement, declared-player ownership and a perfect-recall
  check over shared DAG paths are implemented in the four paths above. The
  tests-first commit `535b1ca` executes 33 tests with zero passes; the retained
  36-test suite and the 266 default / 301 all-feature workspace tests pass on
  Rust 1.75.0 and stable 1.100.0, with formatting, Clippy and rustdoc warnings
  denied, extraction validation, six retained independent review harnesses and
  the extracted-source package smoke passing locally. Artifact qualification is
  unavailable on the macOS host because `bubblewrap` is absent. Independent
  specialist review and hosted Linux CI remain required before this leaf moves
  to `[x]`; no CFR/S04, native-lifecycle or ontology claim is made here.

- Review and integration: PR #37 merged the perfect-recall graph after the
  specialist review and both hosted Rust profiles passed. The review tightened
  the space bound for decisions shared by many paths. The UOGTO executable
  information-set profile remains pending.
  [20 November review](../../../docs/validation/provider-review-20261120.md)

## Phase D — real application consumers

### [ ] T17.09 — external reimbursement consumer qualification

- Owner: [new-drug-reimbursement-game T17](https://github.com/edithatogo/new-drug-reimbursement-game/tree/main/conductor/tracks/t17_game_runtime_ecosystem_20261107).
  This entry is an ownership/dependency link, not a second local implementation.
- Behavior: the application constructs its own real game and qualifies immutable
  Paignia artifacts against an independent expected result. Only capabilities
  used by the application add numerical or native prerequisites.
- Depends on: T17.03. Static acceptance does not require the optional adapter.
- Files: none in this provider leaf; application owner resolves its bounded file
  list. Its `paignia_static_game` source test is a genuine starting call, not an
  installed-artifact/real-model release receipt or a second application.
- Check: record the actual consumer Cargo target, nonzero tests, exact Git/package
  revision and artifact/profile hashes in that repository. Reimbursement-local
  valuation stays there. Paignia T17.12 reads the resulting receipt.

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
- Static reimbursement plus Innovate is an eligible pair. CareOps can supply a
  different real case; an optional Kairos adapter is not a blanket prerequisite.

### [ ] T17.11 — external reimbursement valued-payoff port

- Owner: [new-drug-reimbursement-game T17](https://github.com/edithatogo/new-drug-reimbursement-game/tree/main/conductor/tracks/t17_game_runtime_ecosystem_20261107).
- Behavior: application inputs preserve declared units, perspective and valuation
  provenance and reject incompatible values before calling Paignia.
- Depends on: T17.01; HEOR valuation and Voiage VOI remain separate owners.
- Files/checks: the application owns its bounded implementation and
  `payoff_input_contract` test. No valuation algorithm or second completion
  authority is introduced in Paignia. This leaf does not gate another static
  consumer that has its own qualified payoff contract.

## Phase E — bounded evidence and release eligibility

### [ ] T17.12 — require two real consumers for bounded release eligibility

#### [x] T17.12.guard — validate received evidence without inventing acceptance

- Implement a deterministic fail-closed validator for actual package receipts,
  two distinct external application receipts with matching provider artifacts
  and semantic profile, and explicit S01–S04 dispositions. Receipt files are
  content-hashed; publication and full-track completion are separate decisions.
- Scope: `scripts/validate_game_consumer_evidence.py`, its focused test file,
  `docs/validation/game-consumers.json`, and this plan. Current real application
  receipts are absent; the checked-in report must remain ineligible.
- Tests distinguish two valid synthetic receipt records from duplicate, toy,
  mismatched, missing, corrupted, failed or unsupported-method evidence. These
  test records exercise admission only and never count as actual consumers.
- Tests-first `76b3168` executes 12 tests with 20 errors from the explicit
  unimplemented validator. The initial implementation executes 13 passing tests,
  including malformed-data rejection. The committed real manifest correctly
  reports ineligible (exit 1): immutable artifacts/profile and both application
  receipts are absent. This does not complete parent T17.12 or authenticate
  external claims; owner review of the content-bound receipts remains required.
  Independent review subsequently found identity canonicalization, profile
  binding, scientific-method evidence and feature-coverage gaps. Corrected
  admission now passes 30 controls; the real manifest remains ineligible.
- Current evidence: `docs/validation/receipts/t17-bd880dd-handoff.json` binds a
  clean immutable provider, exact profile and both qualified archive bytes,
  independently matched to the downloaded hosted artifact. The current guard
  runs and returns exit 1 solely for the missing two real application receipts.
  Its selected release capability is backward induction through the graph
  profile; separate bimatrix/ODE claims retain their method/input requirements.

- Behavior: report bounded release as ineligible until two distinct real consumer
  receipts identify the same qualified static artifacts and passing profile.
- Depends on: T17.03, T17.09 and T17.10. Numerical/native receipts apply only to
  the supported capabilities claimed by the artifacts and consumer cases.
- Execution: local evidence validation; reuse the numerical/public-contract
  reviews already recorded instead of requesting repeated human approval.
- Files (4): `scripts/validate_game_consumer_evidence.py` (future),
  `tests/test_game_consumer_evidence.py` (future),
  `docs/validation/game-consumers.json` (future), `extraction-manifest.json`.
- Checks: `python -m unittest discover -s tests -p
  'test_game_consumer_evidence.py'` (future suite), `python
  scripts/validate_game_consumer_evidence.py` (future guard), and
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
