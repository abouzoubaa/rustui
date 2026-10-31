# M66 typed same-process drag and drop

`DragSource(id, value, child)` and `DropTarget(id, binding, child)` wrap the pinned GPUI `on_drag` and typed `on_drop` APIs. A source owns a cloned payload for the native drag gesture. A target accepts only the same Rust type and writes a `DropEvent<T>` to its component-owned `Binding<Option<DropEvent<T>>>`.

`DropEvent<T>` records the source ID, target ID, value, and `DragOperation::{Copy, Transfer}` intent. Create the binding in the component currently rendering the target. A binding captured from another owner or an earlier owner instance is not a supported event channel. The binding owner applies the operation to its model; RustUI does not remove source data or promise an atomic cross-owner move. `DropEvent<T>` implements `PartialEq` when `T` does, so it can be stored in `#[state] Option<DropEvent<T>>`.

The wrappers use stable GPUI IDs and owner-scoped retention. Dynamic collections should wrap each source and target in `Keyed(stable_id, ...)`. Target updates go through the existing weak owner callback, so an event for an unmounted owner does not mutate it. GPUI owns hit testing, drag threshold, preview, payload type matching, drop dispatch, and propagation. Disabled wrappers render their children without registering drag or drop listeners.

The canonical Playground story at `apps/rustui-playground/src/stories/drag_drop_gallery.rs` demonstrates String copy and transfer in one component owner, a mismatched `usize` source, source reordering, target removal, and keyboard buttons for equivalent Copy and Transfer model operations. The buttons operate on the first source item.

This milestone covers basic in-process pointer drag and drop. It does not cover OS file dragging, cross-window transport, touch, external payloads, GPU optimization, atomic cross-owner moves, or accessible drag gestures. Applications provide a keyboard path for the same operation.

## Acceptance

1. Drag A, B, and C into the target in Copy mode. Confirm the source remains unchanged and each value appears once in the destination.
2. Switch to Transfer, drag an item, and confirm it leaves the source and appears once in the destination. Repeat after reversing the source order; identity follows the item rather than its position.
3. Drag a `usize` source over the String target and confirm no event is delivered.
4. Remove the target subtree during an active left-button drag while the gallery owner remains mounted; release, confirm the destination stays unchanged and the app remains responsive, then restore the target and retry.
5. Remove the whole gallery owner during an active left-button drag; release and confirm the parent remains responsive after the child owner is gone.
6. Use only the keyboard to activate Copy and Transfer alternatives and confirm equivalent model changes.

## Bounded native evidence

The native CUA bundle at `target/headstart/milestone66-drag-drop-gui.json` records pointer copy and transfer, reordered-source behavior, target removal and restoration, and rejected incompatible-type and disabled-source/target drops. This is bounded evidence for the same-process typed pointer flow. Unchanged accessibility output on rejected drops is negative evidence only, not a native event trace or broad accessibility result.

M142 verified keyboard-only Copy and Transfer alternatives in canonical Playground. See `target/headstart/evidence/m66-keyboard-alternatives-m142-20261019.json`. M146 verified whole-gallery child-owner removal while a native left-button drag was held over the target; the parent showed the owner-released placeholder before mouse-up, and the app remained responsive afterward. See `target/headstart/evidence/m66-owner-destroyed-during-drag-m146-20261020.json`. M148 verifies removal of only the target subtree during a held native drag while the gallery owner and source remain mounted; the destination stayed empty through release, target restoration succeeded, and a retry copied A. The live snapshot confirmed the left button remained down while the target was absent. See `target/headstart/evidence/m66-target-removed-during-held-drag-m148-20261020.json`. This is bounded negative evidence, not an internal GPUI cancellation or event trace. Buttons operate on the first source item. Screen-reader announcements, cross-platform behavior, broader accessibility, exact-pixel, performance, cross-window, touch, and OS file-drag acceptance remain outside evidence.
