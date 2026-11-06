# M608 — D1c implemented component catalog coverage audit

## Result

The canonical compiled preview gallery has a specimen for every implemented
public component listed in `docs/COMPONENT_CATALOG.md`. This audit covers the
`crates/rustui/src/components/` catalog; M359 layout specimens and the M361
PropertyInspector specimen remain separately recorded milestones.

The 53 implemented public component APIs map to these registered gallery IDs:

| Family | Implemented API coverage |
| --- | --- |
| Basic | `Avatar` → `avatar-specimen`; `Badge` → `badge-specimen`; `Button` → `button-specimen`; `Divider` → `divider-specimen`; `Icon` → `icon-specimen`; `Image` → `image-specimen`; `Label` → `label-specimen`; `Link` → `link-specimen`; `Text` → `text-specimen`; `LocalizedText` → `localized-text-specimen`. |
| Input | `Checkbox` → `checkbox`; `ColorPicker` → `color-picker-specimen`; `ComboBox` → `combo-box-specimen`; `DatePicker` → `date-picker-specimen`; `Picker` → `picker`; `Radio` → `radio`; `SearchField` → `search-field-specimen`; `SecureField` → `secure-field-specimen`; `Slider` → `slider`; `Stepper` → `stepper`; `TextEditor` → `text-editor-specimen`; `TextField` → `text-field-specimen`; `TokenField` → `token-field-specimen`; `Toggle` → `toggle`. |
| Data | `Collection` → `collection-specimen`; `DataGrid` → `data-grid-specimen`; `KeyedList` → `keyed-list-specimen`; `List` → `list-specimen`; `Table` → `table-specimen`; `Tree` → `tree-specimen`. |
| Feedback | `EmptyState` → `empty-state-specimen`; `ErrorState` → `error-state-specimen`; `Progress` → `progress`; `Skeleton` → `skeleton-specimen`; `Spinner` → `spinner-specimen`. |
| Navigation | `Breadcrumb` → `breadcrumb-specimen`; `Disclosure` → `disclosure-specimen`; `NavigationSplit` → `navigation-split-specimen`; `NavigationStack` → `navigation-stack-specimen`; `Sidebar` → `sidebar-specimen`; `TabView` → `tab-view-specimen`; `Toolbar` → `toolbar-specimen`. |
| Overlay | `Alert` → `alert-specimen`; `ContextMenu` → `context-menu-specimen`; `Dialog` → `dialog-specimen`; `Menu` → `menu-specimen`; `Popover` → `popover-specimen`; `Sheet` → `sheet-specimen`; `Toast` → `toast-specimen`; `Tooltip` → `tooltip-specimen`. |
| Advanced | `Canvas` → `canvas-specimen`; `CodeEditor` → `code-editor-specimen`; `PropertyInspector` → `property-inspector-specimen`. |

The gallery also retains its original `text`, `button`, and `text-field`
starter entries. Those do not add component APIs to the 53-item coverage count.

## Excluded scaffold modules and contract gaps

The remaining five component-catalog entries have reserved source modules but
no implemented public component API or behavior contract. They are not suitable
for compiled-local factories yet; no placeholder specimens were added.

The catalog previously described `Outline` as “Tree-based outline composition.”
That was ahead of the source: `outline.rs` explicitly marks itself as a planned
scaffold and defines no public type or runtime behavior. The catalog now labels
it planned, so it remains outside the implemented API count until its contract
is approved and implemented.

| Planned entry | Source | Missing contract before a specimen can be defined |
| --- | --- | --- |
| `Drawer` | `crates/rustui/src/components/overlay/drawer.rs` | Presentation edge and sizing, trigger/open state, dismissal rules, and focus behavior. |
| `Outline` | `crates/rustui/src/components/data/outline.rs` | Relationship to Tree, row identity, selection/expansion, and activation behavior. |
| `ResizablePanel` | `crates/rustui/src/components/advanced/resizable_panel.rs` | Panel composition, size constraints, resize input/keyboard behavior, and retained sizing. |
| `GraphView` | `crates/rustui/src/components/advanced/graph_view.rs` | Node/edge data model, layout contract, interaction, and accessibility behavior. |
| `NodeEditor` | `crates/rustui/src/components/advanced/node_editor.rs` | Node/port/connection model, editing operations, persistence, and interaction/accessibility behavior. |

D1c accepts bounded descriptor values and builds actual RustUI views inside
compiled-local factories. It does not serialize Rust closures, GPUI entities,
or component behavior, and it cannot supply a missing component API or decide
its product semantics. The five entries outside specimen coverage are planned
scaffolds, not implemented APIs: Drawer, Outline, ResizablePanel, GraphView,
and NodeEditor. Their source modules explicitly defer implementation pending
separate approval; this coverage audit applies only to implemented components.

## Validation

- At the original M608 snapshot, the Gallery regression suite passed 70 tests, including the catalog coverage assertion for all 52 APIs. Log: `target/headstart/evidence/d1c-component-catalog-coverage-tests-m608-20261021.log`.
- Playground compile check: `./scripts/cargo-headstart check -p rustui-playground --all-targets --locked --offline` — passed. Log: `target/headstart/evidence/d1c-component-catalog-coverage-check-m608-20261021.log`.

This source-level coverage audit does not establish GUI rendering, interaction,
accessibility, locale, theme, RTL, or cross-platform acceptance for every
specimen.

## Supplemental layout preview coverage (M687)

The broader compiled-preview coverage regression also includes the registered
Card, Container, Grid, HStack, ScrollView, SplitView, VStack, and ZStack
specimens. M687 adds the public GeometryReader API as `geometry-reader-specimen`;
its bounded schema and round-trip contract are recorded in
`docs/tasks/D1C_GEOMETRY_READER_PREVIEW_SPECIMEN.md`. The supplemental list
extends the test beyond the 53 `components/` APIs audited above and does not
change the 53-component count.

## M908 — TokenField compiled-preview specimen (2026-10-23)

The public `TokenField` API now has a compiled-local `token-field-specimen`.
Its editable properties are `placeholder`, `draft`, and `tokens_json`. The
descriptor schema remains scalar-only, so `tokens_json` stores a bounded JSON
string array; the compiled factory decodes it to the component's controlled
`Vec<String>` binding. Invalid arrays are rejected on inspector edits and
before descriptor import remounts the preview. Import/export preserves duplicate
tokens and the current draft and placeholder. The catalog registry now covers
74 specimens total, including the original local fixtures and supplemental
layout specimens.

The focused regression
`token_field_specimen_round_trips_controlled_tokens_and_rejects_invalid_lists`
checks the editable schema, controlled token feedback, duplicate preservation,
normalized descriptor round trip, and invalid-list rejection. The focused
locked/offline test passed 1/1 using
`./scripts/cargo-headstart test -p rustui-playground --lib stories::compiled_preview_gallery::tests::token_field_specimen_round_trips_controlled_tokens_and_rejects_invalid_lists --locked --offline`.
This is source-level coverage only; native rendering, interaction,
accessibility, locale, theme, RTL, and cross-platform acceptance remain
separate.
