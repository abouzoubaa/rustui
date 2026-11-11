# Native macOS popups assessment (N11)

## NP-0 refresh — GPUI Kit 0.8.1 / gpui-pre 0.3.10

Rechecked on `origin/main` commit `deb8c095a421ce34182ab1780f46f9d0f591b192` (2026-11-11). The selected macOS backend still rejects anchored popups: `vendor/gpui-pre-macos-0.3.10/src/platform.rs:751` checks `WindowKind::AnchoredPopup(_)`, and line 752 returns `PopupNotSupportedError`. `src/window.rs:998` still groups `AnchoredPopup` with the unsupported `PopUp` panel branch. GPUI Kit 0.8.1 continues to provide the existing in-window popover/menu behavior; it does not provide the native AppKit implementation.

NP-0 rebased v1–v4 plus the v5 coordinate correction into `patches/proposed/gpui-macos-anchored-popup-kit08-v1.patch`, with pre/post hashes for the vendor source and owned `crates/rustui-gpui-macos` mirror, plus the reversible apply helper. `--check` passed against temporary copies; current vendor and owned source remain unmodified. The four approved v1–v4 artifacts and the now-incorporated v5 proposal are in `patches/retired/`, each with the requested supersession reason. The new patch remains proposed and is not active.

The prior N11 observations below are historical where they name gpui-pre 0.3.9 or the old v4-only helper. This refresh replaces the former N11a–N11j ticket list with NP-1–NP-10.

## Decision and scope

**FINISH, through separate tickets of at most two hours each.** RustUI has a low-level API and NP-0 prepared a replayable candidate patch, but native popups are **not production-ready or active in the selected source tree**. Placement tracking, dismissal, focus behavior and owner native acceptance remain. The older N11 assessment below is retained as historical context; the NP-0 refresh and NP-1–NP-10 plan above are authoritative for current source versions.

Assessment refresh date: 2026-11-11. Production-source baseline: `deb8c095a421ce34182ab1780f46f9d0f591b192` (`origin/main`). Native AppKit GUI behavior is **NOT VERIFIED**; no GUI acceptance was attempted.

## 1a. Approved assets versus selected runtime (historical gpui-pre 0.3.9 baseline)

Both current macOS platform sources still reject native popups:

```console
$ rg -n 'if let WindowKind::AnchoredPopup|return Err\(PopupNotSupportedError' vendor/gpui-pre-macos-0.3.9/src/platform.rs crates/rustui-gpui-macos/src/platform.rs
vendor/gpui-pre-macos-0.3.9/src/platform.rs:775:        if let WindowKind::AnchoredPopup(_) = options.kind {
vendor/gpui-pre-macos-0.3.9/src/platform.rs:776:            return Err(PopupNotSupportedError.into());
crates/rustui-gpui-macos/src/platform.rs:775:        if let WindowKind::AnchoredPopup(_) = options.kind {
crates/rustui-gpui-macos/src/platform.rs:776:            return Err(PopupNotSupportedError.into());
```

The approved v1–v4 files are historical versions, not four applied layers. `scripts/apply_gpui_macos_anchored_popup_patch.py` selects **v4 only**, pins its patch hash, and contains pre/post SHA-256 maps for just two **vendor** files (`platform.rs`, `window.rs`). It has `--check` and `--reverse`; no separate popup JSON manifest or owned-mirror entries accompany these versions. v1–v3 are not selected by that helper. v5 remains proposed and is not selected either.

History explains the retained versions. `git log --format='%h %s' -- scripts/apply_gpui_macos_anchored_popup_patch.py patches/approved/gpui-macos-anchored-popup-v*.patch` identifies the initial runtime integration, screen-conversion revision, then bounded child shutdown:

```text
586e3bc2 fix: bound macOS popup child shutdown
acfc7e97 fix(macos): correct anchored popup screen conversion
344baa28 feat(runtime): add native anchored popup windows
```

The original helper selected v1; the current helper selects v4. The checked-in backend remains pristine by the helper's temporary-apply/restore design, rather than treating the approved directory as an automatically applied stack.

| Entry point inspected | Actual wiring |
|---|---|
| `scripts/apply_gpui_macos_anchored_popup_patch.py` | Applies v4 to the vendor backend; `--check` replays in a temporary directory without changing root sources; reverse restores pinned HEAD preimages. |
| `scripts/cargo-headstart` | On Darwin, temporarily applies that helper and restores sources on exit if this invocation applied it. It is optional. |
| `scripts/verify-release.sh macos` | Explicit helper calls at lines 134, 140 and 147: reverse-check, EXIT restoration, apply. This is transient vendor-only application after exact-source checks. |
| `scripts/verify-release.sh source` | No direct popup apply/verify-list entry. Its Python discovery includes the seven popup script tests. |
| `scripts/check.sh` | Direct stable Cargo commands; no popup helper invocation. |

Reproducible wiring inspection: `rg -n 'anchored_popup|popup_patch' scripts/apply_gpui_macos_anchored_popup_patch.py scripts/cargo-headstart scripts/verify-release.sh scripts/check.sh`; read the surrounding `source)` and `macos)` cases, rather than treating a script-name match as proof of runtime application.

```console
$ python3 scripts/apply_gpui_macos_anchored_popup_patch.py --check
verified
exit=0
$ rustup run 1.97.1 cargo metadata --format-version 1 --locked --offline > /tmp/n11-cargo-metadata.json
exit=0
```

Extracting the selected packages and the macOS dependency from that metadata gives:

```text
gpui-pre-platform 0.3.9 /Users/Abium/Developer/rustui/crates/rustui-gpui-platform/Cargo.toml
 dependency: rustui-gpui-macos path=/Users/Abium/Developer/rustui/crates/rustui-gpui-macos target=cfg(target_os = "macos")
rustui-gpui-macos 0.3.9 /Users/Abium/Developer/rustui/crates/rustui-gpui-macos/Cargo.toml
```

The owning manifest is `crates/rustui-gpui-platform/Cargo.toml`, under the macOS-target dependency `gpui_macos` (`package = "rustui-gpui-macos"`, `path = "../rustui-gpui-macos"`). Thus the helper can successfully replay v4 without enabling popups in the implementation Cargo actually compiles. Both checked-in copies are presently exact preimages. The saved WIP vendor copies are v4 postimages (next section). Existing built application binaries and other developers' checkouts were not inspected.

## 1b. Saved branch versus v4 and v5

```console
$ git rev-parse origin/wip/native-macos-popups
1ba582ee1d7f0a19728a086126055e30c624f460
$ git diff --name-status origin/main...origin/wip/native-macos-popups -- vendor/
M	vendor/gpui-pre-macos-0.3.9/src/platform.rs
M	vendor/gpui-pre-macos-0.3.9/src/window.rs
```

`git show origin/wip/native-macos-popups:<path>` was copied to temporary files and hashed with `shasum -a 256`. The resulting hashes match the helper's v4 postimage constants **exactly**:

| File | Current vendor and owned mirror SHA-256 (preimage) | WIP SHA-256 = v4 postimage |
|---|---|---|
| `src/platform.rs` | `1bf0adb67969e7389063de4d54efa70e9bb0a7ac7331a74ba74245d3bb87d801` | `6f7b88ef87d757c022a505f5b33104e727bd4e8e4ebdc6b7f3c9a5e0823c8aa6` |
| `src/window.rs` | `12d18266972087f25e876a97751af1903401ad2f9b8d9c84eab7fad9495fbd00` | `ab826e99056215713a220895f9917e5b2e39af4b4d49ccdd0fd341bb7c60daaa` |

**Classification: identical to v4 for both changed backend vendor files; different from v4 plus v5.** Main has unrelated vendor history after the old branch fork; the relevant comparison is the saved popup payload, not replacing today's entire vendor tree with the old branch.

`patches/proposed/gpui-macos-anchored-popup-v5-coordinate-fix.patch` is an incremental patch **on v4**, not a standalone backend replacement. Reading its entire diff shows three insertion-only hunks:

```diff
@@ -960,0 +961,12 @@
@@ -978,0 +991,3 @@
@@ -1165,0 +1181,18 @@
```

They add 33 lines: `popup_anchor_rect_in_view`, its call before view-to-window conversion, and `anchor_rect_conversion_respects_parent_bounds_and_flip_state`. The helper is absent from the WIP window source. The test covers bounds `(10,20,300,200)` and anchor `(5,30,10,8)`, expecting flipped origin `(15,50)` and unflipped origin `(15,182)`. An insertion-only replay in memory on the exact WIP/v4 bytes produces window SHA-256 `33a9b7bc0467c07cc9eda1b9c231e32bf0aca144afccd5b2cb1dc8fc867491ac`; no repository source was changed, and this is **not a compile result**.

The old v5 row in `docs/PATCH_TRIAGE.md` reports an exit-101 item-level `let` error and calls for a rebase. That historical compiler failure was **NOT REPRODUCED** here. Applying an incremental, zero-context v5 directly to pristine main is not valid proof against the v4-plus-v5 stack. A new ticket must pin the intended v4 preimage, regenerate contextual hunks, prove replay/reversal on vendor and owned copies, and compile/test the selected native backend before promotion.

## 1c. Existing RustUI surface and tests

| Source | Existing surface / coverage |
|---|---|
| `crates/rustui/src/runtime/window.rs:23` | Public `AnchoredPopupOptions { anchor_rect, size }`; finite geometry validation. |
| `crates/rustui/src/runtime/window.rs:32` | Public `AnchoredPopupHandle` alias of `AnyWindowHandle`. |
| `crates/rustui/src/runtime/window.rs:42` | Public `open_anchored_popup(root, environment, options, parent, cx) -> Result<AnchoredPopupHandle, String>`, exported by `runtime/mod.rs`. Builds a separate ComponentHost/Components Root using GPUI window ownership. |
| Same function | BottomLeft anchor, BottomRight gravity, flip/slide/resize flags, zero offset, **always `grab: true`**. Accepts an explicit environment. No automatic in-window fallback or typed unsupported-error policy. |
| `apps/rustui-playground/src/stories/popover_gallery.rs` | “Open native popup” / close probe, retained handle and mount/release counters. Uses click position as a 1×1 anchor and a 280×180 popup. Errors go to stderr. This is a probe, not production integration or native acceptance. |
| `crates/rustui-gpui/src/platform/test/platform.rs:557` | Test platform opens generic TestWindow instances; it does not exercise AppKit panel placement, key-window behavior or physical screens. |

Commands: `rg -n 'AnchoredPopupHandle|AnchoredPopupOptions|open_anchored_popup' crates/rustui/src apps/rustui-playground/src` and `rg -n 'anchored_popup|popup_parent' crates/rustui/src/runtime/window.rs`. No regular RustUI component currently consumes this runtime helper; the Playground is its caller. Existing in-window Popover remains separate.

Existing tests were run, without adding or changing tests:

```console
$ rustup run 1.97.1 cargo test -p rustui --lib runtime::window::tests:: --locked --offline
running 6 tests
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 1422 filtered out
exit=0
$ python3 -m unittest scripts.tests.test_gpui_macos_anchored_popup_patch -v
Ran 7 tests
OK
exit=0
```

The six GPUI tests are `opens_a_root_window_in_the_test_platform`, `closing_root_window_unmounts_the_owner`, `closing_anchored_popup_unmounts_its_root`, `popup_parent_can_open_another_popup_after_explicit_close`, `root_tab_focuses_and_activates_button`, and `initial_traversal_is_scoped_to_its_root_window` (all under `runtime::window::tests`). Only the two explicitly named popup cases test popup lifecycle; none validates AppKit behavior.

The seven Python tests cover approved-patch presence, check-mode immutability, exact/idempotent replay, exact reverse, unknown-source rejection, and patch-text assertions for parent teardown and anchor-view native-window conversion. The latter assertions do not execute native teardown or coordinate conversion. Four placement unit tests are inside v4's macOS patch, and the fifth flip/bounds test is in v5; these tests are absent from the current unpatched owned backend and were **not run**.

The probe uses a click callback, whereas `crates/rustui-gpui/src/platform/popup.rs` documents that a grabbing popup should be opened while the triggering mouse-down/input is active. That contract also leaves same-application outside clicks to the caller. These details need an explicit adapter/dismissal design, not just enabling panel creation.

Existing in-window menu behavior should be reused and preserved: `crates/rustui-gpui-component/src/menu/popup_menu.rs` binds Escape to Cancel, restores previous focus in `dismiss`, and renders nested menus with `deferred(anchored())` snapped to the current window. This does not establish cross-window native focus behavior or replace the missing native integration.

## 1d. Remaining production work

The v4 payload already creates NSPanel-backed child windows, computes an initial screen frame with flip/slide/resize adjustments, dismisses a grabbing leaf panel on key resignation, and closes child windows during parent teardown. These are useful starting points, not sufficient acceptance proof.

| Area | Present payload / precise gap | Estimate and proof boundary |
|---|---|---|
| Source integration | Vendor-only transient apply misses owned Cargo source; no paired manifest. | 2h: paired replayable patch plumbing, selected-source proof. |
| View/AppKit coordinates | v4 forwards logical top-left anchor coordinates to an NSRect without the v5 view-bounds/flip conversion. | 2h: rebase v5, contextual replay, bounds/flip/scale tests. |
| Displays and edge constraints | Anchor-center display selection, visibleFrame, flip/slide/resize exist. Negative origins, stacked displays, oversized popups and display fallback need coverage. | 2h: placement matrix; actual screens need GUI acceptance. |
| Live placement | Frame computed at creation; no stored live trigger geometry/update path. Child-window movement alone does not prove tracking after trigger layout/scroll/resize or scale changes. | 2h: define and test re-anchoring lifetime using GPUI scheduling. |
| Outside click | Resign-key closes only grabbing panels with no child windows. No new explicit same-app outside-click/nested-stack controller or monitor cleanup. | 2h: deterministic dismissal controller tests plus native click-through/activation checks. |
| Escape and focus return | No new popup Escape policy or remembered owner-element focus restoration. Generic AppKit key changes do not prove correct owner focus. | 2h: topmost-close policy, weak focus-owner recovery; keyboard dispatch tests and native focus acceptance. |
| Parent/owner teardown | Native close enumerates and closes child windows. Python checks only source text; generic test windows cover explicit close/reopen, not NSWindow callbacks. | 2h: nested close, owner removal, stale handles, weak release and unrelated-child regression coverage. |
| RustUI policy / fallback | Low-level opt-in API exists, but always grabs and stringifies errors; no integrated native-preferred/fallback policy. | 2h: preserve current API; design additive policy only with owner approval. Default in-window; fallback for unsupported native capability, surface real errors. |
| First component adapter | No production component wiring. Preserve trigger bounds, environment, bindings, owner lifetime and mouse-down contract. | 2h: one opt-in Popover adapter; other families get separate tickets. |
| Native acceptance | No actual AppKit placement/dismissal/focus evidence in these checks. | 2h owner session: multi-display/scales, activation, Escape, owner close, focus/accessibility. Inconclusive result is reported once. |

Coordinate reasoning follows the inspected source and Apple's contracts: [NSView.isFlipped](https://developer.apple.com/documentation/appkit/nsview/isflipped), [rect conversion](https://developer.apple.com/documentation/appkit/nsview/convert%28_%3Ato%3A%29-3cqqt), and [NSScreen](https://developer.apple.com/documentation/appkit/nsscreen). Distinguish parent-window logical coordinates, view bounds, window coordinates and screen coordinates; do not assume a universal flipped origin or scale factor.

| Can run deterministically in GPUI test windows / fixtures | Requires a real macOS GUI session |
|---|---|
| API geometry/error/fallback policy, explicit open/close/reopen, owner unmount, weak release, key-action controller behavior, environment/binding retention. | AppKit key/resign event order, same-app and other-app outside clicks, native child-window teardown, actual focus return and accessibility window transitions. |
| Pure placement matrix and replay/hash checks; selected-backend compilation separately. | Actual monitor arrangements, visible-frame constraints, Retina/mixed scales, Spaces and activation. |

A TestWindow passing lifecycle tests cannot certify NSPanel integration. Production readiness requires both deterministic coverage and owner native acceptance. No new public API signature is approved by this assessment.

## 1e. Benefit and candidate adopters

| RustUI family | Prospective benefit / constraint |
|---|---|
| Menu/context menu | Menus can extend outside a short or narrow parent window; requires keyboard selection, nested dismissal and focus restoration. |
| Popover | Inspector/help/detail panels avoid parent-window clipping while retaining trigger anchoring and owner lifetime. First adapter candidate. |
| Picker / ComboBox | Long option lists can use available screen space; selection/input focus must survive separate-window ownership. |
| DatePicker / ColorPicker | Calendar/color panels can extend beyond the parent; adopt only after the common popup policy works. |
| Tooltip | Hints can avoid clipping near window edges, but need a passive/non-grabbing mode; the present RustUI helper always grabs. |

These are prospective gains. Normal RustUI controls currently keep their in-window implementations; this assessment enables none of them.

## NP-1–NP-10 plan — each ticket capped at two hours

| Ticket | Rechecked status on 0.3.10 / 0.8.1 | Work and proof | Estimate |
|---|---|---|---|
| NP-1 | **DONE** — approved Kit 0.8.1 backend patch is active and mirrors match. | The RustUI public API test proves open/close/reopen on GPUI's test platform; full and source checks pass. Native AppKit acceptance remains for NP-10. | ≤2h |
| NP-2 | **DONE** — the rebased patch covers view-bounds offsets and flipped/unflipped conversion. | Added fractional logical-point geometry assertions and verified the scale-independent points → view → window → screen path. Native multi-display placement remains an owner native check in NP-3/NP-10. | ≤2h |
| NP-3 | **DONE** — deterministic screen selection and visible-frame placement now cover negative/stacked origins, main-screen fallback and oversized resize constraints. | Rust geometry tests pass for the pure screen-selection/frame helpers; actual multi-display placement remains an owner native check. | ≤2h |
| NP-4 | **DONE** — the opt-in `AnchoredPopupHandle` follows measured trigger bounds and recomputes placement in the open window; parent bounds/display observers reuse the latest anchor rectangle. | `Window::set_anchored_popup_anchor_rect` reaches AppKit `setFrame:display:` through the approved reversible popup patch. Prepaint reads the trigger's own bounds, updates placement on movement, and closes it when fully outside the active clip or visible window. Test-platform coverage proves measurement, movement, resize, scroll-out, and detached-trigger behavior. Owner native check: drag-resize the parent and scroll a list with the popup open. | ≤2h |
| NP-5 | **IN PROGRESS** — nested grabbed popups dismiss leaf-first, and removing a trigger while it remains within visible bounds closes its popup. | Focused macOS test `nested_grabbing_popups_dismiss_leaf_before_parent` and RustUI GPUI test-window test `removing_a_visible_anchor_closes_its_popup` pass. Cross-window click-through still needs the owner native check. | ≤2h |
| NP-6 | **STILL NEEDED** — no native Escape policy or verified focus return to the owner element. | Close the topmost popup, preserve text-editing semantics, restore weak owner focus; key-dispatch tests plus owner native focus check. | ≤2h |
| NP-7 | **CHANGED** — bounded native child-window close logic exists in the candidate, not in active source. | Prove nested popup teardown, owner release, stale handles and unrelated child-window safety; owner native check for AppKit close callbacks. | ≤2h |
| NP-8 | **STILL NEEDED** — RustUI exposes an explicit low-level popup API; no native-preferred policy with in-window fallback is integrated. | Keep the API opt-in, classify unsupported/native errors, and preserve the existing GPUI Kit in-window fallback. The default in-window behavior is already covered by Kit 0.8.1. | ≤2h |
| NP-9 | **STILL NEEDED** — no component uses the native API. | Add the first opt-in Popover adapter with real trigger bounds, environment/state ownership, reopen and in-window fallback tests. Menus and pickers remain separate follow-ups. | ≤2h |
| NP-10 | **STILL NEEDED — owner native check.** | Run the final AppKit matrix for placement, activation, click dismissal, Escape, focus, teardown and accessibility; no automation substitutes for the owner session. | ≤2h |

Unit and GPUI test-window coverage can prove geometry, error/fallback policy, explicit open/close, owner unmount, weak release, key-action handling, environment retention and patch replay. Real screen layouts, outside-click across windows, AppKit activation/focus ordering and native teardown remain **owner native check** items. Keep all existing in-window popover/menu/picker tests unchanged and passing. Native popups remain opt-in.

Every GPUI vendor or owned-source change must come from a patch and its apply script; preserve the mirror hashes and reverse exactly. NP-1 promoted and activated the consolidated macOS patch; subsequent popup changes remain patch-only.

## Housekeeping and validation

The sole remaining local stash was not reachable from existing archive tags. It is now permanently tagged and pushed, without dropping it:

```console
$ git stash list --format='%gd %H %gs'
stash@{0} 62cffca786ddce5258a7862d2ce6e4d275326621 On g10-small-gaps: G10 preserved part 4 for owner-requested rebase
$ git ls-remote --tags origin refs/tags/archive/stash-20261109-1
62cffca786ddce5258a7862d2ce6e4d275326621	refs/tags/archive/stash-20261109-1
```

Each candidate remote tip was checked with `git merge-base --is-ancestor <tip> origin/main` (exit 0) and matched against `git ls-remote --heads origin` before deletion. Deleted merged branches: `g10-small-gaps`, `g11-aliases`, `p0-patch-triage`, `p1-promote-two`, `p2-rebase-promote`, `p2b-slider`, `r15-disabled-controls`, `r15b-dropdown-recovery`, `r20-async-tasks`, `r26-theme-reload-tests`, `roadmap-stages`, `rule-patch-only`, `s11-stash-review`, `t9r-tools-review`. `main` and every `wip/*` were excluded. The separate AGENTS-only rule PR [#53](https://github.com/abouzoubaa/rustui/pull/53) merged as `ae83d288d55cd7e6f36b716505b7a52aacdd717e`. Its two-parent merge and rule commit were verified on fetched origin/main before deleting remote `rule-delete-merged-branches`. The assessment branch follows the same verified-merge-before-deletion rule.

Both the housekeeping branch and the assessment branch passed the same required checks in this session. Assessment commands and results:

```console
$ scripts/check.sh > /tmp/n11-assessment-check.log 2>&1; echo "exit=$?"
exit=0
$ ./scripts/verify-release.sh source > /tmp/n11-assessment-source.log 2>&1; echo "exit=$?"
exit=0
```

`check.sh`: **1,814 passed, 0 failed, 4 existing ignored** across the workspace library suites. Source release gate: **647 Python script tests passed**, plus its other checks. Existing focused tests above also passed. No new tests were added for this read-only assessment, and no product capability/status change is claimed.
