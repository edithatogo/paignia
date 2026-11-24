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

- [ ] Task: Qualify a separate downstream consumer against frozen game and result wire records before expanding the public compatibility promise.
- [ ] Task: Review local mapping terms against upstream UOGTO and qualify an additional executable profile with its own oracle.
