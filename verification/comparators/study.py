"""Scale equivalent normal-form games and time Kuhn policies at a common target."""

import argparse
import json
import os
from pathlib import Path
import platform
from random import Random
import subprocess
import sys

from benchmark import KUHN_RUNNER, measure, summarise, timed_process
from compare import RELEASE, Bundle, compare_kuhn, digest, encode, gambit_case, pyspiel
from static_study import static_study

HERE = Path(__file__).resolve().parent
SEED = 271
REPEATS = 9
SCALES = {"pure": (2, 4, 8), "mixed": (2, 4, 6)}
NASH_CONV_TARGET = 0.02
KUHN_VALUE_ERROR = 0.005


def coordination(size, rng):
    # Every nonempty common support has the unique inverse-payoff mixture.
    diagonal = rng.sample(range(2, 21), size)
    return [
        [(diagonal[row], diagonal[row]) if row == column else (0, 0)
         for column in range(size)]
        for row in range(size)
    ]


def normal_study(bundle, binaries, destination):
    matrix_order = Random(SEED)
    trial_order = Random(SEED)
    workloads = {}
    for method, sizes in SCALES.items():
        for size in sizes:
            name = f"coordination-{method}-{size}"
            matrix = coordination(size, matrix_order)
            oracle = gambit_case(bundle, binaries, destination, name, matrix, method)
            witness_count = oracle if method == "pure" else oracle["witnesses"]
            expected_count = size if method == "pure" else 2**size - 1
            if witness_count != expected_count:
                raise ValueError(f"{name}: expected {expected_count} equilibrium witnesses")
            case = destination / name
            asserted = (case / "asserted.nt").read_bytes()
            expected = {
                "paignia": ((case / "paignia-request.json").read_bytes(), (case / "paignia.json").read_bytes()),
                "gambit": ((case / "game.nfg").read_bytes(), (case / "gambit.csv").read_bytes()),
            }
            rows = []
            for trial in range(REPEATS + 1):
                order = list(expected)
                trial_order.shuffle(order)
                for name_of_tool in order:
                    rows.append({
                        "trial": trial,
                        **measure(name, asserted, expected[name_of_tool], bundle, binaries, method, name_of_tool),
                    })
            workloads[name] = {
                "method": method, "actions_per_player": size, "matrix": matrix,
                "correctness": oracle, "samples": rows,
                "repeated": summarise(rows, ("conversion_ns", "solve_process_ns", "total_ns", "peak_rss_bytes")),
            }
    return workloads


def first_checkpoint(rows, tool):
    for row in rows:
        outcome = row[tool]
        if (0 <= outcome["nash_conv"] <= NASH_CONV_TARGET
                and abs(outcome["values"][0] + 1 / 18) <= KUHN_VALUE_ERROR):
            return row["iterations"]
    raise ValueError(f"{tool}: no checkpoint met the predeclared Kuhn accuracy target")


def accuracy_study(bundle, profile_binary, destination):
    # compare_kuhn checks labelled histories and every returned policy with the
    # independent OpenSpiel value and best-response evaluator before timing.
    verified = compare_kuhn(profile_binary, bundle, destination)
    trial_order = Random(SEED)
    report = {}
    for variant, observations in verified.items():
        case = destination / f"kuhn-{variant}"
        first = {tool: first_checkpoint(observations, tool) for tool in ("paignia", "open_spiel")}
        native_input = (case / f"paignia-{first['paignia']}-request.json").read_bytes()
        native_expected = (case / f"paignia-{first['paignia']}.json").read_bytes()
        open_input = encode({"game": "kuhn_poker", "iterations": first["open_spiel"]})
        oracle = next(row["open_spiel"] for row in observations if row["iterations"] == first["open_spiel"])
        cases = {
            "paignia": ([profile_binary], native_input),
            "open_spiel": ([sys.executable, "-c", KUHN_RUNNER, variant], open_input),
        }
        samples = []
        for trial in range(REPEATS + 1):
            order = list(cases)
            trial_order.shuffle(order)
            for tool in order:
                output, elapsed, rss = timed_process(*cases[tool])
                if tool == "open_spiel":
                    matches = json.loads(output) == oracle
                else:
                    matches = output == native_expected
                if not matches:
                    raise ValueError(f"{variant} {tool}: timed output differs from verified output")
                samples.append({"trial": trial, "tool": tool, "process_ns": elapsed, "peak_rss_bytes": rss})
        report[variant] = {
            "checkpoints": observations, "first_admitted_checkpoint": first,
            "open_spiel_input_sha256": digest(open_input), "samples": samples,
            "repeated": summarise(samples, ("process_ns", "peak_rss_bytes")),
        }
    return report


def command(*args):
    return subprocess.check_output(args, text=True, timeout=20).strip()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--provider", type=Path, required=True)
    parser.add_argument("--paignia-bin", type=Path, required=True)
    parser.add_argument("--gambit-bin", type=Path, required=True)
    parser.add_argument("--ontology-dir", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if command("git", "-C", args.provider, "rev-parse", "HEAD") != RELEASE:
        parser.error("build the provider examples from the signed release")
    if command("git", "-C", args.provider, "status", "--porcelain=v1"):
        parser.error("the provider source checkout must be clean")
    if platform.python_version() != (HERE.parents[1] / ".python-version").read_text().strip():
        parser.error("use the exact repository Python patch release")
    args.output.mkdir()
    binaries = {
        "static": args.paignia_bin.resolve() / "solve",
        "profile": args.paignia_bin.resolve() / "solve_profile",
        "pure": args.gambit_bin.resolve() / "gambit-enumpure",
        "mixed": args.gambit_bin.resolve() / "gambit-enummixed",
    }
    bundle = Bundle(args.ontology_dir)
    tools = {**binaries, "pyspiel": Path(pyspiel.__file__).resolve()}
    metadata = {
        "system": platform.platform(), "machine": platform.machine(),
        "logical_cpus": os.cpu_count(), "python": platform.python_version(),
        "provider": RELEASE, "provider_tree": command("git", "-C", args.provider, "rev-parse", "HEAD^{tree}"),
        "tools_sha256": {name: digest(path.read_bytes()) for name, path in tools.items()},
        "ontology_commit": bundle.revision, "ontology_lock_sha256": bundle.lock_sha256,
        "protocol_sha256": digest((HERE / "study.md").read_bytes()),
        "harness_sha256": {name: digest((HERE / name).read_bytes())
                           for name in ("study.py", "static_study.py", "compare.py", "benchmark.py")},
        "seed": SEED, "nash_conv_target": NASH_CONV_TARGET, "kuhn_value_error": KUHN_VALUE_ERROR,
    }
    (args.output / "environment.json").write_bytes(encode(metadata))
    normal = normal_study(bundle, binaries, args.output)
    static = static_study(binaries["static"], args.output, SEED, REPEATS)
    kuhn = accuracy_study(bundle, binaries["profile"], args.output)
    hashes = {str(path.relative_to(args.output)): digest(path.read_bytes())
              for path in sorted(args.output.rglob("*")) if path.is_file()}
    (args.output / "summary.json").write_bytes(encode({
        "environment": metadata, "normal_form": normal, "static": static, "kuhn": kuhn, "files_sha256": hashes,
    }))


if __name__ == "__main__":
    main()
