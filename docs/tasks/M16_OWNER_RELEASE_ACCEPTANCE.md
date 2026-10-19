# M16: Owner release while a task is running

## Native Playground exercise

The `task_release` story keeps its child as a conditional `Component` with a stable
key. Start the child's three-second task, select **Observe counters** until
**Background started** is 1, remove the child, wait at least three seconds, then
select **Observe counters** again. Expected result: the background result counter
increases, completion deliveries stay at zero, and the child release counter
increases. Restore the child and verify its local task state starts at `Idle`.

Repeat with a fresh story instance if needed. Keep the story open during the wait.
The foreground observation button triggers a rerender so the shared counters become
visible.

## Evidence and limits

The story uses existing `OwnerTask`, conditional child composition, and the existing
`View::on_unmount` hook. The shared completion counter increments only when a terminal
result is applied to the child owner; the work counter increments immediately before
the future returns. This distinguishes completed background work from delivery to a
released owner.

The delayed future uses a blocking sleep on GPUI's background executor, matching the
existing owner-task story's deterministic delay pattern. It is an acceptance probe,
not a general cancellation guarantee for all future types. No runtime or vendor code
is changed. This document describes the exercise and expected observations; native
acceptance is pending until run and recorded.

## Executed native acceptance (M25)

The child task started before removal. After the blocking worker finished,
background counters read started 1 / results 1 / delivered 0, with mounts 1 and
releases 1. Restoration produced a new Idle child (mounts 2 / releases 1),
without delivering the stale result. A new task on that child completed
normally: started 2 / results 2 / delivered 1 and visible Success.
Evidence: `target/headstart/milestone25-owner-release-gui.json`; successful
Playground build: `target/headstart/milestone25-release-build-fixed.log`.
The worker's blocking function finished after removal, so this establishes
owner cleanup and result suppression rather than interruption of blocking code.
