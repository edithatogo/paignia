---
title: Solve a normal-form game
description: Compare pure and mixed Nash results for two complete payoff tables.
---

## Replay the payoff tables

Follow the [interchange setup](../interchange/) and run the native profile
replay. The receipt writes the asserted game, JSON request, JSON result and
RDF result under `target/native-profiles` for each example.

| Case | Check the result |
| --- | --- |
| Prisoner's Dilemma | `prisoners-dilemma/result.json` contains one pure profile. Both players defect and receive utility 1. |
| Matching pennies | `matching-pennies/result.json` contains a mixed witness. Each seat plays heads and tails with probability one half. Both utilities and both deviation gains are zero. |

Keep the player and action order from the request. Compare each candidate's
probabilities and original-unit payoffs to the declared game. Read the
support-search status before interpreting the number of candidates. Tied,
singular and unequal-cardinality supports can require other methods.

[Gambit comparison](https://github.com/edithatogo/paignia/blob/main/docs/validation/receipts/t21-core/README.md) ·
[Mixed Nash contract](https://github.com/edithatogo/paignia/blob/main/docs/validation/mixed-nash.md)
