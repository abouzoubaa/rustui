#!/usr/bin/env python3
"""Capture current RustUI build inputs alongside an existing build log.

This utility records metadata only. It does not parse the log as proof of a
successful build and does not run Cargo, rustc, Git mutations, or benchmarks.
"""

import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import sys


WORKSPACE_ROOT = Path(__file__).resolve().parent.parent
INPUT_FILES = (
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "scripts/cargo-headstart",
    ".cargo/config.toml",
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


def sha256_file(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def file_record(root, relative_path):
    path = root / relative_path
    record = {"path": relative_path}
    try:
        if not path.is_file():
            record["status"] = "missing"
        else:
            record["sha256"] = sha256_file(path)
            record["size_bytes"] = path.stat().st_size
            record["status"] = "captured"
    except OSError as error:
        record["status"] = "unreadable"
        record["error"] = str(error)
    return record


def source_records(root):
    records = []
    for source_root in SOURCE_ROOTS:
        base = root / source_root
        if not base.is_dir():
            records.append({"path": source_root, "status": "missing_directory"})
            continue

        def record_walk_error(error):
            try:
                relative = Path(error.filename).relative_to(root).as_posix()
            except (TypeError, ValueError):
                relative = source_root
            records.append({"path": relative, "status": "unreadable", "error": str(error)})

        for current, dirs, files in os.walk(base, followlinks=False, onerror=record_walk_error):
            dirs[:] = sorted(name for name in dirs if name not in IGNORED_DIRS)
            for name in sorted(files):
                path = Path(current) / name
                relative = path.relative_to(root).as_posix()
                record = {"path": relative}
                try:
                    if path.is_symlink():
                        record["status"] = "symlink_skipped"
                    else:
                        record["sha256"] = sha256_file(path)
                        record["size_bytes"] = path.stat().st_size
                        record["status"] = "captured"
                except OSError as error:
                    record["status"] = "unreadable"
                    record["error"] = str(error)
                records.append(record)
    return records


def git_commit(root):
    try:
        result = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=root,
            check=False,
            capture_output=True,
            text=True,
            timeout=5,
        )
    except (OSError, subprocess.TimeoutExpired):
        return None
    if result.returncode != 0:
        return None
    commit = result.stdout.strip()
    return commit or None


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
    root = args.workspace.expanduser().resolve()
    log = args.log.expanduser().resolve()
    if not root.is_dir():
        parser.error(f"workspace is not a readable directory: {root}")
    try:
        if not log.is_file():
            parser.error(f"build log does not exist or is not a regular file: {log}")
        log_hash = sha256_file(log)
        log_size = log.stat().st_size
    except OSError as error:
        parser.error(f"cannot read build log {log}: {error}")

    captured_at = datetime.now(timezone.utc).isoformat()
    exit_status = args.exit_status
    if exit_status is None:
        build_status = "unverified"
    else:
        build_status = "observed_success" if exit_status == 0 else "observed_failure"

    document = {
        "schema_version": 1,
        "recorded_at_utc": captured_at,
        "purpose": "build provenance capture; not a build, correctness, reuse, or performance verdict",
        "build_log": {
            "path": str(log),
            "sha256": log_hash,
            "size_bytes": log_size,
            "parsed_for_success": False,
        },
        "build_facts_declared_by_caller": {
            "command": args.command,
            "profile": args.profile,
            "features": args.features,
            "exit_status": exit_status,
            "status": build_status,
            "toolchain_strings": {
                "cargo": args.cargo_version,
                "rustc": args.rustc_version,
            },
            "note": "Caller supplied metadata only; no historical toolchain or command flags are inferred.",
        },
        "dependency_reuse_status": "not_evaluated",
        "captured_current_workspace": {
            "workspace_root": str(root),
            "git_commit": git_commit(root),
            "platform": {
                "sys_platform": sys.platform,
                "system": platform.system(),
                "release": platform.release(),
                "machine": platform.machine(),
            },
            "selected_environment": {
                key: os.environ[key] for key in SAFE_BUILD_ENV if key in os.environ
            },
            "input_file_hashes": [file_record(root, path) for path in INPUT_FILES],
            "workspace_crate_app_file_hashes": source_records(root),
        },
        "time_scope_note": (
            "File hashes, platform, Git commit, and selected environment are captured now. "
            "They are not asserted to match the conditions when the supplied log was produced."
        ),
    }

    destination = output_path(root, args.output)
    if destination.resolve() == log:
        parser.error("output path must not replace the supplied build log")
    try:
        destination.parent.mkdir(parents=True, exist_ok=True)
        mode = "w" if args.force and args.output is not None else "x"
        with destination.open(mode, encoding="utf-8") as stream:
            json.dump(document, stream, indent=2, sort_keys=True)
            stream.write("\n")
    except FileExistsError:
        parser.error(f"output already exists; choose another --output path: {destination}")
    except OSError as error:
        parser.error(f"cannot write output {destination}: {error}")

    print(destination)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
