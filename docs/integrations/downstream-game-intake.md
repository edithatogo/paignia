# Downstream games and upstream reuse

Paignia is the core implementation home for reusable game representations,
supported Rust solvers and their diagnostics. [UOGTO](https://github.com/edithatogo/UOGTO)
is the ontology authority. New games discovered in applications must improve
both projects where they expose a reusable gap. Applications retain their
institutions, empirical data, payoff construction, calibration and valuation.

## Required contribution flow

Apply this flow to a new game or a material change in players, information,
timing, strategic choices, payoff meaning, solution concept or execution model.
A new parameter set using unchanged semantics can reference an existing intake
record and add its own application example; it need not create another family.

1. **Describe the game.** Record an intake ID, originating repository and exact
   revision/path, players, actions, information, move/commitment timing, payoff
   ordering and meaning, requested result and a small synthetic example. Separate
   generic requirements from application data and assumptions.
2. **Reconcile UOGTO.** Follow its
   [semantic intake contract](https://github.com/edithatogo/UOGTO/blob/main/docs/contracts/downstream-game-intake.md).
   Record `reuse_existing`, `extension_required`, `unsupported` or
   `mapping_review_pending`, with full IRIs, the ontology revision and profile
   where applicable. Improve definitions,
   constraints, mappings, competency questions or examples when needed. An
   application name alone is not a reason to add a new ontology class.
3. **Reconcile Paignia.** Record `reuse_existing`, `extension_required`,
   `unsupported`, `domain_only` or `assessment_pending`, with the exact API,
   representation, profile and method limits where known. Check existing T17/T18
   work before proposing another solver.
   Add missing reusable Rust capability here, together with independently
   derived expected results, invalid-input checks and supported numerical bounds.
   A parameterised example of an existing family can be a reusable synthetic
   fixture; it does not require a new public API.
4. **Adopt downstream.** Link the same intake ID from the application and both
   upstream records. Pin accepted revisions or artifact hashes, run the relevant
   semantic/native/consumer checks, and record the actual result. New generic
   production implementations belong in Paignia. Existing local implementations
   remain explicitly labelled legacy or pedagogical until a tested migration
   replaces them; they must not silently stand in for unsupported Paignia APIs.

Use existing issues, PRs and Conductor leaves for implementation ownership.
An extension is open until its source, tests and limits are accepted; recording
an intake or adding an ontology term does not make a solver supported. A deferred
request needs a reason, owner and next action. Record semantic and native
dispositions separately so acceptance of one never implies the other.

The UOGTO intake table owns semantic coverage. The provider table below owns
Paignia assessment only. Shared keys are coordination identifiers, not ontology
IRIs, Rust profile IDs or another release-acceptance ledger. Current T17/T18
statuses and [consumer receipts](../validation/game-consumers.json) retain their
existing authority.

## Initial provider intake

This is a coverage-review snapshot from 22 November 2026. Existing method
contracts are linked below; unassessed application compatibility is left open.
Every new intake must pin its own origin and evidence rather than inherit these
historical snapshots.

| Shared intake key | Existing provider evidence | Downstream lead and next action |
| --- | --- | --- |
| `finite-perfect-information` | [Static graph profile](../architecture/game-runtime-contract-v1.md), backward induction and [first application receipt](../validation/receipts/t17-reimbursement-chapter8.json). | Reimbursement, Innovate and Dapane construct application games. Reuse the admitted profile where its sparse-zero, tie and information rules fit; qualify each application separately. T17.10/.12 remain open. |
| `finite-perfect-recall` | [Information-set profile](../architecture/game-information-contract-v1.md) and [CFR/CFR+](../validation/cfr.md). | Review information structures before adapting screening/signalling games. Private types, Bayesian equilibrium and arbitrary signalling games are not established by finite perfect-recall support alone. |
| `two-player-normal-form` | [Mixed Nash](../validation/mixed-nash.md), pure Nash and optional [correlated equilibrium](../validation/correlated-equilibrium.md), each within its documented bounds. | [Innovate pricing adapter](https://github.com/edithatogo/innovate/blob/a79a58d1be51a1c8d4968fd9a535d8b7e5992666/rust/crates/innovate-games/src/lib.rs) already consumes Paignia. Review [NHRA QRE](https://github.com/edithatogo/nhra_game/blob/3fe4aaa08521d103ddffbedd58eaa9d7d5256a55/src/nhra_gt/solvers_jax.py) as a distinct method request. [GTPCNZ best response](https://github.com/edithatogo/gtpcnz/blob/4cae5a2f5ebeddd8e86de4263de55caf13c6ef1a/models/primarycare_model/engines/nash_opt_adapter.py) remains pedagogical; convergence alone is not a Nash certificate. |
| `evolutionary-population` | [Bounded replicator RK4/Euler](../validation/replicator-rk4.md). | Map a consumer's fitness, simplex and time assumptions before reusing the numerical API. A general evolutionary game or native simulation receipt is not implied. |
| `leader-follower-commitment` | T18 [binary commitment](../validation/leader-follower.md) and [Nash payoff hull bounds](../validation/credal-nash.md). | Compare [NHRA's sequential model](https://github.com/edithatogo/nhra_game/blob/3fe4aaa08521d103ddffbedd58eaa9d7d5256a55/src/nhra_gt/subgames/sequential.py), GINSIM enforcement and the existing Tobac handoff before extending support. Make commitment and optimistic/pessimistic/first-index tie semantics explicit; the binary family is not a general Stackelberg solver. |
| `repeated-bargaining` | No accepted Paignia alternating-offer contract is established by this review. | [NHRA Rubinstein formula](https://github.com/edithatogo/nhra_game/blob/3fe4aaa08521d103ddffbedd58eaa9d7d5256a55/src/nhra_gt/subgames/sequential.py) motivates assessment: freeze discount/first-mover domains and disagreement assumptions, derive an independent oracle, then propose bounded Rust support. Do not transplant the perfect-patience shortcut as a proven limit. |
| `cooperative-attribution` | Provider ownership/algorithm assessment pending. | Review Voiage information portfolios and the [value-of-perspective proposal](https://github.com/edithatogo/vop_poc_nz/blob/7778459f1798e3b6631a285b292a3153ee233333/docs/method/COMPONENT_ATTRIBUTION_DESIGN.md). A future generic cooperative-game allocation routine belongs here; application characteristic functions and VOI estimands stay with their owners. Inspect source implementations before claiming that a Shapley-labelled output actually computes Shapley values. |
| `strategic-information` | [Existing Voiage handoff](voiage-handoff.md); no general information-design solver is admitted by that plan. | Reuse finite policies/outcomes where their contract fits; assess missing strategic response methods here and semantic gaps in UOGTO. Voiage retains information-value arithmetic, signed values and study comparisons. |

[GINSIM's screening, proxy-information and compliance models](https://github.com/edithatogo/ginsim/blob/26548c26352dcb571acfc44816cf702a52543fe0/src/model/extended_games.py) need explicit
information-structure and solution-concept assessment across the applicable
keys. A response formula labelled an equilibrium is not enough to choose a
solver. Optional CareOps operational games must first supply an actual game
definition. Neither a repository name nor a roadmap creates an accepted game.

[Voiage's strategic-behaviour source](https://github.com/edithatogo/voiage/blob/11a2bc8685f71c2ff351def742d439c28373cd46/voiage/methods/strategic_behavior.py)
is an originating method-assessment lead. Its fixture-backed status is not
evidence of a general equilibrium solver to transfer unchanged.

## Reuse the relevant upstream repositories

Before implementing shared infrastructure, inspect the current ecosystem
provider and reuse its qualified capability where it fits. Record the interface,
immutable revision/artifact, feature and compiler requirements, dependency
direction, focused consumer check and any gap. If it does not fit, record why
and route an improvement to that owner before creating a competing component.

| Repository/capability | Reuse boundary and source of truth |
| --- | --- |
| UOGTO ontology and semantic fixtures | Consume versioned definitions/profiles with full-IRI mappings and attributed fixtures. Its [executable information-set profile](https://github.com/edithatogo/UOGTO/blob/main/docs/contracts/executable-information-set-profile-v1.md) names a bounded projection. Paignia's typed identity conversion does not implement general RDF/JSON-LD ingestion. No ontology Python runtime becomes a prerequisite for static solving. |
| Kairos time, events, state, RNG and replay | Reuse through the existing optional `paignia-kairos` workspace. Its [manifest](../../crates/paignia-kairos/Cargo.toml) owns the exact pin; its [native contract](../architecture/game-native-execution-v1.md) owns the supported execution boundary. Preserve minimal static `paignia-core`/`paignia-solve` dependencies. |
| Voiage information value; Dapane costing/valuation | Compose via explicit optional adapters or versioned outcomes. Keep their arithmetic and valuation assumptions with those providers. Costs, transfers and funding do not automatically become game utilities. Avoid a reverse runtime dependency from Paignia core into its consumers. |
| Innovate and domain applications | Consume Paignia and contribute reusable gaps, synthetic examples and tests upstream. Do not make Paignia import an application that already depends on it. |

Research tools and evidence corpora can support discovery without becoming
numerical runtime dependencies. Existing dependency guards, feature isolation,
licence/attribution requirements and compiler/consumer qualification still apply.
This workflow adds no new runtime dependency, pin update or publication gate.
