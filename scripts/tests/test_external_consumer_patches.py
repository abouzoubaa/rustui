import tomllib
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CONSUMER = ROOT / "examples/external-consumer"


class ExternalConsumerPatchesTest(unittest.TestCase):
    def test_consumer_needs_only_rustui_and_has_no_patch_section(self) -> None:
        with (CONSUMER / "Cargo.toml").open("rb") as manifest:
            consumer_manifest = tomllib.load(manifest)

        self.assertNotIn("patch", consumer_manifest)
        self.assertEqual(set(consumer_manifest["dependencies"]), {"rustui"})
        self.assertEqual(
            (CONSUMER / consumer_manifest["dependencies"]["rustui"]["path"]).resolve(),
            ROOT / "crates/rustui",
        )

        source = (CONSUMER / "src/main.rs").read_text(encoding="utf-8")
        self.assertIn("use rustui::gpui;", source)
        self.assertIn("use rustui::gpui_component;", source)


if __name__ == "__main__":
    unittest.main()
