# R2: Scoped retained control ownership

## Implementation

`RetainedEntities` now keys GPUI control entities and their subscriptions by a composition scope path plus `Identity`. `RenderContext` exposes `RetentionScope::{Tabs, Tab, Navigation, Page}`, `with_retention_scope`, and `preserve_retention_scope` to the tab and navigation composers. `Page(Vec<String>)` is one segment containing the full route prefix, so preserving a previous page does not also preserve a popped descendant.

Each root render clears frame bookkeeping, records every accessed retained control, and preserves declared inactive scope prefixes. After root lowering returns successfully, the host sweeps entries that were neither used nor under a preserved prefix. Dropping an entry releases the registry's entity and subscriptions. Rendering a second retained control with the same identity in the same scope during one frame panics with a scoped duplicate message; the same identity in different tabs or pages is valid. A panic during lowering skips the sweep; the next frame resets its bookkeeping.

## Retention policy

The composition layer should mark inactive tabs and previous pages still on the navigation path as preserved. Popped pages and controls removed from composition are swept. Bound owner values live on the component and are unaffected by this registry. Root component `View::on_mount`/`on_unmount` behavior remains the existing GPUI host behavior; this work does not add child-view lifecycle callbacks.

## Limits and acceptance

Registry removal releases its references; GPUI or an in-progress frame may hold other references and delay actual entity destruction. Scope paths are internal, and the existing retained entity and subscription caller methods keep their signatures. No second reactive runtime or new dependency was added. Source implementation is complete; workspace compilation and GUI checks are assigned to the parent task. Future checks should exercise inactive-tab state, prior-page state, pop and control removal, same ID in separate scopes, duplicate ID within one scope, and subscription release. No tests or GUI run were performed in this task.

## Integration evidence

The combined R2 workspace check completed successfully (exit 0, 3.09 seconds):
`./scripts/cargo-headstart check --workspace --all-targets --locked`.
Log: `target/headstart/roadmap-milestone4-check.log`. Non-recursive edition-2024
rustfmt checking passed for all four changed Rust files. No tests or GUI checks
were executed; behavioral acceptance remains pending.
