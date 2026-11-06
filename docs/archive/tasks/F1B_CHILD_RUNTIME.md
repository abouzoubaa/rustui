# F1b: Retained child component baseline

`Component(key, || Child::new()) -> ComponentView<Child>` creates an explicit child owner. The constructor is available from `rustui::prelude`. It requires a stable key within the current retention scope and an initializer of type `Fn() -> Child + 'static`. A plain nested `View`, `AnyView`, or `Keyed` description remains part of its parent's owner and does not receive lifecycle hooks.

The parent `ComponentHost` keeps a GPUI `Entity<ComponentHost<Child>>` in a child registry keyed by parent scope and child key. This registry is separate from retained controls, so their local IDs may coincide. Duplicate component keys in one scope and render are rejected even when the concrete child types differ. Repeated rendering with the same key and type reuses the entity and ignores a newly supplied initializer closure. A different concrete type at the same key creates a new host and GPUI entity ID, resetting child state and native element state. A removed visible child releases the parent's registry reference after lowering; GPUI release invokes `on_unmount` once all entity references are gone. GPUI renders the child entity directly, without an added layout box.

Each child host owns its component, retained controls, updater, and lifecycle. Actions and bindings lowered inside its `RenderContext` target that child owner. The root and child use the same mount, release, and notify code. Inactive declared tabs and covered navigation pages keep their registry entries through R2 preservation; removed tabs and popped pages release them. A conditional child omitted from its parent body is swept. Reordering stable keys keeps entities.

The child receives a clone of its parent's `EnvironmentValues` when first created. The parent host's environment is currently immutable through the public API, so this baseline does not refresh it on later parent renders. Captures in the initializer supply initial values only; they are not reconciled as props. A future props/environment API needs an explicit update and notification rule, including inactive children. No independent child-to-parent action API is included. `Action` and `Binding` lenses are owner-relative: a parent lens passed into a child does not target the parent, and matching concrete owner types alone cannot encode which instance was intended. Use `AppAction` for application-level callbacks or keep the interaction at the owner that has the binding until an explicit parent-target API exists.

Source inspection confirms GPUI `Entity<Render>` lowers through `ViewElement` with the entity ID as a native identity boundary and the uncached render path. A workspace compile check and native GUI lifecycle/focus acceptance remain pending in this task record.


## Milestone 10 environment update

The creation-only environment limitation recorded for this earlier slice is
superseded by F1d: retained child snapshots now refresh before native lowering on
parent renders. Typed `Environment` and `EnvironmentReader` provide scoped values.
Inactive retained children catch up when lowered again. Fine-grained observation
and cached-child invalidation remain unsupported; see `F1D_ENVIRONMENT.md`.
