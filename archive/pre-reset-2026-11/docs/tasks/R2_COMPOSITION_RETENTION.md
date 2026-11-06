# R2 composition retention

TabView and NavigationStack now use the runtime retention scopes exposed through `RenderContext`. These scopes identify composed child content while GPUI entities remain owned by the existing retained store. The public constructors and selection bindings are unchanged.

## TabView

`Tabs(container identity)` contains each `Tab(id)` scope. The selected tab is lowered inside its own scope. Every other tab still declared by the current `TabView` is preserved while inactive. A removed tab is not preserved. The selected tab's whole subtree is not preserved, so conditional children that disappear during the current render can be pruned.

## NavigationStack

`Navigation(container identity)` contains `Page(full route prefix)` scopes. The root uses `Page([])` and is preserved while the path is nonempty. Each previous path prefix is preserved only when its destination remains declared. Only the current destination is lowered; an unknown current route continues to render the localized unavailable destination and Back control without retaining a missing destination.

The full prefix is one scope segment per page, rather than nested segments for its route parts. Thus `["a"]`, `["a", "b"]`, and `["a", "b", "a"]` identify different page occurrences. A popped page or a changed prefix is no longer preserved. The existing RustUI Button, localized labels, path update actions, and public constructors remain in place.

## Identity boundary

Dynamic TabView and NavigationStack collections need a stable explicit `.id(...)` on the container. The existing TabView fallback uses the tab count and first tab ID; the NavigationStack fallback is the fixed `navigation-stack` identity. These fallbacks cannot guarantee correct lifetime boundaries for dynamic or multiple sibling collections. This task leaves those public API defaults unchanged.

The root agent owns milestone compilation. GUI retention, focus, and accessibility interaction checks remain for a later integration pass.

## Integration evidence

The combined R2 workspace check completed successfully (exit 0, 3.09 seconds):
`./scripts/cargo-headstart check --workspace --all-targets --locked`.
Log: `target/headstart/roadmap-milestone4-check.log`. Non-recursive edition-2024
rustfmt checking passed for all four changed Rust files. No tests or GUI checks
were executed; behavioral acceptance remains pending.
