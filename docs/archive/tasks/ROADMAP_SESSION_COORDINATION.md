# RustUI roadmap session coordination

Use this protocol for parallel roadmap work. It prevents concurrent writers from sharing a checkout or claiming overlapping files, and keeps a blocked subtask from freezing its whole stage.

## Before a session writes

1. Give every assignment a unique task ID, one Stage, one milestone, one thread ID, and a bounded deliverable. A session owns at most one active writing task.
2. Create or select a dedicated linked worktree. The primary `/Users/Abium/Developer/rustui` checkout is shared and read-only for coordinated roadmap coding. Do not hand off, reset, clean, stash, or rebase its dirty state.
3. Claim the exact files atomically before editing:

   ```sh
   python3 scripts/roadmap_session_coordinator.py claim \
     --task M1234-stage-3-example \
     --thread-id 01abc... \
     --stage 3 \
     --milestone M1234 \
     --worktree /path/to/linked/rustui \
     --file crates/rustui/src/example.rs \
     --next-task 'Finish the focused change and report its evidence'
   ```

   Claims are exact repository-relative paths. Directory claims overlap every descendant. The coordinator rejects the primary checkout, unregistered worktrees, a second writer in the same worktree, a thread with two writing assignments, and any overlapping active file claims.
4. Keep the task goal as the durable stage/milestone objective. The coordinator entry is the short-lived execution lease. Read the live assignments with `python3 scripts/roadmap_session_coordinator.py list` before dispatching work.

## While working and handing off

- One active writer per worktree. Independent sessions use separate worktrees and disjoint file claims. Shared manifests, lockfiles, roadmap ledgers, Playground GUI bundles, and other shared resources require a single named owner or a serialized integration reservation.
- Update the coordinator status before stopping. Use `awaiting_decision` or `blocked_external` only when that task truly needs an external decision or state change; include a concrete independent `--next-task`. Non-writing statuses release file claims immediately. A stage goal remains active while its task waits.
- On completion, set `completed`; this releases claims but preserves the assignment record. For replacement, set `released`, then the new thread claims the task under its own ID. Never reuse an active task ID for different work.
- Before merge/integration, the orchestrator checks the diff against the recorded files, checks the latest Git state, and serializes changes to shared files. The worker does not merge or update the primary checkout.
- If a thread is idle, unloaded, stale, or repeatedly errors, treat its lease as unconfirmed: inspect its worktree and last handoff, release/reassign only after confirming its current state. A task-level wait is not a stage-level blocker.

## Ledger and concurrency guarantees

The coordinator stores its ledger under the Git common directory, shared by every linked worktree. It uses an OS file lock for each read/modify/write and atomic file replacement, so simultaneous claims cannot both succeed for conflicting paths. File claims are cooperative: sessions must run the coordinator before writing, and the orchestrator must verify the diff before integration. Direct editing without a claim is outside the protocol.

Assignments record the owner thread, Stage, milestone, worktree, branch, starting commit, exact file claims, status, next action, and update time. The ledger is local coordination state and is not committed into the product repository.
