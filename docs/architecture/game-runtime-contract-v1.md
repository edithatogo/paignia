# Static game input profile v1 — T17.01.profile

Status: bounded candidate input contract. The complete T17.01 result, identity
and conversion contract remains pending. This document records the one behavior
implemented by the five-file slice in the [T17 plan](../../conductor/tracks/t17_game_runtime_ecosystem_20261107/plan.md),
under the [current ownership map](../integrations/parallel-plan-reconciliation-20261108.md#scoped-ownership-update--15-november-2026).
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
