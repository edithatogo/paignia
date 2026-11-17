# Independent mixed Nash review controls

These fourteen native controls were written independently and accepted the
corrected source at `671898780dcf2d7ef7a74252fa0da23cffebd0c0`, integrated as
`be38d8d`. Rust 1.75 ran all fourteen, all 39 author controls and the twelve
unchanged Python Nash oracle controls successfully. Offline locked execution
and Clippy including pedantic warnings denied also passed.

`tests/review.rs` is the byte-identical accepted harness, SHA-256:
`5d06571759a79556e2e5a216a7debf975dc4d1c4b39075ac9579207e3ee0c041`.

```sh
cargo test --locked --manifest-path verification/mixed_nash_review/Cargo.toml --test review
```

Independent controls found exact profitable deviations of 0.5 hidden as zero
by summation, subtraction and product rounding. The original candidate failed;
the correction preserves low components without increasing tolerances and
rejects unrepresentable product components explicitly. The harness also covers
expected payoffs, a subnormal product boundary, mass error 2^-56, positive tiny
gains/supports, unequal-support-only equilibria, permuted rectangular weighted
rock-paper-scissors, duplicate-action continua, all 12,869 support pairs,
near-rank loss and unsupported Lemke-Howson requests.

Acceptance is for the bounded numerical method and its truthful finite-witness
status. It is not an exact-arithmetic certificate, exhaustive catalogue, full
Kairos caller-retirement receipt or real application acceptance. Root integration
reruns the fourteen unchanged controls against the combined source.
