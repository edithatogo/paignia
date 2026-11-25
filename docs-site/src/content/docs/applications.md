---
title: Applied models
description: Follow the independent evidence behind two downstream Rust integrations.
---

## Drug reimbursement offers

The synthetic Chapter 8 case models a firm making an offer and a buyer
choosing whether to reimburse it under a fixed budget. Run the locked
application test from the receiving repository:

```sh
cargo test --locked --test paignia_chapter8_application
```

The tests compare the selected offer, firm rent and net economic benefit in
health with the chapter's calculation. The application owns the assumptions
and valuation. [Application model and commands](https://github.com/edithatogo/new-drug-reimbursement-game/blob/11ee1003b21f7e21d9b8ff428f783f9a7028eff0/docs/architecture/paignia-chapter8-application.md)

## Sequential innovation pricing

The Innovate synthetic case links rebates and observed demand to a bounded
commitment calculation. Reproduce the qualified adapter and compare its
zero/positive demand regimes and tie cases. Use the candidate revision and
commands in the receipt. [Exact application evidence](https://github.com/edithatogo/paignia/blob/main/docs/validation/receipts/t20-candidate/README.md)
