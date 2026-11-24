# T20 implementation plan

## Phase 1 Repository hardening

- [x] Task: Reconcile alint, RepoHealth, OpenSSF Scorecard and FAIR4RS with their hosted main runs and preserve the score baseline.
- [x] Task: Assess the pinned OpenSSF Security Baseline controls against actual repository, CI and source evidence. Keep individual open controls visible.
- [x] Task: Require up-to-date pull requests and seven checks on the [active main ruleset](https://github.com/edithatogo/paignia/rules/27976609). Preserve the isolated offline scanner and its separate receipts.
- [x] Task: Require signature verification for alint and qualify locked release-mode builds for the public interchange replay and the numerical benchmark.
- [ ] Task: Record the hosted scores and resolve remaining verified RepoHealth, alint, Scorecard and [Baseline](../../../docs/validation/security-baseline.md) findings.
- [ ] Task: Add bounded coverage-guided fuzzing at input admission and retain deterministic regressions. Run release API checks against an immutable baseline.

## Phase 2 Research software and paper

- [ ] Task: Link the FAIR4RS evidence to the T19 public reproduction and validate version, installed archives and archive DOI for the selected release.
- [x] Task: Map the [JOSS checklist](../../../docs/validation/paper-reproducibility.md) to repository evidence and qualify Inara using its upstream example.
- [ ] Task: Apply Inara to the selected paper draft. Retain exact draft hashes, output and review findings.
- [x] Task: Document the CODECHECK execution packet with commands, expected values and output hashes.
- [ ] Task: Freeze the draft's computational edition and record independent CODECHECK execution and the resulting certificate.
- [ ] Task: When a maintained R or Python wrapper is admitted, record package installation and behavior against the same provider pin. Apply rOpenSci pkgcheck or pyOpenSci guidance in that wrapper's repository.

## Phase 3 Release qualification

- [ ] Task: Requalify both existing application cases against the same candidate archives. Record exact consuming revisions, feature scope, expected results and archive hashes.
- [ ] Task: Verify signed version tag, signed or attested archive provenance, installability from archived bytes and DOI resolution before closing the release leaf.

## Phase 4 Numerical research

- [ ] Task: Develop independent feasibility checks for higher-dimensional jointly weighted payoff hulls. Compare exact feasible sets and cost on representative and degenerate cases before proposing a broader API or research claim.
