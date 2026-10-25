# Proposed upstream accessibility patches

These patches are review artifacts only. They are not applied to `vendor/`, do not change dependency revisions, and do not change RustUI's roadmap.

## Pinned sources

- `gpui-pre 0.3.9`: approved Longbridge packaging of official Zed revision `93d745668805029476e6d7fbb19652116fed51de`.
- `gpui-base 0.7.3` and `gpui-component 0.7.3`: Longbridge `gpui-kit` revision `67265a36e260ffe03c5aaebdcd8d654dbf892eab`.
- Both `gpui-base` and `gpui-component` are Apache-2.0. Preserve their upstream `LICENSE-APACHE` files when these changes are proposed upstream.
- Registry package checksums from `Cargo.lock`: base `2e9955b6f4b048fd50ca15fc8e225d80bb00ba9ebc8fecdddde5ad26dbef9270`; component `465a363235cd4062ef5f0953fc096fc764ed6f53dfbd9f409f669f93ea3b1ff9`.

## Patches

### Focus trap accessibility forwarding

`gpui-base-focus-trap-accessibility-v1.patch` proposes forwarding role, node-property, and synthetic-child accessibility hooks through GPUI Components' generic `FocusTrapContainer`. Its source trace and validation gates are recorded in `docs/tasks/FOCUS_TRAP_ACCESSIBILITY_PROPOSAL.md`. The patch SHA-256 is `cf5def8ebfb6284f2e63938dda4bd358b344327e3a6951b55eebc0ceffc9040c`. Zero-fuzz application, isolated checks, and bounded native Dialog exposure checks passed; see the proposal for remaining validation gates. The candidate is not applied to `vendor/`.

### DatePicker accessible name

`gpui-component-date-picker-accessibility.patch` adds an optional `.accessibility_label(...)` builder to the styled component and forwards it to the existing base ComboBox via `aria_label`. This avoids treating the visible placeholder/value as a name. Existing callers remain source-compatible. A RustUI integration should pass a stable label; if the proposal is not accepted, keep the RustUI accessibility capability unavailable rather than presenting a placeholder as the control's name.

Source file SHA256 before patch: `vendor/gpui-component-0.7.3/src/time/date_picker.rs` — `e7c8c87c76b253703a955c578f28278212d54545d9c9a62a76892a3140095248`.

### Slider SetValue

`gpui-base-slider-set-value.patch` registers AccessKit `SetValue`, `Increment`, and `Decrement` only while enabled. It accepts only finite numeric `SetValue` inputs, clamps to slider bounds, applies the same step rounding as pointer updates, emits `SliderEvent::Change`, and updates the end thumb while preserving/clamping the range start. For a range, the single slider node advertises an effective minimum of `max(range.start, slider.min)` so its SetValue bounds match the value it controls; for a single value it advertises the configured minimum. This node controls only the end thumb, so the range start remains inaccessible through this existing one-node API. All three accessibility actions use one state helper for step mapping, thumb updates, notification, and change events. The AccessKit action data uses GPUI's public `gpui::accesskit` re-export; no new direct dependency or GPUI core change is proposed. Missing, wrong-typed, NaN, infinite, and f64 values that overflow f32 are ignored.

Source file SHA256 before patch: `vendor/gpui-base-0.7.3/src/slider.rs` — `b677bee1185b65a2fe4b3ecfcd247395be57d8bf7543c3cc0f9e0eac6ff9e0b6`.

### Tracked dialog opening and close by ID

`gpui-component-tracked-dialog.patch` proposes a separate `TrackedDialogExt`
trait and opaque `DialogId` so an owner can close its own native dialog even
when another dialog is above it. Existing `WindowExt` signatures remain
unchanged. Non-top removal repairs the successor's focus chain; delayed focus
restore uses a stack generation. The exact API, preimage hashes, patch hash,
stack/focus contract, fallback, and unrun validation plan are in
`docs/tasks/O7_DIALOG_KEYED_UPSTREAM_PROPOSAL.md`. This proposal is not applied
or tested, and it does not authorize a RustUI Dialog implementation.

## Required upstream validation

- DatePicker: compile existing call sites unchanged; verify the named node exposes the supplied label with an empty and populated value and remains a ComboBox.
- Slider: verify AccessKit emits `SetValue` for enabled sliders and omits it for disabled sliders; exercise single and range values (including an effective minimum when the range start exceeds the configured minimum), both bounds, start/end ordering, and malformed/missing/non-finite/overflowing `SetValue` data; verify all three actions are ignored when disabled, step rounding matches pointer updates, and bindings receive `SliderEvent::Change`.
- Validate against the exact pinned revisions above. These patch texts have not been applied or compiled in this repository.
- Tracked dialog: verify compile compatibility, three-level stack removal,
  focus/selection restoration, stale and cross-window IDs, native dismissal,
  delayed focus after stack churn, and RustUI owner release using an isolated
  candidate. See the O7 proposal for the full plan; none of these checks ran.


### Disabled Tree accessibility state

`gpui-tree-aria-disabled.patch` now has an approved v1 in `patches/approved/`; see `patches/README.md` and `docs/tasks/TREE_ARIA_DISABLED_PROPOSAL.md` for exact hashes, source scope, and validation limits.

### PopupMenu disabled confirmation

`gpui-component-popup-menu-disabled-confirm.patch` prevents a disabled selected item from running its handler/action or dismissing the menu. The focused source finding, exact package source hash, trade-offs, and validation plan are in `docs/tasks/POPUP_MENU_DISABLED_CONFIRM_PROPOSAL.md`. The patch is review-only and has not been applied or validated.

### Sheet close-button accessible name, v1 proposal

- Patch: `gpui-component-sheet-close-accessibility-v1.patch` (SHA-256 `cc326693b24507f20b4beeb2f4649404167d5dabdade4bbb07d1ebd5b1500104`).
- Source: Longbridge `gpui-kit` revision `67265a36e260ffe03c5aaebdcd8d654dbf892eab`, path `crates/component`, package `gpui-component 0.7.3`; registry package checksum `465a363235cd4062ef5f0953fc096fc764ed6f53dfbd9f409f669f93ea3b1ff9`.
- Source preimage: `vendor/gpui-component-0.7.3/src/sheet.rs`, SHA-256 `0bfd572bda2ba54f9e862401705829073de2d33eb94f03d615452d428fc5d5af`; proposed postimage SHA-256 `bf54674f0df559561004bf7676efb70fe1ce8f0109bf183e109b7dbfe9526f8e`.
- Adds optional `Sheet::close_accessibility_label`; existing callers and behavior remain unchanged when the setter is unused. RustUI still needs a separate wrapper integration that supplies a nonblank localized name before the native close control is named.
- Exact-source and isolated-application evidence, integration contract, validation gates, fallback, and removal conditions: `docs/tasks/SHEET_CLOSE_ACCESSIBILITY_NAME_PROPOSAL.md`. Candidate snapshot: `target/headstart/candidates/sheet-close-accessibility-20261021-r1/`. Review-only; not applied to `vendor/`, compiled, or GUI-validated.

## Sheet close-name validation gate

Before promotion, confirm unchanged Sheet call sites compile, add and run a focused test proving the supplied label reaches the rendered internal close Button (and the unset path remains unchanged), and repeat the native macOS accessibility-tree/VoiceOver probe after RustUI integration. No compile, test, or GUI check has run for this proposal.

### Tree aria-level on the outer TreeItem, v1 proposal

- Patch: `gpui-tree-aria-level-v1.patch` (SHA-256 `4d61d6f249d3102f60725e7c5446c768ab64f0436bc49ba7b309cea20d073e30`).
- Source: Longbridge `gpui-kit` revision `67265a36e260ffe03c5aaebdcd8d654dbf892eab`, package `gpui-base 0.7.3`; registry archive checksum `2e9955b6f4b048fd50ca15fc8e225d80bb00ba9ebc8fecdddde5ad26dbef9270`.
- The package's pristine `src/tree.rs` SHA-256 is `e74e4fcdef6ccec174e3e6ced50f108433e726f3e3d46f3539b1f41d5944e5df`; pristine `.cargo-checksum.json` SHA-256 is `9e96d55b0071b60576bc6ac18c3e9c48e6d0de5ff848821af7623bd151f7c3ed`. The patch preimage is the current aria-disabled-v1 postimage `9ef2efa0a44af810eef3088343b0838179d5e952c6ca8734e5306d5c74736924`; the one-line proposed postimage is `232d865398ba883b9423a5239d4762346b58e20bc7dd932018ab1fc8c79eb2a5`.
- Adds `.aria_level(entry.depth() + 1)` to the existing outer TreeItem row. Depth is zero-based; ARIA tree levels are one-based. The pinned `gpui-pre 0.3.9` already supplies the setter. No RustUI API or dependency revision change is proposed.
- Exact-identity helper: `scripts/apply_gpui_tree_aria_level_patch.py`. Its `--check` verifies the preimage, applies and reverses the patch in temporary copies, and confirms both hashes without changing the checkout. Apply/reverse modes target only the Tree source and require their exact source image.
- Review-only: not applied to `vendor/`. Applying it requires separate user approval. See `docs/tasks/M1222_STAGE8_TREEITEM_ROW_SEMANTICS_BOUNDARY.md` for the focused test command, runtime oracle, and validation status.
- Validation: exact helper round-trip check passed in a temporary copy. No unit test or GUI acceptance was run.
