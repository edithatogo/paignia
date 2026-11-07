# Paignia and Voiage integration handoff

Date: 8 November 2026. Status: planning only; implementation and qualification pending.

The [Conductor scientific-contract refinements](../../conductor/refinements/scientific-contracts-20261108.md)
map the approved acceptance tests into existing T17 leaves. Read them alongside
this handoff; they do not create a second status ledger or take over source transfer.
The [parallel-plan reconciliation](parallel-plan-reconciliation-20261108.md) records
current provider evidence, owner PRs and independently scoped acceptance paths.

## Canonical ownership

The maintainer has identified `edithatogo/paignia` as the reusable Rust game-library
repository. `new-drug-reimbursement-game` is the historical source incubator and a
reimbursement application/consumer, not the permanent home of generic solvers.

The bootstrap observation at `0e28f1bf33ed8c7be5fa1d63b8cc40517b61d22c` is historical.
Paignia PR #2 has merged the bounded native seed as
`12b22ce7571b9c590405bf53c2c1c5862b49cd64`, containing `paignia-core` and
`paignia-solve` v0.1.0 with publishing disabled. Source transfer and reported
extracted-source smoke are not registry publication, general solver validity or
two-real-application acceptance. The linked reconciliation distinguishes those
states; this handoff does not rerun or certify the provider's native checks.

| Owner | Responsibility |
| --- | --- |
| Paignia | Reusable Rust game representations, supported strategic algorithms, response policies, solution catalogues and numerical diagnostics |
| UOGTO | Ontology, semantic identifiers and executable-profile meanings; not numerical acceptance |
| Kairos | Optional dynamic time/events/state/RNG and replay; not a static-solving prerequisite |
| Application/valuation provider | Domain game, payoffs, empirical assumptions, perspective, welfare weights and evidence design |
| Voiage | Information-value estimands, signed-value arithmetic, study comparisons and research decisions; not generic game solving |

Static Paignia solving must not depend directly or transitively on Kairos or Voiage.
Default Voiage must not depend on Paignia or Kairos. Composition belongs in an
optional adapter or versioned artifact exchange. Runtime callbacks do not justify
a Cargo dependency cycle. The two core/solve names are observed source identities,
not released registry identities. Future adapters/types still need source qualification.

## Historical T17 evidence and migration

Preserve T17 task IDs, source commits, mathematical obligations and test evidence
from the reimbursement repository as provenance. Before provider admission, map
those obligations to Paignia's actual source, package/version and acceptance cases.
A completed legacy task does not establish acceptance of a renamed/extracted crate;
a repository's existence does not establish an installed consumer or solver claim.
Do not invent Paignia task IDs or silently translate historical package names into
released package identities.

The accepted source map preserves seventeen IDs: Paignia owns T17.01-T17.08 and
T17.12-T17.17; reimbursement owns T17.09/T17.11; the selected second application
owns T17.10, currently Innovate G01-G04. Incubator references are historical or
external handoffs, not another provider status ledger.

The two-real-application rule belongs to static package/consumer qualification,
not permission to create the already existing Paignia repository. Reimbursement
plus Innovate remains an eligible pair; CareOps is a candidate, not a mandatory
blocker. A Voiage wrapper alone is not a second real application. Dynamic execution
and unclaimed advanced methods must not become blanket static-release prerequisites.

## Optional strategic-information boundary

First qualify a fixed-policy finite case; then qualify recomputed strategic
responses only for a supported game class and solution concept. Information and
mechanism design remains separate research. Carry game/instance/player/policy IDs,
analyst draw, hidden state/type, permitted signal, solver-run and optional replay
identities; information recipients/timing/commitment; payoff units and welfare
assumptions; solver version, status, residuals, tolerance, catalogue coverage and
selection rule; and source hashes and explicit pairing across counterfactuals.

An analyst's hidden-state draw is not automatically visible to players. Preserve
lawful observations and nonanticipative policies. Multiple equilibria require an
explicit selection policy or range; solver restart frequencies are not behavioural
probabilities. Nonconvergence or an unsupported concept is not economic zero.
Preserve negative information values and distinguish transfers from resource costs.
Do not infer Bayesian equilibrium or Bayes correlated equilibrium from mixed-Nash,
correlated-equilibrium or ontology support.

The Voiage finite signed-social-information evaluator is the proposed numerical
bridge. Its legacy thresholded strategic-score helper is not a general equilibrium
VOI estimator and must retain compatibility until a qualified migration. Paignia
must not copy Voiage's information-value arithmetic.

## Pending acceptance work

- [ ] Map historical T17 obligations to actual Paignia source and qualified artifacts; preserve unsupported-method and incomplete-catalogue dispositions.
- [ ] Freeze a field-level mapping to Voiage's paired-outcome and finite signed-information contracts; no universal schema or public API is introduced by this note.
- [ ] Verify an installed Rust consumer outside the provider checkout and a static dependency closure without Kairos/Voiage.
- [ ] Reuse the eight SI-V01-SI-V08 cases in Voiage PR #1288, including ordinary-VOI reduction, equilibrium selection, information leakage, signed values and explicit failures; advanced cases apply only when those capabilities are claimed.
- [ ] Qualify one reimbursement evidence/pricing pilot, then a distinct Innovate application where appropriate. Forecast fit alone is not causal implementation evidence.
- [ ] Pin accepted artifact hashes and record actual tests before changing any consumer dependency. Preserve a lawful historical source route until migration is accepted; do not maintain two independently evolving solver authorities.

None of this work blocks Voiage's ordinary Q01-Q09 programme, initial software paper,
Innovate's native-runtime acceptance, Kairos' core release or the CareOps ED MVP.
No solver implementation, package rename, provider pin, scientific promotion,
external publication or repository merge occurs in this planning change.

## Existing coordinated proposals

- [Voiage #1288](https://github.com/edithatogo/voiage/pull/1288): SI01-SI04 and the eight verification cases.
- [Reimbursement #103](https://github.com/edithatogo/new-drug-reimbursement-game/pull/103): historical T17 obligations and the application pilot.
- [Innovate #574](https://github.com/edithatogo/innovate/pull/574): adoption/incentive producer boundary.
- [Kairos #383](https://github.com/edithatogo/kairos/pull/383): lawful observations and optional native replay.
- [UOGTO #125](https://github.com/edithatogo/UOGTO/pull/125): semantic mapping.
- [CareOps #207](https://github.com/edithatogo/careops-sim/pull/207): optional operational application.

These links identify existing proposals, not merged or scientifically accepted work.
Keep their historical evidence and scopes. Reconcile owner names with Paignia during
review without resetting implementation tasks or bypassing local/hosted gates.
