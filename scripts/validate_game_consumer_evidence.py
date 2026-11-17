#!/usr/bin/env python3
"""Validate content-bound package and application receipts for bounded eligibility.

This is evidence admission, not authentication of an external repository or a
mathematical proof. Receiving owners must review the referenced commands/results.
Registry publication and full-track archival are never performed by this tool.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
PACKAGES = {"paignia-core", "paignia-solve"}
METHODS = {"S01", "S02", "S03", "S04"}
STATIC_PROFILE = "paignia/static-finite-perfect-information-sparse-zero/1.0.0"
METHOD_CAPABILITIES = {
    "S01": {"replicator_rk4"}, "S02": {"mixed_nash"},
    "S03": {"correlated_equilibrium"}, "S04": {"cfr", "cfr_plus"},
}


def canonical_repository(value: object) -> str | None:
    """Normalize GitHub clone/web spellings before counting applications."""
    if not isinstance(value, str):
        return None
    canonical = value.rstrip("/").lower()
    if canonical.endswith(".git"):
        canonical = canonical[:-4]
    if not re.fullmatch(r"https://github\.com/[a-z0-9-]+/[a-z0-9_.-]+", canonical):
        return None
    if canonical.rsplit("/", 1)[1] in {".", ".."} or canonical.endswith(".git"):
        return None
    return canonical


def validate_evidence(document: dict[str, Any], root: Path) -> list[str]:
    """Return every material admission failure; an empty list means eligible."""
    errors: list[str] = []

    def fail(message: str) -> None:
        errors.append(message)

    def text(value: object) -> bool:
        return isinstance(value, str) and bool(value.strip())

    def digest(value: object, length: int = 64) -> bool:
        return isinstance(value, str) and re.fullmatch(rf"[0-9a-f]{{{length}}}", value) is not None

    def positive(value: object) -> bool:
        return type(value) is int and value > 0

    def receipt(reference: object, label: str, *, json_required: bool = True) -> dict[str, Any] | bytes | None:
        if not isinstance(reference, dict) or not text(reference.get("path")) or not digest(reference.get("sha256")):
            fail(f"{label}: missing content-bound receipt")
            return None
        try:
            relative = Path(reference["path"])
            candidate = (root / relative).resolve()
            if relative.is_absolute() or ".." in relative.parts or not candidate.is_relative_to(root.resolve()):
                raise ValueError("receipt path escapes the evidence root")
            data = candidate.read_bytes()
            if not data or hashlib.sha256(data).hexdigest() != reference["sha256"]:
                raise ValueError("content hash mismatch or empty evidence")
            if not json_required:
                return data
            result = json.loads(data)
            if not isinstance(result, dict) or type(result.get("schema_version")) is not int or result["schema_version"] != 1:
                raise ValueError("unsupported or malformed receipt schema")
            return result
        except (OSError, ValueError, RuntimeError) as error:
            fail(f"{label}: {error}")
            return None

    def passing_execution(record: dict[str, Any], label: str) -> None:
        if record.get("result") != "passed" or not positive(record.get("tests_passed")) or any(
            type(record.get(field)) is not int or record[field] != 0
            for field in ("tests_failed", "tests_skipped")
        ):
            fail(f"{label}: nonzero passing tests without failures/skips are required")
        commands = record.get("commands")
        if not isinstance(commands, list) or not commands or not all(text(command) for command in commands):
            fail(f"{label}: executed commands are required")
        receipt(record.get("execution_evidence"), label + " execution", json_required=False)

    if not isinstance(document, dict) or type(document.get("schema_version")) is not int or document["schema_version"] != 1:
        return ["unsupported or malformed eligibility schema"]
    if not digest(document.get("provider_revision"), 40):
        fail("provider_revision: an immutable qualified revision is required")
    if not text(document.get("semantic_profile")) or not digest(document.get("profile_sha256")):
        fail("semantic profile: exact identity and content hash are required")
    if document.get("semantic_profile") != STATIC_PROFILE:
        fail("semantic profile: unsupported static profile identity")
    contract_reference = document.get("profile_contract")
    contract = receipt(contract_reference, "semantic profile contract", json_required=False)
    if isinstance(contract, bytes):
        if not isinstance(contract_reference, dict) or contract_reference.get("sha256") != document.get("profile_sha256"):
            fail("semantic profile: profile hash does not bind the referenced contract")
        try:
            if STATIC_PROFILE not in contract.decode("utf-8"):
                fail("semantic profile: contract does not name the admitted profile")
        except UnicodeDecodeError:
            fail("semantic profile: contract must be UTF-8 text")
    artifacts = document.get("artifacts")
    if not isinstance(artifacts, dict) or set(artifacts) != PACKAGES or not all(digest(value) for value in artifacts.values()):
        fail("artifacts: both qualified crate archive hashes are required")
    expected_artifacts = artifacts if isinstance(artifacts, dict) else {}

    package = receipt(document.get("package_receipt"), "package")
    if isinstance(package, dict):
        if package.get("scope") != "local-cargo-artifact-consumption":
            fail("package: extracted source or registry declarations are not artifact consumption")
        if package.get("provider_revision") != document.get("provider_revision"):
            fail("package: provider revision mismatch")
        if package.get("provider_checkout_dependency") is not False:
            fail("package: checkout isolation is not established")
        if package.get("provider_worktree_dirty") is not False or package.get("package_inputs_match_revision") is not True:
            fail("package: source must match the recorded immutable revision")
        if not digest(package.get("provider_tree_sha"), 40) or not digest(package.get("provider_lock_sha256")):
            fail("package: provider tree and lock hashes are required")
        if not text(package.get("cargo_version")) or not text(package.get("rustc_version")):
            fail("package: compiler and Cargo versions are required")
        archive_records = package.get("artifacts")
        if not isinstance(archive_records, dict) or set(archive_records) != PACKAGES:
            fail("package: both crate artifact records are required")
        elif any(not isinstance(item, dict) or not text(item.get("version")) or
                 item.get("archive_sha256") != expected_artifacts.get(name)
                 for name, item in archive_records.items()):
            fail("package: crate artifact identity mismatch")
        profiles = package.get("profiles")
        seen: set[tuple[str, str]] = set()
        declared_features: dict[str, set[str]] = {}
        if not isinstance(profiles, list):
            fail("package: feature-profile execution records are required")
        else:
            for item in profiles:
                if not isinstance(item, dict) or not text(item.get("package")) or not text(item.get("profile")):
                    fail("package: malformed feature-profile record")
                    continue
                identity = (item["package"], item["profile"])
                if identity in seen:
                    fail("package: duplicate feature-profile record")
                seen.add(identity)
                if item["package"] not in PACKAGES:
                    fail("package: unknown package in feature-profile record")
                enabled = item.get("features")
                if not isinstance(enabled, list) or not all(text(feature) for feature in enabled):
                    fail("package: feature names must be an explicit string list")
                    continue
                if len(enabled) != len(set(enabled)) or "default" in enabled:
                    fail("package: duplicate or reserved explicit feature")
                profile = item["profile"]
                expected_defaults = profile in {"default", "all"}
                if type(item.get("default_features")) is not bool or item["default_features"] is not expected_defaults:
                    fail("package: feature-profile default flag mismatch")
                if profile == "all":
                    declared_features[item["package"]] = set(enabled)
                elif profile in {"minimal", "default"}:
                    if enabled:
                        fail("package: minimal/default profiles cannot add explicit features")
                elif profile.startswith("feature:") and text(profile[8:]):
                    if enabled != [profile[8:]]:
                        fail("package: individual feature record must enable only its named feature")
                else:
                    fail("package: unknown feature-profile identity")
                dependencies = item.get("resolved_dependencies")
                if not isinstance(dependencies, list) or not dependencies:
                    fail("package: resolved dependency identities are required")
                else:
                    dependency_names: set[str] = set()
                    for dependency in dependencies:
                        if not isinstance(dependency, dict) or not text(dependency.get("name")) or not text(dependency.get("version")) or not (
                            dependency.get("source") is None or text(dependency.get("source"))
                        ):
                            fail("package: malformed resolved dependency identity")
                            continue
                        dependency_name = dependency["name"]
                        dependency_names.add(dependency_name)
                        normalized = dependency_name.lower().replace("_", "-")
                        if normalized in {"kairo", "kairos", "voiage", "paignia-kairos"} or normalized.startswith(("kairo-", "kairos-", "voiage-")):
                            fail("package: simulation/valuation dependency leaks into a static profile")
                        if dependency_name in PACKAGES and isinstance(archive_records, dict):
                            artifact = archive_records.get(dependency_name)
                            if not isinstance(artifact, dict) or dependency["version"] != artifact.get("version") or dependency.get("source") is not None:
                                fail("package: resolved native dependency does not match the unpacked artifact")
                    if item["package"] not in dependency_names:
                        fail("package: feature profile omits its resolved native package")
                if not positive(item.get("tests_passed")) or not digest(item.get("consumer_lock_sha256")) or not digest(item.get("resolved_graph_sha256")):
                    fail("package: profile needs nonzero native execution and resolved lock/graph hashes")
            if not {(name, profile) for name in PACKAGES for profile in ("minimal", "default", "all")} <= seen:
                fail("package: minimal/default/all feature-profile evidence is incomplete")
            for name, features in declared_features.items():
                expected_profiles = {"minimal", "default", "all"} | {f"feature:{feature}" for feature in features}
                actual_profiles = {profile for package_name, profile in seen if package_name == name}
                if actual_profiles != expected_profiles:
                    fail(f"package: {name} individual and all-feature evidence do not agree")

    obligations = document.get("scientific_obligations")
    if not isinstance(obligations, dict) or set(obligations) != METHODS:
        fail("scientific obligations: S01, S02, S03 and S04 must each be recorded")
        obligations = {}
    qualified_capabilities: set[str] = set()
    for name, item in obligations.items():
        if not isinstance(item, dict) or not text(item.get("status")) or item["status"] not in {"pending", "unsupported", "supported", "withdrawn"} or not text(item.get("owner")):
            fail(f"{name}: explicit status and owner are required")
            continue
        if item["status"] == "supported":
            method = receipt(item.get("evidence"), name)
            if not isinstance(method, dict):
                continue
            if method.get("scope") != "scientific-method-qualification" or method.get("obligation") != name:
                fail(f"{name}: a typed qualification receipt for this obligation is required")
            for field in ("provider_revision", "semantic_profile", "profile_sha256"):
                if method.get(field) != document.get(field):
                    fail(f"{name}: {field} mismatch")
            method_capabilities = method.get("capabilities")
            if (not isinstance(method_capabilities, list) or not method_capabilities
                    or not all(text(capability) for capability in method_capabilities)):
                fail(f"{name}: exact qualified capabilities are required")
            elif len(method_capabilities) != len(set(method_capabilities)) or not set(method_capabilities) <= METHOD_CAPABILITIES[name]:
                fail(f"{name}: qualification names an unrelated or duplicate capability")
            else:
                qualified_capabilities.update(method_capabilities)
            passing_execution(method, name)
            provenance = method.get("independent_provenance")
            if (not isinstance(provenance, dict) or not text(provenance.get("kind"))
                    or provenance["kind"] not in {"analytic-reference", "external-implementation", "independent-review"}
                    or not text(provenance.get("source")) or not text(provenance.get("version"))):
                fail(f"{name}: versioned independent reference provenance is required")
            else:
                if provenance["kind"] == "external-implementation" and canonical_repository(provenance["source"]) == "https://github.com/edithatogo/paignia":
                    fail(f"{name}: the production provider is not an independent external implementation")
                receipt(provenance.get("evidence"), name + " independent reference", json_required=False)
        elif not text(item.get("replacement_plan")):
            fail(f"{name}: pending or withdrawn methods retain a replacement plan")
    capabilities = document.get("capabilities")
    if not isinstance(capabilities, list) or not capabilities or any(not text(value) for value in capabilities):
        fail("capabilities: a nonempty explicit supported-method list is required")
    else:
        for capability in capabilities:
            if capability in {"backward_induction", "pure_nash"}:
                continue
            if capability not in qualified_capabilities:
                fail(f"capability {capability!r}: no supported scientific-method receipt")

    references = document.get("consumer_receipts")
    repositories: set[str] = set()
    if not isinstance(references, list):
        fail("consumers: receipt list is required")
        references = []
    for number, reference in enumerate(references):
        label = f"consumer {number + 1}"
        consumer = receipt(reference, label)
        if not isinstance(consumer, dict):
            continue
        canonical = canonical_repository(consumer.get("repository"))
        if canonical is None or canonical == "https://github.com/edithatogo/paignia":
            fail(f"{label}: a distinct external application repository is required")
        elif canonical in repositories:
            fail(f"{label}: duplicate application repository")
        else:
            repositories.add(canonical)
        if consumer.get("scope") != "real-application-artifact-consumer" or consumer.get("case_kind") != "application-model":
            fail(f"{label}: toy, wrapper-only and source-smoke cases do not qualify")
        for field in ("provider_revision", "semantic_profile", "profile_sha256", "artifacts"):
            if consumer.get(field) != document.get(field):
                fail(f"{label}: {field} mismatch")
        if not digest(consumer.get("revision"), 40) or not text(consumer.get("case_name")) or not text(consumer.get("expected_result_basis")):
            fail(f"{label}: immutable application revision, actual case and independent expectation are required")
        passing_execution(consumer, label)
    if len(repositories) < 2:
        fail("consumers: two distinct real application receipts are required")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, default=ROOT / "docs/validation/game-consumers.json")
    args = parser.parse_args()
    try:
        document = json.loads(args.manifest.read_text(encoding="utf-8"))
        errors = validate_evidence(document, ROOT)
    except (OSError, ValueError, TypeError, RuntimeError) as error:
        errors = [str(error)]
    print(json.dumps({"schema_version": 1, "eligible": not errors, "errors": errors,
                      "registry_publication": False, "full_track_complete": False}, indent=2))
    return int(bool(errors))


if __name__ == "__main__":
    raise SystemExit(main())
