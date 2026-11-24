# Downstream games and upstream reuse

Paignia is the core implementation home for reusable game representations,
supported Rust solvers and their diagnostics. [UOGTO](https://github.com/edithatogo/UOGTO)
is the ontology authority. New games discovered in applications must improve
both projects where they expose a reusable gap. Applications retain their
institutions, empirical data, payoff construction, calibration and valuation.

## Current applications and partner libraries

The current downstream applications are
[nhra_game](https://github.com/edithatogo/nhra_game),
[gtpcnz](https://github.com/edithatogo/gtpcnz),
[new-drug-reimbursement-game](https://github.com/edithatogo/new-drug-reimbursement-game),
[ginsim](https://github.com/edithatogo/ginsim),
[tobac](https://github.com/edithatogo/tobac) and
[tutorial_health_gt](https://github.com/edithatogo/tutorial_health_gt).
Each uses the relevant upstream capabilities and follows the contribution flow
below for game-theory work. This names intended ownership; it does not claim
that all six already consume qualified Paignia artifacts.

The tutorial teaches these libraries and applications with versioned lessons,
explicit upstream revisions and reproducible synthetic examples. Adopt accepted
capabilities in successive tutorial versions while preserving older lessons'
pins and limitations. New teaching examples feed semantic/example improvements
to UOGTO and reusable game examples or methods to Paignia where appropriate.

Innovate is a parallel innovation-methods library. It consumes Paignia for
generic game capabilities while retaining diffusion, adoption, substitution and
innovation-specific behaviour. Migrate mature generic game implementations
method by method: inventory callers and limits, freeze reference results,
qualify the Paignia replacement, then switch pinned consumers with compatibility
and deprecation notes. Keep the working implementation until numerical and
behavioural parity is demonstrated; do not force a wholesale rewrite.

Voiage owns value of information, value of perspective (VoP) and value of
heterogeneity (VoH). `vop_poc_nz` is the demonstration/application for that work;
versioned adoption uses accepted Voiage interfaces and remains subject to
consumer checks. It is not a direct game-library transfer lead. A demonstrated shared game need can
still enter the appropriate intake. `archive-govt-nz` preserves public sources
and provenance alongside the Hugging Face catalogue; analytical game methods
belong in their libraries and consuming applications.

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
| `finite-perfect-information` | [Static graph profile](../architecture/game-runtime-contract-v1.md), backward induction and [first application receipt](../validation/receipts/t17-reimbursement-chapter8.json). | Reimbursement, Innovate and Dapane construct application games. Reuse the admitted profile where its sparse-zero, tie and information rules fit; qualify each application separately. T17.10 and T17.12 later passed the bounded eligibility review. [T17 archive](../../conductor/archive/t17_game_runtime_ecosystem_20261107/plan.md) |
| `finite-perfect-recall` | [Information-set profile](../architecture/game-information-contract-v1.md) and [CFR/CFR+](../validation/cfr.md). | Review information structures before adapting screening/signalling games. Private types, Bayesian equilibrium and arbitrary signalling games are not established by finite perfect-recall support alone. |
| `two-player-normal-form` | [Mixed Nash](../validation/mixed-nash.md), pure Nash and optional [correlated equilibrium](../validation/correlated-equilibrium.md), each within its documented bounds. | [Innovate pricing adapter](https://github.com/edithatogo/innovate/blob/a79a58d1be51a1c8d4968fd9a535d8b7e5992666/rust/crates/innovate-games/src/lib.rs) already consumes Paignia. Review [NHRA QRE](https://github.com/edithatogo/nhra_game/blob/3fe4aaa08521d103ddffbedd58eaa9d7d5256a55/src/nhra_gt/solvers_jax.py) as a distinct method request. [GTPCNZ best response](https://github.com/edithatogo/gtpcnz/blob/4cae5a2f5ebeddd8e86de4263de55caf13c6ef1a/models/primarycare_model/engines/nash_opt_adapter.py) remains pedagogical; convergence alone is not a Nash certificate. |
| `evolutionary-population` | [Bounded replicator RK4/Euler](../validation/replicator-rk4.md). | Map a consumer's fitness, simplex and time assumptions before reusing the numerical API. A general evolutionary game or native simulation receipt is not implied. |
| `leader-follower-commitment` | T18 [binary commitment](../validation/leader-follower.md) and [Nash payoff hull bounds](../validation/credal-nash.md). | Compare [NHRA's sequential model](https://github.com/edithatogo/nhra_game/blob/3fe4aaa08521d103ddffbedd58eaa9d7d5256a55/src/nhra_gt/subgames/sequential.py), GINSIM enforcement and the existing Tobac handoff before extending support. Make commitment and optimistic/pessimistic/first-index tie semantics explicit; the binary family is not a general Stackelberg solver. |
| `repeated-bargaining` | No accepted Paignia alternating-offer contract is established by this review. | [NHRA Rubinstein formula](https://github.com/edithatogo/nhra_game/blob/3fe4aaa08521d103ddffbedd58eaa9d7d5256a55/src/nhra_gt/subgames/sequential.py) motivates assessment: freeze discount/first-mover domains and disagreement assumptions, derive an independent oracle, then propose bounded Rust support. Do not transplant the perfect-patience shortcut as a proven limit. |
| `cooperative-attribution` | Provider ownership/algorithm assessment pending. | Establish a shared cooperative-game need before proposing a generic allocation routine here. Voiage retains information-source, VoP and VoH methods, characteristic functions and estimands; the demo does not imply a Paignia transfer. Record coalition values, baseline and an independently derived oracle for any proposed generic method. |
| `strategic-information` | [Existing Voiage handoff](voiage-handoff.md); no general information-design solver is admitted by that plan. | Reuse finite policies/outcomes where their contract fits; assess missing strategic response methods here and semantic gaps in UOGTO. Voiage retains information-value arithmetic, signed values and study comparisons. |

[GINSIM's screening, proxy-information and compliance models](https://github.com/edithatogo/ginsim/blob/26548c26352dcb571acfc44816cf702a52543fe0/src/model/extended_games.py) need explicit
information-structure and solution-concept assessment across the applicable
keys. A response formula labelled an equilibrium is not enough to choose a
solver. Historical candidate consumers outside the six applications are not
current application obligations. Neither a repository name nor a roadmap
creates an accepted game.

[Voiage's strategic-behaviour source](https://github.com/edithatogo/voiage/blob/11a2bc8685f71c2ff351def742d439c28373cd46/voiage/methods/strategic_behavior.py)
is an originating method-assessment lead. Its fixture-backed status is not
evidence of a general equilibrium solver to transfer unchanged.

### Recorded consumer case: tutorial coordination, edition 0.2

Shared intake key: `finite-perfect-information`; case identity:
`urn:tutorial-health-gt:coordination:v1`. Provider disposition: **reuse_existing**.
The [paired UOGTO note](https://github.com/edithatogo/UOGTO/blob/main/docs/contracts/downstream-game-intake.md#recorded-consumer-case-tutorial-coordination-edition-02)
records the semantic mapping and separately executed declaration checks.

The [locked tutorial example](https://github.com/edithatogo/tutorial_health_gt/tree/3851a2346c8e9cc457c7bfee67848be13fc8ca46/examples/finite-coordination)
uses `paignia-core` and `paignia-solve` at
`bd880dd756d2ba56cc5f479bc078cea31b53c1d1`, calling `backward_induction` on the
existing static finite perfect-information graph. The private consumer needs
repository access; no private source or licence is copied here.

The commissioner chooses whether to offer; the provider observes an offer and
chooses participation or decline. Invented commissioner/provider utilities are
`(1, 1)` for no offer, `(3, 2)` for participation and `(0, 0)` for decline.
Independent backward reasoning gives `offer -> participate`, utilities `(3, 2)`.
Changing only the provider's participation utility to `-1` gives `no-offer`,
utilities `(1, 1)`, with decline as the off-path provider response. Removing a
referenced terminal must reject the graph rather than return a strategy.

On 22 November 2026, Rust/Cargo 1.100.0 on Linux executed `cargo test --locked`
(one test covering all three cases) and `cargo run --locked` successfully.
Source SHA-256: `ed9af70f81f90d2cc063f389eff7b021fe40d2962cdf46d7f6a39fc69114e18b`;
Cargo lock SHA-256: `1c1810f97e87ec60ae99613d429b5101fae10f1c1af6a047d315781a88d39517`.
UOGTO's successful declaration check separately retains
`native_execution_qualified=false`; it does not certify this Rust graph.

This records a verified use of an existing method and a reusable known-answer
case. No new solver or mandatory provider dependency is needed. Admission of a
public upstream fixture, full RDF mapping and hosted tutorial execution remain
pending. T17/T18 acceptance and the authoritative consumer receipts are unchanged;
the synthetic example is not empirical policy validation.

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
| Voiage information, perspective and heterogeneity value | Reuse Voiage's qualified estimands and arithmetic when a game analysis evaluates information, perspectives or population heterogeneity. Compose through an optional adapter that consumes the relevant providers, or through versioned paired outcomes. Static Paignia and default Voiage retain their existing independent build boundaries. |
| Dapane costing/valuation | Reuse its qualified cost and valuation capabilities through optional adapters or versioned outputs. Declare units, perspective, transfers and the mapping from valuation to game utility. Costs, transfers and funding do not automatically become game utilities. |
| Innovate | Parallel innovation library with an existing Paignia adapter. Retain innovation-specific methods; contribute mature generic game capabilities incrementally after parity and consumer checks. Neither core imports the other's game adapter. |
| The six applications above | Consume the relevant libraries, retain domain models and contribute reusable gaps, synthetic examples and tests upstream. Do not make Paignia import an application that already depends on it. |

These are capability boundaries, not a requirement that every repository fit a
single total hierarchy. An optional composition package may depend on Paignia,
Voiage, Kairos or Dapane as needed; the providers do not import that composition
package. Record the resulting dependency graph and keep it acyclic.

Research tools and evidence corpora can support discovery without becoming
numerical runtime dependencies. Existing dependency guards, feature isolation,
licence/attribution requirements and compiler/consumer qualification still apply.
This workflow adds no new runtime dependency, pin update or publication gate.
