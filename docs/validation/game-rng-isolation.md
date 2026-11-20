# Comparison randomness in native game execution

`NativeGame::start_comparison` selects `paignia/comparison-rng/1.0.0`. It binds
the execution seed, exact game specification and logical game-instance IDs,
scenario ID and paired-draw ID. A comparison shares these inputs across policy
variants. Applications using the static identity converter supply the same
qualified canonical model and node names throughout the comparison.

## Stream addresses

The address encodes each component as its byte length and complete UTF-8 value.
Kairos derives a stream ID from this address in a fixed key namespace and
initializes its ChaCha8 generator from the execution seed and stream ID.
Native entity and session IDs still bind execution and trace identity. Stream
addresses use logical model and comparison IDs so a reordered `World` preserves
the paired draw.

| Stream | Address beyond the comparison context |
|---|---|
| Shared chance | Shared chance purpose, exact chance-node ID, categorical sampler version |
| Independent chance | Independent chance purpose, policy artifact ID, exact chance-node ID, categorical sampler version |
| Policy exploration | Exploration purpose, policy artifact ID, exact decision-node ID, raw draw version |

The application chooses `SharedAcrossPolicies` or `IndependentByPolicy`.
Under either mode a committed chance transition draws once from its logical
node's stream. Paths through a shared chance node use the same draw even when a
detour consumes a different chance stream first. The finite directed acyclic
graph admits at most one visit to each node in a run. Separate logical nodes
receive separate addresses.

The host can issue an owned exploration stream from a fresh decision turn and
give it to its policy along with the lawful observation. The policy controls
draw count and action selection. The interface exposes raw draws, the stream ID
and count while keeping the native generator state private. A second issue at
that turn starts at the same stream position. The simulator draws through the
native integer categorical sampler at the fixed arithmetic grid in the
[execution contract](../architecture/game-native-execution-v1.md).

For comparison evidence, retain the exact model and lock, seed, logical game
instance, scenario, paired draw, coupling mode and policy artifact. The stream
implementation uses the pinned Kairos `DeterministicStream`, ChaCha8,
`rand_chacha` 0.10.0 and `rand` 0.10.3's `WeightedIndex<u128>` for chance.
Distribution provenance values identify `categorical-u128-120/v1` and
`raw-u64/v1`. Policy state and native checkpoint restoration remain in T17.16.

## Distinguishing controls

```bash
cargo test --locked --manifest-path crates/paignia-kairos/Cargo.toml --test rng_isolation
```

The fixture uses real exploration draws to choose between a direct path and a
detour containing an extra chance event. Over 64 seeds it changes exploration
budget, policy artifact and entity construction order. Both paths reach an
eight-outcome common chance node. The paired outcome stays equal, the changed
budget changes at least one realized path, and both streams consume draws. The
independent comparison checks changed policy and paired-draw identities against
the same eight-outcome node.
