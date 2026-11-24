# Native static result envelope v1

`paignia_core::ResultEnvelope` provides opt-in structural admission for a
caller-constructed static result. Its source is `crates/paignia-core/src/result.rs`,
with public types re-exported from the crate root. Existing game, identity,
diagnostic, solver and error APIs retain their signatures and behavior.

Together with the input profile, identity/diagnostic components and the
[opt-in solver emitter](solver-result-v1.md), this satisfies the bounded native
T17.01 contract. The [17 November acceptance receipt](../validation/t17-review-20261117.md)
pins independent review and integrated compiler evidence. Legacy solver entry
points retain their existing return type. No wire format, general equilibrium
catalogue or application acceptance is implied.

## Admitted profile

| Field | Admitted completed-policy value |
| --- | --- |
| Envelope schema | Exactly `1.0.0` |
| Semantic profile | Exactly `paignia/static-finite-perfect-information-sparse-zero/1.0.0` |
| Game class | `ResultGameClass::FiniteAcyclicExtensive` |
| Information structure | `ResultInformationStructure::PerfectInformation` |
| Solution concept | `ResultSolutionConcept::ToleranceAwareBackwardInductionPolicy` |
| Algorithm | `ResultAlgorithm::BackwardInduction`, native method identifier `paignia/backward-induction` |
| Algorithm version | `1.0.0` or `1.0.1`, independently of envelope and input schema |
| Selection | `ResultSelectionRule::InputOrderPairwiseToleranceThenLexicographic` |
| Catalogue | `CatalogueClaim::OneFound` |
| Payload | Exactly one `StaticPolicyResult` |

Both method versions use the pairwise tolerance comparison in stored edge
order. Version `1.0.1` compensates addition rounding within each chance node.
Version `1.0.0` records remain structurally admissible. Historical execution is
reproduced by pinning the original implementation revision. A candidate action replaces the current best when its
utility is greater outside tolerance, or when its action string is lexically
smaller within tolerance. A chain of near ties can therefore change the selected
action when input order changes. This record does not promise input-order
invariance or an exact subgame-perfect equilibrium at positive tolerance. A new
max-relative selection algorithm requires a distinct method version and an
explicit selection rule; it must not silently reuse this version.

Known enum names for normal-form games, other information structures, Nash or
correlated equilibria, and pure-Nash enumeration remain inadmissible through
this profile. Their presence permits an explicit rejected claim; it does not add
implementation support. `FromStr` for method and solution concept uses exact
literals and rejects unknown strings without normalization.

## Source and identity binding

```rust
use paignia_core::{
    GameInstanceId, GameSpecification, ResultEnvelope, ResultEnvelopeError,
};

fn admit_result(
    result: &ResultEnvelope,
    specification: &GameSpecification,
    expected_instance: &GameInstanceId,
) -> Result<(), ResultEnvelopeError> {
    result.validate_against(specification, expected_instance)
}
```

Admission validates the existing identity component and the supplied
`GameSpecification` under the exact input profile. The complete specification
identity must equal `specification.id`, and the complete instance identity must
equal the caller's explicit expected instance. No identifier is trimmed,
shortened, generated, or equated across typed roles. Optional session, policy
artifact and solver-diagnostic trace identities retain their original component
semantics; no causal trace or live state is constructed.

This is a comparison with the source supplied to the call. It is not a content
hash, proof that an instance exists, historical revision lookup, or evidence
that a mutable specification ID still denotes identical bytes. Callers must
retain immutable source/artifact identity in their execution receipts. Static
conversion and runtime handle binding remain separate T17 obligations.

## Payload and payoff interpretation

A `StaticPolicyResult` has a `BTreeMap<NodeId, ActionId>` and the existing sparse
`PayoffVector`. Every decision node in the supplied graph must have exactly one
choice, and that action must occur at that node. Choices cannot refer to missing,
chance, or terminal nodes. This is a complete contingent policy, including
hypothetical unchosen branches, rather than a recorded causal play path.

Every returned payoff must be finite. Every named payoff player must occur as
a controlling player or terminal-payoff key in the supplied graph. Omitted
payoffs retain the input profile's intentional sparse-zero semantics: admission
does not insert zeros or require a dense vector. A terminal-only graph can have
an empty policy and empty payoff vector.

Admission does not recompute expected values, compare alternatives, check that
the chosen action is optimal, or establish that returned payoffs correspond to
the selected policy. A structurally legal but nonoptimal policy is deliberately
admitted by a regression test, making this boundary explicit. Mathematical
correctness belongs to solver execution and independent method controls.

## Status and catalogue

| Status | Required representation |
| --- | --- |
| `Completed` | The declared routine completed; one structurally valid policy with `OneFound`. This is not a convergence or equilibrium certificate. |
| `Unsupported` | Explicit nonblank reason, `NotApplicable` catalogue, no solutions or computed diagnostics. |
| `NotConverged` | Explicit nonblank reason, `NotApplicable` catalogue, no solutions or computed diagnostics. This records unsuccessful execution without inventing zero payoffs. |
| `NoEquilibrium` | Rejected: the admitted finite perfect-information backward-induction profile does not establish this conclusion. |

`OneFound` describes one returned contingent policy. It must not become a claim
to all equilibria. `EnumeratedWithinDeclaredScope` is rejected for this method
both when completeness evidence is absent and when a caller supplies an opaque
evidence reference. A reference is not a certificate, and this method is not an
enumerator. `Approximate` is also a separate, currently inadmissible catalogue
claim; a tolerance-aware selected policy does not automatically become a
qualified approximate-equilibrium method. Failed execution cannot carry any
of these solution claims.

## Tolerance and optional diagnostics

The explicit envelope tolerance is finite and nonnegative. Signed zero and
positive subnormals are accepted without rewriting their bits. No default is
invented. The caller retains responsibility for a scientifically appropriate
tolerance and its units.

Each optional `DeclaredDiagnostic` has a nonblank, unique `definition_id` and
an existing `ResultDiagnosticRecord`. Component schema, kind, value and
tolerance pass the existing admission rules. Its tolerance must equal the
envelope's numeric tolerance exactly; positive and negative zero compare equal
but remain unchanged. Definition identifiers are preserved as opaque strings,
not resolved or certified as ontology terms.

A caller-defined residual or exploitability name does not establish its norm,
calculation, source or interpretation. The validator does not compare its value
with tolerance or manufacture an absent measurement as zero. A magnitude larger
than tolerance can accompany `Completed`, because that status says the declared
routine completed, not that an equilibrium or convergence criterion was proved.
Method-specific qualification must define and verify the diagnostic before any
stronger result claim is exposed.

## Validation order and errors

Admission checks envelope schema, semantic profile, identity component, source
specification, specification/instance equality, supported typed claims and
algorithm version, then tolerance. It then checks failure consistency or the
completed catalogue and payload. Completed diagnostic definitions/components are
checked before policy references and payoffs. Ordered maps make reference and
payoff error precedence reproducible.

`ResultEnvelopeError` exposes `code`, `path` and `detail`. Codes and field paths
are the stable machine-facing values; explanatory detail is descriptive. Nested
identity, diagnostic and input errors retain their original code with a field
prefix. The old error types and their existing source chains remain unchanged.
No payload is repaired. Public mutation always requires a new admission call.

## Verification and remaining acceptance

The tests-first prototype at
`b4ec045` executed all 31 native contract tests on Rust 1.75: six passed and
25 failed because its validator performed no checks (exit 101). That was an
explicit development prototype, not a prior production defect. The implemented
validator passes all 31 tests with no skipped or filtered cases.

Controls cover source/instance mismatch, unknown versions and concepts, invalid
source graphs, class/information/method mismatches, nonfinite and negative
numbers, signed zero, optional diagnostics, status/payload contradictions,
unearned exhaustive claims, invalid and missing policy choices, and finite
sparse payloads. These are public-contract tests, not independent numerical
validation. The parent integration receipt owns final compiler profiles,
complete workspace checks and independent review.

The opt-in emitter now constructs and admits actual native solver records.
Serialization, broader enumerating-method envelopes and immutable content
attestation are separate extensions, not additional requirements for the
selected native profile. Static conversion and real consumer receipts retain
their own T17 leaves. Undefined method diagnostics remain absent. The generic
core adds no simulation, optimization, serialization or application dependency.
