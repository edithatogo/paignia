#!/usr/bin/env python3
"""Check that Cargo ran the required tests without skips or filtering."""

import argparse
from pathlib import Path
import re


SUMMARY = re.compile(
    r"^test result: ok\. (\d+) passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;",
    re.MULTILINE,
)


def check_summary(log: str, count: int, *, minimum: bool = False) -> None:
    """Require one clean summary with the exact or minimum passing count."""
    summaries = SUMMARY.findall(log)
    if len(summaries) != 1 or (
        int(summaries[0]) < count if minimum else summaries[0] != str(count)
    ):
        raise ValueError(f"Expected {'at least' if minimum else 'exactly'} {count} passing tests without skips or filtering")


def check_suite(log: str, target: str, count: int) -> None:
    """Require the named Cargo integration-test suite and its exact summary."""
    suite = re.search(
        rf"Running[^\n]*{re.escape(target)}[^\n]*\n(.*?test result:[^\n]+)",
        log,
        re.DOTALL,
    )
    expected = f"test result: ok. {count} passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;"
    if suite is None or expected not in suite.group(1):
        raise ValueError(f"Expected {count} passing {target} tests without skips or filtering")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("log", type=Path)
    parser.add_argument("count", type=int)
    parser.add_argument("--target", help="Cargo integration-test source path")
    parser.add_argument("--minimum", action="store_true")
    args = parser.parse_args()
    if args.target and args.minimum:
        parser.error("--target and --minimum cannot be combined")
    try:
        if args.target:
            check_suite(args.log.read_text(), args.target, args.count)
        else:
            check_summary(args.log.read_text(), args.count, minimum=args.minimum)
    except ValueError as error:
        raise SystemExit(str(error)) from error


if __name__ == "__main__":
    main()
