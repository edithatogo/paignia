# Paignia

Paignia is the Rust library for reusable game representations, supported game
solving methods and solver diagnostics in the Kairos ecosystem. Its independent
native seed was transferred from
[new-drug-reimbursement-game](https://github.com/edithatogo/new-drug-reimbursement-game).
Paignia is now the implementation home; domain applications consume it as a
dependency. Static solving has no Kairos or host-language runtime dependency.

**Status: pre-release native seed.** Source transfer is not scientific or release
qualification. The initial CI tests both stable Rust and the declared Rust 1.75
minimum. Registry publication is disabled in the package manifests; no published
package or qualified general solver suite is claimed.

## Packages and current implementation

| Package | Current implementation | Dependencies |
|---|---|---|
| `paignia-core` | Finite acyclic game graphs, string IDs, chance/decision/terminal nodes, specification metadata and validation. | Rust standard library only. |
| `paignia-solve` | Backward induction for perfect-information games, chance expectations and bounded pure-Nash enumeration. Explicit tolerance, tie policy and diagnostic trace. | `paignia-core` only. |

Information sets, general mixed Nash, general correlated-equilibrium methods,
CFR/CFR+, qualified RK4, learning and native simulation adapters remain planned.
The inherited S01–S04 scientific obligations are open; none is implemented by
copying a legacy method name. `Solution.trace` is a solver diagnostic over
hypothetical alternatives, not a causal simulation event stream.

Known seed behavior remains explicit: chance edges may have probability zero;
extensive-form missing player payoffs currently act as zero; ties use the
configured tolerance and lexical action ordering. T17.01 owns the stronger
versioned result/semantic contract. Do not infer broader support from the names
of UOGTO classes or from this repository's existence.

## Build and verify

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo doc --workspace --no-deps --locked
python3 scripts/validate_extraction.py
python3 scripts/validate_game_packages.py
```

The source-package smoke script copies the package files into a temporary tree
outside this workspace, materializes inherited Cargo metadata/lints, and builds
a separate native consumer using explicit local path-plus-version dependencies
and an offline generated lockfile. Two native tests check the analytic expected
payoff of a chance decision and rejection of an invalid graph. It uses Python
only as development orchestration; all game execution remains Rust. This check
is **extracted-source consumption**, not registry artifact publication, a full
release receipt, or two real application consumers.

Consume an exact reviewed Git revision until release artifacts are qualified.
Keep domain game definitions and payoff construction in the consumer. The
reimbursement application owns its finite offer/acceptance integration and
valuation; Innovate and CareOps own their respective models.

## Ecosystem boundaries

| Owner | Responsibility |
|---|---|
| Paignia | Generic game representation, supported algorithms, result contracts and diagnostics. |
| UOGTO | Canonical game ontology, full IRIs, executable semantic profiles, SHACL and semantic conformance. |
| Kairos | Native time, event scheduling, ECS state, resources, RNG and replay. An optional Paignia adapter depends on both libraries. |
| Domain applications | Specific games, institutions, agents, payoffs, empirical assumptions and decisions. |
| HEOR capability | Valuation with explicit units, perspectives and assumptions; not game solving or scheduling. |
| Voiage | Value of information and decision uncertainty. |

No Kairos solver implementation, UOGTO ontology asset or Python core was copied
in this transfer. The [transfer map](docs/architecture/paignia-transfer-map.md)
records the candidates, retained ownership, namespaces, licences and remaining
conversion/packaging gaps. `extraction-manifest.json` records exact source paths
and revisions. Apache-2.0 attribution is retained in root and package-local
`LICENSE`/`NOTICE` files.

## Maturity work and ownership

[Conductor T17](conductor/tracks/t17_game_runtime_ecosystem_20261107/index.md)
keeps the existing seventeen identifiers. Paignia owns provider leaves
T17.01–T17.08 and T17.12–T17.17. T17.09 and T17.11 are external handoffs to the
reimbursement application; T17.10 is the second application's receipt. These
are ownership links, not duplicated active plans or completed integrations.

A bounded static release requires an explicit contract, isolated dependencies,
qualified package artifacts and two real application cases against the same
profile. Optional simulation and unrelated future algorithms do not gate that
bounded release. Each advertised numerical method must satisfy its own
independent controls. Routine work uses focused checks and coherent agent
review; there is no repeated human approval loop.
