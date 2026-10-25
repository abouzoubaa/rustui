import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).resolve().parents[1] / "validate_themes.py"


class ValidateThemesTests(unittest.TestCase):
    def test_component_theme_schema_requires_rust_deserialization_fields(self):
        schema_path = SCRIPT.parents[1] / "themes" / "theme.schema.json"
        component = json.loads(schema_path.read_text(encoding="utf-8"))["$defs"]["componentTheme"]

        self.assertEqual(set(component["required"]), {"rustui_type", "parts"})
        self.assertEqual(component["properties"]["rustui_type"]["type"], "string")
        self.assertEqual(component["properties"]["parts"]["type"], "object")
        self.assertNotIn("styles", component["required"])

    def test_boolean_runtime_version_is_not_skipped_as_a_runtime_pack(self):
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            (root / "valid-plan.json").write_text(
                json.dumps({
                    "record_type": "rustui.theme-plan",
                    "plan_version": 1,
                    "id": "valid-plan",
                    "status": "planned",
                    "runtime_schema": None,
                }),
                encoding="utf-8",
            )
            (root / "malformed-runtime.json").write_text(
                json.dumps({"format": "rustui-appearance", "format_version": True}),
                encoding="utf-8",
            )

            result = subprocess.run(
                [sys.executable, str(SCRIPT), "--plans-only", "--root", str(root)],
                capture_output=True,
                text=True,
                check=False,
            )

        self.assertEqual(result.returncode, 1)
        self.assertIn("malformed-runtime.json", result.stderr)
        self.assertNotIn("skipped 1 runtime packs", result.stdout)


if __name__ == "__main__":
    unittest.main()
