# F1a: Keyed composition integration

F1a aligns RustUI's retained control scopes with native GPUI element ancestry.
The `RenderContext::with_element_scope` helper applies a retention scope and
returns a transparent GPUI identity wrapper, so the scoped child keeps the same
layout identity without adding a layout container.

`TabView` scopes its complete rendered root, including the tab bar, under
`Tabs(identity)`. The selected content is nested under `Tab(id)`. Inactive
currently declared tabs continue to be preserved, while selection fallback,
disabled tabs, and style behavior stay unchanged.

`NavigationStack` scopes its complete root under `Navigation(identity)`. The
root child uses `Page([])`, and each selected destination uses `Page(full_path)`.
Previously declared pages continue to be preserved. Unknown routes keep their
existing localized message and Back behavior.

A plain `VStack` or `HStack` does not establish a keyed boundary: its children
are simply composed in order with the existing `.child(view)` operation. The
new `.children(iterable)` convenience eagerly collects views in iteration order
by repeatedly using `.child(view)`. It does not add keys, virtualization, or
macro behavior. Callers that need retention identity should place children
inside an owning keyed component such as `Keyed`, `Tab`, or `Page`.
