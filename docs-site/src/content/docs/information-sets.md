---
title: Train an information-set policy
description: Train and check an average policy on three-card Kuhn poker.
---

## Replay Kuhn poker

Follow the [interchange setup](../interchange/) and run the native profile
replay. Inspect `kuhn-cfr/request.json` under `target/native-profiles`. Nature
deals distinct cards uniformly. The declaration groups decision histories
by the player's card and the public betting sequence.

Open `kuhn-cfr/result.json` and `kuhn-cfr-plus/result.json`. Check that both
runs record 12,000 iterations, list the action probabilities by
information set and report expected payoffs and NashConv. Player one's
expected value is close to −1/18. On the recorded reference workloads the
unilateral gains decrease across the three checked budgets. The result is
an average policy at the stated iteration budget.

To check graph admission and the independently derived response gains, run:

```sh
cargo test --locked -p paignia-solve --test cfr_information_sets
```

[CFR and CFR+ contract](https://github.com/edithatogo/paignia/blob/main/docs/validation/cfr.md) ·
[OpenSpiel checks](https://github.com/edithatogo/paignia/blob/main/docs/validation/receipts/t21-core/README.md)
