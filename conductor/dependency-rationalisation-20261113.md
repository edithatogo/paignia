# Rust dependency rationalisation — paignia (2026-11-13)

**Status:** scoped Conductor adoption contract; code substitutions not yet implemented or certified by this file.

## Repository-specific decisions

- Reconcile active T17 game-runtime and T18 illicit-enforcement tracks; retain narrow game-library scope.
- **Verified:** `paignia-core` and `paignia-solve` have minimal dependencies; `ValidationError` and `SolveError` manually implement Display/Error using debug formatting.
- **Adopt if proven:** `thiserror` for typed errors following golden tests of variant identity, `code()` values and diagnostics; optional Serde only through explicit versioned interchange DTOs.
- **Extend:** Proptest for graph permutations, probability mass, finite payoffs, cycle/reachability rejection, deterministic tie policies, solver trace validity and resource budgets.
- **Benchmark:** Criterion for finite-game DAGs and bounded normal-form games. Rayon only for independently evaluated profiles after determinism and quality acceptance.
- **Retain:** custom backward induction, bounded recursion, ordered maps and domain-specific validators; no unwarranted Tokio, Reqwest, SQLx, Syn/Quote, Ratatui or PyO3.

## Delivery and acceptance contract
- Reconcile existing Conductor tracks and ownership before editing code; do not duplicate completed tasks.
- Write a failing behavioural/regression test first for each runtime change; preserve APIs, schema versions, stdout/stderr, exit codes, error codes and MSRV or explicitly version breaking changes.
- Run the existing repo gates: targeted tests, full workspace tests, fmt, clippy and contract/interop checks. Record actual passing CI SHA; this document is planning, not validation.
- Inventory duplicate crate versions/features with `cargo tree -d` and `cargo tree -e features`; review unused dependencies, `cargo audit`, `cargo deny`, licence, MSRV, SBOM and release implications where applicable.
- Benchmark performance proposals with fixed fixtures and baseline/candidate receipts, including correctness, memory, CPU and determinism.
- Decision for each candidate must be recorded as adopt, retain, reject or defer with evidence, risks and rollback.
- Do not expose identifiable, secret, copyrighted or otherwise restricted raw data in tracing, snapshots, benchmarks or CI artifacts.
