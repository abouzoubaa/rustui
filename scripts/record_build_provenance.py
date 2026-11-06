#!/usr/bin/env python3
"""Capture current RustUI build inputs alongside an existing build log.

This utility records metadata only. It does not parse the log as proof of a
successful build or run Cargo, Git mutations, or benchmarks. It queries the
configured Headstart rustc identity for comparison with the selected binary.
"""

import argparse
import errno
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import sys

from calibrate_cache_workload import compiler_identity
try:
    from p3_candidate_fs import CandidateFSError, PinnedDir, relative_parts
except ModuleNotFoundError:
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    from p3_candidate_fs import CandidateFSError, PinnedDir, relative_parts


WORKSPACE_ROOT = Path(__file__).resolve().parent.parent
INPUT_FILES = (
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "scripts/cargo-headstart",
    ".cargo/config.toml",
    "themes/macos27.json",
    "crates/rustui/examples/cache_comparison.rs",
    "crates/rustui/src/view.rs",
    "crates/rustui/src/runtime/component_host.rs",
    "crates/rustui/src/runtime/render_diagnostics.rs",
    "scripts/compare_cache_workload.py",
    "scripts/collect_runtime_metrics.py",
    "scripts/runtime_metrics.py",
    "scripts/calibrate_cache_workload.py",
    "docs/tasks/P3_DIAGNOSTICS_OVERHEAD_CALIBRATION.md",
)
SOURCE_ROOTS = ("crates", "apps")
IGNORED_DIRS = {".git", "target", "vendor", "node_modules"}
SAFE_BUILD_ENV = (
    "CARGO_BUILD_TARGET",
    "CARGO_TARGET_DIR",
    "CARGO_INCREMENTAL",
    "CARGO_PROFILE_DEV_OPT_LEVEL",
    "CARGO_PROFILE_DEV_DEBUG",
    "CARGO_PROFILE_RELEASE_OPT_LEVEL",
    "CARGO_PROFILE_RELEASE_DEBUG",
    "RUSTFLAGS",
    "CARGO_ENCODED_RUSTFLAGS",
)


def _open_absolute_root(path):
    absolute = Path(os.path.abspath(path))
    current = PinnedDir.open_root(absolute.anchor)
    try:
        for part in absolute.parts[1:]:
            child = PinnedDir.open_child(current, part)
            current.close()
            current = child
        result = current
        current = None
        return result
    finally:
        if current is not None:
            current.close()

def _relative_path(root_path, requested):
    requested = Path(requested)
    lexical_root = Path(os.path.abspath(root_path))
    absolute = Path(os.path.abspath(requested if requested.is_absolute() else lexical_root / requested))
    try:
        relative = absolute.relative_to(lexical_root)
    except ValueError:
        canonical_root = lexical_root.resolve(strict=True)
        boundary = absolute.parent
        while True:
            try:
                if os.path.samefile(boundary, canonical_root):
                    relative = absolute.relative_to(boundary)
                    break
            except OSError:
                pass
            if boundary == boundary.parent:
                raise ValueError(f"path is outside root: {requested}")
            boundary = boundary.parent
    relative_parts(relative.as_posix())
    return relative.as_posix()

def sha256_file(path, root=None):
    owned = not isinstance(root, PinnedDir)
    path = Path(path)
    if owned:
        root = _open_absolute_root(path.parent.resolve(strict=True))
        relative = path.name
    else:
        relative = Path(path).as_posix()
    try:
        fd = root.open_file(relative, os.O_RDONLY | getattr(os, "O_NONBLOCK", 0))
        digest = hashlib.sha256()
        with os.fdopen(fd, "rb") as stream:
            for chunk in iter(lambda: stream.read(1024 * 1024), b""):
                digest.update(chunk)
        return digest.hexdigest()
    finally:
        if owned:
            root.close()



def file_record(root, relative_path):
    owned = not isinstance(root, PinnedDir)
    root_path = Path(root) if owned else None
    pinned = _open_absolute_root(root_path.resolve(strict=True)) if owned else root
    try:
        relative = _relative_path(root_path, relative_path) if owned else Path(relative_path).as_posix()
        relative_parts(relative)
    except (OSError, ValueError, CandidateFSError) as error:
        if owned:
            pinned.close()
        return {"path": str(relative_path), "status": "unreadable", "error": str(error)}
    record = {"path": relative}
    try:
        fd = pinned.open_file(relative, os.O_RDONLY | getattr(os, "O_NONBLOCK", 0))
        with os.fdopen(fd, "rb") as stream:
            before = os.fstat(stream.fileno())
            digest = hashlib.sha256()
            for chunk in iter(lambda: stream.read(1024 * 1024), b""):
                digest.update(chunk)
            after = os.fstat(stream.fileno())
            identity_before = (before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns, before.st_ctime_ns)
            identity_after = (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns, after.st_ctime_ns)
            if identity_before != identity_after:
                raise OSError("file changed while hashing")
            record.update({"sha256": digest.hexdigest(), "size_bytes": after.st_size, "status": "captured"})
    except (OSError, CandidateFSError) as error:
        cause = error
        while getattr(cause, "__cause__", None) is not None:
            cause = cause.__cause__
        if isinstance(cause, OSError) and cause.errno == errno.ENOENT:
            record["status"] = "missing"
            return record
        record.update({"status": "unreadable", "error": str(error)})
    finally:
        if owned:
            pinned.close()
    return record



def source_records(root):
    owned = not isinstance(root, PinnedDir)
    pinned = _open_absolute_root(Path(root).resolve(strict=True)) if owned else root
    records = []

    def visit(relative_dir):
        try:
            directory = PinnedDir.open_child(pinned, relative_dir) if relative_dir else PinnedDir.from_fd(pinned.fd)
        except (OSError, CandidateFSError) as error:
            records.append({"path": relative_dir, "status": "unreadable", "error": str(error)})
            return
        try:
            with os.scandir(directory.fd) as entries:
                names = sorted(entry.name for entry in entries)
            for name in names:
                if name in IGNORED_DIRS:
                    continue
                relative = f"{relative_dir}/{name}" if relative_dir else name
                try:
                    child = PinnedDir.open_child(pinned, relative)
                except (OSError, CandidateFSError):
                    records.append(file_record(pinned, relative))
                else:
                    child.close()
                    visit(relative)
        except OSError as error:
            records.append({"path": relative_dir, "status": "unreadable", "error": str(error)})
        finally:
            directory.close()

    try:
        for source_root in SOURCE_ROOTS:
            try:
                child = PinnedDir.open_child(pinned, source_root)
            except (OSError, CandidateFSError):
                continue
            child.close()
            visit(source_root)
        return records
    finally:
        if owned:
            pinned.close()



def git_commit(root):
    owned = not isinstance(root, PinnedDir)
    pinned = _open_absolute_root(Path(root).resolve(strict=True)) if owned else root
    try:
        result = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            preexec_fn=lambda: os.fchdir(pinned.fd),
            check=False,
            capture_output=True,
            text=True,
            timeout=5,
        )
    except (OSError, subprocess.TimeoutExpired):
        return None
    finally:
        if owned:
            pinned.close()
    if result.returncode != 0:
        return None
    commit = result.stdout.strip()
    return commit or None



def snapshot_digest(captured):
    payload = {
        "git_commit": captured["git_commit"],
        "input_file_hashes": captured["input_file_hashes"],
        "workspace_crate_app_file_hashes": captured["workspace_crate_app_file_hashes"],
    }
    encoded = json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def parse_exit_status(value):
    try:
        status = int(value)
    except ValueError as error:
        raise argparse.ArgumentTypeError("exit status must be an integer from 0 to 255") from error
    if not 0 <= status <= 255:
        raise argparse.ArgumentTypeError("exit status must be from 0 to 255")
    return status


def build_parser():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("log", type=Path, help="existing build log to identify by path and SHA-256")
    parser.add_argument("--workspace", type=Path, default=WORKSPACE_ROOT,
                        help="RustUI workspace root (default: script's workspace)")
    parser.add_argument("--output", type=Path,
                        help="output JSON path (default: unique file under target/headstart/evidence)")
    parser.add_argument("--force", action="store_true",
                        help="allow replacing an explicitly selected existing output file")
    parser.add_argument("--command", help="caller-declared build command string")
    parser.add_argument("--profile", help="caller-declared Cargo profile")
    parser.add_argument("--features", help="caller-declared Cargo feature selection")
    parser.add_argument("--exit-status", type=parse_exit_status,
                        help="caller-observed command exit status; not independently verified")
    parser.add_argument("--cargo-version", help="caller-supplied Cargo version string; no tool is queried")
    parser.add_argument("--rustc-version", help="caller-supplied rustc version string; no tool is queried")
    parser.add_argument("--binary", type=Path,
                        help="built executable to bind into the record (relative to workspace or absolute)")
    return parser


def output_path(root, requested):
    if requested is not None:
        return requested if requested.is_absolute() else Path.cwd() / requested
    directory = root / "target/headstart/evidence"
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S.%fZ")
    base = f"build-provenance-{stamp}-{os.getpid()}"
    candidate = directory / f"{base}.json"
    suffix = 1
    while candidate.exists():
        candidate = directory / f"{base}-{suffix}.json"
        suffix += 1
    return candidate


def main():
    parser = build_parser()
    args = parser.parse_args()
    inherited_repo_fd = os.environ.get("RUSTUI_P3_REPO_ROOT_FD")
    inherited_parent_fd = os.environ.get("RUSTUI_P3_CANDIDATE_PARENT_FD")
    inherited_candidate_fd = os.environ.get("RUSTUI_P3_CANDIDATE_ROOT_FD")
    candidate_id = os.environ.get("RUSTUI_P3_CANDIDATE_ID")
    inherited = any(value is not None for value in
                     (inherited_repo_fd, inherited_parent_fd, inherited_candidate_fd, candidate_id))
    if inherited and not all((inherited_repo_fd, inherited_parent_fd, inherited_candidate_fd, candidate_id)):
        parser.error("incomplete inherited P3 root identity")

    if inherited:
        repo_root = PinnedDir.from_fd(int(inherited_repo_fd))
        candidate_parent = PinnedDir.from_fd(int(inherited_parent_fd))
        try:
            identity_root = PinnedDir.open_child(candidate_parent, candidate_id)
        finally:
            candidate_parent.close()
        candidate_root = PinnedDir.from_fd(int(inherited_candidate_fd))
        if candidate_root.identity() != identity_root.identity():
            identity_root.close()
            candidate_root.close()
            repo_root.close()
            parser.error("inherited candidate directory does not match candidate parent and ID")
        identity_root.close()
        workspace_arg = args.workspace.expanduser()
        if workspace_arg == WORKSPACE_ROOT:
            workspace_relative = "candidate/source"
        elif workspace_arg.is_absolute():
            parser.error("inherited workspace must be relative to the candidate root")
        else:
            workspace_relative = workspace_arg.as_posix()
            try:
                relative_parts(workspace_relative)
            except CandidateFSError as error:
                parser.error(str(error))
        try:
            workspace_root = PinnedDir.open_child(candidate_root, workspace_relative)
        except CandidateFSError as error:
            parser.error(f"cannot open workspace beneath candidate root: {error}")
        workspace_label = f"target/headstart/candidates/{candidate_id}/{workspace_relative}"
        log_relative = args.log.expanduser().as_posix()
        binary_relative = args.binary.as_posix() if args.binary is not None else None
        requested_output = args.output.expanduser() if args.output is not None else None
        if requested_output is not None and requested_output.is_absolute():
            parser.error("inherited output path must be relative to the candidate root")
        output_relative = requested_output.as_posix() if requested_output is not None else None
        log_fs = candidate_root
    else:
        workspace_path = args.workspace.expanduser().resolve(strict=True)
        repo_root = _open_absolute_root(workspace_path)
        candidate_root = PinnedDir.from_fd(repo_root.fd)
        workspace_root = repo_root
        workspace_label = str(workspace_path)
        log_path = args.log.expanduser()
        try:
            log_relative = _relative_path(workspace_path, log_path)
        except (OSError, ValueError) as error:
            parser.error(f"build log must be beneath workspace root: {error}")
        binary_relative = None
        if args.binary is not None:
            try:
                binary_relative = _relative_path(workspace_path, args.binary.expanduser())
            except (OSError, ValueError) as error:
                parser.error(f"binary must be beneath workspace root: {error}")
        requested_output = args.output.expanduser() if args.output is not None else None
        output_relative = None
        if requested_output is not None:
            try:
                output_relative = _relative_path(workspace_path, requested_output)
            except (OSError, ValueError) as error:
                parser.error(f"output must be beneath workspace root: {error}")
        log_fs = workspace_root

    try:
        log_info = None
        log_fd = log_fs.open_file(log_relative, os.O_RDONLY | getattr(os, "O_NONBLOCK", 0))
        with os.fdopen(log_fd, "rb") as stream:
            before = os.fstat(stream.fileno())
            log_info = before
            digest = hashlib.sha256()
            for chunk in iter(lambda: stream.read(1024 * 1024), b""):
                digest.update(chunk)
            after = os.fstat(stream.fileno())
            if (before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns, before.st_ctime_ns) != (
                after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns, after.st_ctime_ns
            ):
                parser.error("build log changed while hashing")
            log_hash, log_size = digest.hexdigest(), after.st_size
    except (OSError, CandidateFSError) as error:
        parser.error(f"cannot read build log beneath pinned root: {error}")

    captured_at = datetime.now(timezone.utc).isoformat()
    exit_status = args.exit_status
    build_status = "unverified" if exit_status is None else ("observed_success" if exit_status == 0 else "observed_failure")
    binary_record = file_record(candidate_root if inherited else workspace_root, binary_relative) if binary_relative else None
    document = {
        "schema_version": 1,
        "recorded_at_utc": captured_at,
        "purpose": "build provenance capture; not build, correctness, reuse, or performance verdict",
        "build_log": {"path": str(args.log), "sha256": log_hash, "size_bytes": log_size, "parsed_for_success": False},
        "selected_binary": binary_record,
        "compiler_identity": compiler_identity() if binary_relative else None,
        "build_facts_declared_by_caller": {
            "command": args.command,
            "profile": args.profile,
            "features": args.features,
            "exit_status": exit_status,
            "status": build_status,
            "toolchain_strings": {"cargo": args.cargo_version, "rustc": args.rustc_version},
            "note": "Caller supplied metadata only; no historical toolchain command flags are inferred.",
        },
        "dependency_reuse_status": "not_evaluated",
        "captured_current_workspace": {
            "workspace_root": workspace_label,
            "git_commit": git_commit(workspace_root),
            "platform": {"sys_platform": sys.platform, "system": platform.system(), "release": platform.release(), "machine": platform.machine()},
            "selected_environment": {key: os.environ[key] for key in SAFE_BUILD_ENV if key in os.environ},
            "input_file_hashes": [file_record(workspace_root, name) for name in INPUT_FILES],
            "workspace_crate_app_file_hashes": source_records(workspace_root),
        },
    }
    captured = document["captured_current_workspace"]
    captured["source_snapshot_sha256"] = snapshot_digest(captured)

    # The output must not replace a file whose bytes this record claims to
    # capture. This also covers currently missing declared inputs, which would
    # otherwise be created after their "missing" status was recorded.
    workspace_prefix = f"{workspace_relative}/" if inherited else ""
    captured_paths = {
        workspace_prefix + entry["path"]
        for entry in (*captured["input_file_hashes"],
                      *captured["workspace_crate_app_file_hashes"])
        if isinstance(entry, dict) and isinstance(entry.get("path"), str)
    }
    if binary_relative is not None:
        captured_paths.add(binary_relative)
    captured_paths.add(log_relative)

    if output_relative is not None and output_relative in captured_paths:
        parser.error("output path must not replace a file captured by build provenance")
    if output_relative is None:
        output_dir = "target/headstart/evidence"
        try:
            evidence_dir = PinnedDir.create_child(candidate_root, output_dir)
            evidence_dir.close()
        except CandidateFSError as error:
            parser.error(f"cannot create evidence output directory: {error}")
        stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S.%fZ")
        output_relative = f"{output_dir}/build-provenance-{stamp}-{os.getpid()}.json"
        output_display = str(Path(workspace_label) / output_relative)
    else:
        parent = Path(output_relative).parent.as_posix()
        if parent != ".":
            try:
                created = PinnedDir.create_child(candidate_root, parent)
                created.close()
            except CandidateFSError as error:
                parser.error(f"cannot create output parent: {error}")
        output_display = str(args.output)
    if output_relative in captured_paths:
        parser.error("output path must not replace a file captured by build provenance")
    try:
        if args.force:
            try:
                existing_fd = candidate_root.open_file(
                    output_relative, os.O_RDONLY | getattr(os, "O_NONBLOCK", 0)
                )
            except (FileNotFoundError, CandidateFSError) as error:
                cause = error
                while getattr(cause, "__cause__", None) is not None:
                    cause = cause.__cause__
                if not isinstance(cause, FileNotFoundError):
                    raise
            else:
                try:
                    existing_info = os.fstat(existing_fd)
                finally:
                    os.close(existing_fd)
                if log_info is not None and (existing_info.st_dev, existing_info.st_ino) == (
                    log_info.st_dev, log_info.st_ino
                ):
                    parser.error("output path must not replace the supplied build log (hard-link alias)")
        candidate_root.atomic_write(
            output_relative, (json.dumps(document, indent=2) + "\n").encode("utf-8"),
            replace=args.force,
        )
    except FileExistsError:
        parser.error(f"output already exists: {output_display} (pass --force to replace)")
    except (OSError, CandidateFSError) as error:
        parser.error(f"cannot write provenance output beneath pinned candidate root: {error}")
    print(output_display)
    if workspace_root is not repo_root:
        workspace_root.close()
    candidate_root.close()
    repo_root.close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
