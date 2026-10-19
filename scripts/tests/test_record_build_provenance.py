"""Focused tests for caller-declared build provenance fields."""

import json
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
                capture_output=True,
                text=True,
            )
            self.assertEqual(missing.returncode, 0, missing.stderr)
            self.assertEqual(missing.stdout.strip(), str(missing_output))
            missing_record = json.loads(missing_output.read_text(encoding="utf-8"))
            self.assertEqual(missing_record["selected_binary"]["status"], "missing")


if __name__ == "__main__":
    unittest.main()
