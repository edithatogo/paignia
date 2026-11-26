# Dapane D2 static decision

Intake [Paignia #110](https://github.com/edithatogo/paignia/issues/110) reuses
Paignia's [native static profile](../../docs/architecture/game-runtime-contract-v1.md)
and `paignia_solve::backward_induction`. The verifier runs Dapane's receiving
bridge at `7b530293dd5575c38c7daa008c0b0e1d51673d82` with a provider built
from both Rust crates at the signed `v0.1.0` revision
`a04a19da06cc50e310a172574ed66f62000b891a`.
[Receiving bridge](https://github.com/edithatogo/dapane/blob/7b530293dd5575c38c7daa008c0b0e1d51673d82/adapters/decision_provider_d2/bridge.py#L643-L687)
[Signed provider source](https://github.com/edithatogo/paignia/tree/v0.1.0)

## Receiving interface

Dapane D2 constructs one finite decision for `planner` at `start`, with
two to 32 actions and one terminal per action. Dapane owns the scenario
and draw identity, the explicit utility transformation and the transfer
checks. It converts the exact rational utilities to binary64 before calling
the provider. The native fixture excludes domestic transfers from utility
and supplies a utility for every terminal.
[Receiving fixture](https://github.com/edithatogo/dapane/blob/7b530293dd5575c38c7daa008c0b0e1d51673d82/adapters/decision_provider_d2/tests/test_bridge.py#L351-L373)

The provider accepts `solve --request FILE --output FILE`. The request is
Dapane's `GameSpecification` blueprint, schema `1.0.0`, with `id` and a
`game` containing `root` and `nodes`. The verifier limits requests to 64 KiB.
It admits the graph with `validate_for_profile` and uses the selected Paignia
solver's existing tolerance and action tie rule. The JSON response contains
`solution.profile`, `solution.source_revision`, `solution.root`,
`solution.selected_action` and the scalar `solution.expected_payoff`. Dapane
checks the executable SHA-256 before invocation and verifies the returned
source, profile, action and utility against the receiving packet.

The [optional Paignia interchange](../../crates/paignia-io/README.md) uses
the same native profile in a different envelope. Its versioned wire requires
a source RDF digest, ontology revision, named graph records and specification,
instance and run identities. The D2 blueprint and scalar response are the
receiving interface for this case.

## UOGTO mapping

The semantic disposition is `reuse_existing`. The mapping follows the
[pinned UOGTO edition](https://github.com/edithatogo/UOGTO/tree/cc0c84201929996580bb4af98774cf2145b224ae)
and the [static RDF example](../uogto_interchange/game.ttl). The prefix
`gt:` expands to `https://w3id.org/uogto/core#`. The prefix `p:` expands
to `https://github.com/edithatogo/paignia#`.

| D2 blueprint | RDF mapping |
| --- | --- |
| Game ID and representation | `gt:GameSpecification` links through `gt:hasRepresentation` to `gt:ExtensiveFormRepresentation`. |
| `planner` and `start` | The specification links to `gt:Player` through `gt:hasPlayer`. `gt:controlsNode` links the player to the `gt:DecisionNode`. The specification links to `start` through `gt:hasInitialState` and enumerates nodes through `gt:hasState`. The decision belongs to a singleton `gt:InformationSet` through `gt:belongsToInformationSet`. |
| Each action and terminal edge | `gt:hasAvailableAction` names the `gt:Action`. `gt:hasTransition`, `gt:transitionsFrom` and `gt:transitionsTo` describe the `gt:Transition` from `start` to its `gt:TerminalNode`. `p:position` and `p:action` preserve the ordering and action identity in the RDF projection. |
| The assessed utility | `p:payoffProfile` links the terminal to `gt:PayoffProfile`. `gt:hasPayoffForPlayer` links it to a `gt:PlayerPayoffLink` with `gt:payoffPlayer` and `gt:payoffValue`. |

## Replay and rollback

Use a clean Dapane checkout at the receiving revision and the project-local
`target/dev-python` interpreter selected by Paignia's `.python-version`.
Run these commands from the Paignia root.

```sh
cargo build --locked --manifest-path verification/dapane_d2/Cargo.toml --target-dir target/dapane-d2
target/dev-python/bin/python -B verification/dapane_d2/verify.py \
  /path/to/pinned/dapane target/dapane-d2/debug/paignia-dapane-d2-replay
```

The original `draw-a` case must select B with utility 20. Increasing A's
utility to 30 must select A and preserve the paired key. A source revision
mismatch and an executable digest mismatch must both fail. Native CI stores
the receipt with the exact revisions, executable and source hashes, and
observed outcomes.

The verification package sets `publish = false`.
Dapane can select a qualified provider at its own integration checkpoint by
pinning the source revision and the digest of the chosen executable. The
rollback is the existing preparation-only `paignia_packet` path, without a
native executable configured.
