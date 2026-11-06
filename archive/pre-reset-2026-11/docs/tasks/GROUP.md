# Group child-list contract

`Group` collects ordered, potentially heterogeneous RustUI views for insertion
into a layout container. A receiving `HStack`, `VStack`, `Grid`, or `ZStack`
splices the group's children into its own child list before rendering. No GPUI
element, flex or grid item, padding, gap, or retention scope represents the
group itself.

```rust
HStack::new()
    .child(Text("Before"))
    .group(Group::new().child(Text("A")).child(Label("B")))
    .child(Text("After"))
```

The resulting children are `Before`, `A`, `B`, `After`, each subject to the
receiving container's existing layout rules. Nested groups flatten in order.
`Group` is not a `View`: the current `View::into_element` contract returns one
GPUI element, which cannot represent several sibling elements without an
extra wrapper. Thus `Container(Group::new())`, `Frame(Group::new())`, and a
component body returning `Group` are outside this contract.

For dynamic lists, wrap each child in `Keyed(stable_id, child)` before adding
it to a group. The group's position and presence add no identity segment;
keyed descendants retain the same scope under the receiving container as
direct children. Reordering keyed children retains their owned entities;
removing a child releases its retained ownership when the host completes the
render. Duplicate keys in the same scope remain invalid. Unkeyed children do
not gain stable identity from `Group`.

This milestone changes no upstream revision, renderer, scheduling mechanism,
theme, or component semantics. Acceptance requires source review of the
flattening path and the `group_gallery.rs` Playground interaction: compare
grouped and direct HStack, VStack, Grid, and ZStack siblings; increment and
edit keyed retained controls, reorder them, remove B, and restore B while
checking mount/release counts. Compilation and native acceptance are recorded
separately.

## M63 executed acceptance

The M63 review bundle observed matching qualitative item placement for direct
and grouped HStack, VStack, Grid, and ZStack pairs. Keyed A/B/C state and edits
survived reorder to C/B/A. Removing B recorded one mount and one release for B
while A and C remained mounted; restoring B created a fresh child while
preserving A/C state. Evidence is in
`target/headstart/milestone63-group-gui.json`; bundle SHA-256
`e917a357fbe391eb428d0cb029d87fecb41cf7aba6e9dc128804618a4dd5d678`.

The M63–M65 workspace suite passed 200 tests, 0 failed across 12 binaries in
`target/headstart/milestone63-65-final-tests.log`. Build succeeded with all
three pinned GPUI dependencies fresh (`target/headstart/milestone63-65-reuse-report.json`),
and the locked/offline all-targets check exited 0
(`target/headstart/milestone63-65-all-targets-check.log`).

Limits: this accepts `.group(...)` child-list use in the four layouts, not
standalone `View` use. Exact pixels, macro grammar extension, an RTL matrix,
native undo, and full accessibility remain unverified. These observations do
not imply platform-wide or performance acceptance.
