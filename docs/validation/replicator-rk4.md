# T17.04 / S01: native continuous replicator steps

The native solver package provides a true classical four-stage RK4 update and a
separately named forward-Euler update. Both operate on a caller-supplied
population and fitness callback. They add no simulation scheduler, graph-result
claim, equilibrium solver, RNG, package dependency or application interpretation.

The bounded Paignia implementation has independent numerical acceptance.
The [integrated T17 receipt](t17-review-20261117.md) records compiler and actual
artifact gates. Kairos Track 110 retains the separate caller/retirement receipt;
provider completion does not close that inherited S01 source obligation or T17.

## Native interface and identity

```rust
use paignia_solve::{replicator_rk4_step, ReplicatorConfig};

let updated = replicator_rk4_step(
    &[0.25, 0.75],
    0.0,
    1.0 / 32.0,
    ReplicatorConfig { mass_tolerance: 1e-12 },
    |time, _population| Ok(vec![1.0 + 2.0 * time, 0.0]),
)?;
```

Both `replicator_rk4_step` and `replicator_euler_step` take
`(&[f64], time, step, ReplicatorConfig, fitness)`. The callback is
`FnMut(f64, &[f64]) -> Result<Vec<f64>, String>`: it receives the actual stage
time and state, and returns a fitness for each population component. A failed
callback returns a typed `ReplicatorError::FitnessCallback` with its stage and
message. Input is borrowed and never mutated. Callback side effects are the
caller's responsibility and are not rolled back on a rejected step.

| Method | Stable method ID | Version | Positive-step fitness evaluations |
| --- | --- | --- | --- |
| `ReplicatorMethod::ClassicalRk4` | `paignia/replicator-classical-rk4` | `1.0.0` | Four |
| `ReplicatorMethod::ForwardEuler` | `paignia/replicator-forward-euler` | `1.0.0` | One |

`ReplicatorStep` records the method/version, start/end time, requested step,
declared mass tolerance, returned population, its actual absolute mass error and
fitness-evaluation count. These are numerical-step records, not finite-game
`ResultEnvelope` values or certificates of a stationary state or equilibrium.
No local-error estimate, adaptive step controller or convergence status is
invented from successful execution.

## Equations and four-stage behavior

For finite population components and fitness values, the implemented right-hand
side is

\[
\bar f(t,x)=\sum_j x_j f_j(t,x),\qquad
g_i(t,x)=x_i\bigl(f_i(t,x)-\bar f(t,x)\bigr).
\]

The RK4 step evaluates

\[
\begin{aligned}
k_1 &= g(t,x),\\
k_2 &= g(t+h/2,x+h k_1/2),\\
k_3 &= g(t+h/2,x+h k_2/2),\\
k_4 &= g(t+h,x+h k_3),\\
x_{\mathrm{next}} &= x+h(k_1/6+k_2/3+k_3/3+k_4/6).
\end{aligned}
\]

This is the classical tableau shown in [John Butcher's University of Auckland
slides, PDF page 22](https://www.math.auckland.ac.nz/~butcher/CONFERENCES/seminar-21-10-04.pdf#page=22).
The source was inspected directly during implementation. The right-hand-side
definition and the independent analytic controls are also recorded in the
[existing frozen reference fixture](../../verification/numerical_oracles/replicator_rk4_v1.json).

Fitness can depend on both time and state. Stages are not four calls to the same
frozen input. The state-dependent control uses
`f1=1/(x1*x2), f2=0` on an interior two-component unit-mass segment. Its equation
reduces to `x1'=1`, so the exact trajectory is linear; this distinguishes genuine
stage-state evaluation from a frozen fitness vector.

The separately named Euler step uses `x_next=x+h*g(t,x)`. It reproduces the
unprojected legacy Euler polynomial on admitted steps. It does not claim full
compatibility with the old clipped/renormalized upstream output outside this
domain, and it is never reported under the RK4 method name.

## Admission and finite arithmetic

The supported population has at least two components. Every initial,
intermediate and final component must be finite and in `[0,1]`. At each state,
absolute mass error must be no larger than the caller's finite, nonnegative
`mass_tolerance`, which must be less than one. The tolerance is an explicit
admission budget. It does not normalize the state, bound numerical trajectory
error, or establish an appropriate scientific tolerance for a particular model.
The numerical controls use `1e-12`; choosing a looser tolerance admits a less
accurate unit-mass representation and does not inherit those accuracy results.

Time is finite and steps are finite/nonnegative. Positive steps require a
finite end time strictly later than the start. RK4 additionally requires a
representable midpoint strictly between them; underflowed or rounded-away stage
times reject before the callback runs. Zero steps preserve population, time and
step bits, including signed zero, and make no fitness calls. Input validity is
still checked for a zero step.

Callback fitness dimension must match the population, and every value must be
finite. Negative fitness is permitted. Mean fitness, derivatives and population
updates must remain finite; otherwise a stage-specific arithmetic error is
returned. A zero-frequency component has exactly zero derivative, avoiding an
unnecessary `0 * infinity` when two extreme finite fitnesses differ beyond the
floating-point range. Finite input alone does not guarantee finite intermediate
arithmetic for nonzero components.

No clipping, renormalization, hidden method substitution or automatic retry is
performed. Explicit RK4 is not unconditionally positivity preserving. A step
that leaves the admitted domain at any intermediate stage rejects even if an
unchecked final polynomial might return to the domain. Callers must choose a
smaller step or a separately qualified method. The callback never receives an
out-of-domain intermediate population from this API.

## Discontinuities and time interpretation

Every call assumes a smooth fitness segment across its four stage evaluations.
At a known discontinuity, end the current integration segment at the event and
start a new one with a new callback. The left segment's final RK4 stage must use
its **left-sided smooth extension** at the endpoint; the next segment uses its
own right-sided extension. Merely aligning a step boundary while passing one
globally right-continuous callback still contaminates the left segment's final
stage.

The frozen control switches the fitness difference from `+1` to `-1` at `t=0.5`.
Two segment-specific callbacks return the population to its initial `1/4`
within `1e-9`; a single right-continuous callback misses by more than `1e-5`.
This is a numerical discontinuity contract. Time has the units chosen by the
model, and the step is not a simulation tick or a native event-scheduling rule.

## Independent controls and numerical tolerances

The unchanged reference file is pinned by SHA-256:

`e793f8f4d2121e58d4195ee2967bfb61fbe533377863a18b3926a08b3210d5e3`.

Its exact rational witnesses and 80-digit analytic literals are transcribed in
the Rust test target with their provenance. Tests do not call the Python
reference implementation and do not include repository-external source files
in the crate. The separate reference harness remains independently executable.

For two components and time-only fitness difference `s(t)`, separation gives

\[
x(t)=\frac{1}{1+((1-x_0)/x_0)\exp(-\int s(u)\,du)}.
\]

The controls compare production results with the frozen literals, using a
`1e-9` finest-grid absolute error bound and successive refinement ratios strictly
between 14 and 18. These are deliberately separate from the reference harness's
80-digit arithmetic thresholds. Actual Rust 1.75 binary64 results were:

| Fitness difference | Step counts | Absolute errors |
| --- | --- | --- |
| `+1` | 8, 16, 32 | `6.49585926e-8`, `4.07407502e-9`, `2.55029442e-10` |
| `-1` | 8, 16, 32 | `8.42467147e-8`, `5.09665771e-9`, `3.13386928e-10` |
| `1+2t` | 32, 64, 128 | `1.69313796e-8`, `1.05393039e-9`, `6.57228716e-11` |

The single-step rational RK4 witness
`408885515741609973/1152921504606846976` is checked within four binary64 epsilons.
The separate Euler result is exactly `11/32` for this dyadic input. Euler and
frozen-stage-time negative controls both fail the RK4 trajectory accuracy bound.
Additional tests check three-strategy constant-fitness dynamics against the
independent closed form `x_i(0)*exp(f_i*t)/sum_j x_j(0)*exp(f_j*t)`, neutral
fitness, simplex faces, signed zero, time representability, callback failures,
dimension errors, overflow, missing mass and oversized steps.

## Source-side characterization and migration boundary

The inspected upstream source is
[Kairos `normal_form.rs` at `cad43b592893579ccbea40343548fda7acd83fd0`](https://github.com/edithatogo/kairos/blob/cad43b592893579ccbea40343548fda7acd83fd0/crates/kairo-ecs-game-theory/src/normal_form.rs),
Git blob `7a93d390ad7ea5225a1e5bf7412a965b22b5cb3d`.
`ReplicatorDynamics::step_continuous_rk4` there performs one Euler increment,
clips negative components, then renormalizes. The same module's
`run_trajectory` calls it with a fixed generated fitness vector. This source
inspection and the original fixtures establish the named discrepancy; the old
API was not executed or modified by this slice.

The caller evidence above is bounded to the inspected module. Feature-aware
repository-wide callers, external consumers, output-version migration and
retirement of the old misleading API remain Kairos Track 110/S01 source-side
work. No provider pin changes or duplicated corrected Kairos implementation are
introduced here.

## Execution evidence

Tests-first commit `0921bfc` executed all 28 new native tests against an explicit
identity prototype: four passed and 24 failed, exit 101. The implemented RK4 and
separately named Euler methods pass all 28. No fixture or acceptance tolerance
was changed to obtain that result. Final shared-tree checks and independent
numerical review belong to the integration receipt.

```bash
cargo test --locked -p paignia-solve --test replicator_rk4
python3 -B verification/numerical_oracles/test_oracles.py ReplicatorOracleTests
```

The standalone package/real-consumer evidence must separately name any claimed
RK4 capability. An existing graph-only consumer smoke is not a replicator
method or real application receipt.

### Integrated acceptance — 17 November 2026

Independent review accepted source `e194e90612d14e99da53b8843dc2431d2a4510f4`
with all 28 author controls, 14 frozen reference controls and ten separately
written native controls. The unchanged external harness is retained at
`verification/replicator_review` and now runs on both CI compiler profiles.
The integrated artifact consumer calls RK4 and checks its independent rational
four-stage witness in every solver profile. The current receipt binds those
results to the exact provider revision. T17.04 is complete for this declared
method; source-side migration and application acceptance remain separately owned.
