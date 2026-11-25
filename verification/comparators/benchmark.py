"""Measure the checked normal-form and Kuhn workloads on a Darwin host."""

import argparse
from contextlib import contextmanager
from hashlib import sha256
import json
import os
from pathlib import Path
import platform
from random import Random
import re
import signal
from statistics import median
import subprocess
import sys
import time

from compare import RELEASE, Bundle, encode, normal_request, parse_graph, pyspiel

FIXTURES = (
    ("binary-003", "pure"),
    ("matching-pennies", "mixed"),
    ("asymmetric-coordination", "mixed"),
    ("rectangular", "mixed"),
    ("rock-paper-scissors", "mixed"),
    ("all-zero", "mixed"),
)
REPEATS = 9
ORDER_SEED = 211
PROCESS_TIMEOUT_SECONDS = 30
KUHN_RUNNER = """
import json
import sys
import pyspiel

request = json.load(sys.stdin)
game = pyspiel.load_game(request["game"])
solver = {"cfr": pyspiel.CFRSolver, "cfr_plus": pyspiel.CFRPlusSolver}
variant = sys.argv[1] if len(sys.argv) > 1 else "cfr_plus"
trainer = solver[variant](game)
for _ in range(request["iterations"]):
    trainer.evaluate_and_update_policy()
policy = trainer.tabular_average_policy()
print(json.dumps({
    "values": pyspiel.expected_returns(game.new_initial_state(), policy, -1, True),
    "nash_conv": pyspiel.nash_conv(game, policy),
}))
"""


def read_verified(directory, path, hashes):
    content = (directory / path).read_bytes()
    if sha256(content).hexdigest() != hashes[path]:
        raise ValueError(f"frozen evidence changed: {path}")
    return content


def emit_nfg(name, request):
    # The timed fixtures have exact integer payoffs. The first seat varies fastest.
    table = request["game"]["payoffs"]
    rows, columns = len(table), len(table[0])
    values = [
        str(int(payoff["value"]))
        for column in range(columns)
        for row in range(rows)
        for payoff in table[row][column]["payoffs"]
    ]
    header = f'NFG 1 R "{name}" {{ "column" "row" }} {{ {rows} {columns} }}\n\n'
    return (header + " ".join(values) + "\n").encode()


def measure(name, inputs, expected, bundle, binaries, method, tool):
    started = time.perf_counter_ns()
    graph = parse_graph(inputs, "nt")
    request = normal_request(graph, inputs, bundle, method)
    if tool == "paignia":
        input_bytes = encode(request)
        invocation = [binaries["profile"]]
    else:
        input_bytes = emit_nfg(name, request)
        invocation = [binaries[method], "-q"]
    converted = time.perf_counter_ns()
    if input_bytes != expected[0]:
        raise ValueError(f"{name}/{tool}: input SHA-256 changed to {sha256(input_bytes).hexdigest()}")
    observed, solve_ns, rss = timed_process(invocation, input_bytes)
    if observed != expected[1]:
        raise ValueError(f"{name}/{tool}: output SHA-256 changed to {sha256(observed).hexdigest()}")
    return {
        "tool": tool, "conversion_ns": converted - started,
        "solve_process_ns": solve_ns, "total_ns": solve_ns + converted - started,
        "peak_rss_bytes": rss,
    }


def timed_process(invocation, input_bytes):
    system = platform.system()
    if system == "Darwin":
        arguments = ["-l"]
        pattern = rb"^\s*(\d+)\s+maximum resident set size$"
        bytes_per_unit = 1
    elif system == "Linux":
        arguments = ["-f", "peak_rss_kib %M"]
        pattern = rb"^peak_rss_kib (\d+)$"
        bytes_per_unit = 1024
    else:
        raise ValueError(f"unqualified peak RSS reporter for {system}")
    command = ["/usr/bin/time", *arguments, *invocation]
    started = time.perf_counter_ns()
    with subprocess.Popen(
        command, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        start_new_session=True,
    ) as process:
        try:
            stdout, stderr = process.communicate(input_bytes, timeout=PROCESS_TIMEOUT_SECONDS)
        except subprocess.TimeoutExpired as error:
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            error.output, error.stderr = process.communicate()
            raise
    finished = time.perf_counter_ns()
    if process.returncode:
        raise subprocess.CalledProcessError(process.returncode, command, stdout, stderr)
    rss = re.search(pattern, stderr, re.MULTILINE)
    if rss is None or int(rss[1]) == 0:
        raise ValueError(f"no process peak RSS for {invocation[0]}")
    return stdout, finished - started, bytes_per_unit * int(rss[1])


@contextmanager
def record_trial(case, trial, tool):
    """Save the completed prefix and the failing trial even without a summary."""
    row = {"trial": trial, "tool": tool}
    try:
        yield row
    except Exception as error:
        row["failure"] = {"type": type(error).__name__, "message": str(error)}
        if isinstance(error, (subprocess.CalledProcessError, subprocess.TimeoutExpired)):
            for stream in ("stdout", "stderr"):
                content = getattr(error, stream)
                if content:
                    row["failure"][f"{stream}_tail"] = content[-8192:].decode(errors="replace")
        raise
    finally:
        with (case / "trials.jsonl").open("a", encoding="utf-8") as journal:
            journal.write(json.dumps(row, sort_keys=True, allow_nan=False) + "\n")


def kuhn_trials(directory, hashes, profile_binary, order):
    fixture = "kuhn-cfr_plus"
    request = read_verified(directory, f"{fixture}/paignia-12000-request.json", hashes)
    expected = read_verified(directory, f"{fixture}/paignia-12000.json", hashes)
    oracle = read_verified(directory, f"{fixture}/open-spiel-12000.json", hashes)
    open_spiel_inputs = encode({"game": "kuhn_poker", "iterations": 12000})
    inputs = {
        "paignia": ([profile_binary], request),
        "open_spiel": ([sys.executable, "-c", KUHN_RUNNER], open_spiel_inputs),
    }
    trials = []
    for index in range(REPEATS + 1):
        sides = list(inputs)
        order.shuffle(sides)
        for tool in sides:
            observed, elapsed, rss = timed_process(*inputs[tool])
            if tool == "paignia":
                matching = observed == expected
            else:
                matching = json.loads(observed) == json.loads(oracle)["observations"]["open_spiel"]
            if not matching:
                raise ValueError(f"Kuhn CFR+ observation changed for {tool}")
            trials.append({
                "trial": index, "tool": tool, "process_ns": elapsed, "peak_rss_bytes": rss,
            })
    return {
        "iterations": 12000,
        "open_spiel_input_sha256": sha256(open_spiel_inputs).hexdigest(),
        "samples": trials,
        "warm_summary": summarise(trials, ("process_ns", "peak_rss_bytes")),
    }


def summarise(trials, keys):
    summary = {}
    for tool in sorted({row["tool"] for row in trials}):
        repeated = [row for row in trials if row["trial"] > 0 and row["tool"] == tool]
        summary[tool] = {}
        for key in keys:
            values = [row[key] for row in repeated]
            summary[tool][key] = {"min": min(values), "median": median(values), "max": max(values)}
    return summary


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--replay", type=Path, required=True)
    parser.add_argument("--paignia-bin", type=Path, required=True)
    parser.add_argument("--gambit-bin", type=Path, required=True)
    parser.add_argument("--ontology-dir", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if platform.system() != "Darwin":
        parser.error("this receipt uses the Darwin process RSS counter")
    binaries = {
        "profile": args.paignia_bin / "solve_profile",
        "pure": args.gambit_bin / "gambit-enumpure",
        "mixed": args.gambit_bin / "gambit-enummixed",
    }
    manifest = (args.replay / "receipt.json").read_bytes()
    receipt = json.loads(manifest)
    if receipt["release_commit"] != RELEASE:
        raise ValueError("replay belongs to another Paignia release")
    for tool, binary in {**binaries, "pyspiel": Path(pyspiel.__file__)}.items():
        if sha256(binary.read_bytes()).hexdigest() != receipt["tools_sha256"][tool]:
            raise ValueError(f"unchecked executable: {tool}")
    bundle = Bundle(args.ontology_dir)
    if (bundle.revision, bundle.lock_sha256) != (receipt["ontology_commit"], receipt["ontology_lock_sha256"]):
        raise ValueError("unchecked ontology bundle")
    workloads = {}
    order = Random(ORDER_SEED)
    for name, method in FIXTURES:
        def checked(filename):
            return read_verified(args.replay, f"{name}/{filename}", receipt["files"])
        asserted = checked("asserted.nt")
        expected = {
            "paignia": (checked("paignia-request.json"), checked("paignia.json")),
            "gambit": (checked("game.nfg"), checked("gambit.csv")),
        }
        trials = []
        for index in range(REPEATS + 1):
            sides = list(expected)
            order.shuffle(sides)
            for tool in sides:
                trials.append({"trial": index, **measure(
                    name, asserted, expected[tool], bundle, binaries, method, tool,
                )})
        keys = ("conversion_ns", "solve_process_ns", "total_ns", "peak_rss_bytes")
        workloads[name] = {"method": method, "samples": trials, "warm_summary": summarise(trials, keys)}
    kuhn = kuhn_trials(args.replay, receipt["files"], binaries["profile"], order)
    args.output.write_bytes(encode({
        "release_commit": receipt["release_commit"], "system": platform.platform(),
        "python": platform.python_version(), "order_seed": ORDER_SEED,
        "replay_sha256": sha256(manifest).hexdigest(), "workloads": workloads, "kuhn_cfr_plus": kuhn,
    }))


if __name__ == "__main__":
    main()
