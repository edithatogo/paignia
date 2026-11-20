# Exact leader-follower inspection family — T18.01

## Supported input and method

`inspection_simultaneous_nash`, `inspection_commitment_at`, and
`inspection_commitment_bound` admit one binary family. The leader's rows are
inspect and skip. The follower's columns are comply and evade. With strictly
positive `c`, `h`, `g`, and `f`, the cells are `(-c, 0)`, `(-c, -f)`, `(0, 0)`,
and `(-h, g)`.

Detection is perfect when the leader inspects. The follower observes the
committed inspection probability and does not observe the later private draw.
An expected-effort budget `B` in `[0, 1]` limits that probability. It does not
forbid a realised inspection when `B < 1`.

At follower indifference, optimistic selection maximises the leader's payoff and
pessimistic selection minimises it. The returned commitment is attained only
when a feasible probability reaches the value. A one-sided threshold limit that
no feasible point reaches is a supremum and carries no strategy witness.

Simultaneous Nash is a separate witness. When `c < h` it is the unique interior
profile `p = g/(g+f)`, `q = c/h`. When `c > h`, skip is strictly dominant and
the unique profile is `(0, 1)`. The infeasible value `q = c/h > 1` is rejected
rather than clipped. When `c = h`, every inspection probability up to the
follower's threshold can be a best response, so the method returns
`LeaderIndifferent` instead of an arbitrary witness.

Each exact arithmetic operation stores its reduced result as a signed `i64`
numerator and a positive `i64` denominator. Subtraction forms the difference
of cross-products in `i128` before reduction. This allows a payoff with
numerator `i64::MIN` to produce a representable deviation gain. A reduced
result outside the storage range returns `Overflow`.

A zero denominator, a nonpositive parameter,
a probability outside `[0, 1]`, and a commitment above the budget are rejected.
No payoff is replaced by zero.

## External check that is pinned

The two Nash witnesses and fourteen commitment certificates are the synthetic
literals in
[`verification/inspection_oracles/inspection_2x2_v1.json`](../../verification/inspection_oracles/inspection_2x2_v1.json).
They were not produced by this solver. The focused test also evaluates the closed forms

```text
U_I(p,q) = -c*p - h*(1-p)*q
U_E(p,q) = q*(g - (g+f)*p)
```

on the four pure profiles and on the unattained approach `p = 2/3`. That path
does not call the four-cell sum.

## What this slice does not establish

- This is not a general Stackelberg algorithm, a multi-channel model, or an
  imperfect-detection model.
- It does not encode the game in the perfect-information extensive-form profile.
  Revealing the realised inspection before the follower acts is a different game.
- Parameter intervals, credal bounds, and solver-residual certificates belong
  to T18.02. The `tobac` and `voiage` handoff belongs to T18.03 and is external.
- Agreement with these rationals does not close T18.01. The synthetic oracle
  still requires its own specialist review, and this native method is not that
  review.
- No operational, welfare, or empirical claim follows from the constants.

```bash
cargo test --locked -p paignia-solve --test leader_follower
```
