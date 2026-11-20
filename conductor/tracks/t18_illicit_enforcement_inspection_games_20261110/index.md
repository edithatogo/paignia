# T18 Illicit-supply border-enforcement and retail-inspection game contracts for tobac

Status: in progress. T18.01 and T18.02 have merged and passed review for the
declared binary game family and Nash payoff hull enclosure. The application
handoff T18.03 remains open in `edithatogo/tobac`.

- [Specification](spec.md)
- [Provider and consumer implementation plan](plan.md)
- [Metadata](metadata.json)
- [Provider review and verification](../../../docs/validation/provider-review-20261120.md)
- [T17 provider plan and external handoffs](../t17_game_runtime_ecosystem_20261107/index.md)
- [Track registry](../../tracks.md)

This track coordinates neutral leader-follower (Stackelberg) inspection/interdiction game primitives and equilibrium-to-credal payoff bound handoffs for `tobac` illicit-supply substitution (`src/tobac/partial_identification.py`, study `M2a`). It introduces no tobacco-domain vocabulary into `crates/paignia-core` or `crates/paignia-solve`.
