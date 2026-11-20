# Bounded Kairos game execution

`paignia-kairos` implements execution schema `1.0.0` for Paignia's finite
perfect-information sparse-zero profile. The adapter uses Kairos
`bac331fca95420a94db240c02ae66c1991b85da7` and the separate locked Cargo
workspace at `crates/paignia-kairos`. The main static workspace can still build
on Rust 1.75. The adapter requires Rust 1.99 or later.

## Host binding and dispatch

`NativeGame::start` borrows the immutable model and `World`, and exclusively
borrows a scheduler with zero live pending events. The host selects the world,
full generational entity handle, game instance, session, policy artifact and seed.
Kairos checks the entity's current generation in that world. The borrow keeps
the world and entity lifetime fixed while this run is in use. Event and entity
IDs are local to their native scheduler and world.

The host passes the `PolicyObservation` from `observe` to its policy. The
host submits the selected legal action with the issued turn and effective time.
The live run and observation revision identify one action commit slot. A turn
from an earlier binding fails even if the supplied labels and revision match.
The model stays immutable, and the host owns authorization of the selected
policy and any external model or policy artifact references.

Each supported transition requires exactly one tick after the scheduler's
current time, with priority zero. The adapter uses `PGAC` for actions and `PGCH`
for chance. It schedules one event, verifies the actual dispatched event and
commits the new graph node and revision. The record includes the full dispatched
event, exact run labels, old and new revisions and the action or original
ordered chance-edge index. The reached terminal supplies realized payoffs.
Solver traces remain diagnostics over hypothetical branches.

Time, session, revision, legality, cancellation and budget checks precede
native scheduling or a chance draw. Simultaneous and information-set execution,
mixed workloads, delayed effects, resource changes and resume are outside schema
`1.0.0`. A dispatch invariant failure stops the run before updating the game node
or revision.

## Chance and replay scope

Kairos derives a ChaCha8 stream from the supplied seed, full generational entity
ID and purpose `paignia/game-execution/chance/v1`. Sampling uses the locked
`rand` 0.10.3 `WeightedIndex<f64>` through the native RNG. It interprets admitted
chance probabilities as ordered weights, with the sampler's floating-point
resolution. A positive weight that vanishes from the cumulative weights is
rejected. Zero-weight edges are never selected.

The same admitted graph, selected actions, native lifetime and scheduler history
reproduce the causal event records for a fixed seed and lock. Common random
numbers across reordered entity construction, isolation from policy exploration,
and checkpoint/resume have separate T17.15 and T17.16 checks.

## Verify

```bash
cargo test --manifest-path crates/paignia-kairos/Cargo.toml --locked --test native_replay
```

This candidate implements the bounded T17.08 fixture using actual native
dispatch. Kairos Track 114 owns upstream `K01.port` acceptance. Consumer
adoption and provider pin promotion require their own evidence.
