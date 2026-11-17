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
        profile_contract = self.reference("profile-contract.md", {"profile": self.profile})
        self.profile_hash = profile_contract["sha256"]
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
                          "consumer_lock_sha256": "f" * 64, "resolved_graph_sha256": "1" * 64,
                          "default_features": profile != "minimal", "features": [],
                          "resolved_dependencies": [{"name": name, "version": "0.1.0", "source": None}]}
                         for name in self.artifacts for profile in ("minimal", "default", "all")],
        }
        self.receipts = [self.consumer("first"), self.consumer("second")]
        self.document = {
            "schema_version": 1,
            "provider_revision": self.provider_revision,
            "semantic_profile": self.profile, "profile_sha256": self.profile_hash,
            "profile_contract": profile_contract,
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

    def test_git_suffix_alias_cannot_create_a_second_application(self) -> None:
        self.receipts[1]["repository"] = "https://github.com/EXAMPLE/FIRST.git/"
        self.assertTrue(self.validate())

    def test_provider_git_suffix_is_not_an_external_consumer(self) -> None:
        self.receipts[0]["repository"] = "https://github.com/edithatogo/paignia.git"
        self.assertTrue(self.validate())

    def test_an_invented_consistent_profile_is_not_admitted(self) -> None:
        self.document["semantic_profile"] = "invented/unsupported/999"
        for receipt in self.receipts:
            receipt["semantic_profile"] = self.document["semantic_profile"]
        self.assertTrue(self.validate())

    def test_profile_hash_must_bind_the_actual_contract_content(self) -> None:
        (self.root / "profile-contract.md").write_text("changed profile contract")
        self.assertTrue(self.validate())

    def test_hash_bound_failed_or_unrelated_method_is_not_support(self) -> None:
        self.document["capabilities"] = ["mixed_nash"]
        self.document["scientific_obligations"]["S02"] = {
            "status": "supported", "owner": "paignia",
            "evidence": self.reference("failed-method.json", {"schema_version": 1, "result": "failed", "tests_passed": 0, "method": "unrelated"}),
        }
        self.assertTrue(self.validate())

    def test_all_features_require_each_individual_profile(self) -> None:
        self.package["profiles"][2]["features"] = ["serialization"]
        self.assertTrue(self.validate())

    def test_feature_profiles_require_exact_flags_and_feature_lists(self) -> None:
        self.package["profiles"][0]["default_features"] = True
        self.assertTrue(self.validate())

    def test_malformed_method_status_returns_failure_without_raising(self) -> None:
        for status in ([], {}, 3, None):
            with self.subTest(status=status):
                self.document["scientific_obligations"]["S01"]["status"] = status
                self.assertTrue(self.validate())

    def test_invalid_path_bytes_return_failure_without_raising(self) -> None:
        self.receipts[0]["execution_evidence"]["path"] = "invalid\x00path"
        self.assertTrue(self.validate())

    def test_runtime_dependency_leak_cannot_be_a_qualified_static_package(self) -> None:
        self.package["profiles"][0]["resolved_dependencies"].append(
            {"name": "kairo-time", "version": "1.0.0", "source": None}
        )
        self.assertTrue(self.validate())

    def method_qualification(self) -> dict:
        return {
            "schema_version": 1, "scope": "scientific-method-qualification",
            "obligation": "S02", "capabilities": ["mixed_nash"],
            "provider_revision": self.provider_revision, "semantic_profile": self.profile,
            "profile_sha256": self.profile_hash, "result": "passed",
            "tests_passed": 4, "tests_failed": 0, "tests_skipped": 0,
            "commands": ["cargo test --locked --test independent_nash_reference"],
            "execution_evidence": self.reference("method-execution.json", {"passed": 4}),
            "independent_provenance": {
                "kind": "analytic-reference", "source": "synthetic independent exact equations",
                "version": "fixture-v1", "evidence": self.reference("method-reference.json", {"rational_witness": ["1/2", "1/2"]}),
            },
        }

    def admit_method(self, method: dict) -> list[str]:
        self.document["capabilities"] = ["mixed_nash"]
        self.document["scientific_obligations"]["S02"] = {
            "status": "supported", "owner": "paignia",
            "evidence": self.reference("method.json", method),
        }
        return self.validate()

    def test_typed_matching_method_qualification_is_structurally_admitted(self) -> None:
        self.assertEqual(self.admit_method(self.method_qualification()), [])

    def test_method_identity_status_counts_and_provenance_must_match(self) -> None:
        cases = (
            ("schema_version", 2), ("scope", "opaque-log"), ("obligation", "S01"),
            ("capabilities", ["cfr"]), ("capabilities", []),
            ("provider_revision", "3" * 40), ("semantic_profile", "unsupported"),
            ("profile_sha256", "4" * 64), ("result", "failed"),
            ("tests_passed", 0), ("tests_passed", True), ("tests_failed", 1),
            ("tests_skipped", 1), ("commands", []), ("execution_evidence", None),
            ("independent_provenance", None),
        )
        for field, value in cases:
            with self.subTest(field=field, value=value):
                method = self.method_qualification()
                method[field] = value
                self.assertTrue(self.admit_method(method))

    def test_independent_reference_requires_source_version_and_content(self) -> None:
        for field, value in (("kind", []), ("source", ""), ("version", ""), ("evidence", None)):
            with self.subTest(field=field):
                method = self.method_qualification()
                method["independent_provenance"][field] = value
                self.assertTrue(self.admit_method(method))

    def test_production_provider_is_not_its_own_external_method_reference(self) -> None:
        method = self.method_qualification()
        method["independent_provenance"].update(
            kind="external-implementation", source="https://github.com/EDITHATOGO/PAIGNIA.git/"
        )
        self.assertTrue(self.admit_method(method))

    def test_complete_individual_and_combined_feature_matrix_is_admitted(self) -> None:
        self.package["profiles"][2]["features"] = ["serialization"]
        self.package["profiles"].append({
            **self.package["profiles"][0], "profile": "feature:serialization",
            "default_features": False, "features": ["serialization"],
        })
        self.assertEqual(self.validate(), [])

    def test_feature_shapes_and_dependency_shapes_fail_without_raising(self) -> None:
        for field, value in (("features", {}), ("features", [True]),
                             ("features", ["x", "x"]), ("default_features", 0),
                             ("resolved_dependencies", None), ("resolved_dependencies", [{}])):
            with self.subTest(field=field, value=value):
                original = self.package["profiles"][0][field]
                self.package["profiles"][0][field] = value
                self.assertTrue(self.validate())
                self.package["profiles"][0][field] = original

    def test_cfr_receipt_cannot_silently_qualify_cfr_plus(self) -> None:
        method = self.method_qualification()
        method.update(obligation="S04", capabilities=["cfr"])
        self.document["capabilities"] = ["cfr_plus"]
        self.document["scientific_obligations"]["S04"] = {
            "status": "supported", "owner": "paignia",
            "evidence": self.reference("method.json", method),
        }
        self.assertTrue(self.validate())


if __name__ == "__main__":
    unittest.main()
