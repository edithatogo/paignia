# Bounded counterfactual regret on admitted information sets — T17.07 / S04

## Supported input and method

`train_counterfactual_regret` trains on the single information-set graph admitted
by T17.17. The source must already satisfy the static finite profile, and the
declaration must satisfy
`paignia/static-finite-information-set-perfect-recall/1.0.0`. Imperfect recall,
an incomplete partition, and a legal-action mismatch are rejected before any
regret table is allocated.

The solver indexes the admitted acyclic graph once. A forward traversal sums
counterfactual reach at shared continuations. A reverse traversal evaluates
continuation payoffs. Work for one player's update is linear in the graph's
nodes and edges. No traversal expands all paths to a shared node.

The two variants use these updates.

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
[OpenSpiel's diagnostic definitions](https://github.com/google-deepmind/open_spiel/blob/4956843d7375e29a4c986425c66f897d7ac15340/open_spiel/python/algorithms/exploitability.py#L121-L157)

The public diagnostics re-admit the supplied graph against its source and check
the exact information-set and action domains. Each strategy row must have finite
probabilities in the unit interval and total mass within `1e-12` of one.
Best-response comparisons use within-state payoff differences and compensated
summation. Continuation values retain a second floating-point component across
chance and decision nodes. NashConv propagates the gains of switching actions before rounding
the two root payoffs. Training scales each player's utilities by a power of two and
restores the original units for expected payoffs. A nonzero product that underflows to zero, a nontrivial product in the
subnormal range, a nonfinite result or a computed negative unilateral gain
returns a numerical error.

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

The Rust tests read the numeric targets, method identifiers, budgets and
tolerances from the conformance fixture. Serde deserializes the original JSON
into test-only structs and rejects duplicate fields. The tests qualify the method version,
the NashConv unit, both budget lists and the single published-value check
against the cited protocol. Replacing the Leduc target with `-0.085606e3`
changes it to −85.606 and fails qualification. The equivalent spelling
`-8.5606e-2` passes.

For each trained Leduc policy, a separate Rust reference walks public betting
histories and keeps a distribution over the opponent's private card. It splits
by board when that card becomes visible. At the responder's turn it maximizes
after summing across compatible hidden cards. It reads only the exported action
labels and probabilities. Both seat values, both best responses and NashConv
must agree with Paignia within `1e-9` at each tested budget.

## Independently trained Leduc checkpoints

OpenSpiel separately trained both variants on its default two-player Leduc
game. Both engines use own-reach averaging without an iteration multiplier.
Vanilla uses simultaneous updates. CFR+ updates seat 1 first in the Paignia
fixture because its player ID `one` sorts before `zero`. The comparison uses
this order in OpenSpiel as well. First-player payoff always means seat 0.
The Rust tests read the pinned OpenSpiel values and NashConv from the
[training fixture](../../fixtures/conformance/leduc-open-spiel-v1.json).

| Variant | Iterations | Paignia payoff | OpenSpiel payoff | Paignia NashConv | OpenSpiel NashConv |
| --- | ---: | ---: | ---: | ---: | ---: |
| Vanilla CFR | 1,000 | −0.09121178 | −0.09121178 | 0.07962657 | 0.07962655 |
| Vanilla CFR | 4,000 | −0.08815894 | −0.08842248 | 0.03271565 | 0.03468534 |
| CFR+ | 1,000 | −0.08573116 | −0.08573022 | 0.01173366 | 0.01173779 |
| CFR+ | 4,000 | −0.08563728 | −0.08563723 | 0.00293114 | 0.00293212 |

Each trainer's NashConv decreases across the two budgets. Paignia's later
NashConv must also fall below OpenSpiel's earlier result for the same variant.
In a two-player zero-sum game the minimax value lies between the seat-0 payoff
after either player's unilateral best response. Thus each policy's distance
to the minimax value is bounded by its NashConv. The difference between the
trainers' payoffs must be at most the sum of their NashConv, with `1e-9` for
roundoff. At 4,000 CFR+ iterations both engines must also be within `0.02` of
the published value `-0.085606` and have NashConv below `0.05`.

OpenSpiel's default seat order with the iteration multiplier disabled gave
CFR+ NashConv of 0.01388640 and 0.00347892. With linear averaging and that
order it gave 0.00049230 and 0.00004309. Linear averaging with the matched
seat order gave 0.00051866 and 0.00005101. These controls show why the tested
variant records its averaging and update schedule. The comparisons cover
the stated game and checkpoints. Equivalence between implementations and
convergence on other games require their own evidence.

### Reproduce the OpenSpiel training run

Use the [OpenSpiel checkout](https://github.com/google-deepmind/open_spiel/tree/4956843d7375e29a4c986425c66f897d7ac15340)
at `4956843d7375e29a4c986425c66f897d7ac15340` and its configured source
dependencies at these revisions.

| Source under the OpenSpiel checkout | Revision |
| --- | --- |
| `open_spiel/abseil-cpp` | `d38452e1ee03523a208362186fd42248ff2609f6` |
| `open_spiel/json` | `55f93686c01528224f448c19128836e7df245f72` |
| `open_spiel/games/bridge/double_dummy_solver` | `091ea94358a4016d4fb6069dea5c452cdc98d0bd` |

In `open_spiel/algorithms/cfr.cc`, change the alternating-update loop in
`CFRSolverBase::EvaluateAndUpdatePolicy` to
`for (int player = game_->NumPlayers() - 1; player >= 0; --player)`.
Copy the [reference driver](../../fixtures/conformance/leduc-open-spiel-reference.cc)
to `open_spiel/paignia_leduc_reference.cc` and append this target to
`open_spiel/CMakeLists.txt`.

```cmake
add_executable(paignia_leduc_reference paignia_leduc_reference.cc)
target_link_libraries(paignia_leduc_reference PRIVATE
  open_spiel::open_spiel
  open_spiel::games::leduc_poker::leduc_poker
  open_spiel::algorithms::cfr
  open_spiel::algorithms::expected_returns
  open_spiel::algorithms::tabular_exploitability
)
```

With `spiel_checkout` set to that source checkout and `build_dir` set to
a fresh build directory, build and run the reference executable.

```bash
OPEN_SPIEL_BUILD_WITH_PYTHON=OFF cmake -S "$spiel_checkout/open_spiel" \
  -B "$build_dir" -DCMAKE_BUILD_TYPE=Release \
  -DOPEN_SPIEL_BUILD_WITH_PYTHON=OFF -DJSON_BuildTests=OFF
cmake --build "$build_dir" --target paignia_leduc_reference --parallel 4
"$build_dir/paignia_leduc_reference"
```

The recorded run used CMake 4.5.1 and AppleClang 21.0.0.21000401 on arm64.
Its full CSV has SHA-256
`84435a0069019b8bff3aba4d1d66da6a7089bd150083d0cd32642a2f7b66a6c4`.
The two matched schedules supply the Rust fixture. The fixture reader checks
each numeric pair against the recorded run. Paignia uses the C++ executable
as development evidence. Its solver and shipped dependencies stay Rust.

## Scope of acceptance

The Kuhn budgets use the independent deal and pure-policy enumerators. The
Leduc budgets additionally use the independently trained OpenSpiel checkpoints.
The numerical acceptance covers finite-game training and the stated payoff
and regret diagnostics. Separate contracts govern live observations, ontology
and simulation replay.
UOGTO accepted the bounded executable information-set profile in
[UOGTO #139](https://github.com/edithatogo/UOGTO/pull/139).
Kairos owns the legacy CFR caller and source disposition in
[Kairos #406](https://github.com/edithatogo/kairos/pull/406).

```bash
cargo test --locked -p paignia-solve --test cfr_information_sets
```
