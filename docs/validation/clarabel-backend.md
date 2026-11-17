# Clarabel LP backend decision — T17.06 / S03

## Decision and scope

Use exactly `clarabel 0.11.1`, with default features disabled and its `serde`
feature explicitly enabled, behind the optional `paignia-solve` feature
`correlated-equilibrium`. The solver package's default feature set is empty.
This is a dependency and backend feasibility decision. It does not introduce a
public Paignia correlated-equilibrium API, close S03, or qualify an application.
The future method needs its own numerical contract, tests and independent review.

The qualified backend path uses binary64, a zero quadratic objective matrix,
`ZeroConeT` and `NonnegativeConeT`, and the Rust `qdldl` linear solver. SDP,
BLAS/LAPACK, faer, Pardiso, Python, Julia and build-information features are not
enabled. No Kairos or Voiage package occurs in the resolved closure. Core and
default solver consumers do not acquire Clarabel or its serialization packages.
The `serde` feature is an explicit upstream compatibility workaround; no JSON
conversion or host-language solver is used to execute the probes.

## Exact upstream inputs and alternatives

| Input | Identity |
|---|---|
| Registry package | `clarabel =0.11.1` |
| Registry archive SHA-256 | `d950c870fad4feed732bc3f7da489d223ba3911eac0522e6512eed7e659cfafe` |
| Registry source's VCS revision | `25540f559592068d0c8a80e46ded1b21760212a1` |
| Declared backend MSRV | Rust `1.70.0` |
| Backend licence | Apache-2.0 |
| Upstream `LICENSE.md` SHA-256 | `6e2f382771f8aab58e4868ddd0afe0c2fefe6b54f9681493a9c3d0f32906cfe1` |
| Provider lock SHA-256 at this dependency slice | `446177417a9e64a5407d43d8f165425c06dc3ed2c803a4adabb0490457fec418` |

These identities come from the downloaded registry archive, its Cargo metadata,
its `.cargo_vcs_info.json`, and the committed provider lock. The authoritative
[manifest](https://github.com/oxfordcontrol/Clarabel.rs/blob/25540f559592068d0c8a80e46ded1b21760212a1/Cargo.toml),
[licence](https://github.com/oxfordcontrol/Clarabel.rs/blob/25540f559592068d0c8a80e46ded1b21760212a1/LICENSE.md),
and [changelog](https://github.com/oxfordcontrol/Clarabel.rs/blob/25540f559592068d0c8a80e46ded1b21760212a1/CHANGELOG.md)
are pinned to that revision. Declared MSRV alone was not accepted as evidence.

Three configurations were investigated before adding the provider dependency:

| Configuration | Rust 1.75 result | Stable 1.100 result | Decision |
|---|---|---|---|
| `0.11.1`, all optional features disabled | Compilation fails with E0433: `SolverError::JsonError` references unavailable `serde_json` | Not needed after the concrete minimum-compiler failure | Reject this configuration |
| `0.10.0`, all optional features disabled | Eight Rust probes pass | Same eight pass | Viable restricted LP fallback; not selected |
| `0.11.1`, only explicit `serde` enabled | Eight Rust probes pass | Same eight pass | Selected |

The `0.11.1` minimal-build failure occurs in upstream
`src/solver/implementations/default/solver.rs`, before any probe executes. We
retain the released source and enable the declared feature, rather than patching
or claiming a passing build with that feature absent.

The `0.10.0` comparison also received source review. The later fix for
[issue 171](https://github.com/oxfordcontrol/Clarabel.rs/issues/171) changes the
nonnegative-cone barrier sign. In `0.10.0`, the solver calls that barrier only
for asymmetric cone collections, so the proposed zero/nonnegative LP path does
not enter it. The [issue 187](https://github.com/oxfordcontrol/Clarabel.rs/issues/187)
offset defect is also inactive at the solver's actual zero-offset call sites.
However, the [improved norm calculations](https://github.com/oxfordcontrol/Clarabel.rs/pull/184)
in `0.11` matter to overflow/underflow robustness of LP residuals, and
[constructor validation](https://github.com/oxfordcontrol/Clarabel.rs/pull/185)
adds a fallible setup API. Those changes justify the explicitly recorded
serialization dependencies of `0.11.1` rather than selecting the older backend.
Neither version removes the need for independent original-unit admission checks.

## Locked native dependency closure

The selected isolated probe and the provider feature resolve the same 35
external normal/build packages for `x86_64-unknown-linux-gnu`. Build-time
procedural macros are included. Cargo's target-specific WebAssembly packages
also occur in the all-target lockfile; they are not a native runtime dependency
and this decision does not qualify WebAssembly execution.

| Packages | Locked versions |
|---|---|
| `clarabel` | `0.11.1` |
| `amd` | `0.2.2` |
| `autocfg` | `1.5.1` |
| `cfg-if` | `1.0.5` |
| `darling`, `darling_core`, `darling_macro` | `0.14.4` each |
| `derive_builder`, `derive_builder_core`, `derive_builder_macro` | `0.11.2` each |
| `either` | `1.19.0` |
| `enum_dispatch` | `0.3.13` |
| `fnv` | `1.0.7` |
| `ident_case` | `1.0.1` |
| `itertools` | `0.11.0` |
| `itoa` | `1.0.19` |
| `lazy_static` | `1.5.1` |
| `memchr` | `2.8.3` |
| `num-traits` | `0.2.19` |
| `once_cell` | `1.21.4` |
| `proc-macro2` | `1.0.107` |
| `quote` | `1.0.47` |
| `serde`, `serde_core`, `serde_derive` | `1.0.229` each |
| `serde-big-array` | `0.5.1` |
| `serde_json` | `1.0.154` |
| `strsim` | `0.10.0` |
| `syn` | `1.0.109`, `2.0.119`, `3.0.10` |
| `thiserror`, `thiserror-impl` | `1.0.69` each |
| `unicode-ident` | `1.0.27` |
| `zmij` | `1.0.23` |

The highest declared Rust requirement among these resolved packages is 1.71;
some packages do not declare one. Actual Rust 1.75 compilation and execution
qualify this lock snapshot. A later resolution is a new compatibility input,
even while the direct Clarabel version remains pinned. No dependency upgrade
to the previously incompatible `thiserror 2` closure was made.

Metadata identifies Apache-2.0 for Clarabel, BSD-3-Clause for `amd`, and MIT or
MIT/Apache alternatives for the other packages; `unicode-ident` additionally
requires Unicode-3.0. `memchr` offers Unlicense or MIT. These notices remain
owned by the dependencies; no upstream source was copied into production.
The native test executable links the platform C, math and compiler-support
libraries and has no Python, BLAS or LAPACK shared-library dependency. This is
an observed Linux build result, not a cross-platform linking guarantee.

## Public backend API used by the probes

The pinned [solver constructor](https://github.com/oxfordcontrol/Clarabel.rs/blob/25540f559592068d0c8a80e46ded1b21760212a1/src/solver/implementations/default/solver.rs)
accepts `CscMatrix<f64>`, vectors, cone descriptors and `DefaultSettings`, and
returns a `Result`. `IPSolver::solve` runs the native solver. Results come from
`solver.solution`, which unwinds scaling and presolve; internal `variables`
must not be presented as original-unit solutions. The
[solution fields](https://github.com/oxfordcontrol/Clarabel.rs/blob/25540f559592068d0c8a80e46ded1b21760212a1/src/solver/implementations/default/solution.rs)
include primal `x`, dual `z`, slack `s`, termination status, both objective
values, iterations, elapsed time and primal/dual residuals.

For a joint vector `x` in row-major order and caller-supplied objective `c`,
the probes maximise `c^T x` using `P = 0` and `q = -c`. The constraint matrix
contains a unit-mass equality, `-I` for nonnegativity, and each ordered pair of
distinct recommended/deviation actions. A row-player deviation has coefficient
`A[k][j] - A[i][j]` in coordinate `(i,j)`; column deviations use
`B[i][l] - B[i][j]`. These deviation rows have upper bound zero. A single
nonnegative cone represents all inequalities. No product of marginals replaces
the joint distribution. The examples' objective is explicitly supplied; they
do not establish a welfare default.

## Executed controls and results

The retained target is
`crates/paignia-solve/tests/clarabel_backend.rs`. Its exact rational cases and
objective bounds come from the unchanged
[`correlated_equilibrium_v1.json`](../../verification/numerical_oracles/correlated_equilibrium_v1.json),
SHA-256 `1271809d3bf6a64c04d48456d42b87448f8c884cc159d0125ec68fc177a4f9ac`.
That original fixture supplies analytic primal/dual certificates, not output
from this backend. The probe builds the LP and independently recomputes mass,
negative-coordinate magnitude, incentive gains and objective from returned
coordinates. These small-case helper sums are not a production arithmetic
contract for arbitrary payoff scales.

The tests-first command fails with exit 101 because the baseline package has
no `correlated-equilibrium` feature. After the dependency change, the explicit
feature target runs eight tests on each compiler: Rust 1.75.0
(`82e1608df`) and stable Rust 1.100.0 (`0ef1bc386`), on native Linux x86-64.

```sh
cargo test --locked -p paignia-solve --features correlated-equilibrium --test clarabel_backend -- --nocapture
```

The target requires that feature. A default-feature build does not execute its
tests and is not recorded as an eight-test pass. Four tests solve independently
certified optima; three reject analytically incorrect substitutions; one proves
that a zero iteration budget yields `MaxIterations`, not a solution.

| Positive case | Exact objective | Observed objective | Largest negative coordinate magnitude | Maximum weighted deviation gain |
|---|---:|---:|---:|---:|
| Correlation welfare gap | `10.5` | `10.500000000025295` | `1.80021e-12` | `4.26087e-12` |
| Dominant actions / unused recommendations | `4` | `4.000000000001184` | `9.37720e-13` | `0` |
| Rectangular 2 by 3 | `0` | `4.89044e-13` | `1.22261e-13` | `1.51101e-13` |
| Negative single action | `-7` | `-7` | `0` | `0` |

Both compilers produced these values. Positive tests require `Solved`, finite
coordinates, mass and nonnegativity residuals below `1e-8`, weighted incentive
gain below `1e-8`, agreement with the independent objective bound, and backend
primal/dual residuals and objective gap below `1e-8`. Solver tolerances are
`1e-10`. The controls distinguish the feasible but suboptimal Nash product
(`28/3`), the product of CE marginals with gain `3/16`, and the infeasible
objective-12 point with gain `1`. They establish backend feasibility for these
bounded cases, not a general CE acceptance policy.

The provider workspace at this dependency slice also passes 140 native tests
with `--workspace --all-targets --all-features --locked` on each compiler.
The resolved-dependency guard passes seven isolated feature profiles. Direct
metadata comparisons on both compilers confirm six external packages in core
and in the solver's minimal/default closure, and the 35-package closure above
when the optional backend is enabled. These results include existing provider
controls; they are not 140 new CE cases.

## Required method boundary after this dependency slice

A later production implementation must own its result type and algorithm
version, rather than exporting Clarabel types as Paignia's public contract.
Complete rectangular payoff tables, explicit objective coordinates, dimensions,
finite values, budgets and every tolerance need checked admission. Payoffs
remain in each player's original units; objective units come from the caller.
The finite graph result profile does not implicitly admit a bimatrix result.

Raw backend coordinates include tiny negative values in the successful cases
above. They must remain visible in an explicitly approximate result with
nonnegativity and mass residuals. Silent clipping, renormalisation or conversion
into an exact probability-domain certificate is prohibited. Recommendation-
weighted and conditional incentive gains are distinct; a zero-mass
recommendation has no conditional ratio.

Production acceptance must separately recompute original-unit primal validity,
each incentive constraint, dual cone membership, dual stationarity and objective
gap from returned variables. Backend residuals alone do not establish those
properties. Sums and products need a reviewed cancellation/overflow/underflow
policy: a vanishing naive sum must not certify a positive deviation as zero.
Arithmetic that cannot support the declared tolerance fails explicitly.

All backend statuses remain distinguishable. `AlmostSolved` is reduced accuracy;
budget exhaustion, insufficient progress and numerical errors are not accepted
equilibria. Infeasibility statuses must not imply that a finite complete game
has no correlated equilibrium. Constructor errors and non-finite diagnostics
also fail explicitly. An independently reviewed production target, frozen
negative controls and method evidence are still required before advertising S03
as supported. Existing package smoke or these direct-backend tests cannot supply
that method receipt or either external application receipt.
