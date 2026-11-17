# Neutral static game conversion v1

`convert_static_game(record, mapping, target_profile)` converts the identities
of an explicitly declared native static record. `restore_static_game` reverses
the conversion using its retained complete ledger. Both are in `paignia-core`;
neither takes Kairos types, runtime references, numeric handles or an ontology
parser. The API does not resolve canonical vocabulary or claim that an arbitrary
legacy Kairos/UOGTO model has already been represented without loss.

## Exact supported semantics

The record and identity-map schema are exactly `1.0.0`. Source and target must
both declare
`paignia/static-finite-perfect-information-sparse-zero/1.0.0`. A conversion
cannot use a new profile string to imply an implemented semantic change.

| Declaration | Admitted value |
| --- | --- |
| Information structure | `PerfectInformation` |
| Chance rule | `AllowZeroAndValidateEveryBranch` |
| Payoff rule | `IntentionalSparseZero` |
| Selection | `InputOrderPairwiseToleranceThenLexicographic` |
| Selection tolerance | Explicit finite nonnegative binary64, retained unchanged |
| Game | Existing valid finite acyclic `GameSpecification`, including shared descendants |

The explicit alternative declarations reject. In particular, a strict-positive
chance constructor, complete-payoff semantics, an unspecified/first-maximum
tie rule, simultaneous decisions or information sets cannot silently become
the selected profile. A caller that discards unsupported source semantics
before constructing this record has not obtained a qualified conversion from
that source. This API validates what it receives; it cannot recover discarded
information or attest the external model's interpretation.

## Identity mapping

`StaticGameRecord` contains the specification and a required game-instance ID.
Session, policy-artifact, solver-run and solver-diagnostic-trace IDs are optional
caller-supplied references. No run, session, trace or policy is created. Absence
stays absent; a supplied blank ID rejects. A solver diagnostic trace never
becomes a causal event trace.

`StaticIdentityMap.bindings` is an ordered ledger of explicit
`StaticIdentityBinding { role, source, target }` entries. Supported roles are
specification, instance, session, policy artifact, solver run, solver diagnostic
trace, player, node and action. Every distinct identity present in its role must
have exactly one binding. Extra/unknown bindings, duplicate sources and duplicate
targets within a role reject. The same lexical string in two different roles is
legal and does not equate the two entities. Role changes are never inferred.

All strings are opaque and preserved in full. No trimming, Unicode
normalization, integer parsing, fragment matching, implicit namespace rewriting or
automatic identity allocation occurs. The ledger retains both full source and
target strings, including canonical UOGTO and historical Kairos IRI forms.
These references do not certify that a synthetic name is an ontology term.
Mapping two different namespaces' `#player` to one target is a collision;
mapping only the fragment cannot match the full source identity.

Generation-like opaque references such as `urn:kairos:entity:7:1` and
`urn:kairos:entity:7:2` remain distinct. This proves string mapping behavior,
not live-generation validity. Actual runtime allocation, stale generations,
handle ownership and causal binding remain the separate native adapter's T17.08
acceptance.

## Preserving observable behavior

The converter copies node variants, graph incidence, decision-edge order,
chance-edge order, chance weights, present payoff entries, numeric bits and the
selection tolerance. It does not fill absent payoffs, remove zero-weight
branches, normalize probabilities, reorder alternatives or execute a solver.
Both source and converted specifications pass the existing profile validator.

Lexical action names participate in the declared solver's tie rule. A bijection
alone therefore does not preserve behavior. At every decision, the converter
requires the target action names to have the same strict lexical order as the
source names. A reversal rejects with `selection_order_changed`, even if the
current supplied payoffs happen not to tie. This conservative rule preserves
the declared selection semantics for that decision's alternatives. Stored edge
order remains unchanged, including the legacy algorithm's near-tie order
dependence. Node-map storage order has no decision or chance meaning.

The returned `StaticGameConversion` contains the converted record and the
unmodified original ledger. Restoration reverses every binding and runs the
same admission checks again. Public mutation therefore requires revalidation.
The ledger is a reversible data mapping, not a cryptographic provenance claim;
coordinated mutation of a record and its ledger is not an authenticated history.
Callers retain immutable artifact/source receipts separately.

## Errors and verification

`StaticConversionError` carries a stable `code`, `path` and explanatory `detail`.
The target profile is checked first. Record admission then checks schema,
source profile, information/chance/payoff/selection declarations, tolerance,
source graph and contextual IDs. Mapping admission checks its schema and each
binding in supplied order, then missing identities. Lexical selection checks
follow. Nested graph errors retain their existing code with a specification
path prefix. No error repairs or mutates source values.

The tests-first prototype in `d3fdb8b` executes 29 controls: all fail against the
explicit unimplemented entry points, exit 101. The implementation passes all
29 controls, including full-namespace/role round trips, zero/subnormal numeric
bits, missing/unknown/duplicate/colliding maps, action-order changes, unsupported
semantic declarations, zero-weight invalid branches and revalidation on restore.

```sh
cargo test --locked -p paignia-core --test semantic_conversion
```

Independent interface review accepted source
`aee7d76f915892edf043deb0a858cb81a354f47f`, running all 29 author controls and
six separately written native controls. The byte-identical external harness is
retained at `verification/conversion_review`; it checks all six input orders of
an order-sensitive tie example before and after admitted renaming. Both CI
compiler profiles rerun those controls. The actual core artifact consumer
executes conversion and restoration in every feature profile. The
[integrated receipt](../validation/t17-review-20261117.md) records the exact
accepted provider revision and completes bounded T17.13.

Serialization, RDF/JSON-LD ingestion,
normal-form conversion, information sets, native lifetimes and application
activation are outside this bounded contract.
