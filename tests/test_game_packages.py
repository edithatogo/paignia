"""Controls for actual artifact admission and unavailable provider source."""

from __future__ import annotations

import importlib.util
import io
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("game_packages", ROOT / "scripts/validate_game_packages.py")
assert SPEC is not None and SPEC.loader is not None
packages = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(packages)


class ArtifactControls(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(prefix="paignia-artifact-tests-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)

    def archive(self, entries: list[tuple[str, bytes, bytes]]) -> Path:
        path = self.root / "test.crate"
        with tarfile.open(path, "w:gz") as archive:
            for name, content, kind in entries:
                info = tarfile.TarInfo(name)
                info.type = kind
                info.size = len(content)
                if kind == tarfile.SYMTYPE:
                    info.linkname = "/etc/passwd"
                archive.addfile(info, io.BytesIO(content))
        return path

    def test_path_escape_is_rejected(self) -> None:
        path = self.archive([("example-1.0.0/../escaped", b"bad", tarfile.REGTYPE)])
        with self.assertRaisesRegex(ValueError, "path"):
            packages.unpack_artifact(path, self.root / "unpacked", "example", "1.0.0")
        self.assertFalse((self.root / "escaped").exists())

    def test_symbolic_link_is_rejected(self) -> None:
        path = self.archive([("example-1.0.0/escape", b"", tarfile.SYMTYPE)])
        with self.assertRaisesRegex(ValueError, "regular"):
            packages.unpack_artifact(path, self.root / "unpacked", "example", "1.0.0")

    def test_duplicate_member_is_rejected(self) -> None:
        path = self.archive([("example-1.0.0/LICENSE", b"one", tarfile.REGTYPE), ("example-1.0.0/LICENSE", b"two", tarfile.REGTYPE)])
        with self.assertRaisesRegex(ValueError, "duplicate"):
            packages.unpack_artifact(path, self.root / "unpacked", "example", "1.0.0")

    def test_missing_notice_rejects_artifact(self) -> None:
        path = self.archive([("example-1.0.0/Cargo.toml", b'[package]\nname="example"\nversion="1.0.0"', tarfile.REGTYPE), ("example-1.0.0/src/lib.rs", b"", tarfile.REGTYPE), ("example-1.0.0/LICENSE", b"license", tarfile.REGTYPE)])
        with self.assertRaisesRegex(ValueError, "NOTICE"):
            packages.unpack_artifact(path, self.root / "unpacked", "example", "1.0.0")

    def test_unexpected_package_identity_rejects_artifact(self) -> None:
        path = self.archive([("different-1.0.0/Cargo.toml", b"", tarfile.REGTYPE)])
        with self.assertRaisesRegex(ValueError, "path"):
            packages.unpack_artifact(path, self.root / "unpacked", "example", "1.0.0")

    def test_only_exact_nonzero_native_execution_is_accepted(self) -> None:
        good = "test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n"
        packages.require_native_tests(good, 3)
        for bad in [good.replace("3 passed", "0 passed"), good.replace("3 passed", "2 passed"), good.replace("0 ignored", "1 ignored"), good.replace("0 filtered", "1 filtered"), good.replace("ok.", "FAILED."), good + good]:
            with self.subTest(output=bad), self.assertRaises(RuntimeError):
                packages.require_native_tests(bad, 3)

    def test_materialization_keeps_rust_version_distinct_from_version(self) -> None:
        manifest = self.root / "Cargo.toml"
        manifest.write_text('[package]\nname="example"\nversion.workspace = true\nrust-version.workspace = true\n')
        text = packages.standalone_manifest(manifest, {"package": {"version": "0.1.0", "rust-version": "1.75"}})
        self.assertIn('version = "0.1.0"', text)
        self.assertIn('rust-version = "1.75"', text)

    def test_provider_file_is_unavailable_to_native_compiler(self) -> None:
        rustc = shutil.which("rustc")
        if rustc is None or shutil.which("bwrap") is None:
            self.fail("rustc and bubblewrap are required for the source-isolation control")
        provider = self.root / "provider"
        consumer = self.root / "consumer"
        provider.mkdir()
        consumer.mkdir()
        (provider / "value.txt").write_text("must be unavailable")
        source = consumer / "main.rs"
        source.write_text(f'fn main() {{ println!("{{}}", include_str!({str(provider / "value.txt")!r})); }}'.replace("'", '"'))
        command = packages.isolated_command([rustc, str(source), "-o", str(consumer / "probe")], provider, consumer)
        denied = subprocess.run(command, cwd=consumer, text=True, capture_output=True)
        self.assertNotEqual(denied.returncode, 0, denied.stdout)
        self.assertIn("couldn't read", denied.stderr)
        source.write_text('fn main() { println!("local source works"); }')
        allowed = subprocess.run(command, cwd=consumer, text=True, capture_output=True)
        self.assertEqual(allowed.returncode, 0, allowed.stderr)


if __name__ == "__main__":
    unittest.main()
