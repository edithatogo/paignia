"""Test-only exact arithmetic for the declared T18 inspection family.

Tests-first placeholders. This module is not a production API or solver.
"""


def rational(value):
    raise NotImplementedError("tests-first rational parser")


def probability(value):
    raise NotImplementedError("tests-first exact domain check")


class Game:
    @classmethod
    def from_mapping(cls, values):
        raise NotImplementedError("tests-first fixed family")


def expected_payoffs(game, p, q):
    raise NotImplementedError("tests-first expectation")


def nash_gains(game, p, q):
    raise NotImplementedError("tests-first witness check")


def follower_best_responses(game, p):
    raise NotImplementedError("tests-first best responses")


def check_follower_response(game, p, q, budget):
    raise NotImplementedError("tests-first response check")


def commitment_at(game, p, budget, tie):
    raise NotImplementedError("tests-first commitment evaluation")


def commitment_bound(game, budget, tie):
    raise NotImplementedError("tests-first affine bound")
