# X1B — Web accessibility host patch proposal

## Decision boundary

M1285 is a narrow proposal to prepare and review a versioned patch for the
Playground search field's browser accessible name. It is ordered after the
approved M855 v3 keyboard/focus patch and targets the same pinned package pair
(`gpui-pre = 0.3.9`, `gpui-pre-web = 0.3.9`). It does not authorize patch
application, dependency changes, or acceptance claims. See
`docs/tasks/M1285_STAGE11_WEB_SEARCH_ACCESSIBLE_NAME_BOUNDARY.md` for source
identities and the narrow acceptance oracle.

This M1285 slice does not implement the broader X1B host adapter. It does not
expose every GPUI control, map all AccessKit state and geometry, route
accessibility actions, or close Stage 11.

## Corrected source boundary

GPUI creates `TreeUpdate`s from its per-frame AccessKit builder and sends them
through `PlatformWindow::a11y_tree_update` (`vendor/gpui-pre-0.3.9/src/window.rs`,
around line 3795). That path alone does not mean Web receives updates:

- The setup that installs `A11yCallbacks` is cfg-gated to non-WASM in
  `vendor/gpui-pre-0.3.9/src/window.rs` near line 1669. Without a WASM active
  flag, GPUI does not build or submit frame tree updates.
- `handle_a11y_action` is also non-WASM-only near line 7033.
- The platform trait supplies no-op defaults for accessibility callbacks, and
  the pinned `WebWindow` has no overrides.

Therefore the existing Web host has no received tree to project. A host-only
patch cannot solve M1285. The package requires a small core change to enable
tree production on WASM (while respecting `new_inaccessible()`), followed by
a Web projection for the focused text input. Action callbacks are not needed
for this name-only projection and remain out of scope.

The Playground's `TextField` supplies `Search N components`; GPUI Components
uses its placeholder as the default semantic label. Web already owns a focused
IME textarea whose value and selection must remain under the IME mirror's
control. M1285 reuses that textarea and projects only the focused AccessKit
text-input label to its `aria-label`. The shell's initial generic label must
be cleared when the focused node has no label.

## Proposed M1285 package

1. Prepare one hash-checked patch across the pinned GPUI and GPUI Web source
   pair, then review it without modifying vendored source. Use the existing
   M855 same-version local override procedure after approval to apply it.
2. In core, make the AccessKit frame tree active on WASM unless accessibility
   was forcibly disabled. Keep native activation and action handling intact;
   do not imply browser action dispatch is implemented.
3. In Web, consume updates and use the focused node's role and label to set or
   remove `aria-label` on the existing IME textarea. Leave textarea value,
   selection, and focus to the current IME bridge.
4. Keep RustUI APIs, Playground controls, dependency revisions, and new
   dependencies unchanged.

The browser has no dependable signal for an attached screen reader, so the
semantic tree must stay current on Web. Measure the resulting tree-build cost
after separate application approval. Do not report this focused-field
projection as a complete AccessKit DOM adapter.

## Acceptance boundary

After approval and application, verify against a fresh hash-bound bundle in
current Chrome on macOS. Record browser and OS versions and the AX snapshot.
The search entry should occur exactly once as a focused textbox, named
`Search N components`, with its current query value. Typing should filter
results. With VoiceOver enabled, verify the announced name, role, and value.
Repeat M855 keyboard focus and Space-activation checks, and inspect that label
updates do not disturb IME value or selection.

This focused-field check does not establish full control-tree exposure,
screen-reader action support, IME acceptance, other browser/OS combinations,
font or glyph coverage, or Stage 11 completion. No new browser run or patch
application is claimed in this proposal.
