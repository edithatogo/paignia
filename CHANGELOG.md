# Changelog

## Unreleased

- Added versioned finite-game profiles, role-specific identities, result
  envelopes, and policy observations in `paignia-core`. [Runtime contract](docs/architecture/game-runtime-contract-v1.md)
- Added backward induction, bounded Nash support enumeration, and CFR/CFR+
  for the supported finite game profiles. [Mixed Nash](docs/validation/mixed-nash.md) · [CFR](docs/validation/cfr.md)
- Added exact binary leader–follower solving and conservative Nash payoff
  hull bounds. [Binary commitment](docs/validation/leader-follower.md) · [Nash bounds](docs/validation/credal-nash.md)
- Added the optional Kairos adapter for bounded native execution, isolated
  random streams, and in-process replay. [Native execution contract](docs/architecture/game-native-execution-v1.md)
- Qualified the same static package artifacts with reimbursement and
  Innovate application cases. [Consumer evidence](docs/validation/game-consumer-handoff.md)
