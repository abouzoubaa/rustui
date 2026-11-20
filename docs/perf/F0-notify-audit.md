# F0-9 RustUI notification audit

Audit date: 2026-11-14. Scope: production Rust source under `crates/rustui/src`; test-only modules excluded. I traced entity `update` / `update_in` closures and `RenderContext::update_entity` calls that mutate view, component, or retained render state. Window/App updates that only operate on platform windows, test harnesses, or app globals are outside this component-state list.

`Yes` means this update directly calls `cx.notify()`, invokes a GPUI state setter that notifies, or invokes RustUI's `ComponentHost` binding writer, which notifies when the owner value changes. `No (render)` means the mutation synchronizes retained state during construction of the current element tree, before that state is read to build the same frame. `No (bookkeeping)` means it changes interaction/cache bookkeeping that does not itself paint output; the visible graph edit goes through the notifying binding writer. No production missing-notify defect was found, so no production code fix or additional test was required.

| File:line | State changed | Notifies? |
|---|---|---|
| `layout/lazy_grid.rs:230,241` | Retained grid seed, cell pruning, generation; row elements are assembled in the active render callback | No (render) |
| `layout/geometry.rs:91,98` | Geometry reader seed/generation and retained child entities in the active container-query callback | No (render) |
| `layout/scroll_view.rs:309` | Retained scroll handle, RTL mode, horizontal extent | No (render) |
| `layout/scroll_view.rs:379` | Window refresh after the scroll view's frame update | Yes (`window.refresh()`) |
| `layout/split_view.rs:324,343` | Retained split-view native handle/configuration and prior layout inputs | No (render) |
| `layout/split_view.rs:423` | Resize panel through GPUI `SplitState::resize_panel` | Yes (GPUI state method) |
| `interaction/focus.rs:137` | Retained focus handle/tab-stop setup while lowering the current element | No (render) |
| `components/advanced/property_inspector.rs:286,314` | Native editor focus/selection/value synchronization | Yes (GPUI editor state methods notify on changed state) |
| `components/data/editable_data_grid.rs:1071,1116,1130,1157,1159` | Current rows/columns and edit validation sync; focus subscription and TableState wiring | No (render synchronization/subscription setup) for 1071/1116/1130/1159; `TableState::refresh(cx)` notifies for 1157. Edits commit through owner updater and notify. |
| `components/advanced/property_inspector.rs:289` | Retained property-row and editor bookkeeping | No (render) |
| `components/advanced/code_editor.rs:233,253,297` | Native editor text, selection, and focus synchronized from current props | Yes (GPUI editor state methods notify on changed state) |
| `components/advanced/code_editor.rs:268,274` | Retained editor/configuration cache used by the current render | No (render) |
| `components/input/combo_box.rs:310,325,336` | Retained disabled/options/owner-selection cache | No (render); confirmation writes owner through notifying binding writer |
| `components/input/combo_box.rs:322,333,384` | GPUI SelectState items, selected value, open/focus state | Yes (GPUI state methods notify on changed state) |
| `components/input/combo_box.rs:351` | Retained confirmed owner value | No (bookkeeping); owner writer below performs notification |
| `components/input/date_picker.rs:277,286,322` | Date selection, open state, and calendar focus state | Yes (GPUI DatePickerState methods notify on changed state) |
| `components/input/picker.rs:232` | Focus restoration and tab-stop setup | No (render); focus operations are applied to Window immediately |
| `components/input/picker.rs:279,309,322` | Retained options, owner selection, and disabled state | No (render); select changes use notifying state methods |
| `components/input/picker.rs:292` | Retained SelectState entity creation/reuse and current focus | No (render) |
| `components/input/picker.rs:306,317` | GPUI SelectState items and selected value | Yes (GPUI state methods notify on changed state) |
| `components/input/picker.rs:341` | Retained confirmed owner value | No (bookkeeping); owner writer below performs notification |
| `components/input/text_field.rs:471,502` | Focus scope and editor props synchronized during render | No (render) |
| `components/input/text_field.rs:519,528` | Retained validation/binding bookkeeping and deferred valid write | Yes (the deferred owner binding writer notifies when the value changes) |
| `components/input/text_field.rs:545,550,590,622,646` | Native input value, selection, focus, and masked state | Yes (GPUI InputState methods notify on changed state) |
| `components/input/text_field.rs:598,606,613` | Read selected range and cache current owner value/binding | No (render) |
| `components/input/slider.rs:239` | Focus restoration/tab-stop setup | No (render); focus operations apply immediately to Window |
| `components/input/slider.rs:264` | Retained slider bounds/value setup | No (render) |
| `components/input/slider.rs:275,379` | GPUI SliderState value and action handling | Yes (`set_value` notifies; action updates its owner through binding writer) |
| `components/input/color_picker.rs:62` | Color picker value/open state | Yes (GPUI state setter notifies) |
| `components/input/color_picker.rs:910,917,926,965` | Retained disabled/writer cache, close when disabled, controlled value sync, trigger focus setup | No (render) for retained cache/focus setup; GPUI `set_open` and `set_value` notify on visible changes |
| `components/input/text_editor.rs:276,283,303,337` | Native editor value/selection/focus | Yes (GPUI InputState methods notify on changed state) |
| `components/input/text_editor.rs:308,313` | Retained editor value/options cache | No (render) |
| `components/navigation/sidebar.rs:234` | Focus and tab-stop setup for current sidebar | No (render); focus operations apply immediately to Window |
| `components/navigation/navigation_split.rs:320,397` | Focus restoration and split state after navigation | Yes (`cx.notify()` in each closure) |
| `components/navigation/navigation_split.rs:512` | Retained narrow-layout mode | No (render) |
| `components/navigation/tab_view.rs:259` | Roving focus and selected tab synchronization | Yes (focus/tab APIs update the current frame; selection writes use notifying owner binding) |
| `components/overlay/drawer.rs:136,158,412,453,486` | Drawer host open/dismiss state and lifecycle | Yes (`DrawerHost::dismiss` notifies; host state mutations notify in their operation) |
| `components/overlay/popover.rs:207,246,255,284,312,316,330,369` | Native popup handle/fallback/anchor bookkeeping | Popup state is consumed on click/render; visible open state is also written through the owner binding updater, which notifies. Anchor bounds are prepaint data for the next click. Lifecycle close removes the popup window. No visible update without an invalidation found. |
| `components/overlay/drawer.rs:428` | Focus recovery for the active drawer lease | Yes (`cx.notify()` in the closure) |
| `components/overlay/drawer.rs:465` | Lease props synchronized while rendering | No (render; comment documents that notifying here loops the root render plugin) |
| `components/data/list.rs:429,446,479,503` | GPUI ListState item count, selection, query, and focus | Yes (GPUI ListState methods notify on changed state) |
| `components/data/tree.rs:700,729,796` | GPUI TreeState selection, disabled/expanded state | Yes (GPUI TreeState methods notify on changed state) |
| `components/data/tree.rs:814` | Retained tree row/entity cache | No (render) |
| `components/data/tree.rs:840,913,927` | Keyboard selection/expand/collapse actions | Yes (`TreeState` setters notify; owner binding writer notifies owner changes) |
| `components/data/table.rs:546,565,583,627` | Table query, selected row/column, and sort | Yes (GPUI TableState setters notify on changed state) |
| `components/data/table.rs:663` | Scroll refresh after table interaction | Yes (`window.refresh()`) |
| `components/data/keyed_list.rs:302` | Keyed-list exit/animation state after data change | Yes (`cx.notify()` when visible rows are removed) |
| `components/data/keyed_list.rs:663,690,719,730` | GPUI searchable/selectable flags, items, selection normalization | Yes for GPUI ListState setters and item replacement (`cx.notify()`); deferred invalid-owner selection uses the notifying binding updater |
| `components/data/collection.rs:479,520,537` | Retained generation, focus/scroll, visible-cell cache | No (render) |
| `components/data/collection.rs:412` | Retained-cell pruning in the current render | No (render) |
| `components/data/collection.rs:586,622` | Selection and keyboard handling | Yes (owner selection binding writer notifies; focus actions update Window immediately) |
| `components/advanced/graph_view/rendering.rs:177` | Retained graph focus/drag/pending/error state normalization for the current element tree | No (render) |
| `components/advanced/graph_view/rendering.rs:423,444-461,706` | Graph drag/pending/error interaction state | Yes for visible pending/error changes (`cx.notify()`); node/edge edits use notifying owner binding writer |
| `components/advanced/graph_view/rendering.rs:781,790,804,809` | Drag gesture bookkeeping; actual graph edits use the owner binding writer | No (bookkeeping); no painted output reads this state between events |
| `view.rs:912,937,954` | Retained child environment refresh, prop reconciliation, and the general retained-entity update wrapper | Prop reconciliation notifies when values change (937-942); environment refresh is read in the active render; generic `update_entity` (954) delegates invalidation to its callback, classified at the component call sites below. |
| `runtime/component_host.rs:396` | Apply owner component mutation | Yes (`cx.notify()` when the component mutation reports a change) |
| `runtime/tasks.rs:116,128,149,173` | Async loading/result/error/idle state; registry cleanup | Yes when `apply` reports a visible component change (`cx.notify()`); cleanup only removes task registry entries |
| `interaction/drag_drop.rs:378,425` | Bound file-drop event value is updated | Yes (RustUI `binding_updater` notifies the owning ComponentHost when the value changes) |
| `interaction/focus.rs:144,150` | Bound focus-in/focus-out value | Yes (binding updater notifies on a changed owner value) |
| `components/input/stepper.rs:253` | Retained focus handle and tab-stop setup | No (render); focus changes apply to Window |
| `components/input/toggle.rs:331` | Disable-time focus recovery for Toggle | Yes when focus is moved (`Window` focus operation schedules the frame); otherwise retained handle setup is current-frame state |
| `components/input/checkbox.rs:419` | Disable-time focus recovery for Checkbox | Yes when focus is moved (`Window` focus operation schedules the frame); otherwise retained handle setup is current-frame state |
| `components/input/radio.rs:379` | Disable-time focus recovery for Radio | Yes when focus is moved (`Window` focus operation schedules the frame); otherwise retained handle setup is current-frame state |
| `components/input/mod.rs:414` | Shared disable-time focus recovery helper | Yes when focus is moved (`Window` focus operation schedules the frame); otherwise retained handle setup is current-frame state |
| `components/input/text_field.rs:693,697,715` | Masked state toggle / focus restoration / clear action | Yes (InputState setter notifies; focus action applies immediately; clear uses notifying owner writer) |
| `components/input/slider.rs:508` | Keyboard value/action state | Yes (SliderState action updates use its notifying setter / owner binding writer) |
| `components/input/color_picker.rs:52,62,296,305,314` | Slider/color value, preview and selection state | Yes (GPUI Slider/ColorPicker state setters notify on changed visible value) |
| `components/overlay/drawer.rs:157,411,438,473,516,523` | Popup window teardown and lease revision checks | Yes for window removal (`remove_window`); lease revision reads do not mutate component state |
| `components/navigation/disclosure.rs:175` | Disclosure value binding | Yes (owner binding updater notifies on change) |
| `components/advanced/graph_view/rendering.rs:423,444-461,706,781,790,804,809` | Drag/pending/error bookkeeping and graph edits | Yes for visible pending/error changes (`cx.notify()`); visible graph edits use notifying owner writer; drag coordinate bookkeeping alone does not paint |
| `adapters/themes.rs:57`, `theme/registry.rs:587` | Global GPUI Kit theme fields and RustUI active-theme registry | Yes (`Theme::update` synchronizes base projection and refreshes every window) |
| `runtime/window.rs:111,137,196,221,332,338` | Window refresh, close, owner teardown and popup anchor bounds | Window lifecycle/geometry operations, not component/view entity state |

## Verification performed

- Existing no-forced-refresh cases remain in `runtime/f0_stale_rendering_tests.rs`: owner/environment/list/transition, theme, text input, grid edit, and popup/menu state.
- The GraphView retained-drag test's test setup now explicitly notifies after restoring its owner graph (commit `d2469db1`); this is test setup, not a production component mutation.
- No additional production `cx.notify()` was added in F0-9 because the audit found no missing RustUI notification path.
