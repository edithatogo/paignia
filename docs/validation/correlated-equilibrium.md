# Bounded correlated equilibrium with interval admission — T17.06 / S03

## Supported input and method

The optional correlated-equilibrium feature provides a native two-player
linear-programming method using the
[qualified pinned backend](clarabel-backend.md). The method identifier is
paignia/clarabel-correlated-equilibrium, version 1.0.0; the backend is
Clarabel 0.11.1 with the Rust QDLDL path. All execution and independent
residual calculations are Rust. The feature is disabled by default.

Inputs are three complete finite binary64 matrices with identical rectangular
shape: row-player payoffs A, column-player payoffs B, and the explicitly
supplied objective coefficients c. There must be one to eight actions for
each player. Coordinate (i,j) means row action i, column action j.
Every cell is explicit; missing values are never imputed. The objective has no
automatic welfare or payoff-sum default. Players retain their individual payoff
units, and the caller owns the objective's units and interpretation.

These local array coordinates do not establish specification, instance, policy
or UOGTO identity. This method record describes its own bounded bimatrix class;
it is not admitted by the separate finite, perfect-information graph profile.
There is no support here for hidden information, information sets, dynamics,
more than two players, or simulation.

The public entry points are:

~~~rust
solve_correlated_equilibrium(A, B, c, config)
evaluate_correlated_equilibrium_candidate(A, B, c, joint, config)
evaluate_correlated_equilibrium_certificate(A, B, c, joint, dual, config)
~~~

The first returns an immutable result observation. The second checks approximate
primal feasibility of unchanged raw joint weights, without claiming objective
optimality. The third independently checks a native primal/dual certificate,
including primal admission and objective gap. A successful Result from an
evaluator means diagnostics were computed; inspect within_tolerance to decide
their declared admission. No Clarabel type is exposed in this API.

## Primal and dual equations

Flatten the joint vector x in row-major order. For a recommended row action
i and a distinct deviation k, the deviation gain is

~~~text
sum_j x[i,j] * (A[k,j] - A[i,j]).
~~~

Column recommendations use

~~~text
sum_i x[i,j] * (B[i,l] - B[i,j]).
~~~

Let D contain these gain coefficients, with row recommendations/deviations
first in increasing order, followed by column pairs; self-deviations are omitted.
The requested program is

~~~text
maximise c^T x
subject to 1^T x = 1, x >= 0, D x <= 0.
~~~

The backend receives zero quadratic matrix, linear coefficients -c, and
constraint rows [1^T; -I; D]. A zero cone imposes mass one and a single
nonnegative cone imposes the inequalities. With r row and s column actions,
there are r*s variables and 1 + r*s + r*(r-1) + s*(s-1) constraints, at most
64 and 177 respectively. A product of marginals never replaces the joint.
Sparse matrix dimensions and storage format are checked before construction.
Finite input payoffs whose differences cannot be represented as finite backend
coefficients return an explicit arithmetic error.

The public CorrelatedEquilibriumDualVariables contains a free mass multiplier
lambda, nonnegative multipliers mu for -I, and nonnegative multipliers y
for D. In the code the free multiplier field is named mass. The independent
stationarity equation for each joint coordinate is

~~~text
lambda * 1 - c - mu + D^T y = 0.
~~~

The dual objective is lambda; the signed gap is lambda - c^T x. Exact
feasible primal and dual points with zero gap establish the usual LP optimum.
The implemented record instead reports finite interval enclosures, configured
residual tests, and an explicitly approximate numerical point. It does not
claim exact feasibility, an exact probability distribution, or a rigorous
distance to the exact CE polytope from these tolerance tests alone.

Dual signs are checked exactly on the finite supplied multipliers: a negative
inequality multiplier is rejected, never clipped or absorbed into a stationarity
tolerance. Original-unit stationarity and objective gap are recomputed from the
payoff tables, objective and dual vectors. They do not reuse the backend's CSC
matrix, residual norm or internally scaled variables.

## Outward arithmetic and visible uncertainty

The admission arithmetic encloses real operations on the supplied binary64
inputs. Each addition/subtraction, product and positive-denominator quotient
uses the immediately adjacent representable floats outwards from the rounded
endpoint calculation. Products and quotients consider all four endpoint pairs.
The implementation uses bitwise adjacent-float steps available on Rust 1.75.
An operation or outward endpoint that is non-finite returns an explicit
arithmetic failure.

This accounts for cancellation and gradual underflow. For example, a positive
quantity that rounds to zero is enclosed out to the smallest positive subnormal;
it cannot prove a zero gain under a zero tolerance. Large cancellation produces
an interval that can be too wide for admission even if the ordinary rounded sum
is zero. This conservative rejection is intentional. It is not repaired by
lowering a residual or changing the requested tolerance.

Exact point identities avoid artificial uncertainty for addition by zero,
opposite finite point values, multiplication by zero or plus/minus one, and
division by one. These identities do not modify caller values or distributions.
The arithmetic assumes ordinary IEEE binary64 operations with gradual
underflow; qualified executions use native Linux x86-64 without fast-math
reassociation or a changed floating-point environment.

Every reported interval endpoint is finite. Mass admission uses an upper bound
on abs(sum(x)-1). Nonnegativity reports the largest negative-coordinate
magnitude. Each player's incentive diagnostic is the maximum of zero and the
upper endpoints of that player's weighted deviation gains. No epsilon is
subtracted from a reported gain.

The conditional gain is separately enclosed by division by the recommendation
mass only when that entire mass interval is strictly positive. Otherwise it is
None, with the mass interval still visible. This covers exact zero mass and
uncertain/nonpositive approximate mass; it does not invent a conditional ratio.
No probability tolerance is used as a small-recommendation cutoff. A tiny
positive recommendation can have a small admitted weighted gain and a large
visible conditional gain. Admission bounds weighted CE constraints, not those
conditional ratios.

## Configuration, status and raw output

| Configuration | Default | Meaning |
|---|---:|---|
| probability_tolerance | 1e-8 | Absolute mass-error and negative-weight budgets; finite, nonnegative and less than one |
| incentive_tolerance | [1e-8, 1e-8] | Per-player weighted deviation budgets in original payoff units; finite and nonnegative |
| dual_tolerance | 1e-8 | Original-unit stationarity residual bound in objective units; finite and nonnegative |
| objective_tolerance | 1e-8 | Absolute signed-gap enclosure magnitude and backend-objective consistency budget; finite and nonnegative |
| max_iterations | 200 | Explicit count from zero through 10,000 inclusive |

The backend feasibility and absolute/relative gap tolerances are independently
fixed to 1e-10; QDLDL is selected explicitly and solver printing is disabled.
The iteration limit is the public work budget. No wall-clock completion bound
is claimed. A tighter admission tolerance can reject a backend Solved point.
Negative/non-finite tolerances, excessive dimensions and malformed inputs are
typed errors. Signed-zero configuration values are retained.

| Result status | Meaning |
|---|---|
| ApproximateOptimum | Backend Solved, finite diagnostics, all independently bounded primal/dual checks admitted, and backend objectives consistent with the original-unit calculations |
| ResidualRejected | Backend Solved, but at least one computed bound exceeds the configured tolerance or a dual multiplier is negative |
| NumericalFailure | Backend Solved, but finite independent certification or finite backend diagnostics are unavailable |
| BackendTerminated | Any backend termination other than Solved |

The original backend termination remains separately accessible. In particular,
AlmostSolved, iteration/time limits, insufficient progress, numerical errors
and infeasibility statuses never become admitted equilibria. An infeasibility
status is an observation about this numerical solve, not proof that a complete
finite game lacks a CE. Invalid/unavailable backend floating diagnostics are
represented as None, never successful zero residuals.

The accepted_candidate() accessor returns a reference only for
ApproximateOptimum. The candidate() accessor may also expose an unaccepted point
under ResidualRejected for inspection. The numerical_error() accessor retains
a post-solve arithmetic failure. The result's admission fields are private:
callers cannot mutate its status or replace its candidate. Public candidate
and diagnostic values can be cloned for inspection, but edited copies must be
reevaluated and do not alter the result.

The raw_joint_weights are unchanged unscaled backend coordinates. Successful
bounded examples include negative coordinates around 1e-12. The record
preserves these and their nonnegativity residual; it does not silently project,
clip, renormalise, or present them as exact valid probabilities. Payoff and
objective intervals are linear moments of these raw weights. Consumers needing
strict probability-domain inputs must make a separate explicit decision; this
API does not manufacture that contract.

## Independent cases, controls and qualification

The unchanged
[correlated_equilibrium_v1.json](../../verification/numerical_oracles/correlated_equilibrium_v1.json)
has SHA-256
1271809d3bf6a64c04d48456d42b87448f8c884cc159d0125ec68fc177a4f9ac.
It contains original rational examples with analytic primal/dual certificates.
Their coefficients, negative controls and optima are retained in the native
integration target; no expected optimum is generated from Clarabel.

The tests-first commit 7a04570 fails with E0432 for the absent public APIs
(exit 101). This is not a passing or zero-test execution. The explicit target
then runs 27 tests:

| Group | Distinguishing evidence |
|---|---|
| Five native optima | Correlation gap 21/2, dominant actions 4, rectangular 2 by 3 objective 0, negative single action -7, and a caller objective unrelated to zero payoffs |
| Four false substitutions | Feasible but suboptimal Nash product, product of CE marginals, infeasible higher objective, and asymmetric column deviations |
| Four raw-probability/recommendation controls | Zero mass, tiny positive recommendation with a large conditional gain, retained negative approximate weight, and invalid mass/negative points |
| Four certificate controls | Independent exact analytic certificate; negative incentive multiplier despite correct stationarity/gap; wrong stationarity; feasible but loose dual objective bound |
| Three arithmetic controls | Exact positive gain hidden by naive 2^54, 1, -2^54 cancellation; positive underflow below the minimum subnormal; unrepresentable payoff differences |
| Two termination/admission controls | Actual zero/one-iteration nonconvergence and rejection of a Solved point under zero admission tolerances |
| Five input controls | Malformed/oversized matrices, all non-finite input roles, candidate/certificate dimensions, invalid tolerances/budget, signed-zero preservation |

Run the target with the feature explicitly enabled:

~~~sh
cargo test --locked -p paignia-solve --features correlated-equilibrium --test correlated_equilibrium
~~~

Without the feature, this target executes no CE tests. Such a run is not CE
qualification. Backend feasibility, native implementation tests, independent
numerical review, packaged feature consumption, hosted compiler profiles and
external application receipts are distinct evidence. This slice does not update
old scientific fixtures, legacy consumer pins, source retirement records or
either real application receipt. The Paignia method and the external Kairos
source-disposition parts of S03 have separate completion records below.

### Local execution and independent review record

The numerical/API candidate at local revision
026ccb8dcb95550cc1afcaddbf5114a89f6fe81a passes all 167 workspace tests with
all targets and features enabled on Rust 1.75.0 and stable 1.100.0, including
the 27 method tests and eight backend probes. The seven isolated extraction
profiles and seven packaged-consumer profiles also pass; the latter run 26
existing static/RK4 consumer checks and compile the CE feature. Executing CE
itself from artifacts is a separate coordinated consumer-gate extension.

Independent public-API review accepts the numerical/API candidate with no
blocker. Thirteen additional native tests pass on Rust 1.75, including a
primal-invalid but dual-valid zero-gap certificate, negative dual signs,
36 three-action dual permutations, rectangular transposition, independent
payoff/objective units, cancellation and arithmetic-rounding boundaries,
both-sign half-subnormals, maximum-float identities/overflow, lost mass,
signed-zero recommendation mass, and five extreme finite backend scales.
The external review harness has SHA-256
7a8189e7f9faa03e842dc704670f58cd5389935fc1c57a7c0df7e361c92435ca
and is retained by the integration coordinator. A following documentation-only
correction adds the code formatting required by stable Clippy to the iteration
budget's field documentation; it changes no arithmetic or API.

### Integrated acceptance — 17 November 2026

T17.06 is complete for the declared optional Paignia CE program. The
[integrated T17 receipt](t17-review-20261117.md) binds the combined source,
compiler gates and actual artifact consumption. The unchanged 13-control
external harness is retained at `verification/correlated_equilibrium_review`;
CI requires 13 executed tests with its feature enabled on both compilers.
The two artifact profiles enabling CE execute the independently frozen 10.5
optimum and a zero-budget nonconvergence case. A feature-disabled zero-test run
is never CE qualification. Kairos Track 110 still owns the old product-of-Nash
API, its callers, migration and retirement. Real application receipts remain
application-owned and are not supplied by these mathematical controls.
