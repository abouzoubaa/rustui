from __future__ import annotations

from contextlib import ExitStack
import hashlib
import json
import os
import resource
import shutil
import tempfile
import threading
import unittest
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from unittest.mock import patch

import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import run_candidate_pipeline as pipeline


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


class CandidatePipelineTests(unittest.TestCase):
    def test_pinned_candidate_fs_rejects_links_and_supports_atomic_copy_and_remove(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            source_path = root / "source"
            source_path.mkdir()
            (source_path / "nested").mkdir()
            (source_path / "nested/file.txt").write_text("source bytes\n")
            destination_path = root / "destination"
            destination_path.mkdir()
            outside = root / "outside"
            outside.mkdir()
            (outside / "secret.txt").write_text("secret\n")
            (source_path / "escape").symlink_to(outside, target_is_directory=True)

            with pipeline.PinnedDir.open_root(root) as pinned_root:
                source = pipeline.PinnedDir.open_child(pinned_root, "source")
                destination = pipeline.PinnedDir.open_child(pinned_root, "destination")
                try:
                    with self.assertRaisesRegex(pipeline.CandidateFSError, "symlink"):
                        destination.copy_tree_from(source)
                    destination.remove_tree("nested")
                    self.assertEqual(source.inventory("nested"), {
                        "file.txt": hashlib.sha256(b"source bytes\n").hexdigest(),
                    })
                    source.unlink("escape")
                    destination.copy_tree_from(source)
                    self.assertEqual(destination.read_bytes("nested/file.txt"), b"source bytes\n")
                    destination.atomic_write("nested/file.txt", b"replacement\n")
                    self.assertEqual(destination.read_bytes("nested/file.txt"), b"replacement\n")
                    self.assertEqual(destination.inventory(), {
                        "nested/file.txt": hashlib.sha256(b"replacement\n").hexdigest(),
                    })
                    destination.remove_tree("nested")
                    self.assertEqual(destination.inventory(), {})
                finally:
                    source.close()
                    destination.close()

    def test_candidate_parent_swap_does_not_redirect_child_process(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            candidates_path = root / "target/headstart/candidates"
            candidates_path.mkdir(parents=True)
            candidate_path = candidates_path / "race-a"
            candidate_path.mkdir()
            (candidate_path / "evidence").mkdir()
            (root / "source").mkdir()
            decoy_path = root / "decoy"
            decoy_path.mkdir()
            (decoy_path / "race-a").mkdir()
            original_parent = root / "candidates-original"
            repo = pipeline.PinnedDir.open_root(root)
            candidates = pipeline.PinnedDir.open_child(repo, "target/headstart/candidates")
            candidate = pipeline.PinnedDir.open_child(candidates, "race-a")
            source = pipeline.PinnedDir.open_child(repo, "source")
            context = {
                "root": root,
                "candidate_path": candidate_path,
                "candidate_relative": "target/headstart/candidates/race-a",
                "source_path": root / "source",
                "source_relative": "source",
                "candidate_fd": candidate.fd,
                "source_root_fd": source.fd,
                "candidate_dir": candidate,
                "source_dir": source,
                "env": {
                    **os.environ,
                    "RUSTUI_P3_REPO_ROOT_FD": str(repo.fd),
                    "RUSTUI_P3_CANDIDATE_PARENT_FD": str(candidates.fd),
                    "RUSTUI_P3_CANDIDATE_ID": "race-a",
                    "RUSTUI_P3_CANDIDATE_ROOT_FD": str(candidate.fd),
                    "RUSTUI_P3_SOURCE_ROOT_FD": str(source.fd),
                },
                "pass_fds": (repo.fd, candidates.fd, candidate.fd, source.fd),
            }
            original_run = pipeline.subprocess.run
            context_token = pipeline._ACTIVE_FD_CONTEXT.set(context)

            def swap_before_child(*args, **kwargs):
                candidates_path.rename(original_parent)
                (root / "target/headstart/candidates").symlink_to(decoy_path, target_is_directory=True)
                try:
                    return original_run(*args, **kwargs)
                finally:
                    (root / "target/headstart/candidates").unlink()
                    original_parent.rename(candidates_path)

            try:
                with patch.object(pipeline.subprocess, "run", side_effect=swap_before_child):
                    passed = pipeline.run_step(
                        "P3-test-child",
                        [sys.executable, "-c", "from pathlib import Path; Path('child-marker').write_text('pinned')"],
                        root, candidate_path / "evidence", "0" * 64, [],
                    )
                self.assertTrue(passed)
                self.assertEqual((candidate_path / "child-marker").read_text(), "pinned")
                self.assertFalse((decoy_path / "race-a/child-marker").exists())
            finally:
                pipeline._ACTIVE_FD_CONTEXT.reset(context_token)
                source.close()
                candidate.close()
                candidates.close()
                repo.close()

    def test_default_patch_run_ids_do_not_collide_within_one_second(self) -> None:
        with (
            patch.object(pipeline, "datetime") as clock,
            patch.object(
                pipeline.uuid,
                "uuid4",
                side_effect=[
                    pipeline.uuid.UUID(int=0x11111111000000000000000000000000),
                    pipeline.uuid.UUID(int=0x22222222000000000000000000000000),
                ],
            ),
        ):
            clock.now.return_value.strftime.return_value = "20261022T120000Z"
            first = pipeline._default_patch_run_id()
            second = pipeline._default_patch_run_id()

        self.assertNotEqual(first, second)
        self.assertRegex(first, r"^pipeline-20261022T120000Z-[0-9a-f]{32}$")
        self.assertRegex(second, r"^pipeline-20261022T120000Z-[0-9a-f]{32}$")

    def fixture(self, root: Path) -> tuple[Path, Path]:
        scripts = root / "scripts"
        scripts.mkdir()
        shutil.copyfile(Path(pipeline.__file__).with_name("validate_candidate_evidence.py"),
                        scripts / "validate_candidate_evidence.py")
        shutil.copyfile(Path(pipeline.__file__).with_name("validate_patch_plan.py"),
                        scripts / "validate_patch_plan.py")
        shutil.copyfile(Path(pipeline.__file__).with_name("p3_candidate_fs.py"),
                        scripts / "p3_candidate_fs.py")
        source_root = root / "target/headstart/candidates/source-a"
        for variant, content in (("baseline", "before"), ("candidate", "after")):
            source = source_root / variant / "source"
            (source / "path-sources/pkg/src").mkdir(parents=True)
            (source / "path-sources/pkg/src/lib.rs").write_text(content)
            (source / "Cargo.toml").write_text("[workspace]\n")
            (source / "Cargo.lock").write_text("version = 4\n")
        patch_file = source_root / "approved.patch"
        patch_file.write_text("test patch\n")
        baseline_files = pipeline.inventory(source_root / "baseline/source")
        candidate_files = pipeline.inventory(source_root / "candidate/source")
        manifest = {
            "schema_version": 1,
            "candidate_id": "source-a",
            "repository": str(root),
            "patch_sha256": digest(patch_file),
            "snapshots": {
                name: {
                    "source": str(source_root / name / "source"),
                    "target": str(source_root / name / "target"),
                    "files": files,
                    "path_tree_sha256": pipeline.tree_digest(
                        {k.removeprefix("path-sources/pkg/"): v for k, v in files.items()
                         if k.startswith("path-sources/pkg/")}
                    ),
                }
                for name, files in (("baseline", baseline_files), ("candidate", candidate_files))
            },
        }
        manifest_path = source_root / "manifest.json"
        manifest_path.write_text(json.dumps(manifest))
        metadata = source_root / "metadata"
        metadata.mkdir()
        checksum = metadata / "checksum.json"
        checksum.write_text("{}\n")
        license_file = metadata / "LICENSE"
        license_file.write_text("test license\n")
        patch_plan = {
            "schema_version": 1,
            "candidate": {"id": "source-a",
                          "snapshot_sha256": manifest["snapshots"]["candidate"]["path_tree_sha256"],
                          "parent_manifest_sha256": digest(manifest_path)},
            "source": {"repository": "https://example.invalid/repo",
                       "revision": "a" * 40,
                       "packages": [{"name": "pkg", "version": "1.0.0",
                                     "checksum_file": "metadata/checksum.json",
                                     "checksum_sha256": digest(checksum),
                                     "license_expression": "MIT",
                                     "license_files": ["metadata/LICENSE"]}]},
            "patches": [{"path": "approved.patch", "sha256": digest(patch_file)}],
        }
        plan_path = source_root / "patch-plan.json"
        plan_path.write_text(json.dumps(patch_plan))
        intake = {
            "schema_version": 1,
            "approved": True,
            "approval_reference": "local-review-42",
            "candidate_id": "run-a",
            "source_candidate_id": "source-a",
            "source_manifest": {
                "path": manifest_path.relative_to(root).as_posix(),
                "sha256": digest(manifest_path),
            },
            "patch_plan": {"path": "patch-plan.json", "sha256": digest(plan_path)},
            "purpose": "pipeline fixture",
        }
        intake_path = root / "approved.json"
        intake_path.write_text(json.dumps(intake))
        return intake_path, source_root

    def test_invalid_run_id_is_rejected_before_candidate_creation(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            intake_path, _ = self.fixture(root)

            with self.assertRaisesRegex(pipeline.PipelineError, "run_id must be a safe identifier"):
                pipeline.run_pipeline(intake_path, root, "bad/run-id")

            self.assertFalse((root / "target/headstart/candidates/run-a").exists())

    def test_pipeline_inventory_enforces_snapshot_entry_budget(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "first.txt").write_text("one")
            (root / "second.txt").write_text("two")

            with patch.object(pipeline.candidate_snapshot, "MAX_SNAPSHOT_ENTRIES", 1):
                with self.assertRaisesRegex(pipeline.PipelineError, "exceeds 1 entries"):
                    pipeline.inventory(root)

    def test_pipeline_inventory_enforces_snapshot_byte_budget(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "payload.txt").write_text("four")

            with patch.object(pipeline.candidate_snapshot, "MAX_SNAPSHOT_BYTES", 3):
                with self.assertRaisesRegex(pipeline.PipelineError, "exceeds 3 regular-file bytes"):
                    pipeline.inventory(root)

    def test_pinned_pipeline_inventory_enforces_snapshot_budgets(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            source_path = root / "source"
            source_path.mkdir()
            (source_path / "first.txt").write_text("one")
            (source_path / "second.txt").write_text("two")
            candidate_path = root / "candidate"
            candidate_path.mkdir()
            source = pipeline.PinnedDir.open_root(source_path)
            candidate = pipeline.PinnedDir.open_root(candidate_path)
            context = {
                "source_path": source_path,
                "source_dir": source,
                "candidate_path": candidate_path,
                "candidate_dir": candidate,
            }
            context_token = pipeline._ACTIVE_FD_CONTEXT.set(context)
            try:
                with patch.object(pipeline.candidate_snapshot, "MAX_SNAPSHOT_ENTRIES", 1):
                    with self.assertRaisesRegex(pipeline.PipelineError, "exceeds 1 entries"):
                        pipeline.inventory(source_path)
                with patch.object(pipeline.candidate_snapshot, "MAX_SNAPSHOT_BYTES", 5):
                    with self.assertRaisesRegex(
                            pipeline.PipelineError, "exceeds 5 regular-file bytes"):
                        pipeline.inventory(source_path)
            finally:
                pipeline._ACTIVE_FD_CONTEXT.reset(context_token)
                source.close()
                candidate.close()

    def test_copy_source_handles_deep_tree_with_bounded_descriptors(self) -> None:
        depth = 1100
        directory_flags = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW
        with tempfile.TemporaryDirectory() as tmp, ExitStack() as cleanup:
            root = Path(tmp)
            pinned = cleanup.enter_context(pipeline.PinnedDir.open_root(root))
            cleanup.callback(pinned.remove_tree, "source")
            cleanup.callback(pinned.remove_tree, "candidate")
            source = root / "source"
            destination = root / "candidate/source"
            source.mkdir()
            destination.parent.mkdir()

            active_fd = os.open(source, directory_flags)
            try:
                for _ in range(depth):
                    os.mkdir("d", dir_fd=active_fd)
                    child_fd = os.open("d", directory_flags, dir_fd=active_fd)
                    os.close(active_fd)
                    active_fd = child_fd
                leaf_fd = os.open("leaf", os.O_WRONLY | os.O_CREAT | os.O_EXCL,
                                  0o600, dir_fd=active_fd)
                try:
                    os.write(leaf_fd, b"x")
                finally:
                    os.close(leaf_fd)
            finally:
                os.close(active_fd)

            original_soft, hard_limit = resource.getrlimit(resource.RLIMIT_NOFILE)
            low_limit = max(32, len(os.listdir("/dev/fd")) + 8)
            if low_limit >= depth or (hard_limit != resource.RLIM_INFINITY and hard_limit < low_limit):
                self.skipTest("cannot set a descriptor limit far below the tree depth")
            limit_changed = False
            try:
                resource.setrlimit(resource.RLIMIT_NOFILE, (low_limit, hard_limit))
                limit_changed = True
                copied = pipeline._copy_source(source, destination)
            finally:
                if limit_changed:
                    resource.setrlimit(resource.RLIMIT_NOFILE, (original_soft, hard_limit))

            self.assertEqual(copied, {
                "/".join(["d"] * depth + ["leaf"]): hashlib.sha256(b"x").hexdigest()
            })
            current_fd = os.open(destination, directory_flags)
            try:
                for _ in range(depth):
                    child_fd = os.open("d", directory_flags, dir_fd=current_fd)
                    os.close(current_fd)
                    current_fd = child_fd
                leaf_fd = os.open("leaf", os.O_RDONLY | os.O_NOFOLLOW, dir_fd=current_fd)
                try:
                    self.assertEqual(os.read(leaf_fd, 2), b"x")
                finally:
                    os.close(leaf_fd)
            finally:
                os.close(current_fd)

    def test_patch_copy_rejects_oversized_input_before_creating_destination(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            source = root / "approved.patch"
            source.write_bytes(b"12345")
            destination = root / "candidate.patch"

            with self.assertRaisesRegex(pipeline.PipelineError, "exceeds the 4-byte copy limit"):
                pipeline._copy_regular_file(source, destination, "approved patch", max_bytes=4)

            self.assertFalse(destination.exists())

    def test_load_json_enforces_p3g_manifest_size_limit(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            exact_limit = root / "exact.json"
            exact_limit.write_bytes(b'{"a":1}')
            oversized = root / "oversized.json"
            oversized.write_bytes(b'{"a":10}')

            with patch.object(pipeline, "MAX_JSON_BYTES", 7):
                self.assertEqual(pipeline.load_json(exact_limit), ({"a": 1}, b'{"a":1}'))
                with self.assertRaisesRegex(pipeline.PipelineError, "exceeds 7 bytes"):
                    pipeline.load_json(oversized)

    def test_intake_accepts_schema_three_multi_file_patch_bindings(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            intake_path, source_root = self.fixture(root)
            manifest_path = source_root / "manifest.json"
            manifest = json.loads(manifest_path.read_text())
            patch_path = source_root / "approved.patch"
            patch_hash = digest(patch_path)
            manifest["schema_version"] = 3
            manifest["patch_sha256"] = hashlib.sha256(
                f"approved.patch:{patch_hash}".encode()
            ).hexdigest()
            manifest["patches"] = [{
                "path": "approved.patch",
                "file": "src/lib.rs",
                "sha256": patch_hash,
                "preimage_sha256": "1" * 64,
                "postimage_sha256": "2" * 64,
            }]
            manifest_path.write_text(json.dumps(manifest))

            plan_path = source_root / "patch-plan.json"
            plan = json.loads(plan_path.read_text())
            plan["candidate"]["parent_manifest_sha256"] = digest(manifest_path)
            plan_path.write_text(json.dumps(plan))
            intake = json.loads(intake_path.read_text())
            intake["source_manifest"]["sha256"] = digest(manifest_path)
            intake["patch_plan"]["sha256"] = digest(plan_path)
            intake_path.write_text(json.dumps(intake))

            source_id, _, accepted, _, _ = pipeline.validate_intake(
                intake, root, intake_path
            )
            self.assertEqual(source_id, "run-a")
            self.assertEqual(accepted["schema_version"], 3)
            self.assertEqual(
                pipeline._patch_bindings(accepted, is_plan=False),
                [("approved.patch", patch_hash)],
            )

    def test_patch_bindings_reject_empty_ordered_patch_lists(self) -> None:
        records = [
            (True, {"patches": []}),
            (False, {"schema_version": 3, "patches": []}),
        ]
        for is_plan, record in records:
            with self.subTest(is_plan=is_plan):
                with self.assertRaisesRegex(pipeline.PipelineError, "at least one patch"):
                    pipeline._patch_bindings(record, is_plan=is_plan)

    def test_pipeline_applies_schema_three_multi_file_candidate(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            intake_path, source_root = self.fixture(root)
            for script in ("apply_candidate_patch.py", "validate_candidate_snapshot.py", "p3_candidate_fs.py"):
                shutil.copyfile(
                    Path(pipeline.__file__).with_name(script), root / "scripts" / script,
                )
            package_name = "gpui-pre-0.3.9"
            package_dirs = {}
            for variant in ("baseline", "candidate"):
                source = source_root / variant / "source"
                old_package = source / "path-sources/pkg"
                package = source / f"path-sources/{package_name}"
                old_package.rename(package)
                package_dirs[variant] = package

            before = {
                "src/lib.rs": b"before-lib\n",
                "src/other.rs": b"before-other\n",
            }
            after = {
                "src/lib.rs": b"after-lib\n",
                "src/other.rs": b"after-other\n",
            }
            for relative, contents in before.items():
                target = package_dirs["baseline"] / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(contents)
            for relative, contents in after.items():
                target = package_dirs["candidate"] / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(contents)

            patch_records = []
            descriptors = []
            for index, relative in enumerate(before):
                patch_name = "approved.patch" if index == 0 else f"file-{index}.patch"
                patch_bytes = (
                    f"--- a/vendor/{package_name}/{relative}\n"
                    f"+++ b/vendor/{package_name}/{relative}\n"
                    "@@ -1,1 +1,1 @@\n"
                    f"-{before[relative].decode()}"
                    f"+{after[relative].decode()}"
                ).encode()
                (source_root / patch_name).write_bytes(patch_bytes)
                patch_hash = digest(source_root / patch_name)
                descriptors.append(f"{patch_name}:{patch_hash}")
                patch_records.append({
                    "path": patch_name,
                    "file": relative,
                    "sha256": patch_hash,
                    "preimage_sha256": hashlib.sha256(before[relative]).hexdigest(),
                    "postimage_sha256": hashlib.sha256(after[relative]).hexdigest(),
                })

            manifest_path = source_root / "manifest.json"
            manifest = json.loads(manifest_path.read_text())
            manifest["schema_version"] = 3
            manifest["patches"] = patch_records
            manifest["patch_sha256"] = hashlib.sha256(
                "\n".join(descriptors).encode()
            ).hexdigest()
            for variant in ("baseline", "candidate"):
                files = pipeline.inventory(source_root / variant / "source")
                manifest["snapshots"][variant]["files"] = files
                package_prefix = f"path-sources/{package_name}/"
                package_files = {
                    path.removeprefix(package_prefix): content_hash
                    for path, content_hash in files.items()
                    if path.startswith(package_prefix)
                }
                manifest["snapshots"][variant]["path_tree_sha256"] = pipeline.tree_digest(
                    package_files
                )
            manifest_path.write_text(json.dumps(manifest))

            plan_path = source_root / "patch-plan.json"
            plan = json.loads(plan_path.read_text())
            plan["candidate"]["parent_manifest_sha256"] = digest(manifest_path)
            plan["patches"] = [
                {"path": record["path"], "sha256": record["sha256"]}
                for record in patch_records
            ]
            plan["candidate"]["snapshot_sha256"] = manifest["snapshots"]["candidate"]["path_tree_sha256"]
            plan_path.write_text(json.dumps(plan))
            intake = json.loads(intake_path.read_text())
            intake["source_manifest"]["sha256"] = digest(manifest_path)
            intake["patch_plan"]["sha256"] = digest(plan_path)
            intake_path.write_text(json.dumps(intake))

            original_step = pipeline.run_step
            patch_output = []

            def synthetic_non_patch_steps(name, command, cwd, evidence_dir,
                                          manifest_hash, events):
                if name == "P3K-exact-patch":
                    passed = original_step(name, command, cwd, evidence_dir,
                                           manifest_hash, events)
                    patch_output.append(
                        (passed, (evidence_dir / events[-1]["log"]["path"]).read_text())
                    )
                    return passed
                log = evidence_dir / f"{len(events) + 1:02d}-{name}.log"
                log.write_text("synthetic success\n")
                events.append({
                    "step": name, "status": "passed", "exit_status": 0,
                    "command": command, "manifest_sha256": manifest_hash,
                    "log": {"path": log.name, "sha256": digest(log)},
                })
                return True

            with patch.object(pipeline, "run_step", side_effect=synthetic_non_patch_steps):
                result = pipeline.run_pipeline(intake_path, root, "schema3-multifile")

            self.assertEqual(result["status"], "partial", patch_output)
            p3k = next(event for event in result["events"]
                       if event["step"] == "P3K-exact-patch")
            self.assertEqual(p3k["status"], "passed")
            patch_result = json.loads(
                (root / "target/headstart/candidates/run-a/patch-checks/"
                 "schema3-multifile/result.json").read_text()
            )
            self.assertEqual(patch_result["status"], "passed")
            self.assertEqual(
                {record["file"] for record in patch_result["patches"]}, set(before)
            )
            patched_copy = Path(patch_result["output_package"])
            if not patched_copy.is_absolute():
                patched_copy = root / patched_copy
            for relative, contents in after.items():
                self.assertEqual((patched_copy / relative).read_bytes(), contents)
            for relative, contents in before.items():
                self.assertEqual((package_dirs["baseline"] / relative).read_bytes(), contents)
                self.assertEqual((package_dirs["candidate"] / relative).read_bytes(),
                                 after[relative])

    @unittest.skipUnless(hasattr(os, "mkfifo"), "FIFO inputs require os.mkfifo")
    def test_source_copy_closes_owned_descriptors_when_destination_creation_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "source/baseline/source").mkdir(parents=True)
            (root / "destination").mkdir()
            with (
                pipeline.PinnedDir.open_root(root / "source") as source,
                pipeline.PinnedDir.open_root(root / "destination") as destination,
            ):
                context_token = pipeline._ACTIVE_FD_CONTEXT.set({
                    "source_root_fd": source.fd,
                    "candidate_fd": destination.fd,
                    "source_path": root / "source",
                    "candidate_path": root / "destination",
                })
                owned = []
                original_from_fd = pipeline.PinnedDir.from_fd
                original_open_child = pipeline.PinnedDir.open_child

                def capture_from_fd(*args, **kwargs):
                    directory = original_from_fd(*args, **kwargs)
                    owned.append(directory.fd)
                    return directory

                def capture_open_child(*args, **kwargs):
                    directory = original_open_child(*args, **kwargs)
                    owned.append(directory.fd)
                    return directory

                try:
                    with (
                        patch.object(pipeline.PinnedDir, "from_fd", side_effect=capture_from_fd),
                        patch.object(pipeline.PinnedDir, "open_child", side_effect=capture_open_child),
                        patch.object(pipeline.PinnedDir, "create_child",
                                     side_effect=pipeline.CandidateFSError("destination unavailable")),
                    ):
                        with self.assertRaisesRegex(pipeline.PipelineError, "destination unavailable"):
                            pipeline._copy_source(root / "source/baseline/source",
                                                  root / "destination/baseline/source")
                    self.assertEqual(len(owned), 3)
                    for descriptor in owned:
                        with self.assertRaises(OSError):
                            os.fstat(descriptor)
                    # Borrowed descriptors still belong to the caller.
                    self.assertEqual(os.fstat(source.fd).st_ino, source.identity()[1])
                    self.assertEqual(os.fstat(destination.fd).st_ino, destination.identity()[1])
                finally:
                    pipeline._ACTIVE_FD_CONTEXT.reset(context_token)

    def test_rejects_fifo_intake_without_blocking(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            fifo = root / "approved.json"
            os.mkfifo(fifo)

            with self.assertRaisesRegex(pipeline.PipelineError, "regular file"):
                pipeline.run_pipeline(fifo, root)

            self.assertFalse((root / "target/headstart/candidates").exists())

    @unittest.skipUnless(hasattr(os, "mkfifo"), "FIFO inputs require os.mkfifo")
    def test_rejects_fifo_source_manifest_without_blocking(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            intake, source = self.fixture(root)
            manifest = source / "manifest.json"
            manifest.unlink()
            os.mkfifo(manifest)

            with self.assertRaisesRegex(pipeline.PipelineError, "regular file"):
                pipeline.run_pipeline(intake, root)

            self.assertFalse((root / "target/headstart/candidates/run-a").exists())

    @unittest.skipUnless(hasattr(os, "mkfifo"), "FIFO inputs require os.mkfifo")
    def test_rejects_fifo_patch_plan_without_blocking(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            intake, source = self.fixture(root)
            plan = source / "patch-plan.json"
            plan.unlink()
            os.mkfifo(plan)

            with self.assertRaisesRegex(pipeline.PipelineError, "regular file"):
                pipeline.run_pipeline(intake, root)

            self.assertFalse((root / "target/headstart/candidates/run-a").exists())

    def test_rejects_symlink_inserted_during_source_copy(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            source = root / "source"
            source.mkdir()
            source_file = source / "input.txt"
            source_file.write_text("pinned contents\n")
            external_file = root / "external.txt"
            external_file.write_text("external secret\n")
            destination = root / "copy"
            original_copy = pipeline.PinnedDir.copy_tree_from

            def insert_symlink(destination_dir, source_dir, **kwargs):
                source_file.unlink()
                source_file.symlink_to(external_file)
                return original_copy(destination_dir, source_dir, **kwargs)

            with patch.object(pipeline.PinnedDir, "copy_tree_from", new=insert_symlink):
                with self.assertRaisesRegex(pipeline.PipelineError, "symlink rejected"):
                    pipeline._copy_source(source, destination)

            copied_path = destination / "input.txt"
            self.assertFalse(copied_path.exists())

    def test_rejects_directory_symlink_inserted_during_source_copy(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            source = root / "source"
            nested = source / "nested"
            nested.mkdir(parents=True)
            (nested / "inside.txt").write_text("pinned contents\n")
            external = root / "external"
            external.mkdir()
            (external / "secret.txt").write_text("external secret\n")
            destination = root / "copy"
            original_open = os.open
            original_copy = pipeline.PinnedDir.copy_tree_from
            swapped = False
            copying = False

            def swap_before_open(path, flags, *args, **kwargs):
                nonlocal swapped
                if copying and path == "nested" and kwargs.get("dir_fd") is not None and not swapped:
                    nested.rename(source / "detached")
                    nested.symlink_to(external, target_is_directory=True)
                    swapped = True
                return original_open(path, flags, *args, **kwargs)

            def copy_with_injection(destination_dir, source_dir, **kwargs):
                nonlocal copying
                copying = True
                try:
                    return original_copy(destination_dir, source_dir, **kwargs)
                finally:
                    copying = False

            with patch.object(pipeline.os, "open", side_effect=swap_before_open), \
                    patch.object(pipeline.PinnedDir, "copy_tree_from", new=copy_with_injection):
                with self.assertRaisesRegex(pipeline.PipelineError, "symlink rejected"):
                    pipeline._copy_source(source, destination)

            self.assertTrue(swapped)
            self.assertFalse((destination / "nested/secret.txt").exists())

    def test_keeps_open_directory_when_source_path_is_swapped(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            source = root / "source"
            nested = source / "nested"
            nested.mkdir(parents=True)
            (nested / "inside.txt").write_text("pinned contents\n")
            external = root / "external"
            external.mkdir()
            (external / "secret.txt").write_text("external secret\n")
            destination = root / "copy"
            original_open = os.open
            original_copy = pipeline.PinnedDir.copy_tree_from
            swapped = False
            copying = False

            def swap_after_open(path, flags, *args, **kwargs):
                nonlocal swapped
                descriptor = original_open(path, flags, *args, **kwargs)
                if copying and path == "nested" and kwargs.get("dir_fd") is not None and not swapped:
                    nested.rename(source / "detached")
                    nested.symlink_to(external, target_is_directory=True)
                    swapped = True
                return descriptor

            def copy_with_injection(destination_dir, source_dir, **kwargs):
                nonlocal copying
                copying = True
                try:
                    return original_copy(destination_dir, source_dir, **kwargs)
                finally:
                    copying = False

            with patch.object(pipeline.os, "open", side_effect=swap_after_open), \
                    patch.object(pipeline.PinnedDir, "copy_tree_from", new=copy_with_injection):
                with self.assertRaisesRegex(pipeline.PipelineError, "symlink rejected"):
                    pipeline._copy_source(source, destination)

            self.assertTrue(swapped)
            self.assertEqual((destination / "nested/inside.txt").read_text(), "pinned contents\n")
            self.assertFalse((destination / "nested/secret.txt").exists())

    @unittest.skipUnless(hasattr(os, "mkfifo"), "FIFO inputs require os.mkfifo")
    def test_rejects_fifo_inserted_during_source_copy_without_blocking(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            source = root / "source"
            source.mkdir()
            source_file = source / "input.txt"
            source_file.write_text("pinned contents\n")
            destination = root / "copy"
            original_copy = pipeline.PinnedDir.copy_tree_from

            def insert_fifo(destination_dir, source_dir, **kwargs):
                source_file.unlink()
                os.mkfifo(source_file)
                return original_copy(destination_dir, source_dir, **kwargs)

            with patch.object(pipeline.PinnedDir, "copy_tree_from", new=insert_fifo):
                with self.assertRaisesRegex(pipeline.PipelineError, "regular file"):
                    pipeline._copy_source(source, destination)

            self.assertFalse((destination / "input.txt").exists())

    def test_copies_snapshot_pins_hashes_and_runs_stages_in_order(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            intake, source = self.fixture(root)
            observed: list[str] = []
            original_step = pipeline.run_step

            def fake_step(name, command, cwd, evidence_dir, manifest_hash, events):
                observed.append(name)
                if name in ("P3E-patch-plan", "P3F-evidence-record"):
                    return original_step(name, command, cwd, evidence_dir, manifest_hash, events)
                log = evidence_dir / f"{len(events) + 1:02d}-{name}.log"
                log.write_text("synthetic success\n")
                events.append({"step": name, "status": "passed", "exit_status": 0,
                               "command": command, "manifest_sha256": manifest_hash,
                               "log": {"path": log.name, "sha256": digest(log)}})
                return True

            with patch.object(pipeline, "run_step", side_effect=fake_step):
                result = pipeline.run_pipeline(intake, root, "check-a")

            self.assertEqual(observed, [
                "P3E-patch-plan", "P3G-snapshot", "P3H-baseline", "P3H-candidate",
                "P3I-J-source-and-graph-comparison", "P3K-exact-patch",
                "P3F-evidence-record",
            ])
            candidate = root / "target/headstart/candidates/run-a"
            self.assertEqual(result["status"], "partial")
            self.assertEqual((candidate / "baseline/source/Cargo.lock").read_text(), "version = 4\n")
            self.assertEqual((candidate / "source-hash-manifest.json").exists(), True)
            self.assertFalse((candidate / "manifest.json").stat().st_mode & 0o222)
            evidence_record = json.loads((candidate / "evidence/P3F-record.json").read_text())
            intake_hash = digest(intake)
            self.assertEqual(
                evidence_record["command"],
                [sys.executable, str(root.resolve() / "scripts/run_candidate_pipeline.py"),
                 str(intake.resolve()), "--root", str(root.resolve()), "--run-id", "check-a"],
            )
            self.assertEqual(evidence_record["provenance"]["approved_intake_sha256"], intake_hash)
            self.assertEqual((candidate / "approved-intake.json").read_bytes(), intake.read_bytes())
            self.assertIn({"path": "approved-intake.json", "sha256": intake_hash},
                          evidence_record["evidence"])
            self.assertEqual((source / "baseline/source/path-sources/pkg/src/lib.rs").read_text(), "before")
            self.assertEqual(result["gates"]["gui"], "not_run")

    def test_source_hash_tree_digest_uses_the_captured_inventory(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            intake, _ = self.fixture(root)
            original_inventory = pipeline.inventory
            calls: dict[str, int] = {}
            candidate_root = root / "target/headstart/candidates/run-a"

            def alternating_inventory(path: Path) -> dict[str, str]:
                files = original_inventory(path)
                try:
                    relative = Path(path).relative_to(candidate_root)
                except ValueError:
                    return files
                if relative.parts and relative.parts[0] in ("baseline", "candidate"):
                    variant = relative.parts[0]
                    calls[variant] = calls.get(variant, 0) + 1
                    if calls[variant] == 2:
                        return {**files, "concurrent-change.rs": "f" * 64}
                return files

            original_step = pipeline.run_step

            def synthetic_step(name, command, cwd, evidence_dir, manifest_hash, events):
                if name in ("P3E-patch-plan", "P3F-evidence-record"):
                    return original_step(name, command, cwd, evidence_dir,
                                         manifest_hash, events)
                log = evidence_dir / f"{len(events) + 1:02d}-{name}.log"
                log.write_text("synthetic success\n")
                events.append({"step": name, "status": "passed", "exit_status": 0,
                               "command": command, "manifest_sha256": manifest_hash,
                               "log": {"path": log.name, "sha256": digest(log)}})
                return True

            with (patch.object(pipeline, "inventory", side_effect=alternating_inventory),
                  patch.object(pipeline, "run_step", side_effect=synthetic_step)):
                pipeline.run_pipeline(intake, root, "single-inventory")

            candidate = candidate_root / "source-hash-manifest.json"
            manifest = json.loads(candidate.read_text(encoding="utf-8"))
            for snapshot in manifest["snapshots"].values():
                self.assertEqual(snapshot["tree_sha256"],
                                 pipeline.tree_digest(snapshot["files"]))

    def test_p3f_record_hash_pins_copied_approved_patch_plan(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            intake, source = self.fixture(root)

            def synthetic_step(name, command, cwd, evidence_dir, manifest_hash, events):
                log = evidence_dir / f"{len(events) + 1:02d}-{name}.log"
                log.write_text("synthetic success\n")
                events.append({
                    "step": name, "status": "passed", "exit_status": 0,
                    "command": command, "manifest_sha256": manifest_hash,
                    "log": {"path": log.name, "sha256": digest(log)},
                })
                return True

            with patch.object(pipeline, "run_step", side_effect=synthetic_step):
                pipeline.run_pipeline(intake, root, "p3e-copy-binding")

            candidate = root / "target/headstart/candidates/run-a"
            record = json.loads((candidate / "evidence/P3F-record.json").read_text())
            plan_copy = candidate / "evidence/approved-patch-plan.json"
            plan_hash = json.loads(intake.read_text())["patch_plan"]["sha256"]
            self.assertEqual(digest(plan_copy), plan_hash)
            self.assertIn(
                {"path": "evidence/approved-patch-plan.json", "sha256": plan_hash},
                record["evidence"],
            )

    def test_failed_local_stage_is_reflected_in_pipeline_evidence_exit_status(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            intake, _ = self.fixture(root)
            original_step = pipeline.run_step

            def fail_snapshot(name, command, cwd, evidence_dir, manifest_hash, events):
                if name == "P3G-snapshot":
                    log = evidence_dir / f"{len(events) + 1:02d}-{name}.log"
                    log.write_text("snapshot rejected\n")
                    events.append({"step": name, "status": "failed", "exit_status": 1,
                                   "command": command, "manifest_sha256": manifest_hash,
                                   "log": {"path": log.name, "sha256": digest(log)}})
                    return False
                if name in ("P3E-patch-plan", "P3F-evidence-record"):
                    return original_step(name, command, cwd, evidence_dir,
                                         manifest_hash, events)
                log = evidence_dir / f"{len(events) + 1:02d}-{name}.log"
                log.write_text("synthetic success\n")
                events.append({"step": name, "status": "passed", "exit_status": 0,
                               "command": command, "manifest_sha256": manifest_hash,
                               "log": {"path": log.name, "sha256": digest(log)}})
                return True

            with patch.object(pipeline, "run_step", side_effect=fail_snapshot):
                result = pipeline.run_pipeline(intake, root, "failed-snapshot")

            candidate = root / "target/headstart/candidates/run-a"
            evidence_record = json.loads((candidate / "evidence/P3F-record.json").read_text())
            p3f = next(event for event in result["events"]
                       if event["step"] == "P3F-evidence-record")
            self.assertEqual(result["status"], "failed")
            self.assertEqual(evidence_record["exit_status"], 1)
            self.assertEqual(p3f["status"], "passed")

    def test_source_drift_after_p3g_fails_final_candidate_integrity_check(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            intake, _ = self.fixture(root)
            original_step = pipeline.run_step

            def mutate_after_snapshot(name, command, cwd, evidence_dir,
                                      manifest_hash, events):
                if name in ("P3E-patch-plan", "P3F-evidence-record"):
                    return original_step(name, command, cwd, evidence_dir,
                                         manifest_hash, events)
                log = evidence_dir / f"{len(events) + 1:02d}-{name}.log"
                log.write_text("synthetic success\n")
                events.append({"step": name, "status": "passed", "exit_status": 0,
                               "command": command, "manifest_sha256": manifest_hash,
                               "log": {"path": log.name, "sha256": digest(log)}})
                if name == "P3H-candidate":
                    candidate = evidence_dir.parent
                    (candidate / "baseline/source/Cargo.lock").write_text(
                        "version = 4\n# drift after P3G\n"
                    )
                return True

            with patch.object(pipeline, "run_step", side_effect=mutate_after_snapshot):
                result = pipeline.run_pipeline(intake, root, "source-drift-after-p3g")

            integrity = next(event for event in result["events"]
                             if event["step"] == "P3G-final-source-integrity")
            initial_snapshot = next(event for event in result["events"]
                                    if event["step"] == "P3G-snapshot")
            candidate = root / "target/headstart/candidates/run-a"
            evidence_record = json.loads(
                (candidate / "evidence/P3F-record.json").read_text()
            )
            self.assertEqual(initial_snapshot["status"], "passed")
            self.assertEqual(integrity["status"], "failed")
            self.assertEqual(integrity["exit_status"], 1)
            self.assertIn("baseline source closure changed", integrity["reason"])
            self.assertEqual(result["status"], "failed")
            self.assertEqual(evidence_record["exit_status"], 1)

    def test_p3f_cannot_leave_mutated_source_with_partial_result(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            intake, _ = self.fixture(root)
            original_step = pipeline.run_step

            def mutate_after_p3f(name, command, cwd, evidence_dir, manifest_hash, events):
                if name in ("P3E-patch-plan", "P3F-evidence-record"):
                    passed = original_step(name, command, cwd, evidence_dir, manifest_hash, events)
                    if name == "P3F-evidence-record":
                        (evidence_dir.parent / "baseline/source/Cargo.lock").write_text("drift\n")
                    return passed
                log = evidence_dir / f"{len(events) + 1:02d}-{name}.log"
                log.write_text("synthetic success\n")
                events.append({"step": name, "status": "passed", "exit_status": 0,
                               "command": command, "manifest_sha256": manifest_hash,
                               "log": {"path": log.name, "sha256": digest(log)}})
                return True

            with patch.object(pipeline, "run_step", side_effect=mutate_after_p3f):
                result = pipeline.run_pipeline(intake, root, "p3f-source-drift")

            candidate = root / "target/headstart/candidates/run-a"
            record = json.loads((candidate / "evidence/P3F-record.json").read_text())
            integrity = next(event for event in result["events"]
                             if event["step"] == "P3G-post-P3F-integrity")
            self.assertEqual(integrity["status"], "failed")
            self.assertIn("baseline source closure changed", integrity["reason"])
            self.assertEqual(result["status"], "failed")
            self.assertEqual(record["exit_status"], 1)

    def test_final_integrity_rejects_hard_link_added_after_snapshot_validation(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            intake, _ = self.fixture(root)
            original_step = pipeline.run_step

            def add_external_hard_link(name, command, cwd, evidence_dir, manifest_hash, events):
                if name in ("P3E-patch-plan", "P3F-evidence-record"):
                    return original_step(name, command, cwd, evidence_dir, manifest_hash, events)
                log = evidence_dir / f"{len(events) + 1:02d}-{name}.log"
                log.write_text("synthetic success\n")
                events.append({"step": name, "status": "passed", "exit_status": 0,
                               "command": command, "manifest_sha256": manifest_hash,
                               "log": {"path": log.name, "sha256": digest(log)}})
                if name == "P3H-candidate":
                    candidate = evidence_dir.parent
                    source_file = candidate / "baseline/source/path-sources/pkg/src/lib.rs"
                    os.link(source_file, root / "outside-hard-link.rs")
                return True

            with patch.object(pipeline, "run_step", side_effect=add_external_hard_link):
                result = pipeline.run_pipeline(intake, root, "hard-link-after-p3g")

            candidate = root / "target/headstart/candidates/run-a"
            integrity = next(event for event in result["events"]
                             if event["step"] == "P3G-final-source-integrity")
            self.assertTrue((root / "outside-hard-link.rs").exists())
            self.assertEqual(integrity["status"], "failed")
            self.assertIn("baseline source closure", integrity["reason"])
            self.assertEqual(result["status"], "failed")

    def test_failed_p3f_stage_is_reflected_in_its_evidence_record(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            intake, _ = self.fixture(root)
            original_step = pipeline.run_step

            def fail_p3f(name, command, cwd, evidence_dir, manifest_hash, events):
                if name == "P3E-patch-plan":
                    return original_step(name, command, cwd, evidence_dir, manifest_hash, events)
                log = evidence_dir / f"{len(events) + 1:02d}-{name}.log"
                failed = name == "P3F-evidence-record"
                log.write_text("synthetic failure\n" if failed else "synthetic success\n")
                events.append({
                    "step": name, "status": "failed" if failed else "passed",
                    "exit_status": 1 if failed else 0, "command": command,
                    "manifest_sha256": manifest_hash,
                    "log": {"path": log.name, "sha256": digest(log)},
                })
                return not failed

            with patch.object(pipeline, "run_step", side_effect=fail_p3f):
                result = pipeline.run_pipeline(intake, root, "failed-p3f")

            candidate = root / "target/headstart/candidates/run-a"
            evidence_record = json.loads((candidate / "evidence/P3F-record.json").read_text())
            p3f = next(event for event in result["events"]
                       if event["step"] == "P3F-evidence-record")
            self.assertEqual(result["status"], "failed")
            self.assertEqual(evidence_record["exit_status"], 1)
            self.assertEqual(p3f["status"], "failed")

    def test_run_step_records_validator_timeout_as_failed_step(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            evidence = root / "evidence"
            evidence.mkdir()
            events: list[dict[str, object]] = []
            with patch.object(pipeline, "STEP_TIMEOUT_SECONDS", 0.1):
                passed = pipeline.run_step(
                    "P3H-baseline",
                    [sys.executable, "-c", "import sys,time; print('partial output', flush=True); time.sleep(5)"],
                    root, evidence, "a" * 64, events
                )

            self.assertFalse(passed)
            self.assertEqual(events[0]["status"], "failed")
            self.assertEqual(events[0]["exit_status"], 124)
            log = evidence / events[0]["log"]["path"]
            self.assertIn("partial output", log.read_text())
            self.assertIn("0.1-second timeout", log.read_text())
            self.assertFalse((evidence / "P3H-baseline.json").exists())

    def test_run_step_bounds_combined_output_and_does_not_write_truncated_p3h_report(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            evidence = root / "evidence"
            evidence.mkdir()
            events: list[dict[str, object]] = []
            command = [sys.executable, "-c", (
                "import os,time; os.write(1,b'12345'); os.write(2,b'67890'); time.sleep(5)"
            )]

            with patch.object(pipeline, "MAX_STEP_OUTPUT_BYTES", 8):
                passed = pipeline.run_step(
                    "P3H-baseline", command, root, evidence, "a" * 64, events
                )

            self.assertFalse(passed)
            self.assertEqual(events[0]["exit_status"], 125)
            self.assertEqual(events[0]["status"], "failed")
            log = evidence / events[0]["log"]["path"]
            self.assertIn("8-byte limit", log.read_text())
            self.assertFalse((evidence / "P3H-baseline.json").exists())

    def test_run_step_accepts_output_exactly_at_limit(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            evidence = root / "evidence"
            evidence.mkdir()
            events: list[dict[str, object]] = []
            command = [sys.executable, "-c", "import os; os.write(1,b'12345678')"]

            with patch.object(pipeline, "MAX_STEP_OUTPUT_BYTES", 8):
                passed = pipeline.run_step(
                    "P3G-snapshot", command, root, evidence, "a" * 64, events
                )

            self.assertTrue(passed)
            self.assertEqual(events[0]["status"], "passed")
            log = evidence / events[0]["log"]["path"]
            self.assertEqual(log.read_text(), "12345678")

    def test_concurrent_run_steps_keep_pinned_fd_context_isolated(self) -> None:
        observed: list[tuple[str, tuple[int, ...]]] = []
        observed_lock = threading.Lock()
        original_popen = pipeline.subprocess.Popen

        def fake_popen(*args, **kwargs):
            with observed_lock:
                observed.append((kwargs["env"]["marker"], kwargs["pass_fds"]))
            return original_popen(
                [sys.executable, "-c", "print('ok')"], **kwargs
            )

        def invoke(marker: str, fd: int, log_name: str) -> bool:
            evidence = root / log_name
            evidence.mkdir()
            context = {"candidate_path": root / f"candidate-{marker}",
                       "source_path": root / f"source-{marker}",
                       "candidate_dir": None, "source_dir": None,
                       "env": {**os.environ, "marker": marker}, "pass_fds": (fd,)}
            token = pipeline._ACTIVE_FD_CONTEXT.set(context)
            try:
                return pipeline.run_step("parallel", ["validator"], root, evidence,
                                         "a" * 64, [])
            finally:
                pipeline._ACTIVE_FD_CONTEXT.reset(token)

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            fd_pairs = [os.pipe(), os.pipe()]
            with (
                patch.object(pipeline, "_fd_anchored_command", side_effect=lambda command, _: command),
                patch.object(pipeline.subprocess, "Popen", side_effect=fake_popen),
                ThreadPoolExecutor(max_workers=2) as workers,
            ):
                results = [
                    workers.submit(invoke, "first", fd_pairs[0][0], "first"),
                    workers.submit(invoke, "second", fd_pairs[1][0], "second"),
                ]
                self.assertEqual([result.result(timeout=10) for result in results], [True, True])
            for read_fd, write_fd in fd_pairs:
                os.close(read_fd)
                os.close(write_fd)

        self.assertCountEqual(observed, [
            ("first", (fd_pairs[0][0],)), ("second", (fd_pairs[1][0],))
        ])

    def test_p3h_report_keeps_stderr_out_of_machine_readable_stdout(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            evidence = Path(tmp)
            events: list[dict[str, object]] = []
            stdout = '{"status":"valid"}\n'
            stderr = "warning: Cargo emitted a diagnostic\n"
            command = [sys.executable, "-c", (
                "import sys; print('" + stdout.strip() + "'); "
                "print('" + stderr.strip() + "', file=sys.stderr)"
            )]
            passed = pipeline.run_step(
                "P3H-baseline", command, evidence, evidence, "a" * 64, events
            )

            self.assertTrue(passed)
            self.assertEqual((evidence / "P3H-baseline.json").read_text(), stdout)
            log = evidence / events[0]["log"]["path"]
            self.assertIn(stdout.strip(), log.read_text())
            self.assertIn(stderr.strip(), log.read_text())

    def test_pipeline_refuses_symlink_at_p3h_report_output(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            intake, _ = self.fixture(root)
            outside = root / "outside-report.txt"
            outside.write_text("keep this file", encoding="utf-8")
            original_step = pipeline.run_step

            def insert_report_symlink(name, command, cwd, evidence_dir, manifest_hash, events):
                if name in ("P3E-patch-plan", "P3F-evidence-record"):
                    return original_step(name, command, cwd, evidence_dir, manifest_hash, events)
                if name == "P3H-baseline":
                    (evidence_dir / "P3H-baseline.json").symlink_to(outside)
                    original_popen = pipeline.subprocess.Popen

                    def fake_popen(*args, **kwargs):
                        return original_popen(
                            [sys.executable, "-c", "print('{\"status\":\"valid\"}')"],
                            **kwargs,
                        )

                    with patch.object(pipeline.subprocess, "Popen", side_effect=fake_popen):
                        return original_step(name, command, cwd, evidence_dir, manifest_hash, events)
                log = evidence_dir / f"{len(events) + 1:02d}-{name}.log"
                log.write_text("synthetic success\n", encoding="utf-8")
                events.append({"step": name, "status": "passed", "exit_status": 0,
                               "command": command, "manifest_sha256": manifest_hash,
                               "log": {"path": log.name, "sha256": digest(log)}})
                return True

            with patch.object(pipeline, "run_step", side_effect=insert_report_symlink):
                with self.assertRaises(pipeline.PipelineError):
                    pipeline.run_pipeline(intake, root, "report-symlink")

            self.assertEqual(outside.read_text(encoding="utf-8"), "keep this file")

    def test_atomic_p3f_record_rewrite_replaces_symlink_without_following_it(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            outside = root / "outside-record.json"
            outside.write_text("keep this file", encoding="utf-8")
            record = root / "P3F-record.json"
            record.symlink_to(outside)

            pipeline.atomic_replace(record, b'{"exit_status":1}\n')

            self.assertEqual(outside.read_text(encoding="utf-8"), "keep this file")
            self.assertFalse(record.is_symlink())
            self.assertEqual(json.loads(record.read_text(encoding="utf-8"))["exit_status"], 1)

    def test_rejects_existing_destination_before_copy(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            intake, _ = self.fixture(root)
            dest = root.resolve() / "target/headstart/candidates/run-a"
            dest.mkdir()
            with self.assertRaisesRegex(pipeline.PipelineError, "already exists"):
                pipeline.run_pipeline(intake, root)

    def test_candidate_pin_failure_keeps_failure_evidence_in_pinned_parent(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            intake, _ = self.fixture(root)
            candidates = root / "target/headstart/candidates"
            original_open_child = pipeline.PinnedDir.open_child

            def fail_candidate_open(parent, relative):
                if str(relative) == "run-a":
                    raise pipeline.CandidateFSError("candidate pin unavailable")
                return original_open_child(parent, relative)

            with (
                patch.object(pipeline.PinnedDir, "open_child", side_effect=fail_candidate_open),
                patch.object(pipeline, "run_step") as run_step,
                self.assertRaisesRegex(pipeline.CandidateFSError, "candidate pin unavailable"),
            ):
                pipeline.run_pipeline(intake, root)

            run_step.assert_not_called()
            self.assertTrue((candidates / "run-a").is_dir())
            failure_path = candidates / "run-a.pipeline-failure.json"
            failure = json.loads(failure_path.read_text())
            self.assertEqual(failure["candidate_id"], "run-a")
            self.assertEqual(failure["status"], "failed")
            self.assertIn("candidate pin unavailable", failure["error"])

    def test_destination_race_does_not_write_into_other_run(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            intake, _ = self.fixture(root)
            dest = root.resolve() / "target/headstart/candidates/run-a"
            original_mkdir = pipeline.PinnedDir.mkdir

            def create_by_competing_run(parent, relative, *args, **kwargs) -> None:
                if relative == "run-a":
                    original_mkdir(parent, relative, *args, **kwargs)
                    raise pipeline.CandidateFSError(f"relative directory already exists: {relative}")
                original_mkdir(parent, relative, *args, **kwargs)

            with patch.object(pipeline.PinnedDir, "mkdir", new=create_by_competing_run):
                with self.assertRaisesRegex(pipeline.PipelineError, "already exists"):
                    pipeline.run_pipeline(intake, root)

            self.assertTrue(dest.is_dir())
            self.assertFalse((dest / "evidence" / "pipeline-failure.json").exists())

    def test_rejects_source_manifest_hash_drift(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            intake, source = self.fixture(root)
            (source / "manifest.json").write_text("{}\n")
            with self.assertRaisesRegex(pipeline.PipelineError, "hash drift"):
                pipeline.run_pipeline(intake, root)
            self.assertFalse((root / "target/headstart/candidates/run-a").exists())

    def test_hash_pins_the_exact_manifest_and_plan_bytes_parsed(self) -> None:
        for artifact in ("manifest.json", "patch-plan.json"):
            with self.subTest(artifact=artifact), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                intake_path, source = self.fixture(root)
                artifact_path = source / artifact
                changed = json.loads(artifact_path.read_text())
                if artifact == "manifest.json":
                    changed["repository"] = "https://changed.invalid/repository"
                    expected_error = "source manifest hash drift"
                else:
                    changed["source"]["revision"] = "f" * 40
                    expected_error = "P3E patch plan hash drift"

                original_load_json = pipeline.load_json

                def replace_before_read(path, *, root=None):
                    if path.resolve(strict=True) == artifact_path.resolve(strict=True):
                        artifact_path.write_text(json.dumps(changed))
                    return original_load_json(path, root=root)

                intake = json.loads(intake_path.read_text())
                with patch.object(pipeline, "load_json", side_effect=replace_before_read):
                    with self.assertRaisesRegex(pipeline.PipelineError, expected_error):
                        pipeline.validate_intake(intake, root, intake_path)

    def test_patch_plan_parent_manifest_hash_must_match_prepared_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            intake_path, source = self.fixture(root)
            plan_path = source / "patch-plan.json"
            plan = json.loads(plan_path.read_text())
            plan["candidate"]["parent_manifest_sha256"] = "f" * 64
            plan_path.write_text(json.dumps(plan))
            intake = json.loads(intake_path.read_text())
            intake["patch_plan"]["sha256"] = digest(plan_path)

            with self.assertRaisesRegex(
                pipeline.PipelineError,
                "P3E plan parent manifest SHA-256 does not match the prepared source manifest",
            ):
                pipeline.validate_intake(intake, root, intake_path)

    def test_patch_plan_rejects_missing_prepared_snapshot_hash(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            intake_path, source = self.fixture(root)
            manifest_path = source / "manifest.json"
            manifest = json.loads(manifest_path.read_text())
            manifest["snapshots"]["candidate"].pop("path_tree_sha256")
            manifest_path.write_text(json.dumps(manifest))

            plan_path = source / "patch-plan.json"
            plan = json.loads(plan_path.read_text())
            plan["candidate"].pop("snapshot_sha256")
            plan["candidate"]["parent_manifest_sha256"] = digest(manifest_path)
            plan_path.write_text(json.dumps(plan))

            intake = json.loads(intake_path.read_text())
            intake["source_manifest"]["sha256"] = digest(manifest_path)
            intake["patch_plan"]["sha256"] = digest(plan_path)
            self.assertRaisesRegex(
                pipeline.PipelineError,
                "P3E plan snapshot SHA-256 does not match the prepared candidate path tree",
                pipeline.validate_intake,
                intake,
                root,
                intake_path,
            )

    def test_patch_plan_snapshot_hash_must_match_prepared_candidate_tree(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            intake_path, source = self.fixture(root)
            plan_path = source / "patch-plan.json"
            plan = json.loads(plan_path.read_text())
            plan["candidate"]["snapshot_sha256"] = "f" * 64
            plan_path.write_text(json.dumps(plan))
            intake = json.loads(intake_path.read_text())
            intake["patch_plan"]["sha256"] = digest(plan_path)

            with self.assertRaisesRegex(
                pipeline.PipelineError,
                "P3E plan snapshot SHA-256 does not match the prepared candidate path tree",
            ):
                pipeline.validate_intake(intake, root, intake_path)

    def test_json_boolean_is_not_accepted_as_prepared_manifest_schema(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            intake, source = self.fixture(root)
            manifest_path = source / "manifest.json"
            manifest = json.loads(manifest_path.read_text())
            manifest["schema_version"] = True
            manifest_path.write_text(json.dumps(manifest))
            data = json.loads(intake.read_text())
            data["source_manifest"]["sha256"] = digest(manifest_path)
            intake.write_text(json.dumps(data))
            with self.assertRaisesRegex(pipeline.PipelineError, "unsupported prepared source manifest"):
                pipeline.run_pipeline(intake, root)
            self.assertFalse((root / "target/headstart/candidates/run-a").exists())

    def test_rejects_source_file_drift_against_manifest_inventory(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            intake, source = self.fixture(root)
            (source / "baseline/source/Cargo.lock").write_text("drift\n")
            with self.assertRaisesRegex(pipeline.PipelineError, "source closure hash drift"):
                pipeline.run_pipeline(intake, root)
            self.assertTrue((root / "target/headstart/candidates/run-a/evidence/pipeline-failure.json").exists())

    def test_rejects_symlink_in_source_closure(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            intake, source = self.fixture(root)
            link = source / "baseline/source/escape"
            link.symlink_to(root / "approved.json")
            with self.assertRaisesRegex(pipeline.PipelineError, "symlink rejected"):
                pipeline.run_pipeline(intake, root)
            self.assertTrue((root / "target/headstart/candidates/run-a/evidence/pipeline-failure.json").exists())

    def test_rejects_unapproved_intake(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            intake, _ = self.fixture(root)
            data = json.loads(intake.read_text())
            data["approved"] = False
            intake.write_text(json.dumps(data))
            with self.assertRaisesRegex(pipeline.PipelineError, "approved=true"):
                pipeline.run_pipeline(intake, root)

    def test_rejects_manifest_parent_swapped_to_symlink_after_containment_check(self) -> None:
        with tempfile.TemporaryDirectory() as tmp, tempfile.TemporaryDirectory() as external_tmp:
            root = Path(tmp)
            intake_path, source_root = self.fixture(root)
            external = Path(external_tmp) / "source-a"
            external.mkdir()
            shutil.copyfile(source_root / "manifest.json", external / "manifest.json")
            checked_resolve_inside = pipeline.resolve_inside

            def swap_after_check(path: Path, boundary: Path, label: str) -> Path:
                resolved = checked_resolve_inside(path, boundary, label)
                if label == "source manifest":
                    source_root.rename(root / "target/headstart/candidates/source-a-saved")
                    source_root.symlink_to(external, target_is_directory=True)
                return resolved

            intake = json.loads(intake_path.read_text())
            with patch.object(pipeline, "resolve_inside", side_effect=swap_after_check):
                with self.assertRaisesRegex(pipeline.PipelineError, "pinned root without symlinks"):
                    pipeline.validate_intake(intake, root, intake_path)

    def test_missing_plan_fails_closed_before_copying_or_checking_candidate_bytes(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            intake, _ = self.fixture(root)
            data = json.loads(intake.read_text())
            data["patch_plan"] = None
            intake.write_text(json.dumps(data))
            observed: list[str] = []

            def fake_step(name, command, cwd, evidence_dir, manifest_hash, events):
                observed.append(name)
                log = evidence_dir / f"{len(events) + 1:02d}-{name}.log"
                log.write_text("synthetic success\n")
                events.append({"step": name, "status": "passed", "exit_status": 0,
                               "command": command, "manifest_sha256": manifest_hash,
                               "log": {"path": log.name, "sha256": digest(log)}})
                return True

            with patch.object(pipeline, "run_step", side_effect=fake_step):
                with self.assertRaisesRegex(pipeline.PipelineError, "require a hash-pinned P3E"):
                    pipeline.run_pipeline(intake, root, "check-b")
            self.assertEqual(observed, [])
            candidate = root / "target/headstart/candidates/run-a"
            self.assertFalse((candidate / "baseline/source").exists())
            self.assertFalse((candidate / "candidate/source").exists())
            self.assertNotIn("P3K-exact-patch", observed)
            failure = json.loads((candidate / "evidence/pipeline-failure.json").read_text())
            self.assertEqual(failure["events"][0]["step"], "P3E-patch-plan")
            self.assertEqual(failure["events"][0]["status"], "not_run")

    def test_invalid_p3e_plan_fails_before_copying_prepared_source_bytes(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            intake, _ = self.fixture(root)
            data = json.loads(intake.read_text())
            plan_path = root / "target/headstart/candidates/source-a/patch-plan.json"
            plan = json.loads(plan_path.read_text())
            plan["source"]["revision"] = "invented"
            plan_path.write_text(json.dumps(plan))
            data["patch_plan"]["sha256"] = digest(plan_path)
            intake.write_text(json.dumps(data))

            with self.assertRaisesRegex(pipeline.PipelineError, "P3E patch plan validation failed"):
                pipeline.run_pipeline(intake, root, "check-invalid-plan")

            candidate = root / "target/headstart/candidates/run-a"
            self.assertFalse((candidate / "baseline/source").exists())
            self.assertFalse((candidate / "candidate/source").exists())
            failure = json.loads((candidate / "evidence/pipeline-failure.json").read_text())
            p3e = next(event for event in failure["events"] if event["step"] == "P3E-patch-plan")
            self.assertEqual(p3e["status"], "failed")
            self.assertTrue((candidate / "evidence" / p3e["log"]["path"]).is_file())

    def test_legacy_patch_hash_mismatch_fails_before_copying_prepared_sources(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            intake, source = self.fixture(root)
            manifest_path = source / "manifest.json"
            manifest = json.loads(manifest_path.read_text())
            manifest["patch_sha256"] = "0" * 64
            manifest_path.write_text(json.dumps(manifest))
            plan_path = source / "patch-plan.json"
            plan = json.loads(plan_path.read_text())
            plan["candidate"]["parent_manifest_sha256"] = digest(manifest_path)
            plan_path.write_text(json.dumps(plan))
            data = json.loads(intake.read_text())
            data["source_manifest"]["sha256"] = digest(manifest_path)
            data["patch_plan"]["sha256"] = digest(plan_path)
            intake.write_text(json.dumps(data))

            with self.assertRaisesRegex(pipeline.PipelineError, "do not match"):
                pipeline.run_pipeline(intake, root, "check-patch-binding")

            candidate = root / "target/headstart/candidates/run-a"
            self.assertFalse((candidate / "baseline/source").exists())
            failure = json.loads((candidate / "evidence/pipeline-failure.json").read_text())
            binding = next(event for event in failure["events"]
                           if event["step"] == "P3E-patch-binding")
            self.assertEqual(binding["status"], "failed")

    def test_patch_series_order_mismatch_fails_before_copying_prepared_sources(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            intake, source = self.fixture(root)
            second_patch = source / "second.patch"
            second_patch.write_text("second patch\n")
            manifest_path = source / "manifest.json"
            manifest = json.loads(manifest_path.read_text())
            manifest["schema_version"] = 2
            patch_records = [
                {"path": name, "sha256": digest(source / name),
                 "preimage_sha256": "1" * 64, "postimage_sha256": "2" * 64}
                for name in ("approved.patch", "second.patch")
            ]
            manifest["patches"] = patch_records
            manifest["patch_sha256"] = hashlib.sha256(
                "\n".join(f"{entry['path']}:{entry['sha256']}" for entry in patch_records)
                .encode("utf-8")
            ).hexdigest()
            manifest_path.write_text(json.dumps(manifest))

            data = json.loads(intake.read_text())
            data["source_manifest"]["sha256"] = digest(manifest_path)
            plan_path = source / "patch-plan.json"
            plan = json.loads(plan_path.read_text())
            plan["candidate"]["parent_manifest_sha256"] = digest(manifest_path)
            plan["patches"] = [
                {"path": entry["path"], "sha256": entry["sha256"]}
                for entry in reversed(patch_records)
            ]
            plan_path.write_text(json.dumps(plan))
            data["patch_plan"]["sha256"] = digest(plan_path)
            intake.write_text(json.dumps(data))

            with self.assertRaisesRegex(pipeline.PipelineError, "do not match"):
                pipeline.run_pipeline(intake, root, "check-patch-order")

            candidate = root / "target/headstart/candidates/run-a"
            self.assertFalse((candidate / "baseline/source").exists())
            failure = json.loads((candidate / "evidence/pipeline-failure.json").read_text())
            binding = next(event for event in failure["events"]
                           if event["step"] == "P3E-patch-binding")
            self.assertEqual(binding["status"], "failed")

    def test_failure_record_error_does_not_mask_pipeline_error(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            intake, _ = self.fixture(root)
            data = json.loads(intake.read_text())
            data["patch_plan"] = None
            intake.write_text(json.dumps(data))
            with patch.object(pipeline, "_write_failure", side_effect=OSError("record unavailable")):
                with self.assertRaisesRegex(pipeline.PipelineError, "require a hash-pinned P3E") as caught:
                    pipeline.run_pipeline(intake, root)
            self.assertIsInstance(caught.exception, pipeline.PipelineError)

    def test_keyboard_interrupt_leaves_failure_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            intake, _ = self.fixture(root)
            original_mkdir = pipeline.PinnedDir.mkdir

            def interrupt_evidence_creation(directory, relative, *args, **kwargs):
                if relative == "evidence":
                    raise KeyboardInterrupt("simulated user interrupt")
                return original_mkdir(directory, relative, *args, **kwargs)

            with patch.object(pipeline.PinnedDir, "mkdir", new=interrupt_evidence_creation):
                with self.assertRaisesRegex(KeyboardInterrupt, "simulated user interrupt"):
                    pipeline.run_pipeline(intake, root)

            failure_path = root / "target/headstart/candidates/run-a/pipeline-failure.json"
            failure = json.loads(failure_path.read_text())
            self.assertEqual(failure["status"], "failed")
            self.assertIn("KeyboardInterrupt", failure["error"])

    def test_patch_disappearing_after_validation_leaves_failure_record(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            intake, source = self.fixture(root)
            original_copy = pipeline._copy_regular_file

            def remove_patch_before_copy(src, dst, label="source snapshot file", **kwargs):
                if src.name == "approved.patch" and dst.parent.name == "run-a":
                    Path(src).unlink()
                return original_copy(src, dst, label, **kwargs)

            with patch.object(pipeline, "_copy_regular_file", side_effect=remove_patch_before_copy):
                with self.assertRaisesRegex(pipeline.PipelineError, "readable regular file"):
                    pipeline.run_pipeline(intake, root)
            failure_path = root / "target/headstart/candidates/run-a/evidence/pipeline-failure.json"
            failure = json.loads(failure_path.read_text())
            self.assertEqual(failure["status"], "failed")
            self.assertIn("approved.patch", failure["error"])

    def test_patch_replaced_with_symlink_before_copy_is_not_read(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            intake, source = self.fixture(root)
            outside = root / "outside.patch"
            outside.write_text("outside contents\n")
            original_copy = pipeline._copy_regular_file

            def replace_patch_then_copy(src, dst, label="source snapshot file", **kwargs):
                if src.name == "approved.patch" and dst.parent.name == "run-a":
                    src.unlink()
                    src.symlink_to(outside)
                return original_copy(src, dst, label, **kwargs)

            with patch.object(pipeline, "_copy_regular_file", side_effect=replace_patch_then_copy):
                with self.assertRaisesRegex(pipeline.PipelineError, "readable regular file"):
                    pipeline.run_pipeline(intake, root)

            candidate = root / "target/headstart/candidates/run-a"
            self.assertFalse((candidate / "approved.patch").exists())
            failure = json.loads((candidate / "evidence/pipeline-failure.json").read_text())
            self.assertEqual(failure["status"], "failed")
            self.assertIn("approved patch", failure["error"])


if __name__ == "__main__":
    unittest.main()
