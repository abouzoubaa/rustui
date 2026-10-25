#!/usr/bin/env python3
"""Atomic file and worktree claims for concurrent RustUI roadmap sessions."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import sqlite3
import stat
import subprocess
import sys
import time
from pathlib import Path


def repo_identity() -> tuple[Path, Path]:
    root = Path(subprocess.check_output(
        ["git", "rev-parse", "--show-toplevel"], text=True, stderr=subprocess.PIPE
    ).strip()).resolve()
    common = subprocess.check_output(
        ["git", "rev-parse", "--git-common-dir"], cwd=root, text=True,
        stderr=subprocess.PIPE,
    ).strip()
    common_path = Path(common)
    if not common_path.is_absolute():
        common_path = root / common_path
    return root, common_path.resolve()


def database_path(common_git_dir: Path) -> Path:
    digest = hashlib.sha256(str(common_git_dir).encode()).hexdigest()[:20]
    base = Path(os.environ.get("XDG_STATE_HOME", Path.home() / ".local/state"))
    return base / "rustui-roadmap" / f"claims-{digest}.sqlite3"


def connect(db_path: Path) -> sqlite3.Connection:
    db_path.parent.mkdir(parents=True, exist_ok=True)
    db = sqlite3.connect(db_path, timeout=15, isolation_level=None)
    db.execute("PRAGMA busy_timeout=15000")
    db.execute("PRAGMA journal_mode=WAL")
    db.execute("PRAGMA foreign_keys=ON")
    db.executescript("""
        BEGIN IMMEDIATE;
        CREATE TABLE IF NOT EXISTS claims (
            id TEXT PRIMARY KEY,
            task TEXT NOT NULL,
            thread TEXT NOT NULL,
            worktree TEXT NOT NULL,
            branch TEXT NOT NULL DEFAULT '',
            base_head TEXT NOT NULL DEFAULT '',
            stage TEXT NOT NULL DEFAULT '',
            milestone TEXT NOT NULL DEFAULT '',
            next_action TEXT NOT NULL DEFAULT '',
            state TEXT NOT NULL CHECK (state IN ('active','released')),
            created_at INTEGER NOT NULL,
            heartbeat_at INTEGER NOT NULL,
            lease_until INTEGER NOT NULL,
            release_state TEXT,
            handoff_terminal_state TEXT,
            evidence TEXT
        );
        CREATE TABLE IF NOT EXISTS claimed_paths (
            claim_id TEXT NOT NULL REFERENCES claims(id),
            path TEXT NOT NULL,
            PRIMARY KEY (claim_id, path)
        );
        CREATE TABLE IF NOT EXISTS claim_baselines (
            claim_id TEXT NOT NULL REFERENCES claims(id),
            path TEXT NOT NULL,
            fingerprint TEXT NOT NULL,
            PRIMARY KEY (claim_id, path)
        );
        CREATE INDEX IF NOT EXISTS claims_state_idx ON claims(state);
        CREATE INDEX IF NOT EXISTS claimed_paths_path_idx ON claimed_paths(path);
        COMMIT;
    """)
    # Migrate databases created by the first coordination draft.
    db.execute("BEGIN IMMEDIATE")
    try:
        columns = {row[1] for row in db.execute("PRAGMA table_info(claims)")}
        for name, definition in (
            ("branch", "TEXT NOT NULL DEFAULT ''"),
            ("base_head", "TEXT NOT NULL DEFAULT ''"),
            ("stage", "TEXT NOT NULL DEFAULT ''"),
            ("milestone", "TEXT NOT NULL DEFAULT ''"),
            ("next_action", "TEXT NOT NULL DEFAULT ''"),
            ("handoff_terminal_state", "TEXT"),
        ):
            if name not in columns:
                db.execute(f"ALTER TABLE claims ADD COLUMN {name} {definition}")
        for claim_id, worktree in db.execute(
            "SELECT id,worktree FROM claims WHERE branch='' AND state='active'"
        ).fetchall():
            path = Path(worktree)
            if path.is_dir():
                branch = git(path, "branch", "--show-current")
                if not branch:
                    branch = f"@detached:{path}:{git(path, 'rev-parse', '--short', 'HEAD')}"
                db.execute("UPDATE claims SET branch=? WHERE id=?", (branch, claim_id))
        db.execute("COMMIT")
    except BaseException:
        db.execute("ROLLBACK")
        raise
    return db


def git(root: Path, *args: str) -> str:
    return subprocess.check_output(
        ["git", "-C", str(root), *args], text=True, stderr=subprocess.PIPE
    ).strip()


def validate_writer_worktree(root: Path, common_git_dir: Path, require_clean: bool = True) -> str:
    """Require a registered, isolated codex/* linked worktree before writing."""
    if root == common_git_dir.parent:
        raise ValueError("the primary checkout is not eligible for parallel roadmap writes")
    listed = subprocess.check_output(
        ["git", "-C", str(root), "worktree", "list", "--porcelain"],
        text=True, stderr=subprocess.PIPE,
    )
    worktree_lines = [line[9:] for line in listed.splitlines() if line.startswith("worktree ")]
    registered = {Path(path).resolve() for path in worktree_lines}
    if root not in registered:
        raise ValueError("worktree is not registered with this Git repository")
    if worktree_lines and root == Path(worktree_lines[0]).resolve():
        raise ValueError("the primary checkout is not eligible for parallel roadmap writes")
    branch = git(root, "branch", "--show-current")
    if not branch.startswith("codex/"):
        raise ValueError("roadmap writer worktrees must use a codex/ branch")
    if require_clean:
        dirty = git(root, "status", "--porcelain=v1", "--untracked-files=all")
        if dirty:
            raise ValueError("writer worktree must be clean when claimed; preserve and resolve its existing changes first")
    return branch


def normalized_path(root: Path, raw: str) -> str:
    if raw.startswith("@shared/"):
        parts = Path(raw[8:]).parts
        if not parts or any(part in ("", ".", "..") for part in parts):
            raise ValueError(f"invalid shared resource path: {raw}")
        return "@shared/" + "/".join(parts)
    candidate = Path(raw)
    if candidate.is_absolute() or ".." in candidate.parts or raw in ("", "."):
        raise ValueError(f"claim paths must be repository-relative: {raw}")
    resolved = (root / candidate).resolve(strict=False)
    try:
        relative = resolved.relative_to(root).as_posix()
    except ValueError as exc:
        raise ValueError(f"claim path escapes repository through a symlink: {raw}") from exc
    if relative in ("", "."):
        raise ValueError("claim the specific files or directories, not the repository root")
    # macOS volumes are commonly case-insensitive; over-locking is safer than a race.
    return relative.casefold() if sys.platform == "darwin" else relative


def overlaps(left: str, right: str) -> bool:
    if left == right:
        return True
    return left.startswith(right.rstrip("/") + "/") or right.startswith(left.rstrip("/") + "/")


def changed_paths(root: Path) -> list[str]:
    raw = subprocess.check_output(
        ["git", "-C", str(root), "status", "--porcelain=v1", "-z", "--untracked-files=all"],
        stderr=subprocess.PIPE,
    )
    records = raw.split(b"\0")
    changed: list[str] = []
    i = 0
    while i < len(records) and records[i]:
        record = records[i].decode("utf-8", errors="surrogateescape")
        changed.append(record[3:])
        if record[:2].strip() in ("R", "C"):
            i += 1
            if i < len(records) and records[i]:
                changed.append(records[i].decode("utf-8", errors="surrogateescape"))
        i += 1
    return sorted(set(changed))


def covered(path: str, claimed: list[str]) -> bool:
    return any(path == scope or path.startswith(scope.rstrip("/") + "/")
               for scope in claimed if not scope.startswith("@shared/"))


def path_fingerprint(root: Path, raw: str) -> str:
    path = root / raw
    try:
        info = path.lstat()
    except FileNotFoundError:
        return "missing"
    mode = stat.S_IMODE(info.st_mode)
    if stat.S_ISLNK(info.st_mode):
        payload = b"symlink\0" + os.fsencode(os.readlink(path))
    elif stat.S_ISREG(info.st_mode):
        payload = b"file\0" + path.read_bytes()
    elif stat.S_ISDIR(info.st_mode):
        entries = []
        for child in sorted(path.rglob("*")):
            relative = child.relative_to(root).as_posix()
            if ".git" in Path(relative).parts:
                continue
            entries.append((relative, path_fingerprint(root, relative)))
        payload = json.dumps(entries, ensure_ascii=True, separators=(",", ":")).encode()
    else:
        payload = f"special\0{info.st_mode}\0{info.st_size}".encode()
    return hashlib.sha256(f"{mode}\0".encode() + payload).hexdigest()


def cmd_claim(args: argparse.Namespace, root: Path, db: sqlite3.Connection) -> None:
    worktree = Path(args.worktree).expanduser().resolve()
    if worktree != root:
        raise ValueError(f"--worktree must be the current Git worktree root ({root})")
    _, common = repo_identity()
    branch = validate_writer_worktree(root, common)
    paths = sorted({normalized_path(root, raw) for raw in args.paths})
    if not paths:
        raise ValueError("at least one file, directory, or @shared resource must be claimed")
    now = int(time.time())
    claim_id = hashlib.sha256(f"{args.thread}\0{args.task}\0{now}\0{os.urandom(16).hex()}".encode()).hexdigest()[:20]
    base_head = git(root, "rev-parse", "HEAD")
    db.execute("BEGIN IMMEDIATE")
    try:
        rows = db.execute("""
            SELECT c.id, c.task, c.thread, c.worktree, p.path, c.branch
            FROM claims c JOIN claimed_paths p ON p.claim_id=c.id
            WHERE c.state='active'
        """).fetchall()
        conflicts = [
            row for row in rows
            if row[2] == args.thread or row[3] == str(worktree)
            or (row[5] and row[5] == branch)
            or any(overlaps(path, row[4]) for path in paths)
        ]
        if conflicts:
            details = [
                {"claim_id": row[0], "task": row[1], "thread": row[2],
                 "worktree": row[3], "path": row[4]}
                for row in conflicts
            ]
            raise ValueError("claim conflicts; resolve the existing owner first: " + json.dumps(details))
        db.execute("""
            INSERT INTO claims(id,task,thread,worktree,branch,base_head,stage,milestone,next_action,
                               state,created_at,heartbeat_at,lease_until)
            VALUES(?,?,?,?,?,?,?,?,?,'active',?,?,?)
        """, (claim_id, args.task, args.thread, str(worktree), branch, base_head, args.stage,
              args.milestone, args.next_action, now, now, now + args.lease_seconds))
        db.executemany("INSERT INTO claimed_paths(claim_id,path) VALUES(?,?)",
                       [(claim_id, path) for path in paths])
        db.execute("COMMIT")
    except BaseException:
        db.execute("ROLLBACK")
        raise
    print(json.dumps({"claim_id": claim_id, "task": args.task, "thread": args.thread,
                      "worktree": str(worktree), "branch": branch, "stage": args.stage,
                      "milestone": args.milestone, "next_action": args.next_action,
                      "paths": paths, "lease_until": now + args.lease_seconds}))


def cmd_adopt(args: argparse.Namespace, root: Path, common: Path, db: sqlite3.Connection) -> None:
    """Claim exact task paths while fingerprinting unrelated pre-existing dirt."""
    worktree = Path(args.worktree).expanduser().resolve()
    if worktree != root:
        raise ValueError(f"--worktree must be the current Git worktree root ({root})")
    branch = validate_writer_worktree(root, common, require_clean=False)
    paths = sorted({normalized_path(root, raw) for raw in args.paths})
    if not paths or any(path.startswith("@shared/") for path in paths):
        raise ValueError("adoption requires exact repository-relative file or directory paths")
    existing_changes = changed_paths(root)
    if not existing_changes:
        raise ValueError("worktree has no changes to adopt; use claim before starting edits")
    now = int(time.time())
    claim_id = hashlib.sha256(
        f"{args.thread}\0{args.task}\0{now}\0{os.urandom(16).hex()}".encode()
    ).hexdigest()[:20]
    db.execute("BEGIN IMMEDIATE")
    try:
        rows = db.execute("""
            SELECT c.id,c.task,c.thread,c.worktree,p.path,c.branch
            FROM claims c JOIN claimed_paths p ON p.claim_id=c.id
            WHERE c.state='active'
        """).fetchall()
        conflicts = [row for row in rows if row[2] == args.thread or row[3] == str(root)
                     or (row[5] and row[5] == branch)
                     or any(overlaps(path, row[4]) for path in paths)]
        if conflicts:
            details = [{"claim_id": row[0], "task": row[1], "thread": row[2],
                        "worktree": row[3], "path": row[4]} for row in conflicts]
            raise ValueError("adoption conflicts; resolve the existing owner first: "
                             + json.dumps(details))
        baseline = sorted({normalized_path(root, raw) for raw in existing_changes
                           if not covered(normalized_path(root, raw), paths)})
        fingerprints = [(claim_id, path, path_fingerprint(root, path)) for path in baseline]
        base_head = git(root, "rev-parse", "HEAD")
        db.execute("""
            INSERT INTO claims(id,task,thread,worktree,branch,base_head,stage,milestone,next_action,
                               state,created_at,heartbeat_at,lease_until)
            VALUES(?,?,?,?,?,?,?,?,?,'active',?,?,?)
        """, (claim_id,args.task,args.thread,str(root),branch,base_head,args.stage,args.milestone,
              args.next_action,now,now,now + args.lease_seconds))
        db.executemany("INSERT INTO claimed_paths(claim_id,path) VALUES(?,?)",
                       [(claim_id,path) for path in paths])
        db.executemany("INSERT INTO claim_baselines(claim_id,path,fingerprint) VALUES(?,?,?)",
                       fingerprints)
        db.execute("COMMIT")
    except BaseException:
        db.execute("ROLLBACK")
        raise
    print(json.dumps({"claim_id": claim_id,"task": args.task,"thread": args.thread,
                      "worktree": str(root),"branch": branch,"stage": args.stage,
                      "milestone": args.milestone,"next_action": args.next_action,
                      "adopted_paths": paths,"baseline_paths": baseline,
                      "lease_until": now + args.lease_seconds}))


def cmd_heartbeat(args: argparse.Namespace, db: sqlite3.Connection) -> None:
    now = int(time.time())
    db.execute("BEGIN IMMEDIATE")
    try:
        result = db.execute("""
            UPDATE claims SET heartbeat_at=?, lease_until=?
            WHERE id=? AND thread=? AND state='active'
        """, (now, now + args.lease_seconds, args.claim_id, args.thread))
        if result.rowcount != 1:
            raise ValueError("claim is missing, released, or owned by another thread")
        db.execute("COMMIT")
    except BaseException:
        db.execute("ROLLBACK")
        raise
    print(json.dumps({"claim_id": args.claim_id, "state": "active", "lease_until": now + args.lease_seconds}))


def cmd_progress(args: argparse.Namespace, db: sqlite3.Connection) -> None:
    db.execute("BEGIN IMMEDIATE")
    try:
        result = db.execute("""
            UPDATE claims SET stage=?,milestone=?,next_action=?,heartbeat_at=?
            WHERE id=? AND thread=? AND state='active'
        """, (args.stage, args.milestone, args.next_action, int(time.time()),
              args.claim_id, args.thread))
        if result.rowcount != 1:
            raise ValueError("claim is missing, released, or owned by another thread")
        db.execute("COMMIT")
    except BaseException:
        db.execute("ROLLBACK")
        raise
    print(json.dumps({"claim_id": args.claim_id, "stage": args.stage,
                      "milestone": args.milestone, "next_action": args.next_action}))


def cmd_release(args: argparse.Namespace, db: sqlite3.Connection) -> None:
    db.execute("BEGIN IMMEDIATE")
    try:
        result = db.execute("""
            UPDATE claims SET state='released', release_state=?, evidence=?
            WHERE id=? AND thread=? AND state='active'
        """, (args.terminal_state, args.evidence, args.claim_id, args.thread))
        if result.rowcount != 1:
            raise ValueError("claim is missing, released, or owned by another thread")
        db.execute("COMMIT")
    except BaseException:
        db.execute("ROLLBACK")
        raise
    print(json.dumps({"claim_id": args.claim_id, "state": "released", "terminal_state": args.terminal_state}))


def cmd_handoff(args: argparse.Namespace, common_git_dir: Path, db: sqlite3.Connection) -> None:
    new_worktree = Path(args.new_worktree).expanduser().resolve()
    new_root = Path(subprocess.check_output(
        ["git", "-C", str(new_worktree), "rev-parse", "--show-toplevel"],
        text=True, stderr=subprocess.PIPE,
    ).strip()).resolve()
    common = subprocess.check_output(
        ["git", "-C", str(new_root), "rev-parse", "--git-common-dir"],
        text=True, stderr=subprocess.PIPE,
    ).strip()
    common_path = Path(common)
    if not common_path.is_absolute():
        common_path = new_root / common_path
    if common_path.resolve() != common_git_dir:
        raise ValueError("new worktree belongs to a different Git repository")
    branch = validate_writer_worktree(new_root, common_git_dir)
    now = int(time.time())
    db.execute("BEGIN IMMEDIATE")
    try:
        old = db.execute("SELECT task FROM claims WHERE id=? AND thread=? AND state='active'",
                         (args.claim_id, args.old_thread)).fetchone()
        if old is None:
            raise ValueError("claim is missing, released, or not owned by the old thread")
        conflict = db.execute("""
            SELECT id,thread FROM claims
            WHERE state='active' AND (worktree=? OR thread=? OR branch=?) AND id<>?
        """, (str(new_root), args.new_thread, branch, args.claim_id)).fetchone()
        if conflict:
            raise ValueError(f"new worktree is already claimed by thread {conflict[1]} ({conflict[0]})")
        db.execute("""
            UPDATE claims SET task=?,thread=?,worktree=?,branch=?,stage=?,milestone=?,next_action=?,
                              heartbeat_at=?,lease_until=?,handoff_terminal_state=?,evidence=?
            WHERE id=? AND thread=? AND state='active'
        """, (args.task or old[0], args.new_thread, str(new_root), branch, args.stage,
              args.milestone, args.next_action, now, now + args.lease_seconds,
              args.terminal_state,
              f"handoff from terminal thread {args.old_thread}: {args.evidence}",
              args.claim_id, args.old_thread))
        db.execute("COMMIT")
    except BaseException:
        db.execute("ROLLBACK")
        raise
    print(json.dumps({"claim_id": args.claim_id, "state": "active", "thread": args.new_thread,
                      "worktree": str(new_root), "branch": branch,
                      "terminal_state": args.terminal_state,
                      "lease_until": now + args.lease_seconds}))


def cmd_audit(args: argparse.Namespace, root: Path, db: sqlite3.Connection) -> None:
    row = db.execute("""
        SELECT worktree,thread,branch,base_head,stage,milestone,next_action
        FROM claims WHERE id=? AND state='active'
    """, (args.claim_id,)).fetchone()
    if row is None or row[1] != args.thread:
        raise ValueError("claim is missing, released, or owned by another thread")
    if Path(row[0]).resolve() != root:
        raise ValueError("run audit from the worktree recorded by the claim")
    if git(root, "branch", "--show-current") != row[2]:
        raise ValueError("current branch differs from the branch recorded by the claim")
    paths = [item[0] for item in db.execute(
        "SELECT path FROM claimed_paths WHERE claim_id=?", (args.claim_id,)
    )]
    baseline = dict(db.execute(
        "SELECT path,fingerprint FROM claim_baselines WHERE claim_id=?", (args.claim_id,)
    ))
    changed = changed_paths(root)
    base_head = row[3] or git(root, "rev-parse", "HEAD")
    committed = subprocess.check_output(
        ["git", "-C", str(root), "diff", "--name-only", "-z", base_head, "HEAD"],
        stderr=subprocess.PIPE,
    ).split(b"\0")
    changed = sorted(set(changed + [path.decode("utf-8", errors="surrogateescape")
                                    for path in committed if path]))
    outside = []
    for path in changed:
        normalized = normalized_path(root, path)
        if normalized.startswith("@shared/") or covered(normalized, paths):
            continue
        if normalized in baseline and path_fingerprint(root, normalized) == baseline[normalized]:
            continue
        outside.append(path)
    if outside:
        raise ValueError("changed files outside this claim: " + json.dumps(outside))
    print(json.dumps({"claim_id": args.claim_id, "state": "within-claim",
                      "changed_paths": changed, "stage": row[4],
                      "milestone": row[5], "next_action": row[6]}))


def cmd_list(args: argparse.Namespace, db: sqlite3.Connection) -> None:
    now = int(time.time())
    rows = db.execute("""
        SELECT c.id,c.task,c.thread,c.worktree,c.branch,c.stage,c.milestone,c.next_action,
               c.state,c.heartbeat_at,c.lease_until,c.release_state,c.handoff_terminal_state,
               c.evidence,p.path
        FROM claims c LEFT JOIN claimed_paths p ON p.claim_id=c.id
        ORDER BY c.created_at,c.id
    """).fetchall()
    claims: dict[str, dict[str, object]] = {}
    for row in rows:
        item = claims.setdefault(row[0], {
            "claim_id": row[0], "task": row[1], "thread": row[2], "worktree": row[3],
            "branch": row[4], "stage": row[5], "milestone": row[6], "next_action": row[7],
            "state": ("suspect" if row[8] == "active" and row[10] < now else row[8]),
            "heartbeat_at": row[9], "lease_until": row[10], "terminal_state": row[11],
            "handoff_terminal_state": row[12], "evidence": row[13], "paths": [],
        })
        if row[14] is not None:
            item["paths"].append(row[14])
    for claim_id, path in db.execute(
        "SELECT claim_id,path FROM claim_baselines ORDER BY claim_id,path"
    ):
        if claim_id in claims:
            claims[claim_id].setdefault("baseline_paths", []).append(path)
    print(json.dumps(list(claims.values()), indent=2))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    claim = sub.add_parser("claim", help="atomically claim a unique worktree and paths")
    claim.add_argument("--task", required=True)
    claim.add_argument("--thread", required=True, help="Codex thread ID owning the work")
    claim.add_argument("--worktree", required=True, help="the isolated Git worktree used by this thread")
    claim.add_argument("--stage", required=True)
    claim.add_argument("--milestone", required=True)
    claim.add_argument("--next-action", required=True)
    claim.add_argument("--lease-seconds", type=int, default=900)
    claim.add_argument("paths", nargs="+")
    adopt = sub.add_parser("adopt", help="claim assigned paths and baseline existing unrelated dirt")
    adopt.add_argument("--task", required=True)
    adopt.add_argument("--thread", required=True)
    adopt.add_argument("--worktree", required=True)
    adopt.add_argument("--stage", required=True)
    adopt.add_argument("--milestone", required=True)
    adopt.add_argument("--next-action", required=True)
    adopt.add_argument("--lease-seconds", type=int, default=900)
    adopt.add_argument("paths", nargs="+")
    heartbeat = sub.add_parser("heartbeat", help="renew an owner-held claim; expiry never releases it")
    heartbeat.add_argument("--claim-id", required=True)
    heartbeat.add_argument("--thread", required=True)
    heartbeat.add_argument("--lease-seconds", type=int, default=900)
    progress = sub.add_parser("progress", help="update the current stage, milestone, and next action")
    progress.add_argument("--claim-id", required=True)
    progress.add_argument("--thread", required=True)
    progress.add_argument("--stage", required=True)
    progress.add_argument("--milestone", required=True)
    progress.add_argument("--next-action", required=True)
    release = sub.add_parser("release", help="release only after confirming the old thread is terminal")
    release.add_argument("--claim-id", required=True)
    release.add_argument("--thread", required=True)
    release.add_argument("--terminal-state", required=True,
                         choices=("completed", "cancelled", "archived", "superseded"))
    release.add_argument("--evidence", required=True,
                         help="link or concise coordinator evidence that the thread stopped")
    handoff = sub.add_parser("handoff", help="atomically transfer paths after verifying the old thread stopped")
    handoff.add_argument("--claim-id", required=True)
    handoff.add_argument("--old-thread", required=True)
    handoff.add_argument("--new-thread", required=True)
    handoff.add_argument("--new-worktree", required=True)
    handoff.add_argument("--task", help="new task label; defaults to the existing task")
    handoff.add_argument("--stage", required=True)
    handoff.add_argument("--milestone", required=True)
    handoff.add_argument("--next-action", required=True)
    handoff.add_argument("--terminal-state", required=True,
                         choices=("completed", "cancelled", "archived", "superseded"))
    handoff.add_argument("--lease-seconds", type=int, default=900)
    handoff.add_argument("--evidence", required=True,
                         help="evidence that the old thread is terminal and handoff is authorized")
    audit = sub.add_parser("audit", help="ensure all worktree changes stay within the active claim")
    audit.add_argument("--claim-id", required=True)
    audit.add_argument("--thread", required=True)
    sub.add_parser("list", help="list active, suspect, and released claims")
    args = parser.parse_args()
    try:
        for name in ("task", "thread", "stage", "milestone", "next_action") if args.command in ("claim", "adopt") else ():
            if not getattr(args, name).strip():
                raise ValueError(f"{name} must not be empty")
        if args.command in ("release", "handoff") and not args.evidence.strip():
            raise ValueError("evidence must not be empty")
        if args.command == "progress" and not all(
            (args.stage.strip(), args.milestone.strip(), args.next_action.strip())
        ):
            raise ValueError("stage, milestone, and next action must not be empty")
        root, common = repo_identity()
        db = connect(database_path(common))
        if args.command == "claim":
            if args.lease_seconds < 60:
                raise ValueError("lease must be at least 60 seconds")
            cmd_claim(args, root, db)
        elif args.command == "adopt":
            if args.lease_seconds < 60:
                raise ValueError("lease must be at least 60 seconds")
            cmd_adopt(args, root, common, db)
        elif args.command == "heartbeat":
            if args.lease_seconds < 60:
                raise ValueError("lease must be at least 60 seconds")
            cmd_heartbeat(args, db)
        elif args.command == "progress":
            cmd_progress(args, db)
        elif args.command == "release":
            cmd_release(args, db)
        elif args.command == "handoff":
            if args.old_thread == args.new_thread:
                raise ValueError("handoff requires a different new thread ID")
            if args.lease_seconds < 60:
                raise ValueError("lease must be at least 60 seconds")
            cmd_handoff(args, common, db)
        elif args.command == "audit":
            cmd_audit(args, root, db)
        else:
            cmd_list(args, db)
        return 0
    except (OSError, sqlite3.Error, subprocess.CalledProcessError, ValueError) as exc:
        print(f"roadmap_claims: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
