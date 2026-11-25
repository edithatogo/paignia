# UOGTO–Paignia interchange profile 1.0.0

This project profile binds an asserted UOGTO graph to a Paignia solver
request and a source-bound result. RDF is the graph data model. Turtle,
N-Triples and JSON-LD are encodings of that graph. The Paignia request is an
application JSON format with its own strict fields. The UOGTO declaration
identifies a compatible source and provider profile. [UOGTO declaration](https://github.com/edithatogo/UOGTO/blob/main/docs/contracts/executable-game-subset-v1.md) ·
[JSON-LD 1.1](https://www.w3.org/TR/json-ld11/)

## Load and save across tools

| Tool | Model loading | Persistence |
| --- | --- | --- |
| Gambit | `read_nfg` loads a finite strategic payoff table and `read_efg` loads a finite game tree with chance and information sets. Gambit also supports AGG, BAGG, GBT and named catalogue models. | NFG and EFG encode the model. Use the format-specific writers and retain exact rational inputs when required. Gambit can materialize compatible OpenSpiel games through its optional adapter. |
| OpenSpiel | `load_game(short_name, parameters)` creates a game registered by the build or by an imported Python module. Some registrations need required parameters or an input game. | `serialize_game_and_state` carries the game description and the serializer's state encoding. The default state encoding records action history. Replay uses the matching game implementation. EFG/NFG export applies to the matching finite subset. |
| UOGTO and Paignia | The source is an asserted RDF graph with the pinned UOGTO and bridge terms. The projection emits the one versioned JSON shape needed by the admitted method. | Retain the asserted source, its SHA-256, the request and result, the executable revision and the mapping version. The checked result-to-RDF projection preserves the source binding. |

[Gambit formats](https://gambitproject.readthedocs.io/en/v16.7.0/formats.html) ·
[OpenSpiel loader](https://openspiel.readthedocs.io/en/latest/api_reference/load_game.html) ·
[OpenSpiel state serialization](https://openspiel.readthedocs.io/en/latest/api_reference/game_serialize_game_and_state.html) ·
[UOGTO external game catalogues](https://github.com/edithatogo/UOGTO/blob/main/docs/contracts/external-game-catalogues-v1.md)

## Namespaces and mappings

The core namespace is `https://w3id.org/uogto/core#`, the project
bridge namespace is `https://github.com/edithatogo/paignia#`, and
provenance uses `http://www.w3.org/ns/prov#`. Preserve source IRIs
exactly, including Unicode, case and fragment. Select the representation
explicitly when the specification carries more than one.

| Source assertions | Wire contract |
| --- | --- |
| UOGTO `GameSpecification`, `hasRepresentation`, `hasInitialState` and `hasState` | `specification`, `source.representation`, `root` and the keyed `nodes`. Keep `instance` and `run` distinct. |
| UOGTO `controlsNode`, `hasTransition`, `transitionsFrom`, `transitionsTo`, `hasAvailableAction` and `probability` with bridge `action` and `position` | Owner and ordered decision edges or chance outcomes. Positions are contiguous integers starting at zero. Preserve edge, action and target identities. |
| Bridge `payoffProfile` and UOGTO `PayoffProfile`, `hasPayoffForPlayer`, `payoffPlayer`, `payoffValue` | Terminal payoff link ID, recipient and finite binary64 value. This RDF projection requires one stated value per declared player. Native graph admission separately supports intentional sparse zero. |
| UOGTO `belongsToInformationSet` and its versioned projector | Use singleton decision sets for static backward induction. For CFR/CFR+ supply the complete perfect-recall partition with owner and members. Preserve common action identities and order throughout each set. |
| UOGTO `hasStrategy`, `StrategyProfile`, `hasPayoff` and bridge `payoffCell` | Two ordered players, each with one to eight strategies, a rectangular table and full recipient order in every cell. Each cell retains both its strategy-profile and payoff-profile ID. |
| Source revision, asserted bytes and the selected representation | `source.ontology_revision`, `source.rdf_sha256` and `source.representation`. The RDF caller verifies provenance against the exact source bytes before solving. |

## Admitted results

- **Static backward induction** uses `paignia/static-finite-perfect-information-sparse-zero/1.0.0` with `solve_static_json`. Its result contains a contingent choice at every decision and the expected payoff by player. Zero-weight branches still undergo admission. Selection uses the documented tolerance and deterministic tie order.
- **CFR and CFR+** use `paignia/static-finite-information-set-perfect-recall/1.0.0` with `solve_profile_json`. Its result contains method, iteration count, average policy, payoffs and NashConv. The training status records completion of the requested budget.
- **Pure and bounded mixed Nash** use `paignia/finite-two-player-normal-form/1.0.0` with `solve_profile_json`. The pure result lists candidate profiles. The mixed result contains checked probabilities and gains from the search of equal-cardinality nonsingular supports. Search status and unresolved supports stay visible.

The [wire reference](../../crates/paignia-io/README.md) defines the message
fields. The source-side [UOGTO JSON Schema](https://github.com/edithatogo/UOGTO/blob/main/validation/executable-game-subset-v1.json)
governs the bounded declaration. Rust's strict deserializer is the wire-shape
authority. Its profile checks validate the graph, information sets, complete
tables and numerical limits. SHACL checks the asserted RDF. A separate
hand-maintained wire schema would duplicate that validation.

## Compatibility and losses

Wire 1.0.0 requires exact profile and version IDs, rejects unknown fields and
limits each request to 1 MiB. Invalid graph references, inconsistent sets,
incorrect recipient order and out-of-profile inputs fail before a result.
RDF entailments, blank-node identity, extra source annotations and original
lexical numeric spellings remain with the source graph. Admitted floating
values and explicitly ordered semantic identities travel in the request.
Unsupported stochastic, strategic or information semantics require their own
profile. Catalogue membership supplies source discovery.

UOGTO's static declaration admits the reviewed `bd880dd` and signed
`a04a19d` provider revisions with the exact package versions. A validator
from an older pinned edition retains its older whitelist. The source binding
and execution receipt identify the exact edition. A new incompatible message
shape or semantics needs a new profile identifier. Keep the old fixtures and
treat an unknown revision as a rejection.

[Replay commands and negative fixtures](../../verification/uogto_interchange/README.md) ·
[Downstream frozen v1 replay](../validation/game-consumer-handoff.md) ·
[T21 comparator receipt](../validation/receipts/t21-core/README.md)
