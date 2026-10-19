import importlib.util
import unittest
from pathlib import Path
from unittest.mock import patch

SCRIPT = Path(__file__).resolve().parents[1] / "verify_dependency_boundary.py"
SPEC = importlib.util.spec_from_file_location("verify_dependency_boundary", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def package(name, path, dependencies=()):
    return {
        "id": f"{name} 1",
        "name": name,
        "manifest_path": str(Path(path) / "Cargo.toml"),
        "source": None,
        "dependencies": list(dependencies),
    }


class DependencyBoundaryTests(unittest.TestCase):
    def run_check(self, root, declarations, resolved_packages):
        workspace_id = "rustui 1"
        member = {"id": workspace_id, "name": "rustui", "dependencies": declarations}
        declared = {"workspace_root": str(root), "workspace_members": [workspace_id], "packages": [member]}
        resolved = {
            "workspace_members": [workspace_id],
            "packages": resolved_packages,
        }
        with patch.object(MODULE, "cargo_metadata", side_effect=[declared, resolved]):
            return MODULE.check()

    def test_allows_vendored_path_dependency(self):
        root = Path("/tmp/rustui")
        errors = self.run_check(
            root,
            [{"name": "vendor-lib", "path": str(root / "vendor/vendor-lib"), "kind": None}],
            [
                package("rustui", root / "crates/rustui", [{"pkg": "vendor-lib 1"}]),
                package("vendor-lib", root / "vendor/vendor-lib"),
            ],
        )
        self.assertEqual(errors, [])

    def test_rejects_optional_allui_declaration_and_transitive_allui(self):
        root = Path("/tmp/rustui")
        errors = self.run_check(
            root,
            [{"name": "Allui", "path": None, "kind": None, "optional": True}],
            [
                package("rustui", root / "crates/rustui", [{"pkg": "intermediate 1"}]),
                package("intermediate", root / "vendor/intermediate", [{"pkg": "Allui 1"}]),
                package("Allui", root / "vendor/allui"),
            ],
        )
        self.assertTrue(any("declared dependency on Allui" in error for error in errors))
        self.assertTrue(any("resolved dependency on Allui" in error for error in errors))

    def test_rejects_unactivated_optional_transitive_allui_and_path(self):
        root = Path("/tmp/rustui")
        errors = self.run_check(
            root,
            [{"name": "intermediate", "path": str(root / "vendor/intermediate"), "kind": None}],
            [
                package("rustui", root / "crates/rustui", [{"pkg": "intermediate 1"}]),
                package("intermediate", root / "vendor/intermediate", [
                    {"name": "Allui", "path": None, "optional": True},
                    {"name": "optional-chorus", "path": "/tmp/chorus/optional", "optional": True},
                ]),
            ],
        )
        self.assertTrue(any("declared dependency on Allui" in error for error in errors))
        self.assertTrue(any("declared path dependency outside workspace" in error for error in errors))

    def test_rejects_external_workspace_and_resolved_path_dependencies(self):
        root = Path("/tmp/rustui")
        errors = self.run_check(
            root,
            [{"name": "chorus", "path": "/tmp/chorus/crates/chorus", "kind": None}],
            [
                package("rustui", root / "crates/rustui", [{"pkg": "intermediate 1"}]),
                package("intermediate", root / "vendor/intermediate", [{"pkg": "chorus 1"}]),
                package("chorus", "/tmp/chorus/crates/chorus"),
            ],
        )
        self.assertTrue(any("path dependency outside workspace" in error for error in errors))
        self.assertTrue(any("resolved local dependency outside workspace" in error for error in errors))


if __name__ == "__main__":
    unittest.main()
