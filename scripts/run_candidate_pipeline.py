#!/usr/bin/env python3
"""Run the bounded, local-only P3 validation chain for an approved snapshot."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import selectors
import signal
import stat
import subprocess
import sys
import time
import uuid
from datetime import datetime, timezone
from contextlib import ExitStack
from contextvars import ContextVar
from pathlib import Path, PurePosixPath, PureWindowsPath
from typing import Any

import validate_candidate_snapshot as candidate_snapshot
from p3_candidate_fs import CandidateFSError, PinnedDir


ID_RE = re.compile(r"[A-Za-z0-9][A-Za-z0-9._-]{0,63}\Z")
HASH_RE = re.compile(r"[0-9a-f]{64}\Z")
GATES = {"build": "not_run", "correctness": "not_run", "gui": "not_run",
         "performance": "not_run", "promotion": "not_run"}


STEP_TIMEOUT_SECONDS = 600
# Leave room for the bounded diagnostic appended to oversized command logs.
MAX_STEP_OUTPUT_BYTES = 64 * 1024 * 1024 - 256
MAX_JSON_BYTES = 32 * 1024 * 1024
MAX_PATCH_BYTES = 32 * 1024 * 1024

_RUNPY_BOOTSTRAP = """import os, runpy, sys
os.fchdir(int(os.environ['RUSTUI_P3_REPO_ROOT_FD']))
script = sys.argv[1]
sys.path.insert(0, os.path.dirname(script))
sys.argv = [script, *sys.argv[2:]]
runpy.run_path(script, run_name='__main__')
"""
_EXEC_BOOTSTRAP = """import os, sys
os.fchdir(int(os.environ['RUSTUI_P3_CANDIDATE_ROOT_FD']))
os.execvpe(sys.argv[1], sys.argv[1:], os.environ)
"""
_ACTIVE_FD_CONTEXT: ContextVar[dict[str, Any] | None] = ContextVar(
    "candidate_pipeline_fd_context", default=None
)


def _fd_path(fd: int) -> Path:
    """Return a path rooted at an open directory descriptor."""
    descriptor_path = Path("/dev/fd") / str(fd)
    if not descriptor_path.exists():
        raise PipelineError("the platform does not expose open descriptors through /dev/fd")
    return descriptor_path


def _child_argument(value: str, context: dict[str, Any]) -> str:
    candidate_path: Path = context["candidate_path"]
    candidate_relative: str = context["candidate_relative"]
    source_path: Path = context["source_path"]
    source_relative: str = context["source_relative"]
    root: Path = context["root"]
    path = Path(value)
    try:
        suffix = path.relative_to(candidate_path)
    except ValueError:
        pass
    else:
        return (Path(candidate_relative) / suffix).as_posix()
    try:
        suffix = path.relative_to(source_path)
    except ValueError:
        pass
    else:
        return (Path(source_relative) / suffix).as_posix()
    if path.is_absolute():
        try:
            relative = path.relative_to(root)
        except ValueError:
            return value
        return relative.as_posix() or "."
    return value


def _fd_anchored_command(command: list[str], context: dict[str, Any]) -> list[str]:
    if len(command) >= 2 and command[0] == sys.executable and command[1].endswith(".py"):
        script = Path(command[1])
        if script.is_absolute():
            script = script.relative_to(context["root"])
        args = [_child_argument(value, context) for value in command[2:]]
        return [sys.executable, "-c", _RUNPY_BOOTSTRAP, script.as_posix(), *args]
    return [sys.executable, "-c", _EXEC_BOOTSTRAP, *command]


class PipelineError(ValueError):
    pass


def sha256(path: Path, *, root: Path | None = None) -> str:
    digest = hashlib.sha256()
    with _open_regular(path, "hash input", root=root) as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def _open_regular(path: Path, label: str, *, root: Path | None = None):
    """Open a regular input, optionally walking it beneath a pinned root FD."""
    context = _ACTIVE_FD_CONTEXT.get()
    if context is not None:
        candidate_path: Path = context["candidate_path"]
        source_path: Path = context["source_path"]
        for anchor, descriptor in ((candidate_path, context["candidate_dir"]),
                                   (source_path, context["source_dir"])):
            try:
                relative = Path(os.path.abspath(path)).relative_to(anchor)
            except ValueError:
                continue
            if not relative.parts:
                raise PipelineError(f"{label} must name a file beneath its pinned root: {path}")
            try:
                return os.fdopen(descriptor.open_file(relative.as_posix(), os.O_RDONLY
                                                      | getattr(os, "O_NONBLOCK", 0)), "rb")
            except (OSError, CandidateFSError) as error:
                raise PipelineError(f"{label} must be a readable regular file beneath its pinned root: {path}") from error
    flags = os.O_RDONLY | getattr(os, "O_NONBLOCK", 0) | getattr(os, "O_NOFOLLOW", 0)
    if root is not None:
        root = root.resolve(strict=True)
        try:
            relative = Path(os.path.abspath(path)).relative_to(root)
        except ValueError as error:
            raise PipelineError(f"{label} is outside its pinned root: {path}") from error
        if not relative.parts:
            raise PipelineError(f"{label} must name a file beneath its pinned root: {path}")
        directory_flags = flags | getattr(os, "O_DIRECTORY", 0)
        current_fd: int | None = None
        descriptor: int | None = None
        try:
            current_fd = os.open(root.anchor, directory_flags)
            for part in (*root.parts[1:], *relative.parts[:-1]):
                child_fd = os.open(part, directory_flags, dir_fd=current_fd)
                os.close(current_fd)
                current_fd = child_fd
                if not stat.S_ISDIR(os.fstat(current_fd).st_mode):
                    raise PipelineError(f"{label} has a non-directory path component: {part}")
            descriptor = os.open(relative.parts[-1], flags, dir_fd=current_fd)
            if not stat.S_ISREG(os.fstat(descriptor).st_mode):
                raise PipelineError(f"{label} must be a regular file: {path}")
            stream = os.fdopen(descriptor, "rb")
            descriptor = None
            return stream
        except OSError as error:
            raise PipelineError(
                f"{label} must be a readable regular file beneath its pinned root without symlinks: {path}"
            ) from error
        finally:
            if descriptor is not None:
                os.close(descriptor)
            if current_fd is not None:
                os.close(current_fd)
    try:
        descriptor = os.open(path, flags)
    except OSError as error:
        raise PipelineError(f"{label} must be a readable regular file: {path}") from error
    try:
        if not stat.S_ISREG(os.fstat(descriptor).st_mode):
            raise PipelineError(f"{label} must be a regular file: {path}")
        return os.fdopen(descriptor, "rb")
    except Exception:
        os.close(descriptor)
        raise


def unique_json(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise PipelineError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def safe_relative(value: Any, label: str) -> Path:
    if not isinstance(value, str) or not value or "\x00" in value or "\\" in value:
        raise PipelineError(f"{label} must be a slash-separated relative path")
    posix, windows = PurePosixPath(value), PureWindowsPath(value)
    if posix.is_absolute() or windows.is_absolute() or windows.drive or any(
        part in ("", ".", "..") for part in value.split("/")
    ):
        raise PipelineError(f"{label} must be a safe relative path")
    return Path(*value.split("/"))


def resolve_inside(path: Path, root: Path, label: str) -> Path:
    root = root.resolve(strict=True)
    lexical = Path(os.path.abspath(path))
    try:
        relative = lexical.relative_to(root)
    except ValueError:
        # macOS may spell the same system path through /var and /private/var.
        try:
            resolved = lexical.resolve(strict=True)
            relative = resolved.relative_to(root)
        except (OSError, RuntimeError, ValueError) as error:
            raise PipelineError(f"{label} is outside repository root") from error
        current = root
        for part in relative.parts:
            current /= part
            if current.is_symlink():
                raise PipelineError(f"symlink rejected in {label}: {current}")
        return resolved
    current = root
    for part in relative.parts:
        current /= part
        if current.is_symlink():
            raise PipelineError(f"symlink rejected in {label}: {current}")
    try:
        resolved = lexical.resolve(strict=True)
        resolved.relative_to(root)
    except (OSError, RuntimeError, ValueError) as error:
        raise PipelineError(f"{label} escapes repository root or is missing") from error
    return resolved


def inventory(root: Path) -> dict[str, str]:
    try:
        context = _ACTIVE_FD_CONTEXT.get()
        if context is not None:
            candidate_path: Path = context["candidate_path"]
            source_path: Path = context["source_path"]
            for anchor, descriptor in ((candidate_path, context["candidate_dir"]),
                                       (source_path, context["source_dir"])):
                try:
                    relative = Path(os.path.abspath(root)).relative_to(anchor)
                except ValueError:
                    continue
                return descriptor.inventory(
                    relative.as_posix() if relative.parts else ".",
                    max_entries=candidate_snapshot.MAX_SNAPSHOT_ENTRIES,
                    max_bytes=candidate_snapshot.MAX_SNAPSHOT_BYTES,
                )
        # Apply P3G's descriptor-pinned entry and byte budgets before the intake
        # runner copies or repeatedly hashes the same prepared source closure.
        return candidate_snapshot._inventory(root.resolve(strict=True))
    except CandidateFSError as error:
        raise PipelineError(str(error)) from error
    except candidate_snapshot.SnapshotError as error:
        raise PipelineError(str(error)) from error
    except OSError as error:
        raise PipelineError(f"cannot inventory source closure {root}: {error}") from error


def tree_digest(files: dict[str, str]) -> str:
    return hashlib.sha256(json.dumps(files, sort_keys=True).encode()).hexdigest()


def atomic_write_new(path: Path, contents: bytes, mode: int | None = None) -> None:
    context = _ACTIVE_FD_CONTEXT.get()
    if context is not None:
        relative = _candidate_relative(path, context)
        if relative is not None:
            try:
                context["candidate_dir"].atomic_write(
                    relative, contents,
                    mode=mode if mode is not None else (stat.S_IRUSR | stat.S_IWUSR),
                    replace=False,
                )
            except CandidateFSError as error:
                raise PipelineError(str(error)) from error
            return
    temporary = path.with_name(f".{path.name}.{uuid.uuid4().hex}.tmp")
    try:
        with temporary.open("xb") as stream:
            stream.write(contents)
            stream.flush()
            os.fsync(stream.fileno())
        if mode is not None:
            temporary.chmod(mode)
        os.link(temporary, path)
    finally:
        temporary.unlink(missing_ok=True)


def atomic_replace(path: Path, contents: bytes) -> None:
    context = _ACTIVE_FD_CONTEXT.get()
    if context is not None:
        relative = _candidate_relative(path, context)
        if relative is not None:
            context["candidate_dir"].atomic_write(relative, contents, replace=True)
            return
    temporary = path.with_name(f".{path.name}.{uuid.uuid4().hex}.tmp")
    try:
        with temporary.open("xb") as stream:
            stream.write(contents)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, path)
    finally:
        temporary.unlink(missing_ok=True)


def _candidate_relative(path: Path, context: dict[str, Any]) -> str | None:
    try:
        relative = Path(os.path.abspath(path)).relative_to(context["candidate_path"])
    except ValueError:
        return None
    if not relative.parts:
        return None
    return relative.as_posix()


def _write_exclusive_text(path: Path, contents: str) -> None:
    raw = contents.encode("utf-8")
    context = _ACTIVE_FD_CONTEXT.get()
    relative = _candidate_relative(path, context) if context is not None else None
    if relative is not None:
        try:
            context["candidate_dir"].atomic_write(relative, raw, replace=False)
        except CandidateFSError as error:
            raise PipelineError(str(error)) from error
    else:
        with path.open("x", encoding="utf-8") as stream:
            stream.write(contents)


def _candidate_exists(path: Path, *, directory: bool = False) -> bool:
    context = _ACTIVE_FD_CONTEXT.get()
    relative = _candidate_relative(path, context) if context is not None else None
    if relative is None:
        return path.is_dir() if directory else path.exists()
    parent, name = relative.rsplit("/", 1) if "/" in relative else (".", relative)
    try:
        parent_dir = (PinnedDir.from_fd(context["candidate_dir"].fd) if parent == "."
                      else PinnedDir.open_child(context["candidate_dir"], parent))
    except CandidateFSError:
        return False
    try:
        try:
            info = os.stat(name, dir_fd=parent_dir.fd, follow_symlinks=False)
        except FileNotFoundError:
            return False
        return stat.S_ISDIR(info.st_mode) if directory else True
    finally:
        parent_dir.close()


def load_json(path: Path, *, root: Path | None = None) -> tuple[Any, bytes]:
    with _open_regular(path, "JSON input", root=root) as stream:
        raw = stream.read(MAX_JSON_BYTES + 1)
    if len(raw) > MAX_JSON_BYTES:
        raise PipelineError(f"JSON input exceeds {MAX_JSON_BYTES} bytes: {path}")
    return json.loads(raw.decode("utf-8"), object_pairs_hook=unique_json), raw


def _patch_bindings(record: Any, *, is_plan: bool) -> list[tuple[str, str]]:
    label = "P3E plan" if is_plan else "prepared source manifest"
    if not isinstance(record, dict):
        raise PipelineError(f"{label} patch declaration must be an object")
    if not is_plan and record.get("schema_version") == 1:
        path, digest = "approved.patch", record.get("patch_sha256")
        if not isinstance(digest, str) or HASH_RE.fullmatch(digest) is None:
            raise PipelineError("legacy source manifest patch_sha256 is missing or malformed")
        return [(path, digest)]

    entries = record.get("patches")
    if not isinstance(entries, list):
        raise PipelineError(f"{label} must declare an ordered patch list")
    if not entries:
        raise PipelineError(f"{label} ordered patch list must contain at least one patch")
    result: list[tuple[str, str]] = []
    for index, entry in enumerate(entries):
        if not isinstance(entry, dict):
            raise PipelineError(f"{label} patches[{index}] must be an object")
        path, digest = entry.get("path"), entry.get("sha256")
        if (not isinstance(path, str) or not path or "\x00" in path
                or not isinstance(digest, str) or HASH_RE.fullmatch(digest) is None):
            raise PipelineError(f"{label} patches[{index}] has an invalid path or SHA-256")
        result.append((path, digest))
    return result


def validate_intake(data: Any, root: Path, intake_path: Path) -> tuple[str, Path, dict[str, Any], str, Path | None]:
    if not isinstance(data, dict) or set(data) != {
        "schema_version", "approved", "approval_reference", "candidate_id",
        "source_manifest", "source_candidate_id", "patch_plan", "purpose",
    }:
        raise PipelineError("intake manifest has missing or unknown fields")
    if type(data["schema_version"]) is not int or data["schema_version"] != 1:
        raise PipelineError("intake schema_version must be 1")
    if data["approved"] is not True:
        raise PipelineError("intake must explicitly set approved=true")
    for field in ("approval_reference", "purpose"):
        if not isinstance(data[field], str) or not data[field].strip() or "\x00" in data[field]:
            raise PipelineError(f"{field} must be a nonempty string")
    candidate_id, source_id = data["candidate_id"], data["source_candidate_id"]
    for label, value in (("candidate_id", candidate_id), ("source_candidate_id", source_id)):
        if not isinstance(value, str) or ID_RE.fullmatch(value) is None or ".." in value:
            raise PipelineError(f"{label} must be a safe candidate identifier")
    if candidate_id == source_id:
        raise PipelineError("destination candidate ID must differ from source candidate ID")
    ref = data["source_manifest"]
    if not isinstance(ref, dict) or set(ref) != {"path", "sha256"}:
        raise PipelineError("source_manifest must contain exactly path and sha256")
    relative = safe_relative(ref["path"], "source_manifest.path")
    if not isinstance(ref["sha256"], str) or HASH_RE.fullmatch(ref["sha256"]) is None:
        raise PipelineError("source_manifest.sha256 must be lowercase SHA-256")
    manifest_path = resolve_inside(root / relative, root, "source manifest")
    manifest, raw = load_json(manifest_path, root=root)
    if hashlib.sha256(raw).hexdigest() != ref["sha256"]:
        raise PipelineError("source manifest hash drift")
    if not isinstance(manifest, dict) or manifest.get("candidate_id") != source_id:
        raise PipelineError("source manifest candidate ID mismatch")
    if type(manifest.get("schema_version")) is not int or manifest["schema_version"] not in (1, 2, 3):
        raise PipelineError("unsupported prepared source manifest schema")
    plan_ref = data["patch_plan"]
    plan_path = None
    if plan_ref is not None:
        if not isinstance(plan_ref, dict) or set(plan_ref) != {"path", "sha256"}:
            raise PipelineError("patch_plan must be null or contain exactly path and sha256")
        plan_relative = safe_relative(plan_ref["path"], "patch_plan.path")
        if not isinstance(plan_ref["sha256"], str) or HASH_RE.fullmatch(plan_ref["sha256"]) is None:
            raise PipelineError("patch_plan.sha256 must be lowercase SHA-256")
        plan_path = resolve_inside(manifest_path.parent / plan_relative,
                                   manifest_path.parent, "P3E patch plan")
        plan, plan_raw = load_json(plan_path, root=manifest_path.parent)
        if hashlib.sha256(plan_raw).hexdigest() != plan_ref["sha256"]:
            raise PipelineError("P3E patch plan hash drift")
        if not isinstance(plan, dict) or not isinstance(plan.get("candidate"), dict):
            raise PipelineError("P3E plan candidate identity is missing")
        if plan["candidate"].get("id") != source_id:
            raise PipelineError("P3E plan must identify the supplied source candidate")
        if plan["candidate"].get("parent_manifest_sha256") != hashlib.sha256(raw).hexdigest():
            raise PipelineError(
                "P3E plan parent manifest SHA-256 does not match the prepared source manifest"
            )
        snapshots = manifest.get("snapshots")
        candidate_snapshot = snapshots.get("candidate") if isinstance(snapshots, dict) else None
        snapshot_sha256 = (candidate_snapshot.get("path_tree_sha256")
                           if isinstance(candidate_snapshot, dict) else None)
        plan_snapshot_sha256 = plan["candidate"].get("snapshot_sha256")
        if (not isinstance(candidate_snapshot, dict)
                or not isinstance(snapshot_sha256, str)
                or HASH_RE.fullmatch(snapshot_sha256) is None
                or plan_snapshot_sha256 != snapshot_sha256):
            raise PipelineError(
                "P3E plan snapshot SHA-256 does not match the prepared candidate path tree"
            )
    return candidate_id, manifest_path.parent, manifest, hashlib.sha256(raw).hexdigest(), plan_path


def _copy_source(source: Path, destination: Path) -> dict[str, str]:
    context = _ACTIVE_FD_CONTEXT.get()
    with ExitStack() as stack:
        try:
            if context is None:
                source_root = stack.enter_context(PinnedDir.open_root(source))
                destination.parent.mkdir(parents=True, exist_ok=True)
                candidate_root = stack.enter_context(PinnedDir.open_root(destination.parent))
                destination_name = destination.name
                source_dir = stack.enter_context(PinnedDir.from_fd(source_root.fd))
                candidate_root.mkdir(destination_name)
                destination_dir = stack.enter_context(PinnedDir.open_child(candidate_root, destination_name))
            else:
                source_root = stack.enter_context(PinnedDir.from_fd(context["source_root_fd"]))
                candidate_root = stack.enter_context(PinnedDir.from_fd(context["candidate_fd"]))
                destination_name = destination.relative_to(context["candidate_path"]).as_posix()
                source_relative = source.relative_to(context["source_path"])
                source_dir = stack.enter_context(PinnedDir.open_child(source_root, source_relative))
                destination_dir = stack.enter_context(PinnedDir.create_child(candidate_root, destination_name))
            before = source_dir.inventory(
                max_entries=candidate_snapshot.MAX_SNAPSHOT_ENTRIES,
                max_bytes=candidate_snapshot.MAX_SNAPSHOT_BYTES,
            )
            destination_dir.copy_tree_from(
                source_dir,
                max_entries=candidate_snapshot.MAX_SNAPSHOT_ENTRIES,
                max_bytes=candidate_snapshot.MAX_SNAPSHOT_BYTES,
            )
            if source_dir.inventory(
                    max_entries=candidate_snapshot.MAX_SNAPSHOT_ENTRIES,
                    max_bytes=candidate_snapshot.MAX_SNAPSHOT_BYTES) != before:
                raise PipelineError(f"prepared source tree changed during intake copy: {source}")
            copied = destination_dir.inventory(
                max_entries=candidate_snapshot.MAX_SNAPSHOT_ENTRIES,
                max_bytes=candidate_snapshot.MAX_SNAPSHOT_BYTES,
            )
            if copied != before:
                raise PipelineError(f"copied source tree changed during intake: {source}")
            return copied
        except CandidateFSError as error:
            message = str(error)
            if "symlink" in message:
                message = f"symlink rejected during source copy: {message}"
            elif "special file" in message or "non-regular" in message:
                message = f"source entry is not a regular file: {message}"
            raise PipelineError(message) from error

def _copy_directory_nofollow(source: Path, destination: Path) -> None:
    directory_flags = os.O_RDONLY | getattr(os, "O_DIRECTORY", 0) | getattr(os, "O_NOFOLLOW", 0)
    try:
        source_fd = os.open(source, directory_flags)
    except OSError as error:
        raise PipelineError(f"source snapshot directory changed during copy: {source}") from error
    try:
        _copy_directory_fd(source_fd, source, destination, directory_flags)
    finally:
        os.close(source_fd)


def _copy_directory_fd(source_fd: int, source: Path, destination: Path,
                       directory_flags: int) -> None:
    destination.mkdir(parents=True)
    source_mode = os.fstat(source_fd).st_mode
    with os.scandir(source_fd) as entries:
        names = [entry.name for entry in entries]
    for name in names:
        child_destination = destination / name
        try:
            child_fd = os.open(name, directory_flags, dir_fd=source_fd)
        except OSError as error:
            try:
                child_mode = os.stat(name, dir_fd=source_fd, follow_symlinks=False).st_mode
            except OSError:
                child_mode = 0
            if stat.S_ISLNK(child_mode):
                raise PipelineError(f"symlink rejected during source copy: {source / name}") from error
            child_fd = None
        if child_fd is not None:
            try:
                _copy_directory_fd(child_fd, source / name, child_destination, directory_flags)
            finally:
                os.close(child_fd)
            continue
        try:
            file_fd = os.open(name, os.O_RDONLY | getattr(os, "O_NONBLOCK", 0)
                              | getattr(os, "O_NOFOLLOW", 0), dir_fd=source_fd)
        except OSError as error:
            raise PipelineError(f"source snapshot entry changed during copy: {source / name}") from error
        try:
            file_mode = os.fstat(file_fd).st_mode
            if not stat.S_ISREG(file_mode):
                raise PipelineError(f"source snapshot entry is not a regular file: {source / name}")
            output_fd = os.open(child_destination, os.O_WRONLY | os.O_CREAT | os.O_EXCL,
                                stat.S_IMODE(file_mode))
            try:
                with os.fdopen(os.dup(file_fd), "rb") as input_stream, os.fdopen(output_fd, "wb") as output_stream:
                    shutil.copyfileobj(input_stream, output_stream)
                    output_stream.flush()
                    os.fchmod(output_stream.fileno(), stat.S_IMODE(file_mode))
            except Exception:
                try:
                    os.close(output_fd)
                except OSError:
                    pass
                raise
        finally:
            os.close(file_fd)
    os.chmod(destination, stat.S_IMODE(source_mode))


def _copy_regular_file(source: Path, destination: Path, label: str = "source snapshot file",
                       *, root: Path | None = None, max_bytes: int | None = None) -> Path:
    """Copy a regular file through a no-follow descriptor and retain its mode."""
    with _open_regular(source, label, root=root) as input_stream:
        source_mode = os.fstat(input_stream.fileno()).st_mode
        contents = input_stream.read() if max_bytes is None else input_stream.read(max_bytes + 1)
        if max_bytes is not None and len(contents) > max_bytes:
            raise PipelineError(f"{label} exceeds the {max_bytes}-byte copy limit: {source}")
        context = _ACTIVE_FD_CONTEXT.get()
        relative = _candidate_relative(destination, context) if context is not None else None
        if relative is not None:
            context["candidate_dir"].atomic_write(
                relative, contents, mode=stat.S_IMODE(source_mode), replace=False
            )
        else:
            with open(destination, "xb") as output_stream:
                output_stream.write(contents)
                output_stream.flush()
                os.fchmod(output_stream.fileno(), stat.S_IMODE(source_mode))
    return destination


def _rewrite_manifest(source_manifest: dict[str, Any], candidate_id: str,
                      source_root: Path, candidate_root: Path, root: Path) -> dict[str, Any]:
    result = json.loads(json.dumps(source_manifest))
    result["candidate_id"] = candidate_id
    snapshots = result.get("snapshots")
    if not isinstance(snapshots, dict):
        raise PipelineError("prepared manifest has no paired snapshots")
    for variant in ("baseline", "candidate"):
        record = snapshots.get(variant)
        if not isinstance(record, dict) or not isinstance(record.get("source"), str):
            raise PipelineError(f"prepared manifest snapshots.{variant}.source is missing")
        src = Path(record["source"])
        if not src.is_absolute():
            src = root / src
        src = resolve_inside(src, source_root, f"{variant} source")
        expected = source_root / variant / "source"
        if src != expected.resolve(strict=True):
            raise PipelineError(f"prepared source must be exactly {variant}/source")
        declared_files = record.get("files")
        if not isinstance(declared_files, dict) or inventory(src) != declared_files:
            raise PipelineError(f"{variant} source closure hash drift")
        dest = candidate_root / variant / "source"
        files = _copy_source(src, dest)
        record["source"] = f"{variant}/source"
        record["target"] = f"{variant}/target"
        record["files"] = files
        path_files = {k: v for k, v in files.items() if k.startswith("path-sources/")}
        package_names = {p.split("/", 2)[1] for p in path_files if p.count("/") >= 2}
        if len(package_names) == 1:
            prefix = f"path-sources/{next(iter(package_names))}/"
            record["path_tree_sha256"] = tree_digest({
                k[len(prefix):]: v for k, v in path_files.items() if k.startswith(prefix)
            })
        else:
            raise PipelineError(f"{variant} snapshot must contain exactly one path-source package")
    # Snapshot absolute paths are local to this candidate; update any provenance path
    # fields that the existing validators consume while preserving other audit data.
    result["repository"] = str(root)
    return result


def run_step(name: str, command: list[str], cwd: Path, evidence_dir: Path,
             manifest_hash: str, events: list[dict[str, Any]]) -> bool:
    context = _ACTIVE_FD_CONTEXT.get()
    log = evidence_dir / f"{len(events) + 1:02d}-{name}.log"
    is_p3h = name.startswith("P3H-")
    report_output: str | None = None
    try:
        proc = subprocess.Popen(
            _fd_anchored_command(command, context) if context else command,
            cwd=None if context else cwd,
            env=context["env"] if context else None,
            pass_fds=context["pass_fds"] if context else (),
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE if is_p3h else subprocess.STDOUT,
            start_new_session=True,
        )
        assert proc.stdout is not None
        streams = {"stdout": proc.stdout}
        if is_p3h:
            assert proc.stderr is not None
            streams["stderr"] = proc.stderr
        captured = {name: bytearray() for name in streams}
        total = 0
        overflow = False
        timed_out = False
        termination_time: float | None = None
        deadline = time.monotonic() + STEP_TIMEOUT_SECONDS
        with selectors.DefaultSelector() as selector:
            for stream_name, stream in streams.items():
                selector.register(stream, selectors.EVENT_READ, stream_name)
            while selector.get_map() or proc.poll() is None:
                now = time.monotonic()
                if now >= deadline and not overflow:
                    timed_out = True
                    try:
                        os.killpg(proc.pid, signal.SIGTERM)
                    except ProcessLookupError:
                        pass
                    termination_time = now
                    deadline = float("inf")
                if termination_time is not None and now - termination_time >= 1:
                    try:
                        os.killpg(proc.pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                    termination_time = None
                wait = min(0.1, max(0.0, deadline - now))
                for key, _ in selector.select(wait):
                    chunk = os.read(key.fileobj.fileno(), 64 * 1024)
                    if not chunk:
                        selector.unregister(key.fileobj)
                        continue
                    remaining = MAX_STEP_OUTPUT_BYTES - total
                    captured[key.data].extend(chunk[:remaining])
                    total += min(len(chunk), remaining)
                    if len(chunk) > remaining and not overflow:
                        overflow = True
                        try:
                            os.killpg(proc.pid, signal.SIGTERM)
                        except ProcessLookupError:
                            pass
                        termination_time = time.monotonic()
                if overflow and termination_time is not None and time.monotonic() - termination_time >= 1:
                    try:
                        os.killpg(proc.pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                    termination_time = None
        for stream in streams.values():
            stream.close()
        proc.wait()
        stdout = captured["stdout"].decode("utf-8", errors="replace")
        stderr = captured.get("stderr", bytearray()).decode("utf-8", errors="replace")
        if overflow:
            status = 125
            output = stdout
            if is_p3h and stderr:
                output += "\n--- stderr ---\n" + stderr
            output += f"\nvalidator output exceeded {MAX_STEP_OUTPUT_BYTES}-byte limit; process terminated\n"
        elif timed_out:
            status = 124
            output = stdout
            if is_p3h and stderr:
                output += "\n--- stderr ---\n" + stderr
            output += f"\nvalidator exceeded {STEP_TIMEOUT_SECONDS}-second timeout\n"
        else:
            status = proc.returncode
            output = stdout
            if is_p3h and stderr:
                output += "\n--- stderr ---\n" + stderr
        if is_p3h and not overflow and not timed_out:
            report_output = stdout
    except OSError as error:
        status, output = 127, f"unable to execute validator: {error}\n"
    if is_p3h and report_output is not None:
        report_path = evidence_dir / f"{name}.json"
        _write_exclusive_text(report_path, report_output)
    _write_exclusive_text(log, output)
    event = {
        "schema_version": 1,
        "step": name, "command": command, "exit_status": status,
        "status": "passed" if status == 0 else "failed",
        "log": {"path": log.name, "sha256": sha256(log)},
        "manifest_sha256": manifest_hash,
        "recorded_at": datetime.now(timezone.utc).isoformat(),
    }
    _append_event(event, evidence_dir, events)
    return status == 0


def _default_patch_run_id() -> str:
    timestamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    return f"pipeline-{timestamp}-{uuid.uuid4().hex}"

def _append_event(event: dict[str, Any], evidence_dir: Path,
                  events: list[dict[str, Any]]) -> None:
    slug = re.sub(r"[^A-Za-z0-9._-]+", "-", event["step"]).strip("-")
    path = evidence_dir / f"event-{len(events) + 1:02d}-{slug}.json"
    raw = (json.dumps(event, indent=2, sort_keys=True) + "\n").encode("utf-8")
    context = _ACTIVE_FD_CONTEXT.get()
    relative = _candidate_relative(path, context) if context is not None else None
    if relative is not None:
        context["candidate_dir"].atomic_write(relative, raw, replace=False)
    else:
        with path.open("xb") as stream:
            stream.write(raw)
    events.append({**event, "event_record": {"path": path.name,
                                               "sha256": hashlib.sha256(raw).hexdigest()}})


def _write_failure(candidate_id: str, destination: Path, evidence: Path,
                   events: list[dict[str, Any]], error: BaseException) -> None:
    failure = {"schema_version": 1, "candidate_id": candidate_id,
               "status": "failed", "error": f"{type(error).__name__}: {error}",
               "events": events, "gates": GATES}
    if _candidate_exists(evidence, directory=True):
        failure_path = evidence / "pipeline-failure.json"
    elif _candidate_exists(destination, directory=True):
        failure_path = destination / "pipeline-failure.json"
    else:
        failure_path = destination.parent / f"{candidate_id}.pipeline-failure.json"
    if not _candidate_exists(failure_path):
        _write_exclusive_text(failure_path, json.dumps(failure, indent=2, sort_keys=True) + "\n")


def run_pipeline(intake_path: Path, root: Path, run_id: str | None = None) -> dict[str, Any]:
    root = Path(root).resolve(strict=True)
    with ExitStack() as stack:
        repo_root = stack.enter_context(PinnedDir.open_root(root))
        token = _ACTIVE_FD_CONTEXT.set(None)
        try:
            return _run_pipeline(intake_path, root, run_id, repo_root, stack)
        finally:
            _ACTIVE_FD_CONTEXT.reset(token)


def _run_pipeline(intake_path: Path, root: Path, run_id: str | None,
                  repo_root: PinnedDir, stack: ExitStack) -> dict[str, Any]:
    intake_path = resolve_inside(intake_path if intake_path.is_absolute() else root / intake_path,
                                 root, "intake manifest")
    intake, intake_raw = load_json(intake_path)
    intake_hash = hashlib.sha256(intake_raw).hexdigest()
    candidate_id, source_root, source_manifest, source_hash, plan_path = validate_intake(intake, root, intake_path)
    if run_id is not None and (ID_RE.fullmatch(run_id) is None or ".." in run_id):
        raise PipelineError("run_id must be a safe identifier")
    source_relative = source_root.relative_to(root)
    source_dir = stack.enter_context(PinnedDir.open_child(repo_root, source_relative))
    source_root = root / source_relative
    candidates = stack.enter_context(
        PinnedDir.create_child(repo_root, "target/headstart/candidates")
    )
    try:
        candidates.mkdir(candidate_id)
    except CandidateFSError as error:
        raise PipelineError(f"candidate ID already exists or is unsafe: {candidate_id}") from error
    try:
        destination_dir = stack.enter_context(PinnedDir.open_child(candidates, candidate_id))
    except CandidateFSError as error:
        # The candidate name was created exclusively above, but a race can
        # prevent us from pinning it. Keep failure evidence beside that name
        # through the already-pinned parent rather than following the path.
        failure = {"schema_version": 1, "candidate_id": candidate_id,
                   "status": "failed", "error": f"{type(error).__name__}: {error}",
                   "events": [], "gates": GATES}
        try:
            candidates.atomic_write(
                f"{candidate_id}.pipeline-failure.json",
                (json.dumps(failure, indent=2, sort_keys=True) + "\n").encode("utf-8"),
                replace=False,
            )
        except CandidateFSError as record_error:
            error.add_note(f"also failed to write pipeline failure evidence: {record_error}")
        raise
    destination = root / "target/headstart/candidates" / candidate_id
    candidate_relative = f"target/headstart/candidates/{candidate_id}"
    source_relative_text = source_relative.as_posix()
    child_env = os.environ.copy()
    child_env.update({
        "RUSTUI_P3_REPO_ROOT_FD": str(repo_root.fd),
        "RUSTUI_P3_CANDIDATE_PARENT_FD": str(candidates.fd),
        "RUSTUI_P3_CANDIDATE_ID": candidate_id,
        "RUSTUI_P3_CANDIDATE_ROOT_FD": str(destination_dir.fd),
        "RUSTUI_P3_SOURCE_ROOT_FD": str(source_dir.fd),
    })
    _ACTIVE_FD_CONTEXT.set({
        "root": root,
        "candidate_path": destination,
        "candidate_relative": candidate_relative,
        "source_path": source_root,
        "source_relative": source_relative_text,
        "candidate_fd": destination_dir.fd,
        "source_root_fd": source_dir.fd,
        "candidate_dir": destination_dir,
        "source_dir": source_dir,
        "env": child_env,
        "pass_fds": (repo_root.fd, candidates.fd, destination_dir.fd, source_dir.fd),
    })
    evidence = destination / "evidence"
    events: list[dict[str, Any]] = []
    owns_destination = True
    try:
        destination_dir.mkdir("evidence")
        approved_intake_path = destination / "approved-intake.json"
        atomic_write_new(approved_intake_path, intake_raw)
        if sha256(approved_intake_path) != intake_hash:
            raise PipelineError("approved intake changed during evidence copy")
        manifest_hash = source_hash
        if plan_path is None:
            _append_event({"schema_version": 1, "step": "P3E-patch-plan",
                           "status": "not_run", "exit_status": None,
                           "reason": "no hash-pinned local P3E plan; candidate source work is blocked",
                           "manifest_sha256": source_hash,
                           "recorded_at": datetime.now(timezone.utc).isoformat()}, evidence, events)
            raise PipelineError("candidate source checks require a hash-pinned P3E patch plan")

        plan_hash = intake["patch_plan"]["sha256"]
        if sha256(plan_path, root=source_root) != plan_hash:
            raise PipelineError("P3E patch plan changed before validation")
        _copy_regular_file(plan_path, evidence / "approved-patch-plan.json", "P3E patch plan",
                           root=source_root)
        if sha256(evidence / "approved-patch-plan.json") != plan_hash:
            raise PipelineError("P3E patch plan changed during evidence copy")
        p3e_ok = run_step("P3E-patch-plan", [
            sys.executable, str(root / "scripts/validate_patch_plan.py"),
            str(plan_path), "--root", str(source_root),
        ], root, evidence, source_hash, events)
        if not p3e_ok:
            raise PipelineError("P3E patch plan validation failed; candidate source work is blocked")
        if sha256(plan_path, root=source_root) != plan_hash:
            raise PipelineError("P3E patch plan changed during validation")
        plan, _ = load_json(plan_path)
        try:
            plan_patches = _patch_bindings(plan, is_plan=True)
            manifest_patches = _patch_bindings(source_manifest, is_plan=False)
            if plan_patches != manifest_patches:
                raise PipelineError(
                    "P3E ordered patch declarations do not match the prepared source manifest"
                )
        except PipelineError as error:
            _append_event({"schema_version": 1, "step": "P3E-patch-binding",
                           "status": "failed", "exit_status": None,
                           "reason": str(error), "manifest_sha256": source_hash,
                           "recorded_at": datetime.now(timezone.utc).isoformat()}, evidence, events)
            raise
        _append_event({"schema_version": 1, "step": "P3E-patch-binding",
                       "status": "passed", "exit_status": 0,
                       "reason": "ordered path and SHA-256 declarations match",
                       "manifest_sha256": source_hash,
                       "recorded_at": datetime.now(timezone.utc).isoformat()}, evidence, events)
        patch_entries = source_manifest.get("patches")
        if not isinstance(patch_entries, list):
            src = source_root / "approved.patch"
            if src.exists():
                src = resolve_inside(src, source_root, "approved.patch")
                declared_patch_hash = source_manifest.get("patch_sha256")
                if not isinstance(declared_patch_hash, str) or HASH_RE.fullmatch(declared_patch_hash) is None:
                    raise PipelineError("single patch has no declared SHA-256")
                if sha256(src, root=source_root) != declared_patch_hash:
                    raise PipelineError("approved patch hash drift")
                _copy_regular_file(src, destination / "approved.patch", "approved patch",
                                   root=source_root, max_bytes=MAX_PATCH_BYTES)
                if sha256(destination / "approved.patch") != declared_patch_hash:
                    raise PipelineError("approved patch changed during intake copy")
        if isinstance(patch_entries, list):
            for entry in patch_entries:
                if not isinstance(entry, dict) or not isinstance(entry.get("path"), str):
                    raise PipelineError("malformed ordered patch series in source manifest")
                patch_name = safe_relative(entry["path"], "patch path")
                if len(patch_name.parts) != 1 or patch_name.suffix != ".patch":
                    raise PipelineError("patch series entries must be local .patch filenames")
                source_patch = resolve_inside(source_root / patch_name, source_root, "patch series input")
                if (not isinstance(entry.get("sha256"), str)
                        or sha256(source_patch, root=source_root) != entry["sha256"]):
                    raise PipelineError(f"patch series hash drift: {patch_name.name}")
                _copy_regular_file(source_patch, destination / patch_name.name,
                                   "patch series input", root=source_root,
                                   max_bytes=MAX_PATCH_BYTES)
                if sha256(destination / patch_name.name) != entry["sha256"]:
                    raise PipelineError(f"patch changed during intake copy: {patch_name.name}")
        copied = _rewrite_manifest(source_manifest, candidate_id, source_root,
                                   destination, root)
        if sha256(source_root / "manifest.json", root=source_root) != source_hash:
            raise PipelineError("source manifest changed during intake")
        if sha256(plan_path, root=source_root) != plan_hash:
            raise PipelineError("P3E patch plan changed during intake")
        manifest_path = destination / "manifest.json"
        manifest_raw = (json.dumps(copied, indent=2, sort_keys=True) + "\n").encode()
        atomic_write_new(manifest_path, manifest_raw,
                         stat.S_IRUSR | stat.S_IRGRP | stat.S_IROTH)
        manifest_hash = hashlib.sha256(manifest_raw).hexdigest()
        immutable = {
            "schema_version": 1,
            "candidate_id": candidate_id,
            "purpose": intake["purpose"],
            "approval_reference": intake["approval_reference"],
            "source_candidate_id": intake["source_candidate_id"],
            "source_manifest_sha256": source_hash,
            "source_snapshot_manifest_sha256": source_hash,
            "p3e_patch_plan_sha256": sha256(plan_path, root=source_root) if plan_path is not None else None,
            "manifest_sha256": manifest_hash,
            "snapshots": {},
        }
        for variant in ("baseline", "candidate"):
            snap = copied["snapshots"][variant]
            source_path = Path(snap["source"])
            if not source_path.is_absolute():
                source_path = destination / source_path
            files = inventory(source_path)
            immutable["snapshots"][variant] = {
                "source": Path(snap["source"]).as_posix(),
                "files": files,
                "tree_sha256": tree_digest(files),
            }
        immutable_path = destination / "source-hash-manifest.json"
        atomic_write_new(immutable_path,
                         (json.dumps(immutable, indent=2, sort_keys=True) + "\n").encode(),
                         stat.S_IRUSR | stat.S_IRGRP | stat.S_IROTH)
        immutable_hash = sha256(immutable_path)

        # P3E already passed before any prepared source bytes were copied.
        validators = []
        validators.append(
            ("P3G-snapshot", [sys.executable, str(root / "scripts/validate_candidate_snapshot.py"),
                              str(manifest_path), "--root", str(destination)])
        )
        # P3H runs separately for both sources; P3I also performs the P3J graph comparison
        # when both normalized schema-2 reports are available.
        h_reports: list[Path] = []
        for variant in ("baseline", "candidate"):
            report_path = evidence / f"P3H-{variant}.json"
            cmd = [sys.executable, str(root / "scripts/report_candidate_sources.py"),
                   str(destination), "--workspace", f"{variant}/source"]
            validators.append((f"P3H-{variant}", cmd))
            h_reports.append(report_path)
        validators.append(("P3I-J-source-and-graph-comparison", [
            sys.executable, str(root / "scripts/compare_candidate_sources.py"),
            str(h_reports[0]), str(h_reports[1]),
        ]))
        patch_run = run_id or _default_patch_run_id()
        validators.append(("P3K-exact-patch", [
            sys.executable, str(root / "scripts/apply_candidate_patch.py"),
            str(manifest_path), "--run-id", patch_run,
        ]))
        failed = False
        for name, command in validators:
            ok = run_step(name, command, root, evidence, manifest_hash, events)
            if not ok:
                failed = True
                break
        if failed:
            for name in ("P3G-snapshot", "P3H-baseline", "P3H-candidate", "P3I-J", "P3K"):
                if not any(e["step"].startswith(name) for e in events):
                    _append_event({"schema_version": 1, "step": name, "status": "not_run",
                                   "exit_status": None, "reason": "blocked by an earlier failed validator",
                                   "manifest_sha256": manifest_hash,
                                   "recorded_at": datetime.now(timezone.utc).isoformat()}, evidence, events)
        if candidates.child_identity(candidate_id) != destination_dir.identity():
            raise PipelineError("candidate path identity changed during validation")
        integrity_errors = []
        try:
            if sha256(immutable_path) != immutable_hash:
                integrity_errors.append("source-hash-manifest changed during candidate validation")
        except (OSError, PipelineError) as error:
            integrity_errors.append(f"source-hash-manifest cannot be revalidated: {error}")
        for variant in ("baseline", "candidate"):
            source_path = destination / variant / "source"
            try:
                if inventory(source_path) != immutable["snapshots"][variant]["files"]:
                    integrity_errors.append(f"{variant} source closure changed during candidate validation")
            except (OSError, PipelineError) as error:
                integrity_errors.append(f"{variant} source closure cannot be revalidated: {error}")
        integrity_ok = not integrity_errors
        _append_event({
            "schema_version": 1,
            "step": "P3G-final-source-integrity",
            "status": "passed" if integrity_ok else "failed",
            "exit_status": 0 if integrity_ok else 1,
            "reason": ("immutable source closures remain unchanged" if integrity_ok
                       else "; ".join(integrity_errors)),
            "manifest_sha256": manifest_hash,
            "recorded_at": datetime.now(timezone.utc).isoformat(),
        }, evidence, events)
        if not integrity_ok:
            failed = True
        primary_log = next(event["log"] for event in events if isinstance(event.get("log"), dict))
        evidence_record = {
            "manifest_sha256": manifest_hash,
            "command": [sys.executable, str(root / "scripts/run_candidate_pipeline.py"),
                        str(intake_path), "--root", str(root)]
                       + (["--run-id", run_id] if run_id is not None else []),
            # The evidence record describes this pipeline invocation. Preserve
            # an earlier validator failure in that invocation's recorded exit
            # status, even though P3F itself can still validate the record.
            "exit_status": 1 if failed else 0,
            "log": {"path": f"evidence/{primary_log['path']}",
                    "sha256": primary_log["sha256"]},
            "evidence": [
                {"path": "approved-intake.json", "sha256": intake_hash},
                {"path": "evidence/approved-patch-plan.json", "sha256": plan_hash},
                {"path": "manifest.json", "sha256": sha256(manifest_path)},
                {"path": "source-hash-manifest.json", "sha256": immutable_hash},
            ],
            "provenance": {"approved_intake_sha256": intake_hash,
                           "source_candidate_id": intake["source_candidate_id"]},
            "gates": {name: "not_run" for name in
                      ("build", "correctness", "gui", "performance")},
        }
        record_path = evidence / "P3F-record.json"
        _write_exclusive_text(record_path, json.dumps(evidence_record, indent=2, sort_keys=True) + "\n")
        p3f_ok = run_step("P3F-evidence-record", [
            sys.executable, str(root / "scripts/validate_candidate_evidence.py"),
            record_path.relative_to(destination).as_posix(), "--root", str(destination),
        ], root, evidence, manifest_hash, events)
        if not p3f_ok:
            failed = True
            evidence_record["exit_status"] = 1
        post_p3f_errors = []
        if candidates.child_identity(candidate_id) != destination_dir.identity():
            post_p3f_errors.append("candidate path identity changed during P3F validation")
        try:
            if sha256(manifest_path) != manifest_hash:
                post_p3f_errors.append("candidate manifest changed during P3F validation")
            if sha256(immutable_path) != immutable_hash:
                post_p3f_errors.append("source-hash-manifest changed during P3F validation")
            for variant in ("baseline", "candidate"):
                if inventory(destination / variant / "source") != immutable["snapshots"][variant]["files"]:
                    post_p3f_errors.append(f"{variant} source closure changed during P3F validation")
        except (OSError, PipelineError) as error:
            post_p3f_errors.append(f"candidate cannot be revalidated after P3F: {error}")
        _append_event({
            "schema_version": 1, "step": "P3G-post-P3F-integrity",
            "status": "failed" if post_p3f_errors else "passed",
            "exit_status": 1 if post_p3f_errors else 0,
            "reason": "; ".join(post_p3f_errors) if post_p3f_errors else "candidate inputs remain unchanged",
            "manifest_sha256": manifest_hash,
            "recorded_at": datetime.now(timezone.utc).isoformat(),
        }, evidence, events)
        if post_p3f_errors:
            failed = True
            evidence_record["exit_status"] = 1
        atomic_replace(
            record_path,
            (json.dumps(evidence_record, indent=2, sort_keys=True) + "\n").encode("utf-8"),
        )
        result = {
            "schema_version": 1, "candidate_id": candidate_id,
            "manifest_sha256": manifest_hash, "status": "failed" if failed else "partial",
            "scope": "local approved source intake, P3E/P3G/P3H/P3I-J/P3K/P3F checks when their pinned inputs are present; no build, correctness, GUI, performance, or promotion",
            "events": events, "gates": GATES,
            "source_hash_manifest_sha256": immutable_hash,
        }
        result_path = evidence / "pipeline-result.json"
        _write_exclusive_text(result_path, json.dumps(result, indent=2, sort_keys=True) + "\n")
        return result
    except (Exception, KeyboardInterrupt) as error:
        # Keep the intake and any copied bytes as a partial candidate; never erase evidence.
        # A concurrent run may have created this destination after our early
        # existence check. Never write failure evidence into a directory we do
        # not own.
        if owns_destination:
            try:
                _write_failure(candidate_id, destination, evidence, events, error)
            except Exception as record_error:
                try:
                    error.add_note(f"also failed to write pipeline failure evidence: {record_error}")
                except Exception:
                    pass
        raise


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("approved_manifest", type=Path, help="explicit local approved intake manifest")
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--run-id", help="unique P3K run ID")
    args = parser.parse_args()
    try:
        result = run_pipeline(args.approved_manifest, args.root, args.run_id)
    except (PipelineError, OSError, RuntimeError, ValueError, KeyError, TypeError) as error:
        print(f"candidate pipeline failed: {error}", file=sys.stderr)
        return 1
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0 if result["status"] == "partial" else 1


if __name__ == "__main__":
    raise SystemExit(main())
