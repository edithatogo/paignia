# Static input and result metadata components — T17.01

Status: bounded components. The input profile, result identity record and
scalar diagnostic admission have distinct leaves and exact-head review gates.
The complete T17.01 result envelope, identity-to-source binding and conversion
contract remain pending. Their bounded increments are in the
[T17 plan](../../conductor/tracks/t17_game_runtime_ecosystem_20261107/plan.md),
under the [current ownership map](../integrations/parallel-plan-reconciliation-20261108.md#continued-bounded-work--16-november-2026).
It is not a release or downstream conversion receipt.

## Exact admission boundary

`paignia_core::GameSpecification::validate_for_profile(&str)` admits the exact
profile identifier below, then invokes the existing specification validator.

| Field | Admitted value / behavior |
| --- | --- |
| Semantic input profile | `paignia/static-finite-perfect-information-sparse-zero/1.0.0` |
| Exported profile constant | `paignia_core::STATIC_GAME_PROFILE_V1` |
| `GameSpecification.schema_version` | Exactly `1.0.0`, independently of the profile version |
| Representation | Existing `Game`: finite, reachable, acyclic directed graph of decision, chance and terminal nodes; shared descendants are permitted |
| Information structure | Existing fully observed decision-node representation; no information-set or simultaneous-action representation is added |
| Validation order | Profile identity, existing schema/metadata, existing graph validation |
| Unknown profile | `ProfileValidationError::UnsupportedSemanticProfile { value }`; code `unsupported_semantic_profile`, path `semantic_profile` |
| Invalid specification | `ProfileValidationError::InvalidSpecification(ValidationError)`; original code, path and error source retained |

The identifier is Paignia-owned. It is not a UOGTO term or a declaration that
arbitrary ontology input is executable. Matching uses the entire string:
whitespace, case changes, missing versions, future patch/major versions and
other representation names reject. A bare `1.0.0` does not identify a profile.
No version ranges, aliases, normalization or fallback are inferred.

The native API takes the profile alongside the existing specification; it does
not add fields to `GameSpecification` or define a new serialized envelope.
Callers opt into the new boundary explicitly, for example:

```rust
use paignia_core::{GameSpecification, ProfileValidationError, STATIC_GAME_PROFILE_V1};

fn admit_static_game(specification: &GameSpecification) -> Result<(), ProfileValidationError> {
    specification.validate_for_profile(STATIC_GAME_PROFILE_V1)
}
```

A converter must establish that its source actually has the claimed information
structure before constructing this graph. Validation cannot recover hidden
information or simultaneous moves already discarded by an unqualified converter.
Normal-form solving elsewhere in `paignia-solve` is not admitted through this
extensive-form input-profile API. Such profiles need separate contracts.

## Inherited input semantics pinned by this profile

These are the current seed semantics at
[`fca2b2f8811c8cd56d27be4c9407f573d54ad1e9`](https://github.com/edithatogo/paignia/blob/fca2b2f8811c8cd56d27be4c9407f573d54ad1e9/crates/paignia-core/src/lib.rs).
The new API delegates to that validator; it does not silently tighten, normalize
or convert the game.

| Input | Rule and observable boundary |
| --- | --- |
| IDs | Existing nonblank string identifiers. Full strings remain distinct; validation does not trim, shorten, rewrite namespaces or convert to integer/runtime handles. It does not validate RDF vocabularies or claim global identity uniqueness. |
| Decisions | Nonempty action list, unique action IDs within that node, nonblank controlling player, and existing target nodes. |
| Chance weights | Finite, nonnegative values, including zero. The existing sum check accepts mass within `1e-9` of one using the stored iteration order. No normalization is performed. |
| Mass tolerance detail | There is no separate per-edge upper-bound check: a single weight slightly above one can pass when the total stays within tolerance. The profile records this inherited behavior explicitly; a stricter probability policy needs a distinct contract/migration. |
| Zero-weight branches | Retained. Their targets, reachability, cycles and terminal values are still validated; zero weight is not permission to omit a branch. |
| Payoff vectors | Present entries must have nonblank player IDs and finite values. Sparse and empty terminal maps remain legal. Validation does not insert zero entries or infer a complete player roster. |
| Sparse-zero interpretation | The existing extensive-form solver reads an absent player's payoff as zero. Choosing this profile declares that omission intentional. It does not detect an accidentally omitted economic input. A complete-payoff profile remains a separate future contract. |
| Structure | Missing roots/targets, duplicate actions, empty decision/chance nodes, cycles and unreachable nodes retain the existing structured errors. |

The native contract tests use synthetic full-IRI strings from the canonical core,
extension and historical namespace forms. These are identity-preservation cases,
not claims that the fixture names are ontology vocabulary terms. Same-local-name
strings from different namespaces are not mapped together. Actual UOGTO mapping,
duplicate-ID conversion and generational-handle conversion stay in their owning
future contracts.

## Compatibility and result limits

This is an additive opt-in API. Existing `GameSpecification` fields,
`GameSpecification::validate() -> Result<(), ValidationError>`, and every
`ValidationError` variant remain unchanged, including exhaustive enum matches.
The new wrapper preserves nested errors through `std::error::Error::source`.
Legacy validation does not automatically assert the new profile; a caller must
request it. The existing solver signatures, source, dependencies and manifests
are unchanged.

This input profile supplies no new solver-result guarantee. In particular:

- The existing solver uses its configured tolerance and lexical action tie rule;
  this slice does not establish reordered-input invariance for tolerance-near
  ties or freeze a new equilibrium-selection/result contract.
- Sparse vectors are retained in returned values; admission does not promise a
  dense payoff catalogue or finite intermediate/aggregate solver arithmetic.
- Solver diagnostic traversal is not causal execution history. Profile admission
  creates no instance/session/policy/trace identity and no simulation state.
- Algorithm/version, supported solution concept, termination, residuals,
  catalogue completeness, serialization and result-schema admission still need
  the remaining T17.01 work and relevant independent method evidence.

T17.13 conversion source work therefore remains blocked on accepted, pinned
T17.01 semantics. Static qualification and optional dynamic qualification stay
independent. No simulation adapter, RNG/replay integration, external provider pin
or registry publication is enabled by this profile. UOGTO semantics, Kairos
execution, application valuation and Voiage arithmetic retain their owners.

## Executable checks and acceptance

The new `crates/paignia-core/tests/runtime_contract.rs` target pins profile and
schema literals independently of production constants. It checks unsupported
versions/representations, exact matching, validation precedence, unchanged
identities, zero-weight invalid targets, mass tolerance, intentional sparse
payoffs, non-finite rejection and preservation of wrapped errors.

```bash
cargo test --locked -p paignia-core --test runtime_contract
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
python3 scripts/validate_extraction.py
python3 scripts/validate_game_packages.py
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked
```

An independent specialist public-contract review and required hosted stable/Rust
1.75 native checks are merge gates. Author review is not that independent gate.
Actual results, exact revisions and remaining holds belong to the T17 plan and
ownership-map receipt; this command list alone is not evidence of execution.

## Result identity record v1 — T17.01.identities

`ResultIdentityRecord::validate()` admits an explicit **identity component** for
a future result envelope. `RESULT_IDENTITY_SCHEMA_VERSION` is exactly `1.0.0`;
the component's `schema_version` must match the entire literal. Input schema and
semantic-profile admission remain separate. No result payload, algorithm,
solution concept, status, residual or catalogue is admitted by this method.
This new public interface requires its own independent exact-head review.

| Field | Native type | Meaning and presence |
| --- | --- | --- |
| `specification_id` | `GameSpecificationId` | Required specification identity; existing `GameSpecification.id` is unchanged |
| `game_instance_id` | `GameInstanceId` | Required instantiated-model identity; not an array position or specification alias |
| `solver_run_id` | `SolverRunId` | Required solver invocation identity; not a session, policy, uncertainty draw or simulation replicate |
| `session_id` | `Option<GameSessionId>` | Optional caller-supplied game-session context; no session is created |
| `policy_artifact_id` | `Option<PolicyArtifactId>` | Optional policy artifact reference; not live policy state |
| `solver_trace_id` | `Option<SolverDiagnosticTraceId>` | Optional diagnostic traversal reference; never a realized causal play trace |

These are different Rust types, with no implicit cross-role conversion. Their
compile-fail API examples reject specification-to-instance, solver-run-to-session
and policy-to-trace assignment. The role and complete string together identify
the reference. Equal lexical strings in different roles are legal and are not
silently equated; a global uniqueness or entity-equivalence rule would require a
separate namespace/source contract. This component does not resolve any ID,
prove an artifact exists, establish a binding to the supplied game, certify
UOGTO vocabulary or validate a conversion. Explicit tuple construction can
assign a string to a role; callers remain responsible for its truthful meaning.

Validation checks the component version first, then identifiers in the table's
order. A required ID, or an ID supplied inside `Some`, must contain a character
other than Unicode whitespace. `None` is accepted for optional context;
`Some` containing an empty ID rejects. Admission borrows the record and leaves
every string and option unchanged. It does not trim, normalize Unicode, shorten full namespaces,
resolve legacy IRIs, allocate runtime handles or generate an identifier.
Nonblank opaque strings may include surrounding spaces or NUL, as in the
existing string identity boundary; they are not thereby certified as RDF IRIs.

| Failure | Structured variant | Code / path |
| --- | --- | --- |
| Any other version, including future patch versions, whitespace, suffixes or lookalikes | `ResultIdentityValidationError::UnsupportedSchemaVersion { value }` | `unsupported_result_identity_schema` / `schema_version` |
| Blank supplied identity | `ResultIdentityValidationError::EmptyIdentifier { path }` | `empty_result_identity` / the exact field name in the table |

The separate error enum preserves all legacy `ValidationError` variants and
input-profile errors. No field is added to an existing game or solver struct;
no legacy solver automatically creates this record. There is no serialization
format, native adapter or causal trace API in this increment.

The existing `runtime_contract` target adds ten cases for exact schema admission,
unknown-version precedence, each required/optional blank role, stable error
codes/paths, non-mutating string preservation and absent static context. Three
compile-fail doctests distinguish native identity roles:

```bash
cargo test --locked -p paignia-core --test runtime_contract
cargo test --locked -p paignia-core --doc
```

Full T17.01 still needs the versioned result envelope and wire representation,
supported game/information/algorithm/solution semantics, unsupported versus
proved absence versus nonconvergence, finite residual/exploitability diagnostics
and justified tolerances, versioned tie/selection with reorder controls, and
evidence-backed catalogue coverage with multiplicity-preserving serialization.
Source binding and semantic conversion also remain unqualified. **T17.13 source
work remains held**; T18 native comparisons retain their separate information,
method and result prerequisites. Static and optional dynamic acceptance remain
independent. This component grants no capability or downstream activation.

## Result diagnostic record v1 — T17.01.diagnostics

`ResultDiagnosticRecord::validate()` admits one **scalar diagnostic component**.
It checks only the component schema and numeric domains; it does not admit a
complete solver result. `RESULT_DIAGNOSTIC_SCHEMA_VERSION` is exactly `1.0.0`.
This new public component requires fresh independent review at its own head.

| Field | Native type | Declared meaning |
| --- | --- | --- |
| `schema_version` | `String` | Exact whole component-version literal `1.0.0`; no ranges, aliases or fallback |
| `kind` | `ResultDiagnosticKind` | Caller-declared `Residual` or `Exploitability` category; no metric formula or supported algorithm is inferred |
| `value` | `f64` | Required finite, nonnegative diagnostic magnitude |
| `tolerance` | `f64` | Required finite, nonnegative caller-supplied tolerance in the metric's intended units |

The enclosing method contract must define the metric, norm, scale, units,
aggregation, game/solution scope, candidate and source binding, computation and
justification of tolerance. Neither category by itself fixes those meanings.
The type defines no wire tags or serialization format and performs no solver
calculation. A diagnostic undefined for a method belongs outside that method's
result; a zero-valued record must not stand in for missing evidence. The future
enclosing result contract must enforce applicability and presence.

Validation order is exact schema, `value`, then `tolerance`. Each numeric field
is checked for finiteness before negativity. Positive/negative infinity and all
NaNs reject; a negative finite magnitude rejects. Both signs of zero are legal,
including zero tolerance. Positive subnormals and all finite positive values,
through `f64::MAX`, are structurally admitted. No number is rounded, clamped,
normalized, replaced by an absolute value or assigned a default. The record is
borrowed and retains the original strings, kind and binary64 bits on success or
failure, including signed zero and rejected NaN payloads.

| Failure | Structured variant | Code / path |
| --- | --- | --- |
| Unsupported complete version string | `ResultDiagnosticValidationError::UnsupportedSchemaVersion { value }` | `unsupported_result_diagnostic_schema` / `schema_version` |
| First non-finite numeric field | `ResultDiagnosticValidationError::NonFiniteNumber { path }` | `non_finite_result_diagnostic` / `value` or `tolerance` |
| First finite negative numeric field | `ResultDiagnosticValidationError::NegativeNumber { path }` | `negative_result_diagnostic` / `value` or `tolerance` |

The error enum is separate from legacy input and identity errors; its error
source is absent. Schema failure preserves the unsupported string. Field order
is an admission rule, not a memory-layout or wire-order promise. The enum and
record are public native data, so construction alone does not guarantee validity
and mutation requires revalidation.

**Admission does not compare `value` with `tolerance` or determine convergence.**
A finite magnitude exceeding its supplied tolerance is a structurally valid
record. A successful check cannot establish equilibrium, prove absence, select
an algorithm/status, justify a tolerance, certify a metric calculation or close
a scientific-method obligation. In particular, this component does not reuse
the input profile's inherited chance-mass tolerance as a production diagnostic
policy. No solver automatically emits the new record, and no existing solver
result, struct, public error variant or dependency changes.

Eleven added native contract tests cover exact version/schema precedence, both
categories, non-finite and negative fields, deterministic first errors, public
mutation and separate error codes/sources. A 72-case finite boundary matrix
preserves signed zero/subnormal/extreme values bitwise; a magnitude above its
zero tolerance is explicitly admitted without a convergence claim. Existing
input/identity tests and three identity compile-fail doctests remain intact.

The published tests-first no-check prototype actually ran 46 contract tests:
39 passed and seven admission checks failed (exit 101). Its exact revision and
final execution evidence belong to the owning plan and PR receipt. Synthetic
cases are development controls, not numerical qualification.

Full T17.01 envelope/source bindings, game/information/solution/algorithm/status,
method-defined diagnostics and production tolerances, selection, completeness
and multiplicity-preserving serialization remain open. **T17.13 remains held**
until the required full semantics are accepted and pinned. T18 and all
method/package/dependency/actual-consumer prerequisites remain separate. No
adapter, consumer or publication is enabled by this component.
