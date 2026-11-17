# Policy observation native contract, version 1.0.0

This bounded T17.14 slice constructs an owned decision/action projection for the
existing static perfect-information profile. It binds a request to an explicitly
supplied expected context. A separate host or native adapter establishes which
session, decision and revision are actually current.

The contract builds on the accepted
[static input profile](game-runtime-contract-v1.md) and role-specific identities
in the [native result contract](game-result-contract-v1.md). It adds no scheduler,
live session, native handle, information-set model, dependency or wire format.

## Public native interface

```rust,ignore
pub fn build_policy_observation(
    source: &GameSpecification,
    expected: &PolicyObservationBinding,
    request: &PolicyObservationRequest,
) -> Result<PolicyObservation, PolicyObservationError>;
```

The exact request/projection schema is `POLICY_OBSERVATION_SCHEMA_VERSION`,
currently `"1.0.0"`. The only admitted semantic profile is the exact
`STATIC_GAME_PROFILE_V1` string:

```text
paignia/static-finite-perfect-information-sparse-zero/1.0.0
```

`PolicyObservationBinding` contains these mandatory role-specific values:

| Field | Native type | Meaning |
| --- | --- | --- |
| `specification_id` | `GameSpecificationId` | Exact identity of the supplied source specification |
| `game_instance_id` | `GameInstanceId` | Caller-declared instantiated game |
| `session_id` | `GameSessionId` | Caller-declared external session |
| `player_id` | `PlayerId` | Controlling player of the selected source decision |
| `decision_node_id` | `NodeId` | Selected decision in the validated source graph |
| `revision` | `ObservationRevision(u64)` | Caller-declared observation revision label |

`PolicyObservationRequest` contains `schema_version`, `semantic_profile`, and
that binding. The successful `PolicyObservation` contains only the same three
fields plus `legal_actions: Vec<ActionId>`.

Every output field is owned. There is no `Game`, `Node`, edge target, payoff,
solver trace, native storage handle, callback, borrowed world reference or opaque
payload in the DTO. The player and current declared node are already carried
by the binding. Legal actions retain exactly the source decision's edge order;
the builder does not sort, deduplicate, rename, rank or select them.

Identifiers are nonblank opaque strings. Full IRIs, Unicode, fragments and
nonblank surrounding whitespace survive unchanged; no string is shortened,
parsed into a native handle, trimmed or normalized. Equal strings in distinct
typed roles are permitted without equating those roles. Revision zero and
`u64::MAX` are valid labels. Construction neither increments revisions nor
interprets them as simulation time.

The structs are ordinary public data records. A caller can manually construct
or modify one, which does not acquire a fresh validation or authenticity claim.
Use the builder when a checked projection is required.

## Admission and deterministic failures

The builder performs these checks in order:

1. Require the exact request schema.
2. Call `GameSpecification::validate_for_profile` with the requested exact
   profile. Unknown profiles fail before source-schema or graph checks. For the
   supported profile, validate the complete source, including unobserved branches.
3. Reject blank expected-context identifiers in field declaration order; then
   reject blank request identifiers in that same order.
4. Compare the expected specification identity with `source.id` exactly.
5. Compare request and expected bindings in declaration order, including revision.
6. Resolve the declared node and require a decision, excluding chance and terminal
   nodes explicitly.
7. Require the declared player to equal that decision's controlling player.

Only after all checks pass are the metadata and action IDs cloned into the DTO.
The source, expected context and request remain unchanged on success or failure.
The current node need not be the graph root: the host may declare another
decision owned by its controlling player, with an independently established
runtime context.

| Failure | Stable code | Path |
| --- | --- | --- |
| Unknown request schema | `unsupported_policy_observation_schema` | `schema_version` |
| Unsupported information/profile declaration | `unsupported_semantic_profile` | `semantic_profile` |
| Invalid source specification or graph | Original validation code | Original path prefixed with `source.` |
| Blank expected/request identifier | `empty_observation_identifier` | `expected.<field>` or `binding.<field>` |
| Expected specification differs from source | `observation_source_mismatch` | `expected.specification_id` |
| Request differs from expected context | `observation_binding_mismatch` | `binding.<field>` |
| Declared node does not exist | `unknown_observation_decision` | `binding.decision_node_id` |
| Declared node is chance or terminal | `nondecision_observation_node` | `binding.decision_node_id` |
| Declared player does not control the decision | `wrong_observation_player` | `binding.player_id` |

`PolicyObservationError` retains typed payloads and an error-source chain for
source-validation failures. The path strings identify native contract locations;
they do not establish a serialized interchange format. Construction errors are
host diagnostics, and can include paths from branches excluded from a successful
projection. They are not policy observation records. A host passing observations
to a policy passes the successful DTO, without forwarding its full source or
source-validation diagnostics as an additional observation channel.

## What the binding establishes

The expected binding is a caller declaration, not a live native authority token.
The equality checks detect crossed specification, instance, session, player,
decision or revision inputs relative to that declaration. For example, a request
for revision 16 fails against an expected revision 17 even when all IDs match.
Two matching declarations of revision 16 are admissible; this library has no
independent runtime from which to discover a newer revision.

The host/adapter therefore supplies the actual current node, lifecycle state,
session authority and revision. This builder does not authenticate identities,
track sessions, ensure monotonic revisions, reject stale native generations,
consume an action, advance a clock, inspect causal history or implement replay.
No observed native-currentness or freshness receipt is claimed by equality of
these supplied labels.

The output schema contains exactly the declared public decision metadata and
legal action IDs. Whether those supplied identifiers themselves are lawful
public labels is part of the caller's selected semantic profile and construction.
This builder does not inspect identifier text for concealed domain information.

## Projection invariance and information boundaries

For two valid source graphs with the same source identity and matching expected
context, if the declared node has the same controlling player and ordered action
IDs, the successful DTO is identical. Changing terminal payoffs, an unobserved
chance distribution, another player's decision data, or the selected decision's
edge targets leaves that projection unchanged when the complete graph remains
valid. Changing action order changes the ordered action vector as declared.

This is a bounded data projection property for the existing perfect-information
profile. These tests do not establish hidden-state equivalence, information-set
membership, perfect recall, simultaneous-action lawfulness or a general partial-
observation game contract. Requests naming hidden, information-set, simultaneous
or unknown profiles fail explicitly. Relabelling an externally stripped game as
the supported profile cannot establish that its private information was removed
lawfully; that semantic conversion remains outside this constructor.

Offline solvers may inspect a complete mathematical graph. This function supplies
a separate minimal input record; it gives a policy no access to that full graph.
Applications may continue static offline solving without creating observations
or sessions. T17.17 and the native adapter remain the owners of the additional
information-set and lifecycle behavior they actually advertise.

## Controls and execution receipt

The focused suite is `crates/paignia-core/tests/policy_observation.rs`. Its 32
tests pin the complete DTO shape through exhaustive destructuring and its owned
lifetime through a scope-drop/`'static` control. The metamorphic cases change
unobserved payoffs, chance outcomes, opponent decision data and current edge
targets while requiring complete DTO equality. Separate cases retain original
action order and full opaque IDs, including Unicode and fragments.

Negative controls distinguish all six binding mismatches, each blank identity,
wrong source identity, unknown/chance/terminal nodes, a consistently supplied
wrong controlling player, unknown schemas and unsupported profiles. Source
schema, graph, payoff and action-list failures retain the original source errors.
Repeated failures retain the same code, path and payload without mutating inputs.

Tests-first commit `d3355c0e9c8c7493ed578d037c3c36ff7d8bdaab` executes 32 tests:
zero pass and 32 fail, exit 101, against an explicit unimplemented builder.
The implemented constructor passes all 32 focused tests on Rust 1.75.0 and
stable 1.100.0. The complete workspace at this slice's base executes 164 native
tests under Rust 1.75.0; formatting, Clippy with warnings denied, and the two-
crate/six-profile extraction validator pass. Independent interface review and
the final integration receipt remain separate; no zero-test result is used as
acceptance.

```bash
cargo test --locked -p paignia-core --test policy_observation
```

The slice changes four paths: core module exports, `observation.rs`, the focused
native test, and this contract. Source fixtures, dependencies, runtime integration,
shared Conductor records and consumer pins are unchanged. Rollback is an ordinary
revert of these additive commits. Full native/session and information-set
qualification remain separate from this bounded constructor.
