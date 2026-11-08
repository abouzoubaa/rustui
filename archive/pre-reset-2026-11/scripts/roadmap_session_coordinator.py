#!/usr/bin/env python3
"""Coordinate exclusive RustUI roadmap work across linked Git worktrees."""

from __future__ import annotations

import argparse
import fcntl
import json
import os
from pathlib import Path, PurePosixPath
import re
import subprocess
import sys
import tempfile
from datetime import datetime, timezone


ACTIVE = "active"
NON_WRITING = {"awaiting_decision", "blocked_external", "completed", "released"}
VALID = {ACTIVE, *NON_WRITING}
TASK_ID = re.compile(r"^[A-Z0-9][A-Z0-9._-]{2,63}$")


class CoordinatorError(Exception):
    pass


def git(*args: str) -> str:
    try:
        return subprocess.check_output(
            ["git", *args], text=True, stderr=subprocess.PIPE
        ).strip()
    except (OSError, subprocess.CalledProcessError) as exc:
        raise CoordinatorError(f"git {' '.join(args)} failed: {exc}") from exc


def paths() -> tuple[Path, Path, Path]:
    root = Path(git("rev-parse", "--show-toplevel")).resolve()
    common = Path(git("rev-parse", "--git-common-dir"))
    if not common.is_absolute():
        common = (root / common).resolve()
    else:
        common = common.resolve()
    worktrees: list[Path] = []
    current = None
    for line in git("worktree", "list", "--porcelain").splitlines():
        if line.startswith("worktree "):
            current = Path(line.removeprefix("worktree ")).resolve()
            worktrees.append(current)
    if not worktrees:
        raise CoordinatorError("git reports no registered worktrees")
    return root, common, worktrees[0]


def now() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="seconds")


def ledger_paths(common: Path) -> tuple[Path, Path]:
    directory = common / "rustui-roadmap-coordination"
    directory.mkdir(mode=0o700, exist_ok=True)
    return directory / "ledger.json", directory / "ledger.lock"


def locked_ledger(common: Path):
    ledger_path, lock_path = ledger_paths(common)
    lock = lock_path.open("a+")
    fcntl.flock(lock.fileno(), fcntl.LOCK_EX)
    try:
        if ledger_path.exists():
            try:
                data = json.loads(ledger_path.read_text(encoding="utf-8"))
            except (OSError, json.JSONDecodeError) as exc:
                raise CoordinatorError(f"cannot read coordination ledger: {exc}") from exc
        else:
            data = {"schema_version": 1, "assignments": {}}
        if data.get("schema_version") != 1 or not isinstance(data.get("assignments"), dict):
            raise CoordinatorError("unsupported or malformed coordination ledger")
        yield data, ledger_path
    finally:
        fcntl.flock(lock.fileno(), fcntl.LOCK_UN)
        lock.close()


def write_ledger(data: dict, path: Path) -> None:
    fd, temporary = tempfile.mkstemp(prefix="ledger.", dir=path.parent)
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as stream:
            json.dump(data, stream, indent=2, sort_keys=True)
            stream.write("\n")
            stream.flush()
            os.fsync(stream.fileno())
        os.chmod(temporary, 0o600)
        os.replace(temporary, path)
        directory_fd = os.open(path.parent, os.O_RDONLY)
        try:
            os.fsync(directory_fd)
        finally:
            os.close(directory_fd)
    finally:
        if os.path.exists(temporary):
            os.unlink(temporary)


def normalize_claims(raw: list[str]) -> list[str]:
    if not raw:
        raise CoordinatorError("claim at least one exact file path")
    claims: set[str] = set()
    for item in raw:
        path = PurePosixPath(item)
        if path.is_absolute() or not path.parts or any(part in {".", ".."} for part in path.parts):
            raise CoordinatorError(f"claim must be a safe repository-relative path: {item}")
        if any(character in item for character in "*?[]"):
            raise CoordinatorError(f"wildcards are not allowed in file claims: {item}")
        claims.add(path.as_posix())
    return sorted(claims)


def overlaps(left: str, right: str) -> bool:
    return left == right or left.startswith(right + "/") or right.startswith(left + "/")


def claim(args: argparse.Namespace) -> None:
    root, common, primary = paths()
    worktree = Path(args.worktree).expanduser().resolve()
    _, registered, _ = paths()
    all_worktrees = {
        Path(line.removeprefix("worktree ")).resolve()
        for line in git("worktree", "list", "--porcelain").splitlines()
        if line.startswith("worktree ")
    }
    if worktree not in all_worktrees:
        raise CoordinatorError(f"not a registered linked worktree: {worktree}")
    if worktree == primary:
        raise CoordinatorError("the primary/shared checkout is read-only for coordinated roadmap work")
    claims = normalize_claims(args.file)
    if not args.next_task.strip():
        raise CoordinatorError("every assignment needs a concrete next action or fallback task")

    with locked_ledger(common) as (data, ledger_path):
        old = data["assignments"].get(args.task)
        if old and old["status"] not in NON_WRITING:
            if old.get("thread_id") != args.thread_id:
                raise CoordinatorError(f"task {args.task} is already held by {old.get('thread_id')}")
        for task_id, assignment in data["assignments"].items():
            if task_id == args.task or assignment.get("status") != ACTIVE:
                continue
            if assignment.get("thread_id") == args.thread_id:
                raise CoordinatorError(f"thread already owns active task {task_id}; release it first")
            if Path(assignment.get("worktree", "/")).resolve() == worktree:
                raise CoordinatorError(f"worktree is already active for task {task_id}")
            for wanted in claims:
                for held in assignment.get("files", []):
                    if overlaps(wanted, held):
                        raise CoordinatorError(f"file claim {wanted} overlaps {held} held by task {task_id}")
        data["assignments"][args.task] = {
            "task_id": args.task,
            "thread_id": args.thread_id,
            "stage": args.stage,
            "milestone": args.milestone,
            "worktree": str(worktree),
            "branch": git("-C", str(worktree), "branch", "--show-current") or "detached",
            "base_head": git("-C", str(worktree), "rev-parse", "HEAD"),
            "files": claims,
            "status": ACTIVE,
            "next_task": args.next_task.strip(),
            "updated_at": now(),
        }
        write_ledger(data, ledger_path)
    print(f"Claimed {args.task} for {args.thread_id} in {worktree}")


def transition(args: argparse.Namespace) -> None:
    _, common, _ = paths()
    with locked_ledger(common) as (data, ledger_path):
        item = data["assignments"].get(args.task)
        if not item:
            raise CoordinatorError(f"unknown task: {args.task}")
        if item.get("thread_id") != args.thread_id:
            raise CoordinatorError(f"task {args.task} is owned by {item.get('thread_id')}")
        if args.status not in VALID:
            raise CoordinatorError(f"invalid status: {args.status}")
        if args.status in {"awaiting_decision", "blocked_external"} and not args.next_task.strip():
            raise CoordinatorError("waiting states require an independent next task to keep the roadmap moving")
        item["status"] = args.status
        item["next_task"] = args.next_task.strip()
        item["updated_at"] = now()
        if args.status != ACTIVE:
            item["files"] = []
        write_ledger(data, ledger_path)
    print(f"Updated {args.task}: {args.status}")


def show(_: argparse.Namespace) -> None:
    _, common, _ = paths()
    with locked_ledger(common) as (data, _):
        assignments = sorted(data["assignments"].values(), key=lambda item: item["task_id"])
        if not assignments:
            print("No coordinated roadmap assignments.")
            return
        for item in assignments:
            files = ", ".join(item.get("files", [])) or "(no file claims)"
            print(
                f"{item['task_id']} | Stage {item['stage']} | {item['status']} | "
                f"thread={item['thread_id']} | worktree={item['worktree']} | files={files}\n"
                f"  next: {item.get('next_task', '')} | updated={item.get('updated_at', '')}"
            )


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser(description=__doc__)
    commands = root.add_subparsers(dest="command", required=True)
    add = commands.add_parser("claim", help="atomically reserve an isolated worktree and file set")
    add.add_argument("--task", required=True)
    add.add_argument("--thread-id", required=True)
    add.add_argument("--stage", required=True)
    add.add_argument("--milestone", required=True)
    add.add_argument("--worktree", required=True)
    add.add_argument("--file", action="append", default=[])
    add.add_argument("--next-task", required=True)
    add.set_defaults(run=claim)
    update = commands.add_parser("status", help="update status and release file claims when not writing")
    update.add_argument("--task", required=True)
    update.add_argument("--thread-id", required=True)
    update.add_argument("--status", choices=sorted(VALID), required=True)
    update.add_argument("--next-task", default="")
    update.set_defaults(run=transition)
    listing = commands.add_parser("list", help="show current assignments and owners")
    listing.set_defaults(run=show)
    return root


def main() -> int:
    args = parser().parse_args()
    try:
        args.run(args)
    except CoordinatorError as exc:
        print(f"coordination error: {exc}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
