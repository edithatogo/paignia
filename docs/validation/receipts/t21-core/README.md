# T21 core comparator results

25 November 2026 · Paignia [signed 0.1.0 source](https://github.com/edithatogo/paignia/tree/a04a19da06cc50e310a172574ed66f62000b891a)

Two native Darwin arm64 replays produced byte-identical receipts and 1,340
hash-matched paired files. The Paignia examples used Rust 1.75.0 and the
release's locked provider source. Gambit 16.7.0 and OpenSpiel 2.0.2 were
built from the verified archives with Apple Clang 21.0.0. OpenSpiel's bindings
used Python 3.15.2 and CMake 4.5.1. The receipt records the five tool hashes,
UOGTO lock and exact replay commit.
[Fixture protocol and commands](../../../../verification/comparators/README.md) ·
[Machine-readable results](results.json)

| Comparison | Result |
| --- | --- |
| Exact Gambit pure Nash | Identical sets on all 256 ordered binary-payoff 2×2 games. 576 equilibrium profiles and two games with no pure equilibrium. |
| Exact Gambit mixed Nash | All 13 fixture witnesses matched across six games. Maximum probability error was `1.12e-16` rounded up, maximum payoff error was zero, and maximum Paignia deviation gain was below `9e-17`. |
| OpenSpiel backward-induction policy evaluation | Buyer–seller root returns matched exactly at `(2, 4)` and the shared-successor variant at `(3, 0)`. Every decision was optimal under its continuation, including the off-path tie in the unfolded tree. |
| OpenSpiel independent Kuhn policy evaluation | Returned Paignia payoffs and NashConv agreed within `1e-9` for CFR and CFR+ at 100, 1,000 and 12,000 iterations. All trained policies met the stated value bound and both trainers' NashConv decreased at each budget. |

| Kuhn at 12,000 iterations | Paignia NashConv | OpenSpiel NashConv |
| --- | ---: | ---: |
| CFR | 0.0041869411 | 0.0002431514 |
| CFR+ | 0.0001084133 | 0.0000164724 |

Paignia uses simultaneous updates for CFR. OpenSpiel's default CFR uses
alternating updates. CFR+ alternates in both engines and OpenSpiel uses
linear averaging. The reported values compare the specified training schedules
at fixed iteration budgets. Rock–paper–scissors reported three unresolved
support pairs. The all-zero game reported one and yielded the four Gambit
vertices of its continuum.

The comparative timing and memory study, broader method coverage and
independent review remain tracked under the
[T21 evaluation plan](../../evaluation-plan.md).

## Additional signed-source checks

On the unchanged signed source, the MSRV build passed 84 targeted controls
for correlated-equilibrium residuals and certificates, bounded commitment,
paired-segment and box Nash feasibility, and RK4. These check oracle values,
degenerate inputs, method admission and feature-specific behavior. They
provide independent checks for the admitted profiles alongside the Gambit
and OpenSpiel comparisons.

## Matched timing pilot

Each normal-form trial regenerated the request or NFG from the same checked
RDF graph and verified input and output bytes against the paired receipt.
Median process time includes startup, input parsing, solving and output.
Peak RSS covers the solver process. Conversion includes the shared ontology
projection, so its cost reflects this verifier.

| Workload | Tool | Median conversion (ms) | Median process (ms) | Median total (ms) | Median peak RSS (MB) |
| --- | --- | ---: | ---: | ---: | ---: |
| all-zero | Gambit | 2.486 | 11.222 | 13.572 | 2.703 |
| all-zero | Paignia | 2.759 | 10.140 | 13.174 | 2.441 |
| asymmetric-coordination | Gambit | 2.527 | 11.253 | 14.255 | 2.720 |
| asymmetric-coordination | Paignia | 2.802 | 9.966 | 13.140 | 2.441 |
| binary-003 | Gambit | 2.800 | 11.843 | 14.754 | 2.408 |
| binary-003 | Paignia | 3.112 | 11.468 | 14.046 | 2.441 |
| matching-pennies | Gambit | 2.553 | 11.087 | 13.522 | 2.671 |
| matching-pennies | Paignia | 2.480 | 10.161 | 12.762 | 2.441 |
| rectangular | Gambit | 3.669 | 12.487 | 15.868 | 2.769 |
| rectangular | Paignia | 3.531 | 10.513 | 14.132 | 2.474 |
| rock-paper-scissors | Gambit | 4.716 | 12.757 | 17.521 | 2.736 |
| rock-paper-scissors | Paignia | 4.743 | 10.331 | 15.193 | 2.474 |

For Kuhn CFR+ at 12,000 iterations, the trials used the pretranslated JSON
and the native `kuhn_poker` registration. Each run reproduced the checked
utilities and NashConv. These process measurements include each tool's
startup and policy extraction. OpenSpiel also starts Python.

| Tool | Median process (ms) | Median peak RSS (MB) |
| --- | ---: | ---: |
| OpenSpiel | 748.548 | 25.641 |
| Paignia | 139.442 | 3.244 |

The [raw receipt](comparator-timings.json) contains the first process and
nine interleaved repeats, ranges, host/runtime identity and the paired
receipt digest. These are small correctness-bound measurements on one
Darwin arm64 host with no system cache purge. A performance ranking,
scaling and platform portability require their own results.
[Reproduction command](../../../../verification/comparators/README.md#timing-pilot)
