# Research software readiness

Use the checks and receipts below when assessing a Paignia revision for
distribution or a software paper.

| Criterion | Paignia evidence | Remaining work |
| --- | --- | --- |
| FAIR4RS findability and access | Public Git history, citation metadata, licence and documented Git installation. The signed 0.1.0 release has a [verified version DOI and archive](receipts/t20-candidate/release-v0.1.0.md). The FAIR workflow checks the citation format and repository identity. | Assess registration in a research software directory. |
| FAIR4RS interoperability and reuse | [Pinned packages and application receipts](game-consumer-handoff.md), [native result semantics](../architecture/game-result-contract-v1.md), the [normative UOGTO profile map and public replay](../../verification/uogto_interchange/README.md), and [T20 static application receipts](receipts/t20-candidate/README.md). | Complete caller-level adoption in [T21](../../conductor/tracks/t21_research_validation_20261125/index.md). Qualify exchange and consumption of discounted games in [T22](../../conductor/tracks/t22_dynamic_game_simulation_20261125/index.md). |
| JOSS installability and functionality | [Installed artifact tests](game-consumer-handoff.md), API documentation, independent [numerical oracles](../../verification/numerical_oracles/README.md) and two scoped applications. T20 records Inara execution for the private draft under review. | Bind the final review edition to exact code and archive hashes, publish the permitted reproducibility evidence and obtain the [independent CODECHECK certificate](paper-reproducibility.md). |
| OpenSSF and native security | The OpenSSF Scorecard workflow retains its check-level report. The [OpenSSF Baseline assessment](security-baseline.md) records the separate Level 1 controls. The Rust policy, Offline AI Security, privacy checks, wire fuzzing, API compatibility checks and [private reporting](../../SECURITY.md) address repository-specific risks. | Resolve material findings on each candidate. Qualify any new packages and entry points under the same dependency, provenance, fuzz and API policy. |
| rOpenSci and pyOpenSci | Their independent testing, examples and support practices inform the contributor guide. | Apply ecosystem packaging and review tools in a maintained R or Python wrapper repository when a consumer needs one. Paignia ships Rust libraries. |

Use the hosted run summary for each check's result and the exact commits and
archive hashes in the receipts when reproducing results. The metadata preflight
verifies citation evidence. Archive retrieval and interoperability have their
own checks. JOSS, rOpenSci and pyOpenSci review decisions belong to their
respective review processes.

## From 0.1.0 to a stable release

The verified 0.1.0 edition establishes a signed, citable starting point for
the bounded methods in its release contract. Its [version DOI and archive
attestations](receipts/t20-candidate/release-v0.1.0.md) identify the shipped
bytes. Treat the version number and the following acceptance evidence as
separate release decisions.

1. **Stabilise the public surface.** Declare the supported Rust API,
   feature flags, compiler and platform range, UOGTO revisions and wire
   profile versions. Publish compatibility and deprecation rules and replay
   older inputs, including rejection cases. Keep experimental methods and
   new semantic profiles explicitly scoped until their contracts settle.
2. **Prove the numerical claims that ship.** Freeze independent oracles,
   admitted inputs, error and stopping criteria, and performance evidence
   for each method and representative application. Reuse the two-host
   Gambit and OpenSpiel evidence and add matched checks for new profiles.
   Record unsupported model families and the conditions under which a
   method exhausts its numerical or workload budget.
3. **Prove installation, operational use and recovery.** Build signed,
   attested archives from the exact release commit. Test clean installs and
   representative downstream calls on the declared targets, check output
   parity and retain a qualified rollback pin. Recheck the version DOI
   against the exact distributed files. A registry or new wrapper needs
   its own installation and support evidence before release.
4. **Maintain security and operability.** Run the locked compiler matrix,
   API checks, bounded fuzzing and dependency, secret, privacy and static
   security checks on the release candidate. Review individual OpenSSF,
   RepoHealth and alint findings, retain the vulnerability reporting route
   and exercise the documented CI runner and release recovery procedures.
   Keep versioned tutorials, change notes and a readable support policy.

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

The candidate at `a04a19da06cc50e310a172574ed66f62000b891a` scored
81/100 on RepoHealth and passed alint with the same two informational notices.
OpenSSF Scorecard increased to 7.4/10, with fuzzing at 10/10. FAIR4RS passed the
metadata preflight and still reports the version tag, citation version and DOI
as open. [Repository Health](https://github.com/edithatogo/paignia/actions/runs/45438493092)
· [Scorecard](https://github.com/edithatogo/paignia/actions/runs/45438493038)
· [FAIR4RS](https://github.com/edithatogo/paignia/actions/runs/45438493053)

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
