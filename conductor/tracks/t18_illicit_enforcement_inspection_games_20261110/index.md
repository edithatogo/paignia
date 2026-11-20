# T18 Illicit-supply border-enforcement and retail-inspection game contracts for tobac

Status: in progress. The native exact 2x2 leader-follower family is implemented in `paignia-solve` and is not yet a closed T18.01 receipt. Interval bounds and the external consumer handoff remain open. Tobacco-control institutions, customs seizure processes, wastewater calibration and partial-identification intervals remain in `edithatogo/tobac`.

- [Specification](spec.md)
- [Provider and consumer implementation plan](plan.md)
- [Metadata](metadata.json)
- [T17 provider plan and external handoffs](../t17_game_runtime_ecosystem_20261107/index.md)
- [Track registry](../../tracks.md)

This track coordinates neutral leader-follower (Stackelberg) inspection/interdiction game primitives and equilibrium-to-credal payoff bound handoffs for `tobac` illicit-supply substitution (`src/tobac/partial_identification.py`, study `M2a`). It introduces no tobacco-domain vocabulary into `crates/paignia-core` or `crates/paignia-solve`.
