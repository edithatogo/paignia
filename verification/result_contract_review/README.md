# Independent native result-contract controls

These nine tests were written independently of the result-envelope and solver
emission implementation. They were executed in a separate consumer against local
revision `9ed0caccf7409f0a65d0081afa99e2e95957c84b` with Rust 1.75.0, alongside
31 envelope and nine emitter author tests. All passed. The integration copy of
the review harness is byte-identical:

`tests/review.rs` SHA-256:
`478597109b0353f13720a1a4fbcb5200ec15c24bd551178358cd4f9ae1aa41b3`.

Run from the repository root:

```sh
cargo test --locked --manifest-path verification/result_contract_review/Cargo.toml --test review
```

The dedicated workspace consumes only public package interfaces. Controls cover
error/source precedence, contingent policy completeness, exact status/catalogue
boundaries, independently classified binary64 values, typed source errors,
diagnostic tolerance equality and overflow in an unchosen branch. A concrete
three-action example confirms the declared legacy order-dependent tie behavior.

This supports the native structural contract and computed-record emitter. It is
not a wire-format, mathematical equilibrium, causal-execution or real-application
qualification receipt. Root integration reran all nine unchanged tests after
copying the harness and generating this standalone consumer lockfile.
