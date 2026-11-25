# T21 scaling and common-accuracy results

25 November 2026 · [Benchmark protocol and commands](../../../../verification/comparators/study.md)

Paignia 0.1.0 (signed source `a04a19da06cc5`), Gambit 16.7.0 and OpenSpiel
2.0.2 ran the same 13 workloads on native Darwin arm64 and Ubuntu Linux
x86_64. Both fresh builds used Rust 1.75.0 for Paignia and CPython 3.15.2
for the verifier and OpenSpiel. Darwin used Apple Clang on an M1 Max.
Ubuntu used GCC 13.3.0 on a Xeon 6975P-C. The exact
[reviewed harness](https://github.com/edithatogo/paignia/tree/9b1a34788fddf5cbf319ea6854df53cef2be719c)
and all 35 harness input digests agree across the two runs.

## Correctness and method bounds

- **Normal form.** Paignia and exact Gambit agree with the analytic witness
  counts. The 2-, 4- and 8-action pure cases yield 2, 4 and 8 profiles.
  The 2-, 4- and 6-action mixed cases yield 3, 15 and 63 witnesses.
  The 4- and 6-action cases also report 6 and 380 unresolved support pairs.
  Their observed witnesses match the analytic sets at the stated tolerance.
- **Backward induction.** All five layered cases preserve labelled
  continuations, root payoffs `(2, 4)` and the first-action tie rule.
  The depth-six binary DAG uses 7 native states and unfolds to 127 EFG
  states. Its ordinary tree peer uses 127 states in both representations.
- **Kuhn.** The first tested checkpoint meeting NashConv ≤ 0.02 and
  first-player value error ≤ 0.005 was 1,000 for Paignia CFR and 100 for
  OpenSpiel CFR. Both CFR+ implementations met the target at 100. The
  receipts retain both training schedules and the complete checkpoint series.
- **Other profiles.** The [core correctness receipts](README.md) cover
  independent checks for correlated equilibrium, commitment, bounded
  dynamics and the other admitted fixtures. Comparative timing here uses
  the equivalent workloads specified above.

## Measurements

Each pair is **Paignia / comparator**, using Gambit for normal form and
OpenSpiel for the remaining rows. Values are medians over nine interleaved
repeat launches after the first timed launch. Normal and static elapsed
times include RDF conversion and process execution. Kuhn starts from the
prepared input. Process peak RSS uses the host's time command. OpenSpiel's
process measurements include Python startup, and its static traversal runs
in Python. Caches were retained after the correctness runs.

| Workload | Darwin ms | Darwin RSS MiB | Linux ms | Linux RSS MiB |
| --- | ---: | ---: | ---: | ---: |
| Mixed Nash, 2 actions each | 9.00 / 10.11 | 2.34 / 2.58 | 2.93 / 3.52 | 3.11 / 5.52 |
| Mixed Nash, 4 actions each | 12.53 / 17.16 | 2.72 / 2.78 | 6.41 / 10.28 | 3.24 / 5.65 |
| Mixed Nash, 6 actions each | 21.31 / 64.65 | 3.50 / 3.23 | 15.43 / 51.38 | 4.00 / 6.00 |
| Pure Nash, 2 actions each | 9.21 / 9.61 | 2.36 / 2.30 | 2.94 / 3.19 | 3.08 / 5.17 |
| Pure Nash, 4 actions each | 12.05 / 12.86 | 2.53 / 2.38 | 6.17 / 6.35 | 3.24 / 5.17 |
| Pure Nash, 8 actions each | 25.21 / 24.39 | 3.08 / 2.47 | 19.49 / 19.07 | 3.46 / 5.30 |
| Binary DAG, depth 3 | 9.35 / 50.59 | 2.16 / 24.25 | 2.90 / 30.23 | 2.88 / 27.49 |
| Binary DAG, depth 6 | 10.12 / 48.78 | 2.17 / 24.31 | 3.68 / 31.26 | 2.87 / 27.67 |
| Binary tree, depth 3 | 12.00 / 50.37 | 2.20 / 24.28 | 5.60 / 32.50 | 2.88 / 27.53 |
| Binary tree, depth 6 | 38.95 / 74.23 | 2.77 / 24.38 | 35.20 / 63.69 | 3.14 / 27.67 |
| Four-way tree, depth 3 | 30.52 / 68.82 | 2.64 / 24.31 | 25.74 / 54.78 | 3.18 / 27.54 |
| Kuhn CFR at target | 13.44 / 48.09 | 3.06 / 24.36 | 6.55 / 32.30 | 3.31 / 27.90 |
| Kuhn CFR+ at target | 9.26 / 48.63 | 3.08 / 24.39 | 2.69 / 31.94 | 3.34 / 27.91 |

## Evidence and review

The two manifests each cover 98 files and 260 timed samples. Readback
verified 47 byte-identical input or exact-output files, 33 paired JSON
outcomes agreeing within the declared tolerance, unique static run
identities and all sample summaries. The independent review checked the
Darwin receipt, equilibrium witnesses and Kuhn evaluation. Its five
findings on failure retention, process lifetime, run identity, build
provenance and CI triggers were addressed before the final runs.

| Summary | SHA-256 |
| --- | --- |
| Darwin arm64 | `55e5c2e7850af3cf9f1a522626ae811fa9bdf2fcfa706cd26050c2e66218a7d7` |
| Linux x86_64 | `e4c525a42538b6387e937b6056dc1532e2e3cfff3fe82fe4ff5a568d7700eac2` |

The Linux workflow retains the raw inputs, outputs, measurements, build
record and host identity. The benchmark PR records the paired Darwin
receipt identity, and the reproduction command builds the same locked
workloads on either host.
[Benchmark PR](https://github.com/edithatogo/paignia/pull/102) ·
[Linux raw evidence](https://github.com/edithatogo/paignia/actions/runs/45609237145)
