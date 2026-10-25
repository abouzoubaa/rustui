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
import unicodedata
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
    # Codex sessions can inherit different XDG_STATE_HOME values. A repository
    # claim ledger must not split across those environments, so pin it to the
    # user's stable state directory.
    base = Path.home() / ".local/state"
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
        CREATE TABLE IF NOT EXISTS claim_transitions (
            sequence INTEGER PRIMARY KEY AUTOINCREMENT,
            claim_id TEXT NOT NULL REFERENCES claims(id),
            kind TEXT NOT NULL,
            from_task TEXT NOT NULL,
            to_task TEXT NOT NULL,
            from_thread TEXT NOT NULL,
            to_thread TEXT NOT NULL,
            from_stage TEXT NOT NULL,
            to_stage TEXT NOT NULL,
            from_milestone TEXT NOT NULL,
            to_milestone TEXT NOT NULL,
            from_base_head TEXT NOT NULL DEFAULT '',
            to_base_head TEXT NOT NULL DEFAULT '',
            added_paths_json TEXT NOT NULL,
            next_action TEXT NOT NULL,
            terminal_state TEXT,
            evidence TEXT,
            occurred_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS claims_state_idx ON claims(state);
        CREATE UNIQUE INDEX IF NOT EXISTS claims_active_task_idx ON claims(task)
            WHERE state='active';
        CREATE INDEX IF NOT EXISTS claimed_paths_path_idx ON claimed_paths(path);
        CREATE INDEX IF NOT EXISTS claim_transitions_claim_idx ON claim_transitions(claim_id, sequence);
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
        transition_columns = {
            row[1] for row in db.execute("PRAGMA table_info(claim_transitions)")
        }
        for name, definition in (
            ("from_base_head", "TEXT NOT NULL DEFAULT ''"),
            ("to_base_head", "TEXT NOT NULL DEFAULT ''"),
            ("terminal_state", "TEXT"),
            ("evidence", "TEXT"),
        ):
            if name not in transition_columns:
                db.execute(f"ALTER TABLE claim_transitions ADD COLUMN {name} {definition}")
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


def record_transition(
    db: sqlite3.Connection,
    claim_id: str,
    kind: str,
    from_task: str,
    to_task: str,
    from_thread: str,
    to_thread: str,
    from_stage: str,
    to_stage: str,
    from_milestone: str,
    to_milestone: str,
    added_paths: list[str],
    next_action: str,
    occurred_at: int,
    from_base_head: str = "",
    to_base_head: str = "",
    terminal_state: str | None = None,
    evidence: str | None = None,
) -> None:
    db.execute(
        """
        INSERT INTO claim_transitions(
            claim_id,kind,from_task,to_task,from_thread,to_thread,from_stage,to_stage,
            from_milestone,to_milestone,from_base_head,to_base_head,added_paths_json,
            next_action,terminal_state,evidence,occurred_at
        ) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)
        """,
        (
            claim_id,
            kind,
            from_task,
            to_task,
            from_thread,
            to_thread,
            from_stage,
            to_stage,
            from_milestone,
            to_milestone,
            from_base_head,
            to_base_head,
            json.dumps(added_paths),
            next_action,
            terminal_state,
            evidence,
            occurred_at,
        ),
    )


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
    normalized = unicodedata.normalize("NFD", relative)
    return normalized.casefold() if sys.platform == "darwin" else normalized


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
            if row[1] == args.task or row[2] == args.thread or row[3] == str(worktree)
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
    if not paths:
        raise ValueError("adoption requires exact repository-relative file, directory, or @shared paths")
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
        conflicts = [row for row in rows if row[1] == args.task or row[2] == args.thread or row[3] == str(root)
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
    now = int(time.time())
    db.execute("BEGIN IMMEDIATE")
    try:
        result = db.execute("""
            UPDATE claims SET stage=?,milestone=?,next_action=?,heartbeat_at=?,lease_until=?
            WHERE id=? AND thread=? AND state='active'
        """, (args.stage, args.milestone, args.next_action, now, now + args.lease_seconds,
              args.claim_id, args.thread))
        if result.rowcount != 1:
            raise ValueError("claim is missing, released, or owned by another thread")
        db.execute("COMMIT")
    except BaseException:
        db.execute("ROLLBACK")
        raise
    print(json.dumps({"claim_id": args.claim_id, "stage": args.stage,
                      "milestone": args.milestone, "next_action": args.next_action,
                      "lease_until": now + args.lease_seconds}))


def cmd_advance(args: argparse.Namespace, root: Path, db: sqlite3.Connection) -> None:
    """Keep the current owner while atomically changing task metadata and extending paths."""
    requested_paths = sorted({normalized_path(root, raw) for raw in args.paths})
    now = int(time.time())
    db.execute("BEGIN IMMEDIATE")
    try:
        current = db.execute(
            """
            SELECT task,thread,worktree,branch,base_head,stage,milestone
            FROM claims WHERE id=? AND state='active'
            """,
            (args.claim_id,),
        ).fetchone()
        if current is None or current[1] != args.thread:
            raise ValueError("claim missing, released, or owned by another thread")
        if Path(current[2]).resolve() != root:
            raise ValueError("run advance from the worktree recorded by the claim")
        branch = git(root, "branch", "--show-current")
        if branch != current[3]:
            raise ValueError("current branch differs from the branch recorded by the claim")

        current_paths = [row[0] for row in db.execute(
            "SELECT path FROM claimed_paths WHERE claim_id=?", (args.claim_id,)
        )]
        added_paths = [
            path for path in requested_paths
            if path not in current_paths and not covered(path, current_paths)
        ]
        task = args.task.strip() if args.task is not None else current[0]
        if not task.strip():
            raise ValueError("task must not be empty")
        if task == current[0] and not added_paths:
            raise ValueError("advance requires a new task ID or at least one additional path")

        changed = set(changed_paths(root))
        base_head = current[4] or git(root, "rev-parse", "HEAD")
        committed = subprocess.check_output(
            ["git", "-C", str(root), "diff", "--name-only", "-z", base_head, "HEAD"],
            stderr=subprocess.PIPE,
        ).split(b"\0")
        changed.update(path.decode("utf-8", errors="surrogateescape")
                       for path in committed if path)
        dirty_added_paths = sorted({
            normalized_path(root, path)
            for path in changed
            if not path.startswith("@shared/")
            and not covered(normalized_path(root, path), current_paths)
            and any(overlaps(normalized_path(root, path), added) for added in added_paths)
        })
        if dirty_added_paths:
            raise ValueError(
                "new paths already contain changes; claim them before editing or use the audited adoption flow: "
                + json.dumps(dirty_added_paths)
            )

        rows = db.execute("""
            SELECT c.id,c.task,c.thread,c.worktree,p.path,c.branch
            FROM claims c JOIN claimed_paths p ON p.claim_id=c.id
            WHERE c.state='active' AND c.id<>?
        """, (args.claim_id,)).fetchall()
        conflicts = [
            {"claim_id": row[0], "task": row[1], "thread": row[2],
             "worktree": row[3], "path": row[4]}
            for row in rows
            if row[1] == task or row[2] == args.thread or row[3] == str(root)
            or (row[5] and row[5] == branch)
            or any(overlaps(added, row[4]) for added in added_paths)
        ]
        if conflicts:
            raise ValueError("advance conflicts with active claims: " + json.dumps(conflicts))

        result = db.execute("""
            UPDATE claims
            SET task=?,stage=?,milestone=?,next_action=?,heartbeat_at=?,lease_until=?
            WHERE id=? AND thread=? AND state='active'
        """, (task, args.stage, args.milestone, args.next_action, now,
              now + args.lease_seconds, args.claim_id, args.thread))
        if result.rowcount != 1:
            raise ValueError("claim changed while advancing; retry after re-reading the ledger")
        db.executemany(
            "INSERT INTO claimed_paths(claim_id,path) VALUES(?,?)",
            [(args.claim_id, path) for path in added_paths],
        )
        record_transition(
            db, args.claim_id, "advance", current[0], task, args.thread, args.thread,
            current[5], args.stage, current[6], args.milestone, added_paths,
            args.next_action, now, base_head, base_head,
        )
        db.execute("COMMIT")
    except BaseException:
        db.execute("ROLLBACK")
        raise

    all_paths = sorted(set(current_paths) | set(added_paths))
    print(json.dumps({
        "claim_id": args.claim_id,
        "from_task": current[0],
        "task": task,
        "thread": args.thread,
        "added_paths": added_paths,
        "paths": all_paths,
        "lease_until": now + args.lease_seconds,
    }))


def cmd_rescope(args: argparse.Namespace, root: Path, db: sqlite3.Connection) -> None:
    """Atomically replace an owner's paths when removed scope has no work."""
    requested_paths = sorted({normalized_path(root, raw) for raw in args.paths})
    if not requested_paths:
        raise ValueError("rescope requires at least one exact path")
    now = int(time.time())
    db.execute("BEGIN IMMEDIATE")
    try:
        current = db.execute(
            "SELECT task,thread,worktree,branch,base_head,stage,milestone FROM claims "
            "WHERE id=? AND state='active'", (args.claim_id,),
        ).fetchone()
        if current is None or current[1] != args.thread:
            raise ValueError("claim missing, released, or owned by another thread")
        if Path(current[2]).resolve() != root:
            raise ValueError("run rescope from the worktree recorded by the claim")
        branch = git(root, "branch", "--show-current")
        if branch != current[3]:
            raise ValueError("current branch differs from the branch recorded by the claim")

        old_paths = [row[0] for row in db.execute(
            "SELECT path FROM claimed_paths WHERE claim_id=?", (args.claim_id,)
        )]
        task = args.task.strip() if args.task is not None else current[0]
        if not task:
            raise ValueError("task must not be empty")
        if task == current[0] and requested_paths == sorted(old_paths):
            raise ValueError("rescope requires a changed task ID or path set")

        changed = set(changed_paths(root))
        base_head = current[4] or git(root, "rev-parse", "HEAD")
        committed = subprocess.check_output(
            ["git", "-C", str(root), "diff", "--name-only", "-z", base_head, "HEAD"],
            stderr=subprocess.PIPE,
        ).split(b"\0")
        normalized_changes = {
            normalized_path(root, path)
            for path in changed
        }
        normalized_changes.update(
            normalized_path(root, path.decode("utf-8", errors="surrogateescape"))
            for path in committed if path
        )
        baseline = dict(db.execute(
            "SELECT path,fingerprint FROM claim_baselines WHERE claim_id=?",
            (args.claim_id,),
        ))

        dropped_changes = sorted(
            path for path in normalized_changes
            if covered(path, old_paths) and not covered(path, requested_paths)
            and not (path in baseline and path_fingerprint(root, path) == baseline[path])
        )
        if dropped_changes:
            raise ValueError(
                "cannot drop claimed paths containing changes: " + json.dumps(dropped_changes)
            )

        added_paths = [path for path in requested_paths if not covered(path, old_paths)]
        dirty_added_paths = sorted({
            path for path in normalized_changes
            if not covered(path, old_paths)
            and any(overlaps(path, added) for added in added_paths)
        })
        if dirty_added_paths:
            raise ValueError(
                "new paths already contain changes; adopt them before rescoping: "
                + json.dumps(dirty_added_paths)
            )

        rows = db.execute("""
            SELECT c.id,c.task,c.thread,c.worktree,p.path,c.branch
            FROM claims c JOIN claimed_paths p ON p.claim_id=c.id
            WHERE c.state='active' AND c.id<>?
        """, (args.claim_id,)).fetchall()
        conflicts = [
            {"claim_id": row[0], "task": row[1], "thread": row[2],
             "worktree": row[3], "path": row[4]}
            for row in rows
            if row[1] == task or row[2] == args.thread or row[3] == str(root)
            or (row[5] and row[5] == branch)
            or any(overlaps(path, row[4]) for path in requested_paths)
        ]
        if conflicts:
            raise ValueError("rescope conflicts with active claims: " + json.dumps(conflicts))

        result = db.execute("""
            UPDATE claims
            SET task=?,stage=?,milestone=?,next_action=?,heartbeat_at=?,lease_until=?
            WHERE id=? AND thread=? AND state='active'
        """, (task, args.stage, args.milestone, args.next_action, now,
              now + args.lease_seconds, args.claim_id, args.thread))
        if result.rowcount != 1:
            raise ValueError("claim changed while rescoping; retry after re-reading the ledger")
        db.execute("DELETE FROM claimed_paths WHERE claim_id=?", (args.claim_id,))
        db.executemany(
            "INSERT INTO claimed_paths(claim_id,path) VALUES(?,?)",
            [(args.claim_id, path) for path in requested_paths],
        )
        removed_paths = [path for path in old_paths if not covered(path, requested_paths)]
        record_transition(
            db, args.claim_id, "rescope", current[0], task, args.thread, args.thread,
            current[5], args.stage, current[6], args.milestone, added_paths,
            args.next_action, now, base_head, base_head,
            evidence=json.dumps({"removed_paths": removed_paths}, sort_keys=True),
        )
        db.execute("COMMIT")
    except BaseException:
        db.execute("ROLLBACK")
        raise
    print(json.dumps({"claim_id": args.claim_id, "task": task, "thread": args.thread,
                      "paths": requested_paths, "removed_paths": removed_paths,
                      "lease_until": now + args.lease_seconds}))


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
    new_base_head = git(new_root, "rev-parse", "HEAD")
    now = int(time.time())
    db.execute("BEGIN IMMEDIATE")
    try:
        old = db.execute("SELECT task,stage,milestone,base_head FROM claims WHERE id=? AND thread=? AND state='active'",
                         (args.claim_id, args.old_thread)).fetchone()
        if old is None:
            raise ValueError("claim is missing, released, or not owned by the old thread")
        conflict = db.execute("""
            SELECT id,thread FROM claims
            WHERE state='active' AND (worktree=? OR thread=? OR branch=? OR task=?) AND id<>?
        """, (str(new_root), args.new_thread, branch, args.task or old[0],
              args.claim_id)).fetchone()
        if conflict:
            raise ValueError(f"new worktree is already claimed by thread {conflict[1]} ({conflict[0]})")
        db.execute("""
            UPDATE claims SET task=?,thread=?,worktree=?,branch=?,base_head=?,stage=?,milestone=?,next_action=?,
                              heartbeat_at=?,lease_until=?,handoff_terminal_state=?,evidence=?
            WHERE id=? AND thread=? AND state='active'
        """, (args.task or old[0], args.new_thread, str(new_root), branch, new_base_head, args.stage,
              args.milestone, args.next_action, now, now + args.lease_seconds,
              args.terminal_state,
              f"handoff from terminal thread {args.old_thread}: {args.evidence}",
              args.claim_id, args.old_thread))
        record_transition(
            db, args.claim_id, "handoff", old[0], args.task or old[0],
            args.old_thread, args.new_thread, old[1], args.stage, old[2],
            args.milestone, [], args.next_action, now, old[3], new_base_head,
            args.terminal_state, args.evidence,
        )
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
    history = [
        {
            "sequence": item[0],
            "kind": item[1],
            "from_task": item[2],
            "to_task": item[3],
            "from_thread": item[4],
            "to_thread": item[5],
            "from_stage": item[6],
            "to_stage": item[7],
            "from_milestone": item[8],
            "to_milestone": item[9],
            "from_base_head": item[10],
            "to_base_head": item[11],
            "added_paths": json.loads(item[12]),
            "next_action": item[13],
            "terminal_state": item[14],
            "evidence": item[15],
            "occurred_at": item[16],
        }
        for item in db.execute(
            """
            SELECT sequence,kind,from_task,to_task,from_thread,to_thread,from_stage,to_stage,
                   from_milestone,to_milestone,from_base_head,to_base_head,added_paths_json,
                   next_action,terminal_state,evidence,occurred_at
            FROM claim_transitions WHERE claim_id=? ORDER BY sequence
            """,
            (args.claim_id,),
        )
    ]
    print(json.dumps({"claim_id": args.claim_id, "state": "within-claim",
                      "changed_paths": changed, "stage": row[4],
                      "milestone": row[5], "next_action": row[6], "history": history}))


def cmd_baseline(args: argparse.Namespace, root: Path, db: sqlite3.Connection) -> None:
    claim = db.execute(
        """SELECT worktree,thread,branch,task,stage,milestone,next_action,base_head
           FROM claims WHERE id=? AND state='active'""",
        (args.claim_id,),
    ).fetchone()
    if claim is None or claim[1] != args.thread:
        raise ValueError("claim is missing, released, or owned by another thread")
    if Path(claim[0]).resolve() != root:
        raise ValueError("run baseline in the worktree recorded by the claim")
    if git(root, "branch", "--show-current") != claim[2]:
        raise ValueError("current branch differs from branch recorded by claim")

    paths = sorted({normalized_path(root, raw) for raw in args.paths})
    if any(path.startswith("@shared/") for path in paths):
        raise ValueError("baseline accepts repository files, not shared resources")

    db.execute("BEGIN IMMEDIATE")
    try:
        current = db.execute(
            """SELECT worktree,thread,branch,task,stage,milestone,next_action,base_head
               FROM claims WHERE id=? AND state='active'""",
            (args.claim_id,),
        ).fetchone()
        if current != claim:
            raise ValueError("claim ownership changed while preparing baseline")
        owned_paths = [
            row[0]
            for row in db.execute(
                "SELECT path FROM claimed_paths WHERE claim_id=?", (args.claim_id,)
            )
        ]
        if any(overlaps(path, owned) for path in paths for owned in owned_paths):
            raise ValueError("claimed paths cannot also be recorded as background baseline")

        changes = {normalized_path(root, raw) for raw in changed_paths(root)}
        missing = [path for path in paths if path not in changes]
        if missing:
            raise ValueError("baseline paths must already be changed: " + json.dumps(missing))

        active_paths = db.execute(
            """SELECT c.id,c.thread,p.path FROM claims c
               JOIN claimed_paths p ON p.claim_id=c.id
               WHERE c.state='active' AND c.id<>?""",
            (args.claim_id,),
        ).fetchall()
        conflicts = [
            {"claim_id": claim_id, "thread": thread, "path": claimed}
            for path in paths
            for claim_id, thread, claimed in active_paths
            if overlaps(path, claimed)
        ]
        if conflicts:
            raise ValueError("baseline conflicts with active claim: " + json.dumps(conflicts))

        existing = dict(
            db.execute(
                "SELECT path,fingerprint FROM claim_baselines WHERE claim_id=?",
                (args.claim_id,),
            )
        )
        fingerprints = {path: path_fingerprint(root, path) for path in paths}
        changed_existing = [
            path for path in paths if path in existing and existing[path] != fingerprints[path]
        ]
        if changed_existing:
            raise ValueError(
                "existing baseline fingerprints differ: " + json.dumps(changed_existing)
            )
        added = [path for path in paths if path not in existing]
        db.executemany(
            "INSERT INTO claim_baselines(claim_id,path,fingerprint) VALUES(?,?,?)",
            [(args.claim_id, path, fingerprints[path]) for path in added],
        )
        if added:
            record_transition(
                db,
                args.claim_id,
                "baseline",
                claim[3],
                claim[3],
                args.thread,
                args.thread,
                claim[4],
                claim[4],
                claim[5],
                claim[5],
                added,
                claim[6],
                int(time.time()),
                claim[7],
                claim[7],
                evidence=args.evidence,
            )
        else:
            baseline_event = db.execute(
                """SELECT evidence FROM claim_transitions
                   WHERE claim_id=? AND kind='baseline' AND added_paths_json=?
                   ORDER BY sequence DESC LIMIT 1""",
                (args.claim_id, json.dumps(paths)),
            ).fetchone()
            confirmation = f"{args.evidence}; verified paths: {json.dumps(paths)}"
            if baseline_event is not None and baseline_event[0] is None:
                db.execute(
                    """UPDATE claim_transitions SET evidence=?
                       WHERE claim_id=? AND kind='baseline' AND added_paths_json=?
                         AND evidence IS NULL""",
                    (args.evidence, args.claim_id, json.dumps(paths)),
                )
            elif baseline_event is None and db.execute(
                """SELECT 1 FROM claim_transitions
                   WHERE claim_id=? AND kind='baseline-confirmed' AND evidence=?""",
                (args.claim_id, confirmation),
            ).fetchone() is None:
                record_transition(
                    db,
                    args.claim_id,
                    "baseline-confirmed",
                    claim[3],
                    claim[3],
                    args.thread,
                    args.thread,
                    claim[4],
                    claim[4],
                    claim[5],
                    claim[5],
                    [],
                    claim[6],
                    int(time.time()),
                    claim[7],
                    claim[7],
                    evidence=confirmation,
                )
        db.execute("COMMIT")
    except BaseException:
        db.execute("ROLLBACK")
        raise

    print(json.dumps({"claim_id": args.claim_id, "baseline_paths": added}))


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
            "baseline_path_count": 0,
        })
        if row[14] is not None:
            item["paths"].append(row[14])
    for claim_id, path in db.execute(
        "SELECT claim_id,path FROM claim_baselines ORDER BY claim_id,path"
    ):
        if claim_id in claims:
            claims[claim_id]["baseline_path_count"] += 1
            if args.verbose:
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
    progress.add_argument("--lease-seconds", type=int, default=900)
    advance = sub.add_parser(
        "advance", help="atomically advance an owner-held task and add exact paths without releasing the claim"
    )
    advance.add_argument("--claim-id", required=True)
    advance.add_argument("--thread", required=True)
    advance.add_argument("--task", help="new task ID; defaults to the current task")
    advance.add_argument("--stage", required=True)
    advance.add_argument("--milestone", required=True)
    advance.add_argument("--next-action", required=True)
    advance.add_argument("--lease-seconds", type=int, default=900)
    advance.add_argument("paths", nargs="*", help="additional paths to add to the existing claim")
    rescope = sub.add_parser(
        "rescope", help="atomically replace an owner's paths when dropped paths have no work"
    )
    rescope.add_argument("--claim-id", required=True)
    rescope.add_argument("--thread", required=True)
    rescope.add_argument("--task", help="new task ID; defaults to the current task")
    rescope.add_argument("--stage", required=True)
    rescope.add_argument("--milestone", required=True)
    rescope.add_argument("--next-action", required=True)
    rescope.add_argument("--lease-seconds", type=int, default=900)
    rescope.add_argument("paths", nargs="+", help="complete replacement set of exact paths")
    release = sub.add_parser("release", help="release only after confirming the old thread is terminal")
    release.add_argument("--claim-id", required=True)
    release.add_argument("--thread", required=True)
    release.add_argument("--terminal-state", required=True,
                         choices=("completed", "cancelled", "archived", "superseded"))
    release.add_argument("--evidence", required=True,
                         help="link or concise coordinator evidence that the thread stopped")
    handoff = sub.add_parser("handoff", help="atomically transfer paths after confirming the old thread stopped")
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
    handoff.add_argument("--confirmed-terminal", action="store_true",
                         help="attest that the old thread was independently checked and is terminal")
    handoff.add_argument("--lease-seconds", type=int, default=900)
    handoff.add_argument("--evidence", required=True,
                         help="evidence that the old thread is terminal and handoff is authorized")
    audit = sub.add_parser("audit", help="ensure all worktree changes stay within the active claim")
    audit.add_argument("--claim-id", required=True)
    audit.add_argument("--thread", required=True)
    baseline = sub.add_parser(
        "baseline", help="fingerprint explicit pre-existing changes outside an owner-held claim"
    )
    baseline.add_argument("--claim-id", required=True)
    baseline.add_argument("--thread", required=True)
    baseline.add_argument("--evidence", required=True, help="why these changes predate the claim")
    baseline.add_argument("paths", nargs="+", help="currently changed repository paths to preserve")
    listing = sub.add_parser("list", help="list active, suspect, and released claims")
    listing.add_argument("--verbose", action="store_true", help="include every baseline path")
    args = parser.parse_args()
    try:
        if args.command == "baseline" and not args.evidence.strip():
            raise ValueError("baseline evidence must not be empty")
        for name in ("task", "thread", "stage", "milestone", "next_action") if args.command in ("claim", "adopt") else ():
            if not getattr(args, name).strip():
                raise ValueError(f"{name} must not be empty")
        if args.command in ("release", "handoff") and not args.evidence.strip():
            raise ValueError("evidence must not be empty")
        if args.command == "handoff" and not args.confirmed_terminal:
            raise ValueError("verify the old thread is terminal, then pass --confirmed-terminal")
        if args.command == "progress" and not all(
            (args.stage.strip(), args.milestone.strip(), args.next_action.strip())
        ):
            raise ValueError("stage, milestone, and next action must not be empty")
        if args.command == "advance" and not all(
            (args.stage.strip(), args.milestone.strip(), args.next_action.strip())
        ):
            raise ValueError("stage, milestone, and next action must not be empty")
        if args.command == "advance" and args.task is not None and not args.task.strip():
            raise ValueError("task must not be empty")
        if args.command == "rescope" and not all(
            (args.stage.strip(), args.milestone.strip(), args.next_action.strip())
        ):
            raise ValueError("stage, milestone, and next action must not be empty")
        if args.command == "rescope" and args.task is not None and not args.task.strip():
            raise ValueError("task must not be empty")
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
            if args.lease_seconds < 60:
                raise ValueError("lease must be at least 60 seconds")
            cmd_progress(args, db)
        elif args.command == "advance":
            if args.lease_seconds < 60:
                raise ValueError("lease must be at least 60 seconds")
            cmd_advance(args, root, db)
        elif args.command == "rescope":
            if args.lease_seconds < 60:
                raise ValueError("lease must be at least 60 seconds")
            cmd_rescope(args, root, db)
        elif args.command == "release":
            cmd_release(args, db)
        elif args.command == "handoff":
            if args.old_thread == args.new_thread:
                raise ValueError("handoff requires a different new thread ID")
            if args.lease_seconds < 60:
                raise ValueError("lease must be at least 60 seconds")
            cmd_handoff(args, common, db)
        elif args.command == "baseline":
            cmd_baseline(args, root, db)
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
