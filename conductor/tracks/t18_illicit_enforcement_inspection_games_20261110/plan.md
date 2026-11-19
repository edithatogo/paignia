# T18 Implementation Plan — Illicit-supply border-enforcement and retail-inspection game contracts for tobac

Status: in progress (`[~]`). All tasks preserve `T17` provider ownership and the neutral extraction vocabulary guard.

## Phase 1 — Neutral leader-follower inspection primitives (`paignia`)

- [~] **T18.01**: Specify and test a domain-neutral leader-follower (Stackelberg) and mixed-strategy inspection game fixture in `paignia-core` / `paignia-solve`, verifying pure/mixed best-response and commitment-value invariants against an independent analytical 2x2 inspection oracle. Native evidence on 20 November 2026: `paignia-solve` evaluates the declared positive-parameter family in exact rationals. The unique Nash witnesses for `c < h` and `c > h`, zero deviation gains, the false commitment profile `(1/3, 0)`, and all fourteen budget/tie certificates match the synthetic literals. A sixteenth grid misses the optimistic value `-1/3`. Equal cost and harm is rejected instead of clipping `c/h`. This does not close T18.01: specialist review, T18.02 interval bounds, and the external T18.03 consumer receipt remain open. The synthetic Python oracle stays a separate test-only reference.
- [ ] **T18.02**: Add interval/credal payoff-perturbation bounds for two-player inspection games so parameter intervals over detection effectiveness or evasion cost yield certified lower/upper equilibrium action-share and payoff intervals with solver residual diagnostics.

## Phase 2 — `tobac` and `voiage` consumer handoff (external consumer-owned)

- [ ] **T18.03** *(consumer-owned in `edithatogo/tobac`)*: Bind `tobac` border-seizure and retail-inspection evasion scenarios (`M2a`, `src/tobac/partial_identification.py`) to `paignia`'s neutral inspection game contract, preserving `dapane` enforcement resource cost provenance and `voiage` paired-draw / credal dominance compatibility.
