# Paignia

Paignia is the core Rust game-theory library for reusable game representations,
supported solving methods and solver diagnostics. UOGTO is its canonical
ontology; Kairos supplies optional native simulation execution. Its independent
native seed was transferred from
[new-drug-reimbursement-game](https://github.com/edithatogo/new-drug-reimbursement-game).
Paignia is now the implementation home; domain applications consume it as a
dependency. Static solving has no Kairos or host-language runtime dependency.

**Status: pre-release native libraries.** The bounded native static contract,
resolved dependency boundary and local Cargo artifact consumption are qualified
on Rust 1.75 and stable. Independent review accepts bounded RK4/Euler, mixed-Nash
support enumeration, optional correlated equilibrium, static identity conversion
and owned policy observations. The [static package review](docs/validation/t17-review-20261117.md)
and [native provider review](docs/validation/provider-review-20261120.md) record
exact sources, checks and limits. Registry publication remains disabled.
The reimbursement Chapter 8 and Innovate sequential pricing cases satisfy the
two-application receipt requirement for the same qualified artifacts and static
profile. [Consumer evidence](docs/validation/game-consumer-handoff.md) · [Changelog](CHANGELOG.md)

The initial source transfer is merged in [PR #2](https://github.com/edithatogo/paignia/pull/2)
at `12b22ce7571b9c590405bf53c2c1c5862b49cd64`.
[Native CI for that revision](https://github.com/edithatogo/paignia/actions/runs/42421109771)
passed on stable Rust and Rust 1.75: formatting, Clippy, 15 library tests,
documentation and two historical extracted-source consumer tests.
That is the historical transfer baseline. The current receipt above records
subsequent contract, package and numerical work without relabelling the old run.

## Packages and current implementation

| Package | Current implementation | Dependencies |
|---|---|---|
| `paignia-core` | Finite acyclic graphs, static-profile admission, perfect-recall information-set admission, typed identities and result envelopes, static identity conversion, owned policy observations, and paired two-by-two payoff tables. | `thiserror`. |
| `paignia-solve` | Backward induction and chance expectations, bounded pure/mixed Nash, CFR/CFR+, exact binary commitment, two-by-two Nash payoff hull bounds, RK4/Euler, and optional correlated equilibrium. | `paignia-core`, `thiserror`. The explicit `correlated-equilibrium` feature adds Clarabel 0.11.1 and its qualified closure. |
| `paignia-kairos` | Optional qualified adapter for a finite perfect-information path through a Kairos scheduler, with isolated random streams and in-process checkpoint replay. | `paignia-core`, pinned Kairos APIs and `thiserror` in a separate workspace requiring Rust 1.99. |

The [information-set profile](docs/architecture/game-information-contract-v1.md)
and [CFR contract](docs/validation/cfr.md) define the admitted finite graphs and
the independent Kuhn and Leduc controls. UOGTO accepted the executable
information-set profile. [UOGTO acceptance](https://github.com/edithatogo/UOGTO/pull/139)

[Binary commitment](docs/validation/leader-follower.md)
and [Nash payoff hull bounds](docs/validation/credal-nash.md) specify the
supported two-player families, projections and diagnostics.
The adaptive segment method reports exact joint-weight witnesses and exclusions
for two paired payoff vertices.

The [mixed-Nash contract](docs/validation/mixed-nash.md) limits enumeration to
verified finite witnesses from equal-cardinality nonsingular supports, and
explicitly rejects Lemke–Howson. It does not promise a complete catalogue.
Kairos retired the former mixed-strategy exports after the caller migration.
[Kairos S02 retirement](https://github.com/edithatogo/kairos/pull/409)

The [correlated-equilibrium contract](docs/validation/correlated-equilibrium.md)
requires actual primal/dual admission of unchanged raw LP coordinates; it reports
an approximate result and preserves solver termination. The
[RK4 contract](docs/validation/replicator-rk4.md) states its smooth-segment,
representable-time and simplex admission limits. Kairos retired its legacy
public RK4 export and retained experimental trajectory reproduction.
[Kairos S01 retirement](https://github.com/edithatogo/kairos/pull/407)

`Solution.trace` records solver analysis over hypothetical alternatives. Native
execution records the realised causal path separately.

[Static conversion](docs/architecture/game-semantic-conversion-v1.md) preserves
the declared graph semantics and decision selection order. The
[observation DTO](docs/architecture/policy-observation-v1.md) exposes owned
declared context and ordered legal actions. Matching caller-supplied context
establishes consistency; the host owns actual session authority and freshness.

The [native execution contract](docs/architecture/game-native-execution-v1.md)
describes the bounded adapter and its independent build. The K01.port receipt
covers the qualified native path, random-stream isolation and in-process replay.
[Kairos K01.port acceptance](https://github.com/edithatogo/kairos/pull/405)

Backward induction compensates rounding loss when summing chance payouts.
Chance edges may have probability zero;
extensive-form missing player payoffs currently act as zero; ties use the
configured pairwise tolerance in stored edge order, followed by lexical action
ordering. Near-tie chains can depend on input order. The [native result contract](docs/architecture/game-result-contract-v1.md)
names that behavior explicitly; it does not promise an exact equilibrium or an
exhaustive catalogue. UOGTO names alone do not establish implementation support.

## Getting started

### Install the libraries

Create a binary package with `cargo new --edition 2021 paignia-example`. Add both
libraries to its `Cargo.toml` using the same reviewed Git revision. The example
uses the revision in the static package handoff. [Package handoff](docs/validation/receipts/t17-bd880dd-handoff.json)

```toml
[dependencies]
paignia-core = { git = "https://github.com/edithatogo/paignia.git", rev = "bd880dd756d2ba56cc5f479bc078cea31b53c1d1" }
paignia-solve = { git = "https://github.com/edithatogo/paignia.git", rev = "bd880dd756d2ba56cc5f479bc078cea31b53c1d1" }
```

### Example

A single player chooses between two terminal outcomes with payoffs of 2 and 0.
Put this in `src/main.rs` and run `cargo run`. The solver prints `accept: 2`.

```rust
use std::collections::BTreeMap;

use paignia_core::{ActionEdge, ActionId, Game, Node, NodeId, PlayerId};
use paignia_solve::{backward_induction, SolveError};

fn main() -> Result<(), SolveError> {
    let player = PlayerId("chooser".into());
    let root = NodeId("root".into());
    let accept = NodeId("accept".into());
    let decline = NodeId("decline".into());
    let terminal = |payoff| Node::Terminal {
        payoffs: BTreeMap::from([(player.clone(), payoff)]),
    };

    let decision = Node::Decision {
        player: player.clone(),
        edges: vec![
            ActionEdge {
                action: ActionId("accept".into()),
                target: accept.clone(),
            },
            ActionEdge {
                action: ActionId("decline".into()),
                target: decline.clone(),
            },
        ],
    };
    let game = Game {
        root: root.clone(),
        nodes: BTreeMap::from([
            (root.clone(), decision),
            (accept, terminal(2.0)),
            (decline, terminal(0.0)),
        ]),
    };
    let solution = backward_induction(&game)?;
    println!(
        "{}: {}",
        solution.choices[&root].0, solution.expected_payoffs[&player]
    );
    Ok(())
}
```

## Build and verify

Use the CPython patch release recorded in `.python-version` for verification.
Create the repository-local environment once and use it for subsequent checks.

```bash
uv venv --python "$(cat .python-version)" target/dev-python
target/dev-python/bin/python --version
cargo fetch --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo test --workspace --all-targets --all-features --locked
cargo test --locked -p paignia-core --doc
cargo test --locked -p paignia-solve --doc --all-features
target/dev-python/bin/python -B -m unittest discover -s tests
cargo doc --workspace --no-deps --all-features --locked
target/dev-python/bin/python scripts/validate_extraction.py
target/dev-python/bin/python scripts/validate_game_packages.py
```

The package guard creates actual `.crate` archives, verifies normalized metadata,
source and attribution, then resolves and tests their unpacked contents in
separate consumers. Linux `bubblewrap` makes the provider checkout unavailable
throughout offline resolution, compilation and execution. The receipt records
artifact hashes, compiler versions, the provider lock and each consumer's
resolved dependency graph/lock. Minimal, default, individual advertised features
and all-feature profiles execute separately. The pinned CPython interpreter orchestrates these
checks; all game execution is Rust. `cargo fetch --locked` caches the full
metadata closure, including platform-specific packages that a native build
does not download. Boundary inspection and artifact consumers then run offline.

Library archive locks are omitted where needed for unpublished sibling packages;
provider and isolated-consumer locks remain checked. This is library artifact
consumption, not a `cargo install --locked` or registry-publication receipt.
The packed-artifact tests include the earlier copied-source assertions. The two
required real application cases have their own receipts.

Use `--artifact-dir target/qualified-packages` with `--receipt` to retain the
exact archives after every profile passes. Hashes are rechecked before any
archive is copied. CI retains both archives alongside the receipt and native
logs in its compiler-specific qualification artifact, so application owners
can test those bytes directly against their own models and expected outcomes.
The [application-owner handoff](docs/validation/game-consumer-handoff.md) gives
the required receipt fields and reproducible qualification procedure.

Keep domain game definitions and payoff construction in the consumer. The
reimbursement application owns its finite offer/acceptance integration and
valuation; the other applications and partner libraries own their models.

## Ecosystem boundaries

New or materially changed games in downstream repositories feed back into
[UOGTO semantic coverage and Paignia's reusable capabilities](docs/integrations/downstream-game-intake.md).
The intake records existing support, missing semantics or methods, synthetic
examples and eventual consumer adoption. An application-specific game can reuse
an existing family without requiring a new ontology class or solver.

The current applications are `nhra_game`, `gtpcnz`,
`new-drug-reimbursement-game`, `ginsim`, `tobac` and `tutorial_health_gt`.
The linked contract names their repositories and versioned adoption workflow.
Innovate is a parallel innovation library that consumes Paignia for generic
game methods; mature implementations migrate incrementally after parity checks.

| Owner | Responsibility |
|---|---|
| Paignia | Generic game representation, supported algorithms, result contracts and diagnostics. |
| UOGTO | Canonical game ontology, full IRIs, executable semantic profiles, SHACL and semantic conformance. |
| Kairos | Native time, event scheduling, ECS state, resources, RNG and replay. An optional Paignia adapter depends on both libraries. |
| Domain applications | Specific games, institutions, agents, payoffs, empirical assumptions and decisions. |
| Dapane | Costing and valuation with explicit units, perspectives and mappings into game utility. |
| Voiage | Value of information, perspective and heterogeneity; reuse through optional composition or versioned outcomes. |
| Innovate | Innovation-specific diffusion, adoption and substitution; generic game capabilities use Paignia as they are qualified. |

No Kairos solver implementation, UOGTO ontology asset or Python core was copied
in this transfer. The [transfer map](docs/architecture/paignia-transfer-map.md)
records the candidates, retained ownership, namespaces, licences and remaining
conversion/packaging gaps. `extraction-manifest.json` records exact source paths
and revisions. Apache-2.0 attribution is retained in root and package-local
`LICENSE`/`NOTICE` files.

## Maturity work and ownership

[Conductor T17](conductor/archive/t17_game_runtime_ecosystem_20261107/index.md)
records all seventeen accepted leaves and their external application receipts.
[T18](conductor/archive/t18_illicit_enforcement_inspection_games_20261110/index.md)
records the accepted provider methods and the application-owned T18.03 handoff.

[T19](conductor/tracks/t19_uogto_interchange_20261124/index.md) qualifies optional
[`paignia-io`](crates/paignia-io/README.md) with a
[public UOGTO replay](verification/uogto_interchange/README.md). Its separate
Cargo workspace keeps serialization outside the static packages' dependency
graphs. [T20](conductor/tracks/t20_release_maturity_20261124/index.md) owns release
qualification, hardening, research software evidence and the paper workflow.

A bounded static release requires an explicit contract, isolated dependencies,
qualified package artifacts and two real application cases against the same
archive set. Each advertised numerical method has its own independent controls.

## Security

Follow [Paignia's security policy](SECURITY.md) to report vulnerabilities
privately. Keep credentials and sensitive data out of public issues and pull requests.
The [OpenSSF Baseline assessment](docs/validation/security-baseline.md) links
the native main ruleset and the remaining evidence for release.

## License

See [LICENSE](LICENSE) for this repository's licensing terms. Retain the
licences and attribution that accompany third-party material.
