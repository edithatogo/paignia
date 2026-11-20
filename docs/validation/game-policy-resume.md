# In-process game and policy replay

`NativeGame::checkpoint` captures a committed transition into a chance node.
This boundary carries the host's owned versioned policy bytes and the current
game node, revision and transition limit. The checkpoint borrows the original
immutable specification and world, retaining the actual entity generation
through capture and restore. The host codec governs the meaning of the policy
bytes and binds them to the exact policy artifact.

`ReplayCheckpoint::resume` checks checkpoint version, policy artifact and codec
identity, and requires a fresh Kairos scheduler. It restores the scheduler via
Kairos's opaque native snapshot and decodes the policy state before installing
the scheduler. Each resume creates a new trace identity and an independent
native branch. The host decoder returns branch-owned policy state.

Entity-bound execution captures and resumes the advancing native RNG through
`DeterministicStream::checkpoint` and `restore`. Comparison execution retains the
versioned seed and stream addresses for deterministically constructing future
chance and exploration streams. A chance boundary follows a committed turn, so
future policy exploration begins at its next issued decision. Capturing a live
decision, durable serialization and cross-process or cross-version migration
remain outside this in-process profile. Frozen offline policy provenance stays
with the policy artifact.

## Verify

```bash
cargo test --locked --manifest-path crates/paignia-kairos/Cargo.toml --test policy_resume
```

The fixture captures after a chance draw and before a second chance and stateful
policy choice. Prior native dispatch and cancellation distinguish scheduler
history from the game revision. The resumed branch reproduces full native
event IDs, time, counters, ordering, chance outcomes, policy action and payoff
across 64 seeds per stream profile. A sibling changes its decoded policy and
reaches a different payoff while the original and checkpoint remain intact.
Negative cases reject
incompatible formats, version, invalid policy bytes and a nonfresh scheduler.

The candidate uses the native scheduler state change in
[Kairos #401](https://github.com/edithatogo/kairos/pull/401). Its upstream
qualification and immutable accepted pin are prerequisites for integration.
