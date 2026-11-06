# NavigationSplit accessibility slice

RustUI owns the navigation and destination composition around the pinned native resizable layout.

- The sidebar already applies a navigation role and its configured accessible name. Its destination rows already use tree-item roles, labels, and selected state in the pinned GPUI Components sidebar implementation (`vendor/gpui-component-0.7.3/src/sidebar/menu.rs`). The Playground story now supplies the optional name, `Workspace navigation`.
- A selected destination now has a RustUI semantics wrapper with the `Region` role and the same name as its displayed destination label. Localized destinations resolve that label from the active message catalog before it is used for the region name; RustUI does not append an English-only suffix. No detail region is created when selection has no matching destination.
- The pinned public resizable API (`vendor/gpui-base-0.7.3/src/resizable/mod.rs`) exposes panel sizing and handle appearance, while the resize handle itself is an internal renderer. It provides no supported way to assign a divider role, label, orientation, or current/minimum/maximum values. RustUI therefore does not fabricate a second accessible resize control or intercept native resize behavior.

## Acceptance boundary

Source-level checks confirm that RustUI supplies the region role and the resolved destination label; the focused localization regression passes. Native GUI accessibility inspection is still needed to confirm the resulting macOS accessibility tree and spoken labels. Keyboard resizing and divider discoverability remain unsupported by the pinned public API; resolving those requires an upstream API capability or an approved revision/patch change.

## Label input validation — 2026-10-20

`NavigationSplitItem` now rejects blank visible labels and blank localization
keys. `NavigationSplitView` rejects blank literal tree names and blank
localization keys. The focused regression verifies all four input boundaries.
This is source-level validation only; localized screen-reader output and native
accessibility acceptance remain open. The focused test passed in
`target/headstart/evidence/navigation-split-label-validation-20261020.log`; the
isolated all-targets check passed in
`target/headstart/evidence/navigation-split-label-check-isolated-20261020.log`.
## Narrow open-action localization — M468

The narrow-mode “Open selected destination” action now resolves its visible
label from the framework catalogs. Its accessibility name interpolates the
selected destination using a localized message in English, French, and Arabic.
When localization is inactive or interpolation fails, the localized action
label (or English fallback) remains available. Catalog interpolation and the
RustUI package check passed; native narrow-width rendering, focus transfer,
and VoiceOver/Accessibility Inspector acceptance remain open. Evidence:
`target/headstart/evidence/m468-navigation-split-i18n-test.log` and
`target/headstart/evidence/m468-navigation-split-rustui-check.log`.
