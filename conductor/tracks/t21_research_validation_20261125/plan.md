# T21 implementation plan

## Phase 1 Game semantics and diagrams

- [x] Task: Compare Paignia, Gambit and OpenSpiel, and record what the current RDF/JSON profiles cover.
- [x] Task: Publish a normative profile map, qualifying representative positive, rejection and version-compatibility fixtures. [Interchange #96](https://github.com/edithatogo/paignia/issues/96)
- [x] Task: Assess Mermaid output for the public buyer–seller game and a shared-successor DAG. Compare diagram readability and stable IDs, then decide whether to add an optional exporter. [Diagram #95](https://github.com/edithatogo/paignia/issues/95)

## Phase 2 Core and per-model evaluation

- [x] Task: Record the staged [evaluation protocol](../../../docs/validation/evaluation-plan.md) and its required independent oracles, admission and benchmark measures.
- [x] Task: Freeze and replay the 0.1.0 correctness fixtures for Gambit pure and bounded mixed Nash and OpenSpiel backward induction and Kuhn CFR/CFR+. [Protocol](../../../verification/comparators/README.md) · [Two-run results](../../../docs/validation/receipts/t21-core/README.md)
- [x] Task: Extend correctness and reproducibility to the remaining admitted profiles. [Signed-source checks](../../../docs/validation/receipts/t21-core/README.md#additional-signed-source-checks)
- [~] Task: Execute the equivalent-game comparison and record benchmark data, unsupported cases, errors and independent review. [Reviewed one-host pilot](../../../docs/validation/receipts/t21-core/README.md#independent-pilot-review). Common-accuracy timing, scaling and the second architecture remain open.
- [~] Task: Freeze and run the baseline and stress tests for each downstream model using its own decision and payoff meaning. [Per-repository dispositions](https://github.com/edithatogo/paignia/issues/97)

## Phase 3 Consumer adoption

- [x] Task: Open the [parity parent and seven repository issues](https://github.com/edithatogo/paignia/issues/97).
- [ ] Task: Reconcile all model and caller inventories. Qualify and switch each compatible production path to the provider, recording current native adoption separately from remaining gaps.
- [ ] Task: Remove duplicate production code after parity and hosted checks. Record an immutable rollback pin and refer missing generic capabilities and ontology gaps to their owners.

## Phase 4 Documentation

- [x] Task: Create the Astro/Starlight documentation structure and retain discoverable release and API links. [Documentation #98](https://github.com/edithatogo/paignia/issues/98)
- [~] Task: Write and run walkthroughs for static decisions, chance, versioned interchange, bounded numerical profiles and the selected applications. Verify site links, build, navigation and accessibility.
