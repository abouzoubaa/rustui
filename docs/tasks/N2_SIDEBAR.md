# N2: Controlled flat sidebar selection

`Sidebar(id, items, selection)` adapts the pinned GPUI Components `Sidebar`,
`SidebarMenu`, and `SidebarMenuItem` to a RustUI-owned selection binding. Each
`SidebarItem::new(id, label)` has a nonempty unique semantic ID. Selection is
`Binding<Option<String>>`; labels and row positions do not define identity.
Reordering items or changing labels therefore keeps the same selected ID active.
An ID that is absent from the current items renders with no active row and stays
in the owner binding until the caller changes it.

`SidebarItem::disabled(true)` applies the upstream disabled state and guards the
bound selection callback. `.localized(message_id)` resolves through the active
RustUI catalog. The first adapter is flat, has no icon or asset dependency, and
uses the upstream default presentation without collapse controls.

The pinned upstream API is in `vendor/gpui-component-0.7.3/src/sidebar/`:
`SidebarMenuItem::active`, `disable`, and `on_click` supply selected state,
disabled styling, and activation; `Sidebar` owns its retained list, layout,
focus, and keyboard behavior. Its menu renderer derives outer row IDs from item
positions. RustUI semantic IDs drive the active selection and callback value,
but do not replace those upstream outer row paths. No arbitrary stateful row
content or custom virtualization is exposed.

Pure tests cover empty/duplicate IDs and selection mapping across reorder and
label changes. In the M37 initial Review.app native run, the accessibility tree
reported Projects selected; reversing rows preserved selection, and relabeling
Projects to Workspaces left that destination selected. Clicking disabled
Settings left selection unchanged. Removing the selected item showed no active
row; restoring it showed Workspaces active again. Selecting Inbox updated the
selection. The inspected binary SHA-256 is
`c01c33d2fd2582d18c854370470759d439bd12b3c66f923bf4ccb69e38bdd240`.
Evidence: `target/headstart/milestone37-sidebar-gui.json`. Keyboard activation,
focus behavior, and full screenreader acceptance remain pending.
