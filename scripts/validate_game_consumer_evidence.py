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
        relative = Path(reference["path"])
        candidate = (root / relative).resolve()
        if relative.is_absolute() or ".." in relative.parts or not candidate.is_relative_to(root.resolve()):
            fail(f"{label}: receipt path escapes the evidence root")
            return None
        try:
            data = candidate.read_bytes()
            if not data or hashlib.sha256(data).hexdigest() != reference["sha256"]:
                raise ValueError("content hash mismatch or empty evidence")
            if not json_required:
                return data
            result = json.loads(data)
            if not isinstance(result, dict) or type(result.get("schema_version")) is not int or result["schema_version"] != 1:
                raise ValueError("unsupported or malformed receipt schema")
            return result
        except (OSError, ValueError) as error:
            fail(f"{label}: {error}")
            return None

    if not isinstance(document, dict) or type(document.get("schema_version")) is not int or document["schema_version"] != 1:
        return ["unsupported or malformed eligibility schema"]
    if not digest(document.get("provider_revision"), 40):
        fail("provider_revision: an immutable qualified revision is required")
    if not text(document.get("semantic_profile")) or not digest(document.get("profile_sha256")):
        fail("semantic profile: exact identity and content hash are required")
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
                if not positive(item.get("tests_passed")) or not digest(item.get("consumer_lock_sha256")) or not digest(item.get("resolved_graph_sha256")):
                    fail("package: profile needs nonzero native execution and resolved lock/graph hashes")
            if not {(name, profile) for name in PACKAGES for profile in ("minimal", "default", "all")} <= seen:
                fail("package: minimal/default/all feature-profile evidence is incomplete")

    obligations = document.get("scientific_obligations")
    if not isinstance(obligations, dict) or set(obligations) != METHODS:
        fail("scientific obligations: S01, S02, S03 and S04 must each be recorded")
        obligations = {}
    for name, item in obligations.items():
        if not isinstance(item, dict) or item.get("status") not in {"pending", "unsupported", "supported", "withdrawn"} or not text(item.get("owner")):
            fail(f"{name}: explicit status and owner are required")
            continue
        if item["status"] == "supported":
            receipt(item.get("evidence"), name, json_required=False)
        elif not text(item.get("replacement_plan")):
            fail(f"{name}: pending or withdrawn methods retain a replacement plan")
    method_ids = {"replicator_rk4": "S01", "mixed_nash": "S02", "correlated_equilibrium": "S03", "cfr": "S04", "cfr_plus": "S04"}
    capabilities = document.get("capabilities")
    if not isinstance(capabilities, list) or not capabilities or any(not text(value) for value in capabilities):
        fail("capabilities: a nonempty explicit supported-method list is required")
    else:
        for capability in capabilities:
            if capability in {"backward_induction", "pure_nash"}:
                continue
            obligation = obligations.get(method_ids.get(capability, ""))
            if not isinstance(obligation, dict) or obligation.get("status") != "supported":
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
        repository = consumer.get("repository")
        canonical = repository.rstrip("/").lower() if isinstance(repository, str) else ""
        if not re.fullmatch(r"https://github\.com/[a-z0-9_.-]+/[a-z0-9_.-]+", canonical) or canonical == "https://github.com/edithatogo/paignia":
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
        if consumer.get("result") != "passed" or not positive(consumer.get("tests_passed")) or any(type(consumer.get(field)) is not int or consumer[field] != 0 for field in ("tests_failed", "tests_skipped")):
            fail(f"{label}: nonzero passing application tests without failures/skips are required")
        commands = consumer.get("commands")
        if not isinstance(commands, list) or not commands or not all(text(command) for command in commands):
            fail(f"{label}: executed commands are required")
        receipt(consumer.get("execution_evidence"), label + " execution", json_required=False)
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
    except (OSError, ValueError, TypeError) as error:
        errors = [str(error)]
    print(json.dumps({"schema_version": 1, "eligible": not errors, "errors": errors,
                      "registry_publication": False, "full_track_complete": False}, indent=2))
    return int(bool(errors))


if __name__ == "__main__":
    raise SystemExit(main())
