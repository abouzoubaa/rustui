# F1d: Scoped environment

`Environment(value, view) -> EnvironmentView<T, V>` overrides one concrete type for that view's subtree. `T` must be `Clone + 'static`; `V` is a `View`. `EnvironmentReader(|value: Option<&T>| -> V { ... }) -> EnvironmentReaderView<T, V, F>` reads the nearest value while lowering and returns an owned view. Both constructors and result types are in `rustui::prelude`.

```rust
Environment(
    Accent::Blue,
    EnvironmentReader(|accent: Option<&Accent>| {
        Text::new(format!("{:?}", accent))
    }),
)
```

The provider clones its parent's `EnvironmentValues` map, inserts its typed value, and temporarily reborrows the same render context. The existing map holds each value in `Rc`, so cloning the map copies entries, while cloning the new `T` supplies its owned value. Nested providers of the same type shadow outward values; siblings and the enclosing view continue with their original environment. The provider adds no GPUI layout box or retention scope. Its map clone costs work proportional to the number of environment types in the snapshot.

Plain nested views and readers lower under their current component owner. Retained `Component` and `.props(...)` children receive the current environment snapshot on first creation and refresh it on each active reuse before GPUI lowers their entity. This assignment does not notify during parent rendering: pinned GPUI's uncached `ViewElement` calls the entity's `render` in the same traversal. A props change still reconciles and notifies as before. Child-owned state, props history, lifecycle, and action/binding ownership remain with the retained entity. Inactive R2-preserved children keep their previous snapshot until their next lowering.

Environment reads depend on a parent render or child render that lowers the reader. There is no per-value subscription, observation of interior mutation, or guarantee for a GPUI cached child render. Active theme and localization services remain separate; a typed `Locale` environment value does not change the active localization service.

Source review completed. Cargo, tests, and native GUI checks are delegated to milestone verification.
