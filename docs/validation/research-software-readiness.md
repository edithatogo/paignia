# Research software readiness

Use the checks and receipts below when assessing a Paignia revision for
distribution or a software paper.

| Criterion | Paignia evidence | Remaining work |
| --- | --- | --- |
| FAIR4RS findability and access | Public Git history, citation metadata, licence and documented Git installation. The FAIR workflow validates the Citation File Format schema, repository identity and licence at the checked commit, and reports version and DOI gaps. | Archive the first qualified release and record its persistent identifier in the citation metadata. Register it in a research software directory when it has a citable release and a maintainer. |
| FAIR4RS interoperability and reuse | [Pinned packages and application receipts](game-consumer-handoff.md), [native result semantics](../architecture/game-result-contract-v1.md), the [executed UOGTO information-set contract](../architecture/game-information-contract-v1.md), and a [public RDF-to-result replay](../../verification/uogto_interchange/README.md). | Qualify additional provider profiles and a distinct downstream wire consumer under T19. |
| JOSS installability and functionality | [Installed artifact tests](game-consumer-handoff.md), API documentation, independent [numerical oracles](../../verification/numerical_oracles/README.md) and two scoped applications. | Apply the [JOSS, Inara and CODECHECK procedure](paper-reproducibility.md) to a named draft and frozen release. Bind claims to versioned reproducibility and impact evidence. |
| OpenSSF and native security | The OpenSSF Scorecard workflow retains its check-level report. The [OpenSSF Baseline assessment](security-baseline.md) records the separate Level 1 controls. The Rust policy, Offline AI Security, privacy checks and [private reporting](../../SECURITY.md) address repository-specific risks. | Review real findings on the release candidate. Establish a repeatable provenance and dependency policy for any published packages, then check compatible public API changes against that baseline. |
| rOpenSci and pyOpenSci | Their independent testing, examples and support practices inform the contributor guide. | Apply ecosystem packaging and review tools in a maintained R or Python wrapper repository when a consumer needs one. Paignia ships Rust libraries. |

Use the hosted run summary for each check's result and the exact commits and
archive hashes in the receipts when reproducing results. The metadata preflight
verifies citation evidence. Archive retrieval and interoperability have their
own checks. JOSS, rOpenSci and pyOpenSci review decisions belong to their
respective review processes.

## UOGTO integration

The [upstream conformance fixture](https://github.com/edithatogo/UOGTO/tree/3000c231b2c102e18bf7c3294bec498f00f1976c/tests/fixtures/paignia-information-set-native)
was replayed with CPython 3.15.2 and Rust 1.75.0 on its original provider and
then with Paignia at [f5ec19c](https://github.com/edithatogo/paignia/commit/f5ec19cfeb817f60a9332f22f8daacd37d2fe174).
Both runs returned identical output. They admitted two distinct full-IRI
members in one information set, preserved the controller and native action
order, and rejected invalid ownership, missing membership, reordered actions
and imperfect recall. The second run changed only the provider binding in a
disposable copy. The example's paired Rust fixture supplies the topology and
payoffs. The ontology projection supplies the declarative information structure.

The additional [T19 public case](../../verification/uogto_interchange/README.md)
starts with an asserted fully observed graph. Its checked mapping projects the
topology, payoff links and chance weights, executes the optional Rust
interchange, and writes a provenance-bound RDF result with the selected actions.
It has a separate profile and oracle from the upstream information-set check.

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
