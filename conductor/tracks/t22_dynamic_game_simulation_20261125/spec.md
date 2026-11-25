# T22 specification

## Extended scope

Game-theoretic simulation models how interacting decision makers choose
actions, receive information and payoffs, and affect the evolution of a
system over time. A model can use finite or continuous state and action
spaces, discrete, event-driven or continuous time, and finite or indefinite
horizons. Evaluation of a fixed policy, computation of an equilibrium,
simulation of learning and measurement of a realised trajectory are separately
specified tasks. A bounded computation can approximate an indefinite-horizon
model with an explicit error or stopping rule.

| Family | Modelling scope | Candidate path |
| --- | --- | --- |
| Repeated and stochastic games | A stage game repeats or actions change a state through a probability kernel. Policies may depend on the state or on history. | Start with finite, fully observed, two-player discounted Markov models and fixed stationary policies. Treat history-dependent policies and broader solution concepts as distinct research questions. [Shapley](https://www.pnas.org/doi/10.1073/pnas.39.10.1095) |
| Differential and hybrid games | Control choices affect a state equation and payoffs in continuous time, with possible discrete changes. | Begin with an analytic small control example. Qualify trajectory error, control constraints and the chosen equilibrium concept before evaluating an integration library. [Başar and Olsder](https://epubs.siam.org/doi/10.1137/1.9781611971132.ch5) |
| Evolutionary, mean-field and agent-based games | Policies or strategies evolve through interactions among agents or through a changing population distribution. | Reuse bounded replicator evidence, compare a small finite population with a distribution-level result, then consider a Kairos execution adapter for sampled interactions. [Lasry and Lions](https://doi.org/10.1007/s11537-007-0657-8) · [OpenSpiel game implementations](https://github.com/google-deepmind/open_spiel) |
| Multi-agent learning | Agents update policies from observed interaction, with specified timing and information. | Start with a fixed-policy or scripted learner comparison and a paired observation trace. Use [OpenSpiel](https://openspiel.readthedocs.io/en/latest/algorithms.html) or [PettingZoo](https://pettingzoo.farama.org/api/aec/) as external reference environments. |

The release contracts remain the source of truth for supported methods.
T22 begins with one synthetic model and a numeric contract. Higher-dimensional
Nash feasibility and the review manuscript retain their existing scope.

## First executable slice

Use a small cyclic, two-player zero-sum stochastic model with finite,
fully observed states and actions, bounded rewards and an explicit discount
factor `0 ≤ γ < 1`. Name the initial distribution and keep state and action
identity stable across input, policy, and diagnostics. One deterministic
repeat and one action-dependent stochastic transition are sufficient to
exercise the model. Keep this profile separate from finite acyclic trees.

Evaluate a specified stationary policy pair before adding a solver for
strategic choices. Compare the expected values to an independent analytical
or exact small-system reference. Report the Bellman residual, iteration cap,
termination reason and a value error bound. For an exact discounted backup,
the sup-norm residual divided by `1 − γ` bounds value error. Account for
numerical error in computing the backup and test the slow-convergence case
as `γ` approaches one. Verify a policy deviation with separate best-response
evaluation before describing a policy as an approximate equilibrium.

Reject nonfinite rewards, incomplete joint actions, invalid probability mass,
out-of-range discounts and work above explicit limits. Include `γ = 0`,
absorbing states, a reachable cycle and a controlled budget-exhaustion result.
A truncated OpenSpiel comparison can use the same labelled finite rollout and
an explicit discounted tail bound for the indefinite-horizon case.

## Dependency choices

| Need | First choice | Evaluate when required |
| --- | --- | --- |
| Discounted tabular evaluation | Rust bounded arrays, existing Paignia identifiers and validation patterns, and independent hand solutions. | Reuse Paignia's optional pinned [Clarabel](https://clarabel.org/stable/) only if the later zero-sum minimax step needs a matrix-game LP. Qualify LP feasibility and the outer residual together. |
| Time, events, random streams and replay | The existing optional [Kairos](https://github.com/edithatogo/kairos) boundary for execution with event timing or sampled trajectories. | Qualify a small seeded execution adapter after the tabular model passes. Transfer event order and observation provenance with the replay. |
| Continuous dynamics | Existing bounded RK4/Euler controls as references for the supported smooth cases. | Compare [ode_solvers](https://github.com/srenevey/ode-solvers) for adaptive explicit integration and [Diffsol](https://github.com/martinjrobins/diffsol) for stiff or event-mediated problems. Check convergence, events, license, closure, toolchain and numerical behavior before choosing a backend. |
| Independent comparison | Analytical cases, OpenSpiel and PettingZoo in verification environments. | Add one pinned external example when it reproduces the same actions, timing, information and reward convention. |

Start without a new production dependency. The default Rust packages
retain the declared 1.75 baseline and static dependency boundary. Keep any
additional numerical backend optional and justify it with a measured case.
If a selected backend needs a newer compiler, qualify it in a separately
versioned build before proposing a change to the default toolchain.

## Ownership and interchange

- **UOGTO** owns semantic classes, competency questions and SHACL. Check the
  pinned [MarkovGame, policy, observation and reward terms](https://github.com/edithatogo/UOGTO/blob/d93dfffe192fed2f833bd9b0681694113f663b9c/ontologies/extensions/marl.ttl),
  the core [transition vocabulary](https://github.com/edithatogo/UOGTO/blob/d93dfffe192fed2f833bd9b0681694113f663b9c/ontologies/core/dynamics.ttl),
  and existing [differential](https://github.com/edithatogo/UOGTO/blob/d93dfffe192fed2f833bd9b0681694113f663b9c/ontologies/extensions/differential-hybrid-games.ttl)
  and [mean-field modules](https://github.com/edithatogo/UOGTO/blob/d93dfffe192fed2f833bd9b0681694113f663b9c/ontologies/extensions/mean-field-games.ttl).
  Assess joint-action indexing, conditional mass, reward timing, horizon,
  discount, observations and policy identifiers. The current
  [discountFactor term](https://github.com/edithatogo/UOGTO/blob/d93dfffe192fed2f833bd9b0681694113f663b9c/ontologies/extensions/learning-in-games.ttl)
  has domain `LearningProcess`. Prove that a term's domain and meaning fit
  model discounting before reusing or extending it.
- **Paignia** owns admitted Rust models, policy evaluation, methods,
  diagnostics and versioned exchange profiles. Use a new named profile when
  the model, examples and graph admission are ready. Require a paired
  RDF/JSON replay with labelled rewards and policies before enabling execution.
- **Kairos** owns native simulation time, events, resources, random streams
  and replay. The optional integration consumes Paignia's model and produces
  a realised path. Keep exact policy evaluation independently callable.
- **Voiage** owns value of information, value of perspective and value of
  heterogeneity. Supply versioned policy outcomes, conditional observations
  and uncertainty with explicit units when an application evaluates them.
  [Voiage](https://github.com/edithatogo/voiage)
- **Dapane** owns costing and valuation. Pass unit-labelled resource use
  and apply validated, study-specific valuation before entering payoff
  data into a game. Keep neutral event records reusable.
  [Dapane](https://github.com/edithatogo/dapane)
- **Innovate and applications** own institutions, empirical parameters and
  payoff calibration. A synthetic repeated funding or innovation-adoption
  model can exercise the first profile. The originating application still
  qualifies any production use under the
  [downstream intake](../../../docs/integrations/downstream-game-intake.md).
