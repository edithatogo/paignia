# Bounded native mixed Nash, version 1.0.0

Status: actual Rust support enumeration and candidate diagnostics implemented
and accepted by independent numerical/API review. The
[integrated T17 receipt](t17-review-20261117.md) binds compiler and package gates.
This method supplies finite binary64 witnesses under explicit
tolerances. It does not certify an exhaustive equilibrium catalogue.

## Supported mathematical and native interface

`paignia_solve::solve_mixed_nash(A, B, algorithm, config)` accepts two complete
finite payoff matrices for a two-player, finite, simultaneous, complete-information
game with independent mixed strategies. There are one to eight row actions and
one to eight column actions. `A[i][j]` is the row player's payoff and `B[i][j]`
is the column player's payoff at that action pair. Rectangular shapes, zero
probabilities and negative payoffs are supported. Every cell is explicit.
Omitted payoffs, ragged or mismatched tables, and non-finite values are errors.

`evaluate_mixed_nash_candidate(A, B, p, q, config)` checks a supplied profile's
probability domain and computes its payoffs and unilateral gains without
enumerating supports. It can evaluate supplied degenerate or unequal-support
witnesses even though the enumerator does not solve those support systems.

Coordinates are local mathematical action indices. This interface neither
invents UOGTO IDs nor defines a graph-to-normal-form conversion, persistent game
identity, serialization format or simulation contract. The graph result envelope
and legacy `NormalFormGame`/pure-Nash methods retain their existing interfaces.
All numerical execution is Rust; there is no Kairos or Python runtime dependency.

The selected algorithm is `MixedNashAlgorithm::SupportEnumeration`, with ID
`paignia/equal-support-enumeration` and version `1.0.0`. A request for
`MixedNashAlgorithm::LemkeHowson` returns `UnsupportedAlgorithm`, no algorithm
version, no scope, no candidates, and `NotRun`/`NotClaimed` search/catalogue
records. Configuration and table validation precede that response. There is no
Lemke–Howson fallback or implementation hidden behind this name.

## Numerical method and search scope

For a row support `I` and column support `J` of common size `k`, solve:

\[
\begin{aligned}
\sum_{j\in J}(A_{ij}-A_{i_0j})q_j&=0 && (i\in I\setminus\{i_0\}),\\
\sum_{j\in J}q_j&=1,\\
\sum_{i\in I}(B_{ij}-B_{ij_0})p_i&=0 && (j\in J\setminus\{j_0\}),\\
\sum_{i\in I}p_i&=1.
\end{aligned}
\]

Each indifference row is divided by its largest absolute coefficient when that
value is nonzero. The unit-mass row is already scaled. Each system has at most
eight variables. Partial-pivot Gaussian elimination solves it, and the resulting
solution is checked against the original scaled equations. No external linear
algebra or optimizer dependency is introduced for this bounded system; this is
not a general-purpose linear algebra API.

A pivot no larger than `pivot_tolerance`, or a scaled equation residual larger
than it, marks that support pair unresolved. An unresolved rank check can mean
an exactly singular system, an inconsistent singular system, numerical near-rank
deficiency, or insufficient binary64 precision. The result does not equate a
threshold decision with a mathematical degeneracy proof. Nonpositive coordinates
are rejected for the selected support; boundary points belong to smaller
supports. Mass/domain failures from the calculated positive solution are also
unresolved. No clipping, uniform replacement, projection or normalization occurs.

Supports are visited first by increasing common cardinality, then lexicographic
row support, then lexicographic column support. There are
`sum(C(m,k)*C(n,k), k=1..min(m,n))` pairs; at 8x8 this is 12,869. A positive
`max_support_pairs` budget stops the procedure exactly after that many visits,
retaining already verified witnesses. The result reports the scope, visited and
total pairs, unresolved pairs, nonpositive support solutions, rejected best
responses, and every admitted candidate. These counts partition visited pairs.

Unequal support sizes, singular solution polytopes, and continuum reconstruction
are not enumerated. For example, the zero 2x2 game yields four pure witnesses
and an unresolved full-support pair. All mixed profiles of that game are
equilibria, so four output records cannot describe its full equilibrium set.
Even a completed nonsingular search carries `FiniteVerifiedWitnessesOnly`.
There is no global exhaustive-catalogue flag or `NoEquilibrium` status.

## Payoffs, unilateral gains and tolerances

The evaluator preserves supplied probability values. Components must be finite
and in `[0,1]`; each sum must be within `probability_tolerance` of one. The
reported probability-mass errors are absolute, and exact zero remains a valid
unsupported-action probability. A small positive probability is never removed
merely because it is below a tolerance.

The expected payoffs are the unmodified bilinear forms `p^T A q` and `p^T B q`.
For every possible pure deviation, the evaluator sums payoff **differences**:

\[
\begin{aligned}
d_r(a)&=\sum_{i,j}p_iq_j(A_{aj}-A_{ij}),\\
d_c(b)&=\sum_{i,j}p_iq_j(B_{ib}-B_{ij}),\\
g_r&=\max(0,\max_a d_r(a)),\qquad
g_c=\max(0,\max_b d_c(b)).
\end{aligned}
\]

For unit-mass distributions these are the usual best-response payoff minus
current payoff. The evaluator accumulates both sides of each weighted payoff
difference in one expansion. It does not subtract separately rounded payoff
totals or round a potentially inexact payoff difference before multiplication.
With admitted mass roundoff, these formulas on the unchanged weights define the
diagnostics; no silently normalized profile is substituted.

Payoff, deviation, probability-mass, support-residual and back-substitution
sums retain low addition parts in a floating-point expansion. Each addition
uses magnitude-ordered Fast-Two-Sum; see
[Shewchuk (1997), sections 2.3–2.4](https://people.eecs.berkeley.edu/~jrs/papers/robustr.pdf).
The low parts remain available when larger terms later cancel. Mass error is
formed by adding minus one to the expansion before rounding, so a residual
smaller than half an ulp at one is retained. The mass error is included in a
typed invalid-mass error even when the displayed rounded total equals one.
Probability products and their weighted payoff terms retain both the rounded
product and its fused multiply-add remainder; Rust's
[`f64::mul_add` contract](https://doc.rust-lang.org/std/primitive.f64.html#method.mul_add)
provides the single-rounding operation. Before extracting a remainder, the
implementation finds the least significant set-bit exponent of each nonzero
operand. Their sum must be at least `-1074`, ensuring that an exact low product
part cannot silently underflow below the smallest binary64 component. A lower
exponent causes `ArithmeticFailure` even if the main rounded product is nonzero.
The `2^-1074` boundary remains admitted. The same checked product accumulation
is used in support residuals and back substitution. Gaussian systems and final
diagnostic rounding still have binary64 limits; this supplies no exact
equilibrium or catalogue certificate.

`best_response_gains` reports `[g_r,g_c]`, `max_gain` is their maximum, and
`sum_gain` is their sum (often called NashConv). No average-exploitability value
is implied. `within_tolerance` is true exactly when both computed gains are at
most the configured equilibrium tolerance. Every emitted search candidate must
pass this test in the original payoff units. It is a binary64 admission test,
not an exact or interval arithmetic proof. Positive admitted gains remain
positive: the `2e-30` control is admitted at the default tolerance and rejected
at zero tolerance without erasing the reported gain.

| Parameter | Default | Interpretation |
| --- | ---: | --- |
| `probability_tolerance` | `1e-12` | Absolute distance of each unchanged probability sum from one; finite, nonnegative, less than one |
| `equilibrium_tolerance` | `1e-10` | Absolute unilateral gain per player, in that player's original payoff units; finite and nonnegative |
| `pivot_tolerance` | `1e-12` | Pivot floor and maximum residual for row-scaled indifference equations; finite, nonnegative, less than one |
| `max_support_pairs` | `12869` | Positive number of support pairs permitted |

Positive affine payoff transformations preserve exact Nash strategies, but an
absolute gain tolerance must be scaled with payoffs if the same approximate
admission decision is desired. Near-threshold floating-point rank or gain
decisions are not guaranteed invariant to all scaling or action reordering.
The tests establish these identities on the stated independently controlled
examples, not universal conditioning guarantees.

Any non-finite intermediate, division failure, nonzero multiplication or
division rounded to zero, or exact expanded-product low part below `2^-1074`
produces a typed `ArithmeticFailure`. Such an error
aborts the call, including any partially collected search results. Thus some
mathematically valid finite games with extreme dynamic range are explicitly
outside executable numerical range. Subnormal rounding and ordinary floating-
point roundoff are not exact error bounds. A zero returned gain is not an
exact-arithmetic equilibrium certificate.

| Status | Meaning |
| --- | --- |
| `CompletedWithinScope` | All declared pairs visited, no unresolved systems, at least one admitted witness |
| `UnresolvedSupports` | All declared pairs visited, with one or more unresolved rank/residual/mass checks |
| `BudgetExhausted` | More declared pairs remain; already admitted witnesses are retained |
| `NoVerifiedWitness` | No candidate passed the declared numerical checks and no support was unresolved; no mathematical absence claim |
| `UnsupportedAlgorithm` | The requested algorithm is not implemented; no substitute search ran |

`AllDeclaredSupportPairsVisited` is procedural accounting and includes unresolved
pairs. It never upgrades `FiniteVerifiedWitnessesOnly` to catalogue completeness.
Selection means all admitted candidates in support order; it makes no economic
choice among equilibria.

## Independent controls and provenance

The mathematical constants are frozen in
[`mixed_nash_v1.json`](../../verification/numerical_oracles/mixed_nash_v1.json),
SHA-256 `9545116fc6aa4b1026bf6b2a6a9b509af92d8ed6ca2204e68bcdccf8e2e126e3`.
They are original rational constructions under Apache-2.0, not solver outputs.
The independent exact-rational harness does not import production code. The
Rust tests copy the constants, allowing isolated `.crate` tests to execute
without a repository-relative fixture, Python runtime or JSON dependency.
The source fixture and harness are unchanged by this implementation.

The 33 native tests cover all eight examples and 12 independently verified
witnesses: strict dominance, matching pennies, three coordination equilibria,
asymmetric interior play, rectangular 2x3 and 3x2 games, the zero-game continuum
and a negative-payoff single-action game. Five false candidates have pinned
positive gains. Additional controls test player/action permutations, positive
affine transformations, large common offsets, the `2e-30` gain, positive tiny
support probabilities, singular and near-singular systems, exact budget counts,
unsupported Lemke–Howson, malformed tables/probabilities/configuration, explicit
overflow/underflow failures, and reproducible support accounting.

The mathematical support equations follow directly from indifference of
positive-probability actions; all-action deviation checks supply the remaining
best-response inequalities. A mixed deviation is a convex combination of pure
deviations, which explains why all pure checks suffice for exact unit-mass
vectors. The [Gambit 16.6.0 algorithm documentation](https://gambitproject.readthedocs.io/en/v16.6.0/algorithms.html)
distinguishes support/extreme-point enumeration and Lemke–Howson paths and their
different catalogue properties. This implementation does not execute Gambit,
copy its algorithms, or inherit its completeness guarantees.

The fixed [Kairos source revision `cad43b5`](https://github.com/edithatogo/kairos/blob/cad43b592893579ccbea40343548fda7acd83fd0/crates/kairo-ecs-game-theory/src/normal_form.rs)
contains `MixedNashSolver::solve_2x2_and_support_enumeration`, whose zero-
denominator fallback chooses half/half. `LemkeHowsonSolver::solve` calls that
helper. `CorrelatedEquilibriumSolver::verify_and_solve` calls the named
Lemke–Howson helper and forms a product of its marginals. This is a bounded
source-level characterization of those definitions and direct in-file calls,
not a complete feature-aware workspace caller inventory or executed upstream
qualification. The dominance fixture proves the half/half candidate has gain
one half for each player while the pure dominant profile has gain zero.

## Execution receipt and remaining qualification

Tests-first commit `4098e1d602ec449fd07de8e7bba5555525a7a469` executes 33 tests:
zero pass, 33 fail, exit 101, against explicit unimplemented methods. The actual
initial implementation passes all 33 tests under Rust 1.75.0. The unchanged independent
Nash oracle runs 12 passing tests on CPython 3.12.14. An initial Clippy run
identified one missing Markdown code delimiter in the test provenance comment;
the comment was corrected without changing a scientific assertion. The complete
workspace runs 137 passing native tests at this source revision, and Clippy with
warnings denied and the two-crate/six-profile extraction validator pass.

Independent review of implementation `77f329d` found that naive accumulation
erased a profitable deviation: with `H=2^54`, `A=[[H,1,-H],[0,0,0]]`, zero `B`,
`p=[0,1]` and `q=[1/4,1/2,1/4]`, the exact gain is `1/2`; the old implementation
reported zero. Changing column order exposed it. The same defect affected a
cancelled expected payoff. Review-fix test commit
`0e64c460047ec21a9ee2ce573ec2ebd340f4db88` executes 36 tests: 33 pass and three
fail (exit 101). Its additional controls cover all six column permutations,
cancelled signed expected payoffs, and a probability-mass error of `2^-56`
hidden when the total is rounded to one first. The expansion sums above address
these sites together; the declared acceptance tolerances are unchanged.

A sum-only repair still lost a gain of `1/2` when the difference `2^54-(-1)`
rounded before multiplication, and a gain of `0.1` when rounded weighted
products cancelled. Both additional tests fail independently (one executed
failure each, exit 101) before the checked fused-product repair; they are in
test commit `c3b0d4211d7248de5b53786bc687f39c4ab80334`. A further low-product-range
control fails before the guard is implemented, despite a nonzero main product.
The corrected focused suite executes 39 passing tests under Rust 1.75.0 and
stable 1.100.0, with the original acceptance tolerances unchanged. The full
Rust 1.75 workspace executes 143 passing native tests; Clippy with warnings
denied also passes. The subsequent accepted review is recorded below; these
earlier counts describe the corresponding historical candidate.

Focused commands:

```bash
cargo test --locked -p paignia-solve --test mixed_nash
python3 -B verification/numerical_oracles/test_oracles.py NashOracleTests
```

The repository-required formatting, Clippy, full native, extracted-source and
package checks remain separate evidence from independent specialist numerical
review. Package consumers that only exercise the graph or pure-Nash methods do
not qualify this mixed method merely by compiling its crate.

### Integrated acceptance — 17 November 2026

Independent review accepted corrected source
`671898780dcf2d7ef7a74252fa0da23cffebd0c0` after all 39 author controls and 14
separately written native controls passed on Rust 1.75. The 14-control harness
is retained unchanged at `verification/mixed_nash_review`, SHA-256
`5d06571759a79556e2e5a216a7debf975dc4d1c4b39075ac9579207e3ee0c041`.
It runs on both CI compiler profiles. Actual package consumers call the method
and check matching pennies plus the independently derived half-unit cancellation
regression in every solver profile. Production tolerances were not loosened.

T17.05 is complete for the declared bounded Paignia algorithm. Kairos Track 110
retains the feature-aware caller, old API/output migration and retirement part
of S02. Gambit execution is not an added gate: the controlling specification
requires independent evidence and permits explicit algorithm boundaries, both
provided here. No consumer pin or old Kairos API is changed by this acceptance.
Rollback is a normal revert of the additive implementation.
