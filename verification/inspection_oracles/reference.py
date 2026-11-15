"""Test-only exact arithmetic for the declared T18 inspection family.

The two matrices, positive parameters and one expected-effort constraint are
fixed by README.md. No production import, optimizer, float, RNG or runtime
fallback is involved. Bound witnesses are not a complete solution catalogue.
"""

import re
from dataclasses import dataclass
from fractions import Fraction


ZERO = Fraction(0)
ONE = Fraction(1)


def rational(value):
    """Admit canonical rational test literals without binary64 conversion."""
    if not isinstance(value, str) or not re.fullmatch(
        r"-?(0|[1-9][0-9]*)(/[1-9][0-9]*)?", value
    ):
        raise ValueError("expected a canonical rational string")
    result = Fraction(value)
    if str(result) != value:
        raise ValueError("expected a reduced canonical rational")
    return result


def probability(value):
    if not isinstance(value, Fraction) or not ZERO <= value <= ONE:
        raise ValueError("expected an exact probability in [0, 1]")
    return value


@dataclass(frozen=True)
class Game:
    """The fixed inspection family; c, h, g and f are strictly positive."""

    c: Fraction
    h: Fraction
    g: Fraction
    f: Fraction

    def __post_init__(self):
        for value in (self.c, self.h, self.g, self.f):
            if not isinstance(value, Fraction) or value <= ZERO:
                raise ValueError("expected strictly positive exact parameters")

    @classmethod
    def from_mapping(cls, values):
        names = ("inspection_cost", "undetected_harm", "evasion_gain", "detected_loss")
        if not isinstance(values, dict) or set(values) != set(names):
            raise ValueError("expected exactly the four named inspection parameters")
        return cls(*(rational(values[name]) for name in names))

    def matrices(self):
        # Rows inspect/skip; columns comply/evade. No sanction transfer to leader.
        return (
            ((-self.c, -self.c), (ZERO, -self.h)),
            ((ZERO, -self.f), (ZERO, self.g)),
        )


def expected_payoffs(game, p, q):
    probability(p)
    probability(q)
    rows, columns = (p, ONE - p), (ONE - q, q)
    return tuple(
        sum(
            (rows[i] * columns[j] * matrix[i][j] for i in range(2) for j in range(2)),
            ZERO,
        )
        for matrix in game.matrices()
    )


def nash_gains(game, p, q):
    """Check a supplied simultaneous witness; do not search for equilibria."""
    leader, follower = expected_payoffs(game, p, q)
    return (
        max(expected_payoffs(game, pure, q)[0] for pure in (ZERO, ONE)) - leader,
        max(expected_payoffs(game, p, pure)[1] for pure in (ZERO, ONE)) - follower,
    )


def follower_best_responses(game, p):
    utilities = {
        "comply": expected_payoffs(game, p, ZERO)[1],
        "evade": expected_payoffs(game, p, ONE)[1],
    }
    best = max(utilities.values())
    return tuple(action for action, utility in utilities.items() if utility == best)


def feasible_commitment(p, budget):
    probability(p)
    probability(budget)
    if p > budget:
        raise ValueError("commitment exceeds expected-effort budget")


def check_follower_response(game, p, q, budget):
    """A mixed follower response may put weight only on best responses."""
    feasible_commitment(p, budget)
    probability(q)
    best = follower_best_responses(game, p)
    if (q < ONE and "comply" not in best) or (q > ZERO and "evade" not in best):
        raise ValueError("follower support contains a non-best response")
    return expected_payoffs(game, p, q)


def require_tie(tie):
    if tie not in ("optimistic", "pessimistic"):
        raise ValueError("unknown follower tie policy")


def commitment_at(game, p, budget, tie):
    """Return q, leader payoff, follower payoff at one feasible commitment."""
    require_tie(tie)
    feasible_commitment(p, budget)
    options = []
    for action in follower_best_responses(game, p):
        q = ZERO if action == "comply" else ONE
        options.append((q, *expected_payoffs(game, p, q)))
    choose = max if tie == "optimistic" else min
    # Selection is by leader utility among follower best responses, never name.
    return choose(options, key=lambda option: option[1])


@dataclass(frozen=True)
class Bound:
    """Test-local bound: p/q are absent when only a supremum exists."""

    value: Fraction
    attained: bool
    p: Fraction | None
    q: Fraction | None


def commitment_bound(game, budget, tie):
    """Exact affine envelope for this family, including feasible open limits.

    Follower utility difference is g - (g + f)*p. Its only breakpoint is
    theta = g/(g+f). On either open side, leader utility is affine, so endpoints
    and one-sided limits bound the whole interval; no probability grid is used.
    """
    require_tie(tie)
    probability(budget)
    theta = game.g / (game.g + game.f)
    points = {ZERO, budget}
    if theta <= budget:
        points.add(theta)
    attained = [(p, commitment_at(game, p, budget, tie)) for p in sorted(points)]
    values = [outcome[1] for _, outcome in attained]
    if theta <= budget:
        # Evasion is uniquely best for p < theta. theta > 0 follows from g,f > 0.
        values.append(expected_payoffs(game, theta, ONE)[0])
    if theta < budget:
        # Compliance is uniquely best for p > theta. Strict feasibility matters:
        # when budget == theta there is no right-hand approach in the domain.
        values.append(expected_payoffs(game, theta, ZERO)[0])
    upper = max(values)
    for p, outcome in attained:
        if outcome[1] == upper:
            return Bound(upper, True, p, outcome[0])
    return Bound(upper, False, None, None)
