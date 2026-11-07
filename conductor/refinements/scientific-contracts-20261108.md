# Paignia Scientific Contract Refinement Plan

> For agentic workers: amend the owning T17 leaf after reconciling the live source-transfer branch. Use a failing assertion, minimal change, passing focused check, review and evidence receipt. This is not a second completion ledger.

**Goal:** Make supported game solutions, lawful policy observations and optional decision integrations scientifically explicit and independently releasable.

**Architecture:** Rust core and qualified solver packages are independent of simulation and valuation. Optional adapters consume Kairos, UOGTO or versioned outcome contracts. Applications construct their games and welfare assumptions; Voiage evaluates information value.

**Tech Stack:** The exact transferred Rust packages and qualified dependency locks; optional Polars/Arrow interchange; isolated independent numerical oracles.

**Spec:** [Existing integration handoff](../../docs/integrations/voiage-handoff.md); transferred `t17_game_runtime_ecosystem_20261107/spec.md` and `plan.md`; repository-standards PR120 `docs/scientific-result-contracts.md`.

## Global constraints

The maintainer's actual provider is `edithatogo/paignia`. Preserve T17.01-T17.17, incubator source/licence history and Kairos S01-S04 numerical obligations. At the inspected bootstrap main there was no solver source; paths below are receiving targets to reconcile against the transfer, not implemented APIs or passing commands. Do not take over the source transfer, reset historical statuses or rename released packages blindly.

Core contracts, named static methods and optional dynamic execution qualify independently. Static acceptance requires the actual supported method receipts and two distinct real application consumers under T17.09/.10/.12; it does not wait for all future algorithms or optional dynamics. Core has no mandatory Kairos/Voiage/UOGTO tooling/dataframe runtime. No Python scientific fallback. Newest dependency candidates need all five qualification checks and a reproducible release lock.

One source of task status stays in T17. These amendments are not new T17 IDs. Keep unimplemented methods explicitly unsupported; do not delete their requirements to obtain a green release. Optional strategic-information research cannot block the ordinary static library.

## Review focus

Information leakage through observations or action masks; one found equilibrium labelled exhaustive; solver noise or restarts treated as economic uncertainty; hidden feature-unified runtime dependencies; and package renaming treated as numerical proof. The following existing-leaf amendments pin each failure.

## T17.01 / .05-.07 — solution guarantees and behavioural interpretation

Files: transferred `crates/paignia-core/src/lib.rs` and its result/contract module, `crates/paignia-core/tests/runtime_contract.rs`; method-specific `paignia-solve` tests already owned by T17.05-.07. Resolve exact type/module names once during transfer and use the mapping thereafter.

Required result fields: game class, information structure, solution concept, algorithm/version, status, tolerance, residuals or exploitability where defined, equilibrium-selection rule, and catalogue completeness. Distinguish `one_found`, `enumerated_within_declared_scope` and `approximate`; do not demand impossible exhaustive claims from an approximate method. Noisy-choice, inertia and adaptive-learning models are behavioural predictions, not renamed Nash solutions.

- [ ] Add a `runtime_contract` test that rejects a claimed exhaustive catalogue lacking completeness evidence; coordination-game multiplicity survives serialisation and one selected solution cannot become all equilibria. Unknown solution concepts and non-finite residuals fail explicitly.
- [ ] Record the distinguishing failed assertion; implement the smallest versioned result-contract change and preserve compatible legacy data under an explicit migration.
- [ ] Run `cargo test --locked -p paignia-core --test runtime_contract` after the transferred target exists, plus the affected method oracle. A zero-test filter is not a pass.
- [ ] Verify each claimed algorithm against the independent fixture and declared supported class; keep unclaimed mixed/CE/CFR methods pending. Commit evidence and the exact API mapping.

Stop if algorithm scope or completeness cannot be established. Do not replace residuals with a successful process exit, or label solver restart frequencies as player behaviour.

## T17.14 / .17 / .08 — lawful observations without hidden-state access

Files: transferred `crates/paignia-core/src/observation.rs`, `crates/paignia-core/tests/policy_observation.rs`, and the optional adapter's existing observation binding. General information-set graph validation remains T17.17; no dynamic dependency is added to it.

- [ ] Construct world states with different opponent-private costs but the same player observation and legal actions. Assert identical serialised policy inputs and identical chosen actions given equal policy RNG state. Include a negative adapter that leaks the cost through an action mask or debug field.
- [ ] Require session/player/information-set and observation revision validation; future events and full-world references are inaccessible to the live policy. Offline solver access to the mathematical game remains allowed and distinct.
- [ ] Observe the failed negative-control assertion, implement/reuse the lawful observation projection, then run `cargo test --locked -p paignia-core --test policy_observation` and the adapter's mapped native replay target.
- [ ] Review nonanticipation and unsupported information structures before promotion. Do not emulate an imperfect-information game by separately solving fully observed hidden-state games and averaging them.

## T17.02 / .03 / .12 — actual optionality and scoped package acceptance

Files: existing extraction/package guard and tests plus supported-feature manifest; reuse the transferred `scripts/validate_extraction.py` and `scripts/validate_game_packages.py` where present rather than introducing a second guard.

- [ ] Seed a normal/build/transitive dependency on Kairos or Voiage in a static standalone consumer and require rejection. Check minimal/default/each advertised feature profile in separate consumers; an all-features workspace build cannot establish isolation.
- [ ] Preserve optional UOGTO and Arrow adapters while keeping changing third-party types out of the stable core/FFI contracts. No blanket pin upgrade or silent root patch is permitted.
- [ ] Package/unpack into clean consumers outside the checkout and run nonzero tests of the actual declared algorithms. Record resolved graph, features, lock/toolchain, source/package hashes and supported scope.
- [ ] Retain the two-real-application requirement for scoped static qualification. An optional dynamic failure blocks a dynamic claim, not unrelated static acceptance; unsupported methods remain listed.

Stop if package contents or source/licence provenance cannot be reproduced. Do not use a Voiage schema-only wrapper as a second real application.

## T17.15 / .08 — randomness and output identities

Files: existing optional-adapter RNG test and interchange/result metadata. Use Kairos's accepted purpose/identity streams; do not add another global generator.

- [ ] Change solver exploration budget while holding a declared paired scenario fixed; exogenous simulation draws must remain invariant under that coupling policy. The negative control must actually draw from both streams.
- [ ] Distinguish analyst parameter draws, structural models, equilibrium/selection, measurement versions, solver runs and simulation replicates. These labels do not assert independence or that players observe the analyst's draws.
- [ ] Run the transferred `paignia-kairos` RNG-isolation/native-replay targets after their prerequisites exist; record raw coupled identities and actual execution, not seed equality alone.

Static stochastic solvers receive explicit RNG configuration without importing Kairos. Deterministic static expectations must not manufacture simulation randomness.

## T17.11 consumer handoff — information estimands and numerical noise

Application-owned valuation remains outside Paignia. Reuse Voiage's simulation-outcome boundary VP01/VP02/VP04 and SI01-SI04/SI-V01-SI-V08 proposal, not a second information-value implementation.

- [ ] Carry recipients, timing, public/private disclosure, feasible commitments, observation model, solution concept, equilibrium selection and welfare mapping into the consumer contract. Conventional EVPI/EVSI, net benefit of research, strategic information value and social information value are distinct estimands.
- [ ] Preserve signed strategic outcomes; unsupported/nonconverged is not economic zero. Verify a fixed-policy ordinary-VOI reduction before any supported recomputed-response case.
- [ ] Supply a finite numerical-noise fixture: one parameter state with two equally weighted simulation replicates A=[-1,1], B=[1,-1]. Conditional expected values are zero, so parameter EVPI is zero. Treating replicate rows as parameter states gives1 and must be rejected by the Voiage consumer. Paignia does not own that EVPI arithmetic.
- [ ] Preserve inner/outer replication budgets, coupling and Monte Carlo error evidence. Define a different estimand explicitly if future process outcomes are themselves observed before action.

## Research and release exit conditions

Prioritise model-discriminating observations and information acquisition/disclosure that changes strategic response. Existing information-design methods and independent implementations remain comparators; integration alone is not novelty. All scientific claims require a frozen game/information model, meaningful negative controls and independent references.

An implementing leaf records concrete input, expected output, prohibited shortcuts and a stop condition. Unresolved scientific choices stop that leaf; they are not permission for a smaller model to invent assumptions. Review/check/commit evidence belongs to the existing owner. No runtime implementation, source transfer, provider pin, task completion, registry publication or merge is delivered by this planning supplement.
