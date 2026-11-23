"""Regression controls for the CI Cargo-log admission rules."""

import importlib.util
from pathlib import Path
import unittest


MODULE = Path(__file__).resolve().parents[1] / "scripts/check_cargo_test_log.py"
SPEC = importlib.util.spec_from_file_location("check_cargo_test_log", MODULE)
assert SPEC and SPEC.loader
check = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(check)


def summary(count: int, ignored: int = 0, filtered: int = 0) -> str:
    return (
        f"test result: ok. {count} passed; 0 failed; {ignored} ignored; "
        f"0 measured; {filtered} filtered out; finished in 0.1s\n"
    )


class CargoLogChecks(unittest.TestCase):
    def test_exact_and_minimum_counts_require_one_clean_summary(self):
        check.check_summary(summary(4), 4)
        check.check_summary(summary(5), 4, minimum=True)
        for log in ("", summary(0), summary(4, ignored=1), summary(4, filtered=1), summary(4) * 2):
            with self.subTest(log=log), self.assertRaises(ValueError):
                check.check_summary(log, 4)
        with self.assertRaises(ValueError):
            check.check_summary(summary(3), 4, minimum=True)

    def test_suite_requires_named_target_and_exact_count(self):
        log = "Running tests/clarabel_backend.rs (target/debug/deps/test)\n" + summary(8)
        check.check_suite(log, "tests/clarabel_backend.rs", 8)
        for candidate in (
            log.replace("clarabel_backend", "other"),
            log.replace("8 passed", "7 passed"),
            log.replace("0 ignored", "1 ignored"),
            log.replace("0 filtered out", "1 filtered out"),
        ):
            with self.subTest(log=candidate), self.assertRaises(ValueError):
                check.check_suite(candidate, "tests/clarabel_backend.rs", 8)


if __name__ == "__main__":
    unittest.main()
