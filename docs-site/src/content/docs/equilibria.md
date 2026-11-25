---
title: Nash and information sets
description: Choose a method by representation and inspect its deviation gains.
---

| Model | Method and evidence |
| --- | --- |
| Complete finite payoff table | Pure Nash enumerates stable deterministic choices. The wire profile takes two players. |
| Two-player payoff tables with up to eight actions per player | Mixed support enumeration supplies verified candidates. Inspect unresolved supports and maximum gain. |
| Finite acyclic game with an admitted perfect-recall partition | CFR or CFR+ trains an average policy at the chosen budget. Evaluate its payoffs and NashConv. |

Run the maintained independent cases:

```sh
cargo test --locked -p paignia-solve --test mixed_nash
cargo test --locked -p paignia-solve --test cfr_information_sets
```

In a reported policy, each player has an expected payoff and an original-unit
gain from a best unilateral deviation. For matching pennies the checked
half-half probabilities yield zero gain for either player. For Kuhn poker
the first player's value is near −1/18 at the declared budget. Use the
method contract for bounds, off-path actions and numeric precision.

[Mixed Nash contract](https://github.com/edithatogo/paignia/blob/main/docs/validation/mixed-nash.md) ·
[CFR contract](https://github.com/edithatogo/paignia/blob/main/docs/validation/cfr.md) ·
[Other game-theory libraries](https://github.com/edithatogo/paignia/blob/main/docs/architecture/game-theory-library-roles.md)

[Replay the normal-form games](../normal-form/) and [train a Kuhn policy](../information-sets/).
