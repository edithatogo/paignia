#!/usr/bin/env python3
"""Offline synthetic reference tests; never import Paignia production code."""

import copy
import json
from fractions import Fraction as F
from pathlib import Path
import unittest

from reference import (
    Game,
    check_follower_response,
    commitment_at,
    commitment_bound,
    expected_payoffs,
    follower_best_responses,
    nash_gains,
    probability,
    rational,
)


def no_duplicate_keys(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate fixture key: {key}")
        result[key] = value
    return result


def exact_values(values):
    return tuple(rational(value) for value in values)


class InspectionOracleTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        path = Path(__file__).with_name("inspection_2x2_v1.json")
        cls.data = json.loads(path.read_text(), object_pairs_hook=no_duplicate_keys)
        cls.cases = {case["id"]: case for case in cls.data["cases"]}

    def game(self, case_id="interior"):
        return Game.from_mapping(self.cases[case_id]["parameters"])

    def test_fixture_identity_and_synthetic_provenance(self):
        self.assertEqual(self.data["format"], "t18.01.synthetic.2x2.v1")
        self.assertEqual(len(self.data["cases"]), 2)
        self.assertEqual(set(self.cases), {"interior", "costly"})
        source = self.data["provenance"]
        self.assertEqual(source["origin"], "original_synthetic")
        self.assertEqual(source["parameter_source"], "declared_test_constants")
        self.assertEqual(source["units"], "arbitrary_signed_utility")
        self.assertIs(source["empirical_records"], False)
        self.assertEqual(source["base_revision"], "fca2b2f8811c8cd56d27be4c9407f573d54ad1e9")
        self.assertEqual(source["t18_spec_blob"], "b1805340504c5601c3cbbe5c51be1853d7eae630")
        self.assertEqual(source["t18_plan_blob"], "5e6f95b0274ca36da6ec7e0f3d9363245629fbec")

    def test_information_effort_and_payoff_assumptions_are_explicit(self):
        self.assertEqual(self.data["row_actions"], ["inspect", "skip"])
        self.assertEqual(self.data["column_actions"], ["comply", "evade"])
        self.assertEqual(self.data["assumptions"], {
            "p": "probability_of_inspect",
            "q": "probability_of_evade",
            "commitment_observation": "distribution_only_before_private_action_draw",
            "effort_constraint": "expected_effort_p_at_most_budget",
            "detection_given_inspection": "perfect",
            "sanction_received_by_inspector": False,
            "utility": "risk_neutral_expected_payoff",
            "parameter_uncertainty": "none",
            "tie_policies": ["optimistic", "pessimistic"],
            "arithmetic": "exact_rational_zero_tolerance",
        })

    def test_rational_literals_are_canonical_and_exact(self):
        for value, expected in [("0", F(0)), ("-3", F(-3)), ("2/7", F(2, 7))]:
            with self.subTest(value=value):
                self.assertEqual(rational(value), expected)
        self.assertEqual(rational("-1/3") + rational("1/3"), F(0))

    def test_inexact_or_malformed_numbers_reject(self):
        bad = [True, 1, 0.5, None, "1.0", "1e0", "NaN", "Infinity", "1/0",
               "2/4", "+1", "01", "-0", " 1", "1\n", "1/-2", ""]
        for value in bad:
            with self.subTest(value=value), self.assertRaises(ValueError):
                rational(value)

    def test_probability_domain_rejects_nonexact_or_outside_unit_interval(self):
        for value in [True, 0, 0.5, "1/2", F(-1, 10), F(11, 10)]:
            with self.subTest(value=value), self.assertRaises(ValueError):
                probability(value)
        for value in [F(0), F(1, 3), F(1)]:
            self.assertEqual(probability(value), value)

    def test_nonpositive_and_missing_parameters_reject(self):
        params = self.cases["interior"]["parameters"]
        for name in params:
            for value in ["0", "-1", 0.5]:
                bad = dict(params, **{name: value})
                with self.subTest(name=name, value=value), self.assertRaises(ValueError):
                    Game.from_mapping(bad)
            bad = dict(params)
            del bad[name]
            with self.subTest(missing=name), self.assertRaises(ValueError):
                Game.from_mapping(bad)
        with self.assertRaises(ValueError):
            Game.from_mapping(dict(params, unknown="1"))

    def test_named_parameter_order_does_not_change_math(self):
        params = self.cases["interior"]["parameters"]
        other = Game.from_mapping(dict(reversed(list(params.items()))))
        self.assertEqual(self.game(), other)

    def test_payoff_matrices_match_independent_literals(self):
        for case_id, case in self.cases.items():
            with self.subTest(case=case_id):
                expected = tuple(tuple(exact_values(row) for row in case[key])
                                 for key in ["leader_matrix", "follower_matrix"])
                self.assertEqual(self.game(case_id).matrices(), expected)
        self.assertEqual(self.game().matrices()[0][0], (F(-1), F(-1)))

    def test_all_four_pure_profiles_have_exact_payoffs_and_gains(self):
        for case_id, case in self.cases.items():
            pairs = {(row["p"], row["q"]) for row in case["pure_profiles"]}
            self.assertEqual(pairs, {("0", "0"), ("0", "1"), ("1", "0"), ("1", "1")})
            self.assertEqual(len(case["pure_profiles"]), 4)
            for row in case["pure_profiles"]:
                with self.subTest(case=case_id, row=row):
                    p, q = rational(row["p"]), rational(row["q"])
                    self.assertEqual(expected_payoffs(self.game(case_id), p, q), exact_values(row["payoffs"]))
                    self.assertEqual(nash_gains(self.game(case_id), p, q), exact_values(row["gains"]))

    def test_known_nash_witnesses_have_zero_deviation_gains(self):
        for case_id, case in self.cases.items():
            row = case["nash"]
            p, q = rational(row["p"]), rational(row["q"])
            with self.subTest(case=case_id):
                self.assertEqual(expected_payoffs(self.game(case_id), p, q), exact_values(row["payoffs"]))
                self.assertEqual(nash_gains(self.game(case_id), p, q), (F(0), F(0)))

    def test_mixed_witness_indifference_and_pure_exclusion(self):
        game = self.game()
        self.assertEqual(game.g / (game.g + game.f), F(1, 3))
        self.assertEqual(game.c / game.h, F(1, 4))
        self.assertEqual(expected_payoffs(game, F(0), F(1, 4))[0], F(-1))
        self.assertEqual(expected_payoffs(game, F(1), F(1, 4))[0], F(-1))
        self.assertEqual(expected_payoffs(game, F(1, 3), F(0))[1], F(0))
        self.assertEqual(expected_payoffs(game, F(1, 3), F(1))[1], F(0))
        for row in self.cases["interior"]["pure_profiles"]:
            self.assertGreater(max(exact_values(row["gains"])), F(0))

    def test_high_cost_case_has_strict_pure_nash_without_invalid_mixing(self):
        game = self.game("costly")
        self.assertGreater(game.c, game.h)
        self.assertEqual(game.c / game.h, F(5, 4))
        for q in [F(0), F(1, 2), F(1)]:
            self.assertGreater(expected_payoffs(game, F(0), q)[0], expected_payoffs(game, F(1), q)[0])
        with self.assertRaises(ValueError):
            nash_gains(game, F(1, 3), F(5, 4))

    def test_false_nash_candidates_are_distinguished(self):
        self.assertEqual([len(case["false_nash"]) for case in self.data["cases"]], [2, 1])
        for case_id, case in self.cases.items():
            for row in case["false_nash"]:
                with self.subTest(case=case_id, row=row):
                    p, q = rational(row["p"]), rational(row["q"])
                    gains = nash_gains(self.game(case_id), p, q)
                    self.assertEqual(gains, exact_values(row["gains"]))
                    self.assertGreater(max(gains), F(0))
                    self.assertEqual(expected_payoffs(self.game(case_id), p, q), exact_values(row["payoffs"]))

    def test_follower_best_responses_below_at_above_threshold(self):
        self.assertEqual(len(self.data["best_response_probes"]), 5)
        for case_id, case in self.cases.items():
            self.assertEqual(rational(case["threshold"]), F(1, 3))
            for row in self.data["best_response_probes"]:
                with self.subTest(case=case_id, p=row["p"]):
                    self.assertEqual(follower_best_responses(self.game(case_id), rational(row["p"])), tuple(row["best"]))

    def test_tie_retains_both_responses_and_mixtures(self):
        game = self.game()
        for q in [F(0), F(1, 4), F(3, 4), F(1)]:
            leader, follower = check_follower_response(game, F(1, 3), q, F(1))
            self.assertEqual(follower, F(0))
            self.assertEqual(leader, -F(1, 3) - F(8, 3) * q)

    def test_nonbest_follower_response_is_rejected(self):
        for p, q in [(F(1, 4), F(0)), (F(1, 2), F(1)), (F(1, 4), F(1, 2))]:
            with self.subTest(p=p, q=q), self.assertRaisesRegex(ValueError, "best response"):
                check_follower_response(self.game(), p, q, F(1))

    def test_all_commitment_bounds_match_literal_certificates(self):
        for case_id, case in self.cases.items():
            budgets = {"0", "1/4", "1/3", "1"} if case_id == "interior" else {"1/4", "1/3", "1"}
            expected_keys = {(b, t) for b in budgets for t in ["optimistic", "pessimistic"]}
            self.assertEqual({(row["budget"], row["tie"]) for row in case["commitment"]}, expected_keys)
            self.assertEqual(len(case["commitment"]), len(expected_keys))
            for row in case["commitment"]:
                with self.subTest(case=case_id, budget=row["budget"], tie=row["tie"]):
                    game, budget = self.game(case_id), rational(row["budget"])
                    bound = commitment_bound(game, budget, row["tie"])
                    self.assertEqual(bound.value, rational(row["value"]))
                    self.assertIs(bound.attained, row["attained"])
                    self.assertEqual(bound.p, None if row["p"] is None else rational(row["p"]))
                    self.assertEqual(bound.q, None if row["q"] is None else rational(row["q"]))
                    if bound.attained:
                        q, leader, follower = commitment_at(game, bound.p, budget, row["tie"])
                        self.assertEqual((q, leader, follower), (bound.q, bound.value, rational(row["follower_payoff"])))
                        self.assertEqual(check_follower_response(game, bound.p, bound.q, budget), (leader, follower))
                    else:
                        self.assertIsNone(row["follower_payoff"])

    def test_optimistic_commitment_improves_on_simultaneous_value(self):
        for case_id, expected_gain in [("interior", F(2, 3)), ("costly", F(7, 3))]:
            case, game = self.cases[case_id], self.game(case_id)
            bound = commitment_bound(game, F(1), "optimistic")
            self.assertEqual(bound.value - rational(case["nash"]["payoffs"][0]), expected_gain)

    def test_optimistic_tie_is_not_pessimistic_or_a_nash_profile(self):
        game = self.game()
        self.assertEqual(commitment_at(game, F(1, 3), F(1), "optimistic"), (F(0), F(-1, 3), F(0)))
        self.assertEqual(commitment_at(game, F(1, 3), F(1), "pessimistic"), (F(1), F(-3), F(0)))
        self.assertEqual(nash_gains(game, F(1, 3), F(0)), (F(1, 3), F(0)))

    def test_pessimistic_full_budget_supremum_has_no_attaining_witness(self):
        for case_id in self.cases:
            game = self.game(case_id)
            bound = commitment_bound(game, F(1), "pessimistic")
            self.assertFalse(bound.attained)
            self.assertEqual((bound.p, bound.q), (None, None))
            self.assertEqual(bound.value, -game.c / 3)
            for k in [1, 2, 4, 8]:
                p = F(1, 3) + F(2, 3 * 2**k)
                q, value, _ = commitment_at(game, p, F(1), "pessimistic")
                self.assertEqual(q, F(0))
                self.assertEqual(bound.value - value, game.c * F(2, 3 * 2**k))
                self.assertLess(value, bound.value)

    def test_exact_budget_boundary_cannot_use_infeasible_right_limit(self):
        for case_id, value, p in [("interior", F(-3), F(1, 3)), ("costly", F(-4), F(0))]:
            bound = commitment_bound(self.game(case_id), F(1, 3), "pessimistic")
            self.assertEqual((bound.value, bound.attained, bound.p, bound.q), (value, True, p, F(1)))
            with self.assertRaisesRegex(ValueError, "budget"):
                commitment_at(self.game(case_id), F(1, 3) + F(1, 1000), F(1, 3), "pessimistic")

    def test_subthreshold_budget_can_be_used_or_left_unused(self):
        cheap = commitment_bound(self.game(), F(1, 4), "optimistic")
        costly = commitment_bound(self.game("costly"), F(1, 4), "optimistic")
        self.assertEqual((cheap.p, cheap.value), (F(1, 4), F(-13, 4)))
        self.assertEqual((costly.p, costly.value), (F(0), F(-4)))

    def test_zero_budget_is_feasible_and_signed(self):
        for case_id in self.cases:
            for tie in ["optimistic", "pessimistic"]:
                bound = commitment_bound(self.game(case_id), F(0), tie)
                self.assertEqual((bound.p, bound.q, bound.value, bound.attained), (F(0), F(1), F(-4), True))

    def test_unknown_ties_reject_instead_of_falling_back(self):
        for tie in [None, "", "lexical", "Optimistic", "optimistic "]:
            with self.subTest(tie=tie):
                with self.assertRaisesRegex(ValueError, "tie"):
                    commitment_at(self.game(), F(1, 3), F(1), tie)
                with self.assertRaisesRegex(ValueError, "tie"):
                    commitment_bound(self.game(), F(1), tie)

    def test_invalid_budget_and_infeasible_strategy_reject(self):
        for budget in [F(-1, 4), F(5, 4), 0.5, True]:
            with self.subTest(budget=budget), self.assertRaises(ValueError):
                commitment_bound(self.game(), budget, "optimistic")
        with self.assertRaisesRegex(ValueError, "budget"):
            check_follower_response(self.game(), F(1, 2), F(0), F(1, 4))
        for p, q in [(F(-1), F(0)), (F(0), F(2)), (0.5, F(0))]:
            with self.subTest(p=p, q=q), self.assertRaises(ValueError):
                expected_payoffs(self.game(), p, q)

    def test_pure_only_or_grid_shortcuts_miss_exact_commitment_value(self):
        for case_id in self.cases:
            game = self.game(case_id)
            optimum = commitment_bound(game, F(1), "optimistic").value
            endpoints = max(commitment_at(game, p, F(1), "optimistic")[1] for p in [F(0), F(1)])
            grid = max(commitment_at(game, F(k, 16), F(1), "optimistic")[1] for k in range(17))
            self.assertLess(endpoints, optimum)
            self.assertLess(grid, optimum)

    def test_revealing_realized_action_changes_the_information_game(self):
        game = self.game()
        # After observing the draw, the follower complies at inspect and evades at skip.
        revealed = F(1, 3) * expected_payoffs(game, F(1), F(0))[0]
        revealed += F(2, 3) * expected_payoffs(game, F(0), F(1))[0]
        self.assertEqual(revealed, F(-3))
        self.assertNotEqual(revealed, commitment_at(game, F(1, 3), F(1), "optimistic")[1])
        self.assertEqual(max(F(-1), F(-4)), F(-1))

    def test_duplicate_fixture_keys_are_not_silently_overwritten(self):
        with self.assertRaisesRegex(ValueError, "duplicate"):
            json.loads('{"p":"0","p":"1"}', object_pairs_hook=no_duplicate_keys)
        self.assertEqual(copy.deepcopy(self.data), self.data)


if __name__ == "__main__":
    unittest.main(verbosity=2)
