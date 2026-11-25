# Complementary game-theory libraries

| Tool | Role in Paignia validation and model exchange |
| --- | --- |
| [Gambit](https://gambitproject.readthedocs.io/en/v16.7.0/) | Exact finite-game oracle on the supported formats. Reuse its finite EFG/NFG conversion and polynomial methods when qualifying new solver domains. |
| [OpenSpiel](https://openspiel.readthedocs.io/en/stable/) | Procedural game sources, matched policy and best-response checks, and CFR comparisons. Keep game parameters and observation conventions with the comparison. |
| [GAMUT](https://gamut.stanford.edu/) | Generate benchmark families at fixed seeds and parameters if the supported domains match. UOGTO records its generators and preset keys. Retain each realized payoff table with the benchmark. |
| [Nashpy](https://nashpy.readthedocs.io/en/stable/) | Secondary two-player support, Lemke–Howson and dynamics references. Use its checker for new replicator or bimatrix cases with matched conventions. Gambit remains the exact normal-form oracle. |
| [QuantEcon](https://quanteconpy.readthedocs.io/en/latest/game_theory.html) | Candidate reference for multiplayer strategic games and learning dynamics. Its normal-form readers can supply test translations. Qualify the selected algorithm and Python runtime before using numerical outputs. |
| [PettingZoo](https://pettingzoo.farama.org/) | Sequential and parallel multi-agent environment APIs. Useful for testing state, action and observation exchange when an admitted runtime adapter is available. |

The Rust packages own execution. The validation plan uses source-bound
comparator evidence and chooses a reference per method. Published catalogue
entries describe external game definitions. [Evaluation protocol](../validation/evaluation-plan.md) ·
[Interchange profile](interchange-profile-v1.md)

## Shared Nash diagnostics

For an exactly normalized candidate profile σ, let Vᵢ(σ) be player i's expected payoff. The
gain gᵢ is its best feasible unilateral-deviation payoff minus Vᵢ(σ).
Each method defines the feasible deviations and computes the payoff
difference in the player's original units. For a probability row admitted within its mass
tolerance, the diagnostics retain the method's own weighting convention.
A pure finite profile can use
payoff table lookups. A mixed profile needs expectations against its
opponents' distributions. A behavioural policy needs a best response that
respects information sets.

`NashDeviationReport` gives the per-player values and gains, the maximum
gain and their sum (`NashConv`). Use the maximum gain as a common
approximation check at an explicit tolerance. Where utility scales differ,
interpret the per-player values and gains in their own units. Solver
guarantees, support completeness and refinement conditions stay with the
method. Correlated equilibrium uses conditional deviations after a
recommendation, and Stackelberg and cooperative solutions have their own
solution criteria. The pure gain uses payoff subtraction. Near a floating-point rounding
boundary it can differ from the solver's legacy comparison of the
alternative payoff with the current payoff plus tolerance. The new report
uses compensated addition. The original CFR scalar and JSON retain their
existing summation and rounding.
