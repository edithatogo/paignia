#!/usr/bin/env python3
"""Build an isolated consumer from extracted source packages, without a registry.

This is an extracted-source smoke check. It does not claim published .crate
artifacts, a released API, scientific qualification or two real consumers.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import tomllib


ROOT = Path(__file__).resolve().parents[1]
PACKAGES = ("paignia-core", "paignia-solve")
CONSUMER_TEST = r'''
use std::collections::BTreeMap;
use paignia_core::{ActionEdge, ActionId, ChanceEdge, Game, Node, NodeId, PlayerId};
use paignia_solve::backward_induction;

#[test]
fn chance_expectation_and_selected_action() {
    let player = PlayerId("player".into());
    let root = NodeId("root".into());
    let certain = NodeId("certain".into());
    let chance = NodeId("chance".into());
    let low = NodeId("low".into());
    let high = NodeId("high".into());
    let game = Game {
        root: root.clone(),
        nodes: BTreeMap::from([
            (root.clone(), Node::Decision {
                player: player.clone(),
                edges: vec![
                    ActionEdge { action: ActionId("certain".into()), target: certain.clone() },
                    ActionEdge { action: ActionId("chance".into()), target: chance.clone() },
                ],
            }),
            (chance, Node::Chance { edges: vec![
                ChanceEdge { probability: 0.5, target: low.clone() },
                ChanceEdge { probability: 0.5, target: high.clone() },
            ] }),
            (certain, Node::Terminal { payoffs: BTreeMap::from([(player.clone(), 4.0)]) }),
            (low, Node::Terminal { payoffs: BTreeMap::from([(player.clone(), 0.0)]) }),
            (high, Node::Terminal { payoffs: BTreeMap::from([(player.clone(), 10.0)]) }),
        ]),
    };
    let solution = backward_induction(&game).expect("supported finite chance game");
    assert_eq!(solution.choices[&root], ActionId("chance".into()));
    // Independent oracle: 0.5 * 0 + 0.5 * 10 = 5, greater than the certain 4.
    assert!((solution.expected_payoffs[&player] - 5.0).abs() <= 1e-12);
}

#[test]
fn invalid_graph_is_rejected() {
    let game = Game { root: NodeId("missing".into()), nodes: BTreeMap::new() };
    assert!(backward_induction(&game).is_err());
}
'''


def run(command: list[str], cwd: Path) -> str:
    result = subprocess.run(command, cwd=cwd, text=True, capture_output=True, check=True)
    if result.stdout:
        print(result.stdout, end="")
    if result.stderr:
        print(result.stderr, end="")
    return result.stdout


def standalone_manifest(source: Path, workspace: dict) -> str:
    """Resolve inherited metadata/lints while retaining the package's source manifest."""
    text = source.read_text(encoding="utf-8")
    package = tomllib.loads(text)["package"]
    for name, value in package.items():
        if value == {"workspace": True}:
            inherited = workspace["package"][name]
            assignment = rf"(?m)^{re.escape(name)}\.workspace[ \t]*=[ \t]*true[ \t]*$"
            text, replaced = re.subn(
                assignment, lambda _: f"{name} = {json.dumps(inherited)}", text
            )
            if replaced != 1:
                raise ValueError(f"expected one inherited package assignment: {name}")
    lint_lines: list[str] = []
    for group, lints in workspace.get("lints", {}).items():
        lint_lines.append(f"[lints.{group}]")
        for name, value in lints.items():
            if not isinstance(value, (str, int, bool)):
                raise ValueError(f"unsupported inherited lint value: {group}.{name}")
            lint_lines.append(f"{name} = {json.dumps(value)}")
        lint_lines.append("")
    text = text.replace("[lints]\nworkspace = true", "\n".join(lint_lines).rstrip())
    if "workspace = true" in text:
        raise ValueError("unresolved workspace inheritance in extracted package")
    resolved = tomllib.loads(text)["package"]
    for name, value in package.items():
        expected = workspace["package"][name] if value == {"workspace": True} else value
        if resolved.get(name) != expected:
            raise ValueError(f"extracted package field changed during materialization: {name}")
    return text


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--receipt", type=Path, help="optional JSON result from successful native execution")
    args = parser.parse_args()
    cargo = shutil.which("cargo")
    if cargo is None:
        parser.exit(1, "cargo is required; extracted native consumer was not executed\n")
    workspace = tomllib.loads((ROOT / "Cargo.toml").read_text())["workspace"]
    receipt: dict = {
        "scope": "extracted-source-package-smoke",
        "registry_artifact": False,
        "two_real_consumer_acceptance": False,
        "source_files_sha256": {},
        "packages": list(PACKAGES),
    }
    with tempfile.TemporaryDirectory(prefix="paignia-extracted-") as temporary:
        staging = Path(temporary).resolve()
        if staging.is_relative_to(ROOT):
            raise ValueError("smoke consumer must be outside the provider workspace")
        for name in PACKAGES:
            source = ROOT / "crates" / name
            destination = staging / name
            shutil.copytree(source, destination, ignore=shutil.ignore_patterns("target", ".git"))
            (destination / "Cargo.toml").write_text(
                standalone_manifest(source / "Cargo.toml", workspace), encoding="utf-8"
            )
            for path in sorted(destination.rglob("*")):
                if path.is_file():
                    key = path.relative_to(staging).as_posix()
                    receipt["source_files_sha256"][key] = hashlib.sha256(path.read_bytes()).hexdigest()
        consumer = staging / "consumer"
        (consumer / "src").mkdir(parents=True)
        (consumer / "tests").mkdir()
        version = json.dumps(workspace["package"]["version"])
        dependencies = "\n".join(
            f'{name} = {{ path = "../{name}", version = {version} }}' for name in PACKAGES
        )
        (consumer / "Cargo.toml").write_text(
            '[package]\nname = "paignia-extracted-consumer"\nversion = "0.0.0"\n'
            'edition = "2021"\npublish = false\n\n[dependencies]\n' + dependencies + "\n",
            encoding="utf-8",
        )
        (consumer / "src/lib.rs").write_text("//! Isolated package smoke consumer.\n")
        (consumer / "tests/extracted_consumer.rs").write_text(CONSUMER_TEST)
        manifest = str(consumer / "Cargo.toml")
        run([cargo, "generate-lockfile", "--offline", "--manifest-path", manifest], consumer)
        output = run(
            [cargo, "test", "--offline", "--locked", "--manifest-path", manifest,
             "--test", "extracted_consumer"],
            consumer,
        )
        if "test result: ok. 2 passed; 0 failed" not in output:
            raise RuntimeError("expected both native consumer tests to execute and pass")
        receipt["tests_passed"] = 2
        receipt["cargo_version"] = run([cargo, "--version"], consumer).strip()
        receipt["provider_checkout_dependency"] = False
        receipt["manifest_resolution"] = "workspace fields materialized; sibling path+version dependencies; offline lockfile"
    if args.receipt:
        args.receipt.parent.mkdir(parents=True, exist_ok=True)
        args.receipt.write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n")
    print("Extracted source-package smoke passed (2 native tests; no registry publication claimed)")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except subprocess.CalledProcessError as error:
        if error.stdout:
            print(error.stdout)
        if error.stderr:
            print(error.stderr)
        raise SystemExit(error.returncode) from error
