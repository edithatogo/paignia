# T17 implementation plan — bounded seed started, qualification pending

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
receipts. All leaves remain open; partial qualification does not close T17.
The [naming supplement](paignia-identity-and-promotion.md) records the bounded
seed rename and source-level consumer slices without adding competing leaves.

## Phase A — package boundary

### [ ] T17.01 — expose a versioned generic game/result contract

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

### [ ] T17.02 — reject Kairos in the static package dependency closure

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

### [~] T17.03 — consume the packaged crates outside this workspace

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

### [ ] T17.04 — implement the declared four-stage replicator step (S01)

- Behavior: advance supported continuous replicator dynamics by the declared
  four-stage RK4 update, retaining the legacy Euler result under its own version.
- Depends on: T17.01; exact upstream API/caller inventory recorded first.
- Execution: specialist, numerical integration.
- Files (4): `crates/paignia-solve/src/lib.rs`,
  `crates/paignia-solve/src/replicator.rs` (future),
  `crates/paignia-solve/tests/replicator_rk4.rs` (future),
  `docs/validation/replicator-rk4.md` (future).
- Check: `cargo test --locked -p paignia-solve --test replicator_rk4`
  (future target). Use analytic logistic evolution and step-refinement order;
  include time-varying fitness, discontinuity handling, positivity/conservation,
  and a negative control that fails for Euler relabelled as RK4.

### [ ] T17.05 — return a qualified mixed-Nash result (S02)

- Behavior: solve the explicitly supported game class with the named mixed-Nash
  algorithm, returning explicit unsupported/nonconverged statuses elsewhere.
- Depends on: T17.01; exact upstream API/caller inventory recorded first.
- Execution: specialist, equilibrium algorithms.
- Files (5): `crates/paignia-solve/src/lib.rs`,
  `crates/paignia-solve/src/mixed_nash.rs` (future),
  `crates/paignia-solve/tests/mixed_nash.rs` (future),
  `fixtures/conformance/mixed-nash-v1.json` (future),
  `docs/validation/mixed-nash.md` (future).
- Check: `cargo test --locked -p paignia-solve --test mixed_nash`
  (future target). Freeze independent fixtures and residual tolerances for
  dominance, matching pennies, coordination, degeneracy, multiple equilibria,
  and rectangular games. Include the upstream dominant-action counterexample.
  Account explicitly for support-enumeration and Lemke–Howson promises; an
  unimplemented algorithm stays unsupported, never silently substituted.

### [ ] T17.06 — solve the declared correlated-equilibrium program (S03)

- Behavior: return a distribution satisfying the actual incentive constraints
  and declared objective, with solver status and residuals.
- Depends on: T17.01; qualified solver-backend decision before dependency work.
- Execution: specialist, optimization and equilibrium constraints.
- Files (5): `crates/paignia-solve/src/lib.rs`,
  `crates/paignia-solve/src/correlated_equilibrium.rs` (future),
  `crates/paignia-solve/tests/correlated_equilibrium.rs` (future),
  `fixtures/conformance/correlated-equilibrium-v1.json` (future),
  `docs/validation/correlated-equilibrium.md` (future).
- Check: `cargo test --locked -p paignia-solve --test correlated_equilibrium`
  (future target). Compare feasibility, incentive residuals, and objective with
  independent solutions, including a case distinguishing a correlated optimum
  from a product distribution. Evaluate existing Clarabel capability before
  custom optimization; any dependency/lock change gets a separate bounded leaf.

### [ ] T17.07 — train CFR/CFR+ on a supported information-set graph (S04)

- Behavior: update a policy on general supported information sets using the
  declared CFR variant rather than a hard-coded Kuhn-game implementation.
- Depends on: T17.01 and T17.17; exact upstream API/caller inventory recorded first.
- Execution: specialist, extensive-game semantics and regret minimization.
- Files (5): `crates/paignia-solve/src/lib.rs`,
  `crates/paignia-solve/src/cfr.rs` (future),
  `crates/paignia-solve/tests/cfr_information_sets.rs` (future),
  `fixtures/conformance/cfr-reference-v1.json` (future),
  `docs/validation/cfr.md` (future).
- Check: `cargo test --locked -p paignia-solve --test cfr_information_sets`
  (future target). Validate legal actions, chance weighting, imperfect-recall
  rejection, and Kuhn/Leduc value and exploitability against pinned independent
  references across iteration counts. Keep game-specific helpers separate.

## Phase C — optional execution and representation gaps

### [ ] T17.08 — execute a supported game trace through the Rust Kairos adapter

- Behavior: translate the supported game execution plan into actual native
  Kairos events for one selected policy/chance path.
- Depends on: T17.01–T17.03, T17.14 and accepted `K01.port`/Kairos provider
  contracts under the deferred scientific-consumer track; pin qualification
  remains separate. If the adapter uses T17.13's mapping, its static converter
  must already be qualified. Native handle/lifetime checks stay in this .08
  leaf; .13 static completion does not wait for them. This is not a static
  package prerequisite.
- Execution: specialist, public execution contract.
- Files (5): `Cargo.toml`, `Cargo.lock`,
  `crates/paignia-kairos/Cargo.toml` (future),
  `crates/paignia-kairos/src/lib.rs` (future),
  `crates/paignia-kairos/tests/native_replay.rs` (future).
- Check: `cargo test --locked -p paignia-kairos --test native_replay`
  (future package/target). A tiny two-action/one-chance case dispatches only the
  chosen causal path. `Solution.trace` includes postorder hypothetical branches:
  preserving it for diagnostics must never schedule those alternatives or treat
  expected payoffs as sampled outcomes. Repeated execution with declared inputs
  gives the same native event order. Use the frozen observation/identity/time
  contract; reject unsupported simultaneous, stale, duplicate, cancellation or
  delayed-effect operations before mutation until separately implemented. Bound
  zero-time loops explicitly. This leaf does not qualify all lifecycle features,
  stochastic stream isolation or checkpoint resume. The adapter depends on game
  core and Kairos; static solving stays independent and still passes T17.02.

### [ ] T17.13 — reject semantically lossy game-profile conversions

- Behavior: convert the supported versioned game representation without silently
  losing identity or changing the declared game semantics.
- Depends on: T17.01 for static semantics/identity; the source map supplies the
  exact legacy surface. Completion of .13 covers the declared static profile.
  Actual native-handle/lifetime/causal binding belongs to T17.08, not a reciprocal
  prerequisite for .13. Static semantic acceptance does not wait for it.
- Execution: specialist, public conversion and compatibility contract.
- Files (4): `crates/paignia-core/src/lib.rs`,
  `crates/paignia-core/src/conversion.rs` (future),
  `crates/paignia-core/tests/semantic_conversion.rs` (future),
  `docs/architecture/game-semantic-conversion-v1.md` (future).
- Check: `cargo test --locked -p paignia-core --test semantic_conversion`
  (future target). Preserve specification/instance/session/trace namespaces,
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

### [ ] T17.14 — expose only the declared player observation to a live policy

- Behavior: construct a policy input containing only the player's permitted
  information and legal actions for the selected executable profile.
- Depends on: T17.01; T17.17 additionally applies when information sets are claimed.
- Execution: specialist, game information and public policy contract.
- Files (4): `crates/paignia-core/src/lib.rs`,
  `crates/paignia-core/src/observation.rs` (future),
  `crates/paignia-core/tests/policy_observation.rs` (future),
  `docs/architecture/policy-observation-v1.md` (future).
- Check: `cargo test --locked -p paignia-core --test policy_observation`
  (future target). Two hidden states with the same lawful observation yield the
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

### [ ] T17.17 — validate an ECS-independent information-set representation

- Behavior: validate the declared finite information-set graph without importing
  Kairos storage or treating ontology labels as implemented semantics.
- Depends on: T17.01 and the UOGTO executable-profile contract.
- Execution: specialist, extensive-game representation.
- Files (4): `crates/paignia-core/src/lib.rs`,
  `crates/paignia-core/src/information_sets.rs` (future),
  `crates/paignia-core/tests/information_sets.rs` (future),
  `docs/architecture/game-information-contract-v1.md` (future).
- Check: `cargo test --locked -p paignia-core --test information_sets`
  (future target). Check player/namespace ownership, consistent legal actions
  within each information set and the declared recall/observation rules. Reject
  unsupported imperfect recall and invalid memberships. The current `Game` has
  no information-set representation; retain its bounded perfect-information
  profile while this extension is pending. CFR acceptance follows in T17.07.

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
