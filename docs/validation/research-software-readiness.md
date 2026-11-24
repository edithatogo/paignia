# Research software readiness

Use the checks and receipts below when assessing a Paignia revision for
distribution or a software paper.

| Criterion | Paignia evidence | Remaining work |
| --- | --- | --- |
| FAIR4RS findability and access | Public Git history, citation metadata, licence and documented Git installation. The FAIR workflow validates the Citation File Format schema, repository identity and licence at the checked commit, and reports version and DOI gaps. | Archive the first qualified release and record its persistent identifier in the citation metadata. Register it in a research software directory when it has a citable release and a maintainer. |
| FAIR4RS interoperability and reuse | [Pinned packages and application receipts](game-consumer-handoff.md), [native result semantics](../architecture/game-result-contract-v1.md), the [executed UOGTO information-set contract](../architecture/game-information-contract-v1.md), and a [public RDF-to-result replay](../../verification/uogto_interchange/README.md). | Use the [completed T19 downstream replay](../../conductor/tracks/t19_uogto_interchange_20261124/plan.md#phase-2-compatibility-and-ontology-coverage) and requalify both static applications against the T20 archives. |
| JOSS installability and functionality | [Installed artifact tests](game-consumer-handoff.md), API documentation, independent [numerical oracles](../../verification/numerical_oracles/README.md) and two scoped applications. | Apply the [JOSS, Inara and CODECHECK procedure](paper-reproducibility.md) to a named draft and frozen release. Bind claims to versioned reproducibility and impact evidence. |
| OpenSSF and native security | The OpenSSF Scorecard workflow retains its check-level report. The [OpenSSF Baseline assessment](security-baseline.md) records the separate Level 1 controls. The Rust policy, Offline AI Security, privacy checks and [private reporting](../../SECURITY.md) address repository-specific risks. | Review real findings on the release candidate. Establish a repeatable provenance and dependency policy for any published packages, then check compatible public API changes against that baseline. |
| rOpenSci and pyOpenSci | Their independent testing, examples and support practices inform the contributor guide. | Apply ecosystem packaging and review tools in a maintained R or Python wrapper repository when a consumer needs one. Paignia ships Rust libraries. |

Use the hosted run summary for each check's result and the exact commits and
archive hashes in the receipts when reproducing results. The metadata preflight
verifies citation evidence. Archive retrieval and interoperability have their
own checks. JOSS, rOpenSci and pyOpenSci review decisions belong to their
respective review processes.

## T20 hosted baseline

The pre-candidate assessment evaluated Paignia at
`0389af46db4557494f465db2c584c80080e9f588`.

| Check | Observed result | Disposition |
| --- | --- | --- |
| RepoHealth | 81/100 (B+). Tooling, coverage and branch-protection heuristics missed active controls. | Native CI runs Rust tests and retains measured line coverage. GitHub enforces the current main ruleset. The release-tag recommendation stays open until the exact candidate qualifies. Contributor count and source ratios describe the project. |
| alint | Passed with notices about code ownership and a local conduct file. | The maintainer owns pull request review, and the repository uses the account conduct policy linked in the contributor guide. |
| OpenSSF Scorecard | 6.9/10. Fuzzing scored 0 and release signing and packaging were unknown. | T20 adds bounded libFuzzer execution and attests the qualified Cargo archives. Signed release verification belongs to the exact version. Review, contributor diversity and project-age scores reflect the current project. |
| FAIR4RS metadata | Citation schema, repository and licence passed. Tags, citation version and DOI remained pending. | Bind the version and DOI to the published archive after verifying its exact bytes. |

[Repository Health and alint](https://github.com/edithatogo/paignia/actions/runs/45431955624)
· [OpenSSF Scorecard](https://github.com/edithatogo/paignia/actions/runs/45431955574)
· [FAIR4RS preflight](https://github.com/edithatogo/paignia/actions/runs/45431955548)
· [Native CI coverage](https://github.com/edithatogo/paignia/actions/runs/45431955799)

## Release checks

Native CI compares both library APIs with the immutable T19 checkpoint using
cargo-semver-checks 0.51.0 under default and all features. The release job then
runs 20,000 libFuzzer executions over both JSON admission entry points with a
fixed seed, a 16 KiB input limit, a 10-second per-input timeout and a 2 GiB
memory limit. The frozen v1 request seeds the run. The normal MSRV and stable
checks exercise the deterministic request-size regression.
[API and fuzz job](../../.github/workflows/ci.yml)
· [cargo-semver-checks](https://github.com/obi1kenobi/cargo-semver-checks)
· [cargo-fuzz](https://rust-fuzz.github.io/book/cargo-fuzz.html)

On main, the candidate job verifies the Rust 1.75 archive digests against the
same run's package receipt and attests the two archives and receipt. Before
distributing the candidate, verify every subject with GitHub CLI and retain
the verification result with the archived bytes. The source digest is the
full candidate commit.

```sh
gh attestation verify "$artifact" --repo edithatogo/paignia \
  --signer-workflow edithatogo/paignia/.github/workflows/ci.yml \
  --source-ref refs/heads/main --source-digest "$candidate" \
  --deny-self-hosted-runners
```

[GitHub artifact attestations](https://docs.github.com/en/actions/concepts/security/artifact-attestations)

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
