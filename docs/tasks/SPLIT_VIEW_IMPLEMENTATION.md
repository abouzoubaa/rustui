# SplitView implementation and native review

`SplitView::horizontal(id)` and `SplitView::vertical(id)` arrange arbitrary
RustUI child views in official GPUI Components resizable panels. Each panel
has an explicit semantic identity:

```rust
SplitView::horizontal("editor")
    .panel(SplitPanel::new("sidebar", Text("Files")).initial_size(180.0))
    .panel(SplitPanel::new("document", Text("Document")))
```

Wrap the split in a parent with bounded width and height. Panels default to a
100-point native minimum. `.size_range(min..max)` changes native constraints;
`.initial_size(points)` applies on first insertion. Invalid dimensions, empty
IDs, and duplicate panel IDs panic before reaching the native layout engine.
An empty split renders an empty full-size element, discards previous panel
preferences, and installs no native panel group. A single panel uses official
panel layout without a divider. Its configured minimum still applies; a parent
smaller than that minimum can overflow. Call `.size_range(0.0..max)` explicitly
when a smaller extent is required. The split compares the effective per-panel
ranges on each render. Unchanged ranges keep the native resize state; a changed
range reconciles measured sizes by panel ID, clamps them to the new ranges, and
seeds fresh native state. GPUI applies the resulting layout constraints.

## Ownership and topology policy

The enclosing `ViewHost` retains one private split entity under a distinct
`SplitView` retention and native element namespace. Neighboring `Keyed`,
Popover, or ContextMenu wrappers may use the same textual ID without colliding;
duplicate sibling SplitViews remain rejected. Panel keys are nested inside
that split scope. The split ID owns the private entity.
That entity owns the official `ResizableState`; it has no owner reference.
Native drag handling, layout, notifications, fitting, divider appearance, and
resize constraints stay in GPUI and GPUI Components. There is no additional
reactive runtime, global size store, dependency, or copied upstream code.

An unchanged axis, ordered panel-ID list, and range list retain the same native
state.
Panel-ID retention scopes preserve child component/control state across
reorder. Identical child control IDs in different panels are isolated.
The native resizable panels have positional element IDs. Reordering a keyed
panel therefore changes its native descendant element-ID path even while its
RustUI retained child component survives. Native focus and accessibility-node
identity across reorder have not been accepted.

The upstream native state stores sizes by position and has no public reorder
operation. A topology or range change therefore seeds fresh native state from
measured size preferences matched to surviving panel IDs and clamped to current
panel ranges. Upstream layout can fit and
clamp these preferences to the new container and constraints; exact pixels or
proportions are not promised through topology changes. A panel omitted from a
render releases its retained child state and size preference. Reintroducing
that ID starts from its requested initial size and a fresh child. Omitting the
whole split releases its native state. Changing axis resets size preferences
because width measurements are not height preferences; surviving child state
remains associated with its panel ID.

Before a fresh native group is prepainted, its positional `sizes()` entries may
still be upstream 100-point placeholders. Reconciliation consumes those entries
only when the public `container_size()` is positive; otherwise it falls back to
the retained initial preferences, then applies the current ranges. It does not
filter the number 100: a measured 100-point panel remains valid.

This guard follows the pinned upstream lifecycle: `ResizableState` starts with
zero group bounds and fills newly synchronized slots with placeholders
(`vendor/gpui-base-0.7.3/src/resizable/mod.rs:33-50`, `174-190`). The public
container extent reads those group bounds (`mod.rs:265-268`). The group appends
its bounds observer after its panel children (`resizable/panel.rs:163-187`),
and `ElementExt::on_prepaint` implements that observer as an appended canvas
(`src/element_ext.rs:15-27`). Under this adapter's fresh-state policy, a positive
group extent therefore follows the initial panel prepaint pass. Zero-extent
groups continue using initial preferences until a measurable layout exists.

## Evidence and pending checks

Source implementation: `crates/rustui/src/layout/split_view.rs`.
Public-only gallery: `apps/rustui-playground/src/stories/split_view_gallery.rs`.
Unit checks cover topology mapping, removal/reinsertion, empty and single-panel
topologies, axis reset, unmeasured preferences, range-only narrowing versus an
unchanged range including zero minimum, namespace isolation, duplicate
scopes/IDs, and invalid dimensions. A bounded native review observed a vertical
C/B divider drag and its size persisting after a parent notification. This
range-reconciliation change passed the 135-test workspace suite and final
Playground build. Native review verified C3/B2/A1 after reversal, B release and
fresh B0 reinsertion, preserved surviving counters on axis change, and narrowing
and restoring A's range without remounts or losing A1. Evidence and executable
hashes: `target/headstart/milestone46-split-view-gui.json`.
The later measured-readiness guard covers a prepaint placeholder of 100
retaining initial 180, a measured 100 remaining valid, and fallback 180
clamping to updated maximum 150; the focused SplitView suite now passes 13/13.
Horizontal drag and post-range-change drag continuity remain unverified. Each
boundary now includes a focusable `splitter` node: Left/Right adjusts a
horizontal split, Up/Down adjusts a vertical split, and the change goes through
the pinned public `ResizableState::resize_panel` API. Its keyboard clamp and AX
minimum/maximum use the same measured feasible interval derived from the
container extent and all sibling ranges. Before measurement, the configured
range remains the fallback; incompatible constraints pin the interval at the
current size. Focused SplitView tests passed 13/13
(`target/headstart/evidence/m525-split-view-tests-20261021.log`).
Native focus traversal, keyboard resizing, screen-reader announcements, and
full GUI acceptance remain unverified; this source change does not close those
native acceptance gates.

The separator uses the `Splitter` accessibility role; its focusable 9-point
interaction area is the resize handle. Its accessible name resolves the
`rustui-splitter-resize-divider` catalog message for English, French, or Arabic.
The visual divider/gutter and pointer-drag behavior remain owned by GPUI
Components. M580's catalog contract and RustUI compile check passed; native AX
tree exposure, VoiceOver announcement, and locale-switching behavior remain
unverified.

In Playground, increment each panel, resize native dividers in both axes, and
force a parent render with **Observe lifecycle**. The unchanged arrangement
must retain its resize state and counters. Reverse the order and confirm
counters follow panel IDs without additional mounts. Remove B, observe its
release, then insert B and confirm its counter restarts at zero with a new
mount. A and C must retain their counters. Switching axis resets size
preferences while retaining surviving child counters. The gallery reports
per-panel mounts/releases after **Observe lifecycle** refreshes the owner.

Unit model checks alone do not establish native resizing, focus preservation,
accessibility acceptance, or exact size behavior after membership changes.

## M586 — bounded native interaction and lifecycle acceptance

The canonical Playground exposed two adjustable `Splitter` nodes named
“Resize divider.” Horizontal and vertical pointer drags and keyboard arrows
changed the native values. A parent observation retained pane sizes; reversing
the keyed panels retained distinct counters without extra mounts. Switching
axis reset size preferences while keeping counters. Removing B released it;
reinserting it mounted a fresh zero-count B while A and C retained their counts.
Narrowing and restoring A’s range did not change panel mounts. Exact AX frame
bounds, VoiceOver announcements, locale switching, complete focus traversal,
and broader visual/platform acceptance remain open. Evidence:
`target/headstart/evidence/m586-splitter-native-interaction-20261021.json`.

## Direct Playground route — 2026-10-26

The existing public-API gallery is now directly reachable as **SplitView +
ResizablePanel** in Framework Demos. Search matches either API name; the Stacks
foundation tab continues to reuse the same gallery. No panel implementation or
state owner is duplicated. The source example shows both APIs. Six focused
registry/search regressions and the optimized Web build passed; browser/native
functional acceptance of this direct route is recorded separately.
