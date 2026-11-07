# Paignia identity and provider handoff

The user selected the existing `edithatogo/paignia` repository. Its initial
README commit is `0e28f1bf33ed8c7be5fa1d63b8cc40517b61d22c`. This native seed
transfers the two generic packages from the reimbursement incubator; it does not
publish registry artifacts or close scientific/consumer qualification.

| Current package | Original source identifier | Ownership |
|---|---|---|
| `paignia-core` | `uogto-game-core` | Paignia generic representation and validation. |
| `paignia-solve` | `uogto-game-solve` | Paignia bounded static algorithms and diagnostics. |
| Future `paignia-kairos` | Proposed `uogto-game-kairos` | Optional Paignia-owned adapter; no package yet. |
| Future semantic/tabular tooling | Earlier `paignia-uogto` / `paignia-arrow` proposals | Add modules/packages only for an actual supported consumer contract. |

Only one native implementation is maintained. The reimbursement application
keeps its domain code and becomes an immutable Paignia dependency consumer; its
source crates are removed when that switch is validated. Old names remain
provenance aliases, not compatibility forks. Native source algorithms are
unchanged by the transfer. Exact source/license history is in
`extraction-manifest.json` and the [transfer map](../../../docs/architecture/paignia-transfer-map.md).

## Existing task IDs and aliases

| IDs | Canonical owner | Work retained |
|---|---|---|
| T17.01–T17.08 | Paignia | Public/static package contract, packaging, S01–S04 and optional native adapter. |
| T17.09 | Reimbursement application | Its actual consumer model and artifact receipt. |
| T17.10 | Second application | Its distinct real model and matching artifact receipt. |
| T17.11 | Reimbursement application | Its valued-payoff units/perspective contract. |
| T17.12–T17.17 | Paignia | Release eligibility, conversion, observations, RNG, replay and information sets. |

PN01 aliases T17.01/.03 identity/compatibility work; PN02 aliases T17.03 package
migration; PN03 aliases T17.01/.04–.07/.13 source/method ownership; PN04 aliases
T17.12 promotion. There are seventeen stable IDs and fourteen provider-owned
implementation leaves, not two competing T17 programmes. External entries in
the provider plan are handoffs, with completion evidence supplied by their
owning repository. Incubator issue #100 retains history and links here.

## Qualification after repository creation

Repository creation has occurred by user direction. The previous incubator's
future repository-extraction eligibility wording now applies to **bounded
package release eligibility**: T17.01–.03 and two real application receipts,
T17.09/.10, feed T17.12. None of those requirements is waived. The source smoke
runs native tests from copied packages outside the workspace with explicit
metadata/lock resolution; it does not claim registry artifacts or actual
application release acceptance.

Static qualification is independent of the optional adapter and unrelated
advanced methods. Dynamic claims add only the native/semantic/RNG/replay
receipts their features use. S01 RK4, S02 mixed Nash, S03 CE and S04 CFR/CFR+
remain open. UOGTO retains canonical semantic profiles; Kairos retains execution
primitives. No copied legacy method is promoted into a qualified Paignia API.

Routine implementation uses bounded leaves, focused checks and coherent review.
Publication is disabled in current Cargo manifests. Source moves, immutable
consumer updates and ordinary documentation work need no new human approval
loop when already authorized.
