# Stage 9 / R35 GraphView and R36 NodeEditor implementation contract

Source implementation is integrated and has passed the recorded source/build checkpoints. Bounded native and Web functional acceptance is recorded below; wider visual, locale/RTL and workload acceptance remains open. Accessibility and VoiceOver are undone/deferred.

## Authority and architectural scope

`IMPLEMENTATION_ROADMAP.md` M364, `ROADMAP_COMPLETION_GATES.md` M364 and
`D1C_COMPONENT_CATALOG_COVERAGE.md` identify GraphView and NodeEditor as missing
catalog APIs. `ROADMAP_REMAINING_AUDIT.md` R35/R36 requires identity/topology,
viewport, selection, editing/validation, ownership, keyboard and accessibility
contracts. The original entries provide no domain-specific layout, workload,
port-cardinality or graph-algorithm requirements. This implementation supplies a
conventional positioned graph and node editor using the approved Canvas/GPUI
path. It does not choose a graph-layout engine or change upstream revisions.

## Public data and validation

- `GraphNode` has a stable caller ID, nonblank display name, finite positive
  world rectangle and ordered named input/output ports. Port IDs are unique
  within a node. Caller geometry controls layout; there is no implicit layout.
- `GraphEdge` has a stable caller ID and typed endpoint pairs. `GraphDocument`
  validates uniqueness, references, output-to-input direction and duplicate
  endpoint pairs. Cycles, self connections and fan-out are permitted. There is
  no business-specific acyclic/type/cardinality rule.
- Nodes/ports/edges are immutable after validation. `GraphEdit` supports adding,
  renaming, moving and removing nodes, connecting and disconnecting ports.
  `GraphDocument::edited` returns a validated replacement or `GraphError`;
  failure preserves the original. Removing a node removes its incident edges.
- `GraphViewport` has finite origin, positive finite zoom and `CanvasSize`.
  Projection uses checked f64 intermediates before reaching Canvas. Unrepresentable
  views display an accessible Reset view control. Keyboard viewport updates which
  cannot project the current nodes are rejected. No document data is discarded.
- `GraphStyle` exposes checked HSLA node/text/edge/selection colors. Callers may
  supply explicit colors from the public theme API. Without an override, the
  surface resolves current RustUI semantic panel/text/border/accent colors on
  every render (existing GPUI semantic colors are the fallback). Disabled text
  follows the existing semantic disabled color. No runtime theme schema is added.

## Controlled interaction and lifecycle

`GraphView(id, document, selection, viewport)` renders Canvas edges and GPUI
node groups with visible names and port labels. Selection and viewport remain
owner bindings. A removed or unknown selection renders as unselected without
rewriting the controlled binding. Pointer selection, arrows/brackets, Enter,
Space and AX Click operate on stable node IDs. Double click, Enter and AX Click
invoke an optional activation action after selecting the node.

`NodeEditor(id, document_binding, selection, viewport)` adds pointer dragging,
keyboard movement/deletion, focusable port connection controls and named
Disconnect controls. Select any output and input in either order to connect;
invalid pairs leave the document unchanged and show the validation error.
Owner toolbar actions use the same public `GraphEdit` API for add/rename/remove.
No implicit graph or node creation policy is imposed by the widget.

With the surface focused, editor arrows move by ten world points, brackets
change selection and Delete/Backspace removes the selected node. In a GraphView,
arrows select adjacent nodes in caller order. A/D/W/S pan by forty screen points,
PageUp/PageDown scale by 1.25/0.8 and Home resets origin/zoom. Escape cancels the
pending connection/drag and clears selection. Normal Tab traversal leaves the
surface and reaches node/port/connection controls. Modified keys, IME and
preferred character input are not intercepted. Ports use Enter/Space and AX
Click. Accessibility labels include node relationships and both port names;
node groups and connection/port controls expose their selected/disabled state.
Disabled surfaces and nodes leave keyboard traversal and blur an already focused
subtree. Enabled node/port/connection controls have visible keyboard focus;
Disconnect controls have a 24-point target and restore surface focus. Escape
cancels a connection even while a descendant port has focus.

GPUI retains focus and current gestures. A separate retained weak epoch token
invalidates older callbacks on every render and on omission, including callbacks
which still retain a gesture entity. Disabling/removing a node cancels its drag;
removing a pending endpoint cancels connection staging. Release and movement
without the left mouse button cancel the drag. Losing surface focus or changing
the viewport also cancels a drag. Native element IDs encode length-prefixed
identity tuples, so colons or Unicode in valid IDs cannot collide. Latest-owner event updates reuse
RustUI's existing component dispatch and `Binding::update_on` path, preserving
intervening owner edits and binding revision rejection. Keyboard movement is
relative to the latest owner geometry; repeated selection events advance from
the latest owner selection before the next render. Invalid/stale edit events
are no-ops. No second scheduler, renderer, reactive runtime or dependency exists.

## Playground and prepared regressions

Standalone searchable GraphView and NodeEditor stories (also reused by the
foundation Graphs tab) have public-API implementations with
real topology, owner add/rename/remove/reset actions, activation counts, viewport
instructions, disabled transitions and omission/remount controls. The compiled
preview registry mounts both real local components using property-free schemas;
no graph/state serialization contract is implied by those specimens.

Prepared checks cover validated topology/references/direction, atomic failure,
cycles and incident-edge cleanup, latest-document edits, finite/overflow geometry,
callback lifetime, stable keyboard selection and native key-event eligibility.
The actual framework event-dispatch helper has owner-current-value and rejected
binding-revision regressions. Playground tests exercise owner edits and the real
preview registry mount/render path. Integration validation on 2026-10-25 passed 13 graph regressions, two existing
production binding-event regressions, and all 193 Playground library tests.
The macOS release checkpoint then passed 1,250 workspace library tests, 43
integration tests, all-target compilation, formatting and an optimized Playground
build. The macro doctest target contained zero cases. These are source/build
results; GUI, platform accessibility and performance acceptance remain separate.

## Acceptance still required

Run focused library/Playground tests and the approved locked/offline all-target
Headstart check after integration. Then validate real drag/release outside the
surface, connection creation/rejection/removal, focus traversal, keyboard movement,
viewport clipping/zoom, mounted-owner replacement and disabled/omitted transitions
in the canonical reusable Playground bundle. Inspect the focused native AX tree
and VoiceOver behavior; check browser semantics, keyboard/IME, fonts, locale/RTL,
platform behavior and representative large graph workloads separately. Prepared
source regressions and successful compilation cannot close those gates.

## Current-selection event dispatch

The keyboard handler reads selection and the editor document through the existing
synchronous owner dispatcher before move/delete/activation. An earlier select-next
event therefore affects a subsequent command even before a redraw. Detached owners
and IDs missing from the current document do not activate or edit. Relative movement
still applies against the latest document inside the binding updater. This covers
component and application activation actions without a new runtime.

## Bounded Web functional acceptance — 2026-10-26

Current optimized Web source `e50b6ff0` passes **9/9 GraphView** and **10/10
NodeEditor** functional checks in one Chromium profile at 1280×720. GraphView
covers pointer/key selection, activation, disabled rejection, unmount/remount,
zoom, reset and pan. NodeEditor covers add/rename/remove, visible disconnect,
connection from cleared pending state, drag, keyboard movement, disabled rejection,
and unmount/remount preserving the document. Graph zoom changed the source node
from 150×100 to 187.5×125 CSS pixels; Home restored it and D panned by −40 pixels.
Node drag moved by (60,30), then Right moved by (10,0); a verified disabled editor
rejected both drag and Right with unchanged geometry.
Evidence: `target/headstart/evidence/web-outline-drawer-20261026/acceptance.json`.
Unsettled snapshots, offscreen-control attempts and uncleared pending-port
sequences are excluded, with the accepted cases repeated from verified states.
Native, wider browser/visual/locale/RTL and representative workload checks remain
open. Accessibility and VoiceOver are undone and deferred by user direction.

## Bounded native functional acceptance — 2026-10-26

The verified reproducible native executable `d857e6b5b687951bcb20fd4fff0e1be2abecf7e2eb3b0c8ff696186e8acbffcf` passes **11/11 GraphView** and **16/16 NodeEditor** bounded functional checks on macOS 27 arm64 at 1320×820 logical pixels. Application/runtime inputs are unchanged from controlled release build `4e00ad14`.

GraphView covers initial topology, pointer/arrow selection, Enter activation, Escape, zoom, Home reset, pan/clipping, disabled pointer/keyboard rejection and omission/remount. NodeEditor covers owner add/rename/remove/reset, stable selection, drag, keyboard movement, invalid-direction feedback, input-first and output-first connections, deletion with incident-edge cleanup, disabled drag/movement/deletion rejection and omission/remount.

Rendered geometry observations at 2× scale: GraphView Source grows approximately 300×200 → 375×250 image pixels on PageUp; Home restores default scale; D moves Transform left approximately 80 image pixels. NodeEditor drag moves Source approximately (+120,+60) image pixels and Right adds (+20,0), consistent with (+60,+30) and (+10,0) world points. Disabled geometry remains unchanged. These are bounded screenshot observations, not latency/FPS metrics.

The first AX-index Disconnect snapshot did not prove removal, so that attempt and the subsequent inconclusive reconnect assertion are excluded. Accepted pointer disconnection verifies zero connections in both rendered and owner evidence before reconnection. No AX action-delivery acceptance or root cause is claimed.

Evidence: `target/headstart/evidence/native-graph-editor-20261026/`, including full state records, screenshots, exclusions, binary/archive hashes and cleanup. The owned test app and disposable bundle are removed; caches and the archive are retained. Release outside the surface, exhaustive focus traversal, IME, locale/RTL, broader visual and representative large-graph workloads remain open. Accessibility and VoiceOver remain undone/deferred.
