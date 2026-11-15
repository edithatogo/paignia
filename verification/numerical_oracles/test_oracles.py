#!/usr/bin/env python3
"""Offline synthetic reference checks; never a production solver or fallback.

Run this file directly from any directory. Python's standard library is the only
dependency. Public Paignia representation/result/algorithm acceptance is outside
this harness: see README.md before interpreting a passing test.
"""

import json
from decimal import Decimal as D, localcontext, ROUND_HALF_EVEN
from fractions import Fraction as Q
from pathlib import Path
import unittest


FIXTURES = Path(__file__).resolve().parent


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate fixture key: {key}")
        result[key] = value
    return result


def reject_constant(value):
    raise ValueError(f"nonstandard JSON number: {value}")


def fixture(name, schema):
    with (FIXTURES / name).open(encoding="utf-8") as source:
        data = json.load(source, object_pairs_hook=unique_object,
                         parse_constant=reject_constant)
    if data["schema"] != schema or data["qualification"] != "synthetic_reference_only":
        raise ValueError("unsupported test fixture schema/qualification")
    return data


def rk4_step(population, time, step, fitness):
    """Reference-only classical four-stage update, pending implementation."""
    raise NotImplementedError("T17.04.oracle tests-first baseline")


def euler_step(population, time, step, fitness):
    """Named mathematical Euler negative control, pending implementation."""
    raise NotImplementedError("T17.04.oracle tests-first baseline")


def trajectory(population, start, end, count, fitness, stepper=rk4_step):
    raise NotImplementedError("T17.04.oracle tests-first baseline")


def logistic(initial, integrated_difference):
    raise NotImplementedError("T17.04.oracle tests-first baseline")


class ReplicatorOracleTests(unittest.TestCase):
    def setUp(self):
        self.context = localcontext()
        context = self.context.__enter__()
        context.prec = 80
        context.rounding = ROUND_HALF_EVEN
        self.addCleanup(self.context.__exit__, None, None, None)
        self.data = fixture("replicator_rk4_v1.json", "paignia.test-oracle.replicator/1")

    def test_fixture_inventory_and_json_controls(self):
        self.assertEqual({c["id"] for c in self.data["smooth_cases"]},
                         {"constant_positive", "time_varying", "constant_negative"})
        self.assertEqual(len(self.data["invalid_inputs"]), 9)
        with self.assertRaises(ValueError):
            json.loads('{"x":1,"x":2}', object_pairs_hook=unique_object)
        with self.assertRaises(ValueError):
            json.loads('{"x":NaN}', parse_constant=reject_constant)

    def test_exact_four_stage_witness(self):
        case = self.data["one_step"]
        calls = []

        def fitness(time, population):
            calls.append((time, population[0]))
            return tuple(map(Q, case["fitness"]))

        result = rk4_step(tuple(map(Q, case["initial"])), Q(case["time"]),
                          Q(case["step"]), fitness)
        self.assertEqual([t for t, _ in calls], list(map(Q, case["stage_times"])))
        self.assertEqual([x for _, x in calls],
                         list(map(Q, case["first_component_stages"])))
        self.assertEqual([x * (1 - x) for _, x in calls],
                         list(map(Q, case["first_component_slopes"])))
        self.assertEqual(result[0], Q(case["first_component_result"]))
        self.assertEqual(sum(result), 1)

    def test_legacy_euler_is_distinguished(self):
        case = self.data["one_step"]
        args = (tuple(map(Q, case["initial"])), Q(case["time"]),
                Q(case["step"]), lambda t, x: (Q(1), Q(0)))
        legacy = euler_step(*args)
        self.assertEqual(legacy[0], Q(case["legacy_euler_result"]))
        self.assertGreater(abs(legacy[0] - rk4_step(*args)[0]), Q(1, 1000))

    def test_analytic_literals(self):
        for case in self.data["smooth_cases"]:
            with self.subTest(case=case["id"]):
                a, b, start, end = (D(case[k]) for k in ("a", "b", "start", "end"))
                integral = a * (end - start) + b * (end * end - start * start) / 2
                self.assertEqual(integral, D(case["integrated_difference"]))
                self.assertLess(abs(logistic(D(case["initial_first"]), integral)
                                    - D(case["analytic_first"])),
                                D(self.data["tolerances"]["analytic_literal_absolute"]))

    def test_smooth_refinement_and_mass(self):
        for case in self.data["smooth_cases"]:
            with self.subTest(case=case["id"]):
                x = D(case["initial_first"])
                errors = []
                for count in case["step_counts"]:
                    states = trajectory((x, 1 - x), D(case["start"]), D(case["end"]),
                                        count, lambda t, p: (D(case["a"]) + D(case["b"]) * t, D(0)))
                    self.assertEqual(len(states), count + 1)
                    for state in states:
                        self.assertTrue(all(0 <= p <= 1 for p in state))
                        self.assertLessEqual(abs(sum(state) - 1),
                                             D(self.data["tolerances"]["mass_absolute"]))
                    errors.append(abs(states[-1][0] - D(case["analytic_first"])))
                self.assertGreater(errors[-1], D("1e-30"))
                self.assertLess(errors[-1], D(case["finest_error_max"]))
                for coarse, fine in zip(errors, errors[1:]):
                    ratio = coarse / fine
                    self.assertGreater(ratio, D(self.data["tolerances"]["refinement_ratio_min"]))
                    self.assertLess(ratio, D(self.data["tolerances"]["refinement_ratio_max"]))

    def test_euler_fails_the_finest_accuracy_bound(self):
        case = self.data["smooth_cases"][0]
        states = trajectory((D("0.25"), D("0.75")), D(0), D(1), 32,
                            lambda t, p: (D(1), D(0)), stepper=euler_step)
        self.assertGreater(abs(states[-1][0] - D(case["analytic_first"])), D("1e-4"))

    def test_time_dependent_stage_times(self):
        calls = []

        def fitness(time, population):
            calls.append(time)
            return (1 + 2 * time, Q(0))

        rk4_step((Q(1, 4), Q(3, 4)), Q(2), Q(1, 8), fitness)
        self.assertEqual(calls, [Q(2), Q(33, 16), Q(33, 16), Q(17, 8)])

    def test_frozen_stage_time_is_rejected_by_analytic_control(self):
        def frozen(population, time, step, fitness):
            return rk4_step(population, time, step, lambda ignored, p: fitness(time, p))

        states = trajectory((D("0.25"), D("0.75")), D(0), D(1), 64,
                            lambda t, p: (1 + 2 * t, D(0)), stepper=frozen)
        target = D(self.data["smooth_cases"][1]["analytic_first"])
        self.assertGreater(abs(states[-1][0] - target), D("1e-4"))

    def test_split_discontinuity_and_unsplit_negative_control(self):
        case = self.data["event_case"]
        initial = D(case["initial_first"])
        event, end = D(case["event_time"]), D(case["end"])
        count = case["steps_per_segment"]
        left = trajectory((initial, 1 - initial), D(0), event, count,
                          lambda t, p: (D(case["left_difference"]), D(0)))
        right = trajectory(left[-1], event, end, count,
                           lambda t, p: (D(case["right_difference"]), D(0)))
        self.assertLess(abs(right[-1][0] - D(case["analytic_first"])),
                        D(case["split_error_max"]))
        unsplit = trajectory((initial, 1 - initial), D(0), end, 2 * count,
                             lambda t, p: (D(1) if t < event else D(-1), D(0)))
        self.assertGreater(abs(unsplit[-1][0] - D(case["analytic_first"])),
                           D(case["unsplit_error_min"]))

    def test_simplex_faces_and_neutral_fitness(self):
        for population in [(Q(0), Q(1)), (Q(1), Q(0))]:
            self.assertEqual(rk4_step(population, Q(0), Q(1),
                                     lambda t, p: (Q(-2), Q(3))), population)
        population = (Q(1, 4), Q(3, 4))
        self.assertEqual(rk4_step(population, Q(0), Q(1),
                                 lambda t, p: (Q(-7), Q(-7))), population)

    def test_zero_step_identity(self):
        population = (Q(1, 4), Q(3, 4))
        self.assertEqual(rk4_step(population, Q(0), Q(0),
                                 lambda t, p: (Q(1), Q(0))), population)

    def test_reference_domain_errors(self):
        for case in self.data["invalid_inputs"]:
            with self.subTest(case=case["id"]), self.assertRaises(ValueError):
                rk4_step(tuple(map(D, case["population"])), D(case["time"]),
                         D(case["step"]), lambda t, p: (D(1), D(0)))

    def test_callback_domain_errors(self):
        for values in [(D(1),), (D("NaN"), D(0)), (D(1), D("Infinity"))]:
            with self.subTest(values=values), self.assertRaises(ValueError):
                rk4_step((D("0.25"), D("0.75")), D(0), D("0.1"),
                         lambda t, p: values)

    def test_oversized_step_does_not_clip_or_normalize(self):
        with self.assertRaises(ValueError):
            rk4_step((Q(1, 4), Q(3, 4)), Q(0), Q(16),
                     lambda t, p: (Q(1), Q(0)))


if __name__ == "__main__":
    unittest.main(verbosity=2)
