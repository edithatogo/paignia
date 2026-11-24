# T17 Paignia native library and ecosystem handoff

T17 is complete with seventeen accepted leaves. The native static contract,
package boundary and archives, RK4/Euler,
mixed Nash, optional correlated equilibrium, CFR/CFR+, static conversion
and owned observations have accepted independent review and integrated compiler/package
checks. The [17 November receipt](../../../docs/validation/t17-review-20261117.md)
records exact source, corrections, nonzero test counts and remaining limits.

T17.09 supplies the first application artifact receipt for the reimbursement
Chapter 8 model. UOGTO accepted the executable information-set profile.
Kairos accepted the bounded adapter, random-stream isolation and checkpoint
replay. The independent Leduc comparison and Kairos's legacy CFR retirement
complete T17.07. T17.11 has an accepted valued-payoff port and synthetic
application comparison. T17.10 accepts Innovate's reviewed price commitment
case, and T17.12 admits both application receipts for graph backward induction.
[Innovate T17.10 acceptance](https://github.com/edithatogo/innovate/pull/696) ·
[Reimbursement T17.11 acceptance](https://github.com/edithatogo/new-drug-reimbursement-game/pull/118) ·
[CFR validation](../../../docs/validation/cfr.md) ·
[Kairos S04 retirement](https://github.com/edithatogo/kairos/pull/406) ·
[Kairos port acceptance](https://github.com/edithatogo/kairos/blob/f7484751c4a058581f2425ca493faec8e85e5637/conductor/contracts/game-port-acceptance.md) ·
[UOGTO information-set profile](https://github.com/edithatogo/UOGTO/blob/3000c231b2c102e18bf7c3294bec498f00f1976c/docs/contracts/executable-information-set-profile-v1.md) ·
[Consumer evidence](../../../docs/validation/game-consumers.json)

- [Specification](spec.md)
- [Provider implementation plan and external owner entries](plan.md)
- [Identity and promotion record](paignia-identity-and-promotion.md)
- [Metadata](metadata.json)
- [20 November provider review](../../../docs/validation/provider-review-20261120.md)
- [T18 inspection/interdiction game contracts for tobac](../t18_illicit_enforcement_inspection_games_20261110/index.md)
- [Source transfer and retention map](../../../docs/architecture/paignia-transfer-map.md)

Paignia now owns T17.01–T17.08 and T17.12–T17.17. The reimbursement application
retains T17.09 (its actual consumer case) and T17.11 (valued-payoff input).
T17.10 belongs to the selected second consumer, currently the Innovate
G01–G04 programme. The external entries preserve IDs and dependency references
without creating duplicate local implementation or completion authority.

The previous incubator T17 and issue #100 retain history and forward to this
provider plan. Inherited S01–S04 obligations remain individually tracked, including Kairos
Track 110's separate source-side regressions and retirement. Source code has one Paignia owner;
application dependency acceptance and actual immutable pins remain consumer work.
