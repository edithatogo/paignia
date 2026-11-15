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


def finite_number(value):
    if not isinstance(value, (Q, D)) or (isinstance(value, D) and not value.is_finite()):
        raise ValueError("reference requires a finite Fraction or Decimal")
    return value


def population_domain(population):
    if len(population) != 2:
        raise ValueError("reference requires exactly two components")
    for value in population:
        finite_number(value)
        if value < 0 or value > 1:
            raise ValueError("population leaves the simplex; no clipping allowed")
    if abs(sum(population) - 1) > D("1e-65"):
        raise ValueError("population mass is not one; no normalization allowed")


def step_domain(population, time, step):
    population_domain(population)
    finite_number(time)
    finite_number(step)
    if step < 0:
        raise ValueError("reference supports forward steps only")


def replicator_rhs(time, population, fitness):
    """Definition of the two-component replicator, not a production callback API."""
    values = fitness(time, population)
    if len(values) != 2:
        raise ValueError("fitness dimension mismatch")
    for value in (*population, *values):
        finite_number(value)
    mean = sum(x * f for x, f in zip(population, values))
    return tuple(x * (f - mean) for x, f in zip(population, values))


def rk4_step(population, time, step, fitness):
    """Classical RK4 on the full two-component state; no projection or clipping.

    Exact stage witnesses and a closed-form ODE solution test this small reference.
    This function cannot be called through any Paignia production entry point.
    """
    step_domain(population, time, step)

    def stage(slope, amount):
        return tuple(x + amount * dx for x, dx in zip(population, slope))

    k1 = replicator_rhs(time, population, fitness)
    k2 = replicator_rhs(time + step / 2, stage(k1, step / 2), fitness)
    k3 = replicator_rhs(time + step / 2, stage(k2, step / 2), fitness)
    k4 = replicator_rhs(time + step, stage(k3, step), fitness)
    result = tuple(x + step * (a + 2 * b + 2 * c + d) / 6
                   for x, a, b, c, d in zip(population, k1, k2, k3, k4))
    population_domain(result)
    return result


def euler_step(population, time, step, fitness):
    """Named unclipped Euler negative control on the admitted small-step domain."""
    step_domain(population, time, step)
    result = tuple(x + step * dx for x, dx in
                   zip(population, replicator_rhs(time, population, fitness)))
    population_domain(result)
    return result


def trajectory(population, start, end, count, fitness, stepper=rk4_step):
    finite_number(start)
    finite_number(end)
    if type(count) is not int or count <= 0 or end < start:
        raise ValueError("invalid reference time grid")
    step = (end - start) / count
    states = [population]
    for index in range(count):
        population = stepper(population, start + index * step, step, fitness)
        states.append(population)
    return states


def logistic(initial, integrated_difference):
    """Separate closed form from d(log(x/(1-x)))/dt = f1-f2."""
    finite_number(initial)
    finite_number(integrated_difference)
    if initial < 0 or initial > 1:
        raise ValueError("initial proportion is outside [0,1]")
    if initial in (0, 1):
        return initial
    return 1 / (1 + ((1 - initial) / initial) * (-integrated_difference).exp())


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
        self.assertEqual(self.data["tolerances"]["analytic_literal_absolute"], "1e-70")
        self.assertEqual(self.data["tolerances"]["mass_absolute"], "1e-65")
        self.assertEqual(self.data["tolerances"]["refinement_ratio_min"], "14")
        self.assertEqual(self.data["tolerances"]["refinement_ratio_max"], "18")
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
