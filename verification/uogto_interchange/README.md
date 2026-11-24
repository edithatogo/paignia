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
output. The verifier checks those declarations and the selected assertions, including
the original numeric literal spellings before RDFLib normalization.
Paignia's mapping terms live in its repository IRI namespace. Their
[upstream review and wider compatibility](../../conductor/tracks/t19_uogto_interchange_20261124/plan.md)
remain tracked.

## Validate and round-trip asserted RDF

Stage the immutable ontology, SHACL shapes, examples and contexts once. The
inventory and round-trip commands operate on the local hash-verified files.

```sh
target/dev-python/bin/python -I verification/uogto_interchange/interchange.py stage \
  --ontology-dir target/uogto-bundle
target/dev-python/bin/python -I verification/uogto_interchange/interchange.py inventory \
  --ontology-dir target/uogto-bundle
target/dev-python/bin/python -I verification/uogto_interchange/interchange.py roundtrip \
  --ontology-dir target/uogto-bundle --input verification/uogto_interchange/game.ttl \
  --output target/game.nt
```

The command accepts a single Turtle or N-Triples graph. Oxigraph preserves
lexical RDF terms, including Turtle numeric shorthand. RDFLib and PySHACL
check UOGTO declaration roles and the pinned shapes with RDFS inference from
the pinned ontology. The exported N-Triples contain the asserted graph. A bundle hash,
RDF, or validation failure exits nonzero before writing output. Game execution
uses the optional Rust package. The frozen static v1 JSON records came from the
published T19 replay on Paignia `11102e0`.

## Replay term coverage

```sh
target/dev-python/bin/python -I verification/uogto_interchange/coverage.py \
  --ontology-dir target/uogto-bundle --output target/term-coverage.json
```

The required replay uses the pinned upstream examples, the static game and
synthetic graphs in `scenarios/`. A class witness is an instance with
scenario context beyond its type assertion. A property witness records its
actual subject and value. Each witness points to the upstream definition and
a successful SHACL-validated graph round trip. The locked upstream JSON-LD
examples are converted using local contexts before replay. Sources, outputs
and the ontology lock have separate SHA-256 digests in the report.

The full gate requires 322 class witnesses and 212 property witnesses on the
pinned revision. These are vocabulary interoperability results. Numerical
methods have their own profiles and acceptance evidence.
