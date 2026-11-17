#!/usr/bin/env python3
"""Qualify real .crate artifacts in native consumers with provider source masked.

Linux bubblewrap makes the provider checkout unavailable during resolution,
compilation and tests. No registry publication, scientific-method qualification
or real-application receipt is inferred. --source-smoke retains the historical
copy-based development check under its explicitly weaker scope.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess
import tarfile
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

ARTIFACT_SOLVER_TEST = CONSUMER_TEST + r'''

#[test]
fn pure_coordination_preserves_both_equilibria() {
    use paignia_solve::{pure_nash_equilibria, NormalFormGame, SolverConfig};
    let row = PlayerId("row".into());
    let column = PlayerId("column".into());
    let a = ActionId("a".into());
    let b = ActionId("b".into());
    let mut payoffs = BTreeMap::new();
    for x in [&a, &b] {
        for y in [&a, &b] {
            let value = if x == y { 1.0 } else { 0.0 };
            payoffs.insert(
                BTreeMap::from([(row.clone(), x.clone()), (column.clone(), y.clone())]),
                BTreeMap::from([(row.clone(), value), (column.clone(), value)]),
            );
        }
    }
    let game = NormalFormGame {
        players: vec![row.clone(), column.clone()],
        strategies: BTreeMap::from([(row.clone(), vec![a.clone(), b.clone()]),
                                    (column.clone(), vec![a.clone(), b.clone()])]),
        payoffs,
    };
    let actual = pure_nash_equilibria(&game, SolverConfig::default()).unwrap();
    let expected = vec![
        BTreeMap::from([(row.clone(), a.clone()), (column.clone(), a)]),
        BTreeMap::from([(row, b.clone()), (column, b)]),
    ];
    assert_eq!(actual, expected);
}

#[test]
fn artifact_emits_a_source_bound_native_result() {
    use paignia_core::{GameInstanceId, GameSpecification, GameSpecificationId,
        ResultIdentityRecord, ResultStatus, SolverRunId, STATIC_GAME_PROFILE_V1};
    use paignia_solve::{backward_induction_result, SolverConfig};
    let source = GameSpecification {
        schema_version: "1.0.0".into(), id: "urn:artifact:spec".into(),
        game: Game { root: NodeId("terminal".into()), nodes: BTreeMap::from([
            (NodeId("terminal".into()), Node::Terminal {
                payoffs: BTreeMap::from([(PlayerId("p".into()), -2.0)]),
            }),
        ]) },
    };
    let instance = GameInstanceId("urn:artifact:instance".into());
    let ids = ResultIdentityRecord { schema_version: "1.0.0".into(),
        specification_id: GameSpecificationId(source.id.clone()),
        game_instance_id: instance.clone(), solver_run_id: SolverRunId("urn:artifact:run".into()),
        session_id: None, policy_artifact_id: None, solver_trace_id: None,
    };
    let result = backward_induction_result(&source, STATIC_GAME_PROFILE_V1, ids, SolverConfig::default()).unwrap();
    assert_eq!(result.envelope.status, ResultStatus::Completed);
    result.envelope.validate_against(&source, &instance).unwrap();
    assert_eq!(result.envelope.solutions[0].expected_payoffs[&PlayerId("p".into())], -2.0);
    assert_eq!(result.diagnostic_trace.len(), 1);
}

#[test]
fn artifact_rk4_matches_the_independent_four_stage_witness() {
    use paignia_solve::{replicator_rk4_step, ReplicatorConfig, ReplicatorMethod};
    let result = replicator_rk4_step(&[0.25, 0.75], 0.0, 0.5,
        ReplicatorConfig { mass_tolerance: 1e-12 }, |_, _| Ok(vec![1.0, 0.0])).unwrap();
    let expected = 408885515741609973.0_f64 / 1152921504606846976.0_f64;
    assert!((result.population[0] - expected).abs() < 2e-15);
    assert_eq!(result.method, ReplicatorMethod::ClassicalRk4);
    assert_eq!(result.fitness_evaluations, 4);
    assert!((result.population.iter().sum::<f64>() - 1.0).abs() < 1e-12);
}
'''

ARTIFACT_CORE_TEST = r'''
use std::collections::BTreeMap;
use paignia_core::{Game, Node, NodeId, PlayerId};

#[test]
fn validates_a_finite_terminal_game() {
    let root = NodeId("https://example.invalid/game#terminal".into());
    let game = Game { root: root.clone(), nodes: BTreeMap::from([
        (root, Node::Terminal { payoffs: BTreeMap::from([(PlayerId("p".into()), -2.0)]) }),
    ]) };
    game.validate().unwrap();
}

#[test]
fn rejects_missing_root() {
    let game = Game { root: NodeId("absent".into()), nodes: BTreeMap::new() };
    assert!(game.validate().is_err());
}
'''


def run(command: list[str], cwd: Path, *, echo: bool = True) -> str:
    result = subprocess.run(command, cwd=cwd, text=True, capture_output=True, check=True)
    if echo and result.stdout:
        print(result.stdout, end="")
    if echo and result.stderr:
        print(result.stderr, end="")
    return result.stdout


def require_native_tests(output: str, expected: int) -> None:
    """Reject absent, filtered, skipped, failed or duplicated test summaries."""
    summaries = re.findall(
        r"^test result: ok\. (\d+) passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;",
        output, re.MULTILINE,
    )
    if expected <= 0 or summaries != [str(expected)]:
        raise RuntimeError(f"expected exactly {expected} native tests without skips or failures")


def isolated_command(command: list[str], provider: Path, staging: Path) -> list[str]:
    """Mask the provider checkout for the complete child process tree.

The host filesystem is read-only except the temporary consumer and Cargo cache.
This is an acceptance guard, not an untrusted-code execution security boundary.
"""
    bubblewrap = shutil.which("bwrap")
    if bubblewrap is None:
        raise RuntimeError("bubblewrap is required; provider-unavailable artifact execution was not performed")
    provider, staging = provider.resolve(), staging.resolve()
    if staging.is_relative_to(provider):
        raise ValueError("artifact consumer must be outside the provider checkout")
    (staging / "build-tmp").mkdir(exist_ok=True)
    wrapped = [bubblewrap, "--die-with-parent", "--ro-bind", "/", "/",
               "--bind", str(staging), str(staging),
               "--proc", "/proc", "--dev", "/dev"]
    cargo_home = Path(os.environ.get("CARGO_HOME", str(Path.home() / ".cargo"))).resolve()
    if cargo_home.exists():
        if cargo_home.is_relative_to(provider):
            raise ValueError("Cargo cache must be outside the masked provider checkout")
        wrapped.extend(["--bind", str(cargo_home), str(cargo_home)])
    # A cache can be an ancestor of the provider. Apply the mask after every
    # writable bind so rebinding that ancestor cannot restore source access.
    wrapped.extend(["--tmpfs", str(provider),
                    "--setenv", "CARGO_TARGET_DIR", str(staging / "target"),
                    "--setenv", "CARGO_NET_OFFLINE", "true",
                    "--setenv", "TMPDIR", str(staging / "build-tmp"), *command])
    return wrapped


def unpack_artifact(archive_path: Path, staging: Path, name: str, version: str) -> Path:
    """Unpack a bounded package without links, duplicate members or path escape."""
    expected_root = f"{name}-{version}"
    destination = staging / expected_root
    seen: set[str] = set()
    total = 0
    with tarfile.open(archive_path, "r:gz") as archive:
        for member in archive:
            # Tar directories may have one trailing slash; otherwise require
            # canonical spelling before PurePosixPath can collapse aliases.
            spelling = member.name[:-1] if member.isdir() and member.name.endswith("/") else member.name
            path = PurePosixPath(spelling)
            if (path.is_absolute() or any(part in {"", ".", ".."} for part in spelling.split("/"))
                    or not path.parts or path.parts[0] != expected_root
                    or "\\" in member.name or "\x00" in member.name):
                raise ValueError(f"invalid artifact path: {member.name}")
            canonical = path.as_posix()
            if canonical in seen:
                raise ValueError(f"duplicate artifact member: {member.name}")
            seen.add(canonical)
            if len(seen) > 10_000:
                raise ValueError("artifact exceeds the bounded package size")
            if member.isdir():
                continue
            if not member.isfile():
                raise ValueError(f"artifact member is not a regular file: {member.name}")
            total += member.size
            if member.size < 0 or total > 64 * 1024 * 1024:
                raise ValueError("artifact exceeds the bounded package size")
            output = staging.joinpath(*path.parts)
            output.parent.mkdir(parents=True, exist_ok=True)
            content = archive.extractfile(member)
            if content is None:
                raise ValueError(f"missing artifact member content: {member.name}")
            with output.open("wb") as target:
                shutil.copyfileobj(content, target)
    for required in ("Cargo.toml", "LICENSE", "NOTICE", "src/lib.rs"):
        if not (destination / required).is_file():
            raise ValueError(f"artifact lacks required {required}")
    package = tomllib.loads((destination / "Cargo.toml").read_text())["package"]
    if package["name"] != name or package["version"] != version:
        raise ValueError("artifact manifest identity does not match its requested package")
    return destination


def build_library_archives(cargo: str, provider: Path, assembly: Path, names: list[str]) -> str:
    """Package unpublished libraries without claiming a registry install lock.

    Current Cargo generates an archive lock even for libraries and tries to
    resolve unpublished sibling packages through the registry. Cargo 1.75
    omits that library lock. Where supported, explicitly retain that omission;
    the qualification separately records the provider lock and checks each
    offline consumer's resolved lock against it. Nothing is published.
    """
    help_text = run([cargo, "package", "--help"], provider, echo=False)
    lock_flags = ["--exclude-lockfile"] if "--exclude-lockfile" in help_text else []
    for name in names:
        run([cargo, "package", "--offline", "--locked", "--no-verify", "--allow-dirty",
             *lock_flags, "-p", name, "--target-dir", str(assembly)], provider)
    return "explicit-library-lock-omission" if lock_flags else "cargo-library-default"


def artifact_qualification(cargo: str) -> dict:
    """Build actual packages and execute each advertised feature profile."""
    package_changes = run(
        ["git", "status", "--porcelain", "--untracked-files=all", "--",
         "Cargo.toml", "Cargo.lock", "crates"], ROOT, echo=False,
    )
    if package_changes.strip():
        raise ValueError("commit the package sources/manifests/lock before artifact qualification")
    metadata = json.loads(run(
        [cargo, "metadata", "--format-version", "1", "--locked", "--offline", "--no-deps"],
        ROOT, echo=False,
    ))
    selected = {p["name"]: p for p in metadata["packages"] if p["name"] in PACKAGES}
    if set(selected) != set(PACKAGES):
        raise ValueError("expected both native packages before packaging")
    provider_lock = (ROOT / "Cargo.lock").read_bytes()
    locked = {(p["name"], p["version"], p.get("source"))
              for p in tomllib.loads(provider_lock.decode())["package"]}
    receipt: dict = {
        "scope": "local-cargo-artifact-consumption", "schema_version": 1,
        "registry_publication": False, "two_real_consumer_acceptance": False,
        "provider_revision": run(["git", "rev-parse", "HEAD"], ROOT, echo=False).strip(),
        "provider_tree_sha": run(["git", "rev-parse", "HEAD^{tree}"], ROOT, echo=False).strip(),
        "provider_worktree_dirty": bool(run(["git", "status", "--porcelain", "--untracked-files=all"], ROOT, echo=False).strip()),
        "package_inputs_match_revision": True,
        "provider_lock_sha256": hashlib.sha256(provider_lock).hexdigest(),
        "cargo_version": run([cargo, "--version"], ROOT, echo=False).strip(),
        "rustc_version": run(["rustc", "--version"], ROOT, echo=False).strip(),
        "artifacts": {}, "profiles": [],
        "provider_checkout_dependency": False,
        "source_isolation": "bubblewrap masks the provider checkout during offline resolution, compilation and tests",
    }
    with tempfile.TemporaryDirectory(prefix="paignia-artifacts-") as temporary:
        staging = Path(temporary).resolve()
        receipt["archive_lockfile_policy"] = build_library_archives(
            cargo, ROOT, staging / "assembly", list(PACKAGES)
        )
        extracted: dict[str, Path] = {}
        for name in PACKAGES:
            package = selected[name]
            archive = staging / "assembly/package" / f'{name}-{package["version"]}.crate'
            extracted[name] = unpack_artifact(archive, staging / "unpacked", name, package["version"])
            hashes: dict[str, str] = {}
            source = Path(package["manifest_path"]).parent
            for file in sorted(extracted[name].rglob("*")):
                if not file.is_file():
                    continue
                relative = file.relative_to(extracted[name]).as_posix()
                content = file.read_bytes()
                hashes[relative] = hashlib.sha256(content).hexdigest()
                if relative in {"Cargo.toml", ".cargo_vcs_info.json", "Cargo.lock"}:
                    continue
                original = source / ("Cargo.toml" if relative == "Cargo.toml.orig" else relative)
                if not original.is_file() or content != original.read_bytes():
                    raise ValueError(f"artifact source/provenance mismatch: {name}/{relative}")
            normalized = tomllib.loads((extracted[name] / "Cargo.toml").read_text())["package"]
            for field in ("license", "rust-version", "edition", "publish"):
                expected = package[{"rust-version": "rust_version"}.get(field, field)]
                if field == "publish" and expected == []:
                    expected = False
                if normalized.get(field) != expected:
                    raise ValueError(f"artifact changed package field: {name}/{field}")
            receipt["artifacts"][name] = {
                "version": package["version"], "archive_sha256": hashlib.sha256(archive.read_bytes()).hexdigest(),
                "files_sha256": hashes, "lock_in_archive": "Cargo.lock" in hashes,
            }
        for name in PACKAGES:
            features = sorted(feature for feature in selected[name]["features"] if feature != "default")
            profiles = [("minimal", False, []), ("default", True, []),
                        *[(f"feature:{feature}", False, [feature]) for feature in features],
                        ("all", True, features)]
            for index, (label, defaults, enabled) in enumerate(profiles):
                consumer = staging / f"consumer-{name}-{index}"
                (consumer / "src").mkdir(parents=True)
                (consumer / "tests").mkdir()
                (consumer / "src/lib.rs").write_text("//! Standalone artifact consumer.\n")
                dependencies = [name] if name == "paignia-core" else list(PACKAGES)
                lines = ['[package]', f'name = "artifact-{name}"', 'version = "0.0.0"',
                         'edition = "2021"', 'publish = false', '[workspace]', '[dependencies]']
                for dependency in dependencies:
                    lines.append(
                        f'{dependency} = {{ path = {json.dumps(str(extracted[dependency]))}, '
                        f'version = {json.dumps("=" + selected[dependency]["version"])}, '
                        f'default-features = {str(defaults if dependency == name else False).lower()}, '
                        f'features = {json.dumps(enabled if dependency == name else [])} }}'
                    )
                lines.extend(['[patch.crates-io]',
                              f'paignia-core = {{ path = {json.dumps(str(extracted["paignia-core"]))} }}'])
                (consumer / "Cargo.toml").write_text("\n".join(lines) + "\n")
                (consumer / "tests/artifact_consumer.rs").write_text(
                    ARTIFACT_CORE_TEST if name == "paignia-core" else ARTIFACT_SOLVER_TEST
                )
                shutil.copy2(ROOT / "Cargo.lock", consumer / "Cargo.lock")
                manifest = str(consumer / "Cargo.toml")
                def isolated(command: list[str], *, echo: bool = True) -> str:
                    return run(isolated_command(command, ROOT, staging), consumer, echo=echo)
                isolated([cargo, "generate-lockfile", "--offline", "--manifest-path", manifest])
                resolved_text = isolated([cargo, "metadata", "--format-version", "1", "--offline", "--locked",
                                          "--manifest-path", manifest], echo=False)
                resolved = json.loads(resolved_text)
                for package in resolved["packages"]:
                    if package["name"] == f"artifact-{name}":
                        continue
                    identity = (package["name"], package["version"], package.get("source"))
                    if identity not in locked:
                        raise ValueError(f"artifact consumer dependency drift: {identity}")
                    if package["name"] in PACKAGES:
                        expected = extracted[package["name"]] / "Cargo.toml"
                        if Path(package["manifest_path"]) != expected or package.get("source") is not None:
                            raise ValueError("consumer did not resolve the exact unpacked artifact")
                output = isolated([cargo, "test", "--offline", "--locked", "--manifest-path", manifest,
                                   "--test", "artifact_consumer"])
                count = 2 if name == "paignia-core" else 5
                require_native_tests(output, count)
                receipt["profiles"].append({
                    "package": name, "profile": label, "default_features": defaults, "features": enabled,
                    "tests_passed": count,
                    "consumer_lock_sha256": hashlib.sha256((consumer / "Cargo.lock").read_bytes()).hexdigest(),
                    "resolved_graph_sha256": hashlib.sha256(resolved_text.encode()).hexdigest(),
                    "resolved_dependencies": [{key: p.get(key) for key in ("name", "version", "source")}
                                              for p in resolved["packages"]],
                })
    return receipt


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


def source_smoke(cargo: str) -> dict:
    """Retain the historical weaker copy-based development check explicitly."""
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
        require_native_tests(output, 2)
        receipt["tests_passed"] = 2
        receipt["cargo_version"] = run([cargo, "--version"], consumer).strip()
        receipt["provider_checkout_dependency"] = False
        receipt["manifest_resolution"] = "workspace fields materialized; sibling path+version dependencies; offline lockfile"
    return receipt


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--receipt", type=Path, help="optional JSON result from successful native execution")
    parser.add_argument("--source-smoke", action="store_true", help="run only the historical copied-source smoke")
    args = parser.parse_args()
    cargo = shutil.which("cargo")
    if cargo is None:
        parser.exit(1, "cargo is required; native package consumer was not executed\n")
    receipt = source_smoke(cargo) if args.source_smoke else artifact_qualification(cargo)
    if args.receipt:
        args.receipt.parent.mkdir(parents=True, exist_ok=True)
        args.receipt.write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n")
    if args.source_smoke:
        print("Extracted source-package smoke passed (2 native tests; no artifact acceptance)")
    else:
        print(f'Cargo artifact qualification passed ({len(receipt["profiles"])} isolated profiles; no registry publication)')
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
