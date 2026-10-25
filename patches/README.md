# Approved upstream patches

GPUI and GPUI Components remain the pinned Longbridge snapshots of the official
upstream sources. Approved patches are applied to the matching pristine source
through a versioned patch file and an exact-identity helper; do not edit the
vendored file by hand. The original package checksum metadata remains the
pristine upstream provenance record. Cargo selects an applied package through
an explicit local path override while its upstream package version stays fixed.

Each future patch set must record source repository/revision, source checksum,
ordered patch checksums, purpose, dependencies, compatibility constraints, fallback,
removal condition, and actual validation evidence. Prepare candidates in isolation;
fail on incompatible patches and preserve the last accepted source lock.

## Applied patches

### gpui-pre-web keyboard focus and activation, v3

- Approved patch: `approved/gpui-pre-web-keyboard-focus-activation-v3.patch`; proposal retained at `proposed/gpui-pre-web-keyboard-focus-activation-v3.patch`.
- Patch SHA-256: `df97bcbf43490e1e60e582da87b5bd63ebc2eb275e662f5c49d622f77b7de59a`.
- Source: pinned Longbridge `gpui-pre-web 0.3.9`, from approved Zed snapshot revision `93d745668805029476e6d7fbb19652116fed51de`. Original registry checksum: `aedd55b99dc77e28df8202b0ea868aa5c90fda4aefff539ea21087d7c2ff4bfb`; pristine `.cargo-checksum.json` SHA-256: `aad9503f554e462d582461d008ea4bc0dda00021812ccf6b876734b55757579e`.
- Patched source hashes: `src/window.rs` `eda543e585912fcb012e2a3559d8fb2e9ad7570ba58746747af380d044085cb8` → `3779ba8d5647009b80d0ea52dc7823806640741d3a1cfa9a0a7752ec35fe7dd8`; `src/events.rs` `9fdfc3695a93083fe881153dcc66356df00eb09761190664cc093520bc3ae001` → `3dca61104bce15bffdd7d9155af769dc166c68508fb6458b4952261b2c3244cb`.
- The patch makes the Web canvas keyboard-tabbable and forwards keydown/keyup only when the event target is that canvas or the GPUI IME mirror. Text and composition input continue through the IME mirror. Cargo selects the same-version local package with `[patch.crates-io]`; original checksum metadata stays unchanged.
- Apply, reverse, or verify with `python3 scripts/apply_gpui_web_keyboard_focus_activation_patch.py --apply`, `--reverse`, or `--check`.
- Compatibility boundary: only the pinned `gpui-pre-web 0.3.9` snapshot. No dependency version or upstream revision changed. Remove the patch when the corresponding upstream host behavior is available and verified.
- Validation: isolated exact-patch checker and rustfmt passed; locked/offline Cargo metadata resolved the local package; optimized Playground Wasm bundle built; fresh-browser load exposed canvas `tabindex=0`, and Space on a keyboard-focused Increment button advanced the counter from 1 to 2. Text-entry and browser screen-reader acceptance remain open. Evidence: `target/headstart/evidence/m855-stage11-web-space-activation-20261023.json` and `.png`.

### gpui-component DatePicker accessible name, v1

- Patch: `approved/gpui-component-date-picker-accessibility-v1.patch`
- Review proposal retained unchanged at:
  `proposed/gpui-component-date-picker-accessibility.patch`
- Both patch files have SHA-256
  `ce1ae916fb402f940e2d1dbdd6c8f8ef3c60ee83ced8ad3890a5cce8a343239e`.
- Source: Longbridge `gpui-kit` revision
  `67265a36e260ffe03c5aaebdcd8d654dbf892eab`, package
  `gpui-component 0.7.3` (Cargo.lock checksum
  `465a363235cd4062ef5f0953fc096fc764ed6f53dfbd9f409f669f93ea3b1ff9`).
  The patch changes only the public DatePicker builder and forwards its optional
  name to the existing focused `gpui_base::DatePicker` ComboBox root.
- Patched source preimage SHA-256:
  `e7c8c87c76b253703a955c578f28278212d54545d9c9a62a76892a3140095248`.
- Patched source postimage SHA-256:
  `aedb052400ef31b2754e58820b54453d1f80c39c0f2ed4548d167b1b18bc3b79`.
- Apply, reverse, and verify with
  `python3 scripts/apply_gpui_component_date_picker_patch.py --apply`,
  `--reverse`, or `--check`. The helper rejects any other source image and
  retains the pristine `.cargo-checksum.json` unchanged.
- Cargo uses the local `vendor/gpui-component-0.7.3` package with the same
  `0.7.3` version through the workspace `[patch.crates-io]` entry. This does not
  move the pinned GPUI or Components revision. Remove the manifest override and
  reverse the helper to return to the accepted pristine package.
- Validation evidence and unrun native acceptance are recorded in
  `docs/tasks/DATE_PICKER_ACCESSIBILITY.md` and the isolated candidate under
  `target/headstart/candidates/date-picker-accessible-name-20261020-r2/`.

Advanced rendering and native composition require explicit approval. GPUI Fast is
research input, not a dependency or the upstream source of truth.


### gpui-pre / gpui-base Tree aria-disabled, v1

- Patch: `approved/gpui-tree-aria-disabled-v1.patch` (SHA-256
  `5762dfec8eba4b723bc1b851c527c75f18e78d76d050fd8bab4485a63ceb14d1`).
- Sources: pinned `gpui-pre 0.3.9` from Zed revision
  `93d745668805029476e6d7fbb19652116fed51de`, and `gpui-base 0.7.3` from
  Longbridge `gpui-kit` revision `67265a36e260ffe03c5aaebdcd8d654dbf892eab`.
  Their registry checksums remain recorded in their unchanged
  `.cargo-checksum.json` files; source-file pre/postimage hashes are in
  `docs/tasks/TREE_ARIA_DISABLED_PROPOSAL.md`.
- The patch adds `StatefulInteractiveElement::aria_disabled`, maps true with
  AccessKit `set_disabled()` and false with `clear_disabled()`, and forwards
  `TreeItem::is_disabled()` from the pinned Tree renderer. A focused AccessKit
  regression is included.
- Apply, reverse, and verify with
  `python3 scripts/apply_gpui_tree_aria_disabled_patch.py --apply`,
  `--reverse`, or `--check`. The helper stages and verifies all changed files
  before installation. Cargo path overrides select the patched local copies
  while the pinned package versions stay fixed.
- Locked/offline Playground all-targets compilation passed. The standalone
  `gpui-pre` unit-test build was blocked before test execution by missing
  upstream font/test-support prerequisites; native AX, VoiceOver, pointer, and
  keyboard acceptance remain open. See the task record above for exact logs.


### GPUI Tree accessible root, row level, and test accessibility activation — v1

This approved same-version series composes after the existing aria-disabled v1
baseline and changes exactly three pinned sources: gpui-base 0.7.3 Tree,
gpui-component 0.7.3 Tree, and gpui-pre 0.3.9 TestWindow. It retains one existing
Tree root, adds the accessible-name API used by RustUI, records one-based levels
on existing TreeItem rows, and activates accessibility only in TestWindow tests.
No package version, revision, checksum metadata or license changes.

The exact ordered patch/source/provenance hashes are recorded in
`approved/gpui-tree-accessibility-series-v1.json`. Order: test-a11y-activation v1,
aria-level v1 (corrected nodes-map test, patch SHA 7729270cd97bfc74039e05d5142d522b78cc9bd563a825ccaa33caeb2d8f77f7), then component-tree-accessible-name v1.
The identical review artifacts remain archived under `proposed/`.

Use `scripts/apply_gpui_tree_accessibility_series.py --check` for a read-only
PRE round trip; `--apply` installs all three exact postimages; `--verify` proves
current POST, reverses/replays in disposable copies and proves the underlying
aria-disabled baseline; `--reverse` restores the exact three PRE images while
leaving the approved aria-disabled baseline installed. The existing aria-disabled
`--check` now delegates composed Tree verification to the full series. Its
apply/reverse modes reject a composed source and require the series to be
reversed first. Standalone aria-level mutation is retired; its old entry point
supports read-only full-series checks only.

CI and all release modes verify POST and the composed baseline before further
checks; the source fixture suite replays PRE/check/apply/POST/verify/reverse for
all three sources and rejects unknown/partial images and altered patches.
Fixture evidence is not native/browser/AX/VoiceOver acceptance. See
`docs/tasks/TREE_ROOT_AND_LEVEL_PROMOTION_DECISION.md`.
