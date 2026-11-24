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
native games produce an error. This edition qualifies the reference case and
the request-to-result direction. Additional interchange directions, semantics
and consumers have separate compatibility cases in T19.
