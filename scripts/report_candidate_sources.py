#!/usr/bin/env python3
"""Report effective Cargo package sources for one prepared local candidate."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import queue
import re
import shutil
import signal
import stat
import subprocess
import sys
import tempfile
import threading
import time
from pathlib import Path
from typing import Any

from p3_candidate_fs import CandidateFSError, PinnedDir, relative_parts


SCHEMA_VERSION = 2
IDENTIFIER_RE = re.compile(r"[A-Za-z0-9][A-Za-z0-9._-]{0,63}\Z")
SCOPE = (
    "locked offline Cargo metadata with candidate-local package paths and absolute-path-free resolved dependency identities, edges, and enabled features; "
    "no provenance approval patch acceptance build test GUI performance or promotion"
)
MAX_CARGO_MANIFEST_BYTES = 32 * 1024 * 1024
MAX_CARGO_MANIFEST_ENTRIES = 100_000
MAX_CARGO_SUPPORT_FILE_BYTES = MAX_CARGO_MANIFEST_BYTES
MAX_CARGO_METADATA_OUTPUT_BYTES = 32 * 1024 * 1024
MAX_REPORT_OUTPUT_BYTES = MAX_CARGO_METADATA_OUTPUT_BYTES


class SourceReportError(ValueError):
    """Candidate inputs or Cargo metadata do not satisfy the P3H boundary."""


def _unique_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise SourceReportError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def _reject_constant(value: str) -> Any:
    raise SourceReportError(f"nonstandard JSON constant: {value}")


def _inside(path: Path, root: Path) -> bool:
    try:
        path.relative_to(root)
        return True
    except ValueError:
        return False


def _check_no_symlink(path: Path, boundary: Path) -> None:
    """Reject a symlink at any existing path component from boundary to path."""
    try:
        boundary = boundary.resolve(strict=True)
        resolved_path = path.resolve(strict=False)
        if not _inside(resolved_path, boundary):
            raise SourceReportError(f"path escapes candidate root: {path}")

        # Permit symlinked system prefixes such as macOS /var -> /private/var,
        # but reject links whose directory entry is inside the candidate.
        current = Path(path.absolute().anchor)
        for part in path.absolute().parts[1:]:
            current = current / part
            if current.is_symlink() and _inside(current.parent.resolve(strict=True), boundary):
                raise SourceReportError(f"symlink rejected in source path: {current}")
            if current.is_file() and current.stat().st_nlink > 1:
                raise SourceReportError(f"hard-linked source file rejected: {current}")
    except SourceReportError:
        raise
    except (OSError, RuntimeError) as error:
        raise SourceReportError(f"cannot inspect source path {path}: {error}") from error


def validate_metadata(
    metadata: Any,
    candidate_root: Path,
    workspace_root: Path,
    request: dict[str, Any],
    *,
    pinned_workspace: PinnedDir | None = None,
    pinned_workspace_relative: str | None = None,
    candidate_id: str | None = None,
) -> dict[str, Any]:
    """Validate Cargo's resolved package paths and return a normalized report."""
    errors: list[str] = []
    workspace_relative = (pinned_workspace_relative if pinned_workspace is not None else
                          workspace_root.relative_to(candidate_root).as_posix()
                          if _inside(workspace_root, candidate_root) else None)
    if workspace_relative is None:
        errors.append("selected workspace is outside the candidate root")
    report: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "status": "invalid",
        "scope": SCOPE,
        "candidate_id": candidate_id or candidate_root.name,
        "workspace_root": workspace_relative,
        "request": request,
        "package_count": 0,
        "source_counts": {},
        "resolve_graph_sha256": None,
        "packages": [],
        "errors": errors,
    }
    if not isinstance(metadata, dict):
        errors.append("Cargo metadata must be a JSON object")
        return report

    packages = metadata.get("packages")
    resolution = metadata.get("resolve")
    if not isinstance(packages, list):
        errors.append("Cargo metadata packages must be a nonempty array")
        packages = []
    elif not packages:
        errors.append("Cargo metadata packages must be a nonempty array")
    if not isinstance(resolution, dict) or not isinstance(resolution.get("nodes"), list):
        errors.append("Cargo metadata must include a resolved dependency graph")
        return report
    metadata_workspace = metadata.get("workspace_root")
    if pinned_workspace is None:
        if metadata_workspace != str(workspace_root):
            errors.append("Cargo metadata workspace_root does not match the selected workspace")
    elif not isinstance(metadata_workspace, str) or not Path(metadata_workspace).is_absolute():
        errors.append("Cargo metadata workspace_root must be absolute")

    nodes = resolution["nodes"]
    package_by_id: dict[str, dict[str, Any]] = {}
    for index, package in enumerate(packages):
        if not isinstance(package, dict):
            errors.append(f"packages[{index}] must be an object")
            continue
        package_id = package.get("id")
        if not isinstance(package_id, str) or not package_id:
            errors.append(f"packages[{index}].id must be a nonempty string")
            continue
        if package_id in package_by_id:
            errors.append(f"duplicate Cargo package id: {package_id}")
        package_by_id[package_id] = package

    node_by_id: dict[str, dict[str, Any]] = {}
    for index, node in enumerate(nodes):
        if not isinstance(node, dict) or not isinstance(node.get("id"), str):
            errors.append(f"resolve.nodes[{index}] must have a string id")
            continue
        node_id = node["id"]
        if node_id in node_by_id:
            errors.append(f"duplicate resolved package id: {node_id}")
        node_by_id[node_id] = node
    if set(package_by_id) != set(node_by_id):
        errors.append("Cargo metadata packages do not match the resolved dependency graph")

    summaries: list[dict[str, Any]] = []
    package_identity_by_id: dict[str, dict[str, str]] = {}
    identity_keys: set[tuple[str, ...]] = set()
    counts: dict[str, int] = {}
    vendor_root = (workspace_root / "vendor").resolve(strict=False)
    for package_id, package in package_by_id.items():
        name, version = package.get("name"), package.get("version")
        manifest_value = package.get("manifest_path")
        source = package.get("source")
        if (not isinstance(name, str) or not name.strip()
                or not isinstance(version, str) or not version.strip()):
            errors.append(f"package {package_id} must have nonempty string name and version")
            continue
        if source is not None and not isinstance(source, str):
            errors.append(f"package {package_id} has an unsupported Cargo source value")
            continue
        if not isinstance(manifest_value, str) or not manifest_value:
            errors.append(f"package {package_id} must have a manifest_path")
            continue
        manifest_lexical = Path(manifest_value)
        if not manifest_lexical.is_absolute():
            errors.append(f"package {package_id} manifest_path must be absolute")
            continue
        try:
            if pinned_workspace is None:
                _check_no_symlink(manifest_lexical, candidate_root)
                manifest = manifest_lexical.resolve(strict=True)
                _check_no_symlink(manifest, candidate_root)
                if not manifest.is_file() or manifest.name != "Cargo.toml":
                    raise SourceReportError(f"package manifest is not a regular Cargo.toml: {manifest}")
                relative = manifest.relative_to(candidate_root).as_posix()
                try:
                    manifest_workspace_relative = manifest.relative_to(workspace_root)
                except ValueError:
                    manifest_workspace_relative = Path("..")
            else:
                assert isinstance(metadata_workspace, str) and pinned_workspace_relative is not None
                manifest_workspace_relative = manifest_lexical.relative_to(Path(metadata_workspace))
                manifest_parts = relative_parts(manifest_workspace_relative.as_posix())
                if manifest_parts[-1] != "Cargo.toml":
                    raise SourceReportError("package manifest is not a regular Cargo.toml")
                descriptor = pinned_workspace.open_file(
                    manifest_workspace_relative.as_posix(), os.O_RDONLY | getattr(os, "O_NONBLOCK", 0)
                )
                try:
                    info = os.fstat(descriptor)
                    if info.st_nlink != 1:
                        raise SourceReportError("package manifest is hard-linked")
                finally:
                    os.close(descriptor)
                relative = (Path(pinned_workspace_relative) / manifest_workspace_relative).as_posix()
                manifest = workspace_root / manifest_workspace_relative
        except (OSError, RuntimeError, ValueError) as error:
            errors.append(f"package {package_id} source cannot be validated: {error}")
            continue

        if source is None:
            if pinned_workspace is None and not _inside(manifest, workspace_root):
                errors.append(f"path package {package_id} is outside the candidate workspace")
            source_kind = "path"
        elif isinstance(source, str) and source.startswith("registry+"):
            if (not _inside(manifest, vendor_root) if pinned_workspace is None else
                    not manifest_workspace_relative.parts
                    or manifest_workspace_relative.parts[0] != "vendor"):
                errors.append(f"registry package {package_id} is outside candidate-local vendor/")
            source_kind = "registry-vendor"
        elif isinstance(source, str) and source.startswith("git+"):
            errors.append(f"git package {package_id} is not an approved candidate-local source")
            source_kind = "git-rejected"
        else:
            errors.append(f"package {package_id} has an unsupported Cargo source")
            source_kind = "unsupported"
        counts[source_kind] = counts.get(source_kind, 0) + 1
        workspace_path = (manifest_workspace_relative.parent.as_posix()
                          if source_kind == "path" and _inside(manifest, workspace_root) else "")
        identity = {
            "name": name,
            "version": version,
            "source_kind": source_kind,
            "source": "path" if source_kind == "path" else source,
            "workspace_path": workspace_path,
        }
        identity_key = (name, version, source_kind, identity["source"], workspace_path)
        if identity_key in identity_keys:
            errors.append(f"duplicate normalized Cargo package identity: {identity_key}")
        identity_keys.add(identity_key)
        package_identity_by_id[package_id] = identity
        summaries.append({
            "id": package_id,
            "name": name,
            "version": version,
            "source": source if isinstance(source, str) else "path",
            "source_kind": source_kind,
            "manifest_path": relative,
            "identity": identity,
        })

    normalized_nodes: list[dict[str, Any]] = []
    for node_id, node in node_by_id.items():
        if node_id not in package_identity_by_id:
            continue
        dependencies = node.get("deps")
        if not isinstance(dependencies, list):
            errors.append(f"resolved package {node_id} deps must be an array")
            continue
        normalized_dependencies: list[dict[str, Any]] = []
        for dep in dependencies:
            if (not isinstance(dep, dict) or not isinstance(dep.get("pkg"), str)
                    or not isinstance(dep.get("name"), str) or not dep["name"].strip()
                    or not isinstance(dep.get("dep_kinds"), list) or not dep["dep_kinds"]):
                errors.append(f"resolved package {node_id} has a malformed dependency")
                continue
            dep_id = dep["pkg"]
            if dep_id not in package_by_id:
                errors.append(f"resolved package {node_id} references unknown package {dep_id}")
                continue
            kinds: list[dict[str, str | None]] = []
            for dep_kind in dep["dep_kinds"]:
                if not isinstance(dep_kind, dict):
                    errors.append(f"resolved package {node_id} has malformed dependency kinds")
                    continue
                kind, target = dep_kind.get("kind"), dep_kind.get("target")
                if (kind is not None and kind not in ("dev", "build")) or (
                    target is not None and (not isinstance(target, str) or not target.strip())
                ):
                    errors.append(f"resolved package {node_id} has malformed dependency kind fields")
                    continue
                kinds.append({"kind": kind, "target": target})
            normalized_dependencies.append({
                "name": dep["name"],
                "package": package_identity_by_id.get(dep_id),
                "kinds": sorted(kinds, key=lambda item: json.dumps(item, sort_keys=True)),
            })
        features = node.get("features")
        if (not isinstance(features, list)
                or any(not isinstance(feature, str) or not feature.strip() for feature in features)
                or len(set(features)) != len(features)):
            errors.append(f"resolved package {node_id} features must be an array of nonempty strings")
            features = []
        normalized_nodes.append({
            "package": package_identity_by_id[node_id],
            "dependencies": sorted(normalized_dependencies, key=lambda item: json.dumps(item, sort_keys=True)),
            "features": sorted(set(features)),
        })
    normalized_nodes.sort(key=lambda item: json.dumps(item["package"], sort_keys=True))
    graph_bytes = json.dumps(normalized_nodes, sort_keys=True, separators=(",", ":")).encode()
    report["resolve_graph_sha256"] = hashlib.sha256(graph_bytes).hexdigest()
    report["resolved_graph"] = {"schema_version": 1, "nodes": normalized_nodes}
    report["package_count"] = len(summaries)
    report["source_counts"] = dict(sorted(counts.items()))
    report["packages"] = sorted(summaries, key=lambda item: item["id"])
    report["status"] = "valid" if not errors else "invalid"
    return report


def _candidate_paths(root: Path, workspace_arg: Path | None) -> tuple[Path, Path]:
    candidate_base = (Path(__file__).resolve().parents[1] / "target/headstart/candidates").resolve(strict=True)
    candidate_lexical = root.absolute()
    if candidate_lexical.name in ("", ".", "..") or IDENTIFIER_RE.fullmatch(candidate_lexical.name) is None or ".." in candidate_lexical.name:
        raise SourceReportError("candidate root name must be a safe 1-64 character identifier")
    _check_no_symlink(candidate_lexical, candidate_base)
    candidate = candidate_lexical.resolve(strict=True)
    if not _inside(candidate, candidate_base) or candidate.parent != candidate_base:
        raise SourceReportError("candidate root must be one existing directory beneath target/headstart/candidates")
    if workspace_arg is None:
        candidates = [candidate / "candidate/source", candidate / "source"]
        workspace = next((item for item in candidates if (item / "Cargo.toml").is_file()), None)
        if workspace is None:
            raise SourceReportError("cannot find candidate/source or source; provide --workspace")
    else:
        workspace = workspace_arg if workspace_arg.is_absolute() else candidate / workspace_arg
    workspace_lexical = workspace.absolute()
    _check_no_symlink(workspace_lexical, candidate)
    workspace = workspace_lexical.resolve(strict=True)
    _check_no_symlink(workspace, candidate)
    if not workspace.is_dir() or not (workspace / "Cargo.toml").is_file() or not (workspace / "Cargo.lock").is_file():
        raise SourceReportError("workspace must contain regular Cargo.toml and Cargo.lock files")
    _check_no_symlink(workspace / "Cargo.toml", candidate)
    _check_no_symlink(workspace / "Cargo.lock", candidate)
    return candidate, workspace


def _metadata_command(
    cargo: str,
    workspace: Path,
    cargo_configs: list[str | Path],
    isolated_target_dir: Path,
    features: list[str],
    all_features: bool,
    no_default_features: bool,
    filter_platform: str | None,
) -> list[str]:
    command = [cargo, "metadata", "--format-version", "1", "--locked", "--offline"]
    for cargo_config in cargo_configs:
        command.extend(("--config", str(cargo_config)))
    command.extend(("--config", f"build.target-dir={json.dumps(str(isolated_target_dir))}"))
    command.extend(("--manifest-path", str(workspace / "Cargo.toml")))
    for feature in features:
        command.extend(("--features", feature))
    if all_features:
        command.append("--all-features")
    if no_default_features:
        command.append("--no-default-features")
    if filter_platform:
        command.extend(("--filter-platform", filter_platform))
    return command


def _run_capped_metadata(
    command: list[str],
    *,
    cwd: Path | None = None,
    env: dict[str, str] | None = None,
    pass_fds: tuple[int, ...] = (),
) -> subprocess.CompletedProcess[str]:
    """Drain Cargo metadata output concurrently while retaining at most 32 MiB."""
    process = subprocess.Popen(
        command, cwd=cwd, env=env, stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE, stderr=subprocess.PIPE, pass_fds=pass_fds,
        start_new_session=True,
    )
    assert process.stdout is not None and process.stderr is not None
    chunks: queue.Queue[tuple[str, bytes | None]] = queue.Queue(maxsize=4)

    def drain(name: str, stream: Any) -> None:
        try:
            while chunk := stream.read1(64 * 1024):
                chunks.put((name, chunk))
        finally:
            chunks.put((name, None))

    readers = [
        threading.Thread(target=drain, args=(name, stream), daemon=True)
        for name, stream in (("stdout", process.stdout), ("stderr", process.stderr))
    ]
    for reader in readers:
        reader.start()

    output = {"stdout": bytearray(), "stderr": bytearray()}
    captured = 0
    finished = 0
    overflow = False
    terminated = False
    termination_time = 0.0
    killed = False

    def terminate() -> None:
        nonlocal terminated, termination_time
        try:
            os.killpg(process.pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
        terminated = True
        termination_time = time.monotonic()

    def escalate() -> None:
        nonlocal killed
        if overflow and terminated and not killed and time.monotonic() - termination_time >= 1:
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            if process.poll() is None:
                process.kill()
            killed = True

    while finished < len(readers):
        escalate()
        try:
            name, chunk = chunks.get(timeout=0.05)
        except queue.Empty:
            if overflow and not terminated:
                terminate()
            escalate()
            continue
        if chunk is None:
            finished += 1
            continue
        remaining = MAX_CARGO_METADATA_OUTPUT_BYTES - captured
        output[name].extend(chunk[:remaining])
        captured += min(len(chunk), remaining)
        if len(chunk) > remaining:
            overflow = True
            if not terminated:
                terminate()
        escalate()

    if overflow:
        escalate()
        try:
            returncode = process.wait(timeout=1)
        except subprocess.TimeoutExpired:
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            if process.poll() is None:
                process.kill()
            returncode = process.wait()
    else:
        returncode = process.wait()
    for reader in readers:
        reader.join()
    process.stdout.close()
    process.stderr.close()

    if overflow:
        raise SourceReportError(
            f"Cargo metadata output exceeded the combined {MAX_CARGO_METADATA_OUTPUT_BYTES}-byte limit"
        )
    decode = lambda value: value.decode("utf-8").replace("\r\n", "\n").replace("\r", "\n")
    result = subprocess.CompletedProcess(
        command, returncode, stdout=decode(output["stdout"]),
        stderr=output["stderr"].decode("utf-8", errors="replace").replace("\r\n", "\n").replace("\r", "\n"),
    )
    return result


def _emit_report(report: dict[str, Any]) -> bool:
    """Emit deterministic JSON within the P3H report-output limit."""
    serialized = json.dumps(report, indent=2, sort_keys=True)
    if len(serialized.encode("utf-8")) + 1 > MAX_REPORT_OUTPUT_BYTES:
        candidate_id = report.get("candidate_id")
        if not isinstance(candidate_id, str) or len(candidate_id) > 64:
            candidate_id = None
        report = {
            "schema_version": SCHEMA_VERSION,
            "status": "invalid",
            "scope": SCOPE,
            "candidate_id": candidate_id,
            "workspace_root": None,
            "request": None,
            "package_count": 0,
            "source_counts": {},
            "resolve_graph_sha256": None,
            "resolved_graph": None,
            "packages": [],
            "errors": [
                f"candidate source report exceeded the {MAX_REPORT_OUTPUT_BYTES}-byte output limit"
            ],
        }
        serialized = json.dumps(report, indent=2, sort_keys=True)
    sys.stdout.write(serialized + "\n")
    return report["status"] == "valid"


def _candidate_cargo_configs(workspace: Path, candidate: Path) -> list[Path]:
    """Return candidate-local Cargo configs in Cargo's low-to-high precedence order."""
    configs: list[Path] = []
    current = workspace
    while _inside(current, candidate):
        config_dir = current / ".cargo"
        for name in ("config.toml", "config"):
            config = config_dir / name
            if config.exists() or config.is_symlink():
                _check_no_symlink(config, candidate)
                if not config.is_file():
                    raise SourceReportError("candidate .cargo config must be a regular file")
                configs.append(config)
                break
        if current == candidate:
            break
        current = current.parent
    # Cargo applies more specific config after ancestor config.
    return list(reversed(configs))


def _file_version(info: os.stat_result) -> tuple[int, int, int, int, int]:
    return (info.st_dev, info.st_ino, info.st_size, info.st_mtime_ns, info.st_ctime_ns)


def _cargo_manifest_hashes(
    workspace: Path, candidate: Path,
) -> dict[tuple[str, str], tuple[str, tuple[int, int, int, int, int]] | tuple[int, int, int, int, int]]:
    """Capture Cargo manifests and directory versions so transient edits cannot pass rechecks."""
    records: dict[
        tuple[str, str],
        tuple[str, tuple[int, int, int, int, int]] | tuple[int, int, int, int, int],
    ] = {}

    entry_count = 0
    pending = [workspace]
    while pending:
        parent = pending.pop()
        before = parent.stat()
        entries: list[tuple[str, bool]] = []
        try:
            with os.scandir(parent) as scan:
                for entry in scan:
                    entry_count += 1
                    if entry_count > MAX_CARGO_MANIFEST_ENTRIES:
                        raise SourceReportError(
                            f"candidate Cargo workspace exceeds {MAX_CARGO_MANIFEST_ENTRIES} entries"
                        )
                    entries.append((entry.name, entry.is_dir(follow_symlinks=False)))
        except OSError as error:
            raise SourceReportError(
                f"cannot enumerate candidate Cargo manifests: {error}"
            ) from error
        entries.sort(key=lambda item: item[0])
        relative_directory = parent.relative_to(workspace).as_posix() or "."
        records[("directory", relative_directory)] = _file_version(before)
        directories = [name for name, is_directory in entries if is_directory]
        for name, is_directory in entries:
            if is_directory or name != "Cargo.toml":
                continue
            manifest = parent / name
            _check_no_symlink(manifest, candidate)
            descriptor = os.open(
                manifest, os.O_RDONLY | getattr(os, "O_NONBLOCK", 0)
                | getattr(os, "O_NOFOLLOW", 0),
            )
            try:
                before_file = os.fstat(descriptor)
                if not stat.S_ISREG(before_file.st_mode) or before_file.st_nlink != 1:
                    raise SourceReportError(f"candidate Cargo manifest is not a regular single-link file: {manifest}")
                if before_file.st_size > MAX_CARGO_MANIFEST_BYTES:
                    raise SourceReportError(
                        f"candidate Cargo manifest exceeds {MAX_CARGO_MANIFEST_BYTES} bytes: {manifest}"
                    )
                with os.fdopen(descriptor, "rb", closefd=False) as source:
                    contents = source.read(MAX_CARGO_MANIFEST_BYTES + 1)
                after_file = os.fstat(descriptor)
                try:
                    path_file = os.stat(manifest, follow_symlinks=False)
                except OSError as error:
                    raise SourceReportError(f"candidate Cargo manifest changed during metadata inspection: {manifest}") from error
                file_version = _file_version(after_file)
                if (_file_version(before_file) != file_version
                        or _file_version(path_file) != file_version):
                    raise SourceReportError(f"candidate Cargo manifest changed during metadata inspection: {manifest}")
                if len(contents) > MAX_CARGO_MANIFEST_BYTES:
                    raise SourceReportError(
                        f"candidate Cargo manifest exceeds {MAX_CARGO_MANIFEST_BYTES} bytes: {manifest}"
                    )
                records[("manifest", manifest.relative_to(workspace).as_posix())] = (
                    hashlib.sha256(contents).hexdigest(), file_version,
                )
            finally:
                os.close(descriptor)
        after = parent.stat()
        if _file_version(before) != _file_version(after):
            raise SourceReportError("candidate Cargo directory changed during metadata inspection")
        pending.extend(parent / name for name in reversed(directories))
    return records


def _pinned_file_bytes(root: PinnedDir, relative: str, *, limit: int | None = None) -> bytes:
    descriptor = root.open_file(relative, os.O_RDONLY | getattr(os, "O_NONBLOCK", 0))
    try:
        if os.fstat(descriptor).st_nlink != 1:
            raise SourceReportError(f"candidate file is hard-linked: {relative}")
        with os.fdopen(descriptor, "rb", closefd=False) as stream:
            contents = stream.read(None if limit is None else limit + 1)
        if limit is not None and len(contents) > limit:
            raise SourceReportError(f"candidate file exceeds size limit: {relative}")
        return contents
    finally:
        os.close(descriptor)


def _candidate_file_bytes(path: Path, relative: str) -> bytes:
    """Read a candidate Cargo support file without an unbounded allocation."""
    with path.open("rb") as stream:
        contents = stream.read(MAX_CARGO_SUPPORT_FILE_BYTES + 1)
    if len(contents) > MAX_CARGO_SUPPORT_FILE_BYTES:
        raise SourceReportError(
            f"candidate Cargo file exceeds {MAX_CARGO_SUPPORT_FILE_BYTES} bytes: {relative}"
        )
    return contents


def _pinned_optional_file(
    root: PinnedDir, relative: str, *, limit: int | None = None,
) -> bytes | None:
    """Read an optional regular file without treating symlinks as absence."""
    parts = relative_parts(relative)
    current = os.dup(root.fd)
    try:
        for part in parts[:-1]:
            try:
                child = os.open(
                    part,
                    os.O_RDONLY | getattr(os, "O_DIRECTORY", 0)
                    | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_CLOEXEC", 0),
                    dir_fd=current,
                )
            except FileNotFoundError:
                return None
            os.close(current)
            current = child
        try:
            descriptor = os.open(
                parts[-1],
                os.O_RDONLY | getattr(os, "O_NONBLOCK", 0)
                | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_CLOEXEC", 0),
                dir_fd=current,
            )
        except FileNotFoundError:
            return None
        try:
            info = os.fstat(descriptor)
            if not stat.S_ISREG(info.st_mode) or info.st_nlink != 1:
                raise SourceReportError(f"candidate config is not a regular single-link file: {relative}")
            with os.fdopen(descriptor, "rb", closefd=False) as stream:
                contents = stream.read(None if limit is None else limit + 1)
            if limit is not None and len(contents) > limit:
                raise SourceReportError(f"candidate Cargo file exceeds {limit} bytes: {relative}")
            return contents
        finally:
            os.close(descriptor)
    except OSError as error:
        raise SourceReportError(f"cannot read candidate file safely: {relative}: {error}") from error
    finally:
        os.close(current)


def _pinned_manifest_hashes(
    workspace: PinnedDir,
) -> dict[tuple[str, str], tuple[str, tuple[int, int, int, int, int]] | tuple[int, int, int, int, int]]:
    records: dict[
        tuple[str, str],
        tuple[str, tuple[int, int, int, int, int]] | tuple[int, int, int, int, int],
    ] = {}
    entry_count = 0
    directory_flags = (os.O_RDONLY | getattr(os, "O_DIRECTORY", 0)
                       | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_CLOEXEC", 0))
    file_flags = (os.O_RDONLY | getattr(os, "O_NONBLOCK", 0)
                  | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_CLOEXEC", 0))

    def identity(info: os.stat_result) -> tuple[int, int]:
        return info.st_dev, info.st_ino

    def begin_scan(fd: int, name: str | None,
                   parent_identity: tuple[int, int] | None,
                   expected: os.stat_result | None = None) -> dict[str, Any]:
        nonlocal entry_count
        before = os.fstat(fd)
        if not stat.S_ISDIR(before.st_mode):
            raise SourceReportError("candidate Cargo path is not a directory")
        if expected is not None and identity(before) != identity(expected):
            raise SourceReportError(f"candidate Cargo directory changed during metadata inspection: {name}")
        names = []
        try:
            with os.scandir(fd) as scan:
                for entry in scan:
                    entry_count += 1
                    if entry_count > MAX_CARGO_MANIFEST_ENTRIES:
                        raise SourceReportError(
                            f"candidate Cargo workspace exceeds {MAX_CARGO_MANIFEST_ENTRIES} entries"
                        )
                    names.append(entry.name)
            names.sort()
        except OSError as error:
            raise SourceReportError(f"cannot enumerate candidate workspace: {error}") from error
        return {
            "name": name,
            "identity": identity(before),
            "before": _file_version(before),
            "parent_identity": parent_identity,
            "names": names,
            "index": 0,
        }

    current_fd = os.dup(workspace.fd)
    stack: list[dict[str, Any]] = []
    try:
        stack.append(begin_scan(current_fd, None, None))
        while stack:
            frame = stack[-1]
            if frame["index"] == len(frame["names"]):
                after = os.fstat(current_fd)
                if _file_version(after) != frame["before"]:
                    raise SourceReportError(
                        "candidate Cargo directory changed during metadata inspection"
                    )
                directory_relative = "/".join(item["name"] for item in stack[1:]) or "."
                records[("directory", directory_relative)] = _file_version(after)
                if len(stack) == 1:
                    break

                child_name = frame["name"]
                parent_fd = os.open("..", directory_flags, dir_fd=current_fd)
                try:
                    parent_info = os.fstat(parent_fd)
                    if identity(parent_info) != frame["parent_identity"]:
                        raise SourceReportError(
                            "candidate Cargo directory parent changed during metadata inspection"
                        )
                    named_child = os.stat(child_name, dir_fd=parent_fd, follow_symlinks=False)
                    if (not stat.S_ISDIR(named_child.st_mode)
                            or identity(named_child) != frame["identity"]):
                        raise SourceReportError(
                            f"candidate Cargo directory changed during metadata inspection: {child_name}"
                        )
                except BaseException:
                    os.close(parent_fd)
                    raise
                os.close(current_fd)
                current_fd = parent_fd
                stack.pop()
                continue

            name = frame["names"][frame["index"]]
            frame["index"] += 1
            info = os.stat(name, dir_fd=current_fd, follow_symlinks=False)
            if stat.S_ISDIR(info.st_mode):
                try:
                    child_fd = os.open(name, directory_flags, dir_fd=current_fd)
                except OSError as error:
                    raise SourceReportError(
                        f"cannot open candidate Cargo directory safely: {name}"
                    ) from error
                try:
                    child_frame = begin_scan(
                        child_fd, name, frame["identity"], expected=info
                    )
                except BaseException:
                    os.close(child_fd)
                    raise
                os.close(current_fd)
                current_fd = child_fd
                stack.append(child_frame)
            elif name == "Cargo.toml":
                relative = "/".join((*[item["name"] for item in stack[1:]], name))
                try:
                    descriptor = os.open(name, file_flags, dir_fd=current_fd)
                except OSError as error:
                    raise SourceReportError(
                        f"cannot open candidate Cargo manifest safely: {relative}"
                    ) from error
                try:
                    before_file = os.fstat(descriptor)
                    if (not stat.S_ISREG(before_file.st_mode) or before_file.st_nlink != 1
                            or identity(before_file) != identity(info)):
                        raise SourceReportError(
                            f"candidate Cargo manifest is not a regular single-link file: {relative}"
                        )
                    if before_file.st_size > MAX_CARGO_MANIFEST_BYTES:
                        raise SourceReportError(
                            f"candidate Cargo manifest exceeds {MAX_CARGO_MANIFEST_BYTES} bytes: {relative}"
                        )
                    with os.fdopen(descriptor, "rb", closefd=False) as source:
                        contents = source.read(MAX_CARGO_MANIFEST_BYTES + 1)
                    after_file = os.fstat(descriptor)
                    try:
                        current_file = os.stat(name, dir_fd=current_fd, follow_symlinks=False)
                    except OSError as error:
                        raise SourceReportError(
                            f"candidate Cargo manifest changed during metadata inspection: {relative}"
                        ) from error
                    file_version = _file_version(after_file)
                    if (_file_version(before_file) != file_version
                            or _file_version(current_file) != file_version):
                        raise SourceReportError(
                            f"candidate Cargo manifest changed during metadata inspection: {relative}"
                        )
                    if len(contents) > MAX_CARGO_MANIFEST_BYTES:
                        raise SourceReportError(
                            f"candidate Cargo manifest exceeds {MAX_CARGO_MANIFEST_BYTES} bytes: {relative}"
                        )
                    records[("manifest", relative)] = (
                        hashlib.sha256(contents).hexdigest(), file_version,
                    )
                finally:
                    os.close(descriptor)
    finally:
        os.close(current_fd)
    return records


def _pinned_cargo_configs(candidate: PinnedDir, workspace_relative: str) -> list[tuple[str, bytes]]:
    """Find candidate Cargo configs from candidate root through workspace."""
    parts = relative_parts(workspace_relative)
    ancestor_paths = ["/".join(parts[:index]) for index in range(len(parts), -1, -1)]
    configs: list[tuple[str, bytes]] = []
    for ancestor in reversed(ancestor_paths):
        config_dir = f"{ancestor}/.cargo" if ancestor else ".cargo"
        for name in ("config.toml", "config"):
            relative = f"{config_dir}/{name}"
            contents = _pinned_optional_file(
                candidate, relative, limit=MAX_CARGO_SUPPORT_FILE_BYTES
            )
            if contents is not None:
                configs.append((relative, contents))
                break
    return configs


def _pinned_workspace(candidate: PinnedDir, workspace_arg: Path | None) -> tuple[str, PinnedDir]:
    if workspace_arg is None:
        for relative in ("candidate/source", "source"):
            try:
                workspace = PinnedDir.open_child(candidate, relative)
            except CandidateFSError:
                continue
            try:
                _pinned_file_bytes(
                    workspace, "Cargo.toml", limit=MAX_CARGO_MANIFEST_BYTES
                )
                _pinned_file_bytes(
                    workspace, "Cargo.lock", limit=MAX_CARGO_SUPPORT_FILE_BYTES
                )
                return relative, workspace
            except SourceReportError:
                workspace.close()
                raise
            except (CandidateFSError, OSError):
                workspace.close()
                continue
        raise SourceReportError("cannot find candidate/source source; provide --workspace")
    relative = workspace_arg.as_posix()
    relative_parts(relative)
    workspace = PinnedDir.open_child(candidate, relative)
    try:
        _pinned_file_bytes(
            workspace, "Cargo.toml", limit=MAX_CARGO_MANIFEST_BYTES
        )
        _pinned_file_bytes(
            workspace, "Cargo.lock", limit=MAX_CARGO_SUPPORT_FILE_BYTES
        )
    except Exception:
        workspace.close()
        raise
    return relative, workspace


def _validate_pinned_roots(args: argparse.Namespace) -> tuple[PinnedDir, PinnedDir, PinnedDir, str]:
    try:
        repo_fd = int(os.environ["RUSTUI_P3_REPO_ROOT_FD"])
        candidate_parent_fd = int(os.environ["RUSTUI_P3_CANDIDATE_PARENT_FD"])
    except (KeyError, ValueError) as error:
        raise SourceReportError("incomplete inherited P3H root descriptors") from error
    repo = PinnedDir.from_fd(repo_fd)
    candidate_parent = PinnedDir.from_fd(candidate_parent_fd)
    try:
        expected_parent = PinnedDir.open_child(repo, "target/headstart/candidates")
        try:
            if expected_parent.identity() != candidate_parent.identity():
                raise SourceReportError("inherited candidate parent does not match repository root")
        finally:
            expected_parent.close()
        relative = args.candidate_root.as_posix()
        parts = relative_parts(relative)
        if len(parts) != 4 or parts[:3] != ("target", "headstart", "candidates"):
            raise SourceReportError("candidate root must be target/headstart/candidates/<id>")
        candidate_id = parts[3]
        if IDENTIFIER_RE.fullmatch(candidate_id) is None:
            raise SourceReportError("candidate root name must be a safe 1-64 character identifier")
        declared_id = os.environ.get("RUSTUI_P3_CANDIDATE_ID")
        if declared_id is not None and declared_id != candidate_id:
            raise SourceReportError("candidate ID does not match inherited candidate root")
        candidate = PinnedDir.open_child(candidate_parent, candidate_id)
        return repo, candidate_parent, candidate, candidate_id
    except Exception:
        repo.close()
        candidate_parent.close()
        raise


def _pinned_report(args: argparse.Namespace, request: dict[str, Any]) -> dict[str, Any]:
    repo, candidate_parent, candidate, candidate_id = _validate_pinned_roots(args)
    try:
        workspace_relative, workspace = _pinned_workspace(candidate, args.workspace)
        try:
            cargo = shutil.which(args.cargo)
            if cargo is None:
                raise SourceReportError(f"Cargo executable not found: {args.cargo}")
            manifest_hashes_before = _pinned_manifest_hashes(workspace)
            lock_before = hashlib.sha256(_pinned_file_bytes(
                workspace, "Cargo.lock", limit=MAX_CARGO_SUPPORT_FILE_BYTES
            )).hexdigest()
            configs = _pinned_cargo_configs(candidate, workspace_relative)
            config_relative = [relative for relative, _ in configs]
            config_sha256 = [hashlib.sha256(contents).hexdigest() for _, contents in configs]
            config_args = [os.path.relpath(relative, workspace_relative) for relative in config_relative]
            with tempfile.TemporaryDirectory(prefix="rustui-p3h-cargo-") as temporary:
                isolated = Path(temporary)
                command = _metadata_command(
                    cargo, Path("."), config_args, isolated,
                    args.features, args.all_features, args.no_default_features,
                    args.filter_platform,
                )
                environment = {
                    key: value for key, value in os.environ.items()
                    if key.startswith("CARGO_")
                }
                environment.update({
                    "CARGO_HOME": str(isolated / "cargo-home"),
                    "CARGO_TARGET_DIR": str(isolated / "target"),
                    "CARGO_NET_OFFLINE": "true",
                })
                bootstrap = (
                    "import os, sys; fd = int(sys.argv[1]); argv = sys.argv[2:]; "
                    "os.fchdir(fd); os.execvpe(argv[0], argv, os.environ)"
                )
                result = _run_capped_metadata(
                    [sys.executable, "-c", bootstrap, str(workspace.fd), *command],
                    env=environment, pass_fds=(workspace.fd,),
                )
            manifest_hashes_after = _pinned_manifest_hashes(workspace)
            if manifest_hashes_after != manifest_hashes_before:
                raise SourceReportError("candidate Cargo manifests changed during metadata inspection")
            lock_after = hashlib.sha256(_pinned_file_bytes(
                workspace, "Cargo.lock", limit=MAX_CARGO_SUPPORT_FILE_BYTES
            )).hexdigest()
            if lock_before != lock_after:
                raise SourceReportError("Cargo.lock changed during locked metadata inspection")
            for (relative, _), expected_hash in zip(configs, config_sha256):
                actual_hash = hashlib.sha256(_pinned_file_bytes(
                    candidate, relative, limit=MAX_CARGO_SUPPORT_FILE_BYTES
                )).hexdigest()
                if actual_hash != expected_hash:
                    raise SourceReportError(
                        f"candidate Cargo config changed during locked metadata inspection: {relative}"
                    )
            if result.returncode != 0:
                raise SourceReportError(
                    f"Cargo metadata exited {result.returncode}: {result.stderr.strip()[-3000:]}"
                )
            metadata = json.loads(
                result.stdout, object_pairs_hook=_unique_object, parse_constant=_reject_constant
            )
            candidate_logical = Path(candidate_id)
            workspace_logical = candidate_logical / workspace_relative
            report = validate_metadata(
                metadata, candidate_logical, workspace_logical, request,
                pinned_workspace=workspace,
                pinned_workspace_relative=workspace_relative,
                candidate_id=candidate_id,
            )
            report["cargo_executable"] = cargo
            report["cargo_command"] = command
            report["cargo_lock_sha256"] = lock_before
            report["cargo_config"] = config_relative[-1] if config_relative else None
            report["cargo_config_sha256"] = config_sha256[-1] if config_sha256 else None
            report["cargo_config_chain"] = config_relative
            report["cargo_config_sha256_chain"] = config_sha256
            report["cargo_working_directory"] = workspace_relative
            report["cargo_home_isolated"] = True
            report["cargo_target_dir_isolated"] = True
            report["cargo_target_dir_override"] = "temporary directory forced by Cargo --config"
            return report
        finally:
            workspace.close()
    finally:
        candidate.close()
        candidate_parent.close()
        repo.close()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("candidate_root", type=Path, help="existing candidate directory under target/headstart/candidates")
    parser.add_argument("--workspace", type=Path, help="workspace path relative to candidate_root (default: candidate/source or source)")
    parser.add_argument("--cargo", default="cargo", help="Cargo executable (default: cargo from PATH)")
    parser.add_argument("--features", action="append", default=[], help="Cargo feature selection; may be repeated")
    parser.add_argument("--all-features", action="store_true")
    parser.add_argument("--no-default-features", action="store_true")
    parser.add_argument("--filter-platform", help="Cargo target triple for metadata filtering")
    args = parser.parse_args()

    request = {
        "mode": "locked-offline",
        "features": args.features,
        "all_features": args.all_features,
        "no_default_features": args.no_default_features,
        "filter_platform": args.filter_platform,
    }
    if ("RUSTUI_P3_REPO_ROOT_FD" in os.environ
            or "RUSTUI_P3_CANDIDATE_PARENT_FD" in os.environ):
        try:
            report = _pinned_report(args, request)
        except (SourceReportError, CandidateFSError, OSError, UnicodeError,
                json.JSONDecodeError, subprocess.SubprocessError, RuntimeError, ValueError) as error:
            report = {
                "schema_version": SCHEMA_VERSION,
                "status": "invalid",
                "scope": SCOPE,
                "candidate_id": args.candidate_root.name,
                "workspace_root": None,
                "request": request,
                "package_count": 0,
                "source_counts": {},
                "resolve_graph_sha256": None,
                "resolved_graph": None,
                "packages": [],
                "errors": [f"cannot report candidate Cargo sources: {error}"],
                "cargo_config": None,
                "cargo_config_sha256": None,
                "cargo_config_chain": [],
                "cargo_config_sha256_chain": [],
                "cargo_working_directory": None,
                "cargo_home_isolated": True,
                "cargo_target_dir_isolated": True,
                "cargo_target_dir_override": None,
            }
        return 0 if _emit_report(report) else 1
    report: dict[str, Any]
    try:
        candidate, workspace = _candidate_paths(args.candidate_root, args.workspace)
        cargo = shutil.which(args.cargo)
        if cargo is None:
            raise SourceReportError(f"Cargo executable not found: {args.cargo}")
        lock_path = workspace / "Cargo.lock"
        manifest_hashes_before = _cargo_manifest_hashes(workspace, candidate)
        lock_before = hashlib.sha256(
            _candidate_file_bytes(lock_path, "Cargo.lock")
        ).hexdigest()
        cargo_configs = _candidate_cargo_configs(workspace, candidate)
        config_relative = [config.relative_to(candidate).as_posix() for config in cargo_configs]
        config_sha256 = [hashlib.sha256(
            _candidate_file_bytes(config, config.relative_to(candidate).as_posix())
        ).hexdigest() for config in cargo_configs]
        with tempfile.TemporaryDirectory(prefix="rustui-p3h-cargo-") as temporary:
            isolated = Path(temporary)
            isolated_target_dir = isolated / "target"
            command = _metadata_command(cargo, workspace, cargo_configs, isolated_target_dir,
                                       args.features, args.all_features,
                                       args.no_default_features, args.filter_platform)
            environment = {key: value for key, value in os.environ.items() if not key.startswith("CARGO_")}
            environment.update({
                "CARGO_HOME": str(isolated / "cargo-home"),
                "CARGO_TARGET_DIR": str(isolated_target_dir),
                "CARGO_NET_OFFLINE": "true",
            })
            # Run outside the repository tree so ancestor .cargo/config.toml files
            # cannot silently replace the candidate's source or build settings.
            result = _run_capped_metadata(command, cwd=isolated, env=environment)
        manifest_hashes_after = _cargo_manifest_hashes(workspace, candidate)
        if manifest_hashes_after != manifest_hashes_before:
            raise SourceReportError("candidate Cargo manifests changed during metadata inspection")
        lock_after = hashlib.sha256(
            _candidate_file_bytes(lock_path, "Cargo.lock")
        ).hexdigest()
        if lock_before != lock_after:
            raise SourceReportError("Cargo.lock changed during locked metadata inspection")
        for config, expected_hash in zip(cargo_configs, config_sha256):
            _check_no_symlink(config, candidate)
            actual_hash = hashlib.sha256(
                _candidate_file_bytes(config, config.relative_to(candidate).as_posix())
            ).hexdigest()
            if actual_hash != expected_hash:
                raise SourceReportError(
                    f"candidate Cargo config changed during locked metadata inspection: {config}"
                )
        if result.returncode != 0:
            raise SourceReportError(f"Cargo metadata exited {result.returncode}: {result.stderr.strip()[-3000:]}")
        metadata = json.loads(result.stdout, object_pairs_hook=_unique_object, parse_constant=_reject_constant)
        report = validate_metadata(metadata, candidate, workspace, request)
        report["cargo_executable"] = cargo
        report["cargo_command"] = command
        report["cargo_lock_sha256"] = lock_before
        report["cargo_config"] = config_relative[-1] if config_relative else None
        report["cargo_config_sha256"] = config_sha256[-1] if config_sha256 else None
        report["cargo_config_chain"] = config_relative
        report["cargo_config_sha256_chain"] = config_sha256
        report["cargo_working_directory"] = "isolated temporary directory"
        report["cargo_home_isolated"] = True
        report["cargo_target_dir_isolated"] = True
        report["cargo_target_dir_override"] = "temporary directory forced by Cargo --config"
    except (SourceReportError, OSError, UnicodeError, json.JSONDecodeError,
            subprocess.SubprocessError, RuntimeError, ValueError) as error:
        report = {
            "schema_version": SCHEMA_VERSION,
            "status": "invalid",
            "scope": SCOPE,
            "candidate_id": args.candidate_root.name,
            "workspace_root": None,
            "request": request,
            "package_count": 0,
            "source_counts": {},
            "resolve_graph_sha256": None,
            "resolved_graph": None,
            "packages": [],
            "errors": [f"cannot report candidate Cargo sources: {error}"],
            "cargo_config": None,
            "cargo_config_sha256": None,
            "cargo_config_chain": [],
            "cargo_config_sha256_chain": [],
            "cargo_working_directory": None,
            "cargo_home_isolated": True,
            "cargo_target_dir_isolated": True,
            "cargo_target_dir_override": None,
        }
    return 0 if _emit_report(report) else 1


if __name__ == "__main__":
    raise SystemExit(main())
