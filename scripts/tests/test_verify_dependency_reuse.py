"""P2B Cargo event-order checks with synthetic JSON logs."""

import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout
from io import StringIO
from unittest.mock import patch


SCRIPT = Path(__file__).resolve().parents[1] / "verify_dependency_reuse.py"
sys.path.insert(0, str(SCRIPT.parent))
import verify_dependency_reuse as verifier
ARTIFACT = {
    "reason": "compiler-artifact",
    "package_id": "registry+https://github.com/rust-lang/crates.io-index#gpui-pre@0.3.9",
    "target": {"name": "gpui", "kind": ["lib"]},
    "profile": {},
    "filenames": ["libgpui.rlib"],
    "fresh": True,
}
FINISHED = {"reason": "build-finished", "success": True}


class DependencyReuseTests(unittest.TestCase):
    def run_log(self, messages):
        with tempfile.TemporaryDirectory() as directory:
            log = Path(directory) / "build.jsonl"
            report = Path(directory) / "reuse.json"
            log.write_text("\n".join(json.dumps(message) for message in messages) + "\n")
            result = subprocess.run(
                [sys.executable, str(SCRIPT), str(log), "--package", "gpui-pre", "--report", str(report)],
                capture_output=True,
                text=True,
                check=False,
            )
            return result, json.loads(report.read_text()) if report.exists() else None

    def test_fresh_artifact_before_finish_confirms_reuse(self):
        result, report = self.run_log([ARTIFACT, FINISHED])
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report["overall_reuse"], "confirmed")

    def test_fresh_artifact_after_finish_cannot_confirm_reuse(self):
        result, report = self.run_log([FINISHED, ARTIFACT])
        self.assertEqual(result.returncode, 2)
        self.assertIn("follows build-finished", result.stderr)
        self.assertIsNone(report)

    def test_exact_log_limit_preserves_hash_and_oversized_log_writes_no_report(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            log = root / "build.jsonl"
            report = root / "reuse.json"
            raw = (json.dumps(FINISHED) + "\n").encode()
            log.write_bytes(raw)
            args = [str(SCRIPT), str(log), "--package", "unlisted", "--report", str(report)]

            with (
                patch.object(verifier, "MAX_LOG_BYTES", len(raw)),
                patch.object(sys, "argv", args),
                redirect_stdout(StringIO()),
                redirect_stderr(StringIO()),
            ):
                self.assertEqual(verifier.main(), 1)
            self.assertEqual(json.loads(report.read_text())["log_sha256"],
                             hashlib.sha256(raw).hexdigest())

            report.unlink()
            log.write_bytes(raw + b" ")
            stderr = StringIO()
            with (
                patch.object(verifier, "MAX_LOG_BYTES", len(raw)),
                patch.object(sys, "argv", args),
                redirect_stdout(StringIO()),
                redirect_stderr(stderr),
            ):
                self.assertEqual(verifier.main(), 2)
            self.assertIn("exceeds", stderr.getvalue())
            self.assertFalse(report.exists())


if __name__ == "__main__":
    unittest.main()
