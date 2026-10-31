"""Focused tests for quad-sort candidate manifest preparation."""

import ast
import importlib.util
from pathlib import Path
import tomllib
import unittest


SCRIPT = Path(__file__).resolve().parents[1] / "prepare_quad_sort_candidate.py"
SPEC = importlib.util.spec_from_file_location("prepare_quad_sort_candidate", SCRIPT)
PREPARER = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(PREPARER)


class QuadSortCandidateManifestTests(unittest.TestCase):
    def test_candidate_preparation_guards_are_not_optimization_sensitive(self):
        tree = ast.parse(SCRIPT.read_text(encoding="utf-8"))
        main = next(
            node for node in tree.body
            if isinstance(node, ast.FunctionDef) and node.name == "main"
        )

        self.assertFalse(
            any(isinstance(node, ast.Assert) for node in ast.walk(main)),
            "preparation checks must remain active under python -O",
        )

    def test_redirects_existing_patch_entry_without_duplicating_table(self):
        manifest = '''[workspace]\nmembers = []\n\n[patch.crates-io]\ngpui-pre = { path = "vendor/gpui-pre-0.3.9" }\ngpui-base = { path = "vendor/gpui-base-0.7.3" }\n\n[profile.dev]\ndebug = 1\n'''

        rewritten = PREPARER.redirect_gpui_patch(manifest)
        parsed = tomllib.loads(rewritten)

        self.assertEqual(
            parsed["patch"]["crates-io"]["gpui-pre"],
            {"path": "path-sources/gpui-pre-0.3.9"},
        )
        self.assertEqual(
            parsed["patch"]["crates-io"]["gpui-base"],
            {"path": "vendor/gpui-base-0.7.3"},
        )
        self.assertEqual(rewritten.count("[patch.crates-io]"), 1)


if __name__ == "__main__":
    unittest.main()
