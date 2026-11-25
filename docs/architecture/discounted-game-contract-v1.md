# Discounted zero-sum game contract

The v1 profile models two players who observe the current state and act
simultaneously. Each player has two actions, in declared order, available in
every state. The immediate reward belongs to player zero and player one's
reward is its negation. A draw from the selected transition row determines
the next state. The same discount applies after every stage, starting with
the second reward. [Shapley, Stochastic Games](https://doi.org/10.1073/pnas.39.10.1095)

The native model admits 1–32 ordered states, distinct nonblank identifiers
within each axis, and finite rewards of absolute magnitude at most `1e12`.
The discount is in `[0,1)`. Initial, transition and policy rows contain
binary64 weights in `[0,1]` whose exact sum is within `1e-12`
of one. A row denotes those supplied weights divided by their exact sum.
Both policies are independently mixed and stationary. State order determines
the initial and transition axes and the order of the policy records.

## Solving and interpreting the result

The public Rust entry points are `paignia_solve::discounted::evaluate` and
`paignia_solve::discounted::solve_zero_sum`. A caller chooses 1–10,000
iterations per value calculation and a finite nonnegative absolute value
tolerance. Starting at zero, each iterate applies the fixed-policy or
minimax Bellman backup to the previous vector. The result reports the
outward residual bound `r`, the per-state error bound `r / (1 - discount)`
and a separate bound for the initial expected value. Every result names
the states in input order and gives all values from player zero's perspective.

The minimax method brackets a two-by-two backup using a feasible mixed
strategy for each side and their guaranteed payoffs. It also evaluates the
returned stationary policies and each player's discounted best response to
the other policy. The two deviation bounds cover the largest possible
unilateral gain over all starts in the admitted state set. Consumers should
choose an acceptable deviation bound and inspect it along with the statuses.
`value_tolerance_reached` describes the value calculations. At
`iteration_limit`, the remaining error bounds quantify the available result.

The recurring-funding reference is synthetic. The two states are stable
and pressure. Commissioner actions are settle and defer. Provider actions
are accept and contend. Each entry gives the commissioner reward and the
probability of the stable successor.

| Current state | Settle, accept | Settle, contend | Defer, accept | Defer, contend |
| --- | --- | --- | --- | --- |
| Stable | (2, 1/2) | (3, 1) | (0, 0) | (1, 1/2) |
| Pressure | (-1, 1/4) | (-1, 1) | (-2, 0) | (1, 0) |

With discount 3/4 and both players always choosing their first action,
the Bellman equations are `5v - 3w = 16` and `-3v + 7w = -16`. Thus the
state values are 32/13 and -16/13. Initial weights 3/4 and
1/4 give 20/13. Substituting these values into all joint outcomes
gives a saddle at (settle, accept) in both states. This reference has no
empirical estimates, patients or institution-specific calibration.

## Optional JSON projection

Call `paignia_io::solve_profile_json` with wire version `1.0.0` and
`semantic_profile` set to `paignia/discounted-zero-sum-2x2/1.0.0`.
The envelope carries the source RDF SHA-256 and ontology commit, along
with separate representation, specification, instance and run identities.
The `game` object has kind `discounted` and declares its ordered players
and actions, states and joint outcomes, initial weights and discount.
`method` has kind `evaluate` with the ordered `policy`, or kind
`minimax`. Both methods require `max_iterations` and `value_tolerance`.
The response preserves the envelope, the player and action identifiers,
the policy weights and the applicable numerical bounds.

The caller validates asserted RDF and binds its hash and ontology
revision before invoking the native entry point. The two-by-two profile
covers expected values. A realised trajectory with a clock or random
stream remains a separate Kairos integration.
