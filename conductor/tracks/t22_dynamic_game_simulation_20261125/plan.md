# T22 implementation plan

## Phase 1 Choose and define the first model

- [x] Task: Freeze the two-state recurring funding reference, including rewards, transition rows, stationary policies and exact values. [Contract](../../../docs/architecture/discounted-game-contract-v1.md)
- [x] Task: Qualify the core and extension terms and answer the competency questions for joint actions, model discounting, reward timing and policy evaluation. [UOGTO profile and suite](https://github.com/edithatogo/UOGTO/pull/145)
- [x] Task: Specify and implement the bounded Rust model contract. It has a separate state and outcome model and no new dependency. [Contract](../../../docs/architecture/discounted-game-contract-v1.md)

## Phase 2 Qualify policy evaluation

- [x] Task: Run reference and rejection fixtures for deterministic and stochastic transitions, cycles, absorbing states, discount edges, exact probability normalization and exhaustion of the workload budget.
- [x] Task: Implement bounded stationary policy evaluation with numerical status, value residual and error bound. Compare the public call with the two-state analytic reference.
- [x] Task: Complete specialist review and locked CI, publish the scoped evidence, and record a consumer intake with a named owner and rollout criteria. The numerical review added an out-of-order mass boundary, a subnormal reward control and a nonzero-deviation oracle. [Evidence](../../../docs/validation/receipts/t22-discounted/README.md) · [Receiving intake](https://github.com/edithatogo/nhra_game/issues/40)

## Phase 3 Qualify exchange and one execution consumer

- [x] Task: Align the UOGTO admission examples and the versioned RDF/JSON profile. The native and installed-consumer replays produced byte-identical requests and responses for both methods and covered the same analytic reference. [Replay evidence](../../../docs/validation/receipts/t22-discounted/README.md)
- [x] Task: Decide whether the reference needs a Kairos seeded trajectory. The selected readout is the expected discounted value with a deterministic policy, so no sampled execution is required.
- [x] Task: Qualify bounded two-player zero-sum Shapley backups and independent deviations. The tests cover pure and mixed saddles and nonzero gains with an exhausted iteration budget.

## Phase 4 Decide the next research slice

- [x] Task: Select Innovate's continuous competing-adoption model as the next reference. Keep the admitted Innovate rates and start with the existing trajectory route. An additional ODE backend needs matched trajectory error and stability evidence. [Innovate #724](https://github.com/edithatogo/innovate/issues/724)
- [x] Task: Define the symmetric two-product logistic reference and route its application qualification to Innovate. UOGTO qualifies time and state semantics. Kairos owns any realised event schedule. Voiage and Dapane join when the decision requires uncertainty or valued resource use. [Reference and owners](https://github.com/edithatogo/innovate/issues/724)
