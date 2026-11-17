# Independent static conversion controls

These six native controls independently accepted source
`aee7d76f915892edf043deb0a858cb81a354f47f` (published source counterpart
`ffc0ff3e0a7010ed3f934765d1a6f2adeff91f04`). Rust 1.75 ran all six external
controls and all 29 author tests. The offline locked rerun and Clippy with
pedantic warnings denied also passed. No production correction was necessary.

`tests/review.rs` is the byte-identical accepted harness, SHA-256:
`52f59db1c9328cd017892c0ae9c881e4d09adc878612f7c9ff7dc51758617884`.

```sh
cargo test --locked --manifest-path verification/conversion_review/Cargo.toml --test review
```

The controls compare hand-derived policies for all six traversal orders of an
order-sensitive near-tie game before and after admitted renaming. Other controls
cover shared graphs, node/player reordering, per-decision rather than unrelated
global action-order constraints, all nine equal-string identity roles, full
namespaces, binary64 bits, absent context, and invalid public mutation on reverse.

Acceptance is for explicit neutral static conversion under the declared
perfect-information profile. It does not certify arbitrary upstream ingestion,
ontology interpretation, live generations or causal execution. Root integration
reruns these six unchanged controls against the combined source.
