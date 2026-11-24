# Paignia optional interchange

`paignia_io::solve_static_json(&[u8])` reads a versioned request and returns a
completed JSON policy from the native backward-induction solver. The
[public UOGTO replay](../../verification/uogto_interchange/README.md) builds a
request from Turtle, checks the result against a hand-derived policy and
exports the result as RDF. This package has its own lock and compiler profile.
A consumer can opt in by adding a separate `paignia-io` dependency at an
immutable revision.

Wire version `1.0.0` accepts the native
`paignia/static-finite-perfect-information-sparse-zero/1.0.0` profile. The request
supplies a specification, instance and run ID, a root and a list of named nodes.
Decision records provide a controller and ordered actions. Chance records
provide ordered named outcomes and probabilities. Terminal records list named
payoff links, player IDs and finite values. The source record supplies the
ontology revision, RDF SHA-256 and representation identity. The caller checks
those source declarations against the exact source as the replay does.

The response echoes the source binding and identities. It includes the wire
and native result versions, method and version, completed status, tolerance,
contingent choices and expected payoffs. The request and result together retain
the ordered decision and chance identities and the original payoff links. A
request over 1 MiB, unknown fields or versions, duplicate records and invalid
native games produce an error. A named payoff link can be reused across
terminal nodes when its recipient and numeric value match exactly. This edition qualifies the reference case and
the request-to-result direction. Additional interchange directions, semantics
and consumers have separate compatibility cases in T19.

## Perfect recall and finite two-player normal form

`paignia_io::solve_profile_json(&[u8])` accepts a separate version `1.0.0`
request using the static wire's source, specification, instance and run
fields. Its `game` member selects one of these profiles.

| Profile | Game members | Native result |
| --- | --- | --- |
| `paignia/static-finite-information-set-perfect-recall/1.0.0` | `kind: perfect_recall`, topology `root` and `nodes`, native `declaration` with schema version, semantic profile and named sets, `method: cfr` or `cfr_plus`, and an iteration count. | Average policy with ordered action IRIs, expected payoffs, iteration count and NashConv. Status records completion of training. |
| `paignia/finite-two-player-normal-form/1.0.0` | `kind: normal_form`, exactly two ordered player records with `id` and one to eight `strategies`, a rectangular `payoffs` matrix of named `profile` and `payoff_profile` records with both `payoffs` (link ID, recipient and value) in that player order, and `method: pure` or `mixed`. | Pure strategy profiles or verified mixed witnesses with native search status, expected payoffs and maximum deviation gain. |

The response echoes the source binding and carries the method identity,
version and status. The normal-form result echoes the ordered source cells.
The mixed method searches equal-cardinality nonsingular
supports. Its returned status describes that search scope. Read the complete
RDF-to-wire examples and numerical checks in the
[interchange replay](../../verification/uogto_interchange/README.md#replay-the-native-profiles).
