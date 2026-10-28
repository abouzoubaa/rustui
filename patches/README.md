# Approved upstream patches

## GPUI Components 0.7.3 Select RTL alignment

`approved/gpui-component-select-rtl-alignment-v1.patch` layers on the
committed-title patch. It allows RustUI Picker and ComboBox to align their
popups to the trigger's logical start while keeping LTR as the default.
Run `python3 scripts/apply_gpui_select_rtl_patch.py --verify` to verify
the composed image; `--apply` and `--reverse` replay the layer.

## GPUI Components 0.7.3 DatePicker RTL alignment

`approved/gpui-component-date-picker-rtl-alignment-v1.patch` layers on the
DatePicker localization patch. It adds an opt-in right-to-left alignment for
the calendar while retaining the existing left-to-right default and shared
dropdown behavior. RustUI supplies the active layout direction. Verify the
composed image with `python3 scripts/apply_gpui_date_picker_rtl_patch.py
--verify`; `--apply` and `--reverse` replay the layer against exact hashes.

## block 0.1.6 stack symbol

`approved/block-0.1.6-stack-symbol-v1.patch` replaces the uninhabited external
static declaration with a byte declaration and takes its address as an opaque
pointer. The dependency remains at version 0.1.6; workspace and external
consumer select the local package. Upstream Cargo metadata and checksum files
remain unchanged. Use `python3 scripts/apply_block_stack_symbol_patch.py
--verify` to verify the selected postimage, or `--apply` and `--reverse` for
hash-checked replay. `scripts/verify_block_stack_symbol_candidate.py` also
exercises the patch in a temporary copy with a macOS C/Rust block FFI probe.
The separate missing upstream notice remains a release gap.

## GPUI Components 0.7.3 select committed title

`approved/gpui-component-select-committed-title-v1.patch` (SHA-256
`422921c1eccc8940abb941af6e890b1c7a759996bbaae57d9ff4819c47126631`)
renders the committed selection snapshot in a Select trigger instead of the
search result cursor. Filtering can move that cursor without confirming a new
value. Apply or verify with `scripts/apply_gpui_component_select_title_patch.py`
using `--apply` or `--check`; `--reverse` restores the exact preimage. The
pinned package version and upstream checksum file remain unchanged.

## GPUI Components 0.7.3 Web CodeEditor syntax

- `approved/gpui-component-web-syntax-v1.patch` (SHA-256 `efdd5e4121c9f2d13b03de76d79e8840734e057b3c3f210750b237b653e4150a`) enables the existing Rust and JSON grammar dependencies on Web and selects Tree-sitter 0.27.1. Its crates.io archive SHA-256 is `a5129de2ccc97808c4583d48f1fb44576d015cf241e432800572bda11ef30c3b`; `web-time` 1.1.0 was already locked.
- Version 3 supersedes the retained versions 1 and 2 runtime patches. `approved/gpui-component-web-syntax-runtime-v3.patch` (SHA-256 `c93c5a3c0c26cc755cdd225a50207c9de6bfee6a712b7fb5dbb79bdee547f24b`) adapts query access, uses a Web-compatible clock for highlighting and syntax-aware editing, uses a reference-counted thread-local browser registry without leaking it, and prepares Web diff presentation on the browser executor. Large-file responsiveness and threaded Web behavior require separate validation.
- Apply, reverse, or verify with `scripts/apply_gpui_component_web_syntax_patch.py` and `scripts/apply_gpui_component_web_syntax_runtime_patch.py` using `--apply`, `--reverse`, or `--check`; apply the manifest first and reverse the runtime first. `scripts/tests/test_gpui_web_syntax_patch.py` checks exact replay and tamper rejection.

## Cosmic Text 0.19.0 initial bidirectional span

- Source: crates.io `cosmic-text 0.19.0`, original package checksum
  `be17b688510d934ce13f48a2beba700e11583e281e0fda99c22bb256a14eda73`.
  Cargo selects the same version from the local vendor path; upstream checksum
  metadata remains unchanged.
- Patch: `approved/cosmic-text-initial-bidi-span-v1.patch` (SHA-256
  `25e91c7a89f5963c4e93b8203bdf4a20025248e6d40f2858f9695599eb7e07ce`).
  `src/shape.rs` changes from SHA-256
  `d94983d9b811c8f7f8b3d3dd8e2dc118d260c28eca829aba9af036ee68582837`
  to `6d183b0243467f7033d32ae61d7bba44c34da9fa4b3cecf514962e5207dc681b`.
- The no-wrap layout must distinguish an absent starting position from an
  explicit zero position. Otherwise an initial span opposite the paragraph
  direction loses its glyphs and width. The change preserves explicit-start
  handling and leaves the upstream version pinned.
- Apply, reverse, and verify with
  `python3 scripts/apply_cosmic_bidi_initial_span_patch.py --apply`, `--reverse`,
  and `--check`. Patch replay/tamper and the bundled-font Rust regression pass;
  the in-app Web Playground displayed the previously missing Arabic date digits.
  Broader mixed-direction, native, and browser acceptance remains open. Remove
  this local patch when an approved upstream revision includes an equivalent fix.

The original DatePicker accessibility patch below is retained for history; its
active successor is `approved/gpui-component-date-picker-localization-v2.patch`,
verified by `scripts/apply_gpui_component_date_picker_patch.py`.

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

### GPUI bidirectional single-line pointer hits, v1

- Approved exact-image patch: `approved/gpui-bidi-input-hit-v1.patch`; metadata and
  pristine package checksums are in `approved/gpui-bidi-input-hit-v1.json`.
  It composes after the approved bidirectional wrapped-row patch without changing
  the pinned GPUI or GPUI Components versions.
- Cosmic provides logical byte ranges, visual positions, and direction for each
  glyph cluster. The patched GPUI line layout uses those clusters to place
  single-line carets and resolve pointer hits inside mixed Arabic/Latin text.
  Ordinary left-to-right lines retain their existing path. RustUI continues to
  use GPUI Components' public input implementation.
- Apply or verify with `python3 scripts/apply_gpui_bidi_input_hit_patch.py
  --apply` or `--verify`; `--reverse` restores the exact preimages. The helper
  rejects mixed or unknown images and preserves upstream checksum files.
- A focused RustUI integration regression passed 1/1. Mac and Web release
  drivers passed; current Chromium resolved intermediate mixed-script pointer
  carets and partial drag selection in the public RustUI input story. The
  pinned GPUI library test target stops before test execution on missing
  upstream font fixtures and `util_macros`. Wider input acceptance remains.

### GPUI bidirectional wrapped rows, v1

- Approved patch: `approved/gpui-bidi-wrapped-rows-v1.patch` (SHA-256
  `5a593507ff8e5db732ca012cf6c4832d69a98054b97df8cc1698548ab488468d`).
  The proposal remains at `proposed/gpui-bidi-wrapped-rows-v1.patch`.
- Sources: pinned `gpui-pre`, `gpui-pre-web`, and `gpui-pre-wgpu` 0.3.9 from
  Zed snapshot `93d745668805029476e6d7fbb19652116fed51de`. The five exact
  source images and unchanged package checksum metadata are recorded in
  `approved/gpui-bidi-wrapped-rows-v1.json`. Cargo selects the local WGPU
  package without changing the package version or upstream revision.
- Cosmic shapes bidirectional rows at the final width; GPUI uses the same row
  geometry for painting and byte-position mapping. Ordinary left-to-right
  text, max-lines callers, and native CoreText retain their prior layout path.
- Apply, verify, or reverse with
  `python3 scripts/apply_gpui_bidi_wrapped_rows_patch.py --apply`, `--verify`,
  or `--reverse`. The helper rejects unknown source or checksum images and
  checks the exact staged patch before replacing files.
- Focused Cosmic width checks passed 2/2 in a fresh target; an isolated GPUI
  caret/hit geometry fixture passed 1/1. Native and Web Playground checks and
  a source-matched optimized candidate Web build passed. Chromium painted the
  complete Arabic Button note across two rows and Increment advanced 0 to 1.
  The pinned GPUI library-test target remains blocked by its existing missing
  test-support fonts and `util_macros`; broader browser input, glyph, native
  visual, and performance acceptance are separate release gates.

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
