# Paignia agent instructions

Read `README.md`, `conductor/index.md`, the selected track's specification and
plan, and the applicable source before editing. Preserve original attribution
and recorded scientific limitations.

- All game execution is Rust. Development orchestration/reference tooling is
  not another supported core and must never supply a silent fallback.
- Keep domain models and valuation outside the generic packages. UOGTO owns
  canonical semantics; Kairos owns native simulation primitives; Voiage owns VOI.
- For every new or materially changed downstream game, follow
  `docs/integrations/downstream-game-intake.md`: record both UOGTO semantic
  coverage and Paignia capability disposition, including reuse of existing
  support. Add reusable representations, methods and synthetic examples here;
  retain application assumptions, data and payoff calibration downstream.
- Assess existing upstream ecosystem capabilities before implementing shared
  infrastructure. Reuse qualified interfaces with immutable pins and focused
  consumer checks; preserve the static dependency boundary and avoid cycles.
- Preserve one authoritative implementation of each supported method. A source
  move or rename does not discharge Kairos S01–S04 scientific obligations.
- Use bounded Conductor leaves, focused checks and ordinary review. Do not add
  repeated human approval requirements for authorized reversible work.
- Record missing tools or failed controls honestly; zero tests is not a pass.

Required native checks for source changes:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
target/dev-python/bin/python scripts/validate_extraction.py
target/dev-python/bin/python scripts/validate_game_packages.py
```

The declared MSRV and stable profiles are checked by hosted CI. Release
qualification and registry publication are separate from these development
checks; package publishing is currently disabled.
