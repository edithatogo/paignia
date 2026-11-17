# Independent correlated-equilibrium controls

These 13 native controls independently accepted source
`026ccb8dcb95550cc1afcaddbf5114a89f6fe81a`, integrated as `df0af24` and published
as `cb30317`. All 27 author tests and all 13 external controls executed on Rust
1.75. The external harness passed pedantic Clippy with warnings denied. The
subsequent source change in `66f99a4` adds documentation backticks only.

`tests/review.rs` is the byte-identical accepted harness, SHA-256:
`7a8189e7f9faa03e842dc704670f58cd5389935fc1c57a7c0df7e361c92435ca`.

```sh
cargo test --locked --manifest-path verification/correlated_equilibrium_review/Cargo.toml --test review --features correlated-equilibrium
```

Require **13 executed tests**. Running this target with its feature disabled
executes zero tests and does not provide review evidence.

Controls include a primal-invalid/dual-valid zero-gap rejection, all 36 separate
row/column permutations of three-action dual constraints, rectangular role
reversal, player/objective scaling, multiplier signs, cancellation in products
and subtraction, both signs of subnormal arithmetic, mass rounding, overflow,
and extreme finite backend inputs.

Acceptance is approximate admission of unchanged raw coordinates under original
unit primal/dual residual bounds. It is not an exact probability distribution,
an exact equilibrium certificate, or a distance-to-polytope guarantee. Root
integration reruns the unchanged controls against the combined source.
