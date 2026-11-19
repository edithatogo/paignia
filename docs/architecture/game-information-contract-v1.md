# Information-set native contract, version 1.0.0

This bounded T17.17 slice admits a complete declared partition of an existing
static game's decision nodes into finite information sets. It adds a separate
validation contract without changing the `Game` representation, adding hidden
state, importing Kairos storage or claiming CFR/CFR+ solving.

The contract layers over the accepted
[static input profile](game-runtime-contract-v1.md). The base graph retains its
finite, acyclic, perfect-information profile. A successful information-set graph
declares which decisions an acting player cannot distinguish; it does not by
itself solve, simulate or observe the game.

## Public native interface

```rust,ignore
pub fn build_information_set_graph(
    source: &GameSpecification,
    declaration: &InformationSetDeclaration,
) -> Result<InformationSetGraph, InformationSetError>;
```

The exact declaration/graph schema is `INFORMATION_SET_SCHEMA_VERSION`, currently
`"1.0.0"`. The only admitted declaration profile is the exact
`INFORMATION_SET_PROFILE_V1` string:

```text
paignia/static-finite-information-set-perfect-recall/1.0.0
```

The source graph itself must pass `STATIC_GAME_PROFILE_V1` before any
information-set admission. The information-set profile is a Paignia
input-validation profile, not a UOGTO vocabulary IRI, solver-result version or
wire format.

`InformationSetDeclaration` contains `schema_version`, `semantic_profile` and an
ordered list of `InformationSetDefinition` records. Each definition supplies:

| Field | Native type | Meaning |
| --- | --- | --- |
| `id` | `InformationSetId` | Opaque, globally unique set identity |
| `player` | `PlayerId` | The single player who owns every member |
| `members` | `Vec<NodeId>` | Decision nodes the player cannot distinguish |

`InformationSetGraph` returns the accepted schema/profile, the exact
`specification_id` of the validated source, a `BTreeMap<InformationSetId,
InformationSet>` and a complete `BTreeMap<NodeId, InformationSetId>`
assignment. Each `InformationSet` carries its `player`, ordered
`legal_actions: Vec<ActionId>` and canonical `members: Vec<NodeId>`.

Every output field is owned. There is no `Game`, graph edge target, payoff,
Kairos component/store handle, scheduler, RNG, callback, borrowed world
reference or opaque payload in the result. `PolicyObservation`,
`ResultEnvelope`, sessions and solver algorithms are separate contracts.

## Declared semantics

The contract admits one bounded profile:

1. Every decision node in the validated source belongs to exactly one declared
   information set. Partial partitions are rejected; chance and terminal nodes
   cannot be members.
2. Each information set is owned by exactly one player, and every member must be
   a decision controlled by that player. Set identities are global and cannot be
   shared across players.
3. All members of a set have exactly the same legal actions in the same declared
   edge order. The order is significant because it indexes a shared policy;
   alternative orders are rejected rather than silently canonicalised.
4. Every member list is nonempty and free of duplicates. Output members are
   sorted by `NodeId`; declared action order is preserved unchanged.
5. Perfect recall is required. All members of a set must share the same
   preceding sequence of that player's own information sets and actions.
   Distinct prior information sets are distinguished even when the chosen action
   identifiers are equal.
6. Shared paths in the source DAG must give every player a unique own history at
   each of that player's decision nodes. A merge that makes the player's own past
   unrecoverable is rejected instead of being resolved by path order.

A player's own history records `(information set, action)` for each earlier
decision that player controlled. It excludes chance outcomes and other players'
actions, which is the standard perfect-recall condition. Nodes in one set may
arise from different chance or opponent branches; that is the declared
indistinguishability the set represents.

The source's zero-probability chance edges remain admissible under the base
profile. Recall is structural: a zero-probability path contributes history in the
same way as a positive-probability path and is not reinterpreted as absent.

Strings are opaque. Full IRIs, Unicode, fragments and nonblank surrounding
whitespace survive unchanged; no identifier is parsed, trimmed, shortened or
converted into a native handle. Equal strings in distinct typed roles remain
distinct roles.

## Admission and deterministic failures

The builder performs these checks in order:

1. Require the exact declaration schema.
2. Require the exact information-set semantic profile.
3. Validate the complete source under `STATIC_GAME_PROFILE_V1`, including
   unobserved branches.
4. Admit declarations in input order: nonblank set identity, duplicate set
   identity, nonblank declared player, nonempty members, then each member's
   nonblank identity, existence, decision kind, declared-player ownership,
   duplicate-within-set membership and duplicate assignment across sets.
5. Require every source decision node to be assigned, in source node order.
6. Require identical ordered legal actions within each set, in set identity
   order.
7. Reject ambiguous own histories at decision nodes, in source node order, then
   reject members whose own histories differ inside a set, in set identity order.

| Failure | Stable code | Path |
| --- | --- | --- |
| Unknown declaration schema | `unsupported_information_set_schema` | `schema_version` |
| Unsupported declaration profile | `unsupported_information_set_profile` | `semantic_profile` |
| Invalid source profile/schema/graph | Original code | Original path prefixed with `source.` |
| Blank set, player or member identifier | `empty_information_set_identifier` | `sets.id`, `sets.player` or `sets.members` |
| Duplicate set identity | `duplicate_information_set` | `sets.<id>.id` |
| Set without members | `empty_information_set` | `sets.<id>.members` |
| Duplicate member inside one set | `duplicate_information_set_member` | `sets.<id>.members.<node>` |
| Unknown member node | `unknown_information_set_member` | `sets.<id>.members.<node>` |
| Chance or terminal member | `nondecision_information_set_member` | `sets.<id>.members.<node>` |
| Declared player does not control a member | `information_set_player_mismatch` | `sets.<id>.members.<node>` |
| Node assigned to two sets | `duplicate_information_set_membership` | `game.nodes.<node>` |
| Decision node left unassigned | `unassigned_information_set_decision` | `game.nodes.<node>` |
| Legal actions differ in identity or order | `information_set_action_mismatch` | `sets.<id>.members.<node>` |
| Members have different own histories | `imperfect_recall` | `sets.<id>.members.<node>` |
| Shared paths leave an own history ambiguous | `ambiguous_recall_history` | `game.nodes.<node>` |

The error retains typed identifiers and an error-source chain for source
validation failures. Path strings identify native contract locations; they are
not a serialized interchange format. Admission never mutates the source or
declaration, and repeated failures return the same error.

## What this contract does not establish

This slice supplies an ECS-independent representation and its admission
controls. It does not implement:

- CFR/CFR+ training or any equilibrium algorithm; T17.07 retains S04.
- Simultaneous-action execution, hidden-state observation, observation
  revisions, session authority or native lifecycle; T17.14 remains the bounded
  policy projection and T17.08 the native adapter.
- UOGTO ontology parsing, ontology labels as implemented semantics, or a wire
  format.
- Claimed legal lawfulness of externally stripped information beyond the
  declared graph, or conversion from an older game profile.
- A complete extensive-form solver or a replacement for `Game`'s existing
  perfect-information validation.

The profile deliberately admits only complete partitions with perfect recall.
Imperfect-recall, partial, simultaneous-action and richer observation profiles
require their own explicit contract and evidence.

## Controls and execution receipt

The focused native suite is `crates/paignia-core/tests/information_sets.rs`. Its
33 tests cover:

- accepted perfect-information singletons, one shared information set, distinct
  remembered information sets, chance merges and consistent DAG merges;
- canonical member ordering, declaration-order invariance, exact source binding
  and owned output isolation;
- opaque Unicode/IRI identifiers and declared action order;
- unsupported schema/profile, invalid source precedence, blank identifiers,
  duplicate set/member/membership records, empty sets, unknown/chance/terminal
  members, player mismatch, unassigned decisions, action identity/order
  mismatches, imperfect recall and ambiguous shared histories;
- deterministic error precedence and repeatability without input mutation;
- a 5,000-decision iterative-chain case.

The tests-first commit `535b1ca` executes 33 tests with zero passes and 33
failures from an explicit unimplemented builder. The implemented module passes
all 33 on stable Rust 1.100.0 at the slice's candidate source revision. Formatting
and workspace Clippy with warnings denied pass at that revision. Hosted Rust
1.75.0 is required before this leaf is recorded as accepted; no MSRV claim is
made from the local stable run.

```bash
cargo test --locked -p paignia-core --test information_sets
```

The slice changes four implementation paths: core module exports, the
`information_sets` module, the focused native test and this contract. It adds no
dependency, shared CI, fixture, solver, consumer pin or native runtime
integration. Rollback is an ordinary revert of these additive commits. T17.07
and the native adapter retain their own scientific and lifecycle evidence.
