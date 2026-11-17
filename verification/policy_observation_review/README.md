# Independent policy-observation controls

These six native controls independently accepted source
`bac35fcc989b01e3e51c166b72c84cc4240ab951`, integrated as `eac79da` and published
as `eee813b`. All 32 author tests and all six external controls executed on Rust
1.75. The external harness passed formatting and pedantic Clippy with warnings
denied. No production correction was necessary.

`tests/review.rs` is the byte-identical accepted harness, SHA-256:
`a16b252db0a14b6b2e7940c7bad138fb24f2996f093cfe818a907ecb6b4ba87a`.

```sh
cargo test --locked --manifest-path verification/policy_observation_review/Cargo.toml --test review
```

Require six executed tests. Controls replace the entire unexposed graph and root
while preserving the DTO, use identical strings across typed roles, distinguish
canonically equivalent Unicode identifiers, check mismatch precedence, reject
invalid zero-probability branches, and test revision consistency and owned output
isolation.

Acceptance covers the declared static perfect-information projection. Matching
caller-supplied context does not establish actual native currentness, freshness,
authentication, hidden-information lawfulness or replay. Root integration reruns
these unchanged controls against the combined source.
