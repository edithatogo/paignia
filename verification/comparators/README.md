# Paignia 0.1.0 differential correctness protocol

This protocol covers the signed Paignia 0.1.0 source at
`a04a19da06cc50e310a172574ed66f62000b891a`. Use its locked Rust dependencies,
the two released profile example entry points and the existing
[UOGTO fixture lock](../uogto_interchange/ontology-lock.json).

| Comparator | Source archive | SHA-256 |
| --- | --- | --- |
| Gambit 16.7.0 | [Release tarball](https://github.com/gambitproject/gambit/releases/download/v16.7.0/gambit-16.7.0.tar.gz) | `35a2d7df4f8181cb216ce9a2b10ca820724768d8c69787a608a4a2725a745278` |
| OpenSpiel 2.0.2 | [PyPI source distribution](https://pypi.org/project/open-spiel/2.0.2/#files) | `6f520dfe499b9e5e1e3a31cf77bf071abb02b98104789e7698021922a064e856` |

## Fixtures and pass criteria

1. Compare every pure equilibrium in **all 256 ordered two-by-two games**
   over utilities of 0 and 1, including games with no pure equilibrium. Use
   Gambit's exact `gambit-enumpure` and require equality of sets in declared
   strategy order. Require the RDF to native projection to preserve the
   labelled utilities before constructing the Gambit input. Payoff gaps in
   this family are 0 or 1, so ties agree at the native solver tolerance.
2. Use exact `gambit-enummixed` for Prisoner's Dilemma, matching pennies,
   asymmetric coordination, rock–paper–scissors, a rectangular coordination
   game with a dominated column and the all-zero game. For the five finite
   catalogues compare the complete fixture equilibrium sets, even if a
   support-system diagnostic remains unresolved. The all-zero fixture must
   report unresolved supports and retain exactly the four Gambit vertices.
   Enumerating those vertices does not enumerate the continuum. Every
   Paignia witness must have nonnegative, finite probabilities, unit mass
   and a matching exact witness within `1e-9` absolute. Independently
   recompute both expected payoffs and unilateral gains from the exact
   witness. Require errors and maximum gain at most `1e-9`.
3. Compare the existing public buyer–seller game and a synthetic variant
   with a repeated successor, a zero-probability edge, an off-path
   decision, and a tie. Unfold the native DAG into OpenSpiel's EFG tree.
   Check seats, branches, chance probabilities and terminal payoffs on
   each translated history. Use OpenSpiel to evaluate Paignia's
   deterministic policy in every continuation. Require payoff agreement
   within `1e-9` at the root, and the chosen action to maximize its seat's
   continuation value at every decision, using Paignia's lexicographic tie
   rule within `1e-12`. The original hand result is `(2, 4)`. The variant
   has root payoff `(3, 0)` and selects the seller's lexicographically
   first action off path. The runnable check is a full policy and
   subgame-optimality check using OpenSpiel's evaluation of each child.
4. Run CFR and CFR+ on the existing Kuhn fixture at 100, 1,000 and 12,000
   full iterations. Compare to OpenSpiel's native `kuhn_poker`, checking
   both policies against its independent evaluator. Require matching deal
   probability, legal actions, players, information sets and payoffs on
   every history before importing a policy. Agree on each returned
   Paignia policy's utility and NashConv within `1e-9`. For each trained
   policy require the existing Kuhn value error bound of `0.005`, strictly
   shrinking NashConv at the three budgets, and final NashConv below
   `0.02`. Record OpenSpiel's default update and averaging schedules
   separately from Paignia's. Iteration-level equality is outside this
   comparison.

Retain every request and paired output with hashes. Repeat with the same
tools and compare the output manifest for determinism. Process calls time
out after 30 seconds and any omitted case or discrepancy fails this replay.
This is a correctness protocol for the named games. Comparative timings,
other profiles, and application-specific acceptance have separate gates in
the [T21 evaluation plan](../../docs/validation/evaluation-plan.md).

## Reproduce

Use a clean checkout containing this harness. Use the repository's Python
version, create its local development environment, and install the existing
[hash-locked verification dependencies](../uogto_interchange/requirements.txt).
The two comparators are verification tools outside Paignia's packages.

1. Verify the archive hashes above and extract each source distribution.
   Build Gambit with `./configure --disable-gui` and
   `make GAMBIT_VERSION=16.7.0 gambit-enumpure gambit-enummixed`. The version
   argument supplies the banner string absent from the source tarball.
   Configure OpenSpiel's `open_spiel` source directory with CMake. Set
   `CMAKE_BUILD_TYPE=Release`, `Python3_EXECUTABLE` to the same repository
   interpreter and `CMAKE_LIBRARY_OUTPUT_DIRECTORY` to a fresh directory.
   Build the `pyspiel` target and use that output directory in `PYTHONPATH`.
2. Stage the [pinned UOGTO files](../uogto_interchange/ontology-lock.json)
   using the existing `interchange.py stage` command. Build Paignia's
   shipped examples with locked dependencies and its MSRV:

   ```sh
   CARGO_TARGET_DIR="$PWD/target" rustup run 1.75.0 cargo build --release --locked \
     --manifest-path crates/paignia-io/Cargo.toml --examples
   ```

3. Replay from the repository root. Choose fresh evidence directories for two runs:

   ```sh
   PYTHONPATH="$OPEN_SPIEL_LIBRARY_DIR" target/dev-python/bin/python verification/comparators/compare.py \
     --paignia-bin target/release/examples --gambit-bin "$GAMBIT_BIN_DIR" \
     --ontology-dir target/uogto-bundle --output "$EVIDENCE_DIR"
   ```

   Compare both runs' `receipt.json` files byte for byte. The receipt pins
   the harness commit, five built executables/modules, UOGTO lock and the
   SHA-256 digest of each input and output. Preserve the build log with the
   compiler, runtime and platform versions.

## Timing pilot

After the two correctness replays match, use either verified paired directory
with the same binaries, Python and ontology bundle on Darwin:

```sh
PYTHONPATH="$OPEN_SPIEL_LIBRARY_DIR" target/dev-python/bin/python verification/comparators/benchmark.py \
  --replay "$EVIDENCE_DIR" --paignia-bin target/release/examples \
  --gambit-bin "$GAMBIT_BIN_DIR" --ontology-dir target/uogto-bundle \
  --output target/comparator-timings.json
```

The harness records the first launch and nine subsequent runs per tool and
fixture, randomly interleaving tools with seed 211. Each trial launches a
fresh process with host caches retained. It reports conversion, process elapsed time and peak process RSS.
The expected inputs and outputs must match the correctness manifest on every
trial. Kuhn CFR+ uses the checked JSON and the native OpenSpiel registration. It
compares process elapsed time at 12,000 iterations with the same observed
utilities and NashConv as the corresponding correctness run.
