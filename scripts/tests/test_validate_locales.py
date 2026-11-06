import contextlib
import io
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from validate_locales import main, validate_catalog_group


class ValidateLocalesTests(unittest.TestCase):
    def test_locale_record_rejects_boolean_schema_version(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            locales = root / "locales"
            locale_dir = locales / "en"
            locale_dir.mkdir(parents=True)
            lab_dir = root / "lab" / "en"
            lab_dir.mkdir(parents=True)
            (locale_dir / "locale.json").write_text(
                json.dumps(
                    {
                        "record_type": "rustui.locale",
                        "schema_version": True,
                        "locale": "en",
                        "direction": "ltr",
                        "status": "runtime",
                        "catalog_format": "rustui-ftl-subset-v1",
                    }
                ),
                encoding="utf-8",
            )
            (locale_dir / "rustui.ftl").write_text("hello = Hello\n", encoding="utf-8")
            (lab_dir / "lab.ftl").write_text("hello = Hello\n", encoding="utf-8")

            with (
                patch(
                    "sys.argv",
                    ["validate_locales", "--root", str(locales), "--lab-root", str(root / "lab")],
                ),
                contextlib.redirect_stderr(io.StringIO()),
            ):
                self.assertEqual(main(), 1)

    def test_catalog_group_requires_every_runtime_locale(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            english = root / "en" / "rustui.ftl"
            english.parent.mkdir()
            english.write_text("hello = Hello\n", encoding="utf-8")

            with self.assertRaisesRegex(ValueError, r"missing=\['fr'\]"):
                validate_catalog_group(root, "rustui.ftl", {"en", "fr"})


if __name__ == "__main__":
    unittest.main()
