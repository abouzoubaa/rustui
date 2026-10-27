# Production consolidation — 2026-10-26

Writing session: thread `01a172cd-8a21-73f1-82cd-62d451c71ec9`, claim
`c44f414bfbf4ef4c4f30`, branch `codex/production-consolidation-01a172cd`,
worktree `/Users/Abium/.codex/worktrees/rustui-production-consolidation/rustui`.
The verified primary integration branch is `work/m118-diagnostics-provenance`;
The writing branch is fast-forward integrated through its latest completed commit.
Reverify live HEAD and dirty state before subsequent integration; preserve unrelated work.

## Current release evidence — 2026-10-27

- **Native binary path provenance:** `RUSTUI_REMAP_BUILD_PATHS=1` release rebuild completed on `4058f18d`. The 40 MiB executable contains zero occurrences of this writing worktree and target directory. It still contains nine absolute Cargo registry paths: `gimli 0.34.0`, `addr2line 0.27.1`, `rustc-demangle 0.1.28`, and `hashbrown 0.17.1`. The build log compiled different `gimli 0.32.3` and `addr2line 0.25.1` versions, so these remaining strings are consistent with the prebuilt Headstart toolchain; the new Cargo-home remap cannot erase them. `scripts/verify-release.sh` now checks for the exact checkout path in the native executable after release build. Focused Headstart tests passed 2/2, shell syntax and diff checks passed. This is source/build provenance evidence, not native GUI acceptance or a fully path-free binary.
- **Compiled CI:** [run 40445906850](https://github.com/abouzoubaa/rustui/actions/runs/40445906850), attempt 2, passed `source` and `macos-and-web-builds` on exact `41241f73` (native tests/package and Web optimized bundle). Attempt 1 failed the macro doctest with E0514 because `RUSTUI_HEADSTART_ROOT` was set runner-wide to the Web-only toolchain; attempt 2 used native Headstart by default and `RUSTUI_WASM_HEADSTART_ROOT` for Web. The ephemeral runner exited and removed `.runner` and `.credentials`. CI success is build evidence, not native/browser interaction or distribution acceptance.
- **Independent same-host reproduction:** local and CI optimized native executables have identical SHA-256 `06c22370b3b1418c037fa9aa620b98017f1be5b9597e4955f17aa8c2c0fb6083`; each has zero checkout-path occurrences and the same nine toolchain registry paths. Fresh local and CI Web bundles from exact `41241f73` have identical manifests and **773/773 packaged files byte for byte**, with matching source SHA-256. Evidence is `target/headstart/evidence/repro-41241f73-20261027.json` plus the local build log. These comparisons do not establish cross-machine reproduction or browser acceptance.
- **Stage 9 browser composition:** the `41241f73` package loaded in the Codex in-app browser. Button count changed 0→1 and remained 1 through French and Arabic; Arabic reported `lang=ar`, `dir=rtl` and computed RTL direction. The reference host reported PASS for both compiled-preview and composition canonical roundtrips; visible Text and composition documents mounted, and column Add/Rename/Remove changed visible state. Live testing found that a leaf root left Add enabled and that Playground Add/Move hardcoded the root ID. Commit `9eb4d32c` fixes both, with focused Headstart tests **2/2** and a Web build passing. The browser then showed Add disabled for a leaf and correct Add/Move order for a `custom-root` column; the served directory was verified after the build's atomic swap. The browser bundle was built before the commit, and its composition source SHA-256 matches the committed file. Evidence: `target/headstart/evidence/browser-reference-41241f73-20261027.json` and `target/headstart/evidence/browser-composition-root-fix-20261027.json`. Exact-commit CI, real OS IME, wider browser/version and accessibility acceptance remain open.

- **Web implementation:** commit `a2ec04b0` packages the Playground image and
  link fixtures, uses browser asset URLs, remaps build paths, rejects checkout
  paths embedded in generated JS/Wasm, and gives path dependencies stable notice
  directories. Focused notice tests passed 20/20; the Web package fixture,
  Playground Headstart check and whitespace check passed.
- **Compiled CI:** [run 40440397456](https://github.com/abouzoubaa/rustui/actions/runs/40440397456)
  passed `source` and `macos-and-web-builds` on exact commit `a2ec04b0`.
  Its dedicated ephemeral macOS ARM64 runner exited and removed credentials.
  [run 40441478568](https://github.com/abouzoubaa/rustui/actions/runs/40441478568)
  passed both jobs on exact commit `24bf5451`; its runner exited and removed
  credentials.
- **Reproducibility:** after a committed local rebuild, its bundle and the
  independent CI checkout matched byte for byte for all **774 files**, including
  the manifest (source SHA-256 `dd6c921caeaa535b04bbf3ebde730dd4cc0183b22a8ca4a1986f95775ac87752`).
  This is same-host/toolchain evidence, not cross-machine reproduction. The
  unsigned local archive was stream-verified for all 774 members: SHA-256
  `e635217042b42906662c36c47ef647e5f375976d5bfd9bc55cde9295c8922344`.
  See `target/headstart/evidence/web-portable-repro-a2ec04b0-20261027.json`.
  The `24bf5451` local and CI bundles also match all 774 files byte for byte.
  Its unsigned candidate archive was stream-verified for all 774 files:
  SHA-256 `4d7d9683dfd124ab52a853ce14777762e1e63125a86ff96e8cc3f4e1d247151b`.
  See `target/headstart/evidence/web-portable-repro-24bf5451-20261027.json`.
- **Browser, bounded:** the current bundle displayed the packaged PNG in the
  in-app Chromium browser and Safari 27 on macOS 27. Safari opened the packaged
  Link fixture at `/assets/link-fixture.html`. Temporary tabs and servers were
  closed; the user's other Safari tabs were preserved. A misleading disabled
  Link story label found during that check has been corrected. Safari 27 on
  macOS 27 opened the packaged local Link fixture from the current `24bf5451`
  build; returning to Playground showed the disabled Link still labelled
  `Disabled link fixture` and the enabled Link activation count at 1.
- **Release state:** the provisional inventory has 37 requirements. The strict
  confirmed lower bounds are **3/37 coding complete and 3/37 acceptance
  complete** (R00–R02); most other rows have substantial source implementation
  but retain an unverified contract or release gate. This milestone strengthens
  R28 and R30 without closing either. Five native and
  three Web dependency-notice gaps, supported OS/browser versions, wider
  functional/locale/RTL and workload acceptance, and signed distribution remain
  open. Accessibility and VoiceOver, Japanese and other languages are explicitly
  undone/deferred under the current release scope.

## Current checkpoint — 2026-10-27

### Card story English, French, Arabic copy

- **Coding:** The Card Playground story now resolves its title, controls, placeholders,
  omitted-card message, and bound-value summary from the active locale catalog. Empty
  initial field values expose translated placeholders while preserving user-entered
  values across locale changes and keyed child retention. The previously compiled but
  unreachable Card gallery is now a selectable sixteenth canonical Playground story,
  with localized title and summary and a catalog-key route. This is a Stage 5/6 / R13
  source improvement, not completion of all Playground copy.
- **Source checks:** Three runtime locale records validated; focused Headstart
  Playground check and formatting passed. Library tests ran 211/212; one search
  index assertion exposed the inserted row and passed 1/1 after correction.
  A source-bound Web/Wasm bundle at `09ffc399` showed Card selectable and
  English/French/Arabic copy in one in-app Chromium profile. French `Bonjour`
  and Arabic `مرحبا` entered into separate fields survived parent update,
  first-card removal/restoration, and locale switches. The Arabic screenshot
  showed RTL placement without visible clipping at 1280×720. Evidence:
  `target/headstart/evidence/card-story-web-09ffc399-20261027.json` and
  `card-story-ar-09ffc399-20261027.jpg`. This is bounded Web acceptance;
  browser-version coverage remains open.
- **Native, bounded:** `./scripts/verify-release.sh macos` on `09ffc399` passed
  1,358 tests across 40 groups and packaged the executable with five unresolved
  notice gaps. The disposable macOS 27 app showed the three locale copies,
  parent update, and retained `Bonjour` and `مرحبا` values after first-card
  removal/restoration. Arabic `typeText` sent no characters; focused paste
  entered `مرحبا`. Evidence: `target/headstart/evidence/card-story-native-09ffc399-20261027.json`
  and `card-story-native-ar-09ffc399-20261027.jpg`. The app quit, its exact
  bundle and standalone executable were removed, and the GUI slot was released.
  Real OS Arabic keyboard/IME, screen reader, other macOS versions, and wider
  workload acceptance remain open.
  The 37-requirement inventory and strict
  3/37 coding and 3/37 acceptance lower bounds are unchanged.

### Narrow Web Playground layout

- **Coding:** At 480×720, the fixed catalog and reference columns clipped French
  Stacks notes and controls. References now appear below the story in its
  existing scrollable preview, leaving the catalog and story as the two main
  columns. No reference links or catalog behavior were removed.
- **Source/build:** `cargo-headstart fmt -p rustui-playground`, focused
  `cargo-headstart check -p rustui-playground --locked --offline`,
  `build-playground-web.sh`, and `git diff --check` exited 0 on macOS. The Web
  manifest records source commit `9a3e29d9` plus the working-tree source
  snapshot SHA-256 `8d024418c37c9cd76c9eb5366149df128e48a6658800c302fcb228617f657172`
  and 770 packaged files. Packaging still reports three unresolved notices.
- **Browser acceptance, bounded:** In the current-source in-app Chromium bundle,
  French and Arabic Stacks labels, notes, and controls were visible at 480×720.
  At 1280×720, scrolling the selected `button` catalog preview exposed its
  `Form.svg` and `Form-1.svg` reference links below the story. The temporary
  tab and server on port 8769 were closed. This does not establish Safari
  narrow-window behavior, other stories, native layout, screen-reader, or
  performance acceptance.

### Stacks layout copy and compiled CI — `3d1b4d01`

- **Coding (Stage 6 / R13):** Reused `LocalizedText` and the existing localized
  control APIs for seven remaining Stacks layout labels, notes, and row editor
  prompts in the active English, French, and Arabic catalogs. This is an
  incremental source change within the **37-requirement provisional inventory**;
  it does not establish full coding or acceptance completion for R13 or the
  project. Japanese and other languages remain deferred.
- **Source/build:** Three runtime locale records and catalog groups validate;
  focused `cargo-headstart check -p rustui-playground --locked --offline`,
  `cargo-headstart fmt --check -p rustui-playground`, `git diff --check`, and a
  source-matched debug build passed. The disposable native executable and bundle
  both had SHA-256 `a9b5694a21bbf3b55352a941f2b41045f226a4cc4a0e78987063b51b16cece59`.
- **Compiled CI:** [run 40435013690](https://github.com/abouzoubaa/rustui/actions/runs/40435013690)
  passed `source` and `macos-and-web-builds` on exact commit
  `3d1b4d0149998dfee0a167192a986944f70b66bb`. The earlier
  [run 40434628004](https://github.com/abouzoubaa/rustui/actions/runs/40434628004)
  passed source and macOS but stopped before Web compilation because the
  ephemeral runner lacked `RUSTUI_WASM_HEADSTART_ROOT`; the rerun supplied the
  existing approved Wasm root and pinned `wasm-bindgen 0.2.129`. The runner
  removed its credentials and exited after the successful job.
- **Native acceptance, bounded:** On macOS 27, the source-matched Playground
  showed English, French, and Arabic Stacks layout labels and notes. The French
  note wrapped and the Arabic note remained visible in the tested wide window.
  The app exited; its disposable bundle and standalone executable were removed.
  Narrow windows, full W1 cycle, broader input and visual behavior remain open.
- **Web acceptance, bounded:** The optimized bundle manifest names exact commit
  `3d1b4d01` and source SHA-256
  `3b586ced6b8b3c15d18da4321aa94f5962494835e4d8c22f5e11f8e5e728e06d`;
  all **770/770** packaged file hashes matched. In one 1280×720 in-app Chromium
  viewport, English and French notes wrapped inside the Stacks canvas and the
  Arabic copy remained visible in RTL. The temporary tab and port 8768 server
  were closed; the pre-existing tab on port 8766 was preserved. This does not
  establish cross-browser, narrow-viewport, IME, accessibility, or performance
  acceptance.
- **Second browser engine, bounded:** The same manifest-bound bundle also loaded
  in Safari 27.0 on macOS 27. In the tested wide Safari window, Stacks switched
  to French and Arabic; the new labels and notes were visible, with Arabic RTL
  placement. The temporary Safari tab was closed while the user's other 15
  tabs remained open. This does not verify older Safari/WebKit versions or
  narrower Safari windows.
- **Release status:** The provisional inventory still has 37 requirements;
  project-wide coding and acceptance completion counts have not been recomputed.
  Five native and three Web dependency-notice gaps, the declared macOS 13
  minimum versus macOS 27 runtime evidence, supported browser versions, wider
  platform behavior, performance, and packaging/distribution gates remain open.
  VoiceOver/accessibility and languages beyond English/French/Arabic remain
  explicitly undone or deferred under the current user direction.

### Compiled CI and Stacks/W1 localization — 2026-10-27

Manual compiled CI [run 40432048842](https://github.com/abouzoubaa/rustui/actions/runs/40432048842) passed both `source` and `macos-and-web-builds` on reviewed commit `c00b6cf2022ffa5a0691474862604ad38dfdf082`. The latter passed optimized macOS package and Web bundle steps. The dedicated, SHA-256-verified ephemeral macOS ARM64 runner used the approved `wasm-bindgen` CLI 0.2.129, exited with code 0, and removed its runner credentials. This is compiled CI evidence for that commit; it does not establish native GUI or browser acceptance.

The subsequent Stacks/W1 source revision localizes the visible retention controls, W1 controls, progress, expected counts, and guidance for English, French, and Arabic. Short direction-aware rows replace the long status text. The three catalog groups validate, formatting passes, and focused Playground Headstart check and debug build pass. The packaged executable matched its build artifact (SHA-256 `3b465de9f804f017124d53f4dcbc69be1954f0efe8ba6abfe9629b201e4b9592`). In the source-matched native Playground, all three locale switches changed the W1 controls and status. At a wide Arabic window, the W1 rows and four buttons were visibly complete; reversing keyed rows changed the translated control and parent mismatch, Reset restored the expected state, and Next advanced the localized count to Alpha 1/Beta 0. An app-name UI lookup also launched an older c611 bundle; its observations were excluded. Both tool-launched apps exited, and only this worktree's disposable bundle and standalone executable were removed. Evidence: `target/headstart/evidence/stacks-w1-locales-native-20261027.log`. Narrow-window, full six-step W1, and Web visual/interaction acceptance for this revision remain open.

The retained Stacks child component now resolves environment mode, count, increment button and notes placeholder from the same active catalogs. Formatting, 3/3 catalog validation and focused Playground Headstart check/debug build pass. The packaged executable matched its build artifact (SHA-256 `bc09841009829770d082ff34f9330318635107246aa6f4cebb1d4ab40d504618`). In the source-matched native app, the mounted Alpha/Beta rows showed Arabic environment/count text and notes placeholders, Arabic increment buttons were visible, and switching to French updated those mounted rows without resetting their count. The owned app exited and its disposable bundle/executable were removed. Evidence: `target/headstart/evidence/stacks-child-locales-native-20261027.log`. The dynamic accessibility action names remain English for the deferred accessibility pass; narrow-window and Web acceptance remain open. Generic story diagnostics below these rows also remain English.

On the current `7b85b29e` Web bundle, French W1 Reset and Next advanced expected and actual Alpha/Beta counts to 1/0, and Arabic/French mounted child copy changed with locale. A 1280×720 browser screenshot exposed clipping of the fourth French retention button in the one-row action group. The repair splits the four existing actions into two RustUI `HStack` rows; formatting, focused Headstart check, optimized Web build and debug native build/package pass. The new Web manifest binds `stories/mod.rs` SHA-256 `33563df25d572f41044c041609145bb1a14d326718d5b5dfa771e5aa85377079` with 770 packaged files and three unresolved notice gaps. At 1280×720, English, French and Arabic show all four controls in two complete rows. Clicking the formerly clipped French environment control changes the button and mounted children to detailed mode. A source-matched macOS 27 app also showed the four French controls in two complete rows at the observed wide window; packaged and build executable hashes matched (`d2897f4bcb1bef2784dc183dc2683a613df995c3a913a3361319a5162353c531`). The owned Web tab/server and native app/bundle/executable were cleaned up. Evidence: `target/headstart/evidence/stacks-controls-two-rows-20261027.log`. Narrow native windows, cross-browser, full W1 cycle and generic Stacks copy remain open.

### Stacks/Focus English, French, Arabic specimen — 2026-10-27

The Focus specimen now resolves its visible heading, explanation, state,
buttons, fields and child counters from the three active Playground catalogs.
Short explanation rows use the existing direction-aware RustUI `HStack` so the
Arabic copy starts at the leading edge without the earlier long-line clipping.
The locale validator passed for **3/3** runtime catalogs and the focused
Playground Headstart check and optimized build exited 0. In the source-bound
native Playground, English, French and Arabic Focus labels changed with the
locale, the Arabic focus action changed its translated state, and the Arabic
counter rendered a localized zero. A wide-window native screenshot showed the
three Arabic explanation rows complete and aligned to the right. The packaged
executable matched its build artifact (SHA-256
`c14be25282374090503e14aee16c52531d0d2a61b3ddeb24d59e8210b9663288`).
The app quit, no test app remained, and its disposable bundle and executable
were removed. Logs: `target/headstart/evidence/focus-gallery-locales-{check,release}-20261027.log`.
Narrow-window, Web, other Stacks controls and general RTL text wrapping remain
separate acceptance work.

### Final preview-action native check — 2026-10-27

At clean `180265ce`, `./scripts/cargo-headstart build -p rustui-playground
--release --locked --offline` exited 0. The disposable macOS Playground bundle
contained the same executable bytes as the build artifact (SHA-256
`4585b43b58fabad6a9a5f96e6e7e02af3da4815616ca4e8b5ebeaacc963e5766`).
With the claimed GUI slot, the Arabic Stacks/Parents specimen showed the
compiled-preview actions vertically inside the canvas; activating `Preview
TextField specimen` mounted generation 2 with its text field. The Arabic
Stacks/Focus first panel was readable at the observed wide native window, but
this does not close narrow-window or full story acceptance. The native app
quit, no RustUI test process remained, and its disposable bundle and standalone
executable were removed. Build log:
`target/headstart/evidence/native-preview-actions-180265ce-20261027.log`.
The independent public-API source consumer also passed a focused Headstart
check on this commit; the earlier full macOS checkpoint already included its
locked, offline check. No production release gate changed from these checks.

### Integrated Playground Grid RTL and locale — 2026-10-27

The shared specimen canvas now scrolls vertically; the Grid demonstration uses
RustUI's existing direction-aware `HStack` so its fixed-width card starts at
the leading edge, and its four visible labels resolve from English, French, or
Arabic catalogs. The focused Playground Headstart check, locale validator, and
Web release build exited 0. The Web manifest binds the current Grid source
SHA-256 `72056ee7f1bf2016f905eb3505e0407c234a4498e437d1bb5e1b6129904f1c5e`
and records 770 packaged files. In the current in-app browser tab, English
Grid remained left-aligned; Arabic Grid moved to the right, reversed 01–03 and
04–06, and rendered the new Arabic labels. Logs:
`target/headstart/evidence/grid-rtl-locale-check-20261027.log` and
`target/headstart/evidence/release-web-grid-rtl-locale-20261027.log`.
The attempted shared-canvas alignment wrapper clipped long Arabic Stacks text
and was reverted before this build. Compiled-preview RTL text/wrapping, broader
native and browser acceptance, and release notices remain open. The same
Arabic Web tab initially showed compiled-preview specimen buttons cut at the
canvas's left edge. A four-button horizontal action row measured 778 pixels,
wider than the approximately 700-pixel canvas. Replacing that row with a
vertical RustUI stack reduced its intrinsic width without changing actions.
The final macOS Playground check and Web release build exited 0; the latter's
manifest binds compiled-preview source SHA-256
`b4457d55c51c7d113a8089b59164a67182955e7edd948f27c036d9804be2f5f9`
and records 770 packaged files. In the current Arabic browser tab, preview
specimen buttons and all four actions begin at x=329 inside the canvas, and
their labels are fully visible after scrolling. Logs:
`target/headstart/evidence/compiled-preview-rtl-final-macos-check-20261027.log`
and `target/headstart/evidence/release-web-compiled-preview-rtl-final-20261027.log`.
This is a bounded layout repair; full RTL, accessibility and platform
acceptance remain open.
Commit `b4cec6a5` is fast-forwarded into the verified project branch and pushed.
On this commit, the source release script exited 0 and the macOS release
script exited 0: 1,356 Rust tests passed across 40 result groups, all-target
checking and optimized Playground packaging passed. The native bundle and
standalone executable both hashed to SHA-256
`4f87011f8cda21ad9f1cd44b2e9fbd89f3505de0bb4c3e56ee05aaae68988732`.
In the owned macOS 27 app, Arabic Grid occupied the right edge with mirrored
cells and Arabic labels; switching to French changed its visible labels and
native accessibility text. The app quit; the disposable bundle and executable
were removed, and no test app remains running. The source CI job passed on
`b4cec6a5` (runs `40427386465`, `40427378319`); compiled CI was skipped.
Logs: `target/headstart/evidence/release-source-grid-rtl-20261027.log` and
`target/headstart/evidence/release-macos-grid-rtl-20261027.log`.

### Integrated ScrollView checkpoint — 2026-10-27

On integrated `3fd612ce`, `./scripts/verify-release.sh macos` exited **0**:
**1,356/1,356** Rust tests across **40** result groups, the all-target check,
and optimized Playground packaging passed. The notice collector still reports
**5** native gaps. `./scripts/verify-release.sh source` and `web` also exited
**0** on the ScrollView source change (`46c46a25`); the Web bundle still has
**3** notice gaps. Logs: `target/headstart/evidence/release-{macos,source,web}-rtl-extent-20261027.log`.
These checks establish source/build status, not RTL visual acceptance.

The RTL ScrollView extent tracker now records a zero horizontal extent when
content shrinks to fit. Without that transition, later expansion compared
against the old width and could leave a newly scrollable RTL view at the wrong
edge. The focused state-transition test passed **1/1** using Headstart; log:
`target/headstart/evidence/rtl-zero-extent-20261027.log`. Native and browser
interaction for this new shrink/expand case remain open. The observed Arabic
Stacks > Focus text clipping and sparse canvas remain unresolved.

### Source release checkpoint — 2026-10-26

On `665e62e0`, `./scripts/verify-release.sh macos` exited **0** with
**1,355/1,355** passing Rust tests across **40** groups and an optimized
Playground package. The packaged executable byte-matches the release build
(SHA-256 `39a54cd2a9fc7da33fc844fa5c407fee78e10b86b04634d4b55078ab1e2fe3fa`).
Dependency notices were collected for **441** observed packages with **five**
unresolved gaps; no licensing completeness claim follows. Build log:
`target/headstart/evidence/release-macos-current-20261026.log`. This is build
and source-test evidence, not native GUI acceptance.

GitHub Actions runs `40421689969` and `40421676391` for integrated HEAD
`011fffe9` both completed successfully. This closes the current CI execution
check only; the native RTL and dependency-notice release blockers remain.

The first `verify-release.sh source` run stopped after **589 Python tests** with
seven setup errors in the semantic-adapter lifecycle fixture. That fixture
expected the bidirectional wrapping patch to be the final image, but the
approved input-hit patch now sits above it. The fixture now reverses the
input-hit image in its temporary copy before replaying the older patch; no
vendored source changed. Its focused **7/7** tests passed. The full source
driver then exited **0**: **589/589** Python tests, **19/19** Web semantic DOM
tests, **8/8** preview transport tests, packaging/CI fixture checks, runtime
locale validation, and theme source validators passed. Logs:
`target/headstart/evidence/release-source-locales-20261026.log`,
`semantic-adapter-stack-test-20261026.log`, and
`release-source-semantic-fixture-fixed-20261026.log`. Native/browser visual,
accessibility, performance, and dependency-notice acceptance remain separate.

### Stage 6 Playground copy — 2026-10-26

The Foundation Demos tab group now resolves all 15 visible tab labels from
English, French, and Arabic catalogs using the public
`TabItem::localized_label` API. The three runtime catalog groups validated,
the focused Playground catalog test passed **1/1**, and
`cargo-headstart check -p rustui-playground` passed. Logs:
`target/headstart/evidence/foundation-tab-*-20261026.log`. This is source
coverage; native visual locale-switch acceptance remains open.

On integrated `9ed62d4d`, the Web release driver passed **18/18** host
IME-plan tests and built the optimized bundle with three unresolved dependency
notice gaps. All **2,934/2,934** source and **770/770** bundle hashes verified.
One in-app Chromium profile showed live English, French, and Arabic Foundation
Demos tab names; the Arabic tab strip remained horizontally scrollable. This
does not close wider RTL layout, keyboard, cross-browser, or deferred
screen-reader acceptance. Evidence:
`target/headstart/evidence/web-foundation-tab-locales-20261026.md`.

The Utility Panel story now resolves its visible title, five tab names,
section headings, and three ColorPicker labels from the English, French, and
Arabic catalogs. `TabItem::localized_label` and
`ColorPickerView::localized_label` retain constructor fallbacks and resolve at
render time. Three runtime locale catalog groups validated; the focused
ColorPicker and Utility Panel tests passed **1/1 each**, and
`cargo-headstart check -p rustui-playground` passed. Logs are in
`target/headstart/evidence/utility-*-20261026.log`. Native visual and live
locale-switch acceptance for this story remain open.

The Hover Gallery now resolves its visible instructions, placeholder, controls,
and changing status from English, French, and Arabic catalogs. The localization
story's explanatory sentence also follows the active locale. `validate_locales.py`
passed for three catalog groups; the two focused Playground catalog tests passed
1/1 each (logs in `target/headstart/evidence/hover-story-locales-20261026.log`
and `target/headstart/evidence/i18n-description-20261026.log`), and
`cargo-headstart check -p rustui-playground` passed. This closes
source coverage for those two stories only. Other story literals, native visual
and live locale-switch checks, real Arabic input, and broader RTL/font acceptance
remain open. The Hover Gallery's accessibility live value still uses its English
fallback pending the deferred accessibility work.

The 2026-10-26 source-bound native run on `88f744b2` confirmed live English,
French, and Arabic Hover Gallery AX text and controls. The first app-name CUA
binding accidentally opened an older c611 binary, so its English-only result
was discarded; the current-source window was selected by absolute bundle path
and verified by executable SHA-256. Native visual acceptance remains **open**:
the current Arabic inner canvas at a scrolled position displayed clipped
story copy and large blank regions; English also overflowed horizontally.
See `target/headstart/evidence/native-hover-locales-20261026.md`. Both test
processes quit, the current-source disposable artifacts were removed, and the
shared GUI slot was released. The RTL layout failure needs a focused repair
and rerun before release.

The current-source `fb6399ca` native replay reproduced the Arabic Stacks >
Focus inner-canvas clipping after a two-page scroll. French at the same
position was readable from the left, though horizontal overflow remained.
Two bounded repair probes did not close the defect: logical-start alignment
still clipped content; tracking a changed RTL scroll extent exposed rightmost
controls but left large blank regions and clipped mid-canvas text. Both edits
were reverted. The focused extent helper test passed **1/1** for its candidate,
which is not release evidence. Exact candidate diff and observations:
`target/headstart/evidence/native-rtl-layout-probes-20261026.md`.

A new source-bound diagnostic replay used the same release executable hash
`39a54cd2a9fc7da33fc844fa5c407fee78e10b86b04634d4b55078ab1e2fe3fa`.
At Arabic Stacks > Focus, the inner canvas reported a **738 px** viewport,
**1,294 px** content width, horizontal offset **−331 px**, and maximum
horizontal offset **556 px** after switching tabs. The old offset explains
immediate clipping at the RTL leading edge; the earlier extent probe showed
that reanchoring alone leaves additional blank and clipped areas. The native
visual gate remains open. Raw diagnostics:
`target/headstart/evidence/native-rtl-current-scroll-diagnostics-20261026.log`.
The test app exited, its disposable bundle was removed, and the GUI slot was
released.

The retained ScrollView extent change passed its focused Rust test **1/1**.
An optimized native build of that change (`c5f96f606456ceb05731e34d1d8039827e483fc775b3d43331f70e11a530c86d`)
reanchored the Arabic Focus canvas to **−556 px**, exposing the right-side
controls. After two inner-canvas pages, long English story text still clipped
at the left, with sparse vertical regions. Two Focus-only frame-width probes
(`max_width(720)` with Start and Stretch alignment) built but failed to restore
readable wrapping; both were reverted. Their diagnostics are under
`target/headstart/evidence/native-rtl-focus-*-probe-*-20261027.log`.
This is partial coding progress; RTL native visual acceptance remains open.

- **Stage 6 bounded native Arabic pointer acceptance:** On integrated source
  `cc11658d`, the packaged macOS 27 Playground executable matched its
  optimized Headstart artifact (SHA-256 `e0ea58deffb199ed48952d247f95dbf3ae1d52ed3c0449d55db666e129d780b3`).
  Arabic Text inputs accepted mixed text by paste; pointer drag selected a
  subset, and replacement updated Name, Search, Notes and the owner label.
  Exact selected range and real Arabic keyboard composition were not observed.
  The sole test app quit, disposable app/executable were removed, and the
  shared GUI slot was released. Evidence:
  `target/headstart/evidence/native-arabic-pointer-20261026.md`.
- **Stage 11 mixed Arabic pointer hit, current milestone:** A same-version,
  exact-image GPUI patch now carries Cosmic bidirectional glyph clusters into
  single-line caret/hit mapping; pinned package revisions and upstream checksum
  files stay unchanged. RustUI's focused public GPUI regression passed **1/1**;
  macOS Playground all-target Headstart check and optimized Web build passed.
  Current macOS release driver passed **1,349 tests across 40 groups** and
  packaged Playground; Web release driver passed **18/18** host IME-plan tests.
  Native/Web dependency notices still have **5/3** unresolved file gaps.
  The rebuilt bundle verified **2,934/2,934** source hashes and **770/770**
  package hashes. In one in-app Chromium profile, Arabic Text inputs Name
  accepted `مرحبا RustUI عالم`; pointer clicks reached intermediate caret
  positions, a drag selected only UTF-16 range 6..11, and replacement updated
  the owner-backed value. Before the patch, the same clicks reached only start
  or end. Evidence: `target/headstart/evidence/bidi-input-hit-20261026/README.md`.
  The pinned GPUI library-test target still stops before execution on missing
  upstream font fixtures and `util_macros`. Real OS Web IME, native macOS,
  cross-browser, wider font/performance and screen-reader acceptance remain
  open; accessibility is user-deferred.
- **Integrated Stage 6 input and code-editor locales:** Through `5bada9f8`,
  Playground uses the English, French, and Arabic catalogs for its main input
  and code-editor controls. The current macOS release driver passed 1,348 tests
  across 39 groups. The Web release driver on `eea0b203` passed 18/18 host
  IME-plan tests and verified 770/770 bundle files, with three unresolved Web
  notice gaps. Bounded French and Arabic browser and native observations are
  recorded below. Coding and
  acceptance remain distinct: other story copy, native input/focus, RTL, wider
  browser coverage, and dependency notices are still open.

- **Stage 6/11 current-source Web story locale:** `./scripts/verify-release.sh web` exited 0 for commit `244196be` using the documented existing Wasm Headstart root at `/Users/Abium/Developer/rustui/target/headstart/wasm-root`; the worktree-local default root was absent, and the first attempt stopped before compilation. The successful run passed **18/18** host IME-plan tests, verified the approved patch stack, built the optimized Web package, and reported three unresolved dependency-notice gaps. The manifest records source SHA-256 `3c62607b85a7874dcbfad77cb88b6d505c6144f1bc8fbf601f83168b8fe50ec2`; **770/770** packaged files matched their recorded hashes, and manifest SHA-256 is `627bc4765dbfbf4252e150a7e95e565982bfe72154a3a04da5dbdb59e21a23a9`. In one in-app Chromium profile, French search for `Champs de texte` returned the framework demo, whose title, summary, bound-value label and Name field were French; switching to Arabic updated those labels and placed the Playground in RTL. The disposable browser tab and owned port 8766 server were closed. Evidence: `target/headstart/evidence/release-web-story-locale-20261026.log`. Search-field gallery copy and other input-story literals remained English in Arabic, and cross-browser, real OS IME, font/glyph, and performance acceptance remain open.

- **Stage 6 framework-story localization:** Commit `c37e0408` reuses the active RustUI catalogs for all 15 Playground demo titles and summaries in English, French, and Arabic. `ListView::localized_labels` resolves display text at render time while retaining source-index selection; localized search maps back to the same story. The bound-value label and Name field in the Text inputs story also resolve through the catalog. Three runtime locale records validated, focused list and Playground search tests passed **1/1 each**, and the changed-source macOS release driver exited 0 with **1,347 passing tests across 39 groups** and an optimized package. The bundled and standalone executable SHA-256 both matched `75c328047addb4667f2c56113bb3d43549ecfd0ef3d6f95b1243028b23ce114f`. In the owned macOS 27 app, the story list, selected title and summary changed to French and Arabic; the input bound-value/Name labels changed with them. Searching for Arabic `حقول النص` returned the selected Text inputs story. The owned app exited, its verified disposable bundle and standalone executable were removed, and the GUI slot was released. Evidence: `target/headstart/evidence/list-localized-label-test-20261026.log`, `target/headstart/evidence/playground-story-locale-test-20261026.log`, and `target/headstart/evidence/release-macos-story-locale-20261026.log`. Other literals inside stories still show English in French/Arabic, and current-source Web plus wider native locale acceptance remain open.

- **Stage 6 native French dead-key check:** On source HEAD `9ca086b00f798116699d0df765b2dccda7a64fb4`, `./scripts/verify-release.sh macos` exited 0 with **1,345 passing tests across 39 groups** and packaged the Playground. The bundled and standalone executable SHA-256 both matched `fb1f5768e8a5e826b80431f1a45cce6d424eb78fbd2ebde9a8d908c9ee1a662e`. In the macOS 27 app, the Text inputs story was set to Français; the focused Name field showed `´` after `alt+e` while the owner-bound value remained empty, then both field and bound value showed `é` after `e`. This is bounded dead-key commit evidence; CUA did not expose the native marked-text range, so active-composition acceptance remains open. The owned app exited, its verified disposable bundle and standalone executable were removed, and the shared GUI slot was released. Build log: `target/headstart/evidence/release-macos-marked-input-20261026.log`.

- **Dependency-notice gap classification:** Target-filtered, `--edges normal`
  reverse trees from the current locked Playground show all five native gap
  versions (`block 0.1.6`, `mac 0.1.1`, `malloc_buf 0.0.6`,
  `pathfinder_geometry 0.5.1`, `seahash 4.1.0`) and all three Web gap versions
  (`hexf-parse 0.2.1`, `mac 0.1.1`, `seahash 4.1.0`) reaching
  `rustui-playground`. Exact command output is retained in
  `target/headstart/evidence/notice-runtime-trees-20261026/`. A normal-edge
  path does not prove each crate contributes bytes to the final executable,
  but these gaps cannot be dismissed as build-only from the current graph.
  The exact published archives/upstream revisions reviewed so far provide no
  complete authoritative notice text; do not fabricate attribution or mark
  license distribution complete.

- **Stage 6 Playground event history:** The bounded five-entry log now retains
  message keys and arguments so existing entries render in the active English,
  French, or Arabic locale after a switch. Empty state, total count, and all
  StoryHost event messages use the same runtime catalogs; machine-readable W1
  diagnostic events remain unchanged. Event-log ownership/order tests **2/2**,
  three locale records validated, and Playground all-target Headstart check
  passed. Evidence:
  `target/headstart/evidence/playground-localized-event-log-test-20261026.log`
  and `target/headstart/evidence/playground-localized-event-log-check-20261026.log`.
  On integrated `c52460f4`, the Web release driver exited 0 with **18/18**
  host IME-plan tests and **770/770** verified package files (source SHA-256
  `d4f64fd65bfa5737f82d94a07da67b0e46445d68aac21dc6a511ce4af6327605`,
  manifest SHA-256
  `72fdd17238b20a30e65114f3064d92df337e848d603088f55210a9581d5997ba`).
  One browser profile showed Arabic increment event `زيد العداد إلى ١`,
  retained it across a switch to French as `Compteur augmenté à 1`, and
  showed the French empty state after clearing. A follow-up on the same
  source-bound bundle dispatched six increments: the log retained exactly
  events 2–6, showed total 6, and translated all five retained entries and
  the total into Arabic after a locale switch. Scrolling the preview exposed
  the rendered entries. The disposable tabs and servers were closed. Evidence:
  `target/headstart/evidence/release-web-localized-event-log-20261026.log`.
  GitHub run `40399810275` passed source CI on the evidence commit; compiled
  CI was skipped on the push. Native and wider story-copy localization
  acceptance remain open.

- **Stage 6 Playground catalog summary:** The header now formats catalog and
  demo counts through the active English, French, or Arabic message catalog,
  including locale-aware numerals. Focused runtime catalog test **1/1**,
  three locale records validated, and Playground all-target Headstart check
  passed. Evidence:
  `target/headstart/evidence/playground-catalog-summary-locale-test-20261026.log`
  and `target/headstart/evidence/playground-catalog-summary-check-20261026.log`.
  On integrated `974c34d4`, the Web release driver exited 0 with **18/18**
  host IME-plan tests and **770/770** verified bundle files (source SHA-256
  `c940b5b0492bab5a6130f72f53b69fd7cb59ab3fdaf2cb1467f3f7efefdee13d`,
  manifest SHA-256
  `764a09fd367147b38e4ceaa8d777dadedf5949fbf07238d743d7979ce12d232a`).
  One browser profile visibly showed `46 éléments du catalogue · 15
  démonstrations du framework` in French and Arabic `٤٦`/`١٥` with translated
  labels. The disposable tab and server were closed. Evidence:
  `target/headstart/evidence/release-web-catalog-summary-20261026.log`.
  GitHub run `40398644371` passed source CI; compiled CI was skipped on the
  push. Native and wider browser acceptance remain open.

- **Stage 6 native reference-state locale follow-up:** Playground reference
  rows now format available and missing states with the filename through the
  active English, French, or Arabic catalog. The focused catalog interpolation
  test passed **1/1**, three runtime locale records validated, and Playground
  all-target Headstart checking passed. Evidence:
  `target/headstart/evidence/playground-reference-states-locale-test-20261026.log`
  and `target/headstart/evidence/playground-reference-states-check-20261026.log`.
  On integrated `7d7cb280`, `./scripts/verify-release.sh macos` exited 0 on
  local macOS 27 arm64 with **1,344 passing tests across 39 reported groups**,
  zero failures, and an optimized Playground package. The source-bound app
  showed `Disponible · Form.svg` in French and `متوفر · Form.svg` in Arabic,
  with translated footer text after switching locale. Bundled executable
  SHA-256: `7f41a2ba446a4995096ea460bf9dcc137f40ce41cb281ed558f963b99d91eba2`.
  The owned process exited; its verified disposable bundle and standalone
  executable were removed; the GUI slot was released. Evidence:
  `target/headstart/evidence/release-macos-reference-states-20261026.log`.
  GitHub run `40397609648` passed source CI on this commit; compiled CI was
  skipped on the push. These checks close the bounded local native reference
  state case, not macOS 13, missing-reference visual behavior, broader RTL,
  input, accessibility, signing, or distribution acceptance.

- **Stage 6/11 reference-link locale follow-up:** Public RustUI
  `LinkView::localized_label_with_arguments` resolves the visible label at
  render time with a stable literal fallback and accessible name that follows
  the visible label unless explicitly overridden. Playground Web reference
  links use it for filename interpolation in English, French, and Arabic.
  Link regressions **17/17**, focused Playground locale interpolation **1/1**,
  three runtime locale records validated, and RustUI/Playground all-target
  Headstart check passed. Evidence:
  `target/headstart/evidence/link-localized-regressions-20261026.log`,
  `target/headstart/evidence/playground-link-locale-test-20261026.log`, and
  `target/headstart/evidence/link-localized-all-targets-20261026.log`.
  On integrated `e43c8e2b`, the Web release driver exited 0: **18/18** host
  IME-plan tests and **770/770** verified bundle files. Source SHA-256:
  `04e7b722808c80b1bad95f597c2b253123804f31f7582517795d1aa0f256edb5`;
  manifest SHA-256:
  `b18d9faebb26100de1e9f7213dd35e53150d0335c7ace89936fa94f0fda8d343`.
  One browser profile showed the visible and named link as `Ouvrir Form.svg`
  in French and `فتح Form.svg` in Arabic. The disposable tab and server were
  closed. Evidence:
  `target/headstart/evidence/release-web-localized-reference-link-20261026.log`.
  This closes the bounded Web reference-link locale check; wider browser and
  native reference behavior remain open. GitHub run `40397191576` passed its
  source job on this commit; compiled CI was skipped on the push.

- **Stage 6/11 dynamic Playground status:** The footer now resolves ready,
  missing-reference, requested-open, and open-error messages from the active
  English, French, or Arabic catalog. It keeps the reference count, key, path,
  and operating-system error as message arguments. The ready message no longer
  names macOS in the Web build. Focused catalog test **1/1**, three runtime
  locale records validated, and Playground all-target Headstart check passed.
  Evidence: `target/headstart/evidence/playground-localized-status-test-20261026.log`
  and `target/headstart/evidence/playground-localized-status-check-20261026.log`.
  On integrated `b87d6671`, `./scripts/verify-release.sh source` exited 0
  with **589 Python tests** plus JavaScript, locale, theme, and package source
  checks. `./scripts/verify-release.sh web` exited 0 using the documented
  matching Wasm Headstart root: **18/18 host IME-plan tests**, optimized bundle,
  and **770/770** package files matching size and SHA-256. Web source SHA-256:
  `0cd6560403dae0beeef37c217783631fd4fe8a42cc8685c8d22ef9f6a8e12757`;
  manifest SHA-256:
  `1d62695d83a4895da7325f0358db15d10ceb828f6c0051ef3ee81588f8cc0600`.
  One local browser profile visibly showed the correct footer after switching
  English, French, and Arabic. The disposable tab and server were closed.
  Evidence: `target/headstart/evidence/release-source-localized-status-20261026.log`
  and `target/headstart/evidence/release-web-localized-status-20261026.log`.
  Native and wider browser acceptance for this footer remain open. English
  reference-link and event-history text still appears in the Arabic view.
- **Latest source CI:** GitHub run `40395608527` on `b87d6671` passed its
  source job; compiled macOS/Web CI remains unrun.
- **Source CI:** GitHub run `40393859519` on `c8ef9d6b` completed successfully:
  source job passed. The compiled macOS/Web job was skipped because this push
  was not a manual `workflow_dispatch` run. No trusted self-hosted Mac runner
  is registered, so compiled CI remains open.

- **Current integrated macOS release and native shell checkpoint:** On clean
  commit `c8ef9d6b`, `./scripts/verify-release.sh macos` exited 0 on local
  macOS 27 arm64: **1,342 passing tests across 39 reported groups**, zero
  failures, and an optimized Playground package. The source-bound disposable
  app showed French and Arabic shell labels; Arabic Light and Dark actions
  visibly changed the theme while retaining Arabic. Bundled executable
  SHA-256: `e5656928109762e8bda81ce8246d475ed9e5de7bbc37ab00c268899317938949`.
  Evidence: `target/headstart/evidence/release-macos-shell-locales-20261026.log`.
  The app was quit, its disposable bundle and standalone executable removed,
  and the shared GUI slot released. This is one local native session; macOS 13,
  wider input/RTL, accessibility, signing, and distribution acceptance remain
  open.

- **Stage 6/11 Button story visible labels, 2026-10-26:** Public RustUI
  `ButtonView::localized_label` now resolves the visible label at render time
  with a stable literal fallback and identity seed; the default accessible
  name follows that visible label. The canonical Button story supplies
  English, French and Arabic count/control messages. Locale validation and
  Playground all-target checking pass; **2/2** focused Button tests pass.
  An optimized Web build from the working source produced **770/770** verified
  bundle files (source SHA-256
  `42b283241e3bba666bb9e24c74042f6029361f223727220f8d80360f1c95fc64`).
  Chromium showed French and Arabic visible labels and Arabic digits; the
  Arabic increment action updated the retained count ٠→١. This is bounded
  Button-story acceptance, not full Playground translation or cross-browser
  localization acceptance. The integrated-source release and bounded native
  checks below have since run; wider localization acceptance remains open.
- **Stage 6/11 Button integrated release and native locale checkpoint, 2026-10-26:**
  On commit `6f8ce90b`, `./scripts/verify-release.sh macos` exited 0 on local
  macOS 27 arm64 with **1,341 passing test cases across 39 reported groups**,
  zero failures, and an optimized Playground package. `./scripts/verify-release.sh
  web` exited 0 with **18 host IME-plan tests** and **770/770** bundle files
  matching its manifest (source SHA-256
  `2da78de2c3f48f48475593d41730738d2d655f5921ab9a5709c6668a43a1afa6`,
  manifest SHA-256
  `986452b4ffe41b2d5c67fd36ec087a2e95bd9976b1883f0106f60ddfb6da4ac8`).
  The current bundle showed the corrected Arabic note and localized controls in
  Chromium. A source-bound native Playground package launched: French showed
  translated controls and `Compteur : 0`; Arabic showed translated controls,
  `العدد: ٠`, and clicking `زيادة العداد` changed it to `العدد: ١`. Its bundled
  executable SHA-256 was
  `936436b142b89e0014188d7acf75dde1af6011ee96f99424a664235026a94fd5`.
  The test app was quit, no Playground process remained, and the disposable
  bundle and standalone executable were removed. This is one local macOS 27
  native session and one Chromium browser profile; broader macOS 13 and Web
  locale, keyboard, IME, visual, and performance acceptance remains open.
- **Stage 6/11 Button story chrome locale follow-up, 2026-10-26:** The
  Button story now resolves its clear-log label, layout summary, bounded-test
  status, and illustrative-source disclaimer through the English, French, and
  Arabic runtime catalogs. Locale validation, Playground all-target check,
  formatting, and the source release gate passed; the gate ran **589 Python
  tests** plus its JavaScript, packaging, locale, and static-theme checks.
  Evidence: `target/headstart/evidence/release-source-button-chrome-20261026.log`.
  Dynamic event-history messages and other Playground chrome are still in
  English; this checkpoint does not close full Playground localization. The
  earlier native and Web bundles predate this source change.
- **Stage 6 Playground search placeholder, 2026-10-26:** Public
  `TextFieldView::localized_placeholder` resolves the visible placeholder from
  the active RustUI catalog on each render, retains its constructor fallback,
  updates retained input presentation after locale changes, and makes the
  default accessible name follow that visible placeholder. An explicit name
  remains authoritative. Playground uses it for search and localizes the two
  sidebar headings in English, French, and Arabic. The focused TextField test
  passed **1/1**; Playground all-target check, locale validation, and formatting
  passed. Logs: `target/headstart/evidence/text-field-localized-placeholder-test-20261026.log`
  and `target/headstart/evidence/playground-localized-search-check-20261026.log`.
  On integrated commit `439a442c`, `./scripts/verify-release.sh source`
  passed **589 Python tests** and its other source gates;
  `./scripts/verify-release.sh macos` passed **1,342 test cases across 39
  reported groups** with zero failures and packaged an optimized app;
  `./scripts/verify-release.sh web` passed **18 host IME-plan tests** and
  packaged a Web bundle with **770/770** manifest hashes matching. The Web
  source SHA-256 is
  `e8262b7d6833b52d83e3fcdd7607c29247b7e024b56b3f43c118a8da3d51c27f`;
  manifest SHA-256 is
  `c634091909d4717ecacdad3536312ddbaefac1c9d6c7a8fa171197fea8428192`.
  Chromium and native macOS 27 both showed the sidebar headings and search
  placeholder switching from English to French and Arabic. Native bundled
  executable SHA-256 was
  `160df5a40b008a65e5f9e453a43eda065ede37e88ab66abb57ebfcd6660b46c1`.
  The native app and source-bound browser tabs were closed, disposable bundle
  removed, and GUI slot released. Logs:
  `target/headstart/evidence/release-source-localized-search-20261026.log`,
  `target/headstart/evidence/release-macos-localized-search-20261026.log`, and
  `target/headstart/evidence/release-web-localized-search-20261026.log`.
  This closes the bounded search-placeholder locale behavior, not full
  Playground localization or wider OS/browser acceptance.
- **TextField locale-key replacement follow-up, 2026-10-26:** The default
  accessible name now resolves from the current placeholder key at render
  time, so a later `localized_placeholder` call cannot leave it tied to an
  earlier key. Explicit accessibility names remain authoritative. The focused
  TextField group passed **39/39** and Playground all-target check passed.
  Evidence: `target/headstart/evidence/text-field-locale-regressions-20261026.log`
  and `target/headstart/evidence/playground-text-field-chain-check-20261026.log`.
  The full release drivers above ran before this narrow follow-up; they are
  current for the preceding commit, while this change has focused validation.
- **Stage 6 Playground shell locale follow-up, 2026-10-26:** More static
  shell copy now resolves through the three current runtime catalogs: search
  empty state, preview/demonstration fallback, references headings and action,
  source-gap labels, app heading, and Light/Dark controls. The existing public
  `LocalizedText` and `ButtonView::localized_label` paths were reused. Locale
  validation, formatting, and Playground all-target check passed; evidence:
  `target/headstart/evidence/playground-shell-locales-check-20261026.log`.
  Dynamic status, catalog/reference filenames and link labels, and other story
  text remain to audit. On integrated `d3dbeea5`, the Web release driver
  exited 0 with **18 host IME-plan tests** and **770/770** manifest files
  matching (source SHA-256
  `8f628241ebffa2a43afd6f72587aa87063562accd6e4592fd58739e5a02c7fc8`,
  manifest SHA-256
  `2cdfafc88c274c2c1e6452021f2484c5ba11bcb459c07c846bffc4bffdef43e9`).
  One Chromium profile showed the app heading, appearance controls,
  references heading, and source-gap note in French and Arabic. In Arabic,
  Light and Dark actions visibly switched the theme and preserved the locale.
  The test tab and owned port 8000 server were closed. Evidence:
  `target/headstart/evidence/release-web-shell-locales-20261026.log`.
  This is bounded Web behavior; native and wider browser acceptance for these
  new shell strings remain open.
- **Stage 6 input-story locale follow-up, 2026-10-26:** Playground's input
  story now uses the English, French, and Arabic catalogs for its principal
  labels and placeholders. `TextEditor::localized_placeholder` resolves the
  visible placeholder and fallback accessible name from the active catalog.
  Locale validation, Playground all-target check, the dark-search matrix
  regression (1/1), and localized-placeholder regressions (2/2) passed.
  The current-source macOS release driver exited 0 with **1,348 tests across
  39 groups** and five unresolved native dependency-notice gaps. The packaged
  executable and standalone release executable both had SHA-256
  `860da0684a2d8b841eb5480fd96295acdf0eb389e541af40e7c2c5ef3cf2f0fa`.
  In that exact app, French and Arabic input labels appeared in the native
  accessibility tree. Its owned process exited; only the verified disposable
  bundle and standalone executable were removed, and the GUI slot was released.
  The Web release driver exited 0 with **18/18 host IME-plan tests** and three
  unresolved Web dependency-notice gaps. The bundle manifest's **770/770**
  files matched their SHA-256 entries; the source SHA-256 was
  `c1dfff5820a2bdc5d9e8e34d9b656bbf5117edaffd8fa88e1db694a392821a3e`.
  One browser profile showed the new French and Arabic input labels. The
  temporary tab and owned port 8766 server were closed. Evidence:
  `target/headstart/evidence/release-macos-input-story-locales-20261026.log`
  and `target/headstart/evidence/release-web-input-story-locales-20261026.log`.
  Other controls and explanatory copy in this story still appear in English;
  wider locale, input, RTL, native, and browser acceptance remain open.
- **Stage 6 token-field locale follow-up, 2026-10-26:** The public
  `TokenField::localized_placeholder` now passes the active locale key to its
  contained `TextField`; the input story also localizes its owner-update
  action. Locale validation and Rust formatting passed, six focused token-field
  tests passed, and Playground all-target check passed. A current-source Web
  bundle verified **770/770** manifest files with source SHA-256
  `22d5b6439c7c0f60c151df755a8e13685a8248b3274c789f3425f74cd4437978`.
  One browser profile showed both controls in English, French, and Arabic;
  the temporary tab and owned port 8766 server were closed. Evidence:
  `target/headstart/evidence/token-field-locale-20261026.log`,
  `target/headstart/evidence/playground-token-field-locale-check-20261026.log`,
  and `target/headstart/evidence/web-token-field-locale-build-20261026.log`.
  Native behavior for this follow-up and remaining English input-story copy
  still need acceptance.
- **Stage 6 code-editor locale follow-up, 2026-10-26:** The input story's
  code-editor heading, instructions, lifecycle note, and seven actions now use
  the English, French, and Arabic catalogs. Public `CodeEditor` options resolve
  its placeholder and accessible name through the active catalog, with the
  supplied literal values as fallbacks. Locale validation, Rust formatting,
  five focused code-editor tests, and Playground all-target check passed.
  The current-source Web bundle verified **770/770** manifest files with
  source SHA-256
  `91bd2d618a75ad43da69bed7291de1cc9f53c8215a49eb3459cbb883673c677f`.
  One browser profile showed the translated heading, instructions, action
  labels, and editor accessible name in French and Arabic. The placeholder
  was not visually exercised because the editor contained sample text. The
  temporary tab and owned port 8766 server were closed. Evidence:
  `target/headstart/evidence/code-editor-locale-tests-20261026.log`,
  `target/headstart/evidence/playground-code-editor-locale-check-20261026.log`,
  and `target/headstart/evidence/web-code-editor-locale-build-20261026.log`.
  Dynamic code-editor status strings and native acceptance remain open.
- **Code-editor locale native integration, 2026-10-26:** The release macOS
  driver on integrated `e41ea35d` exited 0 with **1,348 tests across 39
  groups** and five unresolved native dependency-notice gaps. The packaged
  app and standalone release executable had matching SHA-256
  `cd0e8b48a344504328a34ea2ce38a67b133ea854e48564654c2e8e0a32dadfb9`.
  In that exact app, French and Arabic headings, instructions, action labels,
  editor accessible names, and placeholders appeared in the native
  accessibility tree. The editor was offscreen for direct input, so editing
  and focus retention remain unverified in this run. The owned app exited;
  only its hash-verified disposable bundle and executable were removed, and
  the GUI slot was released. Evidence:
  `target/headstart/evidence/release-macos-code-editor-locales-20261026.log`.
- **Stage 6 Web release integration, 2026-10-26:** The Web release driver on
  `eea0b203` exited 0 with **18/18** host IME-plan tests, **770/770** bundle
  manifest files matching SHA-256, and three unresolved dependency-notice
  gaps. The manifest records source SHA-256
  `91bd2d618a75ad43da69bed7291de1cc9f53c8215a49eb3459cbb883673c677f`.
  No additional browser acceptance was inferred from this build. Evidence:
  `target/headstart/evidence/release-web-stage6-integration-20261026.log`.
- **Current-source cache workload calibration, 2026-10-26:** The existing
  `cache_benchmark` example built with Headstart and the existing calibration
  runner completed **18/18 valid fresh-process records** on local macOS 27
  arm64: one warmup and five measured processes for each small, medium, and
  large scenario. Whole-process CPU medians were **3.898 s**, **4.052 s**,
  and **4.697 s**; peak RSS medians were **126,418,944 B**, **127,090,688 B**,
  and **127,975,424 B**. The measured binary SHA-256 was
  `bd15274869fff11c5fd78566135e5619f4c4c9fbbc4c485a5086e807e38e38ff`.
  It exited; its disposable executable was removed and the GUI slot released.
  Raw records and provenance:
  `target/headstart/evidence/cache-calibration-current-20261026.json` and
  `target/headstart/evidence/cache-calibration-current-20261026.log`.
  The older 2026-10-19 CPU medians are much lower, but its example source,
  compiler binary, lockfile, launcher, and calibration-runner hashes differ;
  this is not an apples-to-apples regression measurement. The difference
  requires investigation before performance acceptance. These values include
  startup and teardown, not frame/GPU timing or input-to-display latency.
- **Integrated macOS/Web release checkpoint, 2026-10-26:** On integrated
  commit `f428a8b8`, `./scripts/verify-release.sh macos` exited 0 on macOS
  27 arm64 using local Headstart: 1,339 passing test cases across 39 reported
  test-result groups, zero failures, optimized Playground build and app
  packaging. `./scripts/verify-release.sh web` exited 0 with the preserved
  matching Wasm Headstart root supplied through `RUSTUI_WASM_HEADSTART_ROOT`:
  18 host IME-plan tests, optimized Web build and package. The new Web manifest
  binds commit `f428a8b8`, source SHA-256
  `dafc9b584ec5077c93c2bcc9cc74a333746391bb656c293a0e11ef1d80e24ed4`,
  and **770/770** files verified against their hashes. The source digest is
  unchanged from the prior RTL browser observation; this run did not repeat
  browser input or visual acceptance. The native notice collector still has
  **5 gaps** and Web has **3**. Logs:
  `target/headstart/evidence/release-macos-20261026.log` and
  `target/headstart/evidence/release-web-20261026-matched-headstart.log`.
- **Release source gate after approved RTL wrapping, 2026-10-26:** The Web
  semantic adapter's exact postimage predates the RTL wrapped-row patch on one
  shared GPUI Web file. A read-only stack verifier now proves the selected RTL
  postimage, virtually reverses it, then replays the semantic adapter proof on
  the approved intermediate image. Its lifecycle test fixture uses that same
  intermediate image. The focused semantic suite passed **7/7**; the complete
  `./scripts/verify-release.sh source` gate passed **589 Python tests**, its
  JavaScript checks, packaging/CI fixtures, locale validation, and static theme
  checks. Log: `target/headstart/evidence/release-source-20261026.log`. This
  repairs release-tooling composition; it adds no native/browser acceptance.
- **Stage 11 RTL story canvas, 2026-10-26:** RustUI `ScrollView` now anchors
  horizontal RTL content at its logical start on initial render and locale
  changes. GPUI ignores `window.refresh()` during a draw, so the offset uses a
  deferred repaint. The focused macOS Headstart regression passed **1/1**.
  The source-bound Web bundle passed **770/770** file hashes (source SHA-256
  `dafc9b584ec5077c93c2bcc9cc74a333746391bb656c293a0e11ef1d80e24ed4`).
  In Chromium at 1280×720, the Arabic Text inputs owner-update button moved
  from clipped x=1007 to fully visible x=714–1001. Manual horizontal scrolling
  remained available; French reset to x=279 and Arabic reselected x=714.
  Evidence: `target/headstart/evidence/web-rtl-scroll-20261026/`. This closes
  that canvas starting-position defect, not wider RTL, native visual, real IME,
  or accessibility acceptance.
- **Stage 11 bidirectional Web wrapping selected, 2026-10-26:** The user-approved
  five-file same-version patch is selected through the exact-image helper and
  local `gpui-pre-wgpu 0.3.9` Cargo path. No GPUI revision or dependency version
  changed; pristine checksum metadata remains intact. The approved manifest
  records pre/post hashes and the helper verifies the applied image. Rustfmt,
  selected native Playground check, optimized Web bundle build, **770/770**
  bundle hashes, **2/2** fresh-target Cosmic row tests and **1/1** selected
  GPUI geometry fixture pass. In Chromium at 1280×720, the selected bundle
  painted the complete 302-byte Arabic Button note on two visible rows and
  the long Arabic Text note on three rows; Increment changed count 0→1.
  Bundle manifest SHA-256 is
  `446198a6ba65675a2a0955bf2514bfa3e3f8d33c61e9d82a6aab30b6ad917150`;
  its source SHA-256 is
  `80a8be5cf0d40d2f81730baad12eef7b5126242b843f35c922123cc6b4d110d3`.
  This closes the specific Arabic catalog painting defect. Wider browser text
  selection, real IME, glyph/font, performance and native visual acceptance
  remain. GPUI's own lib-test target still fails before this test on its
  existing missing test fonts and `util_macros` plus unrelated `aria_disabled`
  test errors. The default Headstart row-probe target reused a stale one-test
  executable after a source edit; a fresh target ran both tests successfully.

- **Stage 11 Arabic wrapped-row probe, 2026-10-26:** The reproducible
  `patches/proposed/cosmic-wrapped-rows-probe/` fixture passed 1/1 with
  `./scripts/cargo-headstart test --manifest-path patches/proposed/cosmic-wrapped-rows-probe/Cargo.toml --locked --offline --lib`.
  Cosmic retained early and late bytes across multiple rows and placed each
  glyph within the requested 180 px width. Arabic rows are right aligned in
  that width: glyph X can exceed the row's own reported `w`. The earlier
  private backend seam left GPUI paint and hit testing on the old single-row
  geometry; the approved five-file patch above addresses those paths and is selected. Coding and acceptance completion counts are unchanged.

- **Current Web inspector/Arabic check, 2026-10-26:** The Web build passed on commit `94789925` using the preserved matching Headstart Wasm toolchain read-only from worktree 4adc; build outputs stayed under this worktree's `target/headstart`. The current bundle manifest verified **770/770** file hashes and source SHA-256 `9815b4c1bfddcead0a197d84e405281334b7b013c69121d9ceb169eb41473b76`. Chromium showed the schema amount and text editors wholly inside the visible pane. Physical clicks focused both controls; replacing amount with `0.625` updated its editor, owner summary, and compiled preview value; replacing text with `Web inspector` updated its editor, owner summary, and compiled preview text field. The same current bundle still paints only a tiny fragment of the long Arabic Button catalog note while the semantic description contains the full Arabic sentence. Thus the inspector behavior passes this bounded browser check; Arabic long-text visual acceptance remains a release blocker. Source CI push and PR runs `40359612677` and `40359627869` passed on this commit; compiled Mac/Web CI remains skipped without a trusted runner.

- **Stage 9 compiled-preview inspector, bounded native repair:** `PropertyInspectorView` now caps its width at 600 px so the schema amount and text editors stay inside the visible Playground pane beside a wide preview sibling. `./scripts/cargo-headstart check -p rustui`, `./scripts/cargo-headstart build -p rustui-playground`, focused `schema_amount_editor_updates_preview_owner`, and `rustup run 1.97.1 cargo fmt --all -- --check` passed. The current-source disposable bundle and executable matched SHA-256 `559a408c3ed1d929a3bf47274ca8d2f8d6cf92f098c821a714b63146ddd8a50a`. Native macOS 27 inspection of the exact worktree bundle showed both fields fully visible; a physical click reached the amount field, and committing `0.625` updated the editor and owner summary. A text-field click and paste updated its owner in the preceding source-matched build. Display-name app selection also launched an older c611 bundle during the first attempt; that observation was discarded, and both tool-launched processes were terminated after UI close. The final current-source process was closed and terminated; the disposable bundle and standalone executable were removed. No Playground process remained. This verifies these inspector behaviors only, not full Stage 9 or release acceptance.

- **Stage 9 compiled preview native acceptance (bounded, 2026-10-26):** Source-matched optimized Playground executable and disposable bundle both had SHA-256 `32382ba3031f31b7a5c2f927d673954a14042b50b237756188b7bb99ea6bbbaf`. In the VStack + HStack story, editing the schema inspector text to `Schema roundtrip 2026` updated the owner and live compiled TextField; Export normalized descriptor included that value; Import remounted the preview at generation 2 with the value retained. Importing malformed `{` reported `invalid preview JSON: EOF while parsing an object at line 1 column 1` and left the generation-2 preview mounted. The owned PID 24855 was terminated after UI quit did not exit it; its disposable bundle and standalone debug executable are absent. A c611 Playground process that appeared during app-control cleanup (PID 28017, absent in the initial process inventory) was also terminated without removing its retained bundle or worktree. No Playground process remained afterward. This closes only the exercised round-trip and malformed-input behavior, not all preview, native, accessibility, or release acceptance.

- **Stage 11 Arabic Web text:** the browser paints only a tiny fragment of the
  long Button and Text catalog notes although their complete Arabic labels are
  in the semantic overlay. An existing-font-fallback candidate compiled but
  did not change the painting, so it was reverted. A temporary short Arabic
  note painted fully in the same panel. A Headstart probe of the pinned Cosmic
  text system produced descending glyph x positions for the 302-byte mixed
  Arabic note (`1003.38` to `917.80` across the first 16 glyphs), while
  GPUI's wrap-boundary calculation at
  `vendor/gpui-pre-0.3.9/src/text_system/line_layout.rs:245` uses increasing x
  deltas. This is a source-level cause candidate, not a repaired browser
  result. Temporary source and lockfile changes were restored; logs and the
  diagnostic record are in `target/headstart/evidence/web-arabic-note-20261026/`.
  A private Cosmic row-shaping seam is prepared in
  `patches/proposed/gpui-cosmic-wrapped-rows-seam-v1.patch`; it changes no
  selected runtime source. Patch applicability and formatting pass. The
  isolated path-patched package now type-checks for `wasm32-unknown-unknown`
  with the pinned Web Headstart toolchain
  (`target/headstart/candidates/rtl-seam-source/wasm-toolchain-check.log`). The
  prior panic occurred on the host selection path. The original lockfile was
  restored byte-for-byte. This seam does not yet implement wrapped-row painting
  or hit testing; Arabic long-text Web visual acceptance remains open.

- **Current source CI:** GitHub runs `40336215259` and `40336202405` both
  completed successfully at integrated commit `8459aea8`. Compiled macOS/Web
  CI jobs were skipped without the trusted self-hosted Mac runner; this does
  not close compiled CI or runtime acceptance.
- **Current native owner update:** exact-path macOS 27 Playground candidate
  kept the Name field editable through a five-second external owner-value
  replacement; typing `z` without another click updated the Name field and
  shared binding. Active OS IME composition and exact timer/keystroke order
  remain open. Evidence and app-provenance correction:
  `target/headstart/evidence/native-owner-update-current-20261026/result.json`.

- **Current local release entry points:** `./scripts/verify-release.sh macos`
  and `./scripts/verify-release.sh web` both exited 0 on commit `6d93147e`.
  The macOS command passed workspace checks, **1,285 library tests** and **53
  integration tests**, built an arm64 optimized Playground, collected 441
  observed-package notices, and packaged the binary at SHA-256
  `32382ba3031f31b7a5c2f927d673954a14042b50b237756188b7bb99ea6bbbaf`.
  The Web command passed 18 host IME-plan tests and produced a **770/770**
  hash-verified manifest at SHA-256
  `f45efa83fc5d468db2b9618be478780eaab8b588b98d20b03ce75734b86d87c6`.
  Native notice gaps remain **5**, Web notice gaps **3**; the macOS bundle has
  only an ad hoc signature and no team ID or sealed resources. Neither entry
  point proves GUI/browser/AX/performance or distribution acceptance. Evidence:
  `target/headstart/evidence/release-macos-6d93147e-20261026/` and
  `target/headstart/evidence/release-web-6d93147e-20261026/`.
- **Headstart incremental recovery:** the first macOS entry-point run and two
  workspace-library reproductions hit an ICE in `rustui-i18n` at
  `icu_decimal::preferences::DecimalFormatterPreferences`. The isolated
  i18n suite passed 82/82. Moving only the refreshed 38 MiB
  `rustui_i18n-2fqxt6404tk53` incremental directory into
  `target/headstart/evidence/compiler-incremental-quarantine-20261026/`
  let the exact workspace and macOS commands pass. No broad cache cleaning
  or dependency change occurred. The compiler defect remains a reproducibility
  risk until the incremental path is stable across future builds.

- **Stage 11 current Web bundle:** `./scripts/build-playground-web.sh` passed
  using the approved Wasm Headstart compiler and `wasm-bindgen 0.2.129` copied
  from the existing RustUI tool cache into this worktree's cache. The manifest
  names `a4d1613c`; all **770/770** listed files match their hashes. Its
  SHA-256 is `7d410bd772680868970ba5aec5244df2b4897359e8f77df89b368e5a0c41cb72`.
  In the in-app Chromium browser, the public-API Text inputs story changed
  bound value after the five-second task while Name stayed focused, and a
  subsequent `e` updated owner and Name to `External owner update 2e`.
  This is a bounded browser check with synthetic key input, not real OS IME
  acceptance. Three Web dependency-notice file gaps remain. The owned tab
  and server were closed and the GUI slot released. Evidence:
  `target/headstart/evidence/web-owner-update-a4d1613c-20261026/`.
- **Stage 9 live theme reload:** local Mac Headstart ran seven watcher tests;
  **7/7 passed** including invalid-file recovery and failed activation
  preserving the last good theme. This is source acceptance; native
  owner-release and live application acceptance remain open. Evidence:
  `target/headstart/evidence/theme-watcher-a4d1613c-20261026/headstart-test.log`.

- Source release checkpoint at integrated commit `a4d1613c` passed:
  `./scripts/verify-release.sh source` exited 0. This covers the source,
  packaging-fixture, locale, theme-adapter, patch-verification, and script
  checks in that target. Log:
  `target/headstart/evidence/release-source-a4d1613c-20261026/verify-release-source.log`.
  It does not execute compiled macOS/Web, GUI, browser, accessibility,
  or performance gates. Output still identifies the unapplied dark-radio
  `plus-lighter` renderer property.

- **Stage 9 D1b external owner update:** The public-API Playground Text inputs
  story now exposes a five-second owner-value change. Headstart locked/offline
  check and build passed; the packaged macOS 27 binary had SHA-256
  `4add853a5a23e3cac1f445065acf3393123c87c27a3a15ad013e51a1b40bf5e3`.
  Native AX/visual observation verified Name focus and shared bound values
  across a delayed update, then ABC Option+E marked input followed by an
  external owner update and E producing `é`. One typing/timer overlap produced
  a mixed value without captured event order and remains inconclusive. This
  closes the bounded focused external-value case, not all IME acceptance.
  The owned app/bundle/binary were cleaned and the GUI slot released. Detail:
  `docs/tasks/D1B_IME_SELECTION_NO_BLUR_ACCEPTANCE_RUN_SHEET.md`.
- **Stage 9 D1b ordered overlap:** exact-path macOS 27 release bundle from
  `3f0f1e5e` passed a controlled 13-character sequence spanning the five-second
  owner callback. AX snapshots show `abcde` before replacement, then
  `External owner update 1f` and later suffixes through `...1fghijklm`;
  owner and Name agreed at each sample. Subsequent typing needed no refocus.
  This resolves the prior mixed-value observation for the sampled sequence,
  while active OS IME composition and wider locale acceptance remain open.
  Evidence: `target/headstart/evidence/native-owner-update-ordered-20261026/`.

- Picker source event/lifecycle gate: local Mac Headstart ran
  `./scripts/cargo-headstart test -p rustui picker --lib` with **45/45 pass**,
  then the retained subscription replacement/owner-removal test with **1/1
  pass**. This covers the focused source paths, not native Picker interaction
  or broader owner lifecycle acceptance. Logs:
  `target/headstart/evidence/picker-headstart-20261026.log` and
  `target/headstart/evidence/picker-subscription-headstart-20261026.log`.
  Headstart also reports future incompatibility in pinned `block 0.1.6`:
  uninhabited static at `vendor/block-0.1.6/src/lib.rs:64`. Current tests
  pass; the future compiler compatibility policy remains open. Diagnostic:
  `target/headstart/evidence/picker-future-incompat-20261026.log`.
- Source CI for `3fd14804` passed in runs
  [40325714173](https://github.com/abouzoubaa/rustui/actions/runs/40325714173),
  [40325705931](https://github.com/abouzoubaa/rustui/actions/runs/40325705931), and
  [40325705466](https://github.com/abouzoubaa/rustui/actions/runs/40325705466).
  All three compiled macOS/Web jobs were skipped; the trusted runner gate is
  still open. This commit changed release notices and the inventory, not
  component source.
- Dependency notice progress: captured Harfrust 0.5.2's verbatim MIT notice
  from its crate-recorded upstream revision `efdae314` with URL and SHA-256
  bound to the vendored manifest. A fresh Web collector run covers 403 observed
  packages and reduces missing notice files from 4 to 3 (hexf-parse, mac,
  seahash). Five native gaps remain; the unique missing-version count is six.
  This is provenance evidence, not a complete license or compatibility claim.
  Evidence: `target/headstart/evidence/web-dependency-notices-harfrust-20261026/`.
- Web SecureField browser check: **4/4 bounded observations pass** against the
  manifest-verified `a8290b24` bundle: nine entered characters rendered as
  bullets; the browser accessibility value stayed `<redacted>`; select-all and
  replacement left one bullet; reveal showed only the replacement `x`, and
  hiding restored a bullet while the accessibility value stayed redacted.
  This used synthetic browser input, not real OS IME. The owned tab and server
  were closed, port 8766 was freed, and the shared GUI slot was released.
  Evidence: `target/headstart/evidence/web-secure-field-20261026/RESULT.md`.
- Web English/French/Arabic input acceptance: **6/6 bounded cases pass** in
  one 1280×720 in-app Chromium profile using the manifest-verified Web bundle
  from application code `a8290b24` (**769/769 package files**). The live mounted
  preview field and owner agree after `Café`, mixed `سلام 123 Café`, logical-end
  deletion, locale switching and full mixed selection across a focused delayed
  parent update. Generation 2 stays mounted; the update advances rerenders
  0→1 and selected replacement advances revision 19→20. Arabic mirrors the
  layout and renders joined labels. Evidence:
  `target/headstart/evidence/web-en-fr-ar-input-20261026/`. The owned tab and
  local server were closed; package/caches retained. Browser text input was
  synthetic; real OS dead-key/IME, other browser engines and full typography
  remain open. A browser Option+E probe entered plain `e`, so it is excluded
  from acceptance rather than called a RustUI defect. **Coding:** existing
  implementation reused; **acceptance:** 6/6 cases for this milestone within
  the provisional 37-area inventory.

- Native English/French/Arabic input acceptance: **8/8 bounded cases pass** on
  application code `a8290b24`, executable `60cc18a69b09…`, macOS 27 arm64.
  Ordinary and mixed Arabic/Latin selection survive focused delayed parent
  updates; French marked accent survives and commits; existing Arabic keyboard
  entry/deletion and bound mixed-text editing pass. All three locale switches
  retain owner value and mounted generation/revision. Existing implementation
  is reused. Evidence: `target/headstart/evidence/native-en-fr-ar-selection-20261026/`.
  Original keyboard configuration restored; owned PID/bundle cleaned and GUI
  slot released. This closes case A and the bounded French dead-key exercise,
  not exhaustive text, typography, browser or external-value-during-composition
  acceptance. Japanese/other languages/scripts and accessibility/VoiceOver
  remain undone/deferred. **Coding:** no new component work; **acceptance:**
  8/8 cases for this milestone, within the still-provisional 37-area inventory.

- Source CI on `f3b18e7a` is complete and successful: runs
  [40317935732](https://github.com/abouzoubaa/rustui/actions/runs/40317935732),
  [40317926088](https://github.com/abouzoubaa/rustui/actions/runs/40317926088), and
  [40317923839](https://github.com/abouzoubaa/rustui/actions/runs/40317923839).
  This is source CI evidence; trusted compiled Mac/Web CI remains open.

- **User release-scope decision, 2026-10-26:** English, French and Arabic are the
  current required languages. Japanese and every other language/script are
  undone/deferred for later, including native Japanese IME acceptance. Continue
  French accents, English input, Arabic RTL/mixed-direction text, locale and font
  verification. The temporary Japanese keyboard and automatically added dictation
  entry were removed; original enabled/selected input sources and ABC layout
  match the before-snapshot, and the pre-existing Settings page was restored.
  No Japanese composition acceptance ran. Existing language/font implementations
  are preserved; the deferral does not establish their acceptance.

- Native GraphView/NodeEditor acceptance passes **11/11 + 16/16 bounded checks** on the verified reproducible `d857e6b5…` candidate. Existing implementation is reused; no component repair was needed. Checks cover current-owner topology edits, drag/key movement, both connection orders, invalid-direction feedback, incident-edge deletion, viewport behavior, disabled rejection and omission/remount. Inconclusive initial disconnect/reconnect attempts are excluded and replaced by visible pointer removal verified empty before reconnect. Evidence: `target/headstart/evidence/native-graph-editor-20261026/`. Owned app/bundle cleaned; GUI slot released. Wider focus, IME, locale/RTL, release-outside-surface and workload acceptance remain open; accessibility/VoiceOver undone/deferred.

- Native component acceptance: **13/13 Outline + 18/18 Drawer** bounded functional
  checks pass on the verified reproducible `d857e6b5…` candidate, macOS 27 arm64,
  1320×820 logical viewport. Outline covers selection/navigation, stable identity,
  branch normalization and unmount/remount; Drawer covers owner edit/action/title
  refresh, all four physical edges, dismissal and owner-retaining release/remount.
  Early incorrect test expectations and unsettled frames are excluded and recorded;
  the settled Top-edge view renders correctly. Evidence:
  `target/headstart/evidence/native-outline-drawer-20261026/`.
  All owned processes/bundles are cleaned and the shared GUI slot released.
  Source coding was already present; this adds acceptance evidence. Wider visual,
  locale/RTL, focus, popup/text-selection and performance gates remain separate.
  Accessibility and VoiceOver remain undone/deferred.

- Web reproduction passes on `39da1d7b`: two initially absent target directories
  each report **496 rebuilt / 0 fresh Cargo artifacts**. Raw/processed Wasm,
  JavaScript, assets and manifest match byte-for-byte: **770/770 package files**
  (769 payload files plus the manifest). Both builds preserve locked sources and
  approved tool identities, with explicit `RUSTUI_REMAP_BUILD_PATHS=1`.
  Evidence: `target/headstart/evidence/web-clean-reproduction-20261026/`.
  This establishes same-host/same-toolchain reproduction, with no new browser
  acceptance for these candidates. Wider acceptance and three Web notice gaps remain.
  Normalized archives from both packages also match, with all 770 files checked;
  retained archive SHA-256 is
  `f479c2d07e774af5b5dba5d799a47e8981e28644fd1aa841ec9f202c6e420b27`.

- Requirement-authority correction: a full compiler-from-source bootstrap was
  introduced as a gate by recent consolidation notes, not the original roadmap,
  completion gates or approved Headstart instructions. Keep the installed
  compiler identity/configuration and experimental limitations disclosed;
  compiler-source binding/bootstrap is a follow-up, not an added release blocker.
  No compiler source, binary, flag or pinned dependency was changed.

- Native reproduction defect fixed: html5ever embedded an absolute generated
  source path, changing the executable between target directories. The opt-in
  `RUSTUI_REMAP_BUILD_PATHS=1` launcher mode and native optimized release gate
  normalize paths while preserving caller flags and approved compiler behavior.
  **Two independent builds are byte-identical**, each with 555 rebuilt / 0 fresh
  Cargo artifact records. Seven reporter/launcher tests pass; the exact remapped
  candidate passes **7/7 functional smoke states**, is archived unsigned with a
  file manifest, and its test app is cleaned up. Evidence:
  `target/headstart/evidence/native-clean-reproduction-20261026/`.
  Cross-machine and native archive reproduction, version policy,
  licensing and signed distribution remain open. No upstream source changed.

- Stage 10 release W1 baseline: **3 fresh app test runs, 90/90 measured cycles,
  630/630 functional child-state observations** pass. Median gross CPU is
  0.807667 CPU seconds/cycle, median RSS is about 125 MiB, and between-run CPU
  range is 4.664%. These are CPU/RSS observations with automation pauses, not
  response time, FPS, leak freedom or an optimization benefit. Raw records,
  verified hashes and cleanup are under
  `target/headstart/performance/w1-release-20261026-4e00ad14/`.
  The W1 reporter's profile/protocol bug is fixed with five focused regressions;
  historical measurement values remain unchanged. Frame/latency and broader
  workload acceptance remain open; accessibility remains undone/deferred.

- Source CI passes on `4636ee53`: [run 40310616581](https://github.com/abouzoubaa/rustui/actions/runs/40310616581).
  Its Ubuntu source job passed; the manually dispatched Mac/Web compiled job was
  skipped. This does not establish Linux product support or compiled CI.
  Root/Playground guidance now matches 15 demos and 46 catalog rows; release
  commands, target evidence, candidate provenance, cleanup and outstanding
  distribution gates are documented in `docs/RELEASE.md`.

- SplitView/ResizablePanel now has a directly searchable public-API Playground
  story reusing the existing gallery. Six focused tests, optimized Mac/Web builds
  and packaging pass. Current source `5a37a8d6` passes **11/11 native** and
  **11/11 Web** bounded functional cases, including both divider axes, owner
  lifetime, reorder, reinsertion and range reconciliation. Together with the
  component checkpoint below, this turn records **55 Web + 11 native** passes;
  these are scoped checks, not project percentages. Evidence:
  `target/headstart/evidence/split-direct-route-20261026/`. All owned test apps,
  tabs/server and disposable native executables are cleaned up; GUI slot released.

- Functional Web checkpoint on source `e50b6ff0`: **44/44 bounded checks** pass
  across Outline (12), Drawer (13), GraphView (9), and NodeEditor (10). These
  exercise owner state, controlled edits, dismissal, lifecycle, viewport, and
  disabled behavior. Manifest verification matched all 769 bundle files and
  2,932 build-input files. Early unsettled/offscreen/uncleared-state attempts
  were excluded and affected checks repeated from verified states. Evidence:
  `target/headstart/evidence/web-outline-drawer-20261026/acceptance.json`.
  This is one Chromium profile at 1280×720; native, broader browser, performance,
  and visual/locale coverage remain open. Accessibility is undone and deferred.

- **User priority decision, 2026-10-26:** accessibility acceptance and further
  accessibility implementation/patch promotion are deferred alongside VoiceOver.
  Mark these checks **undone**; retain existing accessibility behavior and
  previously gathered evidence. Prioritize functional correctness, lifecycle,
  state synchronization, builds, packaging, and performance. The prepared row
  patches stay unapplied; their promotion decision is deferred. This priority
  decision does not establish accessibility acceptance or full production readiness.

- Stage 1 M16 native owner-task acceptance passes **3/3 bounded cases** on
  source `e50b6ff0`: cancellable timer stops on unmount; noninterruptible native
  work finishes without delivery to the removed owner; restored owner completes
  normally. Both restorations start Idle, with exactly one release per removal.
  Evidence: `target/headstart/evidence/native-owner-lifecycle-20261026/acceptance.json`.
  Focused probe tests **2/2**, optimized native and Web builds passed. This closes
  the M16 exercise, not the wider F1E acceptance or VoiceOver, which remains undone.

- Stage 9 D1d client teardown now defaults to the latest accepted reply stamp;
  previously an omitted stamp sent null and rejected teardown after mounting.
  Both profiles retain explicit stale-stamp rejection. **8/8 JavaScript
  transport/client fixtures pass**; combined semantic/transport fixtures pass
  **27/27**. The optimized local Mac Headstart Web build passes, all **769**
  packaged file hashes match, and real Chromium preview/composition edit/export,
  stale disconnect rejection and default teardown pass **2/2 profiles**.
  Evidence: `target/headstart/evidence/d1d-default-disconnect-20261026/`.
  Broader browser, visual and screen-reader acceptance remains open.
- Stage 10 props freshness completed **3/3 native logical workload scenarios**
  (1/8/32 children): every changed props value 1–100 was observed on each side,
  with expected cache-hit and changed-props lowering counts and inactive AX.
  Raw artifact hashes and contents were independently rechecked in
  `target/headstart/evidence/stage10-production-20261026/props-evidence-verification.json`.
  This is lowering/freshness evidence, not visual or latency acceptance.
- Implementation fixes are committed: security `0e1f39a7`, isolated secure-story
  state `5fce25d3`, disabled semantic controls `7914ecda`, selected Web adapter v9
  `9d10622e`, and same-version native focus export/local selection `84f3a477`.
  GPUI-pre 0.3.9 and Components/Base 0.7.3 remain pinned.
- Latest full checkpoints pass: **581 Python tests**, **1,282 workspace library tests**
  (950 RustUI), **53 public integration tests**, workspace all-target checking,
  and optimized native packaging. Macro doctests ran with zero cases.
  Web v9 also passed its optimized build, **18 Rust planner tests**, **19
  JavaScript semantic fixtures**, **6 D1d transport fixtures**, and the bounded
  real-browser checks recorded in the D1d and Web v9 evidence.
- Native focus implementation is complete and its named-field AX retest passes.
  **7/8 bounded native acceptance checks pass** in the current-source retest:
  focused named field, search clearing, keyboard-focused Space activation,
  Enter reset, forward/reverse Tab skipping the disabled button, coordinate
  activation and matching AX/rendered Count:1. The earlier `noWindowsAvailable`
  and AX/pixel mismatch did not reproduce in this run; no root cause is claimed.
  VoiceOver is **undone**, explicitly left for later by the user on 2026-10-26.
  This postpones the check; it does not establish screen-reader acceptance or
  waive the production gate. Evidence:
  `target/headstart/evidence/native-input-retest-20261026/`.
- Native Tree testing found missing Home/End navigation; arrows, collapse and
  expansion worked. RustUI now handles ordinary Home/End in the shared Tree
  adapter, selecting enabled visible edges and preserving modified/IME input.
  Outline inherits the repair. **30/30 focused Tree/Outline tests pass**,
  including real rendered keyboard events and disabled/hidden-row cases.
  The rebuilt native app now passes **2/2 Home/End checks**: End selected Tree,
  then Home selected Controls. Its optimized Web build and **769** packaged
  hashes pass; bounded browser keyboard results are recorded below.
  Evidence: `target/headstart/evidence/native-tree-home-end-20261026/`.
  No vendored source was changed.
- The source-bound in-app browser also passes **2/2 Tree Home/End checks** and
  projects hierarchy levels 1/2 with a focusable Tree root. Separate semantic
  activation gaps remain: primary-mousedown-only Tree rows advertise no Click,
  and selectable Lists project list/listitem with selected state. Two exact
  same-version upstream patches are prepared, **not promoted**: **4/4 isolated
  candidate tests** and **3/3 checker regressions pass**. Selected vendor sources
  are unchanged. See `WEB_ROW_ACCESSIBILITY_PROPOSAL.md`; runtime semantic and
  actual screen-reader acceptance remains open.
- The row-proposal source checkpoint passes **584 Python tests**, **27
  JavaScript semantic/client fixtures**, packaging fixtures and source guards.
  The selected Rust workspace was not recompiled by this source-only check;
  candidate probe results remain separate from production acceptance.
- The native patch verifier requires Cargo's local path selection: registry
  packages are treated as immutable and can reuse an original artifact after
  patching. Metadata, actual local backend compilation and a changed executable
  hash establish the tested source. Six empty-Cargo-cache fixture tests pass.
- Playground's password field uses separate secure state. Its rebuilt browser
  owner is password-type and public labels/inputs do not echo that value.
  Protected replacement moved selection and cleared, but value inspection was
  inconclusive; real OS IME, clipboard and screen-reader acceptance remain open.
- Rustls-webpki 0.103.16 fixes RUSTSEC-2026-0382 with unchanged dependencies and
  verified official source. A fresh OSV query reports no advisories for it.
  Native dependency notices still have **5 gaps** (Web **4**); packaging success
  does not close licensing or publication gates.
- Primary project branch `work/m118-diagnostics-provenance` was fast-forwarded
  from `567f933c` to checkpoint `3d078e7a`. All **1,092 related dirty source
  blobs** and the original staged index were verified in recoverable snapshot
  `codex/preserved-primary-20261026` before integration. **24 unrelated paths**
  and **326 existing ignored Trail cache files** retain exact bytes/modes; the
  primary tracked tree is clean. Four missing Trail documents were restored.
  Integration evidence: `/tmp/rustui-primary-integration-20261026`.
  The later guarded fast-forward to `1ec22c6b` integrates the independent source
  consumer, owner callback API, conditional-macro fix and D1d host. All 24/326
  preserved paths were rehashed and their modes rechecked; tracked tree remains
  clean. See `d1d-integration.json` in the same evidence directory.
- Independent source consumption is now covered by a standalone workspace
  example using only the public RustUI API. Its locked offline compile passed;
  metadata confirms all five approved local GPUI packages and no new dependency
  versions. The example's own root carries the required patches and scoped
  Headstart macro profile settings. This is compile evidence, not GUI acceptance
  or a crates.io publication claim.
- Stage 9 callback preparation is committed as `5b0c57f9`:
  `OwnerAction::dispatch_with` reuses the weak foreground owner updater and
  distinguishes unavailable owners from valid no-notify replies. Two focused
  real-GPUI owner tests pass, including distinct owners and released-owner
  inertness. D1d now has a specified, implemented consumer-owned browser reference
  host: [contract and evidence](D1D_BROWSER_PREVIEW_HOST.md), **10/10 implementation
  contracts**. Both real-browser profiles pass canonical roundtrips, stale-stamp
  rejection and disconnect/reconnect. Actual owner increment/default-deny and
  protected export checks pass. Broader browser/platform acceptance remains open.
  The integration also fixes conditional macro defaults referencing unavailable
  platform types; its compile regression and 40 macro unit tests pass.
- The exhaustive upstream notice audit recovered no complete authoritative
  texts for the five remaining gaps. Exact published-source/revision evidence is
  retained in `/tmp/rustui-remaining-notices-upstream-audit`; no partial header or
  generic license template is represented as an upstream notice.
- Previous unsigned native candidate is archived as
  `target/headstart/release-artifacts/RustUI-Playground-macos-arm64-unsigned-1ec22c6b.tar.gz`;
  its JSON manifest records binary/archive hashes and five notice gaps. No
  current-candidate native GUI acceptance is claimed. The generated app bundle
  was removed after archiving; no RustUI test app remains running. Normalized
  archive timestamps do not establish independent compiler reproducibility.
- The newer unsigned native candidate includes the Tree keyboard repair:
  `target/headstart/release-artifacts/RustUI-Playground-macos-arm64-unsigned-1e25994d.tar.gz`.
  Its manifest records **816** packaged file hashes and **5** unresolved notice
  gaps. Native Home/End acceptance is recorded for this binary; broader native
  acceptance remains open. The disposable bundle/executable were removed and
  shared caches preserved. No signing or external publication was performed.
- Current Chromium locale checks pass for English/French/Arabic live switching,
  decimal/plural/ordinal output, localized accessible names and mirrored RTL
  layout, including bounded Arabic light/dark screenshots. They exposed missing
  Chinese glyphs in the existing Text sample. Selecting GPUI's existing
  `EmojiAndCjk` fallback repairs that sample in the rebuilt Wasm bundle; locked
  sources/versions and the independent source-consumer compile remain valid.
  This is approximate horizontal grapheme fallback, not full script/vertical
  typography acceptance. Evidence: `/tmp/rustui-preservation-20261025/browser/locale-cjk-v1`.
- Branch `codex/production-consolidation-01a172cd` is pushed. Actual GitHub source
  CI now **passes on `6ad24006`**: [run 40271375045](https://github.com/abouzoubaa/rustui/actions/runs/40271375045),
  including all 581 Python tests, semantic/transport JS fixtures and packaging
  gates on Ubuntu 24.04. This exposed and resolved Python 3.12 deep-fixture
  cleanup errors, filesystem-case assumptions and an unnecessary compiler probe
  for a missing binary. The repairs retain depth/descriptor/security checks and
  also pass eight focused local tests. Guarded integration to `6ad24006` retained
  all 24 unrelated files and 326 ignored cache files exactly. This Ubuntu job
  validates tooling, not Linux product support. The repository has **zero
  registered self-hosted runners**, so its separately gated macOS/Web Headstart
  CI job cannot run. Local macOS Headstart and Wasm results remain separate.
- The later source CI run also passes on `a47cec88`:
  [run 40281866505](https://github.com/abouzoubaa/rustui/actions/runs/40281866505).
  Guarded integration of the preview teardown, verified props record and Tree
  keyboard repair retains all **24** unrelated and **326** ignored files exactly
  (bytes and modes), with a clean tracked primary tree. Evidence:
  `/tmp/rustui-primary-integration-20261026/tree-preview-integration.json`.
- Whole-project inventory: **37 provisional requirements**. Full coding and
  acceptance percentages have not been recomputed from every contract; the
  bounded counts above are not project completion percentages. Remaining
  native/browser/locale/RTL/font and IME acceptance,
  performance, reproducibility, notices and signing/distribution remain open.

Evidence: `/tmp/rustui-preservation-20261025/browser/semantic-v9`,
`/tmp/rustui-preservation-20261025/native-focused-element-v1`,
`/tmp/rustui-primary-exclusions-analysis`, and current checkpoint logs under
`/tmp/rustui-production-*-checkpoint-20261026.log`. No test app is running;
owned disposable GUI bundles/executables were cleaned up and the slot released.

## Approval and current integration — 2026-10-25

The user approved pending upstream patches and locale strategy/dependencies.
Same-version repairs use exact, reversible, versioned patch helpers. GPUI 0.3.9
and Components 0.7.3 remain pinned; upstream license/checksum metadata is preserved.

Coding remains a 37-area provisional inventory. Tree root/name/levels, locale,
graph controls, link contrast and descriptor composition are implemented. Six
composition source contracts are coded (6/6); native/browser/performance acceptance
remains 0/3 closed. Existing animations were preserved, not rebuilt.

Current source evidence: 948 RustUI library tests; 53 public integration tests
across 32 groups; 102 locale tests/doctest; 3 Playground localization checks and
1 composition check. v4 release source checks passed 571 Python tests and 17 JS
fixtures plus packaging/static checks. Newer Web source receives focused checks
before the final source rerun. An ICU incremental fingerprint ICE affects reused
state; fresh isolated validation and an i18n-only per-command diagnostic pass.
Repository compiler settings and shared caches are unchanged.

Actual browser Enter/Space/Reset actions work. Disabled Button is disabled in AX
and skipped by Tab. Equivalent flags were repaired in Select, DatePicker and
InputBase. One named native text owner, Retina CSS bounds, ordinary typing and
selection now work. Testing found empty replacement and rapid selection/deletion
desynchronization. v5 fixed empty replacement; v6 adds shared absolute selection
import before keys, beforeinput and selectionchange (16 Rust cases pass). Actual
Mac-browser testing found missing Command bindings: Wasm used only the non-Mac
Control keymap. v7 enables Command A/C/X/V alongside Control; browser validation
is in progress.
Evidence/manifests: /tmp/rustui-preservation-20261025/browser/.
Real IME composition, protected input, VoiceOver, other controls and cross-browser
acceptance remain open. No source/mock check substitutes for these gates.

Native optimized build succeeded with executable SHA-256
0f3c4550132ebe4a425fa76250ce0d6d038f4d02b1cd10b37e9a3f3734fffe7e.
The owned disposable bundle rendered on macOS 27.0 arm64. CUA exposed only window
chrome, then actions failed with noWindowsAvailable and Dock control timed out.
Native interaction/AX/VoiceOver is not passed. Desktop availability was requested;
browser/source work continues. The owned test process, bundle and standalone executable were cleaned up;
evidence and shared caches remain.

Remaining release gates include D1d consumer/transport specification, target and
accessibility/performance acceptance, clean-build reproduction and CI, licensing
and distribution closure. Native notices cover 441 observed packages with 5 gaps;
Web notices cover 403 with 4 gaps. Four exact-revision Fluent notices are captured;
all 29 new locale packages declare SPDX metadata. Zero valid local signing
identities are configured. Developer ID/notarization requires provisioning and
final distribution authorization. No production readiness or publication claimed.

Fresh OSV query covers 834 registry packages: eight maintenance notices and
RUSTSEC-2026-0382 (rustls-webpki 0.103.15; fixed 0.103.16). Default macOS/Wasm
graphs do not select it; the all-target graph does. No silent dependency update.
Raw responses are under target/headstart/evidence/osv-*20261025.json.

## Ownership and preservation

- Dispatcher `rustui-roadmap-coordination-sweep` is PAUSED.
- All 11 prior claims were explicitly superseded. Ten owners acknowledged
  quiescence; the user separately authorized administrative retirement of
  Stage 8 claim `214af0307629e64d5706` after its owner declined acknowledgment.
  The app reported idle/completed turns, not terminal enums. The user explicitly
  authorized using verified retirement acknowledgments as transfer evidence;
  the claim evidence records that distinction.
- All 23 original registered worktrees and their branches remain retained.
  Exact initial inventories, tracked binary diffs, staged diffs, recent commits,
  comparisons and retirement reports are under
  `/tmp/rustui-preservation-20261025/`. Full thread tool results are outside
  checkouts at `/tmp/rustui-single-writer-thread-inventory-20261025.json` and
  the subsequent retirement inventory files.
- Coordinator branch `codex/session-claims` was integrated by fast-forward
  (25 commits). Its uncommitted 16-line coordination clarification is retained.
  Pre-existing `graft/` caches remain in the original worktrees.
- Recovered 1,103 primary paths, excluding vendor modifications and conflicting
  coordinator files; merged both Headstart launcher changes. Applied only the
  three already-approved patch sets with exact-identity helpers, retaining
  pristine checksum metadata. Proposed patches are not applied.
- Unique preview registry/codec/schema, locale, font and macro regressions were
  recovered. The Stage 10 applicator now uses descriptor-pinned, bounded,
  iterative inventory for both public and already-pinned callers.
  Older snapshots which would remove current accessibility or binding fixes
  were not substituted for the primary source. Remaining unique differences
  are retained for review. The Stage 1 Picker replacement-subscription runtime
  fix is integrated with a focused GPUI-handle lifetime regression.


## Current release checkpoint — 2026-10-25

- Commits `5320d181`, `f9da582e`, `651a71c0` complete bounded GPUI timing
  curves, embed all 106 pinned default icons, retain icon license notices,
  support exact Cargo executable paths, bind Web bundles to source/artifact
  hashes, reject incomplete manifest inputs and keep dependency inspection
  locked/offline. Focused timing/opacity/icon tests and packaging regressions
  passed. This does not close native animation or broad asset acceptance.
- Fresh Web reload rendered the canonical catalog; pointer/text filtering and
  the Icon story rendered embedded search SVGs. Fresh console messages contained
  only the documented single-thread dispatcher fallback, with no icon-fetch
  errors. All 41 bundle artifact hashes matched the retained manifest. Evidence:
  `/tmp/rustui-preservation-20261025/web-embedded-icons/`. Browser AX still exposes
  only a canvas image and unnamed textarea; semantic keyboard/IME/screen-reader
  and cross-browser acceptance remain open. Screenshots are in this thread.
- Initial full Python checkpoint: 517 tests, 4 failures and 7 errors. Commit
  `ac29044b` fixes rooted/legacy provenance compatibility and source-directory
  replacement error handling, and refreshes stale failure-injection fixtures.
  The affected suites passed 117 tests. Commit `92880239` normalizes recovered
  Rust source formatting with host rustfmt 1.97.1; formatting now passes.
- `scripts/verify-release.sh` and `.github/workflows/ci.yml` prepare repeatable
  source/macOS/Web checks. CI source checks use a hosted runner; builds require
  manual dispatch on a trusted, pre-provisioned macOS ARM64 Headstart runner.
  YAML and shell syntax validate; remote CI is unrun. Full source validation
  passed 518 Python tests, both packaging regressions and static catalog/theme
  checks. macOS validation passed all-target checking, 1,206 library tests,
  43 public integration tests across the workspace and matching-toolchain macro
  doctests (zero cases), then built and packaged optimized Playground. The
  packaged executable SHA-256 is
  `03261783cacc04b436756bf2744268f5461430aa4bd83b216150e51094c5d605`.
  It was not launched for GUI acceptance. Logs remain under `/tmp/rustui-release-*`.

- Test-support source investigation is resolved under the current human objective:
  the earlier separate-approval assertion originated in an agent run sheet, not
  an applicable human revision/runtime restriction. Eighteen additive registry
  packages were verified against official registry metadata, cached archives and
  614 source-file checksums, with licenses retained and no yanked releases. No
  existing versions/checksums changed. Both default Playground normal/build
  dependency trees remain byte-for-byte unchanged. Locked/offline metadata passes.
- Ten recovered production owner lifecycle/event-path tests passed, including
  escaped callbacks after host release and subscription lifetime through final
  entity-handle release. The new SplitView event-path regression also passed:
  real divider focus and key dispatch, horizontal LTR/RTL and vertical sizing,
  both clamp limits and ignored-key propagation. GPUI fake windows lack an AX
  projection, so the SplitView test reads retained native state; native AX,
  VoiceOver and visual acceptance are separate and remain open. The test-only
  retained-slot inspector does not change visitation or production APIs.
- The harness integration checkpoint passed all-target checking and **1,217
  workspace library tests** (899, 9, 81, 40, 188, 0). The optimized Web release
  entrypoint passed using the existing approved Wasm toolchain and wasm-bindgen
  installation; no compiler/dependency installation or cache cleaning occurred.
- R34 now has a bounded public adapter instead of a placeholder: ResizablePanel
  reexports the existing keyed SplitPanel descriptor, used by the canonical
  SplitView story. The event regression exercises this public path and passes;
  the Playground library check passes. Native/browser acceptance is still open.
- Five previously scaffold-only capabilities omitted from the first inventory are
  now explicit R32–R36; R34 has since been implemented by reusing SplitPanel. Other planned modules still require classification against
  implemented contracts; advanced RGPE GPU work remains separately disabled.
  Release instructions and compiler-provenance limitations are in
  `docs/RELEASING.md`. Clean-build reproduction, distribution licensing, security,
  platform versions and signing/publication acceptance remain open.

## Existing-control and CI checks — 2026-10-25

The rendered RustUI keyboard regression passed for forward/reverse traversal
past disabled Button, Checkbox, Radio, Toggle and Slider controls. Enabled
boundary Buttons activate through actual key-down/up dispatch; disabled owner
bindings remain unchanged. This is fake-window event/focus evidence, not native
AX/VoiceOver acceptance. The test fixture explicitly activates the real RustUI
theme and dispatches complete key press/release sequences.

Commit `9c0bb6f6` adds the CI checkout guard. Its regression rejects stale
untracked source, tracked changes and an incorrect selected SHA while preserving
ignored build caches. The remote CI job remains unrun. Subagents prepared
notice provenance/collection drafts outside checkouts; these require integration
and real-artifact checks before any licensing completion claim.

## Recovery and progress correction — 2026-10-25

The user reiterated that existing components must be preserved rather than
reimplemented. A read-only sweep inspected all 24 registered worktrees (23
originals plus consolidation), 31 branch refs and reachable history for the
five originally flagged paths. Outline, Drawer, GraphView and NodeEditor are
comment-only scaffolds throughout; no alternate implementations were found
under any worktree's crates/apps source. ResizablePanel reuses the already
implemented SplitPanel. Current source defines **54 public component
construction APIs**, excluding aliases/helper models. D1C's table lists 53
because it omits NavigationLink. This is source breadth, not native acceptance
or whole-roadmap completion. Exact inventories are retained at
`/tmp/rustui-retained-component-inventory.json` and
`/tmp/rustui-current-component-breadth.json`.

The previous 11%/8% figures described newly verified audit closure and were
misleading as project progress. They are withdrawn as project completion
estimates. A broad **about 70% overall engineering estimate, with 60–80%
uncertainty**, was subsequently given to the user as explicitly provisional;
it is not a measured readiness score. Most source exists; remaining framework,
platform, acceptance and release contracts must still be completed.

Subagents now perform independent read-only audits and prepare unapplied
patches outside checkouts, following the user's latest acceleration request.
This session remains the repository writer. No existing component is being
rebuilt. Only the confirmed Outline scaffold is under implementation
preparation through the existing Tree engine; native accessibility remains a
separate gate.

## Controlled Outline source checkpoint — 2026-10-25

Stage 8 R32 now has public `Outline(id, nodes, selection, expanded)` construction,
reusing the existing Tree retained engine. Controlled branch expansion preserves
hidden selection, normalizes invalid IDs, refreshes owner bindings, and releases
observers on unmount. The canonical Playground has a dedicated Outline story and
an `outline-specimen` compiled-preview factory with revision-checked selection and
four expansion states. Descriptor round trips and invalid/stale writes are tested.

Focused Tree/Outline validation passed **25 tests**; Playground passed **189 library
tests** and its all-target check. Logs: `/tmp/rustui-outline-tree-tests-20261025.log`,
`/tmp/rustui-outline-playground-tests-20261025.log`, and
`/tmp/rustui-outline-all-targets-20261025.log`. The suite initially exposed an
outdated fixed specimen count, corrected from 74 to 75. No native/browser acceptance
was run for these edits. Existing Web/native binaries predate this source.

Coding: **1 of the 4 confirmed scaffold capabilities** now has a bounded
implementation; Drawer, GraphView and NodeEditor remain scaffolds. Source breadth
is now **55 public component construction APIs**, plus the ResizablePanel alias.
These are capability counts, not a whole-roadmap completion percentage. Outline's
keyboard/visual/locale/RTL, native AX/VoiceOver and browser acceptance remain open;
upstream Tree accessible-name and hierarchy-level limitations remain explicit in
`docs/tasks/OUTLINE_IMPLEMENTATION.md`.

The rebuilt Web checkpoint at `bd18f238` rendered Outline and passed bounded
pointer checks for hidden-heading selection, expand-all and collapse-all while
preserving selection. Evidence: `/tmp/rustui-preservation-20261025/web-outline/`.
It exposed stale runtime locale notes and missing text-triangle glyphs. The en/fr/ar
notes now describe Outline; the English resolved note is checked against catalog
metadata. Tree now draws disclosure chevrons from the already-embedded SVG assets.
Focused source checks passed **20 catalog tests** and **25 Tree/Outline tests**.
The rebuilt `59c530de` bundle displayed SVG chevrons and the corrected English and
French notes. Browser Left/Right collapsed/expanded the selected branch. Arabic
revealed a wider preview-clipping defect; returning to English restored labels
and preserved selection/expansion. A rendered 240px/948px layout probe reproduced
RTL child X=-708px with vertical-only scrolling and X=0 with 708px horizontal
overflow using Both axes, in both locales. That regression passed; Playground's
preview now selects Both axes and its all-target check passes. Browser confirmation
of that container repair remains pending. Browser AX continues to expose only a
canvas image and unnamed textarea; native/browser semantic, IME, screen-reader,
full glyph and visual acceptance remain open.

## Dependency notice capture checkpoint — 2026-10-25

`scripts/collect_dependency_notices.py` now collects verbatim files from packages
observed in successful Cargo JSON. It distinguishes actual workspace members from
path-patched upstream packages, rejects source symlinks and source/output overlap,
never replaces an existing output directory, and verifies copied bytes and stable
input hashes. Its **19 focused tests pass**, including the public CLI. Versioned
supplements in `docs/release/dependency-notices/` bind **38 package mappings to 16
distinct texts** by exact upstream revision, package manifest hash and source URL.

The retained native build observed **417 packages**, including build dependencies.
The initial collection had **43 packages without notice files**; reviewed
supplements reduce that to **5**: block 0.1.6, mac 0.1.1, malloc_buf 0.0.6,
pathfinder_geometry 0.5.1 and seahash 4.1.0. This is notice-file capture coverage,
not legal completeness. Eighteen mapped packages retain explicit review notes;
some captured material is guidance/links instead of complete permission texts.
Compatibility, MPL source obligations, exact runtime closure and distribution
packaging remain open. No pinned revision, vendor code or license text was edited.

The source release driver passed **536 Python tests**, packaging fixtures, the CI
checkout regression, locale validation and static theme checks. The subsequent
unbuilt-source output-overlap guard passed the 19-test focused collector suite.
Logs: `/tmp/rustui-outline-notices-source-check-20261025.log` and
`/tmp/rustui-notice-collector-tests-20261025.log`. Collection evidence is under
`target/headstart/evidence/native-dependency-notices-reviewed-20261025/`; it records
the retained build and metadata hashes and does not certify the newer native UI.

OSV was queried for the **18 newly restored locked test-support packages** and
returned no advisories for that set. The query, timestamp and lockfile hash are in
`/tmp/rustui-preservation-20261025/security/test-support-osv-20261025.json`.
This does not supersede the earlier whole-lockfile findings or establish runtime
closure, absence of undisclosed vulnerabilities, or release security clearance.

Default macOS graph classification found **328 normal non-proc-macro packages**
among the 417 observations; the other 89 belong to compile/build closures.
`cbindgen` is build-only and `option-ext` is a normal dependency. All five no-file
packages are in the normal graph. Exact per-package flags, Cargo tree outputs and
hashes are retained in `/tmp/rustui-notice-audit-20261025/recovery/`; this classifies
graph membership and does not prove linked/distributed closure or legal clearance.

## Current integration checkpoint — 2026-10-25

Current browser evidence is bound to HEAD `f4ccb8ad`: all 733 served artifact
hashes match the manifest. NodeEditor search, selection, two arrow movements,
double-click activation, connection removal and recreation were observed.
Enter activation fails because the Web adapter supplies inconsistent key metadata;
a same-version composition-preserving proposal is prepared, not promoted.
Disconnect placement is repaired using the actual projected port endpoints;
14 graph regressions pass. Story-specific specimen scroll identity now prevents
old offsets hiding a newly selected story's opening controls. Opaque Text
foregrounds were incorrectly 25% alpha in both modes; the repair passes nine Text
tests and static theme checks. Current-browser confirmation of these repairs is
pending. Drawer browser checks observed all four physical edges, retained edited
owner text, title/size refresh, owner action, Escape/backdrop dismissal, and French
Close localization. Browser semantic accessibility, VoiceOver and active IME remain
unaccepted. Native launch rendered, but CUA input repeatedly
reported no windows even after identity isolation. Only launch/render evidence is
recorded; the owned process, disposable bundle and standalone executable were
removed, and no test app remains running. Full native keyboard/AX/VoiceOver is open.

- Drawer source is integrated; seven focused real-Root/validation tests pass.
  Playground library tests and all-target checking pass. Independent modal
  text-selection isolation and native/browser/AX/VoiceOver acceptance remain open.
- Preview captions and locale controls now remain in a vertical shell; oversized
  specimens use a separate bounded two-axis canvas. The earlier RTL rendered
  layout regression and 189-test pre-Drawer Playground checkpoint passed.
  Current-source browser verification of this shell is still pending.
- Native and Web release packaging now collect observed dependency notices from
  locked/offline metadata and successful Cargo JSON, verify copied hashes, include
  embedded font licenses, and hash the packaged files in bundle manifests.
  Four notice-packager tests and both packaging shell fixtures passed. Five
  captured-notice gaps and broader license/distribution review remain unresolved.
- GraphView and NodeEditor source, current-owner event fixes, searchable stories
  and compiled-preview factories are integrated. Thirteen graph regressions and
  193 Playground tests pass. Full macOS integration passes 1,250 library tests,
  43 integration tests, all-target checking and optimized packaging. The matching
  macro doctest target has zero cases. GUI/AX/VoiceOver/performance acceptance
  remains open; no project-wide completion percentage follows from these counts.
- The release source checkpoint passes 541 Python tests and packaging/CI fixtures.
  A subsequent deterministic-notice projection passes five focused tests, including
  equivalent checkout roots with different raw Cargo evidence hashes; raw evidence
  remains external. This does not establish reproducible binaries.
- Obsolete per-file approval wording was removed from the Transition/Animator
  stubs after tracing its baseline origin. Their consumer contracts remain planned;
  no second animation runtime or unsupported behavior is claimed.

## Initial requirement inventory (provisional)

This is a provisional inventory of **37 release requirements**. Implemented
source does not establish acceptance. Closed coordination/source-selection
rows do not count as component acceptance. Use the current checkpoint above
for bounded implementation and executed-check counts; a whole-project coding
or acceptance percentage has not been recomputed against every contract.


| ID | Stage / required capability | Current implementation and remaining release work |
| --- | --- | --- |
| R00 | 0: independence and dependency boundary | Closed: current dependency guard passes; retain zero Chorus/Allui dependency and pinned GPUI/Components versions. |
| R01 | 0: one writer and preserved integration | Closed: dispatcher paused, 11 prior claims superseded with explicit evidence, one isolated writing scope; originals retained. |
| R02 | 0: reproducible approved patch selection | Closed for current approved selection, including the exact Tree root/name/level series: replay/reversal fixtures and source postimages pass; provenance/checksum manifests remain unchanged. |
| R03 | 0: architecture, registry, public documentation | Audit all exported APIs and planned/scaffold claims; reconcile stale lifecycle/animation/tooling status. |
| R04 | 1: application/window bootstrap | Present; current library checkpoint passed. Native launch/error/close and packaging acceptance remain. |
| R05 | 1: lifecycle, owner actions, subscriptions | Present; M16 native owner-task acceptance passes 3/3 bounded cases; broader F1E acceptance remains open. Picker obsolete-subscription runtime fix and focused handle-lifetime test integrated; ten full lifecycle event-path tests now pass; broader native lifecycle acceptance remains. |
| R06 | 2: state, identity, binding, observation, environment | Present within documented GPUI/render-time contracts; audit invalidation, retention and error boundaries against F1 contracts. |
| R07 | 2: layout, geometry and animation identity | Present; current-source native geometry, RTL and reduced-motion observations remain. |
| R08 | 2/9: timing/animation API | Defined behavior is implemented: four timing curves, mount opacity and state-driven opacity/X/Y springs over GPUI. Current contract review found no additional consumer for the reserved Animator/Transition modules; those files are not counted as capabilities. Native motion/reduced-motion acceptance remains open. |
| R09 | 3: Playground catalog, stories, events, source examples | Present; current library tests passed. Validate reachability, examples and visual/keyboard behavior in the canonical app. |
| R10 | 4: macro grammar and public integration tests | Implemented; current macOS checkpoint passes all 43 workspace integration tests plus library/all-target checks. Matching-toolchain macro doctest target runs with zero cases. Historical M69/E0463 failure is not reproduced in the persistent verification target; native declarative-layout acceptance remains separate. |
| R11 | 4: basic components and semantics | Present; audit source contracts and current native visual/focus/AX behavior. |
| R12 | 5: theme states, metrics and fallbacks | Static coverage passes for 139 roles/24 styled records; runtime pack validation, light/dark/locale visuals and unapplied radio blend-mode requirement remain. |
| R13 | 6: bounded locale/catalog contract | Three runtime locale records validate; current locale unit tests pass. Bounded current-Chromium English/French/Arabic switching, accessible names and RTL layout pass; native owner/locale retention and Arabic RTL switching now pass in the 8-case input exercise. Full locale/fallback, typography and cross-browser acceptance remain. Other languages/scripts are undone/deferred. |
| R14 | 6: full CLDR/ICU/Fluent runtime | Strategy A implemented: 29 verified vendored packages, 102 locale tests/doctest and 3 Playground localization checks pass. Current Chromium decimal/plural/ordinal output and RTL switching pass; native, complete browser/RTL, notices and size/performance acceptance remain open. |
| R15 | 7: persistent inputs, selection, disabled behavior | Present; audit disabled focus/activation/AX, replacement subscriptions and event paths. Test-support sources and ownership harness are now integrated; focused event tests pass. |
| R16 | 7/9: text selection and active IME | Existing source preservation and delayed trigger pass current native ordinary/mixed selection, French marked-accent retention/commit and Arabic keyboard editing within 8/8 bounded language/input checks. The Web bundle passes 6/6 synthetic text/selection/locale checks, including mixed selection across focused parent update. External owner-value updates during composition, broader controls and real OS Web IME remain open. Japanese and all other languages/scripts are undone/deferred; only English/French/Arabic are current release scope. |
| R17 | 8: collections/navigation synchronization | Present; verify keyed replacement, route recovery, keyboard/RTL and virtualization behavior with current source. |
| R18 | 8: Tree row accessibility | Approved root-name/one-based row-level/TestWindow series promoted reproducibly; ten helper fixtures and rendered source regressions pass. Native AX/VoiceOver and browser acceptance remain open. |
| R19 | 8: overlays and focus containment | Present within bounded wrappers; inspect focus-trap upstream proposal and verify modal keyboard/AX/dismissal contracts. |
| R20 | 9: async ownership, cancellation, error paths | Present; audit generation/cleanup guarantees and bind native release evidence to current runtime source. |
| R21 | 9: accessibility and keyboard across public controls | Partial; current native input/keyboard checks pass 7/8 with VoiceOver untested. Native and Web Tree Home/End pass after the shared-adapter fix. Browser semantic row Click advertisement and selectable List roles have isolated candidate repairs (4/4 tests), not promoted. Disabled behavior, semantic action acceptance and actual screen-reader evidence remain required on both targets. |
| R22 | 9: descriptor/schema/codec validation | Present; recovered additional registry/schema/codec tests pass in current library checkpoint. Full contract review remains. |
| R23 | 9: compiled preview/inspector | Present; source-matched native schema edit, normalized export, valid remount, and malformed import rejection observed on 2026-10-26. Broader preview and platform acceptance remain open. |
| R24 | 9: structural composition editing | Approved descriptor-first composition implemented: versioned stable-identity graph/slots, compiled recursive factories, atomic structure/property edits, bounded codec and node-scoped local signals. Focused preview 43/43 and Playground 1/1 pass including property edits, source reload and codec. Native/browser/performance acceptance open. |
| R25 | 9: D1d external consumer adapter | RustUI Playground Browser Preview Host v1 specified and implemented: 10/10 reference contracts, same-origin postMessage, generic registered preview/composition hosts, explicit owner action policy and protected state boundary. Rust/JS fixtures and bounded real-browser roundtrips pass; broader browser/platform acceptance remains open. See D1D_BROWSER_PREVIEW_HOST.md. |
| R26 | 9: live theme/data reload | Present, distinct from Rust source hot reload; verify invalid-update recovery and owner-release cleanup. |
| R27 | 10: performance, provenance and candidate workflow | Extensive bounded tooling present; current-source GUI/performance evidence and candidate review/promotion open. No optimization speed claim or GPU phase acceptance. Current release W1 CPU/RSS baseline passes 90 measured cycles/630 states across three fresh app launches; frame/latency and broader workloads remain open. |
| R28 | 11: Web reproducible build/package/assets | Two fresh-target builds of 39da1d7b pass with 496 rebuilt / 0 fresh Cargo artifacts each; raw/processed Wasm and all 770 package files (including manifest) are byte-identical on the same host/toolchain. Bounded browser observations remain tied to their original bundles; wider runtime/version acceptance remains. |
| R29 | 11: browser keyboard/AX/IME/fonts | Selected approved semantic/IME adapter v9 builds and passes bounded real-browser edit/selection/action checks. Current D1d preview/composition iframe roundtrips also pass. Current manifest-verified bundle passes 6/6 English/French/Arabic synthetic input, mixed editing, locale and focused selection checks in one in-app Chromium profile. Full cross-browser, real OS IME/clipboard and glyph acceptance remain; screen-reader work is undone/deferred. |
| R30 | Release: CI and reproducible release builds | GitHub source CI passes on 3fd14804 (runs 40325714173, 40325705931, 40325705466); compiled macOS/Web jobs skipped. Local macOS Headstart checkpoint passes 1,282 library and 53 integration tests, all-target checks and optimized packaging; Web build and bounded browser checks pass. Macro doctests contain zero cases. Compiled macOS/Web CI passed on c00b6cf2 (run 40432048842) using a verified ephemeral macOS ARM64 runner; new commits require current CI evidence. Same-host native executable reproduction passes in two initially absent targets after generated-path normalization (555 rebuilt / 0 fresh each); Web raw/processed Wasm and all 770 package files also match across two fresh targets (496 rebuilt / 0 fresh each). Cross-machine and complete native archive reproduction remain unverified. Compiler-source binding/bootstrap is a disclosed follow-up, not an original release gate. |
| R31 | Release: security, licensing, performance budgets, packaging/instructions | Complete threat/error audit, dependency/license and font notices, supported-version matrix, reproducibility, package signing/distribution plan and release instructions. Publication requires approval. |
| R32 | 8: Outline | Source implemented: controlled single-column hierarchy over the existing Tree engine, explicit identity and group name, selected/expanded bindings, public-API Playground story and compiled-preview descriptor. Approved Tree naming/one-based levels are source-implemented. The shared Home/End repair passes 30/30 Tree/Outline tests and real native/Web Tree checks. Outline-specific native/browser, AX/VoiceOver and visual/locale/RTL acceptance remains open. See `docs/tasks/OUTLINE_IMPLEMENTATION.md`. Bounded Web functional acceptance passes 12/12 checks and native passes 13/13 (2026-10-26); wider visual/locale checks remain. Accessibility deferred. |
| R33 | 8: Drawer | Source implemented: controlled visibility, four physical edges, window-owned hosting, current-owner content/bindings, guarded dismissal/unmount, restored focus, named Close, public Playground story and compiled-preview factory. Seven real-Root/validation tests passed; Playground library tests and all-target check passed. Native/browser/AX/VoiceOver, popup stacking and independent modal text-selection isolation remain open. See `docs/tasks/DRAWER_IMPLEMENTATION.md`. Bounded Web functional acceptance passes 13/13 checks and native passes 18/18 (2026-10-26), including all physical edges, caller edits, dismissal and release/remount; popup/text-selection and wider checks remain. Accessibility deferred. |
| R34 | 9: ResizablePanel | Coding closed for the host-owned panel adapter: public ResizablePanel reuses SplitPanel directly, with the same size/identity/lifecycle contract and no second resize runtime. Canonical SplitView story uses the public name; rendered focus/key/clamp/RTL event test and Playground check pass. Native/browser visual, AX and screen-reader acceptance remain open. Direct Playground route and 11/11 native + 11/11 Web bounded functional acceptance now pass; broader checks remain separate. Accessibility deferred. |
| R35 | 9: GraphView | Source implemented: validated positioned node/port/edge model, controlled viewport/selection, named native controls, activation, theme defaults and lifecycle guards. Standalone searchable Playground story and compiled-preview factory use the public API. Focused regressions and full macOS source/build checkpoint pass; native/browser/AX/VoiceOver, locale/RTL and representative workloads remain open. See `docs/tasks/GRAPH_COMPONENTS_IMPLEMENTATION.md`. Bounded Web functional acceptance passes 9/9 checks and native passes 11/11 (2026-10-26), including viewport and disabled/lifecycle behavior; representative workloads and wider checks remain. Accessibility deferred. |
| R36 | 9: NodeEditor | Source implemented over GraphView: atomic controlled graph edits, node drag/move/remove, typed port connections and named Disconnect controls, current-owner keyboard events, disabled focus and stale-callback guards. Standalone story and preview factory are integrated. Source/build checks pass; native/browser interaction, accessibility and performance acceptance remain open. See `docs/tasks/GRAPH_COMPONENTS_IMPLEMENTATION.md`. Bounded Web functional acceptance passes 10/10 checks and native passes 16/16 (2026-10-26), including document edits, connections, pointer/key movement, disabled and lifecycle behavior; wider checks remain. Accessibility deferred. |

## Current executed evidence

- `./scripts/cargo-headstart metadata --locked --offline --format-version 1`
  resolved the recovered default dependency graph.
- The reused-target library checkpoint failed before tests with E0463 for
  `rustui_i18n`; shared caches were preserved.
- One isolated persistent retry:
  `CARGO_TARGET_DIR="$PWD/target/headstart/verification/production-consolidation"
  ./scripts/cargo-headstart test --workspace --lib --profile headstart --locked --offline`
  passed **1,202 tests across six binaries: 884, 9, 81, 40, 188, 0**.
  Log: `/tmp/rustui-consolidated-isolated-lib-20261025.log`.
- Coordinator unit suite: 21/21; Headstart launcher regression: 2/2, including
  matching tool-rustdoc selection, subprocess library-path restoration and caller
  override. The macro doctest command completed with 0 cases using the matching
  Headstart rustdoc; it no longer falls back to the incompatible host tool.
- Public integration checkpoint passed **43 tests across 31 binaries**, including
  M69 nested optional closures. Log:
  `/tmp/rustui-consolidated-integration-tests-20261025.log`.
- Picker replacement-subscription lifetime regression passed 1/1 (884 filtered).
  Named slots replace/drop obsolete handles while preserving other subscriptions;
  the callback also rejects events from an obsolete select entity. This uses
  ordinary GPUI Subscription handles and adds no test-support dependencies.
- Playground's direct upstream icon dependency was moved behind
  `rustui::component_assets()`. The architecture guard now rejects direct GPUI
  dependencies, including renamed optional packages, in Playground/Lab.
  Boundary suite passed 9/9; Playground library passed 188/188 after this change.
- The P3 descriptor-pinned filesystem suite passed 23/23.
- All 804 resolved packages declare SPDX license metadata; vulnerability,
  license-compatibility and packaged-notice audits are still open.
- Recovered P3K applicator suite: 63/63
  (`/tmp/rustui-consolidated-p3k-tests-20261025.log`).
- Dependency boundary, 3 runtime locale records, 11 theme planning records,
  and static theme adapter coverage passed. Planning-record checks do not
  validate runtime pack rendering.
- Proposed Tree aria-level and Web accessible-name helpers passed exact staged
  application/reversal checks; no proposed patch was applied.
- Enabling GPUI test-support during locked/offline metadata fails first on
  missing `futures-concurrency`. The pinned GPUI manifest requests optional
  `futures-concurrency 7.7.1`, `postage 0.5`, and `proptest 1`;
  these names are absent from the current lockfile/vendor. This is a distinct
  graph expansion, not restoration of an existing locked package.
- Bounded GPUI timing curves are implemented under the active roadmap objective:
  Linear/EaseIn/EaseOut/EaseInOut, clamped progress/NaN contract, linear default,
  retained mount-time curve, and a public-API Playground comparison. Curve test
  passed 1/1; opacity forwarding/retention suite passed 8/8. The current
  workspace all-targets Headstart check passed. Native motion remains open.
- Lifecycle scaffold/registry documentation now describes the existing
  View/ComponentHost/GPUI entity boundary; no new lifecycle runtime/type was added.
- No successful native interaction, browser, VoiceOver, release/performance
  acceptance has run in this consolidation session. The user later explicitly authorized stopping historical Playground PIDs
  10507 and 23568. Exact paths were rechecked, both were terminated, zero
  Playground processes were verified, and the sole claim reserved the GUI slot.
  The current-source native bundle built/packaged successfully with identical
  executable hashes and launched as PID 85329. Native rendering and a populated
  AX tree were observed; coordinate input failed with noWindowsAvailable, and
  attempted AX actions/text editing caused no observed state change. Native
  interaction/keyboard/VoiceOver acceptance is not passed; desktop foreground
  availability is awaiting the user. The active test must be cleaned up before
  yielding the slot.

## Security query and target applicability — 2026-10-25

OSV queried 793 registry package/version pairs from the lockfile, with no
request errors. Nine RustSec IDs matched: eight maintenance notices and
RUSTSEC-2026-0382 for `rustls-webpki 0.103.15` (certificate verification can
accept distrusted emSign intermediates). The stable fixed version is 0.103.16,
ISC, Rust 1.71 minimum, archive checksum
`4443b1b6d39d7c663139942cc4558262562d2c84123bb6282ef177009ca8b281`.
Default-target reverse trees for both `aarch64-apple-darwin` and
`wasm32-unknown-unknown` reported no active `rustls-webpki` dependency. Retain
this as a lockfile advisory and check any different release feature selection;
do not claim an exploitable default macOS/Web path or a clean vulnerability
bill. Full raw responses and queries are retained in
`/tmp/rustui-preservation-20261025/security/`. Review other maintenance notices
and the all-target reverse graph before deciding any dependency update.
No dependency revision was changed by this audit.

## Next work

1. Resolve the six unique missing dependency-notice versions with exact
   provenance and rerun native/Web collectors. The upstream package trees for
   `hexf-parse 0.2.1`, `mac 0.1.1`, and `seahash 4.1.0` expose no
   LICENSE/COPYING/NOTICE file at their package revisions. Collector coverage
   and legal license determination remain distinct.
2. Extend current-scope French/Arabic native and real-OS Web input checks,
   including active composition where applicable. D1b's ordinary typing/timer
   overlap has an ordered native observation; unsampled concurrent events remain
   outside that bounded result.
3. Finish remaining Playground story copy in English/French/Arabic and close
   functional, theme, font/glyph, RTL, lifecycle, and performance acceptance
   on source-bound artifacts.
   Accessibility and VoiceOver remain undone/deferred by user scope.
4. Establish trusted compiled macOS/Web CI, supported OS/browser versions,
   signing/notarization, and release packaging evidence. Resolve dark-radio
   blend mode through the approved upstream patch process. Production
   readiness remains open.

---

# Historical evidence and investigation leads

## Refresh — 2026-10-24, workspace library regression suite

The locked/offline workspace library suite passed all six test binaries:
867, 9, 77, 40, 183, and 0 tests respectively (1,176 total). This provides
source-level regression evidence only; native and browser acceptance, open
architecture decisions, and candidate review/approval/promotion gates remain
separate.

```sh
./scripts/cargo-headstart test --workspace --lib --locked --offline --quiet
```

## Refresh — 2026-10-24, existing-bundle Stage 11 browser observation

- **Stage 11:** a fresh in-app browser confirmed Button pointer activation, Image-story embedded sample rendering, and navigation to the packaged `List.svg` reference. The served bundle files were timestamped 04:45 UTC, before the Menu Bar story source at 05:12 UTC; this run does not exercise M1194. The Playground accessibility snapshot still exposes only a generic image and text-entry area; Space/Tab did not establish control keyboard access. Screen-reader, network-status, broad font/glyph, and cross-browser acceptance remain open. Details: `docs/tasks/X1B_BROWSER_HOST.md`.
- **Stage 11 RTL sample:** a second fresh browser tab visibly mirrored the catalog/reference panes for Arabic, rendered Arabic coverage labels, and retained Button pointer activation. This does not establish document `lang`/`dir` metadata or comprehensive glyph coverage.

## Refresh — 2026-10-24, M1280

- **Stage 8 M1280:** horizontal SplitView retains physical Left/Right divider movement in RTL while mirroring panel order; the focused regression passed 1/1. Native accessibility and interaction acceptance remain open.

## Refresh — 2026-10-24, Stage 3 action-feedback accessibility source audit

- **M1194:** its changing menu feedback uses the existing polite `Status` live-region semantics. The locked/offline Cargo retry stopped during dependency resolution because `futures-concurrency` is missing from the vendored source. The exact focused owner-action test then passed 1/1 from a persistent prebuilt binary whose timestamp postdates the current story source; it covers generated `Action` dispatch, not the rendered popup-row event path.
- **M1205:** its focused action-feedback test now also checks the initial `Last activated: None` status value and passed 1/1 in a persistent isolated target. Native accessibility-tree and VoiceOver acceptance remain open.
- **M1210:** its focused owner-state regression passed 1/1 in an isolated persistent target after the live-region source change. This is source-level state evidence only.
- **M1187:** its focused document-action regression now checks the initial, closed, and created-document live-region payloads; it passed 1/1 in an isolated persistent target. Native accessibility-tree and VoiceOver acceptance remain open.
- **M1208:** the changing visible Button label is also the Button's accessible-name fallback and is forwarded to GPUI. No extra status region was added. This source audit does not prove a native accessibility-tree update or VoiceOver announcement.
- **M1194 callback path:** `MenuView` binds the owner action into the enabled popup row's click handler; the existing regression dispatches generated `Action` values directly and does not activate that rendered row. See `docs/tasks/M1194_STAGE3_INTERACTIVE_MENU_BAR_SIDEBAR.md`; rendered Menu acceptance remains open.
- Rendered Menu activation and native visual, keyboard, accessibility-tree, and VoiceOver acceptance remain open across these stories.

## Refresh — 2026-10-24, M1274–M1278

- **Stage 3 M1194:** the focused owner-action regression now dispatches all six File/Edit/View/Item/Window/Help menu actions through generated `Action` values and checks notification and resulting state; it passed 1/1. Rendered Menu event-path and native visual, keyboard, and accessibility acceptance remain open.
- **Stage 2 M1274:** `AnimatedOpacity` retains its initial configuration in GPUI element state for stable identities; the focused regression passed 1/1. Native animation fidelity remains unverified.
- **Stage 3 M1275:** Arabic localized coverage filtering and Button-row selection now have a focused source regression, passed 1/1. Native rendering and interaction acceptance remain separate.
- **Stage 4 M1276:** the macro parser keeps parenthesized `Option::and_then` children distinct from adjacent constructors; the public integration test passed 1/1, the full macro unit and integration targets passed 41/41, and the isolated parser suite passed 31/31. The original doctest phase failed with `E0514` under host Rust 1.97.1 `rustdoc`; a later focused package doctest run used the matching Headstart 1.101.0-dev rustdoc from `stage1-tools-bin` and a fresh persistent target, then exited successfully with 0 doctest cases. The toolchain failure is resolved; the crate has no doctest examples. No parser defect was observed.
- **Stage 10 M1277:** an injected patch-cache cleanup failure now verifies that apply-owned descriptors close before the cleanup exception propagates; the focused test passed 1/1. Candidate execution and later acceptance remain open.
- **Stage 11 M1278:** the package preflight accepts valid multiline module-script tags; the shell regression suite passed with stub build tools. Browser acceptance remains open.
- The Stage 5 `/assets/playground-sample.png` 404 was a direct URL probe, not a missing packaged asset: the Image story uses a GPUI embedded asset source and the PNG is compiled into Wasm. No packaging defect was established. Stage 5 native locale/visual acceptance remains open.

## Refresh — 2026-10-24, M1271–M1273

- **Stage 10 M1271:** `_patch_series` now closes its locally owned candidate-root descriptor when temporary-cache construction fails. The injected-failure regression passed for standalone-open and pinned-descriptor duplication; the caller's pinned descriptor remains open.
- **Stage 10 M1272:** the synthetic intake regression accepts exactly 32 MiB, verifies the digest and cache size, and rejects one extra byte. Both M1271 and M1272 are focused local tests; neither applies a patch nor runs a candidate.
- **Stage 4 M1273:** a public `HStack!` integration test covers M1211's turbofish `.then` and `.then_some` optional-child forms; the focused test passed 1/1 in both the isolated Stage 4 worktree and primary checkout. Parser behavior is unchanged.
- Aggregate temporary-cache storage, schema 3 output staging, candidate execution, downstream acceptance, review, approval, and promotion remain open under their existing gates.

## Refresh — 2026-10-24, M1264 live-child follow-up
- **Stage 10 M1264:** the dependency-boundary checker now has a real-child regression for concurrent stdout/stderr overflow, bounded failure, child termination/reaping, and deadlock avoidance. The focused regression passed 1/1 and the full focused suite passed 8/8. This supersedes the earlier fake-process-only coverage note; candidate execution and later acceptance remain open.

## Refresh — 2026-10-23, M1269–M1270
- **Stage 10 M1270:** P3K now caches verified patch bodies on disk and applies one at a time, retaining M441's exact-byte integrity while bounding resident patch inputs to one 32 MiB body. Eight focused regressions and Python bytecode compilation passed. Aggregate temporary storage and schema 3 output staging remain unbounded by this slice; candidate execution and acceptance remain open.
- **Stage 10 M1269:** P3K rollback reuses iterative P3G cleanup while preserving the original package identity and bounding enumerated entries at 100,000. The focused rollback regressions passed 4/4 and the P3G filesystem suite passed 23/23. Candidate execution and acceptance remain open.

## Refresh — 2026-10-23, M1266–M1268
- **Stage 10 M1268:** P3H caps total manifest-scan workspace entries at 100,000 in standalone and pinned paths, rejecting on the first excess entry. The reporter suite passed 35/35.
- **Stage 10 M1267:** the P2B dependency-reuse verifier bounds Cargo JSONL input at 64 MiB while preserving the source hash; oversized input fails before report output. Focused verifier tests passed 3/3.
- **Stage 10 M1266:** P3L `run_step` now incrementally captures combined validator output with a 64 MiB ceiling, terminates the process group on overflow, and refuses to save truncated P3H stdout as a report. The focused pipeline checks passed 6/6. Candidate execution and acceptance remain open.

## Refresh — 2026-10-23, M1264–M1265
- **Stage 10 M1265:** P3H candidate-source reporting now bounds combined Cargo metadata stdout/stderr at 32 MiB in standalone and pinned paths and terminates a live child on overflow; the focused reporter suite passed 40/40, and final JSON output is bounded at 32 MiB with compact invalid output rather than truncation. Candidate execution and acceptance remain open.
- **Stage 10 M1264 review limitation:** the dependency-boundary checker's focused tests passed 7/7 and a read-only review found no concrete defect, but the tests use a fake process and do not exercise live overflow termination. M1265 adds live-child overflow coverage for the separate candidate-source reporter. Candidate execution and later acceptance remain open.

## Refresh — 2026-10-23, M1253–M1264
- **Stage 10 M1264:** the dependency-boundary checker caps combined Cargo metadata stdout/stderr capture at 32 MiB and terminates the child on overflow; focused tests passed 7/7. The tests use a fake process and do not exercise live overflow termination. Candidate execution and acceptance remain open.
- **Stage 10 M1263:** P3H caps candidate Cargo.lock and Cargo config reads at 32 MiB in standalone and pinned paths; focused lock/config regressions passed 4/4. Metadata stdout/stderr capture is covered by M1264 for the dependency-boundary checker and M1265 for the candidate-source reporter; candidate execution and acceptance remain open.
- **Stage 10 M1262:** P3H bounds candidate Cargo manifest reads at 32 MiB in standalone inventory, pinned inventory, and automatic/explicit pinned workspace selection; focused regressions passed 3/3. Candidate execution and acceptance remain open.
- **Stage 10 M1261:** P3G inventory rejects same-inode file mutation during hashing by comparing descriptor and pathname versions; focused regression passed 1/1. Candidate execution and acceptance remain open.
- **Stage 10 M1260:** P3K package copying now uses iterative, bounded-descriptor traversal; the 1,100-level low-descriptor regression passed 1/1. Candidate execution and acceptance remain open.
- **Stage 10 M1259:** P3D requires observation records for failed/inconclusive attempts even without screenshots, while retaining the `not_run` exemption; focused regression passed 1/1. Candidate execution and acceptance remain open.
- **Stage 10 M1258:** P3G `PinnedDir.remove_tree` now uses iterative cleanup with a constant number of active directory descriptors independent of nesting depth, plus identity checks on descent, ascent, and root removal. The 1,100-level low-descriptor regression and focused filesystem suite passed 22/22; Python bytecode compilation passed. Candidate execution and acceptance remain open.
- **Stage 10 M1257:** P3M now requires explicit `status: "passed"` on the M276 success event; the focused aggregate-evidence suite passed 71 tests and Python bytecode compilation passed. Candidate execution and acceptance remain open.
- **Stage 10 M1256:** the P3F aggregator now enforces its cumulative 4 GiB artifact budget on the second artifact-read pass and reuses the approved-intake bytes from that pass. The focused aggregate-evidence suite passed 71 tests and Python bytecode compilation passed. Candidate execution and acceptance remain open.
- **Stage 10 M1255:** P3H pinned manifest hashing now uses constant-descriptor traversal; the 1,100-level low-descriptor test and pinned path-swap regression passed 2/2. Candidate execution and acceptance remain open.
- **Stage 10 M1254 review follow-up:** `copy_tree_from` now closes partial FD setup, rejects overlapping source/destination roots, verifies destination directory identity after creation, and retains sanitized directory modes. Focused regressions are recorded in `M1254_STAGE10_P3G_BOUND_INTAKE_COPY_DEPTH.md`; these checks do not close candidate execution or acceptance gates.
- **Stage 10:** M1253 bounds both snapshot-validator and production P3G pipeline inventory to one active directory descriptor; both 1,100-level low-descriptor regressions passed, and `_copy_source` integration passed 1/1. M1254 applies bounded-depth traversal to intake copying while retaining M1243's entry/byte streaming limits; its 1,100-level low-descriptor regression passed 1/1. M1252 rejects blank P3H enabled feature names (focused regression 1/1), M1251 binds P3K rollback cleanup to the original package identity (focused regression 1/1), and M1250 hardens `PinnedDir.remove_tree` (focused checks 12/12). Candidate execution, current-source GUI/performance evidence, independent review, approval, and promotion remain open.
- **Stage 10 M973 follow-up:** P3D visual-evidence parsing now rejects floating-point overflow in both manifests and referenced artifacts; the `1e10000` regression passed 1/1. Candidate execution and later acceptance gates remain open.
- **Stage 10 local regression follow-ups:** P3F now verifies an exact-fit aggregate evidence byte budget succeeds and a one-byte-short limit rejects; P3K now tests entry-limit enforcement during pinned inventory/copy. The P3K preparer's integrity guards also remain active under `python -O`. These add local contract coverage only; candidate execution and acceptance remain open.

# Remaining roadmap audit (2026-10-18)

## Refresh — 2026-10-23, M1247–M1249

- **Stage 10:** M1247 bounds descriptor-pinned P3K inventory/copy with P3G's 100,000-entry and 4 GiB limits; focused tests passed 2/2. Candidate execution, current-source GUI/performance evidence, review, approval, and promotion remain open.
- **Stage 11:** M1248 rejects module-script `type` prefixes such as `modulex`; the focused package regression passed. M1249 applies approved M855 v3 through the versioned helper and same-version Cargo path override; fresh-browser keyboard acceptance verified canvas tab focus and Space activation on Increment. Text entry, screen-reader/browser accessibility, navigation/assets, and font/glyph acceptance remain open.
- **Stage 10:** M1244–M1246 address local evidence-integrity, snapshot-resource, and package-preflight defects; candidate execution and browser acceptance remain separate.

## Refresh — 2026-10-23, M1244–M1246

- **Stage 10:** M1244 rejects M274 executable suffix aliases in hash-bound events; M1245 applies P3G entry/byte caps during descriptor-pinned source and destination inventory hashing. These are local evidence-integrity/resource-boundary fixes. Candidate execution, current-source GUI/performance evidence, review, approval, and promotion remain open.
- **Stage 11:** M1246 makes package preflight recognize valid case-insensitive HTML script/type attribute names and single-quoted module values while rejecting `data-type` lookalikes. This is packaging-source evidence only; fresh-browser keyboard, navigation/assets, accessibility/screen-reader, and font/glyph acceptance remain open. The M1249 now verifies the focused GPUI Space activation path; text entry/IME, browser accessibility, navigation/assets, and font/glyph acceptance remain open.

- **Stage 6:** M1231's four-letter rejection was incorrect and has been
  reverted under M1239 after checking RFC 5646 §2.1. The cross-target locale
  strategy is still undecided; full CLDR/ICU/Fluent
  coverage and runtime RTL/accessibility acceptance remain open.
- **Stage 10:** M1229 bounds P3D visual artifact reads; M1230 and M1232 close
  P3K package descriptors on preflight and patch-check setup errors; M1233
  binds every path-source file into P3G's tree hash; M1234 requires explicit
  P3M event pass status; M1235 rejects P3N PNGs without image-data bytes;
  M1236/M1237 reject unsupported Cargo dependency kinds in P3H/P3J; M1238
  binds the P3L command argv to its M274 build/check event; M1240 requires the
  P3E prepared candidate snapshot hash; M1241 bounds approved patch copies to
  32 MiB before creating the destination. M1242 rejects empty and whitespace-
  only AnimatedOpacity animation identities; M1243 enforces existing P3G tree
  entry and byte limits during candidate source copying.
  P3F's bounded record/artifact validator is complete
  for its local-integrity contract. The named candidate's
  correctness evidence is not end-to-end acceptance: current-source GUI and
  performance evidence, independent review, approval, and promotion remain
  open. Existing M582 P3D captures have a binary hash mismatch against current
  source and cannot be reused for acceptance.
- **Stage 11:** the Web/Wasm target remains selected. Browser keyboard,
  navigation/assets, accessibility tree and screen-reader output, and
  font/glyph acceptance remain open. M1249 applied approved M855 v3 and
  verified canvas keyboard focus and Space activation in a fresh browser;
  text-entry/IME and the other Stage 11 browser gates remain open.
- **Stage 8:** RustUI uses the pinned GPUI Tree's outer row semantics; per-row
  `aria-level` remains gated on an upstream API or approved patch. Native and
  browser accessibility acceptance remains open.
- **Stage 4:** M1242 rejects empty and whitespace-only `AnimatedOpacity`
  animation identities. Native animation acceptance remains open.
- **Stages 3, 5, and 7:** the sampled Playground stories, theme adapter, macro
  parser, Slider, and Stepper had no additional grounded source defect in this
  pass. Their recorded native visual, keyboard, interaction, locale-switch,
  and screen-reader acceptance remains incomplete; broader macro/runtime
  acceptance also remains open.
- **Pending design decisions:** Stage 1 lifecycle, Stage 2 timing, Stage 6
  locale strategy, and Stage 9 composition/D1d boundary. Advanced GPU/native
  composition and upstream patch promotion remain separately gated.

## Correction — Stage 9 serialization status (2026-10-23)

The older notes below predate the completed D1 work. D1a–D1c are now
implemented: the versioned descriptor schema/codec, signal-only local compiled
preview host, and bounded Playground inspector. This does not serialize
arbitrary Rust views or runtime state, and it does not complete UI Studio.
The proposal recommends a descriptor-first composition path; approval for that
broader boundary is pending. See `docs/tasks/SERIALIZATION_BRIDGE_PROPOSAL.md`.

## Verified updates (2026-10-19)
- **Correction — Toggle theme role:** an attempted M89 fix was based on reading the component-local `off: secondary` role without tracing the root role map. `themes/macos27.json` maps root `secondary` to `toolbar`, so Toggle's existing toolbar-background mapping was already correct. The attempted content-surface change and its test were removed; no Toggle theme defect is established by that audit.

- The opt-in Playground theme-file reload described below was already implemented as M65 in `apps/rustui-playground/src/stories/foundation_additions.rs`, with the contract in `docs/tasks/AUTO_THEME_RELOAD.md`. It polls every 350 ms through GPUI, reloads through RustUI's `ThemeFile`, and stops when disabled or unmounted. `rustfmt --edition 2024 --check apps/rustui-playground/src/stories/foundation_additions.rs` and `./scripts/cargo-headstart check -p rustui-playground --locked` passed. No new files were needed.
- A source audit found no Playground-owned theme fallback: the app depends on `rustui`, loads `themes/macos27.json`, and activates it through `rustui::theme::activate_theme`. The framework adapter applies the profile.
- The framework coverage verifier passes for all 139 GPUI Components theme-color roles and all 24 styled macOS 27 component records. `./scripts/cargo-headstart check -p rustui` passed. No GPUI-default color fallback was identified in the audited renderers.
- Remaining declared theme limitation: dark `radio_button.blend_mode = "plus-lighter"` is retained as metadata but not rendered. The pinned GPUI/GPUI Components API has no per-element blend-mode support; the light radio rule does not declare it. No approximation or vendored patch was added.
- The trigger-driven RustUI Dialog wrapper and its public-API Playground story are now implemented. The source/build milestone is separate from GUI acceptance: the dialog has not yet been run and inspected in the canonical app. See `docs/tasks/DIALOG_PLAYGROUND_STORY.md`.

## Completed bounded delivery: Playground theme-file reload (M65)

This milestone is implemented in the existing Theme and Layout story. It is
not a pending recommendation and should not be reimplemented.

### Verified implementation boundary

- `crates/rustui/src/theme/watcher.rs`: `ThemeFile::open` and
  `reload_if_changed(mode, cx)`; invalid reads/parses/activation failures leave
  the last successful document and fingerprint intact.
- `apps/rustui-playground/src/stories/foundation_additions.rs`:
  `FoundationAdditions` stores the theme file and path and already reloads it
  from `AppAction` callbacks.
- `vendor/gpui-pre-0.3.9/src/executor.rs`: `BackgroundExecutor::timer` is an
  upstream scheduling primitive available in the pinned snapshot.
- `crates/rustui/src/runtime/tasks.rs`: owner-scoped `OwnerTask` demonstrates
  GPUI background/foreground handoff, generation checks, cancellation, and
  weak-owner behavior. Its API is one-shot, so do not assume it directly
  provides a repeating watcher.

The task record reports an opt-in 350 ms GPUI polling loop, reload through
RustUI's `ThemeFile`, and stopping when disabled or unmounted. Formatting and
the locked Playground package check are recorded as passing. This is theme-data
reload, not Rust recompilation or arbitrary Rust hot reload. Native GUI
acceptance remains a separate gate unless tied to a specific recorded run.

## Other examined gaps

- **Preview:** `apps/rustui-playground/src/lib.rs::preview` already mounts
  selected story/catalog views and source examples. There is no RustUI
  `Preview`/provider API in framework or macro crates; catalog notes explicitly
  call several compositions illustrative. A new declarative preview contract
  needs a design decision before a source-level implementation can be scoped.
- **Async tasks:** `crates/rustui/src/runtime/tasks.rs` and
  `apps/rustui-playground/src/stories/owner_tasks.rs` implement and demonstrate
  owner-scoped background tasks. Unit tests cover generation checks, cancel,
  terminal cleanup, and generation non-reuse; M72 records bounded native restart,
  cancel, error, and owner-release observations. The M72 source manifest does
  not hash `runtime/tasks.rs` or directly count private registry cleanup, so that
  runtime acceptance is not source-linked. Worker interruption and external
  side-effect cancellation remain unsupported guarantees.
- **Environment/state:** `Environment` and `EnvironmentReader` are public in
  `crates/rustui/src/view.rs`; retained children receive refreshed snapshots on
  parent lowering. `docs/tasks/F1D_ENVIRONMENT.md` explicitly limits this to
  render-time sampling: no per-value subscription, interior-mutation
  observation, or cached-child invalidation. Fine-grained observation would
  change runtime scheduling/invalidation boundaries and is not a small
  independent follow-up.
- **Serialization/UI Studio:** `serialize_theme` remains theme-document export
  only. D1a–D1c provide descriptors, the compiled preview host, and the bounded
  Playground inspector; they do not provide general view/state serialization,
  composition editing, source editing, or D1d external-host integration. The
  descriptor-first proposal is available, and approval for that broader
  boundary remains pending.

## Remaining approval gates for full roadmap claims

- The roadmap's P4 GPUI Fast, upstream patching, presentation/damage work, and
  advanced GPU/native composition require separate phase approval and verified
  compatibility with the pinned upstream snapshot.
- P3 candidate revisions/patch promotion require approved pipeline and candidate
  inputs; current P2 thresholds are proposed and require baseline calibration
  and approval before controlled optimization comparisons.
- Broader UI Studio composition and general state serialization require the
  pending descriptor-first boundary decision; theme JSON export does not
  settle them.
- X1 platform expansion requires a per-platform scope decision and evidence on
  each target. Current native evidence is bounded and does not establish
  platform-wide acceptance.
- Icon/Spinner artwork remains gated on approval of its asset source; no new
  asset/dependency source is implied here.

## Evidence sources inspected

- `docs/IMPLEMENTATION_ROADMAP.md` (stage 9, current verified status, F2/D1/P4/X1 queues)
- `docs/tasks/T1A_THEME_RELOAD.md`, `docs/tasks/T1B_THEME_SERIALIZATION.md`
- `docs/tasks/F2_OWNER_TASKS.md`, `docs/tasks/F1D_ENVIRONMENT.md`
- `crates/rustui/src/theme/watcher.rs`, `crates/rustui/src/runtime/tasks.rs`,
  `crates/rustui/src/view.rs`
- `apps/rustui-playground/src/stories/foundation_additions.rs`,
  `apps/rustui-playground/src/stories/owner_tasks.rs`,
  `apps/rustui-playground/src/lib.rs`, `apps/rustui-playground/src/catalog.rs`
- `vendor/gpui-pre-0.3.9/src/executor.rs`

## Update 2026-10-19 — milestones 93–96

- Build provenance now records caller-declared Cargo features; the focused test
  passed. This remains metadata, not proof of historical build flags.
- Dialog received bounded canonical-Playground checks for visible rendering,
  mouse/keyboard confirm and cancel, and Tab focus indication. Its accessibility
  snapshot still lacks a dialog role/container and reports root-window focus;
  theme/locale switching and full focus semantics remain open. See
  `target/headstart/evidence/dialog-gui-acceptance-20261019.json`.
- RustUI Sheet now wraps the pinned GPUI Components side sheet with trigger,
  optional title, and content. Focused Rust tests and the locked Playground
  package check passed; native visual acceptance remains pending. Drawer stays
  undefined and is not implemented by this Sheet.
- Cache calibration now has real 1/8/32-child scenarios with matched cached and
  uncached sets. Source tests, formatting, and example build passed. No new
  measurements were run; the prior single-workload CPU/RSS result remains the
  only performance evidence.
- Sheet's bounded GUI check now confirms right-edge rendering, title/body, and
  Escape dismissal. Accessibility still omits the title and Sheet role/container;
  broader theme/locale/platform acceptance remains open. Evidence:
  `target/headstart/evidence/sheet-gui-acceptance-20261019.json`.


## Update 2026-10-19 — M98 calibrated cache scenarios

Fixed the cache example's missing active-theme setup and completed the small, medium, and large matrix. All 18 workload records passed; per-scenario CPU and RSS summaries are recorded in `docs/tasks/P3C_CACHE_CALIBRATION.md`, with the raw evidence at `target/headstart/evidence/cache-calibration-scenario-matrix-20261019-fixed.json`. This is whole-process calibration for the mixed workload, not a cached-versus-uncached speed comparison.


## Update 2026-10-19 — M99 NavigationSplitView keyboard acceptance

The canonical Playground keyboard path passed for Up/Down, Home/End, disabled-row skipping, and Return preserving the current selection. This confirms the existing public contract; no NavigationSplitView code change was needed. Evidence: `target/headstart/evidence/navigation-split-keyboard-acceptance-20261019.md`.


## Update 2026-10-19 — M100 Menu ownership and dependency boundaries

Native acceptance confirmed an open Menu popup disappears from the accessibility tree when its RustUI owner is removed. Stale-action activation was not possible in the observed state; component-entity destruction remains unverified. See `docs/tasks/O1_MENU_DROPDOWN.md` and `target/headstart/evidence/menu-owner-removal-20261019.json`.

A read-only manifest/source audit found no confirmed Allui or Chorus dependency violation: resolved packages contain neither, Playground depends on public `rustui`, GPUI dependencies are owned by the framework crate, and Lab delegates to Playground.

## Refresh 2026-10-23 — M1222

M664 is a historical Stage 9 LazyGrid compiled-preview specimen. The current
chronological roadmap head is M1219 (Stage 10 identity-bound M276 correctness
rerun). For candidate `quad-stable-sort-v2-chain-m272-20261020`, the locked /
offline M276 harness passed 10/10 and the P3M aggregator returned `consistent`
with build and correctness passed. GUI, performance, independent review,
approval, and promotion remain open. Stage 11 Web/Wasm browser keyboard,
accessibility-tree, font/glyph, and platform acceptance remain open. Stage 9
D1d external-host integration and broader composition-editing architecture
also remain open; Stage 8 TreeItem semantics-preserving row accessibility and
native accessibility/interaction acceptance remain open. No global completion
percentage is defined by the roadmap because implementation and acceptance
gates are separate.

See `docs/tasks/M1219_STAGE10_IDENTITY_BOUND_M276_RERUN.md`, `docs/tasks/M1220_STAGE11_REBUILD_WEB_BUNDLE.md`, and `docs/tasks/M1221_STAGE10_INDEPENDENT_REVIEW.md`, and `docs/tasks/M1222_STAGE8_TREEITEM_ROW_SEMANTICS_BOUNDARY.md` and
`docs/IMPLEMENTATION_ROADMAP.md`.

## Stage 6 full locale runtime gap — 2026-10-23

`rustui-i18n` has no approved implementation path to full CLDR/ICU/Fluent
coverage with its current dependencies. The detailed audit found only
transitive ICU support crates and no Project Fluent runtime. The next step is
a cross-target locale-data/dependency decision or a host-native adapter
contract; no partial hand-coded locale rule is being presented as full
coverage. See `docs/tasks/STAGE6_FULL_LOCALE_RUNTIME_GAP_AUDIT.md`.

## Refresh 2026-10-23 — M1223

M1223 is the current recorded milestone: Stage 5 ListView reconciliation now
keeps locale-resolved row labels stable across rerenders and replaces them when
the resolved locale changes. Its focused test passed 1/1, and an independent
source review found no regression. Native visual and live locale-switch
acceptance remain open. A follow-up retained-delegate regression now verifies
that the English resolved labels survive an unchanged rerender and that the
French resolved labels replace them after a locale change; it passed 1/1.

Stage 11's M1220 bundle rebuild establishes packaging and static serving only.
M1249 later applied approved M855 v3 and verified focused Button Space
activation in a fresh browser. Selected-story pointer/assets/navigation,
broader keyboard navigation, browser accessibility and screen-reader output,
text entry/IME, and font/glyph coverage remain unverified. Stage 10's named M276
candidate has 10/10 correctness evidence and an independent review; GUI,
performance, approval, and promotion remain open. Stage 8 still lacks outer
tree-row `aria-level` support. Stage 2's timing API remains approval-gated,
and Stage 4 has no named reproducible failure after M1211. Stage 7's M1204
native keyboard and screen-reader acceptance remains open. Stage 5 still needs
live locale-switch and native visual acceptance. Stage 1 lifecycle
implementation has a separate proposal ready for review; its reserved scaffold
requires approval before implementation
(`docs/tasks/STAGE1_LIFECYCLE_API_PROPOSAL.md`). Stages 0 and 3 retain their
other recorded acceptance gaps. The Stage 6 cross-target locale strategy
remains undecided. No roadmap-wide completion percentage or evidence-based
finish date is defined.

## Stage 3 implementation update — 2026-10-23

Closed a source-level interaction gap in M1184: each illustrative Close,
Minimize, and Zoom click now marks the preview active before recording the
selection. The focused regression deactivates the preview before each click and
asserts reactivation; the focused locked/offline test passed 1/1. This does not
close the separate Stage 3 native acceptance gaps.

## Open architecture decisions — 2026-10-23

Focused decision documents are ready for the remaining proposal-gated work:

- Stage 1 lifecycle: approve retaining the current `View::on_mount` /
  `View::on_unmount` contract and GPUI entity-release boundary before aligning
  scaffold/registry documentation
  (`docs/tasks/STAGE1_LIFECYCLE_API_PROPOSAL.md`).
- Stage 2 timing: approve the proposed timing-function API, formulas, first
  consumer, and file boundary before implementation
  (`docs/tasks/STAGE2_TIMING_FUNCTION_PROPOSAL.md`).
- Stage 6 localization: choose the cross-target locale-data/runtime strategy,
  targets, data/version policy, and Fluent contract before changing dependency
  revisions or the public API
  (`docs/tasks/STAGE6_LOCALE_RUNTIME_ARCHITECTURE_DECISION.md`).
- Stage 9: choose the bounded composition-editing boundary and name the D1d
  external consumer/transport before defining remote action integration
  (`docs/tasks/STAGE9_COMPOSITION_AND_D1D_DECISION_MEMO.md`).

These are decision aids, not approvals or implementation. Stage 11 M855 v3 was
subsequently approved and applied under M1249; any new upstream patch remains
separately gated.

## Refresh 2026-10-23 — M1224–M1225

M1224 removes the Web shell's body-level `aria-busy` state so it does not block
the descendant loading live-region announcement; the focused packaging check
passed, with browser screen-reader acceptance still open. M1225 bounds M287
screenshot and accessibility-tree reads to 64 MiB before hashing or PNG
parsing; its focused oversized-artifact regression passed 1/1. This closes a
local resource-boundary defect only. Stage 10 GUI/performance acceptance,
approval, and promotion remain open.

## Refresh 2026-10-23 — M1226–M1228

Stage 10 implementation now requires P3G path-source tree hashes, rejects P3I
path packages with contradictory raw source metadata, and rechecks candidate
inventory file type/link count/identity around hashing to close a replacement
race. Each change has a focused regression passing 1/1. These strengthen local
evidence intake and consistency; they do not establish candidate build,
correctness, GUI, performance, review, approval, or promotion gates.
