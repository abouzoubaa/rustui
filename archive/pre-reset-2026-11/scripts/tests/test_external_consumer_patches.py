import tomllib
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
CONSUMER = ROOT / "examples/external-consumer"


def patches(root):
    with (root / "Cargo.toml").open("rb") as manifest:
        entries = tomllib.load(manifest)["patch"]["crates-io"]
    return {name: (root / entry["path"]).resolve() for name, entry in entries.items()}


class ExternalConsumerPatchesTest(unittest.TestCase):
    def test_consumer_uses_workspace_patch_sources(self):
        self.assertEqual(patches(CONSUMER), patches(ROOT))


if __name__ == "__main__":
    unittest.main()
