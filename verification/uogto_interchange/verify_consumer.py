"""Install the standalone static-wire consumer from an archived provider source."""

import argparse
import io
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
from tempfile import TemporaryDirectory

sys.path.insert(0, str(Path(__file__).resolve().parent))
from interchange import digest
from verify import HERE, ROOT, encode, require, run_solver, source_state


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    revision = source_state()
    archive = subprocess.check_output([
        "git", "archive", revision, "Cargo.toml", "Cargo.lock",
        "crates/paignia-core", "crates/paignia-solve", "crates/paignia-io",
    ], cwd=ROOT)
    with TemporaryDirectory(prefix="paignia-wire-consumer-") as disposable:
        staging = Path(disposable)
        provider = staging / "provider"
        provider.mkdir()
        with tarfile.open(fileobj=io.BytesIO(archive)) as source:
            source.extractall(provider, filter="data")
        consumer = staging / "consumer"
        shutil.copytree(HERE / "consumer", consumer)
        subprocess.run([
            "cargo", "install", "--locked", "--offline", "--path", str(consumer),
            "--root", str(staging / "installed"), "--target-dir", str(staging / "target"),
        ], cwd=staging, check=True, timeout=900)
        executable = staging / "installed/bin/paignia-wire-consumer"
        frozen = HERE / "frozen-static-v1"
        request = (frozen / "request.json").read_bytes()
        expected = (frozen / "result.json").read_bytes()
        run = run_solver(executable, request)
        require(run.returncode == 0, run.stderr.decode())
        require(run.stdout + b"\n" == expected, "Installed consumer changed the frozen static response")
        artifact = executable.read_bytes()
        receipt = {
            "provider_revision": revision,
            "archived_provider_sha256": digest(archive),
            "consumer_lock_sha256": digest((consumer / "Cargo.lock").read_bytes()),
            "installed_consumer_sha256": digest(artifact),
            "wire_request_sha256": digest(request),
            "wire_response_sha256": digest(expected),
            "cargo": subprocess.check_output(["cargo", "--version"], cwd=staging, text=True).strip(),
            "status": "passed",
        }
    require(source_state() == revision, "Provider changed during consumer replay")
    args.output.write_bytes(encode(receipt))
    print("Installed static v1 consumer replay passed")


if __name__ == "__main__":
    main()
