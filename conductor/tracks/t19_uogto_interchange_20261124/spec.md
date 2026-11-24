# T19 specification

Admit and solve one public finite perfect-information game from asserted UOGTO
RDF. Use UOGTO for canonical game, player, representation, node, action, transition
and payoff vocabulary. Declare the revision and any local mapping terms explicitly.
Project ordered actions, named chance outcomes, payoff links, and source identities
into an opt-in versioned JSON boundary. Execute the existing native solver and
return a result bound to its source, input profile, run identity, method revision
and method status. Retain identities using whole IRIs.

The optional package must preserve native game validation and work with Rust 1.75.
Check malformed, duplicate, version, graph and payoff cases. An external caller
must be able to replay the public example and compare its result with a
hand-derived answer. Keep game execution in Rust. The staged profile covers
backward induction for a finite acyclic graph with perfect information. Expand to
other methods, formats or information-set graphs through separate verified
compatibility cases.

UOGTO owns changes to canonical terms and profile publication. Paignia owns its
projection and executable reproduction case. A fit-for-purpose local bridge term
needs an explicit mapping and a review disposition before any upstream promotion.
