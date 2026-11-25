"""Regenerate or check the two documentation flowcharts against admitted models."""

import argparse
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
INTERCHANGE = ROOT / "verification/uogto_interchange"
sys.path.insert(0, str(INTERCHANGE))

from interchange import digest, parse_graph  # noqa: E402
from render_mermaid import diagram, label, stable_id  # noqa: E402
from verify import CASE, project  # noqa: E402


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--solver", type=Path, default=ROOT / "target/release/examples/solve")
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    if label('"\n|<&') != "#34;#10;#124;#60;#38;":
        raise ValueError("diagram label delimiters were not escaped")
    original = (INTERCHANGE / "frozen-static-v1/request.json").read_bytes()
    shared_source = (ROOT / "docs-site/fixtures/buyer-seller-shared.nt").read_bytes()
    shared, _ = project(parse_graph(shared_source, "nt"), digest(shared_source))
    shared["instance"] += "-shared"
    shared["run"] += "-shared"
    for name, wire, payoffs in (
        ("buyer-seller", original, (2, 4)),
        ("shared-seller", json.dumps(shared).encode(), (3, 0)),
    ):
        result = json.loads(subprocess.check_output([args.solver.resolve()], input=wire, timeout=30))
        expected = result["expected_payoffs"]
        if (expected[str(CASE.acheteur)], expected[str(CASE["décideur"])]) != payoffs:
            raise ValueError(f"{name} differs from the independent root value")
        target = ROOT / "docs-site/src/assets" / f"{name}.mmd"
        rendered = diagram(json.loads(wire))
        if name == "shared-seller":
            seller = stable_id("n", str(CASE.respond))
            if sum(line.endswith(" " + seller) for line in rendered.splitlines()) != 2:
                raise ValueError("shared seller successor lost an edge")
            if result["choices"][str(CASE.respond)] != str(CASE["pass"]):
                raise ValueError("shared seller tie changed")
        if args.write:
            target.write_text(rendered, encoding="utf-8")
        elif target.read_text(encoding="utf-8") != rendered:
            raise ValueError(f"{name} diagram asset differs from the admitted game")
    print("Both static diagrams match admitted game identities, choices and payoffs")


if __name__ == "__main__":
    main()
