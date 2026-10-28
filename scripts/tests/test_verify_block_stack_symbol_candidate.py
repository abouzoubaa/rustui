import importlib.util
from pathlib import Path
import shutil
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location(
    "block_candidate", ROOT / "scripts/verify_block_stack_symbol_candidate.py"
)
candidate = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(candidate)


@unittest.skipUnless(sys.platform == "darwin", "candidate probe requires macOS")
class CandidateIntegrityTest(unittest.TestCase):
    def test_rejects_tampered_source_and_patch_before_running_probe(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            for path in candidate.EXPECTED:
                destination = root / path
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(ROOT / path, destination)
            for path in (candidate.SOURCE, candidate.PATCH):
                file = root / path
                original = file.read_bytes()
                file.write_bytes(original + b"tampered")
                with self.assertRaisesRegex(ValueError, "unexpected input hash"):
                    candidate.verify(root)
                file.write_bytes(original)


if __name__ == "__main__":
    unittest.main()
