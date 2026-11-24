# Bounded generated-game assurance

This maintenance leaf adds executable tests for the accepted static mixed-Nash
provider. It does not change T17 status, solver ownership, independent numerical
fixtures, application receipts, optional methods, or release qualification.

`crates/paignia-solve/tests/generated_game_invariants.rs` constructs synthetic
finite 2-by-2 games, then invokes the production Rust APIs:

- 576 strict-dominance games vary signed action advantages, common payoffs and
  opponent externalities. The unique actions and utilities have closed forms.
  Each game also checks that a half/half false witness exposes the expected
  profitable deviations for both players.
- 1,944 games derive from an asymmetric zero-sum family with an analytic interior
  equilibrium. Indifference determines both players' mixing probabilities.
  Independent positive payoff scales, additive offsets, and independent
  row and column reversals preserve the corresponding strategies and transformed
  utilities. The independent review confirmed that the mix distinguishes
  row reversal in all 1,944 cases and column reversal in 1,896 cases. The
  other column mixes are uniform.
- 24 mutations place NaN or either infinity in every payoff position for each
  player and must produce the typed non-finite input error.

The generators enumerate their bounded domains, use no RNG or new dependency,
and assert the executed case totals. Failures reproduce with the test name and
matrices in the native test output; there is no claimed coverage-guided fuzzing
or sanitizer execution. The 1e-10 assertion tolerance is test-only; production
solver tolerances are unchanged. Degenerate games and unresolved-support claims
remain covered by the existing independent and native controls.

The ordinary Native CI already runs this integration target on stable and the
Rust 1.75.0 MSRV through `--all-targets`. The separate path-scoped PR/weekly/manual campaign uses
that existing MSRV, a 480-second build budget, a 60-second execution budget and a
10-minute job limit. Compiler, commit and test logs are retained for 14 days.

```sh
cargo test --locked -p paignia-solve --test generated_game_invariants -- --test-threads=1 --nocapture
```
