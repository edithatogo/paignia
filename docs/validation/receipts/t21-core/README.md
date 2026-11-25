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
