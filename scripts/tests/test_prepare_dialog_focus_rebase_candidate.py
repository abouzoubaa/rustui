from __future__ import annotations

import hashlib
import importlib.util
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch


SCRIPT = (Path(__file__).resolve().parents[1]
          / "prepare_dialog_focus_rebase_candidate.py")
SPEC = importlib.util.spec_from_file_location("dialog_candidate_preparation", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
PREPARATION = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PREPARATION)


class DialogCandidatePreparationTests(unittest.TestCase):
    def test_rejects_approved_patch_replaced_during_copy(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "approved.patch"
            destination = Path(directory) / "candidate-approved.patch"
            approved = b"approved patch bytes\n"
            source.write_bytes(approved)
            expected_hash = hashlib.sha256(approved).hexdigest()
            copy2 = PREPARATION.shutil.copy2

            def replace_after_copy(source_path: Path, destination_path: Path) -> None:
                copy2(source_path, destination_path)
                destination_path.write_bytes(b"different patch bytes\n")

            with patch.object(PREPARATION.shutil, "copy2", side_effect=replace_after_copy):
                with self.assertRaisesRegex(ValueError, "changed during copy"):
                    PREPARATION.copy_approved_patch(source, destination, expected_hash)


if __name__ == "__main__":
    unittest.main()
