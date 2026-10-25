# Sheet close-button accessible name proposal

## Status and finding

Review-only proposal for pinned `gpui-component 0.7.3`. It is not applied to `vendor/` and does not authorize a dependency revision change or RustUI integration.

The native Sheet builds its close control as an icon-only `Button::new("close")` in `vendor/gpui-component-0.7.3/src/sheet.rs:201-207`. The public Sheet builder has no close-name setter (`:76-118`). The pinned Button already exposes `Button::accessibility_label` (`vendor/gpui-component-0.7.3/src/button/button.rs:393-396`) and forwards it to the rendered button (`:746-819`). RustUI's `SheetView::accessibility_label` and localized counterpart name the sheet content (`crates/rustui/src/components/overlay/sheet.rs:147-185`); the wrapper's `window.open_sheet` closure currently supplies only title and body (`:240-251`). The M480 story records the native close button as unnamed (`docs/tasks/SHEET_PLAYGROUND_STORY.md:99`).

## Proposed change

`patches/proposed/gpui-component-sheet-close-accessibility-v1.patch` adds an optional `Sheet::close_accessibility_label(impl Into<SharedString>)` builder and forwards its value to the internal close Button's existing `.accessibility_label(...)`. The default remains `None`, preserving existing call-site behavior. The patch adds no dependency and does not alter the pinned package version.

This patch adds the upstream hook; by itself it does not fix the M480 observation. A later, separately reviewed RustUI wrapper change must expose a distinct close-name API, resolve its localization key, reject a blank resolved label, and call the new builder inside `window.open_sheet`. The dialog/content name and trigger name must remain separate from the close-button name.

## Source provenance and candidate evidence

- Source repository/revision: Longbridge `gpui-kit`, `67265a36e260ffe03c5aaebdcd8d654dbf892eab`, package path `crates/component`.
- Package: `gpui-component 0.7.3`; registry checksum `465a363235cd4062ef5f0953fc096fc764ed6f53dfbd9f409f669f93ea3b1ff9`. The checksum is recorded by the package `.cargo-checksum.json` and the baseline `HEAD:Cargo.lock`.
- Source preimage: `vendor/gpui-component-0.7.3/src/sheet.rs`, SHA-256 `0bfd572bda2ba54f9e862401705829073de2d33eb94f03d615452d428fc5d5af`; it matches the package checksum manifest entry for `src/sheet.rs`.
- Proposed postimage SHA-256: `bf54674f0df559561004bf7676efb70fe1ce8f0109bf183e109b7dbfe9526f8e`.
- Patch SHA-256: `cc326693b24507f20b4beeb2f4649404167d5dabdade4bbb07d1ebd5b1500104`.
- Isolated candidate: `target/headstart/candidates/sheet-close-accessibility-20261021-r1/`, containing pristine source, proposed result, isolated application copy, and `provenance.json`.
- Ran `git apply --check` against the clean pinned vendor preimage and the isolated copy, then applied the proposal only to the isolated copy. Its resulting source hash matched the recorded postimage; the vendor source remained at its preimage hash. No build, test, or GUI check was run.

## Compatibility and review contract

- Add only the optional field, public builder, and forwarding call in `gpui-component/src/sheet.rs`; use the existing Button accessible-name API.
- `None` must preserve current behavior; a supplied label must appear as the internal close Button's accessible name without changing its icon, role, click action, focus behavior, or close routing.
- Preserve existing `Sheet::new` call sites. Keep the package revision and all other vendored files unchanged.
- RustUI integration must pass a nonblank localized label. Do not reuse the dialog/content label or trigger label as a substitute.
- Before promotion, validate exact-source apply/reverse/check behavior and checksums, compile existing call sites with the repository's approved Cargo launcher, add and run a focused test that inspects the rendered Sheet close Button's accessible name and unset path, then repeat the native macOS accessibility-tree/VoiceOver probe. Check English, French, and Arabic resolved names through the RustUI wrapper; report spoken announcement separately from tree evidence.

## Fallback and removal

If the patch no longer applies to the approved source preimage, do not adjust its context to fit a different source silently; regenerate provenance and review against that exact pinned revision. If the proposal is declined or upstream provides no supported hook, leave the M480 close-button gap open and report it as such. Do not claim the dialog name or trigger name fixes the close control.

Remove this local patch when the selected GPUI Components source provides an equivalent close-button naming hook or a meaningful default accessible name and the RustUI wrapper uses that upstream behavior. Any source revision change or patch application remains subject to explicit approval.
