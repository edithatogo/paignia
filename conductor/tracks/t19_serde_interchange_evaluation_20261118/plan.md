# Implementation and admission plan

- [ ] Reconcile T17/T18 ownership, live provider- and external-owned leaf status; specify exactly which consumer needs a wire format.
- [ ] Document a no-go if no consumer requires it; keep core serde-free and avoid `clarabel` feature changes.
- [ ] When an actual consumer is identified, define a versioned JSON/CBOR format requirements matrix, canonicalization policy only if hashes/signatures require it, typed DTO mapping and schema evolution strategy in a new optional I/O crate.
- [ ] Independently test frozen game and result fixtures, malformed inputs, precision, stable IDs, missing/unknown fields, zero/negative tolerances, nonfinite data, reproducibility and downstream consumer version compatibility.
- [ ] Qualify exact head native/installed Rust compiler gates plus independent result oracles; record the source SHA and external consumer acceptance. Do not mark T17 leaves complete solely due to I/O tests.
