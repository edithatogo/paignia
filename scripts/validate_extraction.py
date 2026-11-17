#!/usr/bin/env python3
"""Validate provenance and the resolved normal/build static dependency closure.

Each generic crate is resolved as an isolated consumer dependency for minimal,
default, individual and combined features. Cargo's package identities, not
manifest aliases, determine whether a simulation or valuation runtime leaked in.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import shutil
import subprocess
import tempfile
import tomllib
from pathlib import Path


def cargo_metadata(cargo: str, manifest: Path, *, locked: bool = True) -> dict:
    """Read actual resolved metadata, retaining all target-specific edges."""
    command = [cargo, "metadata", "--format-version", "1", "--offline",
               "--manifest-path", str(manifest)]
    if locked:
        command.append("--locked")
    result = subprocess.run(command, cwd=manifest.parent, check=True,
                            text=True, capture_output=True)
    return json.loads(result.stdout)


def dependency_closure(metadata: dict, root_id: str) -> tuple[list[dict], list[str]]:
    """Traverse normal/build edges, rejecting incomplete or unknown graph data."""
    resolve = metadata.get("resolve")
    if not isinstance(resolve, dict) or not isinstance(resolve.get("nodes"), list):
        raise ValueError("Cargo did not supply a resolved dependency graph")
    packages = {package["id"]: package for package in metadata["packages"]}
    nodes = {node["id"]: node for node in resolve["nodes"]}
    if root_id not in packages or root_id not in nodes:
        raise ValueError("static root is absent from the resolved dependency graph")
    pending = [(root_id, [])]
    visited: set[str] = set()
    closure: list[dict] = []
    errors: list[str] = []
    while pending:
        package_id, ancestors = pending.pop()
        if package_id in visited:
            continue
        visited.add(package_id)
        if package_id not in packages or package_id not in nodes:
            raise ValueError(f"incomplete resolved dependency: {package_id}")
        package = packages[package_id]
        name = package["name"]
        path = [*ancestors, name]
        canonical = name.replace("_", "-").lower()
        if (canonical in {"kairo", "kairos", "voiage", "paignia-kairos"}
                or canonical.startswith(("kairo-", "kairos-", "voiage-"))):
            errors.append("forbidden static dependency: " + " -> ".join(path))
        closure.append({key: package.get(key) for key in ("id", "name", "version", "source")})
        for dependency in nodes[package_id]["deps"]:
            kinds = dependency.get("dep_kinds")
            if not isinstance(kinds, list) or not kinds:
                raise ValueError("resolved dependency has no dependency kind")
            if any(item.get("kind") not in (None, "normal", "build", "dev") for item in kinds):
                raise ValueError("unknown resolved dependency kind")
            if any(item.get("kind") != "dev" for item in kinds):
                pending.append((dependency["pkg"], path))
    return sorted(closure, key=lambda item: item["id"]), sorted(errors)


def feature_profiles(features: dict) -> list[tuple[str, bool, list[str]]]:
    """Check default and each public/implicit optional feature independently."""
    individual = sorted(name for name in features if name != "default")
    return [("minimal", False, []), ("default", True, []),
            *[(f"feature:{name}", False, [name]) for name in individual],
            ("all", True, individual)]


def validate_resolved_boundaries(root: Path, names: list[str], cargo: str) -> list[dict]:
    """Resolve independent consumers without workspace feature unification.

The committed provider lock seeds each consumer. Cargo may add the new probe
root but must preserve every provider dependency version; an exact second
`--locked` resolution then records the graph. Dev-only dependencies are not
part of a downstream package's runtime/build closure.
"""
    root = root.resolve()
    base = cargo_metadata(cargo, root / "Cargo.toml")
    packages = {package["name"]: package for package in base["packages"]
                if package["id"] in base["workspace_members"]}
    allowed = {(p["name"], p["version"], p.get("source")) for p in base["packages"]}
    # The lock covers optional dependencies even when the default graph does not.
    lock = tomllib.loads((root / "Cargo.lock").read_text())
    allowed.update((p["name"], p["version"], p.get("source")) for p in lock["package"])
    reports: list[dict] = []
    for name in names:
        package = packages.get(name)
        if package is None:
            raise ValueError(f"missing static package {name}")
        for profile, default_features, features in feature_profiles(package["features"]):
            with tempfile.TemporaryDirectory(prefix="paignia-boundary-") as temporary:
                probe = Path(temporary)
                (probe / "src").mkdir()
                (probe / "src/lib.rs").write_text("// Metadata-only dependency probe.\n")
                dependency_path = Path(package["manifest_path"]).parent
                (probe / "Cargo.toml").write_text(
                    '[package]\nname = "paignia-boundary-probe"\nversion = "0.0.0"\n'
                    'edition = "2021"\npublish = false\n[workspace]\n[dependencies]\n'
                    f'{name} = {{ path = {json.dumps(str(dependency_path))}, '
                    f'version = {json.dumps("=" + package["version"])}, '
                    f'default-features = {str(default_features).lower()}, '
                    f'features = {json.dumps(features)} }}\n'
                )
                shutil.copy2(root / "Cargo.lock", probe / "Cargo.lock")
                cargo_metadata(cargo, probe / "Cargo.toml", locked=False)
                metadata = cargo_metadata(cargo, probe / "Cargo.toml")
                root_ids = [p["id"] for p in metadata["packages"]
                            if p["name"] == name and Path(p["manifest_path"]) == Path(package["manifest_path"])]
                if len(root_ids) != 1:
                    raise ValueError(f"ambiguous static dependency root {name}")
                closure, errors = dependency_closure(metadata, root_ids[0])
                for resolved in closure:
                    identity = (resolved["name"], resolved["version"], resolved.get("source"))
                    if identity not in allowed:
                        errors.append(f"isolated dependency drifted outside the provider lock: {identity}")
                reports.append({
                    "root": name, "profile": profile, "default_features": default_features,
                    "features": features, "dependencies": closure, "errors": sorted(errors),
                    "lock_sha256": hashlib.sha256((probe / "Cargo.lock").read_bytes()).hexdigest(),
                })
    return reports


def main() -> int:
    """Validate the committed extraction manifest and crate boundaries."""

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--receipt", type=Path, help="write successful dependency evidence")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    manifest = json.loads((root / "extraction-manifest.json").read_text(encoding="utf-8"))
    errors: list[str] = []
    crates = [root / path for path in manifest["crates"]]
    declared_names: set[str] = set()
    dependencies: dict[str, set[str]] = {}
    for crate in crates:
        cargo_path = crate / "Cargo.toml"
        source_path = crate / "src"
        if not cargo_path.is_file() or not source_path.is_dir():
            errors.append(f"incomplete extraction crate: {crate.relative_to(root)}")
            continue
        cargo = tomllib.loads(cargo_path.read_text(encoding="utf-8"))
        name = cargo["package"]["name"]
        declared_names.add(name)
        dependencies[name] = set(cargo.get("dependencies", {}))
        provenance = manifest.get("source_provenance", {}).get(name, {})
        if provenance.get("current_path") != crate.relative_to(root).as_posix():
            errors.append(f"missing or mismatched source provenance: {name}")
        for dependency in manifest["allowed_internal_dependencies"].get(name, []):
            declaration = cargo.get("dependencies", {}).get(dependency, {})
            if not isinstance(declaration, dict) or not declaration.get("version"):
                errors.append(f"internal dependency needs a distribution version: {name} -> {dependency}")
        for source in source_path.rglob("*.rs"):
            content = source.read_text(encoding="utf-8").lower()
            for term in manifest["forbidden_vocabulary"]:
                if re.search(rf"\b{re.escape(term.lower())}\b", content):
                    errors.append(
                        f"forbidden extraction vocabulary {term!r}: {source.relative_to(root)}"
                    )
    expected = set(manifest["allowed_internal_dependencies"])
    if declared_names != expected:
        errors.append(
            f"manifest crate names differ: expected {sorted(expected)}, got {sorted(declared_names)}"
        )
    for crate, allowed in manifest["allowed_internal_dependencies"].items():
        internal = dependencies.get(crate, set()) & declared_names
        if internal != set(allowed):
            errors.append(
                f"unexpected internal dependencies for {crate}: {sorted(internal)}"
            )
    profiles: list[dict] = []
    if not errors:
        cargo = shutil.which("cargo")
        if cargo is None:
            errors.append("cargo is required; resolved static dependency checks were not executed")
        else:
            try:
                profiles = validate_resolved_boundaries(root, sorted(expected), cargo)
                errors.extend(f'{item["root"]}/{item["profile"]}: {error}'
                              for item in profiles for error in item["errors"])
            except (ValueError, subprocess.CalledProcessError) as error:
                errors.append(str(error))
                if isinstance(error, subprocess.CalledProcessError) and error.stderr:
                    errors.append(error.stderr.strip())
    if errors:
        print("extraction validation failed:")
        for error in errors:
            print(f"- {error}")
        return 1
    if args.receipt:
        args.receipt.parent.mkdir(parents=True, exist_ok=True)
        args.receipt.write_text(json.dumps({
            "scope": "resolved-static-normal-build-dependencies", "profiles": profiles,
            "provider_lock_sha256": hashlib.sha256((root / "Cargo.lock").read_bytes()).hexdigest(),
        }, indent=2, sort_keys=True) + "\n")
    print(f"extraction validation passed ({len(crates)} crates, {len(profiles)} isolated feature profiles)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
