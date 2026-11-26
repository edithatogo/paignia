"""Run the pinned receiving bridge against the signed Paignia source."""

import argparse
import json
import subprocess
import sys
import tomllib
from copy import deepcopy
from hashlib import file_digest
from pathlib import Path

PAIGNIA_REVISION = "a04a19da06cc50e310a172574ed66f62000b891a"
DAPANE_REVISION = "7b530293dd5575c38c7daa008c0b0e1d51673d82"
PACKAGE_SOURCE = "git+https://github.com/edithatogo/paignia.git"
PROFILE = "paignia/static-finite-perfect-information-sparse-zero/1.0.0"


def run_git(root, *args):
    return subprocess.run(
        ["git", "-C", str(root), *args], check=True, capture_output=True, text=True
    ).stdout.strip()


def sha256(path):
    with path.open("rb") as stream:
        return file_digest(stream, "sha256").hexdigest()


def replay(receiver, binary):
    """Verify the build pin, receiving policy, and two independent rejection controls."""
    if run_git(receiver, "rev-parse", "HEAD") != DAPANE_REVISION:
        raise ValueError("Unexpected Dapane receiving revision")
    if run_git(receiver, "status", "--porcelain"):
        raise ValueError("The Dapane receiving checkout must be clean")
    lock_path = Path(__file__).with_name("Cargo.lock")
    packages = {
        row["name"]: row["source"]
        for row in tomllib.loads(lock_path.read_text())["package"]
        if row["name"] in {"paignia-core", "paignia-solve"}
    }
    source = f"{PACKAGE_SOURCE}?rev={PAIGNIA_REVISION}#{PAIGNIA_REVISION}"
    if packages != dict.fromkeys(("paignia-core", "paignia-solve"), source):
        raise ValueError("Cargo.lock must bind both Paignia crates to the signed release")

    adapter = receiver / "adapters" / "decision_provider_d2"
    sys.path[:0] = [str(adapter), str(adapter / "tests")]
    from bridge import D2Error, paignia_game_solve_native
    from test_bridge import d2_paignia

    payload = d2_paignia()
    artifact_digest = f"sha256:{sha256(binary)}"
    payload["provider"].update(revision=PAIGNIA_REVISION, artifact_digest=artifact_digest)
    # These are the original known-answer utilities in Dapane's native fixture.
    payload["actions"] = [
        {"id": "A", "utility": {"numerator": "10", "denominator": "1"}},
        {"id": "B", "utility": {"numerator": "20", "denominator": "1"}},
    ]

    def check_solution(request, action, utility):
        result = paignia_game_solve_native(request, str(binary))
        expected = {
            "profile": PROFILE,
            "source_revision": PAIGNIA_REVISION,
            "root": "start",
            "selected_action": action,
            "expected_payoff": utility,
        }
        if (
            result["provider_execution"] != "native_invoked"
            or result["key"] != request["key"]
            or result["paignia_solution"] != expected
        ):
            raise ValueError(f"Receiving solution differs from the oracle for action {action}")
        return result["paignia_solution"]

    controls = {"original": check_solution(payload, "B", 20.0)}
    altered = deepcopy(payload)
    altered["actions"][0]["utility"]["numerator"] = "30"
    controls["changed_utility"] = check_solution(altered, "A", 30.0)

    for key, value, code in (
        ("revision", "44bebdae9ca53a113681041e8fe5dcc16f57b918", "native_provider_error"),
        ("artifact_digest", "sha256:" + "0" * 64, "provider_digest_mismatch"),
    ):
        rejected = deepcopy(payload)
        rejected["provider"][key] = value
        try:
            paignia_game_solve_native(rejected, str(binary))
        except D2Error as error:
            if error.code != code:
                raise ValueError(f"Expected {code}, received {error.code}") from error
            controls[f"mismatched_{key}"] = error.code
        else:
            raise ValueError(f"Dapane accepted an inconsistent {key}")

    return {
        "dapane_revision": DAPANE_REVISION,
        "receiving_bridge_sha256": sha256(adapter / "bridge.py"),
        "receiving_fixture_sha256": sha256(adapter / "tests" / "test_bridge.py"),
        "paignia_revision": PAIGNIA_REVISION,
        "artifact_digest": artifact_digest,
        "cargo_lock_sha256": sha256(lock_path),
        "draw_id": payload["key"]["draw_id"],
        "controls": controls,
    }


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("receiver", type=Path)
    parser.add_argument("binary", type=Path)
    args = parser.parse_args()
    print(json.dumps(replay(args.receiver.resolve(), args.binary.resolve()), indent=2))
