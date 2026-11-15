# T18.01 synthetic inspection oracle

Status: independently specified fixture implemented; qualification pending. This directory is a
test-only analytical reference, not a production solver, runtime fallback,
public contract or scientific/consumer qualification receipt.

## Scope and ownership

Dylan authorised this bounded lane on 16 November 2026. Its
[pre-implementation claim](https://github.com/edithatogo/paignia/pull/14#issuecomment-6896113657)
reserves only these four new files: `README.md`, `inspection_2x2_v1.json`,
`reference.py` and `test_inspection.py`. Coordinator
`6af93fe8-b14c-83ec-8de0-728c856bcbb3` retains shared integration. Its
[ownership receipt](https://github.com/edithatogo/paignia/pull/14#issuecomment-6896175943)
acknowledges this lane as worker `6af9c112-7efc-83ec-8798-c10017de8c5f`.

The reference implements the independent analytical part of
[T18.01](../../conductor/tracks/t18_illicit_enforcement_inspection_games_20261110/plan.md).
The live base is `fca2b2f8811c8cd56d27be4c9407f573d54ad1e9`.
The T18 specification and plan blobs are respectively
`b1805340504c5601c3cbbe5c51be1853d7eae630` and
`5e6f95b0274ca36da6ec7e0f3d9363245629fbec`.
All 13 published branch trees, current open PR file lists/discussions, T18
registration PR5 and the coordinator ownership record were checked. No
published intersecting fixture claim was found. This observation does not
assert direct acknowledgement from other live sessions.

PR15 at `af7b9783e63943cce3988873160166ca0adfc73f` retains its numerical-oracle
namespace and separate independent reviewer. No source or review is duplicated
there. Only this directory changes; shared Conductor, production Rust, APIs,
manifests, lockfiles, CI, registries and consumer pins stay with their owners.

## Local increment

- [~] T18.01.oracle: specify exact synthetic 2x2 inspection payoffs, pure/mixed
  best responses and commitment values; test ties, effort constraints,
  supremum attainment and distinguishing invalid shortcuts.
- [ ] Independent specialist numerical review and coordinator acceptance.

These are local fixture milestones, not a second parent-track status ledger.
T18.01 production integration remains open. T18.02 interval/credal bounds and
residual guarantees, and T18.03 consumer integration, are outside this slice.

## Prerequisites retained

Full T17.01 is incomplete. Its result/identity/algorithm/status/residual,
information-structure, selection and catalogue guarantees, plus appropriate
production tolerances, must be accepted and pinned before native comparisons.
The candidate input contract at PR14
`0c88607a142508d1fe4f1153b963fd1411b49395` admits a finite perfect-information
extensive-form graph; it does not admit simultaneous play or commitment with
an unobserved random action. This reference does not encode either game as that
profile. T17.13 remains held. Other T17 scientific/package and real-consumer
obligations remain open. All runtime game execution stays Rust-owned.

No real records or empirical calibration are used. This is one synthetic
channel with binary effort and arbitrary signed utility units. Mathematical
fixture agreement cannot establish an operational or scientific claim.

## Declared model and information

The inspector's rows are **inspect**, **skip**. The counterparty's columns are
**comply**, **evade**. With strictly positive exact constants `c`, `h`, `g`, `f`,
the two payoff matrices are:

| Inspector action | Comply: (inspector, counterparty) | Evade: (inspector, counterparty) |
| --- | --- | --- |
| Inspect | `(-c, 0)` | `(-c, -f)` |
| Skip | `(0, 0)` | `(-h, g)` |

`c` is the cost of inspection; `h` is harm from undetected evasion; `g` is the
gain from undetected evasion; `f` is the net loss if detected. Detection is
perfect conditional on inspection, and prevents `h`. No sanction is credited
to the inspector. These actor payoffs are not an aggregate welfare measure or
an accounting assertion about real transfers. All parameters and expectations
are original synthetic constants, not estimated quantities.

Let `p = Pr(inspect)` and `q = Pr(evade)`.

- **Simultaneous model:** neither player observes the other's realised action
  before choosing. The two randomisations are independent in the supplied Nash
  witnesses. All probabilities are in `[0,1]`.
- **Commitment model:** the inspector credibly commits to `p`; the follower
  observes that distribution and chooses a best response before the inspector's
  private action is realised. The follower does not observe the draw. Effort is
  bounded by `p <= B`, with `B` in `[0,1]`. This is an expected-effort constraint,
  not a per-realisation cap that would forbid a full inspection when `B < 1`.
- At follower indifference, **optimistic** selection maximises inspector
  utility; **pessimistic** selection minimises it. Both conventions are analysed
  explicitly. Neither is silently assigned to a production result or real actor.

There is no parameter uncertainty, learning, time, RNG execution, hidden type,
multiple channel, imperfect detection or empirical inference in this model.
T18.02 needs a separate interval/credal specification and certificate.

## Original exact derivation

The matrix expectations are

```text
U_I(p,q) = -c*p - h*(1-p)*q
U_E(p,q) = q*(g - (g+f)*p).
```

Holding the other player's strategy fixed, inspect versus skip has utility
difference `h*q - c`; evade versus comply has difference `g - (g+f)*p`.
The independent best-response gains of a supplied simultaneous witness are

```text
R_I = max(-c, -h*q) - U_I(p,q)
R_E = max(0, g - (g+f)*p) - U_E(p,q).
```

Both gains must be exactly zero for a Nash witness. The harness calculates
expectations from the four matrix cells and deviations from pure alternatives;
these closed forms provide a second author calculation without importing the
reference module. No production solver is called in either calculation.

### Simultaneous cases

| Case | `(c,h,g,f)` | Nash `(p,q)` | Payoffs `(U_I,U_E)` |
| --- | --- | --- | --- |
| `interior` | `(1,4,1,2)` | `(1/3,1/4)` | `(-1,0)` |
| `costly` | `(5,4,1,2)` | `(0,1)` | `(-4,1)` |

For `interior`, indifference gives `p = g/(g+f) = 1/3` and `q = c/h = 1/4`.
All four pure profiles have a strictly profitable deviation by at least one
player. There cannot be a boundary mixed equilibrium: at pure inspection the
follower strictly complies, at pure skipping it strictly evades, and the
corresponding inspector response changes. Thus this case has a unique interior
Nash equilibrium.

For `costly`, inspect's advantage `4*q-5` is strictly negative for every feasible
`q`, so skip is strictly dominant. The follower then strictly evades, yielding
the unique pure equilibrium. The interior formula's `q=5/4` is infeasible and
must reject, not be clipped or normalised.

The fixture pins all eight pure-profile payoff/gain pairs, both Nash witnesses
and three false Nash candidates. In particular, the optimistic commitment
profile `(1/3,0)` is not Nash: without commitment the inspector can gain `c/3`
by skipping. The two solution concepts must not share a result label.

### Commitment and budget cases

The follower's only breakpoint is `theta = g/(g+f) = 1/3` in both cases:

| Commitment | Follower best responses | Inspector payoff |
| --- | --- | --- |
| `p < theta` | Evade only | `-h + (h-c)*p` |
| `p = theta` | Every mixture of comply and evade | Between `-h+(h-c)*theta` and `-c*theta` |
| `p > theta` | Comply only | `-c*p` |

At the threshold, compliance improves inspector utility by `h*(1-theta) > 0`.
It is selected only under optimistic ties; pessimistic ties select evasion.

| Case and budget | Optimistic value; attaining `p` | Pessimistic value; attainment |
| --- | --- | --- |
| `interior`, `B=1` | `-1/3`; `p=1/3` | Supremum `-1/3`; **not attained** |
| `interior`, `B=1/4` | `-13/4`; `p=1/4` | `-13/4`; attained at `p=1/4` |
| `interior`, `B=1/3` | `-1/3`; `p=1/3` | `-3`; attained at `p=1/3` |
| `interior`, `B=0` | `-4`; `p=0` | `-4`; attained at `p=0` |
| `costly`, `B=1` | `-5/3`; `p=1/3` | Supremum `-5/3`; **not attained** |
| `costly`, `B=1/4` | `-4`; `p=0` | `-4`; attained at `p=0` |
| `costly`, `B=1/3` | `-5/3`; `p=1/3` | `-4`; attained at `p=0` |

Each row has two literal certificates, fourteen in total. Values are signed:
negative utility is never silently replaced by zero.

For full-budget pessimistic `interior`, the best value on `p <= theta` is `-3`,
whereas every `p > theta` has value `-p < -1/3`. For `costly`, the best value on
`p <= theta` is `-4`, whereas every `p > theta` has value `-5*p < -5/3`.
The feasible sequence `p_k = 1/3 + (2/3)/2^k`, `k >= 1`, approaches the stated
supremum from below, with exact payoff gap `c*(2/3)/2^k`. Neither supremum is an
attained maximum. The fixture represents this by `attained=false` and absent
`p`, `q` and follower payoff, not by a fictitious equilibrium witness.

When `B=theta`, this right-hand sequence is infeasible. The pessimistic maximum
therefore falls to `-3` in `interior` and `-4` in `costly`. Below the threshold,
the evasion branch's slope is `h-c`: the cheaper inspector uses all the budget,
while the costly inspector optimally leaves it unused. These are analytic
controls for strict versus non-strict effort comparisons.

### Why the bound calculation covers the feasible continuum

`reference.py` evaluates actual endpoints `0`, `B` and the threshold if feasible,
then includes each feasible one-sided threshold limit. The payoff is affine on
each remaining interval, so these values bound every point in `[0,B]`. A left
approach exists when `theta <= B`; a right approach exists only when `theta < B`.
At indifference, utility is affine in the follower mixture, so the two pure
endpoints also bound every mixture. The largest candidate value is a maximum
only if an actual feasible point reaches it; otherwise it is a supremum.

The returned point is one witness, not an exhaustive strategy catalogue. This
proof applies only to the declared positive-parameter, binary, perfect-detection
family. It is not a general Stackelberg algorithm or a production certificate
format. Finite grid/probe tests are distinguishing controls, not a proof over
the continuum. Pure-only and sixteenth-grid searches miss these exact optima.

Revealing the realised inspection before the follower acts changes the game.
At `p=1/3` in `interior`, conditional compliance on inspection and evasion on
skipping gives `-3`, rather than optimistic hidden-draw commitment's `-1/3`.
A fully observed chance/decision tree must not be used as an equivalent model.

## Provenance, arithmetic and reproduction

All matrices, parameters and literal expected values above were constructed for
this fixture and derived algebraically here. No tables, implementation or data
were copied from PR15, production Rust, third-party software or real records.
The repository [Apache-2.0 notice](../../LICENSE) applies to this original work.
For the background definition of a leader committing to a strategy followed by
a best response, see von Stengel and Zamir (2010),
[Leadership games with convex strategy sets](https://doi.org/10.1016/j.geb.2009.11.008),
*Games and Economic Behavior* 69(2), 446-457. That conceptual reference is not
the source of these constants or an independent review of this implementation.

The committed command uses CPython 3.12.14 and only its standard library:

```bash
python3 -B verification/inspection_oracles/test_inspection.py
python3 -B scripts/validate_extraction.py
```

It runs 28 named tests, including subcases. JSON numeric values are canonical
reduced rational strings, converted directly to `Fraction`. All comparisons
have zero tolerance. Floats, booleans, non-finite/decimal text, noncanonical
fractions, missing/extra parameters, nonpositive parameters, invalid
probabilities/budgets, overspending and unknown tie rules reject explicitly.
This is mathematical reference-domain validation; it does not define Paignia
public errors or justify any binary64 production tolerance.

Tests-first commit `26c9165b2093997edc4b2d6368ca4518c5df704a` ran all 28 tests
and exited 1 with 100 errors including subtests, caused by explicit
`NotImplementedError` reference placeholders. Those were harness-development
errors, not production failures or passing tests. The implementation replaces
all placeholders; the current focused run passes all 28 tests without skips.

## Coordinator handoff requirements

1. Receive the final exact-head authoring/test packet and record accepted scope
   in coordinator-owned Conductor/ownership files. Parent T18.01 remains open.
2. Add the explicit harness command above to shared CI under a pinned/declared
   CPython 3.12.14 interpreter. The current native workflow does not execute it.
   No manifest, dependency, lockfile or production export change is required.
3. Obtain independent specialist numerical review of this exact fixture head.
   Author review and calculations independent of this module are still author
   evidence. PR15's reviewer is not assigned this directory; no worker is created.
4. Before any production comparison, complete/accept/pin the relevant T17.01
   contracts and method/information-structure requirements, including explicit
   tie semantics, nonattainment, signed values and a justified tolerance policy.
5. Keep T18.02/03, T17.13, package/consumer and operational gates separate. There
   is no inspection-record ingestion, adapter activation or scientific claim.

Rollback is a normal revert of the four additive files. Publication of the
draft PR or this handoff does not authorise merge, release or gate changes.
