# Bounded counterfactual regret on admitted information sets — T17.07 / S04

## Supported input and method

`train_counterfactual_regret` trains on the single information-set graph admitted
by T17.17. The source must already satisfy the static finite profile, and the
declaration must satisfy
`paignia/static-finite-information-set-perfect-recall/1.0.0`. Imperfect recall,
an incomplete partition, and a legal-action mismatch are rejected before any
regret table is allocated.

Two variants share one tree walk:

| Variant | Identifier | Update |
| --- | --- | --- |
| Vanilla CFR | `paignia/vanilla-cfr` | Both players' instantaneous regrets are computed from the same frozen regret-matching snapshot, then added together. |
| CFR+ | `paignia/cfr-plus` | Players update in `PlayerId` order. The next player sees the updated strategy, and cumulative regret is floored at zero. |

The instantaneous strategy is regret matching on the positive part of cumulative
regret, or the uniform distribution when that part sums to zero. The returned
strategy is the average of the strategies that were played, weighted by the
updating player's own reach at each visited decision. It is not linear CFR, and
an information set that was never reached keeps the uniform distribution.

Chance multiplies the counterfactual reach by the declared edge probability,
including a declared zero-probability edge. Another player's action multiplies
that reach by the current strategy probability. The updating player's own action
does not. Terminal payoffs use the sparse-zero rule: a player omitted from the
terminal map receives zero. No payoff is imputed from a previous node.

The iteration count must be from 1 through 100,000. The caller chooses the
length. Training does not inspect a tolerance and does not declare convergence.

## Diagnostics

`cfr_profile_values` evaluates the supplied behavioral strategy by walking the
same admitted graph. `cfr_best_response_value` chooses one declared action per
information set of the named player, after the information sets below it, and
`cfr_nash_conv` sums the two unilateral gains in a two-player game. All
thresholds in these tests use that sum. OpenSpiel reports two-player
constant-sum exploitability as NashConv divided by two.
[OpenSpiel's diagnostic definitions](https://github.com/google-deepmind/open_spiel/blob/master/open_spiel/python/algorithms/exploitability.py#L109-L143)

## External check that is pinned

Kuhn poker, with ante 1 and bet 1, has first-player equilibrium value `-1/18`.
The citation and the rational constant live in
[`fixtures/conformance/cfr-reference-v1.json`](../../fixtures/conformance/cfr-reference-v1.json).
The focused test builds the 6-deal tree itself, reads the trained probabilities,
and evaluates the deals in a separate arithmetic path. That path is the check
against Kuhn's constant. The trainer is not used as its own oracle.

The same test enumerates the 64 pure strategies of each player against the
other player's mixed average strategy. That enumeration is the independent
NashConv check for this game.

A second Kuhn test repeats that pair of checks at 100, 1,000, and 12,000
iterations for both variants. It compares each seat's best-response payoff with
the pure-policy enumerator at every budget. NashConv shrinks at each step. Absolute
error against `-1/18` stays under `5e-3` at every budget; that error is not
itself monotone. These budgets are not an OpenSpiel or Gambit iteration curve.

With both Leduc players checking and calling, each earns zero. An independent
integer calculation enumerates all 120 ordered deals. It chooses a raise using
the player's private card in round one and their private and public cards in
round two. Against a caller, each such raise adds 2 or 4 matched chips. The
optimal extra return is 176/120, or 22/15 chips per hand in either seat. The
test checks each public best-response value and the all-call NashConv of 44/15.
Schauenberg reports 1.466667 against CallPlayer in §5.1.3 (p. 62, Fig. 5.1 on
p. 63) and the first-to-act equilibrium value of −0.085606 in §5.1.5 (Fig. 5.3,
p. 65). [Schauenberg, *Opponent Modelling and Search in Poker*, MSc thesis, University of Alberta, 2006](https://poker.cs.ualberta.ca/publications/schauenberg.msc.pdf)

## What this slice does not establish

- The Leduc comparison uses the two-player OpenSpiel rule set and the published
  first-player value `-0.085606`. Four thousand CFR+ iterations must land within
  `0.02`, with NashConv below `0.05`. That is not an exact equilibrium, a
  vanilla-CFR curve, or a claim that S04 is closed.
- No iteration curve is claimed to match OpenSpiel, Gambit, or any other
  implementation. Those programs use their own averaging and update schedules.
  The Kuhn budgets above only show this trainer's NashConv shrinking
  under its own independent enumerator.
- This is not an equilibrium catalogue, a welfare claim, or a production
  tolerance. Running more iterations is not automatic acceptance.
- Kairos storage, ontology labels, live observations, and stream isolation are
  not involved.
- T17.17 remains separately unaccepted until its own specialist review and
  hosted CI are recorded. This trainer only consumes a graph that the current
  admission function accepts.

```bash
cargo test --locked -p paignia-solve --test cfr_information_sets
```
