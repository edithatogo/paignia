# Paignia immediate transfer and retention map

Paignia now owns the generic Rust game implementation in the user-selected
`edithatogo/paignia` repository. The existing seed packages have moved from the
reimbursement incubator with unchanged algorithms. The application retains its
model and finite integration test; its immutable Paignia dependency switch is
prepared in the application-owned change.
Registry publication, advanced numerical methods and consumer release acceptance
remain pending. `extraction-manifest.json` records source provenance.

## Source snapshots

| Repository | Inspected revision | Role |
|---|---|---|
| `edithatogo/new-drug-reimbursement-game` | `f6c1c87b6b00875b33c1e19401c1de09481913ce` | Main after PR #102; source of the two Rust packages. |
| Same repository, PR #101 | `c7b2539b60ecb6457c75164c2a759730f5c1ec7c` | Naming supplement reconciled with main; PN aliases now point to the existing 17 T17 leaves. |
| `edithatogo/kairos` | `e76b97be2ea3712645933d903822ff6381c8883e` | Source/API audit only; no runtime code copied by this change. |
| `edithatogo/UOGTO` | `a5f90557c40f054255a1d6c2541fd76f07aa5090` | Semantic/reference audit only; no ontology or Python code copied. |

These audit references do not update application provider pins or qualify
newer upstream versions. Historical receipts retain their original package names.

The prepared source commit is
`9ee388502bfe5f71d22b363dbb9c74f032a07ad4` in the incubator. Paignia starts from
README bootstrap `0e28f1bf33ed8c7be5fa1d63b8cc40517b61d22c`. The code transfer
preserves the source algorithms; only package metadata/notices and repository
ownership change here.

## Immediate source disposition

| Source paths / assets | Immediate disposition | Receiving responsibility and open work |
|---|---|---|
| Incubator `crates/uogto-game-core/{Cargo.toml,src/lib.rs}` | Transfer to Paignia `crates/paignia-core`; preserve finite graph validation and inline tests. | Paignia representation owner. Result/identity contract, information sets and independent artifact qualification remain T17.01/.03/.17. |
| Incubator `crates/uogto-game-solve/{Cargo.toml,src/lib.rs}` | Transfer to Paignia `crates/paignia-solve`; update import and versioned core dependency without changing algorithms. | One native backward-induction/pure-Nash implementation. T17.03 packaging and numerical additions remain open. |
| Incubator Chapter 7/8 economics and `fixtures/conformance/chapter*-*.csv` | Retain in reimbursement application. | Application owns economic assumptions, valuation, offer grid and expected outcomes; no domain data enters generic core. |
| Kairos `game-theory/src/normal_form.rs` representation/best-response/dominance/pure-Nash code | Retain at source pending selective transfer; do not copy a second pure-Nash solver. | Reconcile with Paignia types and one implementation under T17.01; preserve source/caller history. |
| Kairos `ClarabelConvexSolver` and `tests/clarabel_solver_oracles.rs` | Candidate optional solver backend, not transferred now. | T17.06 owns method-specific feasibility/incentive/residual qualification and a bounded dependency decision. |
| Kairos legacy RK4, mixed-Nash/Lemke–Howson, CE and Kuhn-CFR APIs | Preserve source failures and transfer obligations; no qualified Paignia API created by renaming. | S01–S04 remain open in T17.04–T17.07 and Kairos Track 110's source regression/retirement record. |
| Kairos `game-theory/src/extensive_form.rs` and `graph_relations.rs` | Retain until runtime-independent game representation and optional adapter mapping are implemented. | T17.08/.13–.17 separate `ComponentStore`/generational handles from game IDs, permitted observations and causal execution. |
| Kairos `game-ontology/src/lib.rs` and ingestion/codegen tests | Retain restricted fixture tooling; future Paignia import/build responsibility. | Declare supported RDF subset, use full-IRI mappings, fix generated-file packaging before promising an adapter. |
| Kairos `open-game-theory-ontology` schemas/generated fixtures | Retain as legacy fixtures; do not relabel as canonical UOGTO. | Resolve missing provenance and namespace mapping before adoption. |
| UOGTO `ontologies/**`, `shapes/**`, JSON-LD conventions, competency questions and semantic examples | Retain canonical ownership in UOGTO. | SGC.01–SGC.08 own executable semantic profile and scoped semantic conformance; Paignia consumes a pinned subset. |
| UOGTO `scripts/validate.py`, runner/payoff-scope tests and interoperability fixture inventory | Retain reference validation; reuse pinned, attributed fixtures when implementing Rust mapping. | UOGTO validates semantics; Paignia validates Rust conversion/unsupported input. An illustrative runtime binding is not proof of actual execution. |
| UOGTO `uogto/runner/engine.py`, playground and `llm_player.py` | Retain or relabel reference/demo tools; no Python production core moves into Paignia. | Runner inspects RDF and looks up payoffs. Current `LLMPlayer` selects the first action and invokes no LLM. |

Kairos abbreviated source paths above are under
`crates/kairo-ecs-game-theory` / `crates/kairo-ecs-game-ontology` respectively.

## Semantics, packaging and performance gaps

Kairos legacy fixtures use `https://kairos.dev/ontology/game-theory#`; UOGTO
owns `https://w3id.org/uogto/core#` and `https://w3id.org/uogto/extensions#`.
No namespace changes in this seed. Local-name matching must not merge distinct
IRIs. The legacy fixture manifests omit author/date/licence/review-SHA fields
required by their own provenance policy; record or resolve them before reuse.

The Kairos generated-components feature includes a file outside its crate root.
Any future transfer must package the generated source or explicitly exclude
that feature. The current fixture parser is not a general RDF/JSON-LD parser.
UOGTO keeps declarative semantics; Paignia owns Rust execution of the supported
subset rather than duplicating ontology authority.

Keep the seed's known behavior visible: extensive-form missing payoffs are
currently treated as zero, zero-probability edges are allowed, ties use an
explicit tolerance and lexical rule, and information sets are absent. Its
`Solution.trace` is solver analysis over hypothetical alternatives, not a
simulation event stream. T17.01/.13 reconcile these rules before conversion.

Kairos `children_of` scans its component store for each parent. The optional
adapter should construct a validated adjacency snapshot once, preserve an
explicit ECS/game identity map, and invalidate that snapshot on topology
changes. Static solving then avoids repeated ECS scans. Benchmark the existing
BTreeMap/string baseline against compact storage before claiming speedups;
the source inspection alone supplies no measured performance result.

## Licences and notices

| Material | Inspected licence | Handling |
|---|---|---|
| Existing Paignia seed from this repository | Apache-2.0 | Original contributor notices remain in root and crate-local `LICENSE`/`NOTICE` files so future package artifacts can retain them; source revision/path are recorded in the extraction manifest. |
| Kairos source candidates | MIT OR Apache-2.0 | Preserve source notices and chosen licence when actually transferred; no Kairos source is copied in this change. |
| UOGTO code/reference tooling | MIT | Preserve UOGTO contributor attribution on a future code transfer. |
| UOGTO ontology and documentation | CC-BY-4.0 | Keep canonical resources in UOGTO; any redistributed fixture needs its own attribution/licence/provenance. Do not silently apply the Rust licence to it. |

No book text, scans or third-party solver implementation is transferred.
Independent reference jobs are separate from native runtime dependencies.

## Acceptance and rollback

The reimbursement-owned application integration target is `paignia_static_game`. It constructs
and solves one synthetic finite reimbursement offer case through public Paignia
APIs, using existing application valuation. It is an executable starting point,
not a receipt for released artifacts or empirical economic validity. Hosted
Rust CI must run it with a nonzero test count. T17.03 still needs packaged
artifacts outside the provider checkout; T17.09/.10 need two real consumer
receipts for the same profile. The optional Kairos adapter and advanced methods
do not gate bounded static qualification when not claimed.

Rollback is a normal revert of the provider transfer or a consumer pin rollback. The source
baseline is the main revision above; immutable application dependency changes are verified by that consumer, and
no compatibility implementation is forked. The provider native CI also runs
two isolated extracted-source smoke tests; it does not replace real-consumer
release qualification.
