# T21 scaling and common-accuracy protocol

Freeze this protocol before collecting timing samples. Use the signed Paignia
0.1.0 source, Gambit 16.7.0 and OpenSpiel 2.0.2. The
[correctness protocol](README.md) specifies the source archives and ontology
lock. Capture the exact built binaries, the Python patch release, compiler and
host architecture. Run the same workloads on native Darwin arm64 and native
Linux x86_64. Report timings separately per host because the two environments
have different runtime costs.

## Equivalent workloads

- **Pure Nash** uses two-seat coordination at 2, 4 and 8 actions per player.
  **Mixed Nash** uses 2, 4 and 6. Seed 271 selects distinct positive integer
  payoffs on the matching diagonal. Off-diagonal payoffs are zero. A pure
  equilibrium lies on a matching diagonal. Every nonempty common support has
  one mixed equilibrium, with probabilities proportional to the reciprocal
  of the selected diagonal payoffs. Require the expected profile count and
  the existing exact-Gambit comparison, including the (10^{-9}) mixed
  probability and payoff tolerances. A case with unresolved search scope
  retains its status.
- **Kuhn CFR and CFR+** use the existing paired history, action, payoff and
  policy checks at 100, 1,000 and 12,000 iterations. Time a fresh process at
  each implementation's first *tested* checkpoint where NashConv is at most
  0.02 and the first player's expected payoff is within 0.005 of −1/18. Use
  the same independently verified evaluator and retain the complete
  checkpoint series. Averaging and update schedules remain visible.
- **Backward induction** uses alternating chance and decision layers with
  payoff vector (2, 4) at every terminal. Compare depth 3 and 6 binary
  trees and shared-successor DAGs, plus a depth 3 four-way tree. Every
  decision has a tie and must select the first action. Check every
  labelled continuation in OpenSpiel EFG before timing. Measure Paignia's
  DAG solver and a Python reference backward traversal over OpenSpiel states.
  Conversion includes unfolding for EFG.
- Leduc, correlated-equilibrium, commitment and bounded dynamics retain
  their independently checked correctness receipts. The timed workload
  measures the named overlapping operations.

## Measurement

Record one first launch and nine interleaved repeats per tool and workload.
Retain host caches and randomize tool order with seed 271. Normal-form
conversion begins at RDF parsing and ends with the encoded input bytes. The
timed child includes process startup, input parsing, solver execution and
output. Its maximum resident set size comes from Darwin or GNU time and is
reported in bytes. Kuhn receives prepared inputs and includes the
OpenSpiel Python process startup. The benchmark places a 30-second bound on
each process and returns a failure for timeout, changed outputs or an
unmet target. Keep the paired files, measured samples and process failures.
Compare workload identities and trial records across hosts. Interpret the
measurements within the declared sizes, toolchains and acceptance criteria.
