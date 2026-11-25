# Product boundary

Paignia supplies reusable Rust game representation, supported solvers and
explicit diagnostics. The accepted native profile supports finite
perfect-information backward induction/chance expectations, source-bound result
emission, reversible static identity conversion and owned decision observations.
Separate bounded mathematical APIs supply pure Nash, mixed-Nash support
enumeration, optional correlated equilibrium, classical RK4 and named Euler.
Their input and numerical contracts have independent specialist controls;
normal-form tables and ODE states are not admitted as extensive-game graphs.
The bounded perfect-recall information-set profile supports native CFR and
CFR+ with independent Kuhn and Leduc controls. The binary commitment solver and
Nash payoff hull enclosure expose their parameter domains and diagnostics.
[Provider review](../docs/validation/provider-review-20261120.md)

Kairos accepted execution of a bounded finite perfect-information path,
including isolated random streams and in-process checkpoint replay. UOGTO
accepted the executable perfect-recall information-set profile.
[Kairos acceptance](https://github.com/edithatogo/kairos/pull/405) ·
[UOGTO acceptance](https://github.com/edithatogo/UOGTO/pull/139)

The optional pinned Clarabel backend stays in solver scope; core and default
solver consumers do not acquire that backend or its serialization dependencies.
The accepted correlated-equilibrium method adds actual incentive constraints,
original-unit primal/dual admission and explicit approximate status. Backend
feasibility alone does not establish that method's qualification.

Static packages are independent of Kairos. The optional execution adapter
consumes Kairos; neither generic provider imports the adapter. UOGTO remains the
semantic authority. Applications own their games and payoffs. HEOR valuation
and Voiage information, perspective and heterogeneity value remain distinct
capabilities. Reuse Voiage and Dapane through qualified optional composition or
versioned artifacts with explicit utility assumptions and no dependency cycles.

The [game intake contract](../docs/integrations/downstream-game-intake.md)
names the six current downstream applications and the UOGTO/Paignia feedback
required for their game changes. Innovate remains a parallel innovation library;
its mature generic game methods move to Paignia gradually after parity and
consumer qualification. Versioned health-game tutorials adopt accepted library
and application capabilities over time.

The long-term goal is independently verified numerical behavior, efficient
compact representations and reproducible package/consumer contracts. Comparative
frontier claims require workload-specific measurement; source transfer supplies
no benchmark or new scientific result.

[T22](tracks/t22_dynamic_game_simulation_20261125/index.md) extends the
research scope to strategic interaction over time. State, actions, observations,
rewards, evolution rules and time horizon characterize a model. Proposed work
includes repeated and stochastic games, continuous-time differential games and
population or learning simulations. The first target is a discounted finite
stochastic game with independently checked policy values.
