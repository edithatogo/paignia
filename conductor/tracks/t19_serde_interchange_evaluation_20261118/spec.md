# Conductor: evaluate optional Serde interchange only after a consumer

## Current source boundary
- `paignia-core` depends on `thiserror`, not Serde; `paignia-solve` depends on the core, `thiserror` and optional `clarabel` with its own Serde feature. This is appropriate: the game solver is not a JSON/wire-format library.
- `crates/paignia-core/src/result.rs` explicitly states that the native `ResultEnvelope` has no serialized wire format. The versioned result metadata is validated in Rust. No evidence supports replacing numerical, solver or game representation dependencies with Serde.
- `new-drug-reimbursement-game`, Kairos and UOGTO interoperability may ultimately need a serialized, lossless specification/result envelope; require a named real consumer rather than inventing an API for hypothetical consumers.

## Design decision
**No adoption in `paignia-core` now.** Evaluate a separate optional `paignia-io` crate with Serde-derived explicit versioned wire DTOs only once T17 consumer handoffs establish a real requirement, pinned revisions and frozen fixtures. Preserve the native domain model as the single validator and stable algorithmic authority.

## Acceptance boundary
Round-trip exact identifiers and order-sensitive choices, finite probabilities/payoffs, enum identity, provenance, schema versions, errors on malformed/duplicate/unknown data as defined by the wire contract, and cross-language/C++ FFI parity where relevant. No wire-roundtrip alone can certify equilibrium or numerical correctness. No public I/O or publisher claims before approved consumer acceptance.
