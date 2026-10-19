# NavigationSplit accessibility slice

RustUI owns the navigation and destination composition around the pinned native resizable layout.

- The sidebar already applies a navigation role and its configured accessible name. Its destination rows already use tree-item roles, labels, and selected state in the pinned GPUI Components sidebar implementation (`vendor/gpui-component-0.7.3/src/sidebar/menu.rs`). The Playground story now supplies the optional name, `Workspace navigation`.
- A selected destination now has a RustUI semantics wrapper with the `Region` role and a name derived from its displayed destination label (`<label> details`). Localized destinations resolve that label from the active message catalog before it is used for the region name. No detail region is created when selection has no matching destination.
- The pinned public resizable API (`vendor/gpui-base-0.7.3/src/resizable/mod.rs`) exposes panel sizing and handle appearance, while the resize handle itself is an internal renderer. It provides no supported way to assign a divider role, label, orientation, or current/minimum/maximum values. RustUI therefore does not fabricate a second accessible resize control or intercept native resize behavior.

## Acceptance boundary

Source-level checks can confirm that RustUI supplies the region role and computed label, and that the example sets the navigation name. Native GUI accessibility inspection is still needed to confirm the resulting macOS accessibility tree and spoken labels. Keyboard resizing and divider discoverability remain unsupported by the pinned public API; resolving those requires an upstream API capability or an approved revision/patch change.
