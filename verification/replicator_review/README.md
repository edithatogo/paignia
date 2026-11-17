# Independent native replicator controls

These ten controls were written independently of the production RK4/Euler
implementation and executed on Rust 1.75.0 against original method revision
`e194e90612d14e99da53b8843dc2431d2a4510f4` (integrated as `3fc0283`).
The reviewer also executed the 28 production controls and 14 frozen Python
replicator oracle controls. All passed; no blocking numerical or interface
finding remained. The independent harness passed locked offline resolution
and Clippy including pedantic warnings denied.

`tests/review.rs` is byte-identical to the accepted review harness, SHA-256:
`c2087a18a423dc4eb86e71f0cdf79f72c25a0b4d878144af18fbb77ceed8c99f`.

Run from the repository root:

```sh
cargo test --locked --manifest-path verification/replicator_review/Cargo.toml --test review
```

The dedicated workspace consumes the public solver interface. Independent
expectations include cubic forcing with a rational quadrature witness,
state-dependent separable evolution, strategy permutation/common-fitness-shift
invariance, an inactive extreme-fitness strategy, failures at all four stages,
mass failure despite component bounds, representable stage times, invalid zero
steps and an asymmetric discontinuity. These controls distinguish RK4 from an
Euler update or a callback frozen at the initial state.

This accepts the bounded nonprojecting native methods. It does not supply
adaptive error control, Kairos caller retirement, real application consumption
or full S01/T17 closure. The frozen Python oracle and its tolerances remain
unchanged. Root integration reruns the unchanged ten controls.
