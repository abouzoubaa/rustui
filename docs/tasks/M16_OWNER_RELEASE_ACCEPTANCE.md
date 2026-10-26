# M16: Owner release while a task is running

## Native Playground exercise

The `task_release` story keeps its child as a conditional `Component` with a stable
key. It now distinguishes two cases; the run sheet's original blocking-work
expectation must not be applied to the cancellable timer.

1. Start **Start 3 second task**, then select **Observe counters** until
   **Background started** increases. Remove the child before three seconds,
   wait beyond the deadline, and observe again. The GPUI timer future is
   cancellable: no result or owner delivery should be recorded, and the child
   release counter should increase. Restore the child and verify `Idle`.
2. On macOS, start **Start non-cancellable native work** and observe that its
   background-start counter increases before removal. Remove the child, wait
   beyond three seconds, and observe again. This work finishes its in-flight
   background poll: results should increase, owner deliveries should not, and
   the child release counter should increase. Restore and verify `Idle`.
3. Let a fresh restored child complete its ordinary timer task while mounted.
   Its result and owner-delivery counters should increase and show Success.

Repeat with a fresh story instance if needed. Keep the story open during the wait.
The foreground observation button triggers a rerender so the shared counters become
visible.

## Evidence and limits

The story uses existing `OwnerTask`, conditional child composition, and the existing
`View::on_unmount` hook. The shared completion counter increments only when a terminal
result is applied to the child owner; the work counter increments immediately before
the future returns. This distinguishes completed background work from delivery to a
released owner.

The ordinary delayed future uses GPUI's executor timer on both targets. The
additional native-only acceptance probe performs a bounded blocking sleep on
GPUI's background executor; it creates no thread or second scheduler and is
compiled out on Web/Wasm. No framework runtime or vendor code is changed.
Native acceptance of the current two-case exercise remains pending until run
and recorded; arbitrary future interruption is not promised.

## Historical native acceptance (M25, original blocking probe)

The child task started before removal. After the blocking worker finished,
background counters read started 1 / results 1 / delivered 0, with mounts 1 and
releases 1. Restoration produced a new Idle child (mounts 2 / releases 1),
without delivering the stale result. A new task on that child completed
normally: started 2 / results 2 / delivered 1 and visible Success.
Evidence: `target/headstart/milestone25-owner-release-gui.json`; successful
Playground build: `target/headstart/milestone25-release-build-fixed.log`.
The worker's blocking function finished after removal, so this establishes
owner cleanup and result suppression rather than interruption of blocking code.
