# T18 Specification — Illicit-supply border-enforcement and retail-inspection game contracts for tobac

## Boundary and ownership

`paignia` owns neutral game-theoretic representations, equilibrium concepts and solver diagnostics (`crates/paignia-core`, `crates/paignia-solve`). It forbids domain-specific vocabulary inside its Rust crates (enforced by `scripts/validate_extraction.py`).

`edithatogo/tobac` owns tobacco/vaping excise policy, border seizure and retail compliance processes, wastewater/customs partial-identification intervals (`src/tobac/partial_identification.py`) and study `M2a` (*Excise, illicit supply and partial identification*).

## Scope

1. **Neutral inspection and interdiction game representation (`T18.01`)**:
   - Define domain-neutral extensive-form / bimatrix leader-follower (Stackelberg) commitment and mixed-strategy inspection game structures in `paignia-core`, where an inspector allocates bounded audit/interdiction effort across channels and an adaptive counterparty chooses evasion/routing effort.
2. **Equilibrium and set-valued payoff bounds under parameter intervals (`T18.02`)**:
   - Support solving or bounding equilibrium evasion/interdiction shares when counterparty cost or detection-elasticity payoffs lie in a convex interval/credal set, returning explicit lower/upper equilibrium payoff and action-share bounds with solver residual receipts.
3. **Consumer handoff to `tobac` partial identification and `voiage` (`T18.03`, external consumer-owned)**:
   - Map `tobac` customs/retail enforcement scenarios and `dapane`-valued enforcement costs into `paignia`'s neutral inspection game contract, feeding the resulting strategic evasion bounds into `tobac`'s `restriction_bounds` (`src/tobac/partial_identification.py`) and `voiage`'s credal EVSI (`tobac_credal_endogenous_voi_20261110`).

## Non-goals and claim boundary

- No tobacco, customs, vaping or healthcare terms enter `crates/paignia-core` or `crates/paignia-solve`.
- Planning registration does not close `T17` scientific obligations (`S01`–`S04`) or claim an installed `tobac` runtime dependency on `paignia`.
