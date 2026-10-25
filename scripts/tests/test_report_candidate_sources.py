"""Focused tests for candidate-local Cargo source report validation."""

from __future__ import annotations

import contextlib
import io
import json
import os
import resource
import signal
from pathlib import Path
import shutil
import sys
import subprocess
import tempfile
import unittest
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import report_candidate_sources as reporter  # noqa: E402


class ReportCandidateSourcesTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.candidate = Path(temporary.name) / "candidate-1"
        self.workspace = self.candidate / "candidate/source"
        (self.workspace / "path-sources/upstream/src").mkdir(parents=True)
        (self.workspace / "vendor/registry-crate-1.0.0/src").mkdir(parents=True)
        self.path_manifest = self.workspace / "path-sources/upstream/Cargo.toml"
        self.registry_manifest = self.workspace / "vendor/registry-crate-1.0.0/Cargo.toml"
        self.path_manifest.write_text("[package]\nname='upstream'\nversion='1.0.0'\n", encoding="utf-8")
        self.registry_manifest.write_text("[package]\nname='registry-crate'\nversion='1.0.0'\n", encoding="utf-8")

    def metadata(self, candidate_root: Path | None = None, path_manifest: Path | None = None,
                 registry_manifest: Path | None = None) -> dict:
        candidate_root = candidate_root or self.candidate
        workspace = candidate_root / "candidate/source"
        path_manifest = path_manifest or workspace / "path-sources/upstream/Cargo.toml"
        registry_manifest = registry_manifest or workspace / "vendor/registry-crate-1.0.0/Cargo.toml"
        path_id = f"path+file://{path_manifest.parent}#upstream@1.0.0"
        registry_id = "registry+https://example.invalid/index#registry-crate@1.0.0"
        packages = [
            {"id": path_id, "name": "upstream", "version": "1.0.0", "source": None,
             "manifest_path": str(path_manifest)},
            {"id": registry_id, "name": "registry-crate", "version": "1.0.0",
             "source": "registry+https://example.invalid/index", "manifest_path": str(registry_manifest)},
        ]
        return {
            "workspace_root": str(workspace.resolve()),
            "packages": packages,
            "resolve": {"nodes": [
                {"id": path_id, "deps": [{"name": "registry-crate", "pkg": registry_id,
                                             "dep_kinds": [{"kind": None, "target": None}]}],
                 "features": ["default"]},
                {"id": registry_id, "deps": [], "features": ["default"]},
            ]},
        }

    def validate(self, metadata: dict, candidate_root: Path | None = None) -> dict:
        candidate_root = (candidate_root or self.candidate).resolve()
        return reporter.validate_metadata(metadata, candidate_root,
                                          (candidate_root / "candidate/source").resolve(), {})

    def run_standalone_report(self, cargo: str = "/usr/bin/cargo") -> tuple[int, dict]:
        output = io.StringIO()
        with (
            mock.patch.object(sys, "argv", ["report_candidate_sources.py", str(self.candidate)]),
            mock.patch.object(
                reporter, "_candidate_paths", return_value=(self.candidate, self.workspace)
            ),
            mock.patch.object(reporter.shutil, "which", return_value=cargo),
            contextlib.redirect_stdout(output),
        ):
            exit_code = reporter.main()
        return exit_code, json.loads(output.getvalue())

    def run_standalone_report_output(
        self, report: dict, output_limit: int,
    ) -> tuple[int, str]:
        (self.workspace / "Cargo.toml").write_text(
            "[workspace]\nmembers = []\n", encoding="utf-8"
        )
        (self.workspace / "Cargo.lock").write_text("version = 4\n", encoding="utf-8")
        metadata_result = subprocess.CompletedProcess(
            args=[], returncode=0, stdout=json.dumps(self.metadata()), stderr=""
        )
        output = io.StringIO()
        with (
            mock.patch.object(sys, "argv", ["report_candidate_sources.py", str(self.candidate)]),
            mock.patch.object(
                reporter, "_candidate_paths", return_value=(self.candidate, self.workspace)
            ),
            mock.patch.object(reporter.shutil, "which", return_value="/usr/bin/cargo"),
            mock.patch.object(
                reporter, "_run_capped_metadata", return_value=metadata_result
            ),
            mock.patch.object(reporter, "validate_metadata", return_value=dict(report)),
            mock.patch.object(reporter, "MAX_REPORT_OUTPUT_BYTES", output_limit),
            contextlib.redirect_stdout(output),
        ):
            exit_code = reporter.main()
        return exit_code, output.getvalue()

    def run_pinned_report_output(
        self, report: dict, output_limit: int,
    ) -> tuple[int, str]:
        output = io.StringIO()
        with (
            mock.patch.object(
                sys, "argv", ["report_candidate_sources.py", str(self.candidate)]
            ),
            mock.patch.dict(os.environ, {"RUSTUI_P3_REPO_ROOT_FD": "3"}),
            mock.patch.object(reporter, "_pinned_report", return_value=dict(report)),
            mock.patch.object(reporter, "MAX_REPORT_OUTPUT_BYTES", output_limit),
            contextlib.redirect_stdout(output),
        ):
            exit_code = reporter.main()
        return exit_code, output.getvalue()

    def report(self) -> dict:
        return {
            "schema_version": reporter.SCHEMA_VERSION,
            "status": "valid",
            "scope": reporter.SCOPE,
            "candidate_id": self.candidate.name,
            "workspace_root": "candidate/source",
            "request": {"mode": "locked-offline"},
            "package_count": 0,
            "source_counts": {},
            "resolve_graph_sha256": None,
            "resolved_graph": {"schema_version": 1, "nodes": []},
            "packages": [],
            "errors": [],
        }

    def test_report_emission_accepts_exact_limit(self) -> None:
        report = self.report()
        expected = json.dumps(report, indent=2, sort_keys=True) + "\n"
        output = io.StringIO()
        with mock.patch.object(
            reporter, "MAX_REPORT_OUTPUT_BYTES", len(expected.encode("utf-8"))
        ), contextlib.redirect_stdout(output):
            is_valid = reporter._emit_report(report)

        self.assertTrue(is_valid)
        self.assertEqual(output.getvalue(), expected)

    def test_report_output_within_limit_is_canonical_for_both_entry_paths(self) -> None:
        for pinned, run_report in (
            (False, self.run_standalone_report_output),
            (True, self.run_pinned_report_output),
        ):
            with self.subTest(pinned=pinned):
                exit_code, output = run_report(self.report(), 4096)

                self.assertEqual(exit_code, 0)
                parsed = json.loads(output)
                self.assertEqual(output, json.dumps(parsed, indent=2, sort_keys=True) + "\n")
                self.assertLessEqual(len(output.encode("utf-8")), 4096)

    def test_report_output_overflow_is_bounded_for_both_entry_paths(self) -> None:
        output_limit = 2048
        oversized_report = self.report()
        oversized_report["large_field"] = "x" * output_limit

        for pinned, run_report in (
            (False, self.run_standalone_report_output),
            (True, self.run_pinned_report_output),
        ):
            with self.subTest(pinned=pinned):
                exit_code, output = run_report(oversized_report, output_limit)

                self.assertEqual(exit_code, 1)
                self.assertLessEqual(len(output.encode("utf-8")), output_limit)
                parsed = json.loads(output)
                self.assertEqual(parsed["status"], "invalid")
                self.assertNotIn("large_field", parsed)
                self.assertIn(f"{output_limit}-byte output limit", parsed["errors"][0])

    def make_overflowing_cargo(self, directory: Path) -> Path:
        cargo = directory / "fake-cargo"
        cargo.write_text(
            f"#!{sys.executable}\n"
            "import os, signal, threading, time\n"
            "signal.signal(signal.SIGTERM, signal.SIG_IGN)\n"
            "stdout = threading.Thread(target=lambda: os.write(1, b'x' * 262144))\n"
            "stderr = threading.Thread(target=lambda: os.write(2, b'y' * 262144))\n"
            "stdout.start(); stderr.start()\n"
            "stdout.join(); stderr.join(); time.sleep(30)\n",
            encoding="utf-8",
        )
        cargo.chmod(0o755)
        return cargo

    def test_metadata_output_accepts_exact_limit_and_terminates_overflow(self) -> None:
        with mock.patch.object(reporter, "MAX_CARGO_METADATA_OUTPUT_BYTES", 16):
            exact = reporter._run_capped_metadata([
                sys.executable, "-c",
                "import os; os.write(1, b'x' * 8); os.write(2, b'y' * 8)",
            ])
            self.assertEqual(exact.returncode, 0)
            self.assertEqual(exact.stdout, "x" * 8)
            self.assertEqual(exact.stderr, "y" * 8)
            utf8 = reporter._run_capped_metadata([
                sys.executable, "-c",
                "import os; os.write(1, 'café'.encode()); os.write(2, b'\\xff')",
            ])
            self.assertEqual(utf8.stdout, "café")
            self.assertEqual(utf8.stderr, "\ufffd")

            original_popen = subprocess.Popen
            processes: list[subprocess.Popen] = []

            def capture_process(*args, **kwargs):
                process = original_popen(*args, **kwargs)
                processes.append(process)
                return process

            with mock.patch.object(reporter.subprocess, "Popen", side_effect=capture_process):
                with self.assertRaisesRegex(
                    reporter.SourceReportError, "combined 16-byte limit"
                ):
                    reporter._run_capped_metadata([
                        sys.executable, "-c",
                        "import os, signal, subprocess, sys, threading, time; "
                        "signal.signal(signal.SIGTERM, signal.SIG_IGN); "
                        "subprocess.Popen([sys.executable, '-c', "
                        "'import signal,time; signal.signal(signal.SIGTERM, "
                        "signal.SIG_IGN); time.sleep(30)']); "
                        "a=threading.Thread(target=lambda: os.write(1, b'x' * 262144)); "
                        "b=threading.Thread(target=lambda: os.write(2, b'y' * 262144)); "
                        "a.start(); b.start(); a.join(); b.join(); time.sleep(30)",
                    ])

        self.assertEqual(len(processes), 1)
        self.assertIsNotNone(processes[0].returncode)
        self.assertEqual(processes[0].returncode, -signal.SIGKILL)

    def test_standalone_report_rejects_combined_metadata_output_overflow(self) -> None:
        (self.workspace / "Cargo.toml").write_text(
            "[workspace]\nmembers = []\n", encoding="utf-8"
        )
        (self.workspace / "Cargo.lock").write_text("version = 4\n", encoding="utf-8")
        cargo = self.make_overflowing_cargo(self.candidate)
        original_popen = subprocess.Popen
        processes: list[subprocess.Popen] = []

        def capture_process(*args, **kwargs):
            process = original_popen(*args, **kwargs)
            processes.append(process)
            return process

        with mock.patch.object(reporter, "MAX_CARGO_METADATA_OUTPUT_BYTES", 16), mock.patch.object(
            reporter.subprocess, "Popen", side_effect=capture_process
        ):
            exit_code, report = self.run_standalone_report(str(cargo))

        self.assertEqual(exit_code, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("combined 16-byte limit" in error for error in report["errors"]))
        self.assertEqual(len(processes), 1)
        self.assertEqual(processes[0].returncode, -signal.SIGKILL)

    def test_pinned_report_rejects_combined_metadata_output_overflow(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = Path(directory) / "repo"
            candidates = repo / "target/headstart/candidates"
            candidate = candidates / "candidate-output-overflow"
            workspace = candidate / "candidate/source"
            (workspace / "path-sources/upstream/src").mkdir(parents=True)
            (workspace / "vendor/registry-crate-1.0.0/src").mkdir(parents=True)
            (workspace / "Cargo.toml").write_text(
                "[workspace]\nmembers = []\n", encoding="utf-8"
            )
            (workspace / "Cargo.lock").write_text("version = 4\n", encoding="utf-8")
            (workspace / "path-sources/upstream/Cargo.toml").write_text(
                "[package]\nname='upstream'\nversion='1.0.0'\n", encoding="utf-8"
            )
            (workspace / "vendor/registry-crate-1.0.0/Cargo.toml").write_text(
                "[package]\nname='registry-crate'\nversion='1.0.0'\n", encoding="utf-8"
            )
            cargo = self.make_overflowing_cargo(Path(directory))
            output = io.StringIO()
            original_popen = subprocess.Popen
            processes: list[subprocess.Popen] = []

            def capture_process(*args, **kwargs):
                process = original_popen(*args, **kwargs)
                processes.append(process)
                return process

            try:
                with reporter.PinnedDir.open_root(repo) as repo_fd, reporter.PinnedDir.open_root(
                    candidates
                ) as candidate_parent_fd:
                    environment = {
                        "RUSTUI_P3_REPO_ROOT_FD": str(repo_fd.fd),
                        "RUSTUI_P3_CANDIDATE_PARENT_FD": str(candidate_parent_fd.fd),
                        "RUSTUI_P3_CANDIDATE_ID": candidate.name,
                    }
                    with (
                        mock.patch.dict(os.environ, environment),
                        mock.patch.object(
                            sys,
                            "argv",
                            ["report_candidate_sources.py",
                             f"target/headstart/candidates/{candidate.name}",
                             "--workspace", "candidate/source", "--cargo", str(cargo)],
                        ),
                        mock.patch.object(reporter, "MAX_CARGO_METADATA_OUTPUT_BYTES", 16),
                        mock.patch.object(
                            reporter.subprocess, "Popen", side_effect=capture_process
                        ),
                        contextlib.redirect_stdout(output),
                    ):
                        exit_code = reporter.main()

                report = json.loads(output.getvalue())
                self.assertEqual(exit_code, 1)
                self.assertEqual(report["status"], "invalid")
                self.assertTrue(
                    any("combined 16-byte limit" in error for error in report["errors"])
                )
                self.assertEqual(len(processes), 1)
                self.assertEqual(processes[0].returncode, -signal.SIGKILL)
            finally:
                if candidate.exists():
                    shutil.rmtree(candidate)

    def test_reports_candidate_path_and_vendored_registry_sources(self) -> None:
        report = self.validate(self.metadata())
        self.assertEqual(report["status"], "valid", report["errors"])
        self.assertEqual(report["source_counts"], {"path": 1, "registry-vendor": 1})
        self.assertEqual(report["package_count"], 2)
        self.assertEqual(len(report["resolve_graph_sha256"]), 64)
        self.assertEqual(report["schema_version"], 2)
        self.assertEqual(report["resolved_graph"]["schema_version"], 1)
        self.assertNotIn(str(self.candidate), str(report["resolved_graph"]))

    def test_rejects_non_array_packages_without_traceback(self) -> None:
        for packages in (None, {}, 1, "malformed"):
            with self.subTest(packages=packages):
                metadata = self.metadata()
                metadata["packages"] = packages

                report = self.validate(metadata)

                self.assertEqual(report["status"], "invalid")
                self.assertTrue(any("packages must be a nonempty array" in error
                                    for error in report["errors"]))

    def test_graph_hash_ignores_absolute_candidate_root(self) -> None:
        other = self.candidate.parent / "candidate-2"
        shutil.copytree(self.workspace.parent, other / "candidate")
        first = self.validate(self.metadata())
        second = self.validate(self.metadata(candidate_root=other), other)
        self.assertEqual(first["resolved_graph"], second["resolved_graph"])
        self.assertEqual(first["resolve_graph_sha256"], second["resolve_graph_sha256"])

    def test_rejects_malformed_dependency_edges_and_features(self) -> None:
        metadata = self.metadata()
        metadata["resolve"]["nodes"][0]["deps"][0]["dep_kinds"] = "normal"
        metadata["resolve"]["nodes"][1]["features"] = [1]
        report = self.validate(metadata)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("malformed dependency" in item or "features must" in item
                            for item in report["errors"]))

    def test_rejects_duplicate_enabled_features(self) -> None:
        metadata = self.metadata()
        metadata["resolve"]["nodes"][0]["features"] = ["default", "default"]

        report = self.validate(metadata)

        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("features must be an array" in item for item in report["errors"]))

    def test_rejects_blank_enabled_features(self) -> None:
        for feature in ("", " ", "\t"):
            with self.subTest(feature=feature):
                metadata = self.metadata()
                metadata["resolve"]["nodes"][0]["features"] = [feature]

                report = self.validate(metadata)

                self.assertEqual(report["status"], "invalid")
                self.assertTrue(any("features must be an array" in item
                                    for item in report["errors"]))

    def test_pinned_report_survives_candidate_path_swap_during_cargo(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = Path(directory) / "repo"
            candidates = repo / "target/headstart/candidates"
            candidate = candidates / "candidate-fd-race"
            workspace = candidate / "candidate/source"
            (workspace / "path-sources/upstream/src").mkdir(parents=True)
            (workspace / "vendor/registry-crate-1.0.0/src").mkdir(parents=True)
            (workspace / "Cargo.toml").write_text("[workspace]\nmembers = []\n", encoding="utf-8")
            (workspace / "Cargo.lock").write_text("version = 4\n", encoding="utf-8")
            (workspace / "path-sources/upstream/Cargo.toml").write_text(
                "[package]\nname='upstream'\nversion='1.0.0'\n", encoding="utf-8"
            )
            (workspace / "vendor/registry-crate-1.0.0/Cargo.toml").write_text(
                "[package]\nname='registry-crate'\nversion='1.0.0'\n", encoding="utf-8"
            )
            metadata = self.metadata(candidate_root=candidate)
            for package in metadata["packages"]:
                package["manifest_path"] = str(Path(package["manifest_path"]).resolve())
            metadata_json = json.dumps(metadata)
            fake_cargo = Path(directory) / "fake-cargo"
            held = candidates / "candidate-fd-race-held"
            script = f"""#!{sys.executable}
import json, os
from pathlib import Path
candidate = Path({str(candidate)!r})
held = Path({str(held)!r})
os.rename(candidate, held)
replacement = candidate / 'candidate/source'
replacement.mkdir(parents=True)
(replacement / 'Cargo.toml').write_text('[workspace]\\nmembers = []\\n')
(replacement / 'Cargo.lock').write_text('version = 4\\n')
print({metadata_json!r})
"""
            fake_cargo.write_text(script, encoding="utf-8")
            fake_cargo.chmod(0o755)
            output = io.StringIO()
            candidates.mkdir(parents=True, exist_ok=True)
            try:
                with reporter.PinnedDir.open_root(repo) as repo_fd, reporter.PinnedDir.open_root(candidates) as candidate_parent_fd:
                    environment = {
                        "RUSTUI_P3_REPO_ROOT_FD": str(repo_fd.fd),
                        "RUSTUI_P3_CANDIDATE_PARENT_FD": str(candidate_parent_fd.fd),
                        "RUSTUI_P3_CANDIDATE_ID": candidate.name,
                    }
                    with mock.patch.dict(os.environ, environment), mock.patch.object(
                        sys,
                        "argv",
                        ["report_candidate_sources.py", f"target/headstart/candidates/{candidate.name}",
                         "--workspace", "candidate/source", "--cargo", str(fake_cargo)],
                    ), contextlib.redirect_stdout(output):
                        exit_code = reporter.main()
                report = json.loads(output.getvalue())
                self.assertEqual(exit_code, 0, report["errors"])
                self.assertEqual(report["status"], "valid")
                self.assertEqual(report["candidate_id"], candidate.name)
                self.assertTrue(candidate.exists(), "the pathname should still name Cargo's replacement")
            finally:
                if candidate.exists():
                    shutil.rmtree(candidate)
                if held.exists():
                    os.rename(held, candidate)

    def test_pinned_manifest_inventory_is_depth_safe_under_low_fd_limit(self) -> None:
        depth = 1100
        directory_flags = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW
        manifest_contents = b"[package]\nname = 'deep'\nversion = '1.0.0'\n"
        with tempfile.TemporaryDirectory() as temporary:
            workspace_path = Path(temporary) / "workspace"
            workspace_path.mkdir()

            def remove_deep_chain(root: Path) -> None:
                root_fd = os.open(root, directory_flags)
                current_fd = os.dup(root_fd)
                levels = 0
                try:
                    while True:
                        try:
                            child_fd = os.open("d", directory_flags, dir_fd=current_fd)
                        except FileNotFoundError:
                            break
                        os.close(current_fd)
                        current_fd = child_fd
                        levels += 1
                    try:
                        os.unlink("Cargo.toml", dir_fd=current_fd)
                    except FileNotFoundError:
                        pass
                    for _ in range(levels):
                        parent_fd = os.open("..", directory_flags, dir_fd=current_fd)
                        os.rmdir("d", dir_fd=parent_fd)
                        os.close(current_fd)
                        current_fd = parent_fd
                finally:
                    os.close(current_fd)
                    os.close(root_fd)

            current_fd = os.open(workspace_path, directory_flags)
            try:
                for _ in range(depth):
                    os.mkdir("d", dir_fd=current_fd)
                    child_fd = os.open("d", directory_flags, dir_fd=current_fd)
                    os.close(current_fd)
                    current_fd = child_fd
                manifest_fd = os.open(
                    "Cargo.toml", os.O_WRONLY | os.O_CREAT | os.O_EXCL,
                    0o600, dir_fd=current_fd,
                )
                try:
                    os.write(manifest_fd, manifest_contents)
                finally:
                    os.close(manifest_fd)
            finally:
                os.close(current_fd)

            original_soft, hard_limit = resource.getrlimit(resource.RLIMIT_NOFILE)
            limit_changed = False
            try:
                with reporter.PinnedDir.open_root(workspace_path) as workspace:
                    descriptor_count = len(os.listdir("/dev/fd"))
                    low_limit = max(32, descriptor_count + 8)
                    if low_limit >= depth:
                        self.skipTest("cannot set a descriptor limit far below the tree depth")
                    if hard_limit != resource.RLIM_INFINITY and hard_limit < low_limit:
                        self.skipTest("hard file descriptor limit is too low for the regression")
                    resource.setrlimit(resource.RLIMIT_NOFILE, (low_limit, hard_limit))
                    limit_changed = True

                    records = reporter._pinned_manifest_hashes(workspace)
                    relative = "/".join(["d"] * depth + ["Cargo.toml"])
                    manifest_hash, _ = records[("manifest", relative)]
                    self.assertEqual(
                        manifest_hash, reporter.hashlib.sha256(manifest_contents).hexdigest()
                    )
                    self.assertEqual(
                        sum(kind == "directory" for kind, _ in records), depth + 1
                    )
            finally:
                if limit_changed:
                    resource.setrlimit(resource.RLIMIT_NOFILE, (original_soft, hard_limit))
                remove_deep_chain(workspace_path)

    def test_rejects_dependency_edges_without_kinds(self) -> None:
        metadata = self.metadata()
        metadata["resolve"]["nodes"][0]["deps"][0]["dep_kinds"] = []
        report = self.validate(metadata)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("malformed dependency" in item for item in report["errors"]))

    def test_rejects_blank_dependency_kind_metadata(self) -> None:
        metadata = self.metadata()
        metadata["resolve"]["nodes"][0]["deps"][0]["dep_kinds"] = [
            {"kind": " ", "target": ""}
        ]

        report = self.validate(metadata)

        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("malformed dependency kind fields" in item
                            for item in report["errors"]))

    def test_rejects_unknown_dependency_kind(self) -> None:
        metadata = self.metadata()
        metadata["resolve"]["nodes"][0]["deps"][0]["dep_kinds"] = [
            {"kind": "runtime", "target": None}
        ]

        report = self.validate(metadata)

        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("malformed dependency kind fields" in item
                            for item in report["errors"]))

    def test_rejects_blank_dependency_aliases(self) -> None:
        for alias in ("", " \t "):
            with self.subTest(alias=alias):
                metadata = self.metadata()
                metadata["resolve"]["nodes"][0]["deps"][0]["name"] = alias

                report = self.validate(metadata)

                self.assertEqual(report["status"], "invalid")
                self.assertTrue(any("malformed dependency" in item for item in report["errors"]))

    def test_metadata_command_forces_isolated_target_over_candidate_config(self) -> None:
        isolated_target = self.candidate.parent / "p3h isolated target"
        configs = [self.candidate / ".cargo/config.toml", self.workspace / ".cargo/config.toml"]
        command = reporter._metadata_command("/usr/bin/cargo", self.workspace, configs,
                                             isolated_target, [], False, False, None)
        first_config = command[command.index("--config") + 1]
        second_config_index = command.index("--config", command.index("--config") + 1)
        self.assertEqual(first_config, str(configs[0]))
        self.assertEqual(command[second_config_index + 1], str(configs[1]))
        target_config = command[command.index("--config", second_config_index + 1) + 1]
        self.assertEqual(target_config, f'build.target-dir="{isolated_target}"')

    def test_selects_candidate_legacy_cargo_config_and_prefers_toml(self) -> None:
        candidate_config_dir = self.candidate / ".cargo"
        candidate_config_dir.mkdir()
        candidate_config = candidate_config_dir / "config.toml"
        candidate_config.write_text("[net]\noffline = true\n", encoding="utf-8")
        config_dir = self.workspace / ".cargo"
        config_dir.mkdir()
        legacy = config_dir / "config"
        legacy.write_text("[net]\noffline = true\n", encoding="utf-8")
        self.assertEqual(reporter._candidate_cargo_configs(self.workspace, self.candidate),
                         [candidate_config, legacy])

        toml = config_dir / "config.toml"
        toml.write_text("[net]\noffline = true\n", encoding="utf-8")
        self.assertEqual(reporter._candidate_cargo_configs(self.workspace, self.candidate),
                         [candidate_config, toml])

    def test_rejects_cargo_config_hash_drift_during_metadata(self) -> None:
        lockfile = self.workspace / "Cargo.lock"
        lockfile.write_text("version = 4\n", encoding="utf-8")
        config = self.candidate / ".cargo/config.toml"
        config.parent.mkdir()
        config.write_text("[net]\noffline = true\n", encoding="utf-8")
        metadata_result = subprocess.CompletedProcess(
            args=[], returncode=0, stdout=json.dumps(self.metadata()), stderr=""
        )

        def change_config_during_cargo(*_args, **_kwargs):
            config.write_text("[net]\noffline = false\n", encoding="utf-8")
            return metadata_result

        output = io.StringIO()
        with (
            mock.patch.object(sys, "argv", ["report_candidate_sources.py", str(self.candidate)]),
            mock.patch.object(reporter, "_candidate_paths", return_value=(self.candidate, self.workspace)),
            mock.patch.object(reporter.shutil, "which", return_value="/usr/bin/cargo"),
            mock.patch.object(reporter, "_run_capped_metadata", side_effect=change_config_during_cargo),
            contextlib.redirect_stdout(output),
        ):
            exit_code = reporter.main()

        report = json.loads(output.getvalue())
        self.assertEqual(exit_code, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("Cargo config changed during" in error for error in report["errors"]))

    def test_rejects_package_manifest_hash_drift_during_metadata(self) -> None:
        (self.workspace / "Cargo.lock").write_text("version = 4\n", encoding="utf-8")
        metadata_result = subprocess.CompletedProcess(
            args=[], returncode=0, stdout=json.dumps(self.metadata()), stderr=""
        )

        def change_manifest_during_cargo(*_args, **_kwargs):
            self.path_manifest.write_text(
                "[package]\nname='changed-after-metadata'\nversion='9.9.9'\n",
                encoding="utf-8",
            )
            return metadata_result

        output = io.StringIO()
        with (
            mock.patch.object(sys, "argv", ["report_candidate_sources.py", str(self.candidate)]),
            mock.patch.object(reporter, "_candidate_paths", return_value=(self.candidate, self.workspace)),
            mock.patch.object(reporter.shutil, "which", return_value="/usr/bin/cargo"),
            mock.patch.object(reporter, "_run_capped_metadata", side_effect=change_manifest_during_cargo),
            contextlib.redirect_stdout(output),
        ):
            exit_code = reporter.main()

        report = json.loads(output.getvalue())
        self.assertEqual(exit_code, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any(
            "candidate Cargo manifests changed during metadata inspection" in error
            for error in report["errors"]
        ))

    def test_cargo_manifest_read_is_bounded(self) -> None:
        self.path_manifest.write_bytes(b"x" * 17)

        with mock.patch.object(reporter, "MAX_CARGO_MANIFEST_BYTES", 16):
            with self.assertRaisesRegex(reporter.SourceReportError, "exceeds 16 bytes"):
                reporter._cargo_manifest_hashes(self.workspace, self.candidate)

    def test_pinned_cargo_manifest_read_is_bounded(self) -> None:
        self.path_manifest.write_bytes(b"x" * 17)

        with reporter.PinnedDir.open_root(self.workspace) as workspace:
            with mock.patch.object(reporter, "MAX_CARGO_MANIFEST_BYTES", 16):
                with self.assertRaisesRegex(reporter.SourceReportError, "exceeds 16 bytes"):
                    reporter._pinned_manifest_hashes(workspace)

    def test_cargo_manifest_inventory_entry_count_is_bounded(self) -> None:
        with mock.patch.object(reporter, "MAX_CARGO_MANIFEST_ENTRIES", 1):
            with self.assertRaisesRegex(reporter.SourceReportError, "exceeds 1 entries"):
                reporter._cargo_manifest_hashes(self.workspace, self.candidate)

    def test_pinned_cargo_manifest_inventory_entry_count_is_bounded(self) -> None:
        with reporter.PinnedDir.open_root(self.workspace) as workspace:
            with mock.patch.object(reporter, "MAX_CARGO_MANIFEST_ENTRIES", 1):
                with self.assertRaisesRegex(reporter.SourceReportError, "exceeds 1 entries"):
                    reporter._pinned_manifest_hashes(workspace)

    def test_pinned_workspace_selection_bounds_cargo_manifest_reads(self) -> None:
        workspace_manifest = self.workspace / "Cargo.toml"
        workspace_manifest.write_bytes(b"x" * 17)
        original_read = reporter._pinned_file_bytes
        reads: list[tuple[str, int | None]] = []

        def track_read(root, relative: str, *, limit: int | None = None) -> bytes:
            reads.append((relative, limit))
            return original_read(root, relative, limit=limit)

        with reporter.PinnedDir.open_root(self.candidate) as candidate:
            with (
                mock.patch.object(reporter, "MAX_CARGO_MANIFEST_BYTES", 16),
                mock.patch.object(reporter, "_pinned_file_bytes", side_effect=track_read),
            ):
                for workspace_arg in (None, Path("candidate/source")):
                    with self.assertRaisesRegex(
                        reporter.SourceReportError, "exceeds size limit"
                    ):
                        reporter._pinned_workspace(candidate, workspace_arg)

        self.assertEqual(reads, [("Cargo.toml", 16), ("Cargo.toml", 16)])

    def test_standalone_cargo_lock_read_is_bounded(self) -> None:
        (self.workspace / "Cargo.lock").write_bytes(b"x" * 17)

        with mock.patch.object(reporter, "MAX_CARGO_SUPPORT_FILE_BYTES", 16):
            exit_code, report = self.run_standalone_report()

        self.assertEqual(exit_code, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("Cargo.lock" in error and "exceeds 16 bytes" in error
                            for error in report["errors"]))

    def test_standalone_cargo_config_read_is_bounded(self) -> None:
        config = self.candidate / ".cargo/config.toml"
        config.parent.mkdir()
        config.write_bytes(b"x" * 17)
        (self.workspace / "Cargo.lock").write_text("version = 4\n", encoding="utf-8")

        with mock.patch.object(reporter, "MAX_CARGO_SUPPORT_FILE_BYTES", 16):
            exit_code, report = self.run_standalone_report()

        self.assertEqual(exit_code, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any(".cargo/config.toml" in error and "exceeds 16 bytes" in error
                            for error in report["errors"]))

    def test_pinned_workspace_selection_bounds_cargo_lock_reads(self) -> None:
        (self.workspace / "Cargo.toml").write_text("[workspace]\n", encoding="utf-8")
        (self.workspace / "Cargo.lock").write_bytes(b"x" * 17)

        with reporter.PinnedDir.open_root(self.candidate) as candidate:
            with mock.patch.object(reporter, "MAX_CARGO_SUPPORT_FILE_BYTES", 16):
                with self.assertRaisesRegex(reporter.SourceReportError, "exceeds size limit.*Cargo.lock"):
                    reporter._pinned_workspace(candidate, None)

    def test_pinned_cargo_config_read_is_bounded(self) -> None:
        config = self.candidate / ".cargo/config.toml"
        config.parent.mkdir()
        config.write_bytes(b"x" * 17)

        with reporter.PinnedDir.open_root(self.candidate) as candidate:
            with mock.patch.object(reporter, "MAX_CARGO_SUPPORT_FILE_BYTES", 16):
                with self.assertRaisesRegex(reporter.SourceReportError, "exceeds 16 bytes.*config.toml"):
                    reporter._pinned_cargo_configs(candidate, "candidate/source")

    def test_rejects_package_manifest_mutation_restored_during_metadata(self) -> None:
        (self.workspace / "Cargo.lock").write_text("version = 4\n", encoding="utf-8")
        original = self.path_manifest.read_bytes()
        metadata = self.metadata()
        metadata["packages"][0]["name"] = "transient-upstream"
        metadata_result = subprocess.CompletedProcess(
            args=[], returncode=0, stdout=json.dumps(metadata), stderr=""
        )

        def mutate_and_restore_manifest(*_args, **_kwargs):
            self.path_manifest.write_text(
                "[package]\nname='transient-upstream'\nversion='1.0.0'\n",
                encoding="utf-8",
            )
            self.path_manifest.write_bytes(original)
            return metadata_result

        output = io.StringIO()
        with (
            mock.patch.object(sys, "argv", ["report_candidate_sources.py", str(self.candidate)]),
            mock.patch.object(reporter, "_candidate_paths", return_value=(self.candidate, self.workspace)),
            mock.patch.object(reporter.shutil, "which", return_value="/usr/bin/cargo"),
            mock.patch.object(reporter, "_run_capped_metadata", side_effect=mutate_and_restore_manifest),
            contextlib.redirect_stdout(output),
        ):
            exit_code = reporter.main()

        report = json.loads(output.getvalue())
        self.assertEqual(exit_code, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any(
            "candidate Cargo manifests changed during metadata inspection" in error
            for error in report["errors"]
        ))

    def test_rejects_registry_manifest_outside_candidate_vendor(self) -> None:
        outside = self.candidate.parent / "external/Cargo.toml"
        outside.parent.mkdir()
        outside.write_text("[package]\n", encoding="utf-8")
        report = self.validate(self.metadata(registry_manifest=outside))
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("escapes candidate root" in item or "outside candidate-local vendor" in item
                            for item in report["errors"]))

    def test_rejects_path_package_outside_workspace(self) -> None:
        outside = self.candidate / "untracked/Cargo.toml"
        outside.parent.mkdir()
        outside.write_text("[package]\n", encoding="utf-8")
        report = self.validate(self.metadata(path_manifest=outside))
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("outside the candidate workspace" in item for item in report["errors"]))

    def test_rejects_git_source_even_when_manifest_is_local(self) -> None:
        metadata = self.metadata()
        metadata["packages"][0]["source"] = "git+https://example.invalid/repo#revision"
        report = self.validate(metadata)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("git package" in item for item in report["errors"]))

    def test_rejects_non_string_cargo_source_without_crashing(self) -> None:
        for source in ({"unexpected": "object"}, ["unexpected", "array"], 42):
            with self.subTest(source=source):
                metadata = self.metadata()
                metadata["packages"][0]["source"] = source
                report = self.validate(metadata)
                self.assertEqual(report["status"], "invalid")
                self.assertTrue(any("unsupported Cargo source value" in item
                                    for item in report["errors"]))

    def test_rejects_blank_package_identity_fields(self) -> None:
        for field in ("name", "version"):
            for value in ("", "   "):
                with self.subTest(field=field, value=value):
                    metadata = self.metadata()
                    metadata["packages"][0][field] = value

                    report = self.validate(metadata)

                    self.assertEqual(report["status"], "invalid")
                    self.assertTrue(any("nonempty string name and version" in error
                                        for error in report["errors"]))

    def test_rejects_incomplete_resolved_graph(self) -> None:
        metadata = self.metadata()
        metadata["resolve"]["nodes"].pop()
        report = self.validate(metadata)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("do not match" in item for item in report["errors"]))

    def test_rejects_manifest_symlink(self) -> None:
        real = self.workspace / "path-sources/upstream/real.toml"
        real.write_text("[package]\n", encoding="utf-8")
        linked = self.workspace / "path-sources/upstream/linked.toml"
        linked.symlink_to(real)
        metadata = self.metadata(path_manifest=linked)
        report = self.validate(metadata)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("symlink rejected" in item for item in report["errors"]))

    def test_rejects_manifest_hard_link_to_file_outside_candidate(self) -> None:
        external = self.candidate.parent / "external-Cargo.toml"
        external.write_text(self.path_manifest.read_text(encoding="utf-8"), encoding="utf-8")
        self.path_manifest.unlink()
        self.path_manifest.hardlink_to(external)

        report = self.validate(self.metadata())

        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("hard-linked source file rejected" in item
                            for item in report["errors"]))


if __name__ == "__main__":
    unittest.main()
