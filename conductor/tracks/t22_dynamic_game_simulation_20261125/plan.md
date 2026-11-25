# T22 implementation plan

## Phase 1 Choose and define the first model

- [ ] Task: Select one synthetic recurring funding or adoption game with two labelled states and two actions per player. Freeze the rewards, transition rows, stationary policies and analytic value reference.
- [ ] Task: Record the matching core and extension IRIs, identify semantic gaps in UOGTO, and define competency questions for joint actions, model discounting, reward timing and policy evaluation.
- [ ] Task: Specify the smallest Rust model contract and its limits. Review independence from the existing acyclic game and numerical support contracts.

## Phase 2 Qualify policy evaluation

- [ ] Task: Write reference and rejection fixtures for both deterministic and stochastic transitions, cycles, absorbing states, discount edges and exhaustion of the workload budget.
- [ ] Task: Implement bounded stationary policy evaluation with numerical status, value residual and error bound. Compare the actual public call to the independent reference.
- [ ] Task: Complete specialist review and locked CI, publish the scoped evidence, and record a consumer intake with a named owner and rollout criteria.

## Phase 3 Qualify exchange and one execution consumer

- [ ] Task: Align the UOGTO admission examples and a versioned RDF/JSON profile. Replay a frozen model, policy and result while preserving identity, probability mass and reward timing.
- [ ] Task: Qualify an optional Kairos seeded trajectory only if the selected use needs time, events or sampling. Compare execution receipts and replay with the agreed contract.
- [ ] Task: Evaluate a bounded two-player zero-sum equilibrium method after policy evaluation. Verify independent deviations and the error bound around both the matrix-game backups and the outer iteration.

## Phase 4 Decide the next research slice

- [ ] Task: Choose a real follow-on need from differential control, population-level dynamics, or agent learning. Compare the existing implementation route with the smallest qualified dependency and record the decision.
- [ ] Task: Define one independent numerical or trajectory reference for that need and route UOGTO, Kairos, Voiage, Dapane and application changes to their respective owners.
