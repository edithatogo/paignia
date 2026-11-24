# Research software readiness

Use the checks and receipts below when assessing a Paignia revision for
distribution or a software paper.

| Criterion | Paignia evidence | Remaining work |
| --- | --- | --- |
| FAIR4RS findability and access | Public Git history, citation metadata, licence and documented Git installation. The FAIR workflow validates the Citation File Format schema, repository identity and licence at the checked commit, and reports version and DOI gaps. | Archive the first qualified release and record its persistent identifier in the citation metadata. Register it in a research software directory when it has a citable release and a maintainer. |
| FAIR4RS interoperability and reuse | [Pinned packages and application receipts](game-consumer-handoff.md), [native result semantics](../architecture/game-result-contract-v1.md), and an [executed UOGTO information-set contract](../architecture/game-information-contract-v1.md). | Map more provider profiles to full UOGTO IRIs. Qualify source-to-native transformation of the selected fixture's actions, probabilities and payoff identities. |
| JOSS installability and functionality | [Installed artifact tests](game-consumer-handoff.md), API documentation, independent [numerical oracles](../../verification/numerical_oracles/README.md) and two scoped applications. | Prepare a versioned, reproducible example and evidence for claimed performance and research impact. A submission also needs a software paper, state-of-the-field comparison and disclosures under the current review criteria. |
| OpenSSF and native security | The OpenSSF Scorecard workflow retains its check-level report. The Rust policy, Offline AI Security, privacy checks and [private reporting](../../SECURITY.md) address repository-specific risks. | Review real findings on the release candidate. Establish a repeatable provenance and dependency policy for any published packages, then check compatible public API changes against that baseline. |
| rOpenSci and pyOpenSci | Their independent testing, examples and support practices inform the contributor guide. | Apply ecosystem packaging and review tools in a maintained R or Python wrapper repository when a consumer needs one. Paignia ships Rust libraries. |

Use the hosted run summary for each check's result and the exact commits and
archive hashes in the receipts when reproducing results. The FAIR metadata preflight verifies specific evidence.
Archive retrievability and interoperability require their own checks. JOSS, rOpenSci and pyOpenSci review decisions belong to
their respective review processes.

## References

- [FAIR4RS principles](https://doi.org/10.15497/RDA00068)
- [Citation File Format 1.2.0](https://github.com/citation-file-format/citation-file-format/tree/1.2.0)
- [EVERSE FAIR research software guidance](https://everse.software/RSQKit/fair_rs)
- [OpenSSF Open Source Project Security Baseline](https://baseline.openssf.org/)
- [JOSS review checklist](https://joss.readthedocs.io/en/latest/review_checklist.html)
- [rOpenSci software peer review](https://devguide.ropensci.org/softwarereview_policies.html)
- [rOpenSci pkgcheck](https://docs.ropensci.org/pkgcheck/)
- [pyOpenSci package guide](https://www.pyopensci.org/python-package-guide/)
- [OpenSSF Scorecard checks](https://github.com/ossf/scorecard/blob/v5.7.0/docs/checks.md)
- [OpenSSF Best Practices criteria](https://www.bestpractices.dev/en/criteria)
