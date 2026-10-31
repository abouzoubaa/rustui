"""Focused tests for caller-declared build provenance fields."""

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


SCRIPT = Path(__file__).resolve().parents[1] / "record_build_provenance.py"


class RecordBuildProvenanceTests(unittest.TestCase):
    def test_records_caller_declared_features_and_profile(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            log = root / "build.log"
            output = root / "provenance.json"
            log.write_text("build output\n", encoding="utf-8")

            subprocess.run(
                [
                    sys.executable,
                    str(SCRIPT),
                    str(log),
                    "--workspace",
                    str(root),
                    "--output",
                    str(output),
                    "--profile",
                    "dev",
                    "--features",
                    "serde,unstable-render",
                ],
                check=True,
                capture_output=True,
                text=True,
            )

            document = json.loads(output.read_text(encoding="utf-8"))
            facts = document["build_facts_declared_by_caller"]
            self.assertEqual(facts["profile"], "dev")
            self.assertEqual(facts["features"], "serde,unstable-render")

    def test_cli_reports_output_status_and_missing_binary(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            log = root / "build.log"
            output = root / "provenance.json"
            log.write_text("build output\n", encoding="utf-8")

            result = subprocess.run(
                [
                    sys.executable, str(SCRIPT), str(log), "--workspace", str(root),
                    "--output", str(output), "--exit-status", "0",
                ],
                capture_output=True,
                text=True,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(result.stdout.strip(), str(output))
            record = json.loads(output.read_text(encoding="utf-8"))
            self.assertEqual(record["build_facts_declared_by_caller"]["exit_status"], 0)
            self.assertIsNone(record["selected_binary"])

            missing_output = root / "missing-provenance.json"
            missing = subprocess.run(
                [
                    sys.executable, str(SCRIPT), str(log), "--workspace", str(root),
                    "--output", str(missing_output), "--binary", "missing-binary",
                ],
                env={**os.environ, "RUSTUI_HEADSTART_ROOT": str(root / "no-compiler")},
                capture_output=True,
                text=True,
            )
            self.assertEqual(missing.returncode, 0, missing.stderr)
            self.assertEqual(missing.stdout.strip(), str(missing_output))
            missing_record = json.loads(missing_output.read_text(encoding="utf-8"))
            self.assertEqual(missing_record["selected_binary"]["status"], "missing")
            self.assertIsNone(missing_record["compiler_identity"])

    def test_force_rejects_output_hard_link_to_build_log(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            log = root / "build.log"
            output = root / "build-log-alias.json"
            log.write_text("build output\n", encoding="utf-8")
            output.hardlink_to(log)

            result = subprocess.run(
                [
                    sys.executable, str(SCRIPT), str(log), "--workspace", str(root),
                    "--output", str(output), "--force",
                ],
                capture_output=True,
                text=True,
            )

            self.assertNotEqual(result.returncode, 0)
            self.assertIn("output path must not replace the supplied build log", result.stderr)
            self.assertEqual(log.read_text(encoding="utf-8"), "build output\n")

    def test_force_rejects_output_path_that_is_a_captured_build_input(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            log = root / "build.log"
            output = root / "Cargo.toml"
            log.write_text("build output\n", encoding="utf-8")
            original_input = "[workspace]\nmembers = []\n"
            output.write_text(original_input, encoding="utf-8")

            result = subprocess.run(
                [
                    sys.executable, str(SCRIPT), str(log), "--workspace", str(root),
                    "--output", str(output), "--force",
                ],
                capture_output=True,
                text=True,
            )

            self.assertNotEqual(result.returncode, 0)
            self.assertIn("output path must not replace a file captured by build provenance",
                          result.stderr)
            self.assertEqual(output.read_text(encoding="utf-8"), original_input)


if __name__ == "__main__":
    unittest.main()
