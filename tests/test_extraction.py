"""Independent controls for the resolved static dependency boundary."""

from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location(
    "validate_extraction", ROOT / "scripts/validate_extraction.py"
)
assert SPEC is not None and SPEC.loader is not None
guard = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(guard)


class ResolvedDependencyTests(unittest.TestCase):
    """Use Cargo itself so alias, feature and edge-kind controls are not mocks."""

    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(prefix="paignia-boundary-test-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.cargo = shutil.which("cargo")
        if self.cargo is None:
            self.fail("cargo is required to execute resolved-graph negative controls")
        self.write_crate("paignia-core")
        self.write_crate("neutral-helper")
        self.write_crate("kairo-time")
        self.write_crate("voiage")
        (self.root / "Cargo.toml").write_text(
            '[workspace]\nresolver = "2"\nmembers = ["crates/*"]\n'
        )

    def write_crate(self, name: str, extra: str = "") -> None:
        path = self.root / "crates" / name
        (path / "src").mkdir(parents=True, exist_ok=True)
        (path / "src/lib.rs").write_text("pub fn available() {}\n")
        (path / "Cargo.toml").write_text(
            f'[package]\nname = {json.dumps(name)}\nversion = "0.1.0"\n'
            'edition = "2021"\n' + extra
        )

    def reports(self) -> list[dict]:
        subprocess.run(
            [self.cargo, "generate-lockfile", "--offline"], cwd=self.root,
            check=True, capture_output=True, text=True,
        )
        return guard.validate_resolved_boundaries(
            self.root, ["paignia-core"], self.cargo
        )

    def test_minimal_and_default_profiles_are_individually_resolved(self) -> None:
        reports = self.reports()
        self.assertEqual({r["profile"] for r in reports}, {"minimal", "default", "all"})
        self.assertTrue(all(r["errors"] == [] for r in reports))
        self.assertTrue(all(r["root"] == "paignia-core" for r in reports))

    def test_transitive_renamed_dependency_is_rejected(self) -> None:
        self.write_crate("paignia-core", '[dependencies]\nhelper = { package = "neutral-helper", path = "../neutral-helper" }\n')
        self.write_crate("neutral-helper", '[dependencies]\nclock = { package = "kairo-time", path = "../kairo-time" }\n')
        reports = self.reports()
        self.assertTrue(all(any("paignia-core -> neutral-helper -> kairo-time" in e for e in r["errors"]) for r in reports))

    def test_build_dependency_is_rejected(self) -> None:
        self.write_crate("paignia-core", '[build-dependencies]\nclock = { package = "kairo-time", path = "../kairo-time" }\n')
        self.assertTrue(all(r["errors"] for r in self.reports()))

    def test_transitive_build_dependency_is_rejected(self) -> None:
        self.write_crate("paignia-core", '[dependencies]\nneutral-helper = { path = "../neutral-helper" }\n')
        self.write_crate("neutral-helper", '[build-dependencies]\nvoiage = { path = "../voiage" }\n')
        self.assertTrue(all(r["errors"] for r in self.reports()))

    def test_dev_only_dependency_is_not_in_the_static_closure(self) -> None:
        self.write_crate("paignia-core", '[dev-dependencies]\nvoiage = { path = "../voiage" }\n')
        self.assertTrue(all(not r["errors"] for r in self.reports()))

    def test_optional_dependency_is_checked_separately(self) -> None:
        self.write_crate("paignia-core", '[dependencies]\nclock = { package = "kairo-time", path = "../kairo-time", optional = true }\n[features]\nsimulation = ["dep:clock"]\n')
        reports = {r["profile"]: r for r in self.reports()}
        self.assertFalse(reports["minimal"]["errors"])
        self.assertFalse(reports["default"]["errors"])
        self.assertTrue(reports["feature:simulation"]["errors"])
        self.assertTrue(reports["all"]["errors"])

    def test_platform_specific_dependency_is_not_hidden_on_linux(self) -> None:
        self.write_crate("paignia-core", '[target.\'cfg(windows)\'.dependencies]\nvoiage = { path = "../voiage" }\n')
        self.assertTrue(all(r["errors"] for r in self.reports()))

    def test_foreign_workspace_members_do_not_unify_features(self) -> None:
        self.write_crate("paignia-core", '[dependencies]\nclock = { package = "kairo-time", path = "../kairo-time", optional = true }\n[features]\nsimulation = ["dep:clock"]\n')
        self.write_crate("neutral-helper", '[dependencies]\npaignia-core = { path = "../paignia-core", features = ["simulation"] }\n')
        reports = {r["profile"]: r for r in self.reports()}
        self.assertFalse(reports["default"]["errors"])
        self.assertTrue(reports["feature:simulation"]["errors"])


class MalformedMetadataTests(unittest.TestCase):
    def test_absent_resolved_graph_is_rejected(self) -> None:
        with self.assertRaisesRegex(ValueError, "resolved"):
            guard.dependency_closure({"packages": [], "resolve": None}, "root")

    def test_unknown_dependency_kind_is_rejected(self) -> None:
        metadata = {
            "packages": [{"id": "root", "name": "paignia-core"}, {"id": "dep", "name": "harmless"}],
            "resolve": {"nodes": [
                {"id": "root", "deps": [{"pkg": "dep", "dep_kinds": [{"kind": "new-unknown-kind"}]}]},
                {"id": "dep", "deps": []},
            ]},
        }
        with self.assertRaisesRegex(ValueError, "kind"):
            guard.dependency_closure(metadata, "root")


if __name__ == "__main__":
    unittest.main()
