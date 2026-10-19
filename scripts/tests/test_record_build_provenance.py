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


if __name__ == "__main__":
    unittest.main()
