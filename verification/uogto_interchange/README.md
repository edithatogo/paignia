# Replay a UOGTO game in Paignia

The public [Turtle case](game.ttl) describes a fully observed finite graph with a
buyer, a seller, a chance node and complete terminal payoffs. At the response
node the seller chooses take for 6 rather than pass for 3. The buyer's lottery
returns expected payoffs `(0.25 × 8 + 0.75 × 0, 0.25 × −2 + 0.75 × 6) = (2, 4)`.
The buyer takes that lottery over a guaranteed 1. The
[expected policy](expected-policy.json) is the hand-derived comparison.

From a clean checkout, select Rust 1.75.0 or stable and Paignia's
[CPython 3.15.2 environment](../../README.md#build-and-verify). Run:

```sh
uv venv --seed --python "$(cat .python-version)" target/dev-python
target/dev-python/bin/python -m pip install --require-hashes --only-binary :all: \
  -r verification/uogto_interchange/requirements.txt
target/dev-python/bin/python -I verification/uogto_interchange/verify.py \
  --output target/uogto-interchange
```

The verifier builds the locked Rust example from the clean checkout and
records the executable's SHA-256 and compiler selection. It retrieves two
immutable UOGTO modules and checks their SHA-256 digests. For a disconnected replay, stage those same bytes and pass
`--ontology-dir <directory>`. After fetching the locked Cargo dependencies, use
`CARGO_NET_OFFLINE=true` when running the verifier. The verifier retains the JSON request and result,
N-Triples result, and a receipt with provider and ontology revisions, source
and output hashes, and rejected cases. The package and its provenance fields
are [described in the wire contract](../../crates/paignia-io/README.md).

## Asserted profile

| Data | Mapping |
| --- | --- |
| Graph and ownership | UOGTO `GameSpecification`, `ExtensiveFormRepresentation`, `hasInitialState`, `hasState`, `hasPlayer` and `controlsNode`. A decision must have a declared singleton information set in this profile. |
| Choices and chance | UOGTO `hasAvailableAction`, `hasTransition`, `transitionsFrom`, `transitionsTo` and `probability`. Paignia `position` supplies a contiguous order and `action` connects a decision transition with its available action. |
| Utilities | Paignia `payoffProfile` binds each terminal to a UOGTO `PayoffProfile`. Each `PlayerPayoffLink` retains its IRI and supplies `payoffPlayer` and `payoffValue` for one declared player. The projection requires complete terminal payoffs. |
| Result | UOGTO `GameSolution` and `PlayerPayoffLink` describe the policy and expected utilities. PROV-O relates the policy and run to the specification and the source transition and payoff records. Paignia fields record each contingent choice with its player and state, plus method, status, tolerance and provenance. |

The [pinned UOGTO edition](https://github.com/edithatogo/UOGTO/tree/cc0c84201929996580bb4af98774cf2145b224ae)
provides the canonical classes and property declarations used in the input and
output. The verifier checks those declarations and the selected assertions.
Paignia's mapping terms live in its repository IRI namespace. Their
[upstream review and wider compatibility](../../conductor/tracks/t19_uogto_interchange_20261124/plan.md)
remain tracked.
