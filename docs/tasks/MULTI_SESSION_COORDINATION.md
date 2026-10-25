# Multi-session roadmap coordination

Every writing session owns one clean, registered Git worktree on a unique
`codex/` branch. Before editing, it acquires an atomic claim for the task,
thread, stage/milestone, worktree, and exact repository-relative files. Claims
are stored in one SQLite database outside the checkout and shared by all linked
worktrees for this repository.

```sh
python3 scripts/roadmap_claims.py claim \
  --task M1234 --thread <codex-thread-id> \
  --stage "Stage 5" --milestone M1234 --next-action "implement resolver" \
  --worktree "$(git rev-parse --show-toplevel)" \
  crates/rustui/src/example.rs
```

For an existing session that already has changes, create a unique `codex/`
branch in its own linked worktree, then adopt its exact assigned paths before
continuing. Adoption records those paths as the session's write scope and
fingerprints unrelated pre-existing dirty paths as a baseline. `audit` allows
those paths only while their content stays unchanged; a later edit or commit
outside the claim fails the audit:

```sh
python3 scripts/roadmap_claims.py adopt \
  --task M1234 --thread <codex-thread-id> \
  --stage "Stage 5" --milestone M1234 --next-action "finish resolver" \
  --worktree "$(git rev-parse --show-toplevel)" \
  crates/rustui/src/theme/resolver.rs \
  @shared/target/headstart/evidence/M1234.log
```

Continue tracking after claim or adoption:

```sh
python3 scripts/roadmap_claims.py progress --claim-id <claim-id> \
  --thread <codex-thread-id> --stage "Stage 5" --milestone M1234 \
  --next-action "add focused regression"
python3 scripts/roadmap_claims.py heartbeat --claim-id <claim-id> \
  --thread <codex-thread-id>
python3 scripts/roadmap_claims.py audit --claim-id <claim-id> \
  --thread <codex-thread-id>
python3 scripts/roadmap_claims.py list
python3 scripts/roadmap_claims.py list --verbose
```

The default list reports a count for pre-existing baseline paths; `--verbose`
includes their names for a detailed audit.
`progress` also renews the lease; use `heartbeat` during long work between
milestone updates.

The registry serializes acquisitions in SQLite transactions. A claim on a
directory conflicts with claims on that directory, any parent, or any child.
Use `@shared/<name>` for logical shared resources such as roadmap ledgers;
coordinator-owned ledgers are edited one at a time. The registry rejects the
primary checkout, detached/non-`codex/` branches, dirty worktrees at claim time,
and reuse of an active task ID, thread, worktree, or branch. A session may edit only
inside its claimed paths; run `audit` at turn boundaries to check the actual
Git changes against the claim.

Leases are a liveness signal only. An expired lease appears as `suspect` and
continues to lock all paths. The coordinator must inspect the thread state and
worktree before releasing or transferring ownership. Never release a claim
because a session is idle, blocked, not loaded, or has missed a heartbeat.

```sh
python3 scripts/roadmap_claims.py release --claim-id <claim-id> \
  --thread <codex-thread-id> --terminal-state completed \
  --evidence "Reviewed terminal thread status and retained worktree"
```

For handoff, first verify the old session is terminal and preserve its worktree
and uncommitted changes. Then transfer the claim atomically to a separate clean
worktree on a unique `codex/` branch. The replacement must inspect and
deliberately integrate the old work before continuing.

```sh
python3 scripts/roadmap_claims.py handoff --claim-id <claim-id> \
  --old-thread <stopped-thread-id> --new-thread <replacement-thread-id> \
  --new-worktree /path/to/clean/replacement/worktree \
  --stage "Stage 5" --milestone M1234 --next-action "continue implementation" \
  --terminal-state superseded --evidence "Terminal state checked in Codex"
```

The database prevents collisions only among cooperating sessions. It cannot
stop an unregistered process from writing. Therefore each assignment must
include its claim ID, exact paths, branch, and next action; the coordinator must
check claims before assignment and review audit output before integration.
