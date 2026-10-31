import importlib.util
import io
import signal
import sys
import threading
import time
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
    def test_boundary_inspection_cannot_resolve_or_rewrite_the_approved_lock(self):
        with patch.object(MODULE, "_capture_cargo_metadata", return_value=(0, b"{}", b"")) as capture:
            for no_deps in (False, True):
                self.assertEqual(MODULE.cargo_metadata(no_deps), {})
                command = capture.call_args.args[0]
                self.assertIn("--locked", command)
                self.assertIn("--offline", command)

    def test_playground_cannot_bypass_rustui_with_renamed_upstream_assets(self):
        root = MODULE.ROOT
        app = package("rustui-playground", root / "apps/rustui-playground", [
            {"name": "gpui-kit-assets", "rename": "icons", "optional": True},
        ])
        framework = package("rustui", root / "crates/rustui", [
            {"name": "gpui-kit-assets"},
        ])
        metadata = {
            "workspace_root": str(root),
            "workspace_members": [app["id"], framework["id"]],
            "packages": [app, framework],
            "resolve": {"nodes": [
                {"id": app["id"], "deps": []},
                {"id": framework["id"], "deps": []},
            ]},
        }
        with patch.object(MODULE, "cargo_metadata", return_value=metadata):
            errors = MODULE.check()
        self.assertEqual(len(errors), 1)
        self.assertIn("rustui-playground", errors[0])
        self.assertIn("gpui-kit-assets", errors[0])

    class FakeProcess:
        def __init__(self, stdout=b"", stderr=b""):
            self.stdout = io.BytesIO(stdout)
            self.stderr = io.BytesIO(stderr)
            self.killed = False

        def wait(self):
            return 0

        def kill(self):
            self.killed = True

    def test_real_child_overflow_is_killed_and_reaped_without_pipe_deadlock(self):
        cap = MODULE.MAX_CARGO_METADATA_OUTPUT_BYTES
        bytes_per_pipe = cap // 2 + 64 * 1024
        child = f"""
import os
import threading
import time

chunk = b"x" * 65536
def write_pipe(fd):
    remaining = {bytes_per_pipe}
    while remaining:
        sent = os.write(fd, chunk[:min(len(chunk), remaining)])
        remaining -= sent

writers = [threading.Thread(target=write_pipe, args=(fd,)) for fd in (1, 2)]
for writer in writers:
    writer.start()
for writer in writers:
    writer.join()
time.sleep(30)
"""
        real_popen = MODULE.subprocess.Popen
        processes = []

        def capture_process(*args, **kwargs):
            process = real_popen(*args, **kwargs)
            processes.append(process)
            return process

        finished = threading.Event()
        outcome = {}

        def capture():
            try:
                outcome["result"] = MODULE._capture_cargo_metadata([sys.executable, "-c", child])
            except Exception as exc:
                outcome["error"] = exc
            finally:
                finished.set()

        worker = threading.Thread(target=capture)
        started = time.monotonic()
        completed = False
        try:
            with patch.object(MODULE.subprocess, "Popen", side_effect=capture_process):
                worker.start()
                completed = finished.wait(timeout=20)
        finally:
            if processes:
                process = processes[0]
                if process.poll() is None:
                    process.kill()
                    process.wait(timeout=5)
            worker.join(timeout=5)

        self.assertTrue(completed, "capture deadlocked while draining the child's full stdout/stderr pipes")
        self.assertFalse(worker.is_alive(), "capture worker did not finish after the child was reaped")
        self.assertEqual(len(processes), 1)
        process = processes[0]
        self.assertIsNotNone(process.returncode, "capture did not wait for and reap the child")
        self.assertLess(time.monotonic() - started, 20, "overflow termination took too long")
        self.assertIsInstance(outcome.get("error"), RuntimeError)
        self.assertEqual(
            str(outcome["error"]),
            f"cargo metadata stdout and stderr exceeded the {cap}-byte limit",
        )
        if sys.platform == "win32":
            self.assertNotEqual(process.returncode, 0, "overflow child exited naturally instead of being killed")
        else:
            self.assertEqual(process.returncode, -signal.SIGKILL, "overflow child was not killed by the capture helper")

    def test_cargo_metadata_accepts_output_within_combined_limit(self):
        output = b'{"ok":true}'
        process = self.FakeProcess(stdout=output)
        with (
            patch.object(MODULE, "MAX_CARGO_METADATA_OUTPUT_BYTES", len(output)),
            patch.object(MODULE.subprocess, "Popen", return_value=process),
        ):
            self.assertEqual(MODULE.cargo_metadata(no_deps=True), {"ok": True})
        self.assertFalse(process.killed)

    def test_cargo_metadata_kills_process_when_combined_output_exceeds_limit(self):
        process = self.FakeProcess(stdout=b"{}", stderr=b"x" * 9)
        with (
            patch.object(MODULE, "MAX_CARGO_METADATA_OUTPUT_BYTES", 10),
            patch.object(MODULE.subprocess, "Popen", return_value=process),
        ):
            with self.assertRaisesRegex(RuntimeError, "exceeded the 10-byte limit"):
                MODULE.cargo_metadata(no_deps=True)
        self.assertTrue(process.killed)

    def run_check(self, root, declarations, resolved_packages):
        workspace_id = "rustui 1"
        member = {"id": workspace_id, "name": "rustui", "dependencies": declarations}
        declared = {"workspace_root": str(root), "workspace_members": [workspace_id], "packages": [member]}
        nodes = []
        for item in resolved_packages:
            edges = []
            for dep in item["dependencies"]:
                target_id = dep.pop("pkg", None)
                if target_id:
                    edges.append({"pkg": target_id, "dep_kinds": []})
            nodes.append({"id": item["id"], "deps": edges})
        resolved = {
            "workspace_members": [workspace_id],
            "packages": resolved_packages,
            "resolve": {"nodes": nodes},
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

    def test_follows_cargo_resolve_edges_to_transitive_declarations(self):
        root = Path("/tmp/rustui")
        errors = self.run_check(
            root,
            [{"name": "intermediate", "path": None, "kind": None}],
            [
                package("rustui", root / "crates/rustui", [{"pkg": "intermediate 1"}]),
                package("intermediate", root / "vendor/intermediate", [
                    {"name": "Allui", "path": None, "optional": True},
                ]),
                package("Allui", root / "vendor/allui"),
            ],
        )
        self.assertTrue(any("declared dependency on Allui" in error for error in errors))

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
