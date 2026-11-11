# Async ownership audit (R20)

## Public API inventory

| API | Implementation | Ownership and restart contract |
| --- | --- | --- |
| `OwnerTask::start` | `crates/rustui/src/runtime/tasks.rs` | Returns an `AppAction`; dispatch starts background work owned by the current `ComponentHost`. Reusing a task ID cancels the previous run and advances its generation. |
| `OwnerTask::start_with_executor` | `crates/rustui/src/runtime/tasks.rs` | Same contract, additionally supplies GPUI's `BackgroundExecutor` to the work factory. |

The public re-exports in `runtime/mod.rs` and `prelude.rs` refer to these same APIs.
`view.rs` has no `.task()` modifier or public spawn helper. `OwnerTaskReader`
supplies the existing component owner's controller; rendering the reader does not
start work. Both starters in this audit obtain their controller through a rendered
`OwnerTaskReader` inside a real retained `Component` child.

The following candidates do not start asynchronous work:

| Candidate | Reason |
| --- | --- |
| `OwnerTask::cancel` | Removes a named task and applies `Idle`; never starts work. |
| `List`, `ListView::new` | No async loader API. Private `StringListDelegate::perform_search` filters synchronously and returns `Task::ready(())`. |
| `KeyedList`, `KeyedListView::new` | No async loader API. Private `KeyedListDelegate::perform_search` filters synchronously, queues a foreground selection reconciliation with `Window::defer`, and returns `Task::ready(())`. |
| `Component`, `View::on_mount`, `View::on_unmount` | Lifecycle hooks/retention only; no implicit asynchronous work. |
| Re-exported GPUI executor methods | Upstream GPUI APIs, not additional RustUI task starters. Their tasks must remain retained by the owner's work future; deliberately detached application work is outside the ownership contract. |

The inventory searched `crates/rustui/src`, `crates/rustui-core/src`, and
`crates/rustui-macros/src` for `spawn`, `async`, `Future`, `Task::`, `.task(`,
and loaders, and inspected the public declarations around every production hit.
Animation frame callbacks and synchronous deferred callbacks are not futures or
public async loaders.

## Matrix

`crates/rustui/src/runtime/async_tests.rs` contains one shared runner and an API ×
check case table. Each generated case uses a GPUI test window and deterministic
executor. A manually released, waker-driven future replaces time and I/O; there
are no real sleeps. Pending work is polled before each check.

| API | Unmount | Restart / stale result | Error / later success | Owner update | No leak |
| --- | --- | --- | --- | --- | --- |
| `OwnerTask::start` | PASS | PASS | PASS | PASS | PASS |
| `OwnerTask::start_with_executor` | PASS | PASS | PASS | PASS | PASS |

`OwnerTaskReader` is exercised in every cell, including obtaining a fresh
controller during re-render without starting work. The non-starters above have
N/A for all five async checks because they own no asynchronous run.

Test names are `runtime::tasks::async_audit_tests::{start,start_with_executor}::`
followed by `unmount`, `restart`, `error`, `owner_update`, or `no_leak`.

- `unmount`: removes the retained child through its parent's rendered tree,
  checks one unmount and one future drop, releases the old result, and asserts no
  callback/state write. A retained action cannot resurrect its owner.
- `restart`: changes the input under the same logical task ID, checks cancellation,
  and accepts only the newest result. Also races already-produced old success and
  error results against a replacement before foreground delivery.
- `error`: observes exactly one error notification, updates and re-renders the
  owner, then checks that Loading and a later success clear the error.
- `owner_update`: forces an unrelated owner update and verifies a real re-render
  without restarting or dropping pending work; explicit changed-input dispatch
  then replaces that work exactly once.
- `no_leak`: removes the retained child, drains the executor, and asserts its weak
  entity cannot upgrade, despite a retained controller and action. The task future
  is dropped and no old completion can write state.

Different task IDs identify independent concurrent slots, not automatic input
keys. Input changes restart the same slot by dispatching another start action;
rendering by itself does not dispatch an action. Existing task tests separately
cover independent slots, explicit cancellation, window close, and registry
rejection of obsolete generations.

## Result

| Run | PASS | FAIL | N/A | Ignored |
| --- | --- | --- | --- | --- |
| Before runtime changes (initial matrix) | 10 | 0 | 0 | 0 |
| Final matrix | 10 | 0 | 0 | 0 |

The existing implementation passed every applicable case. No production code or
public signature changed, and no R20b tests were ignored. `docs/status.json` is
unchanged because this ticket verifies existing capabilities rather than changing
a product capability.

Focused command: `rustup run 1.98.0 cargo test -p rustui async_audit_tests --locked --offline`.
Full validation: `scripts/check.sh` and `./scripts/verify-release.sh source`.

