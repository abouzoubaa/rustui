# M66 typed same-process drag and drop

`DragSource(id, value, child)` and `DropTarget(id, binding, child)` are RustUI
wrappers over pinned GPUI's `on_drag` and typed `on_drop`. The source copies a
`Clone + 'static` value into GPUI's native active drag, which owns the payload
for the gesture. The target accepts exactly the same Rust type and writes an
owned `DropEvent<T>` to a component-owned
`Binding<Option<DropEvent<T>>>`. The event includes the source and target IDs,
Create that binding from the component currently rendering the target.
A binding captured from another owner or an earlier owner instance is not
a supported event channel. `DropEvent<T>` implements `PartialEq` when `T`
does, so `#[state] Option<DropEvent<T>>` can receive it. The Playground
shows both a direct state sink and a custom model-applying writer.
value, and `DragOperation::{Copy, Transfer}` intent. The binding owner applies
that intent to its model. RustUI does not automatically remove source data,
promise an atomic cross-owner move, or synthesize a click event.

The wrappers use their own retention scopes and stable GPUI IDs. A dynamic
collection should also wrap each source or target in `Keyed(stable_id, ...)`.
The target's binding update goes through the existing weak component owner
callback; an event targeting an unmounted owner does not mutate it. GPUI owns
hit testing, threshold, drag preview, payload type matching, drop dispatch,
and propagation. Disabled wrappers leave their children rendered without
registering a drag or drop listener.

`apps/rustui-playground/src/stories/drag_drop_gallery.rs` demonstrates String
copy and transfer within one component owner, a mismatched `usize` source,
source reorder, target removal, and keyboard-equivalent copy and transfer
buttons. The gallery's transfer mutates the source and destination in one
component binding write. Other applications decide their own model policy.

This milestone covers basic in-process pointer drag/drop. It does not claim OS
file dragging, cross-window transport, touch support, external payloads, drag
GPU optimization, or accessible drag gestures. Applications must provide a
keyboard path for the same operation; the gallery includes one.

## Acceptance

1. Drag A/B/C into the target in Copy mode. Confirm the source remains and
   the destination gains the value once.
2. Switch to Transfer, drag an item, and confirm it leaves the source and
   appears once in the destination. Reverse sources and repeat; identity must
   follow the item rather than its position.
3. Drag the `usize` source over the String target and confirm no event.
4. Remove the target while a drag is active and release. Confirm no stale
   target event or crash, then restore and retry. This removes a target
   subtree while the gallery component owner remains mounted; it does not
   establish owner-release or cross-owner cancellation behavior.
5. Use the keyboard alternative buttons and confirm equivalent model changes.

## Bounded native evidence

M66's core typed pointer copy/drop/transfer flow was delivered and announced.
The native CUA bundle at `target/headstart/milestone66-drag-drop-gui.json`
records copy into the destination, a component-state drop, transfer removing a
source item, reordered-source behavior, a second copy and transfer, target
removal/restoration, and rejected numeric-type or disabled-source/target drops.
This is bounded pointer evidence for the same-process typed flow; unchanged AX
on rejected drops is only negative evidence, not a native event trace or broad
accessibility acceptance.

Keyboard-only alternative acceptance remains pending. Actual destruction of
the component owner while a drag is in flight was not tested; removing the
target subtree while its gallery owner remained mounted is a different case.
OS/file drag, cross-window transport, touch, cross-owner binding and
exact-pixel/performance/full accessibility acceptance also remain outside this
evidence.
