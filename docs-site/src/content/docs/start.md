---
title: Start with Paignia 0.1.0
description: Use the signed sources and run a finite decision game.
---

Use Rust 1.75.0 or later for Paignia's core, solve and optional interchange
packages. The signed release commit is
`a04a19da06cc50e310a172574ed66f62000b891a`.

```sh
git clone https://github.com/edithatogo/paignia.git
cd paignia
git switch --detach v0.1.0
cargo test --workspace --all-targets --locked
cargo run --locked --manifest-path crates/paignia-io/Cargo.toml --example solve \
  < verification/uogto_interchange/frozen-static-v1/request.json
```

The example chooses the lottery, then the seller chooses to take the offer
when reached. The expected payoffs are 2 for the buyer and 4 for the seller.
The response records the source identity, policy, tolerance and method.

For direct Rust embedding, use the
[locked Rust example in the README](https://github.com/edithatogo/paignia/tree/v0.1.0#example).
For independent reproduction, retain the exact release archive and commit.
[Signed release](https://github.com/edithatogo/paignia/releases/tag/v0.1.0) ·
[Version DOI](https://doi.org/10.5281/zenodo.24515285)
