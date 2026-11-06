# M71: Typed PropertyInspector v1

## Scope

Implement a controlled property editor composition using the existing RustUI
`TextField`, `Toggle`, and `Stepper` adapters. V1 supports `Binding<String>`,
`Binding<bool>`, and `Binding<i32>` rows. The integer type follows the current
`Stepper` public signature.

## Contract

- Each inspector and row has an explicit stable, nonblank ID. Row IDs are unique
  within an inspector; constructor validation rejects blanks and duplicates.
- Rows render in caller order and use keyed retention so reordering does not
  associate a retained input with a different row.
- Each row binding belongs to the same component owner that renders the
  inspector. The inspector forwards typed bindings to native RustUI controls;
  it does not add reflection, serialization, or cross-owner binding support.
- Inspector and row disabled states are passed to the native control builders.
- The inspector identity establishes a typed retention scope, so sibling
  inspectors may safely reuse row IDs.

## Excluded

No arbitrary property metadata, validation rules, nested groups, dynamic type
erasure, cross-owner bindings, GPUI/Allui runtime changes, or new dependencies.

## Evidence and verification

Unit tests cover ID validation, caller row order, stable row IDs through reorder,
and writes through each typed binding to its owner. The Playground gallery covers
all three editors, row reordering, live values, and a disabled integer editor.
The final M69–M71 workspace suite passed 236 tests across 12 binaries, the
build succeeded with all three pinned GPUI dependencies fresh, and the
locked/offline all-targets check exited 0. Bounded native acceptance is complete
and recorded in `target/headstart/milestone71-property-inspector-gui.json`. The
The Name TextField received its row label as a native accessibility label, and
the native control exposed the accessible name “Name”. Independently edited
primary and sibling `name` rows retained separate owner values. Toggle and integer bindings updated
the owner, row reorder preserved values, and disabled row/inspector interactions
left values unchanged. Reenabling controls allowed further edits.

This does not establish native cursor/undo preservation, inspector-removal
behavior, full keyboard-only or accessibility-speech acceptance, locale/theme
coverage, or performance. Bindings remain limited to the rendering owner; no
reflection or serialization is provided.
