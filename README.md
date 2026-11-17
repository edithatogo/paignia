# Paignia

Paignia is the Rust library for reusable game representations, supported game
solving methods and solver diagnostics in the Kairos ecosystem. Its independent
native seed was transferred from
[new-drug-reimbursement-game](https://github.com/edithatogo/new-drug-reimbursement-game).
Paignia is now the implementation home; domain applications consume it as a
dependency. Static solving has no Kairos or host-language runtime dependency.

**Status: pre-release native libraries.** The bounded native static contract,
resolved dependency boundary and local Cargo artifact consumption are qualified
on Rust 1.75 and stable. Independent review accepts the bounded four-stage
RK4 and separately named Euler methods. The [current T17 receipt](docs/validation/t17-review-20261117.md)
records exact source, checks and limits. Registry publication remains disabled;
two real application receipts and the remaining track obligations are open.

The initial source transfer is merged in [PR #2](https://github.com/edithatogo/paignia/pull/2)
at `12b22ce7571b9c590405bf53c2c1c5862b49cd64`.
[Native CI for that revision](https://github.com/edithatogo/paignia/actions/runs/42421109771)
passed on stable Rust and Rust 1.75: formatting, Clippy, 15 library tests,
documentation and the two-test extracted-source consumer described below.
That is the historical transfer baseline. The current receipt above records
subsequent contract, package and numerical work without relabelling the old run.

## Packages and current implementation

| Package | Current implementation | Dependencies |
|---|---|---|
| `paignia-core` | Finite acyclic game graphs; exact static-profile admission; typed identity roles; source-bound native result-envelope admission. | `thiserror`; no Kairos, Voiage, application or host-language runtime. |
| `paignia-solve` | Backward induction/chance expectations, bounded pure-Nash enumeration, opt-in versioned result emission, four-stage RK4 and separately named Euler replicator steps. | `paignia-core`, `thiserror`; static execution stays independent of Kairos. |

Information sets, mixed Nash, correlated equilibrium, CFR/CFR+, learning and
native simulation adapters have separate implementation/acceptance work. The
[RK4 contract](docs/validation/replicator-rk4.md) states its smooth-segment,
representable-time and simplex admission limits. Its implementation does not
close the external Kairos caller/retirement part of S01. `Solution.trace` is
solver analysis over hypothetical alternatives, not a causal simulation stream.

Known seed behavior remains explicit: chance edges may have probability zero;
extensive-form missing player payoffs currently act as zero; ties use the
configured pairwise tolerance in stored edge order, followed by lexical action
ordering. Near-tie chains can depend on input order. The [native result contract](docs/architecture/game-result-contract-v1.md)
names that behavior explicitly; it does not promise an exact equilibrium or an
exhaustive catalogue. UOGTO names alone do not establish implementation support.

## Build and verify

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo test --locked -p paignia-core --doc
python3 -B -m unittest discover -s tests
cargo doc --workspace --no-deps --locked
python3 scripts/validate_extraction.py
python3 scripts/validate_game_packages.py
```

The package guard creates actual `.crate` archives, verifies normalized metadata,
source and attribution, then resolves and tests their unpacked contents in
separate consumers. Linux `bubblewrap` makes the provider checkout unavailable
throughout offline resolution, compilation and execution. The receipt records
artifact hashes, compiler versions, the provider lock and each consumer's
resolved dependency graph/lock. Minimal, default, individual advertised features
and all-feature profiles execute separately. Python 3.11+ orchestrates these
checks; all game execution is Rust. Cargo dependencies must be cached first.

Library archive locks are omitted where needed for unpublished sibling packages;
provider and isolated-consumer locks remain checked. This is library artifact
consumption, not a `cargo install --locked` or registry-publication receipt.
`--source-smoke` retains the older two-test copied-source check under its weaker,
explicit scope. Neither check supplies the two required real application cases.

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
