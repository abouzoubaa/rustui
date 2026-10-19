# F2 — Owner-scoped GPUI tasks

`OwnerTaskReader` runs in a component body and passes an `OwnerTask<C>` bound to
that exact GPUI host. A parent may pass the controller through child props. It is
distinct from a child's local `self.action` target.

```rust
OwnerTaskReader(|tasks: OwnerTask<SearchPage>| {
    VStack::new()
        .child(Button(
            "Load",
            tasks.start("search", || async { fetch_results().await }, SearchPage::set_result),
        ))
        .child(Button("Cancel", tasks.cancel("search", SearchPage::set_result)))
})
```

`set_result` accepts `AsyncState<T, E>` and returns `bool` to indicate whether
GPUI should notify the owner. Store that state in the owning component. `start`
sets `Loading`, replaces any task with the same stable ID, and runs a `Send`
future on GPUI's background executor. The retained foreground task applies
`Success` or `Error` on the UI thread. `cancel` drops the task and sets `Idle`.
Different IDs can run concurrently.

Each start receives a monotonic host generation. A result is applied only if its
ID and generation still match, so a queued result from an older run cannot
overwrite a restart. The controller holds a weak GPUI entity; callbacks after
host release are ignored, and dropping the host drops retained task handles.
Dropping a task stops future polling but cannot interrupt arbitrary blocking
code already executing. Work with external side effects may need its own
cancellation policy; dropping a future does not undo those effects.

After applying a terminal result, a separate GPUI foreground task removes the
matching registry entry in a later poll. This avoids dropping a completed task's
own handle while it is being polled. Cleanup checks the generation again, so a
restart between completion and cleanup remains owned.

The reader checks its concrete owner type during lowering and does not start
work during lowering. `OwnerTask<C>` compares by weak entity identity so
unchanged props do not repeatedly reconcile it. Unit checks cover restart,
cancel, terminal cleanup, and generation non-reuse. No separate executor,
reactive runtime, vendor change, or dependency was introduced.

Playground's `stories/owner_tasks.rs` provides slow success, fast restart, error,
and cancel actions for one request ID. The state text and button accessibility
labels expose the current status. `stories/task_release.rs` covers child-owner
removal and restoration.

## Executed native acceptance

The M16 Review binary exercised Loading, success, error, cancellation with Idle
readback, and replacement with the restarted result visible after the original
completion window. Record and binary hash: `target/headstart/milestone16-owner-task-gui.json`.
That binary predates deferred terminal registry cleanup. Switching foundation
tabs preserves component owners and does not prove owner destruction.

M72 native acceptance on a rebuilt Review binary passed slow-then-fast restart
(the restarted Success remained after the original delay), start-then-cancel
(Idle remained after the original delay), error display, and restart after a
completed request ID. Removing a pending child owner showed
`background_started=1`, `background_finished=1`, `completion_delivered=0`, and
one mount/one unmount. Restoring the child showed Idle with two mounts and one
unmount. All six checks are recorded in
`target/headstart/milestone72-owner-tasks-gui.json`; the build log
`target/headstart/milestone72-build.log` exited 0.

The pending worker finished; the evidence shows result delivery was suppressed
after cancellation or owner release, not that arbitrary blocking work was
interrupted. Terminal request restart was visible, but private registry
allocation cleanup was not directly counted. This is bounded native acceptance;
cross-platform, keyboard-only, and full screen-reader acceptance remain open.
