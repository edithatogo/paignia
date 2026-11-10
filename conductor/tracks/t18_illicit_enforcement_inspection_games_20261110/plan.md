# T18 Implementation Plan — Illicit-supply border-enforcement and retail-inspection game contracts for tobac

Status: new (`[ ]` pending). All tasks preserve `T17` provider ownership and the neutral extraction vocabulary guard.

## Phase 1 — Neutral leader-follower inspection primitives (`paignia`)

- [ ] **T18.01**: Specify and test a domain-neutral leader-follower (Stackelberg) and mixed-strategy inspection game fixture in `paignia-core` / `paignia-solve`, verifying pure/mixed best-response and commitment-value invariants against an independent analytical 2x2 inspection oracle.
- [ ] **T18.02**: Add interval/credal payoff-perturbation bounds for two-player inspection games so parameter intervals over detection effectiveness or evasion cost yield certified lower/upper equilibrium action-share and payoff intervals with solver residual diagnostics.

## Phase 2 — `tobac` and `voiage` consumer handoff (external consumer-owned)

- [ ] **T18.03** *(consumer-owned in `edithatogo/tobac`)*: Bind `tobac` border-seizure and retail-inspection evasion scenarios (`M2a`, `src/tobac/partial_identification.py`) to `paignia`'s neutral inspection game contract, preserving `dapane` enforcement resource cost provenance and `voiage` paired-draw / credal dominance compatibility.
