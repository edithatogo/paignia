# Native provider review on 20 November 2026

## Information sets and CFR

The `paignia/static-finite-information-set-perfect-recall/1.0.0` profile admits a
complete, ordered partition of a finite graph's decision nodes. The specialist
review covered player and action ownership, perfect recall on shared paths,
and bounded space. The final validation avoids a player-by-node table.
[PR #37](https://github.com/edithatogo/paignia/pull/37)

Vanilla CFR and CFR+ operate on the admitted graph. Review fixes cover shared
counterfactual reach, best responses grouped by information set, probability
mass validation, and numerical loss in continuation values and deviation gains.
An independent Kuhn enumerator checks both players across three budgets. The
Leduc reference checks both players' values and best responses for each trainer
at 1,000 and 4,000 iterations. CFR+ also checks the published first-player
value at 4,000 iterations.
[CFR contract and references](cfr.md)
/ [T17 integration](https://github.com/edithatogo/paignia/pull/53)
/ [Native CI](https://github.com/edithatogo/paignia/actions/runs/44729717948)

T17.17 and T17.07 remain open for the executable UOGTO information-set
contract and Kairos's source-side S04 disposition. UOGTO #136 addresses a
perfect-information declaration and has failing qualification checks at
`672cde1988b9d1bbfc699c1d4c09277dbe47a713`. The separate Kairos port is the
prerequisite for T17.08, T17.15 and T17.16.
[UOGTO #136](https://github.com/edithatogo/UOGTO/pull/136)
/ [Kairos Track 114](https://github.com/edithatogo/kairos/blob/bac331fca95420a94db240c02ae66c1991b85da7/conductor/tracks/114-dependency-and-consumer-contracts/plan.md)
/ [Kairos Track 110](https://github.com/edithatogo/kairos/blob/bac331fca95420a94db240c02ae66c1991b85da7/conductor/tracks/110-scientific-method-completion/plan.md)

## Binary inspection and Nash bounds

T18.01 covers the declared exact binary commitment and simultaneous Nash
family. Review covers reduced-rational arithmetic, follower ties, and the
distinction between attained values and suprema.
[Commitment contract](leader-follower.md)
/ [PR #41](https://github.com/edithatogo/paignia/pull/41)

T18.02 implements exact coordinate projections on supported parameter
intervals and conservative Nash enclosures for convex hulls of paired
two-by-two payoff tables. The general hull method reports payoff and action
share bounds, cell counts, and maximum deviation gain over the retained cells.
Tests cover interior equilibria, ties, nonvertex equilibria, and regret on the
retained cells. Commitment projections have their own parameter limits.
[Credal Nash contract](credal-nash.md)
/ [PR #54](https://github.com/edithatogo/paignia/pull/54)
/ [Native CI](https://github.com/edithatogo/paignia/actions/runs/44730300281)

The T18 source merged in order from #41 through #50, then #54 at
`a8b0f976d4bfc2c3b7c38cef709bf6fd771d1995`. Stable and minimum-version Rust,
both synthetic oracle suites, and privacy checks passed on the reviewed heads.
The offline jobs were queued and are recorded in the merge commits under the
maintainer's T17/T18 instruction. T18.03 remains with the application owner.

## Integrated verification and release eligibility

PR #55 integrates the final #54 regression, uses the exact CPython 3.15.2
project pin, and preserves the native, package isolation, numerical, inspection
and privacy controls. The package source-isolation tests use Linux
`bubblewrap` in hosted CI.
[Verification PR #55](https://github.com/edithatogo/paignia/pull/55)

The static handoff binds the accepted `bd880dd` artifacts. The release
validator reported ineligible because its manifest contains zero application
receipts. T17.09 and T17.10 supply the two distinct real cases. T17.11 is the
reimbursement application's valued-payoff contract. Registry publication
remains disabled.
[Application handoff](game-consumer-handoff.md)
/ [Receipt manifest](game-consumers.json)
