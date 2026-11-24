# Contributing to paignia

Paignia provides reusable game representations, solving methods and diagnostics in Rust. Useful contributions include focused fixes, independently calculated reference cases, documentation and bounded method implementations.

## Before you start

Read [AGENTS.md](AGENTS.md), [the project index](conductor/index.md), and the selected track's specification and plan. Check existing issues and pull requests before starting overlapping work. Keep game execution in Rust, domain models and valuation in consuming applications, and canonical game semantics with UOGTO. Preserve Kairos and Voiage ownership boundaries.

For bugs, provide the affected version or commit, environment, minimal reproduction, expected result and observed result. For a substantial proposal, describe the use case and compatibility impact in an issue before expanding implementation scope. Use existing issue templates when offered.

## Games discovered in downstream applications

Follow the [downstream-game intake contract](docs/integrations/downstream-game-intake.md)
for every new or materially changed game, information structure, solution method
or payoff interpretation. Record both the UOGTO semantic disposition and the
Paignia implementation disposition, including an evidenced reuse decision when
the existing contracts suffice. Use one shared intake ID and link existing
upstream work before creating another request.

Extend UOGTO's definitions, constraints, examples or mappings where needed; add
reusable Rust representations, algorithms and independently checked synthetic
examples in Paignia. Keep empirical inputs, domain institutions, payoff
calibration and valuation with the application. Pin the accepted contracts and
provider revisions, then record a focused consumer check before changing its
advertised support. A pending request is not a supported capability.

Before adding shared infrastructure, assess the existing upstream repositories
in the contract's reuse table. Record the capability, interface/pin, dependency
direction and consumer evidence, or why an existing provider does not fit.
Preserve static solving without a simulation runtime or domain dependency.

The contract names the six current applications, versioned tutorial adoption,
and Innovate's parallel innovation-methods role. Existing mature game methods
migrate only after independent parity and caller checks. VoP/VoH methods belong
in Voiage, with `vop_poc_nz` as their demonstration.

## Development and validation

Use the toolchain profiles in [Native CI](.github/workflows/ci.yml); both stable Rust and the declared minimum Rust version matter. From the repository root, the established native checks are:

```sh
uv venv --python "$(cat .python-version)" target/dev-python
target/dev-python/bin/python --version
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo doc --workspace --no-deps --locked
target/dev-python/bin/python scripts/validate_extraction.py
target/dev-python/bin/python scripts/validate_game_packages.py
```

Keep the committed lockfile and original source attribution. Add regression or reference cases for changed behaviour, including invalid inputs and method limitations. An extracted-source consumer test does not qualify a registry artifact or prove scientific validity. Publishing remains a separate operation.

### Rust coverage

**Native CI** measures `paignia-core` and `paignia-solve` on stable Rust with
all workspace features enabled. The `rust-workspace-coverage` artifact
contains an LCOV report and a per-file summary. To reproduce the summary,
install `cargo-llvm-cov` 0.9.2 and the `llvm-tools` component for stable Rust, then
run `cargo llvm-cov --locked --workspace --all-targets --all-features` from the
repository root. [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)

### Repository lint and health

The **Repository Health** workflow runs on pull requests, pushes to `main`
and merge groups. It can also run on demand from the Actions tab.

- Run `alint check` at the repository root. The workflow reads `.alint.yml`,
  annotates findings and fails on errors. [alint v0.22.1](https://github.com/asamarts/alint/releases/tag/v0.22.1)
- Run `repohealth . --format markdown` in a full Git checkout. The workflow
  publishes its informational score and recommendations in the run summary.
  The report detects test tools, coverage configuration, and build commands
  using checkout heuristics. Native CI compiles the workspace, runs the Cargo
  tests, and publishes measured Rust line coverage. GitHub displays the
  inherited community policies and the effective rules for `main`.
  [Main branch rules](https://github.com/edithatogo/paignia/rules/27976609)
  [RepoHealth v0.5.3](https://github.com/spbuilds/repohealth/releases/tag/v0.5.3)

When upgrading RepoHealth, update its version and Linux amd64 archive SHA-256
together in `.github/workflows/repository-health.yml`.

## Pull requests

Create a focused branch from the current `main` branch and submit a pull request to this repository. Link the relevant issue or existing Conductor task, explain the resulting behaviour, and update affected examples, contracts and documentation. Keep unrelated changes in separate pull requests.

Report executed commands, outcomes and the commit they validate. Identify unavailable checks and remaining limitations explicitly. Follow the repository's existing review and automated-check requirements; retain required gates and current ownership. Address findings and verify the final head before integration. Repository changes do not themselves authorise a release or external operational action.

## Security, rights and conduct

Follow the [security policy](https://github.com/edithatogo/.github/blob/main/SECURITY.md). Use private vulnerability reporting if available; otherwise request a private contact without publishing vulnerability details. Keep credentials, personal information and restricted source material out of issues, fixtures, logs and artifacts.

Read the [existing licence](LICENSE) and preserve attribution and third-party terms. Follow the [code of conduct](https://github.com/edithatogo/.github/blob/main/CODE_OF_CONDUCT.md) and keep technical discussions respectful.

## Basis for this guide

This guide follows [GitHub's contribution-guideline guidance](https://docs.github.com/en/communities/setting-up-your-project-for-healthy-contributions/setting-guidelines-for-repository-contributors) and the contribution-process and testing principles in [OpenSSF Best Practices](https://www.bestpractices.dev/en/criteria). It does not assert certification.
