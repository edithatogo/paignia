"""Independent synthetic receipts test admission, never real consumer completion."""

from __future__ import annotations

import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest


SPEC = importlib.util.spec_from_file_location(
    "consumer_evidence", Path(__file__).resolve().parents[1] / "scripts/validate_game_consumer_evidence.py"
)
assert SPEC and SPEC.loader
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class EvidenceTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.provider_revision = "a" * 40
        self.profile = "paignia/static-finite-perfect-information-sparse-zero/1.0.0"
        self.profile_hash = "b" * 64
        self.artifacts = {"paignia-core": "c" * 64, "paignia-solve": "d" * 64}
        self.package = {
            "schema_version": 1, "scope": "local-cargo-artifact-consumption",
            "provider_revision": self.provider_revision, "provider_checkout_dependency": False,
            "provider_worktree_dirty": False, "package_inputs_match_revision": True,
            "provider_tree_sha": "9" * 40, "provider_lock_sha256": "e" * 64,
            "cargo_version": "cargo 1.75.0", "rustc_version": "rustc 1.75.0",
            "artifacts": {name: {"version": "0.1.0", "archive_sha256": digest}
                          for name, digest in self.artifacts.items()},
            "profiles": [{"package": name, "profile": profile, "tests_passed": 2,
                          "consumer_lock_sha256": "f" * 64, "resolved_graph_sha256": "1" * 64}
                         for name in self.artifacts for profile in ("minimal", "default", "all")],
        }
        self.receipts = [self.consumer("first"), self.consumer("second")]
        self.document = {
            "schema_version": 1,
            "provider_revision": self.provider_revision,
            "semantic_profile": self.profile, "profile_sha256": self.profile_hash,
            "artifacts": self.artifacts,
            "capabilities": ["backward_induction", "pure_nash"],
            "scientific_obligations": {key: {"status": "pending", "owner": "paignia",
                                           "replacement_plan": f"T17.{index + 4:02d}"}
                                       for index, key in enumerate(("S01", "S02", "S03", "S04"))},
        }

    def reference(self, name: str, value: object) -> dict[str, str]:
        data = json.dumps(value, sort_keys=True).encode()
        (self.root / name).write_bytes(data)
        return {"path": name, "sha256": hashlib.sha256(data).hexdigest()}

    def consumer(self, name: str) -> dict:
        return {
            "schema_version": 1, "scope": "real-application-artifact-consumer",
            "repository": f"https://github.com/example/{name}", "revision": "2" * 40,
            "provider_revision": self.provider_revision, "semantic_profile": self.profile,
            "profile_sha256": self.profile_hash, "artifacts": self.artifacts.copy(),
            "case_kind": "application-model", "case_name": f"{name} model",
            "expected_result_basis": "Independent analytic payoff derivation",
            "result": "passed", "tests_passed": 3, "tests_failed": 0, "tests_skipped": 0,
            "commands": ["cargo test --locked --test application_model"],
            "execution_evidence": self.reference(f"{name}-execution.json", {"executed": 3, "failed": 0}),
        }

    def validate(self) -> list[str]:
        self.document["package_receipt"] = self.reference("package.json", self.package)
        self.document["consumer_receipts"] = [self.reference(f"consumer-{i}.json", receipt)
                                               for i, receipt in enumerate(self.receipts)]
        return MODULE.validate_evidence(self.document, self.root)

    def test_two_matching_real_application_records_are_structurally_admitted(self) -> None:
        self.assertEqual(self.validate(), [])

    def test_duplicate_repository_spelling_does_not_make_two_consumers(self) -> None:
        self.receipts[1]["repository"] = "https://github.com/EXAMPLE/FIRST/"
        self.assertTrue(self.validate())

    def test_toy_and_source_smoke_receipts_are_not_real_applications(self) -> None:
        for field, value in (("case_kind", "toy"), ("scope", "extracted-source-package-smoke")):
            with self.subTest(field=field):
                old = self.receipts[1][field]
                self.receipts[1][field] = value
                self.assertTrue(self.validate())
                self.receipts[1][field] = old

    def test_artifact_profile_and_provider_mismatches_reject(self) -> None:
        for field, value in (("provider_revision", "3" * 40), ("semantic_profile", "different"),
                             ("profile_sha256", "4" * 64), ("artifacts", {"paignia-core": "5" * 64})):
            with self.subTest(field=field):
                old = self.receipts[1][field]
                self.receipts[1][field] = value
                self.assertTrue(self.validate())
                self.receipts[1][field] = old

    def test_failed_zero_skipped_or_boolean_test_counts_reject(self) -> None:
        for field, value in (("result", "failed"), ("tests_passed", 0), ("tests_passed", True),
                             ("tests_failed", 1), ("tests_skipped", 1)):
            with self.subTest(field=field):
                old = self.receipts[1][field]
                self.receipts[1][field] = value
                self.assertTrue(self.validate())
                self.receipts[1][field] = old

    def test_missing_second_consumer_rejects(self) -> None:
        self.receipts.pop()
        self.assertTrue(self.validate())

    def test_dirty_source_or_incomplete_package_profiles_reject(self) -> None:
        self.package["provider_worktree_dirty"] = True
        self.assertTrue(self.validate())
        self.package["provider_worktree_dirty"] = False
        self.package["profiles"].pop()
        self.assertTrue(self.validate())

    def test_every_scientific_obligation_requires_a_disposition(self) -> None:
        for key in ("S01", "S02", "S03", "S04"):
            original = self.document["scientific_obligations"].pop(key)
            self.assertTrue(self.validate())
            self.document["scientific_obligations"][key] = original

    def test_pending_method_cannot_be_advertised(self) -> None:
        for method in ("replicator_rk4", "mixed_nash", "correlated_equilibrium", "cfr"):
            self.document["capabilities"] = [method]
            self.assertTrue(self.validate())

    def test_corrupted_or_missing_content_bound_execution_evidence_rejects(self) -> None:
        path = self.root / self.receipts[0]["execution_evidence"]["path"]
        path.write_text("tampered")
        self.assertTrue(self.validate())
        path.unlink()
        self.assertTrue(self.validate())

    def test_references_cannot_escape_the_evidence_root(self) -> None:
        self.receipts[0]["execution_evidence"]["path"] = "../outside.json"
        self.assertTrue(self.validate())

    def test_unknown_schema_and_malformed_receipt_fail_closed(self) -> None:
        self.document["schema_version"] = 2
        self.assertTrue(self.validate())
        self.document["schema_version"] = 1
        self.receipts[0] = []
        self.assertTrue(self.validate())

    def test_malformed_artifact_map_reports_failure_instead_of_crashing(self) -> None:
        for value in (None, ["paignia-core"], "not a mapping"):
            with self.subTest(value=value):
                self.document["artifacts"] = value
                self.assertTrue(self.validate())


if __name__ == "__main__":
    unittest.main()
