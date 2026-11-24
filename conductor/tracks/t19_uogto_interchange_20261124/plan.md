# T19 implementation plan

## Phase 1 Public reference case

- [x] Task: Reconcile the closed interchange proposal and the executed UOGTO information-set profile.
- [x] Task: Pin the semantic source and exercise a public RDF game with ordered actions, chance weights, named payoffs and an independently derived result.
- [x] Task: Exercise opt-in versioned interchange and verify native graph validation, identity preservation and rejection of invalid input.
- [x] Task: Run the full path on Rust 1.75 and stable and compare the retained outputs with the hand-derived policy. Keep source and output digests in the CI artifacts.

The [reference case](../../../verification/uogto_interchange/README.md) reads
two pinned UOGTO modules, preserves whole IRIs and uses the existing native
solver. The hand oracle selects the lottery and take actions with expected
payoffs `(2, 4)`. The native and RDF negative controls are part of the same
replay.

## Phase 2 Compatibility and ontology coverage

- [x] Task: Qualify a separate downstream consumer against frozen game and result wire records before expanding the public compatibility promise.
- [x] Task: Review local mapping terms against upstream UOGTO and qualify an additional executable profile with its own oracle.

## Phase 3 Full UOGTO vocabulary interchange

- [x] Generate the 322-class and 212-property inventory from the complete pinned ontology. Qualify an offline asserted-RDF command with the upstream shapes and the existing static v1 replay.
- [x] Prove every declaration in a meaningful graph with a definition-based assertion or upstream shape and retain the term evidence in CI.
- [x] Qualify perfect-recall CFR and finite two-player Nash interchange with independent numerical oracles. Report the qualified native profiles separately from vocabulary coverage.


The [coverage replay](../../../verification/uogto_interchange/README.md#replay-term-coverage)
binds one asserted witness per class and property to its upstream definition
and a verified input/output graph. The four applied packs are in the same
denominator. Native execution qualification has its own evidence.


The native evidence includes Kuhn poker for CFR and CFR+, Prisoner's Dilemma
for pure Nash and matching pennies for mixed Nash. The installed consumer
replays the original static v1 records outside the provider checkout. Its receipt
qualifies installation and wire compatibility in a standalone consumer fixture.
