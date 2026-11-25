# T22 implementation plan

## Phase 1 Choose and define the first model

- [x] Task: Freeze the two-state recurring funding reference, including rewards, transition rows, stationary policies and exact values. [Contract](../../../docs/architecture/discounted-game-contract-v1.md)
- [~] Task: Qualify the core and extension terms and answer the competency questions for joint actions, model discounting, reward timing and policy evaluation. [UOGTO #144](https://github.com/edithatogo/UOGTO/issues/144)
- [x] Task: Specify and implement the bounded Rust model contract. It has a separate state and outcome model and no new dependency. [Contract](../../../docs/architecture/discounted-game-contract-v1.md)

## Phase 2 Qualify policy evaluation

- [x] Task: Run reference and rejection fixtures for deterministic and stochastic transitions, cycles, absorbing states, discount edges, exact probability normalization and exhaustion of the workload budget.
- [x] Task: Implement bounded stationary policy evaluation with numerical status, value residual and error bound. Compare the public call with the two-state analytic reference.
- [~] Task: Complete specialist review and locked CI, publish the scoped evidence, and record a consumer intake with a named owner and rollout criteria. The numerical review added an out-of-order mass boundary, a subnormal reward control and a nonzero-deviation oracle.

## Phase 3 Qualify exchange and one execution consumer

- [ ] Task: Align the UOGTO admission examples and a versioned RDF/JSON profile. Replay a frozen model, policy and result while preserving identity, probability mass and reward timing.
- [x] Task: Decide whether the reference needs a Kairos seeded trajectory. The selected readout is the expected discounted value with a deterministic policy, so no sampled execution is required.
- [x] Task: Qualify bounded two-player zero-sum Shapley backups and independent deviations. The tests cover pure and mixed saddles and nonzero gains with an exhausted iteration budget.

## Phase 4 Decide the next research slice

- [ ] Task: Choose a real follow-on need from differential control, population-level dynamics, or agent learning. Compare the existing implementation route with the smallest qualified dependency and record the decision.
- [ ] Task: Define one independent numerical or trajectory reference for that need and route UOGTO, Kairos, Voiage, Dapane and application changes to their respective owners.
