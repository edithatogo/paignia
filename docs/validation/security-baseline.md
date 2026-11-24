# OpenSSF Project Security Baseline

This assessment uses [v2026.08.28](https://baseline.openssf.org/versions/2026-08-28)
and its [Level 1 checklist](https://github.com/ossf/security-baseline/blob/a26a7963f6fd098c1e82829322ec9e1196a4f294/docs/versions/2026-08-28-checklist.md).
The source baseline is Paignia
[`1dd21b8`](https://github.com/edithatogo/paignia/commit/1dd21b8de39a3f762ef6a226d99d6ef7e8130787).
Recheck native settings and release conditions on each candidate, using the
linked evidence for every control.

| Control | Disposition and evidence |
| --- | --- |
| OSPS-AC-01.01 | Open. Repository access and direct collaborator roles are visible. The API did not return account MFA state. Confirm eligible access paths separately. |
| OSPS-AC-02.01 | One direct administrator at assessment. GitHub collaborator invitations and permission assignment are explicit. Recheck if access is delegated. |
| OSPS-AC-03.01 | GitHub's active [main ruleset](https://github.com/edithatogo/paignia/rules/27976609) requires a pull request and checks bound to GitHub Actions. It targets `refs/heads/main` and requires the current base. |
| OSPS-AC-03.02 | The same active ruleset protects main from deletion and non-fast-forward updates. |
| OSPS-BR-01.01 | Workflows reviewed with actionlint and the [Scorecard dangerous-workflow check](https://github.com/edithatogo/paignia/actions/runs/45389701446). Revalidate new paths that consume event metadata. |
| OSPS-BR-01.03 | Read-only workflow defaults, checkout without credential persistence, GitHub-hosted checks for public PRs and a separately qualified [isolated scanner](../../.github/workflows/offline-ai-security.yml). Retest any new CI privilege or event. |
| OSPS-BR-03.01, OSPS-BR-03.02 | The canonical Git and project channels in [citation metadata](../../CITATION.cff) and [installation guidance](../../README.md) use HTTPS. Include the first release archive in this check. |
| OSPS-BR-07.01 | GitHub secret scanning and push protection are enabled. [Privacy and secrets](../../.github/workflows/privacy-secrets.yml) scans accepted revisions in CI. |
| OSPS-DO-01.01, OSPS-DO-02.01 | Release checkpoints. Qualify the versioned [usage](../../README.md) and [reporting](../../CONTRIBUTING.md) guidance with the release archives. |
| OSPS-GV-02.01, OSPS-GV-03.01 | Public [issues](https://github.com/edithatogo/paignia/issues) and [contribution instructions](../../CONTRIBUTING.md). |
| OSPS-LE-02.01, OSPS-LE-03.01 | Full [Apache-2.0 text](../../LICENSE) with package copies and attribution validated by CI. |
| OSPS-LE-02.02, OSPS-LE-03.02 | Release checkpoints. Verify licence and notices in the exact public assets. |
| OSPS-QA-01.01, OSPS-QA-01.02 | Public [source](https://github.com/edithatogo/paignia) and [change history](https://github.com/edithatogo/paignia/commits/main/) at stable URLs. |
| OSPS-QA-02.01 | Cargo manifests and locks record direct dependencies. Verification wheels are separately pinned to hashes. |
| OSPS-QA-04.01 | [Application and partner-codebase inventory](../integrations/downstream-game-intake.md). |
| OSPS-QA-05.01, OSPS-QA-05.02 | Tracked inventory and Scorecard binary-artifact check cover source. Build archives and reports are retained as CI artifacts. |
| OSPS-VM-02.01 | [Security contact and private report route](../../SECURITY.md). GitHub private vulnerability reporting was enabled at assessment. |

GitHub requires Rust 1.75, stable Rust, stable workspace coverage, alint, the
FAIR4RS preflight, community and privacy checks on main. Other configured
checks still contribute separate evidence, including CodeQL, dependency policy
and Offline AI Security. The single-maintainer ruleset has zero required human
approvals.

For the release, assess the signed manifest and build reproducibility controls
and retain their receipts under [T20](../../conductor/tracks/t20_release_maturity_20261124/index.md).
[Scorecard](../../.github/workflows/openssf-scorecard.yml) measures automated
signals from a specific revision. This assessment also tracks controls that
need release artifacts, reviewer evidence or native settings.

## T20 follow-up on the pre-candidate revision

The review at `0389af46db4557494f465db2c584c80080e9f588` found zero open
Dependabot and secret scanning alerts. Five CodeQL alerts (#1, #3, #4, #5
and #6) all flag assertion messages in the correlated-equilibrium tests.
The arguments come from literal synthetic payoff matrices, a fixed set of
action permutations and their certificates. The test paths load no release,
customer or user data. The diagnostic messages provide context for numerical
failures. The five alerts are assessed as false positives on this source
edition. [CodeQL run](https://github.com/edithatogo/paignia/actions/runs/45431956077)

The native diagnostics confirmed active secret scanning, push protection and
Dependabot security updates. The authenticated API still returned no account
MFA state for OSPS-AC-01.01. The [0.1.0 release readback](receipts/t20-candidate/release-v0.1.0.md)
records the signed version tag, signed checksum manifest and public archive
licence and DOI. The candidate workflow attests the package receipt and both
verified archive digests to their main revision.
[Active main ruleset](https://github.com/edithatogo/paignia/rules/27976609)
