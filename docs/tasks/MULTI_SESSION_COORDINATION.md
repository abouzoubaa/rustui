# Multi-session roadmap coordination

For every writer, the `--thread` value must be that session's **own Codex
thread ID**. The orchestrator must obtain it from the session inventory and put
the literal ID in that session's assignment; never reuse a generic placeholder
or substitute the orchestrator's ID. Before the first edit, the session must
check `list` and confirm its claim records its own thread ID, actual worktree,
unique `codex/` branch, and only its exact assigned paths. If any field differs
or claim acquisition fails, stop before writing and report the mismatch. The
orchestrator independently checks the same ledger entry before treating the
assignment as active. The coordinator cannot infer Codex session identity from
a shell process.

Every session must invoke the canonical coordinator at
`/Users/Abium/.codex/worktrees/session-coordination/rustui/scripts/roadmap_claims.py`.
Run it with the shell's current directory set to your own linked worktree so it
uses that worktree's Git common directory and the shared claim database. Avoid a
checkout-local copy unless it is confirmed to match this canonical revision.

Every writing session owns one clean, registered Git worktree on a unique
`codex/` branch. Before editing, it acquires an atomic claim for the task,
thread, stage/milestone, worktree, and exact repository-relative files. Claims
are stored in one SQLite database outside the checkout and shared by all linked
worktrees for this repository.

```sh
python3 /Users/Abium/.codex/worktrees/session-coordination/rustui/scripts/roadmap_claims.py claim \
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
python3 /Users/Abium/.codex/worktrees/session-coordination/rustui/scripts/roadmap_claims.py adopt \
  --task M1234 --thread <codex-thread-id> \
  --stage "Stage 5" --milestone M1234 --next-action "finish resolver" \
  --worktree "$(git rev-parse --show-toplevel)" \
  crates/rustui/src/theme/resolver.rs \
  @shared/target/headstart/evidence/M1234.log
```

Continue tracking after claim or adoption:

```sh
python3 /Users/Abium/.codex/worktrees/session-coordination/rustui/scripts/roadmap_claims.py progress --claim-id <claim-id> \
  --thread <codex-thread-id> --stage "Stage 5" --milestone M1234 \
  --next-action "add focused regression"
python3 /Users/Abium/.codex/worktrees/session-coordination/rustui/scripts/roadmap_claims.py heartbeat --claim-id <claim-id> \
  --thread <codex-thread-id>
python3 /Users/Abium/.codex/worktrees/session-coordination/rustui/scripts/roadmap_claims.py audit --claim-id <claim-id> \
  --thread <codex-thread-id>
python3 /Users/Abium/.codex/worktrees/session-coordination/rustui/scripts/roadmap_claims.py list
python3 /Users/Abium/.codex/worktrees/session-coordination/rustui/scripts/roadmap_claims.py list --verbose
```

The default list reports a count for pre-existing baseline paths; `--verbose`
includes their names for a detailed audit.
`progress` also renews the lease; use `heartbeat` during long work between
milestone updates.

Renew before starting any operation likely to outlast the lease, and confirm
the returned claim state is `active` before editing again. An expired lease is
displayed as `suspect`; it does **not** release the claim or its paths. The
shared database still treats that claim as active, so another session cannot
acquire those paths until the owner renews, or the coordinator verifies the
owner has stopped and performs an explicit release or handoff. Never work
around a `suspect` claim by changing the thread ID or editing without an active
claim.

Use `advance` when the same session moves to another task ID or needs additional
paths. It extends the existing claim; it never drops prior paths or releases the
owner. The task update, new paths, transition history, and lease renewal are one
SQLite transaction:

```sh
python3 /Users/Abium/.codex/worktrees/session-coordination/rustui/scripts/roadmap_claims.py advance --claim-id <claim-id> \
  --thread <codex-thread-id> --task M1235 --stage "Stage 9" \
  --milestone M1235 --next-action "implement the next bounded registry slice" \
  crates/rustui/src/preview/registry.rs docs/tasks/m1235_registry_slice.md
```

Omit `--task` to keep the current task ID while expanding its scope. New paths
must be clean when added; if they already contain changes, use the audited
adoption flow. The command rejects active task, thread, worktree, branch, or
path conflicts without changing the claim. `audit` returns the claim's recorded
advance and handoff history.

Use `rescope` to correct an overbroad or mistaken path set without releasing
ownership. It replaces the complete path set in one transaction, checks the
owner, worktree, branch, active conflicts, and changed paths, and records the
removed paths in transition evidence. It refuses to drop any path containing
uncommitted or committed changes and refuses to add a dirty path. A rejected
rescope leaves the original claim intact:

```sh
python3 /Users/Abium/.codex/worktrees/session-coordination/rustui/scripts/roadmap_claims.py rescope \
  --claim-id <claim-id> --thread <your-own-codex-thread-id> --task M1236 \
  --stage "Stage 8" --milestone M1236 --next-action "write the scoped acceptance sheet" \
  docs/tasks/m1236_acceptance_run_sheet.md
```

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
python3 /Users/Abium/.codex/worktrees/session-coordination/rustui/scripts/roadmap_claims.py release --claim-id <claim-id> \
  --thread <codex-thread-id> --terminal-state completed \
  --evidence "Reviewed terminal thread status and retained worktree"
```

For handoff, first verify the old session is terminal and preserve its worktree
and uncommitted changes. Then transfer the claim atomically to a separate clean
worktree on a unique `codex/` branch. The replacement must inspect and
deliberately integrate the old work before continuing. The CLI cannot observe
Codex thread liveness; the orchestrator must inspect the old thread and confirm
it has stopped before using `--confirmed-terminal`. This is an explicit
cooperative gate, not an automatic liveness check.

```sh
python3 /Users/Abium/.codex/worktrees/session-coordination/rustui/scripts/roadmap_claims.py handoff --claim-id <claim-id> \
  --old-thread <stopped-thread-id> --new-thread <replacement-thread-id> \
  --new-worktree /path/to/clean/replacement/worktree \
  --stage "Stage 5" --milestone M1234 --next-action "continue implementation" \
  --terminal-state superseded --confirmed-terminal \
  --evidence "Terminal state checked in Codex"
```

The database prevents collisions only among cooperating sessions. It cannot
stop an unregistered process from writing. Therefore each assignment must
include its claim ID, exact paths, branch, and next action; the coordinator must
check claims before assignment and review audit output before integration.

## Assigning parallel writers

Every writer must be a distinct Codex thread with its own literal thread ID,
registered clean worktree, unique `codex/` branch, and claim. Do not assign
repository edits to a nested agent unless that agent can independently satisfy
and verify all four requirements; otherwise use it for read-only discovery and
have the registered thread make the change. Before opening a parallel task,
check every active and suspect claim for overlapping paths, task IDs, threads,
worktrees, or branches. `suspect`, idle, and not-loaded are not terminal states
and never free ownership. A completed milestone can be followed by a new task
only through the same owner's atomic `advance`, or through a verified terminal
handoff to a new thread and clean worktree.

## Preserving pre-existing work in an active claim

`advance` and `rescope` reject adding files that already contain changes. If an
active claim's audit discovers unrelated work that was already present in its
worktree, the owner may record those exact changed files as a fingerprinted
baseline. This does not add them to the owner's write scope. The command is
owner-only, accepts only currently changed repository paths outside the claim,
rejects paths overlapping any other active claim, and makes later edits fail
audit. Do not baseline work created after the claim or use a directory-wide
baseline when exact files are available.

```sh
python3 /Users/Abium/.codex/worktrees/session-coordination/rustui/scripts/roadmap_claims.py baseline \
  --claim-id <claim-id> --thread <codex-thread-id> \
  --evidence "present before the claim; independently confirmed" \
  graft/.cache/existing-file.json graft/.graph/existing-file.json
```

After baseline, run `audit` and inspect its output before continuing. Baseline
fingerprints are preserved in the shared registry and shown by `list --verbose`.
