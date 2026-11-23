# Native package handoff to application owners

This guide prepares the evidence for T17.09, T17.10 and the bounded T17.12
eligibility decision. The deliverable is two real application receipts for the
**same qualified package bytes and semantic profile**, supported by executed
application models and independently justified expected results.

The [consumer manifest](game-consumers.json) records the first application
receipt for the reimbursement Chapter 8 case. The eligibility check requires
a second distinct application receipt against the same archives and profile.
The [evidence guard](../../scripts/validate_game_consumer_evidence.py) defines
schema version `1`; the [package qualifier](../../scripts/validate_game_packages.py)
defines how the native archives and package receipt are produced.

## Ownership and the receiving decision

| Owner | Required handoff |
| --- | --- |
| Paignia, T17.03 | Qualify `paignia-core` and `paignia-solve`, retain the exact archives, package receipt, compiler details and profile contract. |
| [Reimbursement application, T17](https://github.com/edithatogo/new-drug-reimbursement-game/tree/main/conductor/tracks/t17_game_runtime_ecosystem_20261107), T17.09 | Execute its own real model against those artifacts, justify the expected result independently and retain the application revision and execution evidence. Its existing `paignia_static_game` source test is a starting point. |
| [Innovate G01–G04](https://github.com/edithatogo/innovate/pull/570), T17.10 | Supply the second distinct real application case against the same artifact and profile hashes. CareOps may supply an alternative real application case under the plan. |
| Reimbursement application, T17.11 | Preserve its payoff units, perspective and valuation provenance, and reject incompatible inputs. HEOR valuation and Voiage remain separate owners. |
| Paignia, T17.12 | Review the received evidence, bind the retained files by hash, run the guard and record its bounded eligibility result. |

These are ownership links from the [T17 plan](../../conductor/tracks/t17_game_runtime_ecosystem_20261107/plan.md),
not assertions that the linked application work is complete. Application owners
retain their game definitions, data, institutions, payoff construction and
empirical assumptions. Each application needs only the algorithm and native
capabilities its case actually uses. Static reimbursement plus Innovate can
qualify without an optional Kairos adapter. The reimbursement valuation port is
not a prerequisite for another application's independently qualified payoff
contract.

Other [ecosystem responsibilities](../../README.md#ecosystem-boundaries) remain
with their owners. Kairos owns native scheduling, execution ports and its
scientific source/caller-retirement records. UOGTO owns canonical ontology
identities, semantic profiles and information-set semantics. Those handoffs stay
scoped to the capabilities that need them; accepting the local static artifacts
does not complete the upstream obligations.

## 1. Produce and retain one provider bundle

Start from a clean, committed provider revision whose required native checks
have passed. The package command rejects uncommitted package inputs; the
receiving guard also rejects a package receipt that records a dirty worktree.
Use the repository-local CPython interpreter selected by `.python-version`,
the selected Rust toolchain and Linux `bubblewrap` as specified in the
[build instructions](../../README.md#build-and-verify).

These commands run actual package qualification. Choose a fresh destination for
each handoff so a previous receipt cannot be confused with a new run:

```bash
set -euo pipefail
test "$(target/dev-python/bin/python --version)" = "Python $(cat .python-version)"
mkdir -p target
PAIGNIA_HANDOFF_DIR="$(mktemp -d "target/consumer-handoff-$(git rev-parse HEAD)-XXXXXX")"
git rev-parse HEAD 'HEAD^{tree}' > "$PAIGNIA_HANDOFF_DIR/provider-identity.txt"
cargo --version > "$PAIGNIA_HANDOFF_DIR/toolchain.txt"
rustc --version >> "$PAIGNIA_HANDOFF_DIR/toolchain.txt"
cargo fetch --locked
target/dev-python/bin/python scripts/validate_game_packages.py \
  --receipt "$PAIGNIA_HANDOFF_DIR/package-receipt.json" \
  --artifact-dir "$PAIGNIA_HANDOFF_DIR/archives" \
  2>&1 | tee "$PAIGNIA_HANDOFF_DIR/package-execution.log"
cp docs/architecture/game-runtime-contract-v1.md \
  "$PAIGNIA_HANDOFF_DIR/profile-contract.md"
(
  cd "$PAIGNIA_HANDOFF_DIR"
  sha256sum archives/*.crate package-receipt.json profile-contract.md \
    package-execution.log provider-identity.txt toolchain.txt > SHA256SUMS
)
```

The script creates actual `.crate` files, verifies packaged source and metadata,
unpacks them, and runs native consumers with the provider checkout unavailable.
It separately executes minimal, default, each advertised feature, and all-feature
profiles. It retains archives only after every profile passes, rechecking each
archive's hash before copying. `--source-smoke` is a separate copied-source check
and cannot produce this artifact handoff.

The package receipt records the provider commit and tree, provider lock hash,
Cargo and Rust versions, both archive versions/hashes, and every profile's
resolved dependencies, lock hash, graph hash and positive native test count.
Archive library locks may be omitted for unpublished siblings, as recorded by
`archive_lockfile_policy`; provider and isolated-consumer locks are still checked.
These are library artifact receipts, not a registry installation claim.

[Native CI](../../.github/workflows/ci.yml) also uploads the actual `.crate` files,
`cargo-artifact-qualification.json`, dependency evidence and native logs in
`native-package-qualification-1.75.0` and
`native-package-qualification-stable`. Its current retention period is 30 days.
Retain the selected run's bundle with the application evidence before that
window expires. Record the run and exact commit; a moving branch URL alone is
not an immutable input.

For the currently selected handoff, start from the
[qualified bundle index](receipts/t17-bd880dd-handoff.json). Use its canonical
`package_receipt` and durable `provider_revision` with `game-consumers.json`.
The downloaded workflow receipt retains GitHub's executed PR merge revision;
preserve it unchanged as hosted provenance. The index records the separately
executed canonical receipt and verified provider-tree/archive-byte equivalence.

**Choose one complete qualification bundle for both applications.** Compiler
profiles may package different bytes even at the same provider commit. Do not
mix archives from one bundle with another bundle's receipt or regenerate each
application's archives independently. The two archive hashes in the package
receipt must equal the hashes computed from the files actually delivered.
The receiving [manifest](game-consumers.json) records the selected
`provider_revision`, `profile_sha256` and both `artifacts` hashes. Use those
populated bindings to coordinate application runs; this reusable guide does not
pin a separate provider version.

The copied profile contract supplies the exact bytes whose SHA-256 becomes
`profile_sha256`. Its admitted profile identity is:

```text
paignia/static-finite-perfect-information-sparse-zero/1.0.0
```

Freeze those bytes with the bundle. A later editorial change to the live
architecture document does not silently update an already qualified profile
hash. The guard requires the retained contract to be nonempty UTF-8 text naming
this exact profile; the receiving owner must also review its actual semantics.

## 2. Execute the application's own model

After transfer, run `sha256sum -c SHA256SUMS` from the received bundle directory.
The application owner checks both delivered archive hashes against the package
receipt. Unpack the qualified archives into a dedicated artifact
location outside the provider checkout, preserving their contents, `LICENSE`
and `NOTICE`. Resolve the application to those unpacked packages. An exact Git
source dependency remains a useful development pin, but it does not establish
that this application executed the qualified archives.

Perform application resolution, compilation and test execution in a clean
environment where the provider checkout is unavailable, or mask it for the
entire run. Retain the isolation setup and evidence with the application log.
This establishes the application artifact boundary; a dependency graph alone
cannot prove that a build script or test did not read provider source. The
receiving owner reviews this evidence because the current application-receipt
schema does not itself reconstruct or enforce the isolation environment.

Cargo's normalized solver archive depends on the core package by version. An
application using local unpacked artifacts therefore needs an exact path/version
for each direct dependency and a workspace-level `paignia-core` patch for that
sibling dependency. The following is an **incomplete configuration template**;
replace every placeholder with the version and directory from the selected
bundle, and select only the features used by the application:

```toml
[dependencies]
paignia-core = { path = "<unpacked-core-directory>", version = "=<core-version>", default-features = false }
paignia-solve = { path = "<unpacked-solver-directory>", version = "=<solver-version>", default-features = false }

[patch.crates-io]
paignia-core = { path = "<unpacked-core-directory>" }
```

Resolve and retain the application's own lockfile. Inspect `cargo metadata` to
verify that the native packages resolve to the selected unpacked archives, with
their exact versions and intended feature configuration. Retain that metadata
and the lock hash with the execution log. The guard does not currently parse an
application lockfile or reconstruct its dependency graph; receiving owners must
review this evidence rather than infer artifact use from the receipt's label.

Run an actual application test target that constructs its real game, calls the
claimed Paignia capability, and asserts both the result and relevant rejection
behavior. An import, a compile-only target or a provider fixture copied into a
second wrapper does not establish an application model. Keep unsupported,
nonconverged, incomplete-search and arithmetic-error results visible; do not
reinterpret them as a valid equilibrium or as proof that no equilibrium exists.

The following command outline is **not execution evidence**. Substitute the
application's actual paths and target, then retain the real commands, output and
exit status. Generate or deliberately update its lockfile before the locked run;
never report a resolution failure or a zero-test target as a pass.

```bash
set -euo pipefail
git rev-parse HEAD 'HEAD^{tree}' > <application-evidence-dir>/application-identity.txt
cargo --version > <application-evidence-dir>/toolchain.txt
rustc --version >> <application-evidence-dir>/toolchain.txt
cargo metadata --format-version 1 --locked \
  --manifest-path <application-Cargo.toml> \
  > <application-evidence-dir>/resolved-graph.json
cargo test --locked --manifest-path <application-Cargo.toml> \
  --test <actual-application-model-target> \
  2>&1 | tee <application-evidence-dir>/model-execution.log
sha256sum <application-Cargo.lock> \
  <application-evidence-dir>/resolved-graph.json \
  <application-evidence-dir>/model-execution.log
```

Commit the model, test and dependency configuration before recording its
immutable application `revision`. Retained evidence must correspond to that
revision and those inputs. Preserve model input hashes or stable references,
payoff units, perspective, algorithm/version, tolerances and feature choices in
the case evidence when they affect the result. These details substantiate the
application claim; they are not additional top-level fields enforced by schema
version `1`.

## 3. Establish an independent expected result

Write the expected-result basis before accepting the production output. Suitable
bases include a small analytic derivation, an independently maintained solver
with a pinned implementation/version, or an independent mathematical review with
retained calculations and controls. State the assumptions, the applicable game
class and the comparison tolerance. Preserve the derivation or external result
and its version with the execution evidence.

For a finite decision game, for example, the basis can derive each player's
continuation payoffs, the chosen actions and the treatment of ties independently
of Paignia. The two application receipts described here assert the admitted
finite perfect-information graph input profile and must substantiate a real
case within that profile. The guard's capability names, including `pure_nash`,
do not admit a normal-form or ODE input representation under that profile.

Separate scientific-method evidence may concern other declared input classes.
For example, an approximate mixed-strategy method case should verify all
profitable pure deviations in original payoff units and retain positive gains
even when admitted by tolerance. An ODE method case needs an appropriate
analytic or independently computed trajectory and error criterion. These
examples describe method qualification, not additional input classes accepted
by the finite-graph application receipt. Application-specific valuation
assumptions also need an application owner; a solver comparison does not
validate those assumptions.

A production output copied into an expected-value file is circular. Compilation,
package smoke tests and structural receipt admission do not supply independent
mathematical evidence. No particular external solver, wire serialization or
additional algorithm is required for a case that already has an adequate,
independent basis within its declared supported scope.

## 4. Supply a content-bound application receipt

Each application supplies a schema-version-`1` JSON receipt and the actual log
referenced by `execution_evidence`. The receiving Paignia owner retains those
files under the Paignia evidence root and records their SHA-256 hashes.

| Receipt field | Required value or meaning |
| --- | --- |
| `schema_version` | Integer `1`. |
| `scope` | Exactly `real-application-artifact-consumer`. |
| `case_kind` | Exactly `application-model`. |
| `repository` | The application's HTTPS GitHub `owner/repository` URL. |
| `revision` | The actual immutable application commit, 40 lowercase hexadecimal characters. |
| `case_name` | A nonblank identifier for the real application case. |
| `expected_result_basis` | A nonblank explanation of the independent expectation and where its retained evidence can be reviewed. |
| `provider_revision` | Exactly the provider commit in the package receipt and receiving manifest. |
| `semantic_profile` | Exactly the admitted profile literal above. |
| `profile_sha256` | Exactly the SHA-256 of the selected, retained profile contract. |
| `artifacts` | Exactly the keys `paignia-core` and `paignia-solve`, each mapped to its qualified archive SHA-256. |
| `result` | `passed`, supported by the referenced execution. |
| `tests_passed` | An integer greater than zero, reflecting the actual case execution. |
| `tests_failed`, `tests_skipped` | Integer `0` for each. |
| `commands` | A nonempty list of the actual executed, nonblank command strings. |
| `execution_evidence` | A content reference containing `path` and `sha256` for the real, nonempty execution log. |

All SHA-256 values are 64 lowercase hexadecimal characters. Every referenced
`path` is relative to the **Paignia repository root**, even when a custom manifest
is passed to the guard. Absolute paths, `..` components, paths escaping through
symlinks, missing files, empty files and hash mismatches reject. JSON receipt
references must resolve to schema-version-`1` objects; execution logs and
independent-provenance files may be nonempty raw text or other retained bytes.
Calculate hashes from the final retained bytes, after any line-ending conversion
or transfer, and preserve them thereafter.

The guard canonicalizes repository case, a trailing slash and a single `.git`
suffix before counting applications. Two differently spelled URLs for the same
repository remain one application. The Paignia repository itself does not count.
Receiving owners must inspect the cases: a declared scope alone cannot establish
that a repository contains a real independent application model.

### Non-acceptance template

This intentionally incomplete JSON is **not a receipt and must fail admission**.
Its placeholders and null test counts must be replaced only from real retained
execution; do not turn it into a passing record by inserting illustrative data.

```json
{
  "schema_version": 1,
  "scope": "real-application-artifact-consumer",
  "case_kind": "application-model",
  "repository": "https://github.com/<owner>/<application>",
  "revision": "<actual-application-commit>",
  "case_name": "<actual-model-case>",
  "expected_result_basis": "<independent derivation or pinned reference and retained location>",
  "provider_revision": "<qualified-provider-commit>",
  "semantic_profile": "paignia/static-finite-perfect-information-sparse-zero/1.0.0",
  "profile_sha256": "<retained-profile-contract-sha256>",
  "artifacts": {
    "paignia-core": "<qualified-core-archive-sha256>",
    "paignia-solve": "<qualified-solver-archive-sha256>"
  },
  "result": "<actual-outcome>",
  "tests_passed": null,
  "tests_failed": null,
  "tests_skipped": null,
  "commands": ["<actual-executed-command>"],
  "execution_evidence": {
    "path": "<retained-log-path-relative-to-Paignia-root>",
    "sha256": "<retained-log-sha256>"
  }
}
```

## 5. Record only the scientific capabilities being qualified

The receiving manifest lists explicit `capabilities` and records all four
scientific obligations below. The guard admits the baseline `backward_induction`
and `pure_nash` capability names without an additional S01–S04 method receipt;
the package and two real application requirements still apply. Advertising an
additional capability requires its corresponding supported method receipt.

| Obligation | Permitted method-receipt capabilities | Owning plan leaf |
| --- | --- | --- |
| S01 | `replicator_rk4` | T17.04 |
| S02 | `mixed_nash` | T17.05 |
| S03 | `correlated_equilibrium` | T17.06 |
| S04 | `cfr`, `cfr_plus` | T17.07 |

Every obligation needs a nonblank `owner` and one of `pending`, `unsupported`,
`supported` or `withdrawn`. A non-supported disposition retains a nonblank
`replacement_plan`. A `supported` disposition needs an `evidence` content
reference to a real method receipt with these fields:

- `schema_version: 1`, `scope: "scientific-method-qualification"` and the exact
  `obligation` key.
- The same `provider_revision`, `semantic_profile` and `profile_sha256` as the
  receiving manifest.
- A nonempty, duplicate-free `capabilities` list drawn only from that obligation's
  permitted names.
- The same execution fields required above: actual commands, a content-bound
  execution log, a positive passing test count, no failures or skips, and
  `result: "passed"`.
- `independent_provenance` with `kind`, `source`, `version` and an `evidence`
  content reference. The permitted kinds are `analytic-reference`,
  `external-implementation` and `independent-review`. The source and version must
  identify the actual independent basis; the provider itself cannot be its own
  external implementation reference.

The top-level profile binds this receipt to the selected handoff context. It
does not turn a population ODE, matrix-game method or information-set method
into the finite graph profile. Retain each method's own game class, algorithm
version, approximation/status semantics and limitations in its qualification
evidence. Implemented source and passing provider tests alone do not justify
changing an obligation to `supported`. Preserve outstanding original
source/caller-retirement or native obligations in the T17 plan; this guard does
not close them.

## 6. Assemble the manifest and run admission

The receiving owner reviews both application cases and the package bundle, then
updates a manifest using the same identity values throughout. Its required
fields are `schema_version`, `provider_revision`, `semantic_profile`,
`profile_sha256`, `profile_contract`, `artifacts`, `package_receipt`,
`consumer_receipts`, `capabilities` and `scientific_obligations`.

`profile_contract`, `package_receipt` and each entry of `consumer_receipts` are
`{"path": "<root-relative-file>", "sha256": "<actual-content-hash>"}` references.
The contract reference's hash must also equal `profile_sha256`. Keep the package
receipt generated by the qualifier intact. The guard requires both native
artifact records and complete, internally consistent feature-profile evidence;
a manually shortened package receipt cannot stand in for the executed matrix.

Run from the Paignia checkout:

```bash
target/dev-python/bin/python -B -m unittest discover -s tests -p test_game_consumer_evidence.py
target/dev-python/bin/python scripts/validate_game_consumer_evidence.py \
  --manifest <received-evidence-manifest.json>
```

The regression suite checks the guard; its synthetic successful cases are not
application receipts. The second command admits the actual referenced evidence:
exit `0` with `eligible: true` means its bounded checks passed; exit `1` reports
ineligibility and explicit errors. Running without `--manifest` reads
`docs/validation/game-consumers.json`, which remains ineligible until real
qualifying evidence is received.

Resolve a provider commit, profile or archive-hash mismatch by selecting a single
qualified bundle and executing the affected application against it. Editing its
receipt to match a different unexecuted bundle does not resolve the mismatch.
Likewise, rerun a failed or skipped case after its cause is fixed; do not edit
counts or substitute provider smoke output. A missing method qualification may
be addressed by supplying that method's actual evidence or by accurately limiting
the declared supported capability list while retaining its outstanding obligation.

The guard validates content bindings and required declarations. It does not
authenticate external repositories, prove mathematical correctness or establish
that a declared application model is adequate. The receiving review supplies
those judgments from the retained evidence. Its report always keeps
`registry_publication: false` and `full_track_complete: false`. A successful
bounded consumer handoff therefore does not publish packages or archive T17;
those decisions still require their own completed obligations and evidence.
