"""P2B Cargo event-order checks with synthetic JSON logs."""

import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


SCRIPT = Path(__file__).resolve().parents[1] / "verify_dependency_reuse.py"
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


if __name__ == "__main__":
    unittest.main()
