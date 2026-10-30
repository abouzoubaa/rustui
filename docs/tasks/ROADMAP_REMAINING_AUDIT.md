## R13/R17 native TabView RTL keyboard event — 2026-10-30

On parent `f24c2168` plus this regression, the macOS-hosted GPUI TabView test switched from English to Arabic with the first tab selected. Left moved focus past a disabled middle tab without writing the owner, and Enter selected the third tab. The focused test passed 1/1 (`target/headstart/evidence/native-tab-view-rtl-20261030.log`, SHA-256 `45e612c2cdb4871029611447b5e4afe94820e52d84835936f441f6d459eabbaf`). This verifies the bounded direction-aware event path, not rendered RTL layout, native Playground or broader acceptance.

## R17 native TabView keyboard event path — 2026-10-30

On parent `74369056` plus this regression, a macOS-hosted GPUI test window focused a controlled TabView with a disabled middle tab. Right moved focus past the disabled tab without writing the selection; Enter selected the third tab; Right wrapped and Space selected the first. The focused TabView suite passed 26/26 (`target/headstart/evidence/native-tab-view-suite-20261030.log`, SHA-256 `8ee14cb0f4be9c04a209e8ce4ea622f4a835303466cd4bf485c9b4c4b012bcb6`). This is bounded test-window evidence, not native Playground, RTL, visual or broader acceptance.

## Native workspace library integration checkpoint — 2026-10-30

On clean source commit `5f3b467e`, macOS `./scripts/cargo-headstart test --profile headstart --locked --offline --workspace --lib` passed 1,397 tests across six library targets (1,033 + 9 + 83 + 40 + 232 + 0); no tests failed or were ignored. Log: `target/headstart/evidence/native-workspace-lib-5f3b467e-20261030.log`, SHA-256 `d4ec8994f04f2828417beb45e428fc2f0702a218851e46812169b8b5f8834702`. Separately, the focused RustUI input group passed 222/222 on this commit (`native-input-integration-5f3b467e-20261030.log`, SHA-256 `7cfa4001bb5626a6dc6799999ff9066ee150544fa4a5f51694ca0e069e6df787`); those 222 are included in the workspace count. These tests do not establish native Playground, OS IME, accessibility, licensing or production acceptance.

## R15 native ColorPicker external owner update — 2026-10-30

On parent `8e11a38b` plus the regression, a macOS-hosted GPUI test window edited incomplete hex text, then the owner cleared the color and supplied a different one while the popup was open. The retained value and hex field synchronized to each owner change; a subsequent unfinished edit and Escape preserved the new owner color, and a later valid hex commit still updated it. The focused test passed 1/1 (`target/headstart/evidence/native-color-picker-owner-update-20261030.log`, SHA-256 `2da3b6a7b3ed025d67244f98bffde7235b9dc568af3c944440967bf3eee24ce8`). This is test-window evidence for these transitions, not native Playground, concurrent updates, OS IME or broad acceptance.

## R15 native ColorPicker cancellation — 2026-10-30

On parent `efc9ba0f`, a macOS-hosted GPUI test window showed Escape closing the picker while retaining incomplete `#FF` text, which could reappear on reopening (`target/headstart/evidence/native-color-picker-cancel-before-20261030.log`, SHA-256 `250da0ac41c863fe2a60984f9e2fc5d09af63f2c464fb92426ba23092237c6a3`). Dismissal now resynchronizes the hex and slider state from the retained committed value without emitting a color change; this preserves already committed slider changes. The event-path regression checked cancellation, reopening and a subsequent valid commit. The ColorPicker suite passed 19/19 (`target/headstart/evidence/native-color-picker-cancel-suite-20261030.log`, SHA-256 `c6d7dff2cd8b658fc122d2cc2f6edb9740ffcc1709ca81bc1915e17055e7d2ae`). External owner updates during editing, pointer dismissal, actual native Playground and broader input acceptance remain open.

## R15 native ColorPicker hex commit — 2026-10-30

On parent `beca9f3f`, entering incomplete `#FF` and pressing Enter closed the picker without changing the color because the input's Enter also reached the parent popover's Confirm handler. The failing macOS-hosted test log is `target/headstart/evidence/native-color-picker-hex-invalid-20261030.log` (SHA-256 `2807aaa3f86b1fe3b28f7f2d1a1e5d4da820c0fe932cd6b996a0d5588e9f3f50`). RustUI now contains the parent Confirm action at the hex field while retaining the input's own commit event. The event-path regression checks incomplete input stays open and preserves the owner, valid `#FF0000` stays uncommitted during editing, and Enter commits red and closes the picker. The ColorPicker suite passed 19/19 (`target/headstart/evidence/native-color-picker-hex-suite-20261030.log`, SHA-256 `77ad847a1725e97655536b03447ddf640bb6a5d73c8384a8c49349349a817800`). Actual native Playground, OS input/IME, visual and broader acceptance remain open.

## R15 native ColorPicker mode keyboard navigation — 2026-10-30

The underlying mode tabs did not take keyboard focus, making the HSLA controls unreachable by keyboard. RustUI now exposes the mode bar in focus traversal and handles Left/Home for Palette and Right/End for HSLA, without changing pointer tab selection. On parent `decd621f` plus this change, a macOS-hosted GPUI test window opened the picker by Enter, switched Palette → HSLA → Palette → HSLA with arrow keys, then focused and adjusted all four HSLA sliders. The ColorPicker suite passed 19/19, including swatch activation (`target/headstart/evidence/native-color-picker-mode-keyboard-20261030.log`, SHA-256 `730cfe2c4f259ef00162ea88d4c5f18a570417da2003f0887fc357b3a4247a82`). Keyboard hex editing/commit, native Playground, visual behavior and broader keyboard and accessibility acceptance remain open.

## R15 native ColorPicker HSLA keyboard controls — 2026-10-30

The ColorPicker HSLA panel used the underlying pointer-oriented GPUI sliders directly, without RustUI Slider's keyboard handling. Each HSLA slider now participates in focus traversal and reuses RustUI's horizontal slider keyboard policy; changes are emitted through the existing slider event subscription to the owner-bound color. In a macOS-hosted GPUI test window, Right and End changed and clamped Hue, and Home/End changed Saturation, Lightness and Opacity while focus moved through the four controls. The focused ColorPicker suite passed 19/19 on parent `0d426838` plus this change (`target/headstart/evidence/native-color-picker-hsla-suite-20261030.log`, SHA-256 `aca4f1a23df6abf12f2cc5d6476edf22c0d6a6000d66cd23f66212f556f5309a`). Actual native Playground input, pointer behavior and broader locale, visual and accessibility acceptance remain open.

## R15 native ColorPicker keyboard swatch selection — 2026-10-30

On parent `8acdd578`, focusing a featured swatch then pressing Enter left the owner color unchanged because the parent popover handled Confirm and closed before swatch activation (failing log `target/headstart/evidence/native-color-picker-swatch-keyboard-20261030.log`, SHA-256 `8967dbe01aa1f6d35608234fe4fb348404fa28cc48ecbb15d4d0479c11d1d178`). RustUI now handles Confirm on each swatch, selects its color through the retained state, and stops the parent action. A macOS-hosted GPUI test window verified Space and Enter on two focused featured swatches, owner-bound color updates and popup closure after an English-to-Arabic switch. The focused ColorPicker suite passed 18/18 (`target/headstart/evidence/native-color-picker-swatch-suite-20261030.log`, SHA-256 `91bb7d8bd0e526a3d79b8c6c4910523c4dfacfd745db51e128e6fa0008cf580b`). Native Playground, pointer interaction and broader keyboard and locale acceptance remain open.

## R13/R15 native ColorPicker keyboard trigger — 2026-10-30

On parent `7d3999bd` plus this test change, a macOS-hosted GPUI test window focused the ColorPicker trigger and sent Enter key-down and key-up events; the picker opened and remained open through an English-to-Arabic locale switch after programmatic HSLA tab selection. The focused regression passed 1/1 (`target/headstart/evidence/native-color-picker-trigger-keyboard-20261030.log`, SHA-256 `db2a94648006ab9e578658ed1a8cd488c50329b70bafbc90b9bb48f6b7dec6ef`). This does not establish pointer interaction, native Playground visual behavior, OS input handling, or complete locale acceptance.

## R03 native input public contract documentation — 2026-10-30


`docs/PUBLIC_API.md` now describes the current GPUI Base checkable controls, the RustUI-composed ColorPicker popup and retained GPUI color state instead of the old ColorSelect implementation, and the current Stepper range, displayed locale and direction behavior. The descriptions were checked against the source on parent `9f7db1c8`; `git diff --check` passed. This documentation correction does not close public API completeness, native visual/keyboard or release acceptance.

## R30 current GitHub source CI — 2026-10-30

GitHub Actions push [run 41060841118](https://github.com/abouzoubaa/rustui/actions/runs/41060841118) completed successfully on exact commit `9028d57fd87779545b801730c83f6a0345c96712`: its `source` job passed, while `macos-and-web-builds` was skipped. Earlier runs were canceled as newer commits arrived. This confirms that source job only; it is not compiled macOS CI, native GUI acceptance, licensing clearance or release readiness. The local native test checkpoint remains recorded separately.

## R13/R15 native ColorPicker palette direction — 2026-10-30

ColorPicker's trigger contents, featured-swatches row and individual crayon rows now use the existing logical input-row layout, applying GPUI Row in LTR and RowReverse in RTL. Swatch identities, color values and HSLA slider semantics are unchanged. On parent `abd5a0c1` plus this change, the focused native-hosted ColorPicker suite passed 18/18, including the retained-locale test and logical-row style checks (`target/headstart/evidence/native-color-picker-rtl-rows-20261030.log`, SHA-256 `306429fa3a84b6b09c1b21785d58b66a18a059b751d2e21b334796e206c7ea8f`). Native visual, pointer, narrow-width and broad RTL acceptance remain open.

## R13/R15 native ColorPicker retained locale transition — 2026-10-30

A macOS-hosted GPUI Root/test window rendered a ColorPicker with owner-bound color; the test set its retained state open and selected HSLA mode, switched from English to Arabic, then verified the popup remained open, mode and owner/retained color were preserved, and the rendered test-window tree exposed the Arabic Palette tab label. The focused ColorPicker suite passed 18/18 on parent `8552deba` plus this regression (`target/headstart/evidence/native-color-picker-locale-suite-20261030.log`, SHA-256 `50c75171b902b4ce66bc25718786db764d374b65379692ff66faf5417323359c`). Programmatic state setup and the tree observation do not prove pointer or keyboard popup operation, visible text geometry, native Playground appearance, narrow widths or full locale acceptance.

## R30 current native workspace integration tests — 2026-10-30

On clean commit `414c9d98` (only a validation-ledger change after library-tested source `64005a18`), local macOS `./scripts/cargo-headstart test --workspace --test '*' --profile headstart --locked --offline` exited 0: 60 tests passed across 34 suites, with zero failures or ignored tests. Log `target/headstart/evidence/native-workspace-integration-414c9d98-20261030.log`, SHA-256 `e161b6b7302a6fe52ef2da6f07d5fedd8e2325fb95d45428ca90db94e8572e8a`. Together with the preceding 1,395 library tests this is 1,455 passing native-hosted tests on the same code, not native Playground interaction, real IME, packaging, licensing or release acceptance.

## R30 current native workspace library tests — 2026-10-30

On clean source commit `64005a18592dcb0e05823d0b465368d72bfeacf2`, local macOS `./scripts/cargo-headstart test --workspace --lib --profile headstart --locked --offline` exited 0: 1,395 tests passed across five nonempty library suites (1,031 + 9 + 83 + 40 + 232); one additional library suite had zero tests. Log `target/headstart/evidence/native-workspace-lib-64005a18-20261030.log`, SHA-256 `5483373bed05a08fcf8474eff038694970e73a52e39423e3a38806d7463110c1`. This is current native-hosted library/test-window evidence, not integration targets, native Playground interaction, real IME, release packaging, licensing or production acceptance.

## R13/R15 native ColorPicker popup localization — 2026-10-30

ColorPicker now resolves its Palette tab and Hue, Saturation, Lightness and Opacity slider captions through the RustUI English, French and Arabic catalogs, with English fallbacks when messages are unavailable. Its caption/slider rows follow the active LTR or RTL direction; color values and slider direction are unchanged. On parent `e305a451` plus this change, three runtime catalog groups validated (`target/headstart/evidence/native-color-picker-popup-locales-20261030.log`, SHA-256 `26c87c1b1c67fe8271a60680339e798d687e722ec4b60dcc86d7f8c26ca462e5`) and the focused native-hosted ColorPicker suite passed 17/17 (`native-color-picker-popup-tests-layout-20261030.log`, SHA-256 `4ca838370f1357dc89163d8f35fe5cd6e8e5ce48b97cbbc601afbf59bb54566e`). The first test compile failed because test catalog include paths were one directory short; that was corrected (failed log SHA-256 `8579e8bf6b7edea38a9b9da3b64c29313d11184a5b932e14952c15c61daad994`). Native visual, actual popup interaction, narrow-width and locale-switch acceptance remain open; the protected-bundle cleanup rejection remains unresolved.

## R31 native maintenance notice review — 2026-10-30

The four unmaintained packages selected in the default macOS Playground normal/build graph are not described as security vulnerabilities by their matched OSV advisories. `paste 1.0.15` is a procedural macro selected by Components, Metal and image dependencies; `instant 0.1.13` is selected by GPUI Base/Components; `rustybuzz 0.20.1` is reached through `usvg 0.46.0`, and `ttf-parser 0.25.1` through `fontdb` and `rustybuzz` under that SVG chain. Selection in Cargo's graph does not prove that a particular runtime code path executes. The advisories name possible alternatives (`web-time`, `pastey`/`with_builtin_macros`, `harfrust`, `skrifa`), but they are not drop-in validation or approval to change pinned upstream dependencies; no replacement was made. The four exact reverse-tree logs have SHA-256 `7e834c43ce5aced95f1f178d46ffb960829f5651b21c45b2883c2ce8eeeb73dd` (instant), `a60057cf7e8046d590a3dbaaa7e363b6fb37d5747c1ea1c1e4db9c6aee79f1f6` (paste), `cdd9fb9573ee0d53340e4183674ce4feff037d457f3d696a6cba6d0177133f42` (rustybuzz), and `10dfc67f706b6abd82aeda7fab502b934e1c3bb1061bca311fe026e1a4f4c55f` (ttf-parser), under `target/headstart/evidence/native-advisory-tree-*-20261030.log`. This is maintenance classification, not native security clearance or an assessment of future advisories.

## R31 current native dependency advisory scan — 2026-10-30

At 2026-10-30 12:54 UTC, an OSV query covered all 833 distinct registry-sourced package/version pairs in `Cargo.lock` (SHA-256 `b70bbf39c2a467905d278bd0216ecfcce20023944c32bfa27fee4f0dac132a43`), with complete batch responses and no pagination tokens. Nine package/version pairs matched: `block-buffer 0.10.4` matched GHSA-qwgh-2vcv-g2f7, which describes possible memory corruption after a caught panic in an inline-buffer implementation; `instant`, `paste`, `rustls-pemfile`, `rustybuzz`, `ttf-parser`, `zbus_names`, `zvariant` and `zvariant_derive` matched unmaintained notices. OSV reports a range match, not proof that the described failure mechanism applies to this older version. Default-feature `aarch64-apple-darwin` Playground reverse trees using normal/build edges did not select `block-buffer 0.10.4`, `rustls-pemfile`, `zbus_names`, `zvariant` or `zvariant_derive`; they selected `instant`, `paste`, `rustybuzz` and `ttf-parser`, including possible build/proc-macro dependencies. The all-target reverse tree reaches `block-buffer 0.10.4` through `oo7` and GPUI Linux. Results are scoped to this lockfile and selected feature/target graph; no exploitability or security-clearance conclusion is made. Raw lock scan SHA-256 `3eb5ffa060302b8ce2e1d07e1d02b4ac7b7a82139df2bb6a2db521e953ecfaef`, full advisory responses `9919b3c0aa10cbfeeccf580a2bb4f1a7495aa7cce94e5b844d7e3950dd8f05c0`, and native/all-target `block-buffer` reverse-tree logs `4dc8e4fe7f86c681d8b81d94c4e76a3dda13457f37def5513d5850195a7778f6` / `29423f86650935e344fdcc4840c9b0c083e4c0ec103b92bb29ed2126e6dd4f22`; all are under `target/headstart/evidence/`. No dependency version was changed.

## R13/R15 native checkable controls across locale change — 2026-10-30

A macOS-hosted GPUI test window rendered Checkbox, Radio and Toggle with owner bindings. Keyboard activation set Checkbox in English; switching to Arabic preserved it, and keyboard activation then set Radio and Toggle. Switching back to English preserved all three owner values. The focused test passed 1/1 on parent `63c08a11` plus the test. Log `target/headstart/evidence/native-checkable-locale-transition-final-20261030.log`, SHA-256 `987f9a4f23fd1e5d7dbf0d66f776aaa194c6927960443531a25855b15eb6a0ce`. This does not establish visible label translations, real native visual direction, pointer operation or broad locale/input acceptance; the protected-bundle cleanup rejection remains unresolved.

## R13/R15 native checkable-control direction — 2026-10-30

Checkbox, Radio and Toggle now apply active logical direction to their indicator/label row: LTR uses GPUI Row and RTL uses RowReverse. The control state, event handlers, focus and Toggle thumb behavior are unchanged. On parent `c2b2668f` plus this change, the macOS-hosted `components::input` suite passed 218/218, including a GPUI style regression for each underlying control in both directions. Log `target/headstart/evidence/native-checkable-rtl-20261030.log`, SHA-256 `2b4d1dfff8f228e58eb5114b3a8b505d323cfaedda3ed38fce57ff54492ad05f`. These are source and test-window results, not native Playground visual/RTL, broader keyboard or release acceptance. The protected-bundle cleanup rejection remains unresolved.

## R15 native Stepper keyboard event path — 2026-10-30

A macOS-hosted GPUI test-window regression focuses the rendered Stepper increment button, sends Enter key-down/up events, confirms owner values 0 → 1 → 2 with no further increment beyond the configured maximum, then focuses decrement and confirms 2 → 1. The focused Stepper suite passed 17/17 on parent `73804997` plus the test (`target/headstart/evidence/native-stepper-event-suite-20261030.log`, SHA-256 `ccc916e00f5b961580a03bdeee877c7ea563cd743a6bc23c5fd2423052bfe177`). Earlier harness attempts failed because `simulate_keystrokes` sends key-down without the key-up that GPUI button activation requires; the focused retry log SHA-256 is `1682c6f329c66b70b3b318736c4bacc7c9cfd8c7eac91c70a3ea3d1bc4afd0c5`. This event-path test is not a native Playground interaction, visual, locale-switch or broader input acceptance.

## R13/R15 native Stepper direction — 2026-10-30

Stepper's horizontal label/value/control row now follows the active layout direction: GPUI Row in LTR and RowReverse in RTL. The vertical increment/decrement order remains unchanged. On parent `c685421b` plus this change, the native-hosted focused Stepper suite passed 16/16, including a GPUI style regression for both directions. Log `target/headstart/evidence/native-stepper-rtl-20261030.log`, SHA-256 `4448d99a3db15ce5fdccd69bfaa3fa1c75af619112582ef40e25443682518eb0`. This verifies source-level layout selection, not actual native rendering, keyboard or broader locale acceptance; the protected-bundle cleanup rejection remains unresolved.

## R13/R15 native Stepper numeric display — 2026-10-30

Stepper now formats its displayed clamped integer with the active RustUI locale using the existing integer formatter, retaining the unlocalized decimal fallback when no locale is active. The controlled integer and increment/decrement behavior do not change. The focused native-hosted Stepper suite passed 15/15, including English, French, Arabic negative numbers and the no-locale fallback, on parent `9d41fb5c` plus this change. Log `target/headstart/evidence/native-stepper-number-locale-20261030.log`, SHA-256 `0ca19f3deda2981faaf0b2bf15cd595d7d16ff1c49d6d718a6f27f6a91747c29`. A current native Playground visual/locale-switch check remains open; the protected-bundle cleanup rejection remains unresolved.

## R17 native Tree keyboard event path — 2026-10-30

The Tree leaf-activation statement in M155 of the historical roadmap predates the current `TreeView::on_activate` implementation. A GPUI test-window regression now focuses the retained Tree, sends Enter on a controlled selected leaf, sends Down past a disabled leaf, and sends Enter on the next enabled leaf; both owner activations and the selected ID are checked. The test installs the same Tree Enter binding as RustUI application bootstrap. Without that binding, the first test setup failed to activate (`native-tree-leaf-keyboard-retry-20261030.log`, SHA-256 `f03ba5f5cf81a50745f177a439c6439bae324406bb89d13ac52a1ce312fc128b`); the corrected focused test passed (`native-tree-leaf-keyboard-final-20261030.log`, SHA-256 `c332069f4e662e6826b5ef2051117800b039151998867f478369828593bbde3a`) and the Tree/Outline suite passed 34/34 (`native-tree-keyboard-suite-20261030.log`, SHA-256 `9b9be5db7023d23c8df829f10adbeb0fb1310ef6fe4332155dd53b48b33c35f7`). This is a macOS-hosted GPUI test-window event-path check on parent `22b8ba8e` plus the test, not a native Playground or broad keyboard acceptance result.

## R30 current native all-target check — 2026-10-30

On clean commit `01de6d2c`, local macOS `./scripts/cargo-headstart check --workspace --all-targets --profile headstart --locked --offline` exited 0. It emitted warnings, including six for the Playground library test target. Log `target/headstart/evidence/native-all-targets-01de6d2c-20261030.log`, SHA-256 `c66c0ea059ced96e5b6aa93183dd367b0e03a06a669e40c3ab76d1036ed811b4`. This confirms compilation of the selected native targets, not execution, GUI acceptance, license clearance or production readiness.

## R31 native upstream notice search — 2026-10-30

The complete recursive GitHub tree responses for the previously source-matched upstream commits of `block 0.1.6` (`47178790cfc9d4a8b092051d8b413b78bd31254a`, 16 entries), `mac 0.1.1` (`66afc663b68a65633ea165c742b5a9c6734581c2`, 13 entries), and `malloc_buf 0.0.6` (`a7811e5f4c6f9685dd57ad9bfe34e9bf6b0ba4f9`, 4 entries) reported `truncated: false` and no basename containing license, licence, copying or notice, case-insensitively. Raw responses are `target/headstart/evidence/native-notice-tree-{block,mac,malloc-buf}-20261030.json`, with SHA-256 respectively `a41e05fe112d31934a867ae5c43eac6e0ef82e29705fdd30d6b83eae186aa59e`, `58e2d70061cd76bd9422b20adc470feb8a982f4b54b47d5469b53a1560b7433d`, `8356f2597ab05fde7560752eb9d8c356699967e803f8d7ae34f0eab97fb859b4`. The API returned HTTP 422 for the recorded Pathfinder revision. These searches do not establish absence of text under other filenames, license obligations or clearance; all five native notice gaps remain open.

## R13/R33 native Drawer example starter — 2026-10-30

The canonical Drawer example starts with an empty editable note and uses its existing translated Notes placeholder instead of prepopulating English sample text. User edits remain owner state, and compiled-preview specimens still receive their caller-supplied body. On parent `abfc0795` plus this change, three runtime locale catalog groups validated (`target/headstart/evidence/native-drawer-empty-locales-20261030.log`, SHA-256 `26c87c1b1c67fe8271a60680339e798d687e722ec4b60dcc86d7f8c26ca462e5`); the compiled-preview Drawer specimen test passed 1/1 (`native-drawer-empty-tests-20261030.log`, SHA-256 `76544e5d160eec6f61b0886d7a59368a82e9b0de1821253db280ae0798942d90`). Native visual and interaction acceptance on this change remains open; the protected-bundle cleanup rejection remains unresolved.

## R27/R35 native graph geometry edit — 2026-10-30

`GraphDocument::edited(MoveNode)` now returns after validating the new rectangle: an existing validated document's geometry-only edit cannot alter node identity, the node index or edge topology. Other edits retain full validation and index rebuilding. The new connected-graph regression compares the moved result with a fully reconstructed document, checks the original and node index, removes an incident node, and rejects a missing node; existing coverage rejects invalid geometry. On parent `667e176d` plus this change, GraphView tests passed 20/20 (log `target/headstart/evidence/native-graph-move-tests-20261030.log`, SHA-256 `3c468789701b89770a51244aa4e63352e8fd4aae982713b5a7f2b8480d9499b2`) and NodeEditor-filtered tests passed 1/1 (`native-node-editor-move-tests-20261030.log`, SHA-256 `2f67b6e19f539858d61e7371791eb2547426c28ae261e598eb3d1b4c3c9d38b6`). The same local release model benchmark recorded 50 moves on 1,024 nodes/1,023 edges at 15,095–16,436 µs across five samples, versus 33,453–35,072 µs in the prior separate run; 128-node moves took 1,904–3,122 µs. After log SHA-256 `fe455c53f5f3d26b39a6ca820cd7a66696b1675378c92bf615985ec3f2102360`. These separate, unpaired runs measure only this model operation and do not establish rendering, latency budgets, GUI performance or release acceptance.

## R27/R35 native connected-graph model workload — 2026-10-30

On macOS 27.0 (26A5425a), Apple M1 Max, `./scripts/cargo-headstart run --release --locked --offline -p rustui --example graph_edit_benchmark` built and ran on parent `7eb6536f` plus the new example. Five samples each used a chain of connected nodes, each with an input and output port, followed by 50 validated `MoveNode` edits. For 128 nodes/127 edges, construction took 73–123 µs and 50 moves 4,451–5,118 µs. For 1,024 nodes/1,023 edges, construction took 519–685 µs and 50 moves 33,453–35,072 µs. Log `target/headstart/evidence/native-graph-edit-benchmark-20261030.log`, SHA-256 `e3936989eb0d8fc0de21c3dcee7938e900dce8ab713a0ef5b601da1713eb2693`. These single-process model measurements include cloning inputs for construction and the public edit operation, but not rendering, event dispatch, frame latency, memory, interaction or release budget acceptance. The example was formatted after the run without semantic changes. Native GUI/performance acceptance remains open.

## Native-first priority — 2026-10-30

The user directed this session to finish its current Web/Wasm task and then focus on native RustUI while Web/Wasm work is deferred for later. The embedded Web preview viewport change was integrated in `93432e2a`, with its source-check record in `96126e1e`. Preserve existing Web work and its open requirements; do not treat this priority change as Web acceptance or release completion. Continue native implementation, macOS validation, English/French/Arabic and RTL, keyboard behavior, performance, licensing, packaging and CI. Accessibility and VoiceOver remain deferred as previously directed. The protected disposable-bundle cleanup rejection and held GUI reservation remain unresolved; do not disturb the preserved bundles.

## Current source release check — 2026-10-30

On clean commit `93432e2a`, `./scripts/verify-release.sh source` exited 0: 628 Python tests and 47 Node tests passed, along with the configured patch and packaging checks. Log `target/headstart/evidence/source-gate-93432e2a-20261030.log`, SHA-256 `cb64e94a73b5fc099aebca27dfd194a0560e5412adf6a2a8d6063ead3fc9840a`. This is source-check evidence; macOS and Web runtime acceptance, compiled CI, licensing and release readiness remain open.

## Catalog encoding allocation bound — 2026-10-29

Catalog encoding now counts a conservative lower bound of serialized text before cloning the caller's catalog. If the counted strings alone exceed the existing one-megabyte document limit, encoding rejects it before that clone; the bounded JSON writer still enforces the final byte limit. The focused codec suite passed 13/13, including an otherwise valid catalog with oversized allowed-value text (log `target/headstart/evidence/catalog-text-budget-tests-20261029.log`, SHA-256 `4ec657dadc19b9c22331f1d9ecd57c63d3d178a91d24597a33ec22a7b61e8030`). On parent `99167cee` plus this change, the Web release driver passed with 18/18 host IME-plan tests and three unresolved notice gaps (log `target/headstart/evidence/catalog-text-budget-web-20261029.log`, SHA-256 `8b8b8ffa6cd017ad83a055cc1d6f855c9d7bba4f950828b3778e5caa111acddf`; manifest SHA-256 `57505d02ecb121217725ea7b6994a082399b9b466730a9461f1d22fb9e2013da`). This bounds the counted text before the clone, not every possible allocation or the whole preview security review.

## Sheet owner action — 2026-10-29

The existing Sheet story now also demonstrates a button bound directly to the original owner, with localized labels and a visible owner count. Locale validation passed. On parent `ce4d8c40` plus the story change, the Web release driver passed with 18/18 host IME-plan tests and three unresolved notice gaps (log `target/headstart/evidence/sheet-owner-action-web-20261029.log`, SHA-256 `0e4279d769e050e7460f601681b215e282c8161279d059d1baacdbac2323b5f5`; manifest SHA-256 `25426f311ed05a7ba5b0a715d979c50fb632fd0ce78c7cbe2ede52192663ae3f`). In one in-app Chromium session, two direct owner actions changed the owner count from 0 to 2, a nested component action changed only its own count from 0 to 1, and both counts remained after closing and reopening the Sheet. The owned tab and server were closed. Native and wider browser acceptance remain open.

## Sheet owner binding — 2026-10-29

The existing public-API Sheet story now includes a directly owner-bound field alongside its existing nested focus component, with English, French and Arabic labels. On parent `e95dfddb` plus the story, a source-bound Web baseline focused that field but typing `abc` left both the field and owner value unchanged; the nested component's field accepted typing. RustUI Sheet now lowers retained content with the original owner and reconciles current content and labels, as Dialog does. The focused Sheet tests passed 20/20, including retained-content reconciliation (log `target/headstart/evidence/sheet-owner-tests-20261029.log`, SHA-256 `b84b1800f1074fcc991ae3b0fb136ed3b86b3105fcaa2389c6581a1d3fc3f40d`); locale validation passed. The final Web release driver passed with 18/18 host IME-plan tests and three unresolved notice gaps (log `target/headstart/evidence/sheet-owner-web-final-20261029.log`, SHA-256 `de785a65e903e2cd47debc6203e28cfb102c21cf9dd4052710534321156d7b89`; baseline and final manifests SHA-256 `bb52c729c386c1ab42b30109ee0429d59348f62d5552f64735091239436d3946` and `19b7cd47978c69acfd415dcbd4f47f9e9e7c5391c02f13579e9516880f44ba9c`). In one in-app Chromium session on the final bundle, direct typing updated both the field and owner value, the nested field and action worked, and their values remained after close/reopen. Test tabs and servers were closed. Native, real IME, other browsers and broader Sheet acceptance remain open.

## Select popup RTL — 2026-10-29

The pinned select's versioned RTL v3 layer and a reproducible three-file popup layer propagate direction to the search input and option rows. The popup patch verifier checks exact images, replay and rollback; all six related patch tests passed (log `target/headstart/evidence/select-popup-rtl-patch-tests-20261029.log`, SHA-256 `8ba8813b43d9af0b1afff5c6e66e3451c8f17fae7fd902354a95d0add0106de5`). RustUI ComboBox tests passed 12/12 (log `target/headstart/evidence/select-popup-rtl-combo-tests-20261029.log`, SHA-256 `ed6119ed60581e68280d6b3ba94cb29a80be30039622d9578db4bd8a26a68d7c`). On parent `db0246b2` plus the changes, the Web release driver passed with 18/18 host IME-plan tests and three unresolved notice gaps (log `target/headstart/evidence/select-popup-rtl-web-20261029.log`, SHA-256 `05d2dc9ec65b791b9550d73827ea14f9bf99252d866a0bd4f2b39e9c55de936a`; manifest SHA-256 `cd33aafebac452b4f8808601331a02782e0af6da9134245e1124e9afd17ec2c1`). In one in-app Chromium session, the Arabic search placeholder and query appeared right-aligned, options appeared on the right with the selected check at the left, and filtering and saving the plain-text option worked. Switching to English showed the left-aligned search and rows with the check at the right. The owned tab and server were closed. Native, real IME, other browsers and wider popup acceptance remain open.

The first source release run exposed two conflicts in the existing row-accessibility proposal checker, which expected the unpatched list image (log `target/headstart/evidence/select-popup-rtl-source-initial-failed-20261029.log`, SHA-256 `b86bb761c0b66e25508f5bb9b8c3d29e56883c071ec6d8d65693107623030c82`). The checker now verifies the complete RTL patch and reconstructs the proposal's original input only in a temporary directory. Ten focused patch and proposal-checker tests passed, including altered inputs and interrupted replacement (log `target/headstart/evidence/select-popup-rtl-integration-tests-20261029.log`, SHA-256 `9237e5badebc6da7a315f7892f001602466cb25650f19b1a58926cfd63a03822`). The source release suite then passed 628 Python tests and 45 Node tests, plus patch and packaging checks (log `target/headstart/evidence/select-popup-rtl-source-final-20261029.log`, SHA-256 `18f7152789b1a851aabdae7de7a1b0bf90bedbd045d960aef247bc08a117d01e`).

## Save dialog format search localization — 2026-10-29

The existing Save dialog ComboBox now supplies an English, French and Arabic format-search prompt through the public RustUI API. Locale validation passed and the focused Playground dialog-gallery tests passed 2/2 (log `target/headstart/evidence/dialog-format-search-test-20261029.log`, SHA-256 `9242c20f53c0f68b9b57bdeb60116e637a7dab327a5ef3a2608bfdda88934d52`). On parent `7e4bfa76` plus the change, the Web release driver passed with 18/18 host IME-plan tests and three unresolved notice gaps (log `target/headstart/evidence/dialog-format-search-web-20261029.log`, SHA-256 `7581c5554a3a40a9af9458a7c829ceabf3408ff8a74976a4cb2a074d3d57e59c`; manifest SHA-256 `ae79f2d479b3cfd97f9b57c57ca6c0b17f65d92aacaba0cdbb105a1a5dae0458`). In one in-app Chromium session the prompt appeared in each language; Arabic search for `عادي` filtered to the plain-text option and Save reported it. Popup rows and the search field still use left-oriented layout in Arabic; native and cross-browser acceptance remain open. The owned tab and local server were closed.

## Select trigger RTL — 2026-10-29

The versioned `gpui-component-select-rtl-alignment-v2.patch` replaces v1 over the same pinned package and adds RTL trigger ordering and title alignment while retaining LTR defaults. The replay verifier and its three tests passed, as did the committed-title patch fixture; RustUI ComboBox tests passed 12/12 (log `target/headstart/evidence/select-trigger-rtl-test-20261029.log`, SHA-256 `e0a4673a4e32192d0d5d3384fd0b9e28741db41e727f7286728bca1d2e29ffd0`). On parent `9f8a91dc` plus the patch, the Web release driver passed with 18/18 host IME-plan tests and three unresolved notice gaps (log `target/headstart/evidence/select-trigger-rtl-web-20261029.log`, SHA-256 `13433eb15fa5996b2658cc7cc7b4cbf7e05f038da56971b73685e5449ec09754`; manifest SHA-256 `543bfc92275c09b52f224e521c4c4d25402e15c0ba98e373378dd075feedbe1a`). In one in-app Chromium session, the English trigger remained left-aligned; Arabic selected text appeared at the right, selecting plain text updated the displayed selection and Save reported it. Arabic popup rows remained left-aligned and its search placeholder remained English; wider RTL, native and cross-browser acceptance remain open. The owned test tab and server were closed.

The source release driver also passed on the same candidate: 625 Python tests and 45 Node tests, plus its patch and packaging checks (log `target/headstart/evidence/select-trigger-rtl-source-20261029.log`, SHA-256 `babe219da9d524cf82ef3b68d6cfa07b181daea1673e97a1839d8e423bb9dea8`).

## Web dialog owner binding fix — 2026-10-29

Dialog content now renders bindings and actions against the original RustUI component owner rather than the retained dialog host. Retained content and its accessibility label are reconciled when the owner rebuilds the dialog. The focused owner-binding regression passed 1/1 (log `target/headstart/evidence/dialog-owner-regression-20261029.log`, SHA-256 `730922c0eae715d6a73a9bb6545a4ca4d0c696e4e2ea6a2543c4138d53168292`); dialog tests passed 21/21, including retained-content reconciliation (log `target/headstart/evidence/dialog-owner-suite-20261029.log`, SHA-256 `eecfa6ebff87b178fe71d39573eb908bddbf2dd435b5d97ece64242830ea750e`). On parent `8a4df831` plus this change, the Web release driver passed with 18/18 host IME-plan tests and three unresolved notice gaps (log `target/headstart/evidence/dialog-owner-web-20261029.log`, SHA-256 `ff039e1b6c178b960878a708d9f25cc863e6ad62ee8c9cd20867d0e67138fc6e`; manifest SHA-256 `d45c8bc3a21b6dc6fc17a92cc74d4189ee0f36bdd74d5b07b6f8b4be79220f6a`). In one in-app Chromium session on that bundle, typing `abc` changed Save As to `Untitledabc`, Save reported that name, and reopening retained it. The owned tab and server were closed. This fixes the observed browser automation failure; native input, real IME, cross-browser behavior and broader dialog acceptance remain open.

## Web dialog text-entry comparison — 2026-10-28

On the source-matched final dialog bundle (manifest SHA-256 `5e72153c5f3f7dc000e5131f681f4dfe02f5e0ed577e2f5052c38e20911c04b3`), a single in-app Chromium session used coordinate clicks and the same `typeText` input method on two fields. The ordinary catalog search accepted `dialog` and filtered its results. After opening Save dialog, clicking Save As focused `gpui-ime-owner`, but typing `abc` and pressing `x` left its value `Untitled`; Save reported `Saved Untitled as Rich Text Format.` This distinguishes the modal path from the ordinary field under this automation, but does not establish the cause or physical keyboard behavior. Investigate event routing and modal focus before changing code; real IME and native input remain unverified. The test tab and its local server were closed.

A second in-app Chromium session on the same bundle selected the `text_field` story and used coordinate click and `typeText` on its standalone RustUI `TextField("Name", ...)`. The field and its bound-value display both changed to `abc`. This strengthens the comparison: the same public control works outside the dialog in this bounded test, but it does not identify the modal failure's cause. The test tab and its local server were closed.

## Save dialog localization and RTL — 2026-10-28

The existing public-API Save dialog story now translates its trigger, title, form placeholders, option labels, actions and status into English, French and Arabic. Save stores a name/format snapshot, so later edits do not change the reported result; known format labels resolve in the active locale. The story's container, trigger and text-field placement follow layout direction. RustUI's dialog title reserves 32 physical-right pixels for the pinned Components close button, preventing the observed Arabic title collision; this applies to LTR titles as well. Locale validation passed; focused RustUI Dialog tests passed 21/21 (log `target/headstart/evidence/dialog-title-clearance-20261028.log`, SHA-256 `590c514eec914f728c6c64f1a97cb0b986f416288259cb803f55f1b9916ed4a2`) and Playground dialog tests passed 4/4 (log `target/headstart/evidence/dialog-locales-final-20261028.log`, SHA-256 `46df904f9f3a602198f5fde3c0053d639e0bb2a45ef5a868377839ab23326dbf`). On parent `809181b5` plus the changes, the Web release driver passed 18/18 IME-plan tests and generated the final bundle (log SHA-256 `f265d3c98814c2effa2706175a277f14e415b8a6b0de2ca325144c53d17eb8be`, manifest SHA-256 `5e72153c5f3f7dc000e5131f681f4dfe02f5e0ed577e2f5052c38e20911c04b3`); three Web notice gaps remain.

In one in-app Chromium session, French title, options, Save and Cancel appeared and produced translated statuses; an earlier Arabic candidate displayed translated Save and status. The final bundle showed the Arabic story and trigger on the right, the title clear of Close, and the text fields on the right; Save produced an Arabic status. A separate final-bundle English check showed the LTR dialog and Cancel. The Arabic ComboBox selected title remained left-aligned. Browser automation focused the modal name field but typing did not change it in either the localized candidate or the earlier `8916e3d9` bundle, so text entry is unverified and the cause remains to investigate; this observation alone does not establish a physical-keyboard defect. Real IME, wider RTL, native dialog and cross-browser acceptance remain open. Owned tabs and servers were closed; the rejected macOS bundle remains preserved.

## Web animation identity locale interaction — 2026-10-28

In one in-app Chromium session using the source-matched bundle with manifest SHA-256 `8916e3d96af6818c6b8be51ac8006284553eafff8bdeafed67f720bd062a21ea`, the animation identity story displayed Arabic labels and right-side controls/fields. Editing the spring A/B values to Alpha/Beta and updating the parent preserved both; hiding and restoring A preserved Alpha and Beta. Editing fade A/B to Gamma/Delta and invoking external replacement changed both A values while retaining Beta and Delta. Switching to French and then English changed the visible labels and preserved those field values; the screenshot showed French/English left-to-right placement. This is bounded single-browser interaction evidence; real IME, other browsers, native interaction on this localized revision, visual animation fidelity and full localization acceptance remain open. The owned browser tab and localhost server were closed; the retained macOS bundle and held slot are unchanged.

## Browser preview roundtrips — 2026-10-28

With the bundle described in the following entry (manifest SHA-256 `8916e3d96af6818c6b8be51ac8006284553eafff8bdeafed67f720bd062a21ea`), all 2,944 captured source file hashes matched clean commit `316126c1`. In one in-app Chromium session, the reference host reported PASS for both the compiled-preview and composition canonical roundtrips, including a stale-stamp error, disconnect and reconnect. Each profile was explicitly disconnected; the owned browser tab and temporary localhost server were closed. These are bounded protocol observations; they do not test the artificial empty-error reply in a browser, establish independent visual rendering, cross-browser behavior, full preview acceptance or licensing clearance. The macOS disposable bundle remains preserved and its GUI-slot claim remains held because the earlier cleanup command was rejected.

## Browser preview error handling — 2026-10-28

The reference preview client formerly treated an empty-string `error` as success because it tested truthiness; a focused regression reproduced a missing rejection. It now rejects every non-null error and refuses a handshake carrying any non-null error. The full transport suite passed 10/10, including empty-error request and handshake cases (log `target/headstart/evidence/preview-empty-error-20261028.log`, SHA-256 `286383c9c436eb3189a072aedd14d89268e0024e7da251cb364a540ef2c14866`). On parent `936b087a` plus the change, the locked/offline Web release driver passed, including 18/18 IME-plan tests (log `target/headstart/evidence/web-preview-errors-20261028.log`, SHA-256 `871cb46b4583dd74a87e6c013dfd4891290c034c8a7c80e3395f74fb07651425`; bundle manifest `8916e3d96af6818c6b8be51ac8006284553eafff8bdeafed67f720bd062a21ea`). Packaged `reference-client.js` and source SHA-256 both equal `0fd6c30add0e06f49f1b7b8bc05aebb6980aa5d8fb85510586ac66aea9d0b0ed`. Three Web notice gaps remain; no real-browser interaction was performed for this change.

## Core state and notice provenance — 2026-10-28

On clean commit `e00096f0`, locked/offline `cargo-headstart test -p rustui-core --lib` passed 9/9 core library tests, including binding owner-type rejection and stale-context update behavior. Log `target/headstart/evidence/core-state-e00096f0-20261028.log`, SHA-256 `fbdeddf34ba901338f8c83042d8ad2ba01eaec6b8b375b0f4de62d2699c77e6a`. This source-level evidence does not close the broader owner lifecycle, invalidation or native acceptance requirements.

The exact upstream root directory listings for `seahash` revision `94b632aeac099031c373599313d5b5f0acbbaec0` and `hexf` revision `4225763d744183d720f575ae96d04161b4d08ea0` contain no root LICENSE, COPYING or NOTICE file. The saved API responses are `target/headstart/evidence/seahash-upstream-root-20261028.json` (SHA-256 `4dbe2460db91bd3a4e7e7adb4d597d74331457bcab2e37c33b813169900b8ee0`) and `target/headstart/evidence/hexf-upstream-root-20261028.json` (SHA-256 `204e113907934b8c961947f7a837f52ced75ef9cfe7924e7ccc8378d52de73ef`). These listings do not establish absence of license text elsewhere or complete licensing obligations. Both notice gaps remain open.

## Identity field locale completion and CI runner status — 2026-10-28

Animation identity field headings and placeholders now resolve in English, French and Arabic through the existing public TextField localization API; editable values and stable IDs are preserved. Three locale groups validate; the locked/offline Headstart Playground catalog-filtered suite passed 58/58 (log `target/headstart/evidence/identity-field-locales-20261028.log`, SHA-256 `f9568dbbdc4523aefcd06a4316df74f52031c8e08db572edaf2beb632b7f52f0`), and a focused locale value/label regression passed 1/1 (log `target/headstart/evidence/identity-field-labels-20261028.log`, SHA-256 `e273b86988772200e08b59ef9f14fa569c6f0f58be702a4febf8421e36646558`). Native and Web rendering and interaction remain unverified for this change.

GitHub reported zero repository self-hosted runners. Push run 40786915175 for `cdea9287` skipped the `macos-and-web-builds` job; the workflow dispatch job requires a trusted self-hosted Mac and invokes the macOS gate. The protected-bundle rejection remains unresolved, so dispatch was not attempted. This does not rule out organization-level runners and does not close compiled CI.

## Current Web build — 2026-10-28

On clean commit `3788e196`, `RUSTUI_WASM_HEADSTART_ROOT=/Users/Abium/Developer/rustui/target/headstart/wasm-root RUSTUI_WEB_OUTPUT_NAME=playground-web-identity-locales-20261028 ./scripts/verify-release.sh web` exited 0. The selected patch stack verified, 18/18 host IME-plan tests passed, and the optimized Playground bundle was generated in the fresh `target/headstart/playground-web-identity-locales-20261028` directory. Log SHA-256 `185f445ae4520fdaa9b8ba7f2932be713c2a1cdc2b039fd549edbdee034d2208`; bundle manifest SHA-256 `7508910a2d94fb9c3318d23991260cfb56af3f51c7d828df1571d84665807235`. The notice collector observed 408 packages and three unresolved gaps. This is build and bounded host-test evidence, not browser interaction, real IME, compiled CI or license clearance. The held GUI slot and protected-bundle incident remain open.

## Current source release gate — 2026-10-28

On clean commit `8b065450`, `./scripts/verify-release.sh source` exited 0. It ran 625 Python tests and three Node suites with 20, 8 and 15 passing tests, plus the configured patch-stack verifiers, packaging and Web build fixtures, locale validation, planning-theme validation and theme adapter coverage. Log: `target/headstart/evidence/source-gate-8b065450-20261028.log`, SHA-256 `baa82041bde4a9b16a3bcc3cbe5be7a68d05d6ebfb7d4996282ebb4d16f4c939`. The source gate does not establish compiled macOS/Web CI, native/browser interaction, license clearance, signing or release readiness; the GUI cleanup rejection and existing protected-bundle incident remain open.

## Animation identity localization — 2026-10-28

A focused runtime formatting regression now verifies that the animation identity story resolves localized mirror labels, both field values, and the locale-formatted update count in English, French and Arabic. It passed 1/1 via locked offline Headstart on the current source (log `target/headstart/evidence/identity-message-format-20261028.log`, SHA-256 `21ac0c1a9c994ad9ba29a57b58802a684c176809960a4d252a79781fcbc8c50b`). This does not establish rendered layout or interaction.


The existing animation child-identity story now localizes its instructions, action labels, visibility status, omitted-field messages and value mirrors in English, French and Arabic; the existing field identities, initial values and actions are unchanged. A locked offline macOS Headstart Playground check passed (log `target/headstart/evidence/identity-locales-check-20261028.log`, SHA-256 `d84eb101257dc30fb969bb43ac5d9f48403fc16d25b63a887ba77951c9a59a46`), all three catalog groups passed `validate_locales.py`, and the Playground library catalog-filtered suite passed 58/58 (log `target/headstart/evidence/identity-locales-catalog-tests-20261028.log`, SHA-256 `e4691efbe218eaa7b1f1bf9c31a23fd1afee5c2fc5a82a938becba7effd8d29d`). An initial test exposed invalid interpolation syntax; corrected messages passed the runtime catalog checks. Native and Web rendering, RTL layout and interaction for this new change remain unverified. The disposable GUI bundle cleanup rejection and held GUI slot are recorded below.

## Native motion and locale observation — 2026-10-28

On commit `db0d7b48`, the isolated macOS Playground bundle executable and release executable both had SHA-256 `8a0ca06c99b1d71f1308e38aab3543331d8a259a7eca2628f6e02d49a2e94632`. In one native session, the English fade restart advanced generation 0 to 1; the spring target changed 1 to 0.15, activity 0 to 1, and remount identity 0 to 1. French labels appeared and changing the spring target restored 1. Arabic labels and digits appeared; its spring target changed 1 to 0.15, activity 1 to 2, and remount identity 1 to 2. The Arabic spring sample and controls appeared on the right. These observations do not establish motion fidelity, reduced-motion behavior, keyboard operation, or full native/locale acceptance. The app quit and no Playground process remained. Automatic approval rejected the command to remove the disposable `target/headstart/native-validation/RustUI Playground.app` because it used `rm -rf`; the bundle remains and the GUI slot remains reserved pending authorized cleanup. The separate protected-bundle incident remains open.

## Current checkpoint — 2026-10-28: Web link opener isolation

- **R30 isolated macOS packaging:** `package-app.sh` accepts `RUSTUI_PLAYGROUND_BUNDLE_PATH` for an explicitly fresh bundle while continuing to use the persistent Cargo artifact directory. It atomically refuses an already-existing explicit destination. The packaging fixture passed, including preservation of the default bundle and rejection of a second use of the explicit path. On parent `dc2a8e8a` plus this packaging change, a locked offline remapped macOS release build and direct isolated packaging succeeded; the build and bundle executable hashes both equal `8a0ca06c99b1d71f1308e38aab3543331d8a259a7eca2628f6e02d49a2e94632`. The generated bundle is `target/headstart/isolated-packaging-20261028/RustUI Playground.app`; the collector observed 446 packages and five missing native notice versions (report SHA-256 `b54108cf8f69b27bff145e190964b961d1e13c4e0d6a889e756baf79e18ada9a`). The two previously protected bundle executable hashes were unchanged. This direct isolated build/package is not the prohibited macOS release gate, a GUI interaction, signing, licensing clearance or release acceptance; the protected-bundle incident remains open.

- **R24 node-count preflight:** Composition encoding now rejects more than 4,096 nodes before scanning text or cloning the graph. For multiply invalid oversized input, the node-limit error can precede other validation errors. The preview suite passed 50/50, and the focused regression passed with both excess nodes and oversized text (log SHA-256 `fab5c70acf1dbb01db85243d3eeb9e24237f25b9a6b7fc5651801af21f1dca42`). The Web release build passed before the test-only regression was extended (log `00aff5645f73437d3dd30dd40c0c40b139af331d24b86644b903f902f5efacbb`); this does not bound cloning of a valid-size graph or establish runtime acceptance.
- **R10/R30 integration checkpoint:** On clean commit `9f0f9de1`, locked offline Headstart workspace integration tests passed 60/60 across 34 suites (log `target/headstart/evidence/workspace-integration-9f0f9de1-20261028.log`, SHA-256 `bf4a8e6b2b248a4de34b6b9045c3f5602f4329fab8f883b537c4d3bd9f5ea227`). This is source-level integration evidence, not native UI, compiled CI or release acceptance.

- **R22/R24 pre-clone text budget:** Preview and composition encoders now sum serialized property-string UTF-8 bytes before cloning; if text alone exceeds 1 MiB, the document cannot fit in the existing output limit and is rejected immediately. Escaping can only increase the JSON size; the bounded writer still checks complete output. For multiply invalid oversized input the size error may now precede schema errors. The preview suite passed 50/50, including cumulative multibyte boundaries and a schema-valid document whose property text exceeds the cap (log SHA-256 `cb74b763380e562bf82ea84281fdcd5aa1d4cb0a3547896c3af485314390f425`). The Web release build passed with 18 IME-plan tests (log `173729c028d15e10bd8f286276a08263018817e37a7889d028b687b39aa27ffb`; manifest `4bb1f96af589090b8139c0525480cde1a3b60fef3bf6f8c543ec61758d093b44`). This is a conservative string lower bound, not a bound on all graph validation or cloning costs.

- **R24 current-source Web composition:** On clean commit `6204772e3134dc38362325e0b3e8b6dee3e51358`, the Web release build passed (log SHA-256 `a87399c12a6857f4290b51488d1b650e5a2d55d6ccc2dc84c444b58417e52938`; manifest `2ed08b973346aa1bcfe7c287a9326259fe15ea483a40a07c8cc5b10920f31bfc`). All 2,944 captured source hashes matched the checkout. In one in-app browser session the reference host's composition roundtrip passed edits, export, stale-stamp rejection, disconnect and reconnect. Separately, mounting a column and inserting a child rendered `Current composition`; editing its label advanced the revision to 2 and visibly rendered `Updated composition`. The host was disconnected and unmounted, its owned tab and server closed, and the GUI slot released. This is bounded browser evidence, not broader cross-browser, native, accessibility or performance acceptance.

- **R24/R27 composition validation:** Composition validation previously revalidated the same immutable catalog for every node. It now validates the catalog on the first node and validates each subsequent document against that same catalog, preserving per-node checks and first-node error ordering. A temporary Headstart-profile probe with 513 nodes and 67 schemas measured seven warmed samples: median 9,679 µs before and 1,803 µs after, on this machine and workload only (probe source SHA-256 `e0473063eb5cadd684e3a7a1ab1ad61282185151e318c0252ed4380ee6e24841`; before/after logs `47d01995f3150cf6cab26baa8340b00064a5c2d9b4d6a125137bd589d0fe62ec` and `8405f753bfb1467228ef8f98302f6e21c6f7df87029124951af363e239113d1c`). The temporary probe was removed; a regression checks that an invalid later node still fails. The preview suite passed 48/48 (log `c1eff439eb6078561654c39bd197ac24c76a5f1c6845af84db596c3ef1445384`). A first Web package correctly rejected promotion because a source comment changed during its build; the stable-source rerun passed, including 18 IME-plan tests (log `cd4c6f17054110bf26a26917b98cbc11d6bddbb60ab359f7b810330ef803b4d3`; manifest `e5c7b5d5e3d392d5347f2ce4ac9e8948ad1ccefa9dbeb2a7ad777e29826a6ab1`). This is not an application-level performance benchmark, runtime composition acceptance or a complete resource bound.

- **R22/R30 integration checkpoint:** On clean commit `15ca30c4`, locked offline Headstart workspace library tests passed 1,372 tests in five nonempty suites; one additional suite had zero tests (log `target/headstart/evidence/workspace-lib-15ca30c4-20261028.log`, SHA-256 `8dfb5e559926c17c893e5f6c94d1937fad7568c41dcbaf52dd9e8e1e81ac0260`). GitHub Actions run 40781912789 passed the source job on the same commit; the macOS/Web job was skipped. The preview encoder now reuses the validated document instead of cloning it a second time; the affected `preview::` tests passed 47/47 on that change (log SHA-256 `ad1b33e5733da9384256ce006f943db2ef8d3c38463d46197e864aa847aae3dd`). Composition still clones and validates its graph, and runtime, compiled CI and release acceptance remain open.

- **R22/R24 bounded JSON output:** Preview/catalog and composition serialization previously built the full JSON string before enforcing the 1 MiB limit. They now write through a bounded buffer and stop serialization on overflow while preserving their respective size errors and pretty JSON. Boundary regressions cover exact-size ASCII output, oversized ASCII and multibyte UTF-8, and early termination of a large sequence. The macOS Headstart preview suite passed 46/46 before the added early-stop test; both bounded-output tests then passed (logs SHA-256 `527ddcd649d0e67cef9486829b00edab8af3c6738ed3c44e58603a9f903c4c7c` and `5d8ee253e15a8e2e5f008c69e8b5458c57b5933b21b52adcffbfee387ea40df3`). The Web release build passed with 18 IME-plan tests (log SHA-256 `a3482bf6d5409906860331f1e2ce6574a50a3b6f92d189985096658497adb5fd`; manifest `4b1d549fbc459ce149103a1cfa2dcf10e9dfcfc5e8d7d9e4fd7b2b00a39ebb1f`). This bounds encoded output buffering only; document clones, validation, transport runtime and overall resource limits remain separately open.

- **R13 animation and spring story locales:** Existing motion story labels, instructions and numeric status now use English, French and Arabic catalogs; animation and spring sample surfaces follow layout direction. Locale metadata validation, three catalog initialization tests and the final locked offline macOS Headstart Playground check passed (check log SHA-256 `e34e7a1751c4bb6b670579ec17d54bfe55fb5a99e4d7e8d28025b50a3f1c2e80`; catalog log `46eaaf31907ba5b17a75f18f87dda875d5846f21a8f43a5fdc736458ded5196f`). A first Web build displayed French labels and confirmed fade restart increments its generation and spring target changes to 0.15. It exposed left-aligned Arabic sample surfaces; after direction-aware wrapping, the final Web build displayed them on the right and Arabic spring activity, target and remount controls updated their states. The final Web build passed including 18 IME-plan tests (log SHA-256 `bb63e61fe53c8243921151f2f09967433dc52f647727680e86304f04de91e11c`; manifest `a9ef1aeede3185f966969460c818a70cd5f714e50e010d729bf644784c47eabe`). This is bounded Web observation, not native motion, reduced-motion, keyboard, cross-browser or wider locale acceptance. The owned tab and server were closed.

- **R22/R30 current consumer and preview checks:** On clean source `d54b5964`, the locked offline Headstart check of all targets in `examples/external-consumer` passed (log SHA-256 `b778a3bf6557639ce9db55d308f8567318976e17fdd7fc83daccc4f726535635`). The RustUI library's `preview::` test filter passed 45/45 (log SHA-256 `f058466b5681892c806d93fc6e3d1dd658da8dfe557c8f1bc6757fe92639a9d0`). Logs are under `target/headstart/evidence/`. These establish compilation of that consumer and the exercised preview source contracts only; runtime, full schema review, packaging and release acceptance remain open.

- **R03 public API documentation:** Added a consumer-facing counter example showing `#[component]`, owned state, an action and startup theme activation. The current RustUI crate documentation built with broken intra-doc links and invalid code-block attributes denied; both crate doctests passed, including the new compile-only application example (doc log SHA-256 `85339e79c5cda2a2f904a691b601739c7981d71445ffa860a20fa8ccec9a1535`; doctest log SHA-256 `262c9895074dc432e1c90bbf8e0b5ac2ed253f5ee56833ba087b511b049d4ffb`, under `target/headstart/evidence/`). The example was not launched, and this does not complete the public API or documentation audit.

- **R13 focus gallery RTL:** The Arabic focus gallery had right-aligned labels but left-aligned fields and remove/restore controls. Direction-aware RustUI `HStack` wrappers now align these controls with the active layout direction. On a Web build of parent `88870cbb` plus this change, Arabic scope A/B fields, outside field and remove/restore actions appeared at the right; typing in A, incrementing its child, removing and restoring the scope retained the bound text and reset the child counter. Switching to English placed the same fields and controls at the left. The Web release build and 18 IME-plan tests passed (log SHA-256 `473c14df972d4f2765aad7a56bf2093d6894faf0b80763d5f5f4abccd486702c`; manifest `7f726d5d9b77f999ac919a51aee648f4478343e343a19805ec6543dace42d737`). This is bounded Web visual and pointer/typing evidence; native, keyboard scope behavior, other sizes and broader locale acceptance remain open. The owned tab and server were closed.

- **R08 native animation scheduling probes:** On parent `ff001d6b`, all four opacity probe runs initially aborted because the example did not activate the RustUI component theme (failure report `target/headstart/opacity-probe-verification-965-1793226043995251000.json`, SHA-256 `e1e58006920c65b3a32ada272a4d78bd3b6f5b4375caf38e688f70b3357eba00`). The opacity and spring probe startup now activates the bundled theme. With the rebuilt executable (SHA-256 `b0459aa7222801e11e8ec0f80f5d563e1a1e58ff076ccdd8f82c2ecfacd2d392`), the native opacity verifier completed normal, reduced-motion, limited and limited reduced-motion runs; normal versus 10-FPS-limited host lowering deltas were 67 versus 10 (report `target/headstart/opacity-probe-verification-1351-1793226117200008000.json`, SHA-256 `f6fed51db31037c98ccf8e57bc22a833be15ac2d64e1e8d8976224d8fa7e177d`). The native SpringOpacity verifier passed normal and reduced-motion runs (report `target/headstart/spring-opacity-verification-1738-1793226152439741000.json`, SHA-256 `dab7f1052c09ed04feaddbc131a484984a68a246f32391b110f75af6e497975e`). These measure isolated host scheduling, not displayed frames, motion fidelity or all animation acceptance. The owned executable was removed after confirming no probe process remained.

- **R26 Playground bundled theme reuse:** Startup and the Card and Decoration gallery appearance actions now share one parsed immutable copy of the embedded macOS theme instead of parsing it again for each action. They still activate the bundled theme, including after another theme has been selected. A focused macOS Headstart test confirms repeated requests share the document (1/1; `target/headstart/evidence/bundled-theme-cache-20261028.log`, SHA-256 `ffbe865a9d5142fdadd061f2c72d989fc209e5ca900a5e4d02006da473520da0`). The Web release build and its 18 IME-plan tests passed on parent `a848122d` plus this change (log SHA-256 `622a9d2e834159ad0c92b02a543fce30faf457817ea2570bfc9eb418efa6f8fd`; manifest `f36f49498d18f70d9395750422f4f7430e2ea9dc085b9ab30cdac0119570179c`); three Web notice gaps remain. This removes repeat parsing in those paths but does not measure activation latency, first-load latency or application performance acceptance. Native and wider browser appearance interaction remains to be checked on this candidate.

- **R26 bounded Web appearance observation:** On that Web bundle, the Card story's light action visibly changed the Playground to light and its dark action changed it back to dark. The owned browser tab and local server were closed. Native appearance, the Decoration action, retained state under theme changes and performance budgets remain open.

- **R13 keyboard story RTL and Web observation:** In a current-source Web session, the French keyboard story displayed translated instructions and controls; its primary button advanced the count from 0 to 1 and the optional button changed to its enable label. Arabic text was right-aligned while fields and actions were left-aligned. Direction-aware RustUI `HStack` wrappers corrected the keyboard story controls: in the rebuilt Web bundle the Arabic fields and actions appeared at the right, the primary action advanced the Arabic count, and the optional button switched to its enable label. The Web release build passed on parent `3a3d24f9` plus this change (log `target/headstart/evidence/keyboard-locale-rtl-web-build-20261028.log`, SHA-256 `eab652b0f5dad9100101ff60481f6af5adae2f592298fb1fe943dff4282fb281`; manifest `935c9b0c937abbfa939fe2ae398919a2c1fb2f88518e361ce8f9d426355052d4`). These are bounded pointer and visual observations; shortcut key dispatch, native, other viewport and broader locale acceptance remain open. The owned browser tab and server were closed.

- **R13 keyboard shortcut story:** The Playground's existing shortcut instructions, field placeholders, action labels and numeric counts now use English, French and Arabic catalogs; shortcut bindings are unchanged. The locked offline Headstart Playground check, locale validator, formatting check and three catalog initialization tests passed (test log `target/headstart/evidence/keyboard-story-catalog-20261028.log`, SHA-256 `6281972d67ff36ddf6f1ba4dd2374e11a1c1a67543488ad556423bd4bc05948b`). Native and browser presentation, RTL and interaction acceptance remain open.
- **R30 source CI:** GitHub Actions run 40776816866 passed its source job on `c928c27f93ca03e66f5a1d13990dd5e00eeabbc4`; the macOS/Web build job was skipped on this push. This is not compiled CI or release acceptance.

- **R13/R15 calendar heading source gate:** The current heading candidate passed `./scripts/verify-release.sh source` with an explicitly recorded zero exit status: 625 Python tests and the JavaScript and other source checks passed. Log `target/headstart/evidence/calendar-heading-source-gate-recheck-20261028.log`, SHA-256 `1f37ad5c66767b755428ad30ebca57f650ca31354e2a3bc79b1fefc2f2aa5c8a`. This does not close native, browser, or release acceptance.

- **R30 `block` FFI candidate verification:** `scripts/verify_block_stack_symbol_candidate.py` verifies the exact pristine source, upstream checksum, candidate patch and fixture hashes, then applies, reverses and reapplies the patch in a temporary copy. On macOS, the installed Headstart compiler accepted it with `-D uninhabited_static`; a C/Rust probe exercised a C call to a Rust stack block and copied block, a Rust call to a C-created block, and one-time captured-value destruction. The verifier passed (log `target/headstart/evidence/block-ffi-verifier-20261028.log`, SHA-256 `b8c8db0059e689349606fe5dc06172a7481503a2789eb4997e29c712ce51524a`); a separate integrity test rejected altered source and patch inputs. At that earlier checkpoint the candidate was unpromoted; the later R30 entry records its integration and validation. The distinct dependency notice gap remains.

- **R13/R15 two-month calendar:** The public RustUI two-month DatePicker is now demonstrated in Playground with English, French and Arabic labels and shares its date with the one-month example. In the initial native and Web Arabic checks, October appeared right of November and selecting November 1 updated both fields; switching to English retained the date. Those observations exposed month headings shifted toward the center. The `gpui-calendar-multi-month-heading-v1.patch` centers each heading above its grid and keeps navigation arrows at the outside edges. On the corrected macOS and Web builds, English and Arabic headings aligned with their grids and the next-month arrow advanced them. A separate Web observation showed the French two-month label, French month names and Monday-first weekdays. The focused layout test and 14 patch replay/tamper tests passed; locale validation and formatting passed. Native/Web release builds passed (logs `target/headstart/evidence/calendar-heading-native-build-20261028.log` SHA-256 `aeb6d064e90cbd56f06f80ba496d578321c58a81873364ed6843dec112ad1abd`, and `target/headstart/evidence/calendar-heading-web-build-20261028.log` SHA-256 `1adc24164b651f92a7c90c3d3e6c9a220fc6ae4887fd102219d7933bac5ab3c2`; Web manifest `9336204617166f91176ae05e2240ee8232f1e53f1efc4921b93e1262859406d0`). The native bundle matched executable SHA-256 `ced74ec624b53c3ae7f0a7a0ccae91c6ebc2f6361798c5a215a514f0f244440a`; the owned app, bundle, tab and server were closed, and the GUI slot released. These are bounded observations; narrow windows, broader calendar interactions and release acceptance remain open.

- **R13/R15 Arabic calendar RTL:** A reproducible three-file patch mirrors weekday and day rows, chronological multi-month order, month/year grids and navigation chevrons, with Left moving to the next day in RTL. A GPUI test-window check asserted positions for both directions in day, month, year and two-month layouts (log `target/headstart/evidence/calendar-rtl-layout-20261028.log`, SHA-256 `c84f658794d6b72709a8930f8a1353ae45826548dec221e834b9c4fb92d2071d`); the focused DatePicker suite passed 26/26 (log `target/headstart/evidence/calendar-rtl-date-picker-suite-20261028.log`), and 12 composed patch tests passed. Locked offline native and Web builds passed (logs SHA-256 `41d7acf78fa93f29339748c7287477018a96b9455208cb78ae99f5ebfada5753` and `93982aa6465aa2e94b6caa6766333d72e428ae8d4a730e9fa091158b1af5cdfc`; Web manifest `0137b0147e03123b3bbc10253a957688a052623a49265d187ebdb571c7109c8e`). A source-bound macOS bundle (executable SHA-256 `2312b16711b9d989d9296be6fb8f5cfd4e9bffddcad67d2014d42719f829bcd6`) showed mirrored day, month and year views, and Left then Enter selected October 29; one Web session showed the RTL day view and the same selection. The source release gate passed 623 Python tests, 20 semantic DOM tests and accompanying checks (log `target/headstart/evidence/calendar-rtl-source-gate-20261028.log`, SHA-256 `a25b6615dad86c6c777ad1c20a0e318dd76fc1f950687f415b50cd28e1fa23be`). The owned app, disposable bundle, browser tab and server were closed; GUI slot released. This is bounded evidence; broader month/year interactions, multi-month runtime, locales, keyboard and platform acceptance remain open.

- **R15 DatePicker keyboard defect (bounded fix integrated):** On the patched macOS 27 Playground, Enter opened the focused date picker, Right did not move the displayed day, and Enter closed the calendar without selecting a date. A new GPUI test-window regression on `20de1e9c` starts with 28 October 2026 and sends Enter → Right → Enter; it currently fails because the owner remains at October 28 rather than October 29 (log `target/headstart/evidence/date-picker-keyboard-before-20261028.log`, SHA-256 `f4f62066da01d99159abf3be24c6677b26875726092a9e87a4ff5ae78f8cf172`). The patch integrated in `79402434` intercepts Confirm while a keyboard cursor exists; the focused GPUI regression now passes, including no write before confirmation and Escape cancellation (log `target/headstart/evidence/date-picker-keyboard-cancel-20261028.log`). After cursor cleanup on closing, the DatePicker source suite passed 22/22 (log `target/headstart/evidence/date-picker-keyboard-suite-20261028.log`). The two-file patch verifier passed replay, source, patch and checksum tamper rejection, and simulated second-file replacement rollback (3 Python tests); interruption or rollback failure can still leave a mixed image and must be recovered from the recorded exact images. The composed stack verifier now checks both exact keyboard postimages and replays earlier date-limit, localization and RTL layers in an isolated copy; the source release gate passed 617 Python tests and its remaining checks (log `target/headstart/evidence/calendar-keyboard-source-gate-20261028.log`, SHA-256 `58e84c19d083b8c9522f0d9971c6ff4bf4e11729d5747e8acbaf4255967963b0`). The focused keyboard test also passed navigation beyond both NaiveDate boundaries without changing the bound value (log `target/headstart/evidence/date-picker-keyboard-boundaries-20261028.log`). The keyboard handler was subsequently changed to follow the actual left-to-right grid in both layout directions until visual RTL is implemented. Two focused GPUI keyboard tests now pass, covering cancellation, boundary overflow, refusing a disabled day, moving beyond it, and the RTL-configured picker (log `target/headstart/evidence/date-picker-keyboard-disabled-rtl-20261028.log`). The composed verifier and its nine focused tests passed again on that later patch image (log `target/headstart/evidence/calendar-keyboard-stack-20261028.log`); the full source-gate result above predates this change. Focused GPUI tests also passed for canceling and confirming a range (`target/headstart/evidence/date-picker-keyboard-range-20261028.log`) and preserving the time while choosing a date (`target/headstart/evidence/date-picker-keyboard-time-20261028.log`). On the current candidate, the locked offline macOS release build passed (log `target/headstart/evidence/calendar-keyboard-native-build-20261028.log`, SHA-256 `8586e931f46e9417d7c36a8989f482a7ba291b822a1e93bb595586a5ca07b00e`) and the Web release driver passed, including 18 IME-plan tests (log `target/headstart/evidence/calendar-keyboard-web-build-20261028.log`, SHA-256 `fdd9b27dfc0eb44958ffbbc164b46113ff998f548ff243d052a86735f2bd11eb`; bundle manifest SHA-256 `dd9e03771afb81e3f316cff68c07fb6595aa9f5ce298cc9e533f2d808a994426`). Web packaging still records three dependency-notice gaps. In one current-bundle in-app browser interaction, the English calendar opened by pointer on 28 October; Right visibly highlighted 29, but both Return and Enter left the popup open with no selected trigger value. The cause was the semantic mirror treating its combo-box HTML button as a native button for Enter; the experimental `gpui-web-combobox-enter-v1.patch` forwards that key to GPUI while retaining ordinary button handling. On the rebuilt bundle, one in-app browser session opened the calendar by pointer, moved the highlight from October 28 to 29 with Right, and confirmed with Enter; the popup closed and the field visibly showed 10/29/26. A separate browser observation confirmed Enter opens the focused closed DatePicker after Escape. The owned tabs and local servers were closed and the GUI slot released. The Web driver passed (log `target/headstart/evidence/web-combobox-enter-build-20261028.log`, SHA-256 `4798312dd389d0997e23a81b2198d46477133805e0bab03ef67068208d8e5699`; manifest `b3da1cd956ed1205dd3c3ba28a4ca200e8112fef016276431bc88a804d9f4359`); semantic DOM tests passed 20/20 and patch/semantic stack tests passed 3/3 and 7/7. The refreshed source release gate passed 620 Python tests, 20 semantic DOM tests and the remaining source checks (log `target/headstart/evidence/web-combobox-source-gate-20261028.log`, SHA-256 `48b90c24d3b1311a2147576ac424bce21f18c88bdc06f0f8d5694705333260fa`). This is bounded interaction evidence, not complete Web acceptance. On macOS 27, an owned disposable bundle whose executable SHA-256 matched the native build (`c472be93aa0938b5ca1c30796e50c0e84efa9e1c1fdb21f66c1db880358bda12`) opened by pointer, moved the highlight from October 28 to 29 with Right, and confirmed 10/29/26 with Enter; Enter reopened the focused picker and Escape after Right preserved 10/29/26. The owned app exited, its disposable bundle was removed, and the GUI slot was released. This is bounded native evidence, not comprehensive DatePicker acceptance. The mirrored RTL layout is integrated; broader keyboard and locale acceptance remain open. Before the RTL implementation above, the calendar rendered Arabic weekdays in physical left-to-right order. The owned app exited and its disposable bundle was removed.

- **R13/R15 Picker and ComboBox popup alignment:** RustUI now forwards its layout direction to the shared Components Select; the approved `gpui-component-select-rtl-alignment-v1.patch` layers on the committed-title patch and keeps LTR as the default. The Playground ComboBox uses a 300-pixel menu on its 220-pixel trigger to make alignment observable. Composed replay/tamper and prior-layer tests passed 4/4; the RustUI input suite passed 209/209 (log SHA-256 `e27f5b9fa19cd21afe595e2a9820ee9035df9742b7642a1c5b88092e10c41d4b`). The source gate passed 614 Python tests and accompanying checks (log `18bab98b200edf48453ebc98284f03415de465bf6ad928a5a2f14f9e5bdf9d68`). Locked offline native and optimized Web builds passed on uncommitted changes based on `9e8d7460` (logs `2fcb045eeec3f567706b197c116a42b41ebfb2adc7211260e3a261d16fa15492` and `e8ae9a93fa0cdfe83866fec5b02c2b4b03001f4df5dea7d56adf21302bf9e22a`; Web manifest `58df871714d89f10af41e7d13c33e79f72ccc3c9c71cc4972bbf1424d2e6a583`). In macOS 27 and one 1280×720 Chromium session, the wider ComboBox menu aligned right in Arabic and left in English; selecting AppKit in Arabic survived the switch. The native bundle executable matched the build SHA-256 `ddecf14beb7620626f59d24554c11221db1fcb54c0963445ec0b79f4394c1159`. Owned test processes and disposable resources were removed and the GUI slot released. Picker uses the same direction path, but its separate native/browser interaction, keyboard, narrow-window and accessibility acceptance remain open.

- **R13/R15 DatePicker popup alignment:** RustUI passes its active layout direction to the pinned Components DatePicker. The approved `gpui-component-date-picker-rtl-alignment-v1.patch` layers on the existing localized DatePicker patch and aligns the popup to the right in RTL while retaining physical left alignment in LTR. The source release gate passed 611 Python tests and accompanying checks (log SHA-256 `fff7f26414b7952d60a2931efac58df1a94188aea31e7b9b2a976d408701b048`). The composed patch replay, reverse, tamper and previous-layer tests passed 4/4; RustUI DatePicker tests passed 21/21 (log SHA-256 `9e20c463e9b36cfde85acd8150d9f19a3207b670749587cd8d85aab210cb726b`). Locked offline native and optimized Web builds passed on uncommitted changes based on `91190b74` (logs `1983a5fdf4fb7343dabcc3dea71fdb66a39654c40314a2aaa4697db146c08425` and `84ec07899b2bfe5b14b3faf4770db8cb159fb9d2c0a6fb6af3fdb35818af5ae9`; Web manifest `beca0ba7b79bcf1e69f6d84731dbc7aafd6719c0d318b9886e08a57c1080338a`). On macOS 27 and one 1280×720 Chromium session, the Arabic popup aligned to the trigger's right edge and the English popup to its left; selecting 28 October in Arabic and switching to English retained the date. The owned app, bundle, browser tab and server were closed; the GUI slot was released. This is bounded functional and visual evidence, not comprehensive keyboard, narrow-window, cross-browser, or accessibility acceptance.

- **R13/R15 DatePicker RTL popup candidate:** The native Arabic calendar appeared at the physical left of its wide trigger. The Components DatePicker calls the shared dropdown positioner, whose `Align::Start` resolves to the trigger's left edge; it does not consult text direction. `patches/proposed/gpui-component-date-picker-rtl-alignment.patch` (SHA-256 `70fd646f76e0e4e2a279c7c914af0e29302cd7d5d3c52affb24b1a1a5199aa15`) prepares a DatePicker-only optional logical-start alignment while retaining the default LTR path. `git apply --check` passed against the existing localized DatePicker postimage `83e639bbaf3194f81c33864f8e885f683faa403154e40253454215ed8462467d`. At that checkpoint it was not applied; the subsequent implementation and validation are recorded above. No other dropdown is modified by the patch.

- **R13/R15 native DatePicker locale check:** In the macOS 27 Playground, the release executable built with the patched source on parent `d279eafa` (SHA-256 `67fb75777a26bbe1e1724f0f8b22e788adc731b4bcb6e7d37355320ea1053dbd`, identical to the disposable bundle) was exercised with current product source unchanged since `293ae112`. In Arabic, the calendar displayed October 2026 with a Saturday-first row; selecting 28 produced a complete visible date `2026/10/28` and an accessibility value `28\u200f/10\u200f/2026`. Switching to French retained the selection and displayed `28/10/2026`; English displayed `10/28/26`. The Arabic calendar grid and date used Latin digits in this `ar` locale, and the popup appeared at the left end of the wide trigger; this observation does not establish whether alternate numbering or popup alignment is required. No full keyboard, narrow-window, or broader locale acceptance is claimed. The owned app exited, its disposable bundle was removed, and the GUI slot was released.

- **R30 standalone consumer native smoke:** On commit `89ba8aa7`, the locked/offline optimized Headstart macOS build succeeded (log `target/headstart/evidence/consumer-theme-native-build-20261028.log`, SHA-256 `0c3b0346677725e2feb1544216b64c8709437e5d48f166c8b849f91b54708a64`). The owned disposable bundle executable matched build SHA-256 `0f3a6d4f8b9ca9d69db53f68555635fc6cc81d4d09e582410a61fa10f1813877`. On macOS 27 the consumer displayed `Count: 0`, and Increment changed it to `Count: 1`. Its process exited and disposable bundle was removed; the GUI slot was released. This is bounded native smoke evidence, not comprehensive native or release acceptance. The public localization startup example now also activates the caller-selected theme before rendering.

- **R28/R30 standalone consumer theme startup:** In an in-app Chromium test of the source consumer at `5a6369d0`, the Wasm loaded but the component render panicked because no RustUI theme was active. The example now parses the bundled macOS 27 theme and activates it in `on_launch` before rendering, using the public API; the source-consumption guide explains its include path. The locked/offline optimized Wasm build passed (log SHA-256 `3acb88467fa9308bedcad115f22aaf976503ab7a1992006b0a102a994e180b05`, Wasm SHA-256 `66755301900c56ecd677791a68c4103002c635d5ad8b62fbe3a71e40cc706571`); the locked/offline macOS consumer check passed (log SHA-256 `6dd741abe5f036175a528a043bc2bfb50d8ce151a37873a66507ecf4f6383d82`), as did rustfmt. After wasm-bindgen packaging into a temporary localhost page, Chromium displayed `Count: 0`; clicking Increment displayed `Count: 1`. The owned tab, server and temporary package were removed. This is a bounded browser smoke test on uncommitted changes based on `5a6369d0`, not a production Web packaging, cross-browser or native runtime acceptance.

- **R28 standalone consumer Wasm compilation:** On clean commit `345f4e04`, the independent RustUI consumer passed locked/offline Headstart `check` and optimized `build` for `wasm32-unknown-unknown` using the approved Wasm toolchain root, LLVM Clang and host `rust-lld`. Corrected check and build logs have SHA-256 `8bd0a2e2ef57b64bb9555c18254398a02615e16ef3546970cc86c801921b9145` and `2dfbc8bcd56e69e52fc768957d5bd46e15b9d3c69254aa3b77fffa11facbfcd9`, respectively; the built `.wasm` has SHA-256 `2f781843361ce44016cdb146e0c1357625bc619ebe0aefea484de4cb4f0114fa`. Initial attempts failed because the Wasm root was supplied under the wrong variable and then because `rust-lld` was absent from PATH; the corrected runs succeeded. This establishes compilation and linking only, not browser packaging, startup, interaction, or release acceptance.

- **R28/R30 standalone consumer source alignment:** The external example now selects the same ten approved local patch paths as the workspace, including `cosmic-text` and `gpui-pre-wgpu`, without changing dependency versions. Its lockfile was updated for those local sources. A source regression compares the two patch maps and passed 1/1; the locked offline macOS consumer check passed (`target/headstart/evidence/consumer-patch-alignment-20261028.log`, SHA-256 `86c3f5aa2325ddd2b3c909af112369713c47cbb99d722da30cc20fd1042f8cd1`). The source-consumption guide no longer states a stale patch count. This does not establish Web consumer compilation or runtime acceptance.

- **R30 native compatibility smoke:** On macOS 27, the owned disposable Playground bundle contained the release executable built from the patched changes on parent `d279eafa`; both executable hashes were `67fb75777a26bbe1e1724f0f8b22e788adc731b4bcb6e7d37355320ea1053dbd`. The app opened, the Button count advanced from 0 to 1, and the List filtered to DatePicker, selected source row 5, and restored seven rows while preserving selection after search was disabled. The owned process exited and the disposable bundle was removed; the existing protected bundles were untouched. This is a bounded native smoke test, not complete component, OS, accessibility, or release acceptance.

- **R30 `block 0.1.6` compatibility patch:** The approved, versioned patch now selects the local pinned package in both the workspace and external consumer. Patch replay, reversal and tamper tests passed (4 tests, `target/headstart/evidence/block-patch-integrity-20261028.log`, SHA-256 `bc27a80813d88f0a1f0f8c0a4eb417780cb12f8189dd48a5c890ed62e3da9354`); a macOS C/Rust block FFI probe was also run as recorded in the preceding candidate evidence. The RustUI List tests passed 19/19 (log SHA-256 `14c63f728ae4178cc8cdf543faa8d3a10db7d546043782a872d809370794f836`); workspace and external consumer checks and an optimized native Playground build finished successfully (logs `block-patch-workspace-20261028.log`, `block-patch-consumer-20261028.log`, `block-patch-native-build-20261028.log`). The source check log reports 606 Python tests passed, and the Web build log reports 18 IME-plan tests passed and optimized packaging, with three notice gaps (`block-patch-source-20261028.log`, `block-patch-web-20261028.log`). These logs were produced on uncommitted changes based on `d279eafa`; native Playground smoke, release CI for the resulting commit, and licensing acceptance remain open. Existing protected Playground bundle paths must not be overwritten.

- **R30 `block 0.1.6` future compiler compatibility candidate:** The pinned crate declares `_NSConcreteStackBlock` as an uninhabited `Class` static and takes a reference to it. An isolated copy failed the installed Headstart compiler with `-D uninhabited_static` (log `target/headstart/evidence/block-probe-original-20261028.log`, SHA-256 `ca27661fabd14d4053d9b9a0f5651aa81fa0450a03a4e94817aa0e9e5c271317`). A candidate declares the external symbol as a byte and obtains its address as an opaque pointer; it compiled under the same lint and a standalone macOS probe invoked a copied closure successfully (logs `block-probe-candidate-20261028.log` and `block-probe-runtime-20261028.log`, SHA-256 `3ad9344c4c923eb486f69370c7ee123fa359348e7cccf71648032de9e6fa50a8` and `59a8e07e7c99cbfa32cc838c7a096664e8aa7db391630699ee39c0f629755507`). The candidate diff and original executed probe are preserved in `patches/proposed/block-0.1.6-stack-symbol.patch` and `patches/proposed/block-stack-symbol-rust-probe.rs`; patch dry-run passed and temporary binaries were removed. At that checkpoint it was not promoted; subsequent promotion and validation are recorded above. The separate `block` notice gap remains open.

- **R30 current-source integration:** On clean commit `50c0a143`, `./scripts/cargo-headstart check --workspace --all-targets --locked --offline` passed on macOS (log `target/headstart/evidence/workspace-check-50c0a143-20261028.log`, SHA-256 `07ed7eee3cba19c4b6e85a8f26d22a54fb15b1277feede8f5d49322b3e0e3432`). GitHub Actions [run 40757102734](https://github.com/abouzoubaa/rustui/actions/runs/40757102734) passed its source job on that exact commit; the macOS/Web job was skipped and no self-hosted runner was registered at inspection. This is not compiled CI acceptance. The compiler's existing future-incompatibility warning for `block 0.1.6` remains: `_NSConcreteStackBlock` is declared as an uninhabited static. The current report is `target/headstart/evidence/block-future-incompat-50c0a143-20261028.log` (SHA-256 `702fe99e65d2f1236b20a83ad285600729676e8a9d9185d10d83cbd7bb17aa5f`); this is a future-toolchain risk, not a current build failure.

- **R17 KeyedList search transition:** The retained KeyedList had the same hidden-filter defect when search was disabled. A GPUI test-window regression reproduced it (log `target/headstart/evidence/keyed-list-disable-search-before-20261028.log`, SHA-256 `d5bac7aee731926ead759135e9b2e57ec3c2039e56169ed9e9775caf58791c1e`). Disabling search now clears the retained query while preserving the owner's stable selected ID; re-enabling search starts unfiltered. The focused KeyedList suite passed 13/13 (log `target/headstart/evidence/keyed-list-disable-search-after-20261028.log`, SHA-256 `a79a16c078ab2e6e9ab4548beb6231d30b50d878afd8e0e6c1acb21acf6104f2`). The Web release check passed with 18/18 host IME-plan tests on uncommitted changes based on `560b2e52` (log SHA-256 `cb2e08e8f4257147a10ef9cd16ac102485a4b31bf855db13d7d32a6a7cadfe98`; manifest `f48699c25572f99a263184c9901a328aac7e8885ff0453bf45ec14c2bfb819d8`); three notice gaps remain. The regression sets the query programmatically; native and browser input acceptance remain open.

- **R17 native List search transition:** On clean commit `eecdaafd`, the locked/offline Headstart release build passed (log `target/headstart/evidence/list-search-native-build-eecdaafd.log`, SHA-256 `324e3297570c315591208125d270ce1ccf7263fd392989a310554b6af287d998`). The owned macOS 27 validation bundle and build executable matched SHA-256 `8db660da5f9443c791827f9a067166ea7d3a740b08f2a34df167e4e1b3625bd2`. In the native List demo, typing `DatePicker` filtered the rows; the row's `Pick` action selected source index 5; turning search off restored all seven rows and retained that selection; turning it back on displayed an empty search field. The owned process, disposable bundle and standalone copy were removed, and the shared GUI slot was released. This is a bounded native interaction check, not comprehensive List, pointer, keyboard, locale, or platform acceptance.

- **R17 List search in Playground:** The List demo now exposes a search toggle with English, French, and Arabic labels. Locale validation and the macOS Headstart Playground check passed. A Web bundle built from the uncommitted changes on parent `2990c47b` passed the Web release check and 18/18 host IME-plan tests (log `target/headstart/evidence/list-search-story-web-20261028.log`, SHA-256 `6783f2be63e364026e01b900e4bdf552876bb156fa4294c3f300af51ca82c2a5`; manifest SHA-256 `384c5d5f5a59c67ed0eb6cbb9c8a0ff3e9e59fc3cf4ac10b374e66747eab8d96`). In one 1280×720 in-app Chromium session, typing `DatePicker` filtered to that row, selecting it produced source index 5, disabling search restored all seven rows while selection remained, and re-enabling showed an empty search field. The toggle label resolved in French and Arabic. The owned tab and server were closed. This is bounded Web evidence; native interaction, cross-browser behavior, and the rest of the List story's locale coverage remain open. Three Web notice gaps remain.

- **R17 controlled List search transition:** Disabling search previously hid the search field while retaining its filter, leaving matching rows only and no visible way to clear it. A GPUI test-window regression reproduced that behavior (`target/headstart/evidence/list-disable-search-before-20261028.log`, SHA-256 `7825ef773e96706df8ca0c8e513393b7288c25a69245d0c982529fe810f467e1`). RustUI now clears the retained query through the existing Components API when search is disabled. The focused List suite passed 19/19, including re-enabling search and preserving an owner-selected row (`target/headstart/evidence/list-disable-search-after-20261028.log`, SHA-256 `44e6777fdfee3a2852bc3cafc57dc846af6a994d18720198b15a939b8d6eac4f`). The regression sets the query programmatically; native and browser user-input acceptance remain open.
- **R17 Web build:** With the List fix uncommitted on parent `e67a97f0`, the Web release check passed, including 18/18 host IME-plan tests and optimized packaging; three dependency-notice gaps remain. Log SHA-256 `854fa6c5d9ffbc18f65273069b0b49846ba6a81196d9e35fd8d26f09d83ee9dc`, bundle manifest SHA-256 `40ef65c0748300a6d2ff13d138a57b005bfa8dd6ca3d3c13b6497c67f14475d2`. This is build evidence, not browser interaction acceptance.

- **R35/R36 graph change browser check:** The source-bound `3b667a4e` Web bundle (manifest SHA-256 `d02ccf88ec0ed228a7556370bcfe71d78b25e207b9c7b03c437cf8a9e952f7e5`; graph source SHA-256 `fe5340d5f3d4d4d604e606ba71b62f5c6528165f74f69fd6d8ac16f726b1c1ac` matched the manifest) was exercised in one 1280×720 in-app Chromium session. GraphView pointer selection set `Selected: source`, and Enter raised `Activations` from 0 to 1. In NodeEditor, selecting and renaming source updated both node and edge relationship text; removing it removed the incident edge and cleared selection; Add node then displayed a new node with input and output. The temporary tab and server were closed. This is bounded Web interaction evidence, not native, cross-browser, full keyboard, accessibility, or performance acceptance.

- **R30 integration checks after graph changes:** On clean commit `3b667a4e`, `./scripts/verify-release.sh source` exited 0 with 603 Python tests and its Node and shell checks (log `target/headstart/evidence/graph-source-gate-3b667a4e-20261028.log`, SHA-256 `81db9a0382960cf80e17088bf0b42e24554e973d8cee08ba070e07132355dea0`). With the existing Wasm Headstart root, `./scripts/verify-release.sh web` exited 0, passing 18/18 host IME-plan tests and producing the optimized Web package; it still reported three dependency-notice gaps. Web log SHA-256 `78be584ed0cc648e122cb091b909c5b1184134321a5874f70a9771a03b9756a7`, manifest SHA-256 `d02ccf88ec0ed228a7556370bcfe71d78b25e207b9c7b03c437cf8a9e952f7e5`. These checks do not establish native or browser interaction, performance, or release acceptance.

- **R35/R36 incident-edge rendering preparation:** Graph rendering now constructs incident-edge lists in one pass instead of scanning all edges for each node's relationship text. It preserves graph edge order and includes self-connections once. The Headstart advanced-component suite passed 37/37 on macOS, including those cases (log `target/headstart/evidence/graph-incident-index-20261028.log`, SHA-256 `a63e1cc1dcd33cba5ae57383edc241a64d3148593855f0a3fb53ea20fd279765`). This introduces temporary per-render indexing; no rendering latency or allocation improvement has been measured, and representative native/Web workload acceptance remains open.

- **R27/R35 model lookup measurement:** `graph_lookup_benchmark` compares the indexed public `GraphDocument::node` lookup with a linear scan over the same 1,024-node graph. On macOS 27.0 (26A5425a), Apple M1 Max, `./scripts/cargo-headstart run --release -p rustui --example graph_lookup_benchmark` completed six alternating-order samples of 102,400 successful lookups each: indexed 2,235–3,523 µs, linear 85,667–90,416 µs. Log: `target/headstart/evidence/graph-lookup-benchmark-20261028.log` (SHA-256 `d2c8178fe23ef3fe02c143bf77173ea990971be061e23f59532f6c6b03c00a32`). This measures only model lookups in one process, excludes construction, edits, rendering, memory and missing-key workloads, and does not establish a release performance budget or end-to-end speedup.

- **R35/R36 graph endpoint lookup:** `GraphDocument` now maintains an internal node-ID index for endpoint lookup, including rendering, and rebuilds it after successful edits. A regression covers removal, shifted indices, reinsertion, and preservation of the original document. The Headstart advanced-component suite passed 36/36 on macOS (log `target/headstart/evidence/graph-node-index-20261028.log`, SHA-256 `416f08a89017d173bebbcc0e792c026becc7aac6432b45aa9ab63796c109c2fb`). This adds index memory and edit rebuild work; no end-to-end performance gain has been measured. Per-node relationship construction and representative rendering workloads still require performance assessment.

- **R35/R36 graph validation scaling:** `GraphDocument` validation now builds a temporary node index once, avoiding a full node scan for each edge endpoint while retaining existing error ordering and port checks. The focused Headstart GraphView suite passed 13/13 on macOS, including a 512-node, 511-edge graph and a missing endpoint at the end (log `target/headstart/evidence/graph-validation-index-20261028.log`, SHA-256 `0760f816c5872a284d286fe0f7409aae1cdb5e60df003f1d00f0771d8b2dce59`). This is an algorithmic improvement with functional coverage; it is not a measured end-to-end performance result. Rendering and representative workload acceptance remain open.

- **R17 List keyboard event path:** A GPUI test window renders RustUI's controlled List, focuses its retained list state, and sends Down through actual key dispatch. The owner selection changes from index 0 to 1 with exactly one binding write. After directly filtering the delegate to retain original rows 0 and 2, successive Down keys select source indices 0 and 2. `./scripts/cargo-headstart test -p rustui --lib keyboard_selection_updates_controlled_owner -- --nocapture` passed 1/1 on macOS; log `target/headstart/evidence/list-filter-keyboard-20261028.log` SHA-256 `08f60644e2793754ed0c2827ecd6909dd40a0c1fd83614ef50e7f55393416d05`. This does not exercise search-field input, Safari behavior, or complete List acceptance.

- **R31 Web release checkpoint:** On clean commit `d01fde11`, `RUSTUI_WASM_HEADSTART_ROOT=/Users/Abium/Developer/rustui/target/headstart/wasm-root ./scripts/verify-release.sh web` passed the Web package checks and 18/18 host IME-plan tests. Log: `target/headstart/evidence/web-noopener-release-d01fde11-20261028.log` (SHA-256 `f0b06c1ddc8168a6faf5b8eec92c9179f3f8c10dd6ea76fc7d643bec0b5b7f24`); the resulting Web manifest SHA-256 was `b3ae5ce9a2d021af75378db33cffcd7dd48177b4a660575f1862308f2c3c5abf`. The package reported three unresolved dependency-notice gaps. Safari 27 loaded the bundle, but automated interaction did not select the Link story, so opener isolation was not verified in Safari and no Safari defect was established. The temporary tab and server were closed. This checkpoint does not close R31 or broader release acceptance.

- **R31:** RustUI Web platform URL opening now uses `_blank` with `noopener` through the reproducible, versioned `gpui-pre-web-noopener-v1.patch`; the pinned upstream version and checksum metadata remain unchanged. The Link API documents that applications must validate untrusted destinations and may use custom schemes. The patch replay/reversal/tamper regression passed, and the local source gate passed 603 Python tests and accompanying checks (log `target/headstart/evidence/web-noopener-source-final-20261028.log`, SHA-256 `a153bf020c02d5ee5f4e0e21c2051b1a23c9a4dd0eca5432249904b535399a67`). With the same local fixture in one in-app Chromium session, the unpatched bundle displayed `Opener: present`; the patched final bundle opened the fixture and displayed `Opener: none`. The final source-bound Web build passed (manifest SHA-256 `9427f8c04394bcfe4958ea8df82158a5683ee19156f550dc71cf96b89a828057`, build log `target/headstart/evidence/web-noopener-final-20261028.log`). The owned tabs and servers were closed. This is bounded browser evidence, not cross-browser or complete security acceptance; CI for this change and other release gates remain open.
- The fixture was initially edited outside the claim. Its exact content was preserved outside the checkout, the clean file restored, the claim expanded to `assets/link-fixture.html`, and the preserved content restored with matching SHA-256; the subsequent claim audit passed.

## Current checkpoint — 2026-10-28: localized Button arguments and Popover

- **R13/R19:** RustUI Button now supports `localized_label_with_arguments` for live translated labels while preserving its literal fallback and identity; selecting a static localized label clears prior arguments. The existing Popover ownership example uses this for lifecycle and child counts and translates its controls, status, inherited value and notes placeholder in English, French and Arabic. Focused Button tests passed 7/7, Popover catalog tests passed 1/1, locale validation and the optimized Web build passed. In one in-app browser session, French owner and child increments persisted after switching to Arabic with the Popover open; the inherited value changed, removing the owner released the child, and restoring the owner recreated the closed trigger. The source-bound Web manifest SHA-256 is `94bc554b590db5476d1cbc2d4019224723568f5df465fa29fc7f7643d945136f`. Logs are `target/headstart/evidence/popover-localized-{button,catalog,web}-20261028.log`. These are bounded Web observations; native, keyboard and broader RTL/locale acceptance remain open. The owned browser tab and server were closed.

## Current checkpoint — 2026-10-28: localized Menu trigger arguments

- **R13/R17:** RustUI Menu now accepts locale-aware arguments for its translated trigger label through `localized_label_with_arguments`, preserving the original literal fallback when a translation is missing, blank or cannot be formatted. Calling the existing static `localized_label` clears prior arguments. The Playground Menu example uses this API and translated action labels/counts in English, French and Arabic, with logical-start alignment. Focused framework localization tests passed 14/14, the Playground count-format test passed 1/1, locale validation passed, and the optimized Web build succeeded. The in-app browser showed the French and Arabic labels and options, incremented in each language, retained the count across the switch, and kept the Arabic disabled item inactive. The Web manifest SHA-256 is `21a0c48b3912b62337a5318581735cdabc70513ef9d18ec5c91e8405756e7fdb`; framework, Playground and build logs are `target/headstart/evidence/menu-localized-arguments-{framework,playground,web}-20261028.log`. This is bounded browser evidence; native, keyboard and wider RTL acceptance remain open. The owned browser tab and server were closed.

## Current checkpoint — 2026-10-28: TextEditor owner update during preedit

- **R16:** Extended the existing GPUI test-window regression for a retained multiline TextEditor. After an external owner value replaces active preedit, the marked range is cleared; a subsequent text commit updates both editor and owner consistently. The pinned upstream multiline `set_value` contract resets selection to the start, so this regression expects the committed character there. The focused Headstart test passed 1/1; log `target/headstart/evidence/text-editor-owner-preedit-cancel-20261028.log` has SHA-256 `adbbb3cadef8745edb3961fbfad9f19ca3ff905c5a351f11468ca68f9cda552f`. This is test-window evidence, not a real OS IME or native/browser acceptance result.

## Current checkpoint — 2026-10-28: localized Tooltip story

- **R13:** The existing Tooltip Playground story now uses the public RustUI localization APIs for its heading, instructions, three tooltip messages, target labels and click counter in English, French and Arabic. Direction-aware row wrappers align the tooltip targets with the locale's logical start. The focused catalog/count test passed 1/1 and locale validation passed; the Web build passed with the source changes on parent `db61790c`. Test log SHA-256 `c300e185fd7d32b531d89e06018a3104e6a3ca80665c5bf5da7785c0af67dc84`; Web build log SHA-256 `7ca7208e9a8f37525d8cb25eecd3520cb4b46ac62eae709c9e8ba73b0cc7fd7a`; Web manifest SHA-256 `fbe9ee953658e2a923c611bbb533c0fb5db5362b53e754dae780871006fe57e3`. In one in-app browser session, French and Arabic story text displayed, targets aligned left in French and right in Arabic, the counter survived switches, an Arabic click incremented it, and a disabled click left it unchanged. The Arabic disabled tooltip text was visible. This is bounded Web evidence; native, wider keyboard, tooltip dismissal and broader locale/RTL acceptance remain open. The owned browser tab and test server were closed.

## Current checkpoint — 2026-10-28: trusted CI validates Web tool provisioning

- **R30:** GitHub Actions [run 40741127714](https://github.com/abouzoubaa/rustui/actions/runs/40741127714) passed both source and compiled macOS/Web jobs on `16a655aaad226d9e18cc7853f62f699ccac50a3c`. The one-use macOS ARM64 runner installed the new `wasm-bindgen 0.2.129` tool successfully, passed its preflight, built and packaged the macOS Playground, and produced the Web bundle. The combined log records 1,431 passing Rust tests across 42 result groups, with no failed or ignored tests. The Web bundle manifest SHA-256 is `1e7e136fceb017e6b0739a0eec7c9f029e5f7cf8456d5319f546339b10a0bbd0`; the retained full CI log is `target/headstart/evidence/ci-40741127714-16a655aa-20261028.log` (SHA-256 `c19f8cbcea96ebd9bc7d3bad29bcf58da3af2e78d35bf50a30cdaf21c49739c0`). The new runner archive matched GitHub's published SHA-256 and all 9,390 extracted files matched the archive. The runner reused the dedicated prior CI checkout, its build cache and the approved Wasm sysroot; this is not a fresh-cache or cross-machine reproducibility result. It removed its credentials and registration and exited; runner inventory is empty. GUI/browser interaction, performance, notices and release acceptance remain open.

## Current checkpoint — 2026-10-28: trusted CI Web tool provisioning

- **R30:** The trusted macOS ARM64 job now provisions the pinned `wasm-bindgen 0.2.129` CLI from its official release when the Headstart tool path is absent. The archive is pinned to SHA-256 `81d4a23d56b3c3eb8187658329116d50e0b228a93b343825fb71f70179051cd1`; installation verifies the archive and executable version, preserves an existing tool, and refuses incompatible or concurrent installations. A fresh temporary macOS fixture downloaded and ran the official binary, whose SHA-256 was `6ad2f953d8abcb36d6b084df221fb2f0716217854c1b8df938aa2f5678d1b948`; the three focused preservation/tamper tests and existing Web packaging fixture passed. The changed workflow still requires a fresh trusted CI run, and Wasm toolchain provisioning and the other release gates remain separate.
- The local source release gate also passed with these changes, including 602 Python tests and the accompanying checks. Log `target/headstart/evidence/ci-bindgen-provision-source-20261028.log` has SHA-256 `70f334ee5c6479909b15c33a3f993717a6e19155e82b52705fbb9505e7a168a9`. This is local source evidence, not execution of the changed CI workflow.

## Current checkpoint — 2026-10-28: current-source workspace validation
- **R30:** On clean source `a279d4656c4448f3e35be5548e181887bac994d6`, the locked, offline Headstart macOS workspace all-target check and external-consumer check passed. The workspace library tests passed 1,353 tests across six result groups; integration tests passed 60 tests across 34 groups, with no failures or ignored tests. Logs under `target/headstart/evidence/` are `all-targets-a279d465-20261028.log` (SHA-256 `c463b1cda836b9eb868a8cc213c5346529c588db7fef6f5ad719419b88aa65ba`), `external-consumer-a279d465-20261028.log` (`95282559c4e68ba39e058477e5d48ebb6b74d54f03f4657b6274fe8bab4b6bc5`), `workspace-lib-a279d465-20261028.log` (`8616a3392fe4e1ec0954a02e3afc1444bf3f3af169b71c1a863bbfbefa46d0c6`), and `workspace-integration-a279d465-20261028.log` (`b0c4c908fed53b4b227a96eb56e4e3bda76457f784d74ce3d0b805ea91a98ae3`). These checks do not establish native or browser interaction, performance, licensing or release acceptance; the provisional group counts are unchanged.

## Current checkpoint — 2026-10-28: committed Select title during search

- **R30 source gate:** `./scripts/verify-release.sh source` passed with the select patch and multi-option changes present, including 599 Python tests and the accompanying Node, shell, locale and theme checks. Log: `target/headstart/evidence/select-title-source-gate-20261028.log` (SHA-256 `fbbb9060757fed922ba7b2750a47ad4a00203280cda8a8c8a7069c25728545d4`). After that run, the patch was regenerated with zero context to remove patch-file whitespace; its exact replay/reverse/tamper test and postimage check passed again, with the same runtime source. This is local source validation, not a compiled CI or release acceptance result.

- **R13/R15:** A two-option localized Form Accessories example exposed a Select defect: filtering highlighted an unconfirmed option and the trigger temporarily displayed that option instead of the committed value. The approved reproducible `gpui-component-select-committed-title-v1.patch` renders the existing committed selection snapshot; dependency revisions and upstream checksum files are unchanged. Patch replay/reverse and tamper rejection passed, RustUI input tests passed 209/209 and the Playground form tests passed 4/4. An optimized Web bundle built from uncommitted changes on parent `1b170dae` (manifest SHA-256 `8bb771415d08d923d2a0d9fd855e2e2fecbace2210fb66d420104285e71ec857`). In one 1280×720 in-app browser session, searching French `aut` highlighted “Autre” while the trigger retained committed “Libellé”; confirming “Autre” updated both bound controls and switching to Arabic showed “بديل” in both. The earlier unpatched bundle also showed Arabic Picker selection updating the shared ComboBox and remaining selected after an English switch. This is bounded browser evidence; native, wider keyboard and locale/RTL acceptance remain open. Three Web dependency-notice gaps remain. The owned tab and server were closed; logs are under `target/headstart/evidence/select-title*-20261028.log` and `multi-select-gallery-tests-20261028.log`.

## Current checkpoint — 2026-10-28: localized selection options

- **R13/R15:** Picker and ComboBox now support per-option locale keys, keeping the caller's stable binding value separate from the displayed title and falling back to that value for absent/blank translations. Rebuilding translated options reconciles retained selection; option search uses the displayed title. The Form Accessories examples opt in for their shared selection. RustUI input tests passed 209/209 and Playground form tests passed 4/4. A Web bundle built from these uncommitted changes on parent `95ebe59d` (manifest SHA-256 `a9f4bd056cff588ec729a4249f0c55a1c261fbd597e2c38cf6c3ac6f50dcae1e`); in one 1280×720 in-app browser session both controls showed French titles, the ComboBox menu offered the French option, an Arabic switch updated both selected titles, the Picker menu offered the Arabic option, and switching back to English retained the shared selection. This is bounded single-option Web evidence; multi-option UI, keyboard, native, broader locale/RTL and accessibility acceptance remain open. Three Web dependency-notice gaps remain. The owned tab and server were closed; logs are `target/headstart/evidence/localized-select*-20261028.log`.

## Current checkpoint — 2026-10-28: disclosure indicator

- **R13/R17:** Disclosure replaces the previously missing Unicode triangle glyph with existing vector chevrons: closed right in LTR, closed left in RTL and down when open. The focused Disclosure suite passed 7/7. A Web bundle built from these uncommitted changes on parent `ea952344` (manifest SHA-256 `c1c454d93eee860914ca32086516fe46cd7de82a06bed814e268f2990b73e591`). In one 1280×720 in-app browser session the French control showed a right chevron, clicking it changed the indicator to down, switching to Arabic retained the open state, and clicking it closed the control with a left chevron. Native visual, keyboard and wider RTL acceptance remain open. The owned tab and server were closed. Logs: `target/headstart/evidence/disclosure-indicator-tests-20261028.log` and `target/headstart/evidence/disclosure-indicator-web-20261028.log`.

## Current checkpoint — 2026-10-28: localized menu and disclosure triggers

- **R13:** Menu and Disclosure accept optional localized trigger labels with their original text as fallback; Disclosure preserves an explicit accessible-name override. The Playground's form gallery uses these APIs. The focused localized-trigger tests passed 13/13 and the Playground Headstart check passed. A Web bundle built from these uncommitted changes on parent `26fdf02d` (manifest SHA-256 `e89e6f343ca2b738dbb0c2342b857621d0d223d58997ef715fcd2156df3ea213`); in one 1280×720 in-app browser session the two labels displayed in French and Arabic. This is bounded visual evidence; interaction, native and wider RTL acceptance remain open. Picker and ComboBox selected values remain English, and the disclosure indicator appeared as a small missing-glyph box in that browser session. Three Web dependency-notice gaps remain. The owned tab and server were closed; logs are `target/headstart/evidence/menu-disclosure-localization-tests-20261028.log` and `target/headstart/evidence/form-triggers-web-20261028.log`.

## Current checkpoint — 2026-10-28: form gallery locale filtering

- **R13:** Form Accessories headings, row names, filter, action status and Button/Checkbox/Radio/Stepper/Toggle labels resolve in English, French and Arabic. Filter matching uses the three runtime catalogs and retains the existing English identities and component state. The focused gallery suite passed 4/4 and locale catalogs validated. A Web bundle built from these uncommitted source changes on parent `504f8190` (manifest SHA-256 `675b74234ab2e7a827ca274f2429d2a5efb5a025834d72f47a2f399d2c808292`); in one 1280×720 in-app browser session French `interrupteur` selected the Switch row, switching to Arabic retained the query and state, and Arabic `مربعات` selected the Checkboxes row. The owned tab and server were closed. At that checkpoint ComboBox, Picker, Disclosure and Menu example values still showed English; the subsequent trigger-label work is recorded above. Native and broader RTL acceptance remain open. Three Web dependency-notice gaps remain. Logs: `target/headstart/evidence/form-localization-tests-20261028.log` and `target/headstart/evidence/form-localization-web-20261028.log`.

## Current checkpoint — 2026-10-28: owner task cancellation race

- **R20:** A GPUI test-window regression completes background work, cancels the task before foreground delivery, and confirms the owner receives Idle without the stale success. The full owner-task suite passed 7/7; log `target/headstart/evidence/owner-cancel-completed-race-20261028.log` SHA-256 `dac888a8a9c202695e6cfa6ee6c9a24f2ca27c228f2109707a5b712226a36218`. This tests one simulated scheduling order; native and broader concurrency acceptance remain open. The macOS host workspace all-target check also passed on clean source `ea79fd1e`; the existing `block 0.1.6` future-incompatibility warning remains. Log: `target/headstart/evidence/all-targets-ea79fd1e-20261028.log`.

## Current checkpoint — 2026-10-28: graph display labels

- **R13/R35/R36 rename check:** In another 1280×720 in-app browser session using manifest `fb1cb2b0cf37382118bfbf8968c8e5de90856784f574b22f100670c4c0cfa259`, selecting the source node and invoking Rename displayed “Nœud renommé” after switching to French and “عقدة أعيدت تسميتها” after switching to Arabic; selection remained on the source node. This closes only that bounded rename/locale interaction. The owned browser tab and server were closed. Source CI run `40732345495` passed on `d528380e`; its compiled macOS/Web job was skipped.

- **R13/R35/R36:** Graph nodes and ports now accept optional localized display keys with literal fallbacks; node renames can set a new key, while ordinary renames clear any previous key. The Playground's original, added and renamed node labels and its port labels have English, French and Arabic catalog entries. The focused graph suite passed 12/12, the Playground graph suite passed 3/3, and runtime locale validation passed. An optimized Web bundle built from these uncommitted source changes on parent `55931139` (manifest SHA-256 `fb1cb2b0cf37382118bfbf8968c8e5de90856784f574b22f100670c4c0cfa259`); one 1280×720 in-app browser run displayed the original nodes and ports in French and Arabic, and a node added in Arabic remained present and displayed its French label after switching languages. This is bounded Web visual and interaction evidence; native, wider rename scenarios, RTL and graph acceptance remain open. Three Web dependency-notice gaps remain. Logs are under `target/headstart/evidence/graph-*-20261028.log`; the owned tab and server were closed.

## Current checkpoint — 2026-10-28: Web CI tool preflight

- **R30:** The trusted macOS/Web CI job now checks the required `wasm-bindgen` version before running the macOS build and tests. The Web build script exposes `--check-bindgen` using its existing version validation; it does not install the tool or build a bundle. The Web packaging fixture passed, including rejection of a mismatched version and a check that preflight does not create a bundle. This avoids the late failure seen in run `40726542190` attempt 1; provisioning the pinned tool and a fresh-runner reproducibility check remain open. The changed workflow has not yet run in CI.

## Current checkpoint — 2026-10-28: compiled CI results

- **R30:** GitHub Actions run `40726542190` is pinned to `12747eeb`. Attempt 1 passed source CI and the macOS checks and optimized package, but its Web build failed because the fresh runner lacked `wasm-bindgen 0.2.129`. Its full log is retained in `target/headstart/evidence/ci-40726542190-attempt1-20261028.log` (SHA-256 `f00fb357752d4ad2b4415616174bd088b4dabe0965ff9f7e98ae1a535ed82eb4`). The isolated runner was supplied the matching existing executable (SHA-256 `caa8f16f89f285066715f68725b7b52aa7becd9e557b5021ff8f5dddb34d25c6`). The overall run completed successfully: the source job passed in attempt 1 and the compiled macOS/Web job passed in attempt 2; its log SHA-256 is `5c775be5d20c625f7ece308cf473bab8ea67974fe07f67754df44882264f4d70`, and the Web bundle manifest SHA-256 is `069d3c92460f4cadfd2e089997c711f93cd249aa58e4b7ed9572cbdb87b437be`. Both attempt logs are retained in `target/headstart/evidence/`; the first attempt's package is preserved in the runner's `target/headstart/evidence/ci-40726542190-attempt1/`. The ephemeral runner deregistered and removed its credentials. This passes the compiled CI jobs for that revision; GUI/browser, notices, performance and release acceptance remain separate.

## Current requirement inventory — 2026-10-28

This is a **provisional classification of all 37 ledger groups**, not a
percentage of RustUI completed. It summarizes the existing ledger evidence, not
a new requirement-by-requirement source audit. The groups vary greatly in size and contain
both product capabilities and release-process work. “Implemented portions”
means the named capability exists in source; it does not assert that its full
roadmap contract has been audited or accepted.

| Implementation status | Groups | Meaning and remaining work |
| --- | --- | --- |
| Coding recorded complete | 4: R00–R02, R34 | Dependency boundaries, single-writer coordination, approved patch selection, and the ResizablePanel adapter. R34 still needs broader acceptance. |
| Implemented portions, full contract or acceptance open | 28: R04–R20, R22–R26, R28–R29, R32–R33, R35–R36 | Bootstrap, lifecycle, state/layout, animations, Playground, macros, components, themes, localization, collections, overlays, async ownership, preview/composition, Web, Outline, Drawer, GraphView, and NodeEditor have substantial source implementations. Some still have known missing behavior; others need contract review, integration or runtime evidence. These 28 are not 28 fully coded groups. |
| Cross-cutting work or deferred work requiring separate assessment | 5: R03, R21, R27, R30–R31 | Public documentation/contract audit; keyboard and accessibility; performance and candidate workflow; CI/reproducibility; security, notices, packaging and release preparation. Each has existing work and open requirements. Keyboard remains in scope; broader accessibility and VoiceOver are explicitly deferred. |

Acceptance remains **3 of 37 groups recorded fully accepted** (R00–R02), not
3 of 37 features tested. The other 34 include passing focused tests and bounded
native/browser observations, alongside missing or deferred evidence. They have
not been exhaustively reclassified as fully accepted. Current test evidence
includes the 1,343 workspace library and 60 integration tests on `d7a9c4d0`,
and subsequent focused menu and browser checks; these cover different source
revisions and do not by themselves close whole-group acceptance.

Release priorities: resolve the six dependency-notice versions and review
license obligations; complete in-scope English/French/Arabic and RTL behavior;
finish current-source native and browser input, lifecycle and functional checks;
measure representative performance and validate budgets; establish compiled
CI, supported-version scope, reproducibility and distribution. Accessibility,
VoiceOver and languages beyond the three selected remain undone/deferred.

## Current checkpoint — 2026-10-28: consumer and all-target compilation

- **R30 current-source compilation:** On source `2a1b5d25`, `./scripts/cargo-headstart check --manifest-path examples/external-consumer/Cargo.toml --profile headstart --locked --offline` and `./scripts/cargo-headstart check --workspace --all-targets --profile headstart --locked --offline` both passed on the macOS host. The latter covers Cargo targets, not additional operating systems. Retained logs under `target/headstart/evidence/` are `external-consumer-2a1b5d25-20261028.log` (SHA-256 `1bd0d7dcdfb7ec0ecb1fff6d0690ff2ae101f37c08a96980cc12bd7d035851b5`) and `all-targets-2a1b5d25-20261028.log` (SHA-256 `d194cf7aebce025d592cd1332cb8e89b50f371d635ccae066c5d9716152d6060`). The known `block 0.1.6` future-incompatibility warning remains. These checks do not establish GUI, browser, compiled CI, packaging or performance acceptance.

## Current checkpoint — 2026-10-28: context-menu gallery locale copy

- **R13 partial source implementation:** The context-menu gallery's owner state and controls, menu labels, help text and numeric status now resolve in English, French and Arabic. The Playground Headstart check, three-catalog validator and focused locale-resolution test passed. A 1280×720 in-app browser check of source `7ed298a5` (Web manifest SHA-256 `30810011083695a2384451ed4d0de0318f631a6e2d956e60caaa88701c70ddc0`) displayed French menu labels; Increment changed 0 to 1, the disabled item left its count at 0, and Remove owner dismissed the menu and hid the target until restored. Arabic labels rendered, but its two surrounding buttons were left-aligned and the popup labels appeared left-aligned. A fresh 1280×720 in-app browser check of source `6ce7d541` (Web manifest SHA-256 `54396ba0e3faa8689ae6680ef4420829a9bdde346be2329b94a70deaa9f64d1b`) showed those buttons aligned right, and the order button changed the Arabic status to reversed. The RustUI menu adapter now carries the active direction into popup label layout; the focused menu suite passed 12/12. In a fresh 1280×720 in-app browser check of source `8c95f4ed` (Web manifest SHA-256 `4d3fdb6b17590e5b9f6ccc78ca08a2afbb28f9f9951b0e85b7c34b6c600b66a9`), Arabic popup labels aligned right and Increment changed its count from 0 to 1; French popup labels aligned left. This is bounded visual and interaction evidence. Broader native/browser acceptance remains open. This does not close R13.

## Current checkpoint — 2026-10-28: local Headstart workspace tests

- **R30 current-source test evidence:** On clean source `d7a9c4d097c382422388f2434fa945549fe2ada7`, `./scripts/cargo-headstart test --workspace --lib --profile headstart --locked --offline` passed 1,343 tests across five nonempty suites, with no failures or ignored tests. `./scripts/cargo-headstart test --workspace --test '*' --profile headstart --locked --offline` passed 60 integration tests across 34 suites, with no failures or ignored tests. Logs are retained under `target/headstart/evidence/` as `workspace-lib-headstart-d7a9c4d0-20261028.log` (SHA-256 `fd859c497b76b1582d6f310e2b0195e729e24aad4c310561f5e03e5d422861ff`) and `workspace-integration-headstart-d7a9c4d0-20261028.log` (SHA-256 `bbdf6e9355661e7d29cb9fce2dda8953c07add7089bdae633229de4c6b67f669`). GitHub run `40721973539` passed source CI but skipped compiled macOS/Web jobs. No Playground test application remained running. These results do not close GUI, browser, packaging, compiled CI, or release acceptance.

## Current checkpoint — 2026-10-28: Pathfinder notice provenance

- **R31 evidence, notice gap still open:** `pathfinder_geometry 0.5.1` records Git revision `a5e98fac00f433fe2ff4b2135383d82491b8bfc5`, which the listed upstream repository does not currently resolve. All nine vendored Rust files match upstream revision `15101e90dd0a21fa3aecd6dfc001aa3340d9b851`, whose root contains MIT and Apache license files. Its original manifest matches the vendored original manifest except for version `0.5.0` versus `0.5.1`. This is source correspondence, not proof that the matching upstream revision is the exact published release or that the notice obligations are fully satisfied. File URLs and SHA-256 values are retained in `target/headstart/evidence/pathfinder-geometry-provenance-20261028.json` (SHA-256 `3270ccf1d82a4aabe6bf7d5539a7429bb7d3e0addcfff208e0b7a64212b54788`). No notice mapping or release clearance was inferred.

## Current checkpoint — 2026-10-28: feedback button direction

- **R13/R15 source correction:** The two standalone feedback actions now sit in direction-aware RustUI horizontal stacks, addressing the left-aligned Arabic controls observed in the prior Web check. The focused Playground locale test passed 1/1. At 1280×720, a fresh in-app browser check of source `699f97c3` (Web bundle-manifest SHA-256 `641c1a8a86d6bc9902c1a074c23b653f115b416511ac60de74618fa779a2cf88`) showed both actions aligned to the Arabic content; Retry changed its count to 1 and the notification action opened the toast. This bounded observation does not close broader RTL or native acceptance.

## Current checkpoint — 2026-10-28: feedback browser observation

- **R13/R29 bounded Web observation:** Source `990bb72c` Web bundle (manifest SHA-256 `c622aa9fc6343899528e1064a11291ef20204008af88ed9e00b1214251f14aaa`) ran in the in-app browser at 1280×720. French feedback labels and empty/error text rendered; a persistent toast displayed its French title, message and action. Arabic feedback text and a localized toast also appeared. This is not full interaction, RTL or browser acceptance: the French error card overflowed its fixed height, and the Arabic toast overlapped the header area. The fixed card heights were removed in `7a43edc7`; a fresh 1280×720 in-app browser check of its Web bundle (manifest SHA-256 `2ca1f5ccf9b79c1a2914b3143b8f02ca702a3d8af4e0c45044e23c18caa88cdb`) showed the French error content and retry count inside the card, and clicking Retry changed the count from 0 to 1. The focused Playground locale test passed 1/1. RustUI now offers opt-in logical toast placement, and the feedback gallery uses it. At 1280×720, the in-app browser displayed the Arabic toast on the left, clear of the header text, and its action changed the action counter to 1. This bounded Web observation used source `44671268`, with bundle-manifest SHA-256 `14e74a5485477d15334dbc0f6f2bbe87f68b7568dbaa2f6a1f24f90caf8f2f27`; broader RTL and native acceptance remain open. The focused toast suite passed 10/10 and the Playground locale test passed 1/1. The temporary browser tab and owned server were closed.

## Current checkpoint — 2026-10-28: isolated Web build output

- **R28/R30 tooling:** The Web packaging script accepts an optional constrained `RUSTUI_WEB_OUTPUT_NAME` under `target/headstart`, allowing a separate validation bundle while an existing default bundle is served. The package fixture suite passed with checks for isolated output, preservation of the default bundle, and rejection of traversal names. Browser and release acceptance remain separate.

## Current checkpoint — 2026-10-28: localized toast content

- **R13 partial source implementation:** RustUI Toast now resolves optional message, title and action-label keys when shown, using literal fallbacks for missing or blank translations; announcement text derives from resolved content. Playground's toast examples supply English, French and Arabic keys. Focused Headstart tests passed 9/9 toast, 44/44 feedback, and 1/1 Playground locale; the three runtime locale catalogs validated. This is source evidence only: displayed native/browser toast behavior, broader gallery copy and RTL acceptance remain open. R13 is not closed.

## Current checkpoint — 2026-10-28: localized feedback states

- **R13 partial source implementation:** RustUI EmptyState and ErrorState now support localized visible titles and details, falling back to their supplied literal text for missing or blank translations. Default accessible names follow the resolved title; explicit names retain their existing precedence. Playground uses these APIs for its English, French and Arabic empty/error examples. The feedback component suite passed 44/44, the Playground locale test passed 1/1, and the locale validator passed. Native/browser rendered and RTL acceptance remain open; toast messages and other gallery copy still need localization. This does not close R13.

## Current checkpoint — 2026-10-28: feedback gallery locale copy

- **R13 partial source implementation:** The feedback gallery heading, persistent-toast button, five kind/retry buttons and two activation counters now resolve in English, French and Arabic. A focused test passed 1/1 for numeric interpolation in all three locales; the Playground Headstart check and runtime locale validator passed. Remaining feedback content, including toast and empty/error messages, is still English. Native/browser presentation and RTL acceptance remain open; this does not close R13.

## Current checkpoint — 2026-10-28: theme parser measurement

- **R26, bounded local performance evidence:** In a release-profile, locked, offline Headstart run at source commit `3028f2b1`, the 821,287-byte bundled macOS theme was parsed and validated 20 times per path, alternating execution order after one warmup each. Baseline median/p95 were 237,706/240,669 µs; the duplicate-key-checking parser median/p95 were 240,013/244,667 µs. This is a single-machine micro-measurement, not an app responsiveness benchmark or performance acceptance; no general overhead guarantee follows from it.
- Full build/run log, input hashes, and the temporary measurement source are retained under `target/headstart/evidence/theme-parse-measurement-*20261028*`. The temporary example was removed after preserving its source. Broader theme and release performance gates remain open.

# Production consolidation — 2026-10-26

Writing session: thread `01a172cd-8a21-73f1-82cd-62d451c71ec9`, claim
`c44f414bfbf4ef4c4f30`, branch `codex/production-consolidation-01a172cd`,
worktree `/Users/Abium/.codex/worktrees/rustui-production-consolidation/rustui`.
The verified primary integration branch is `work/m118-diagnostics-provenance`;
The writing branch is fast-forward integrated through its latest completed commit.
Reverify live HEAD and dirty state before subsequent integration; preserve unrelated work.

## User-selected current scope

English, French, and Arabic remain in scope. The user explicitly set aside
VoiceOver and broader accessibility work, plus Japanese and other languages
and scripts, for later. Keep these items marked undone; do not count them as
passed or imply accessibility or language support beyond the evidence. The
37-group counts are provisional lower bounds and do not measure the amount of
code completed or an adjusted-scope release percentage. Older checkpoint counts
are retained as historical records.

## Current release evidence — 2026-10-27

- **R12/R26 ambiguous theme JSON, 2026-10-28:** Theme parsing now rejects duplicate object keys at every depth instead of silently accepting the last value; the original JSON still reaches existing raw-number validation. The nested-key regression failed before the fix (log SHA-256 `1686e27dea413d3b19bae190d5848b3236f1d300fc43d3e1d50f18f93723b6e4`). The theme suite passed 86/86 (log `89c9c0d91b7318e856652ab8c858b5d3e001c96d63f109917b5479e3adfd60b2`); a subsequent watcher regression confirmed an ambiguous reload retains the last good theme (1/1, log `2d18b05e7629fee8f8ef625614e9c99090c2ebfa07d60b0b9b84980dad78ca19`). The optimized Web build passed with three notice gaps (log `9d4ac70eb528eb57a0f87d31f18b9eb7ebdc23a33d2801613d891779ebec7f05`, manifest `713cb01d3b77dc9149b36dcf97a035e9ae24040991a112e3641ae8febdd068c2`), and one Chromium session showed the Playground starting normally. The build preceded the watcher test-only edit. The additional JSON pass has not been performance-benchmarked; native, broader Web reload and performance acceptance remain open. The owned tab and server were closed.

- **R19/R33 Drawer keyboard containment, 2026-10-28:** A real GPUI test-window regression sends five Tab and five Shift+Tab keystrokes to the open Drawer, requiring each direction to visit at least two controls while focus stays inside. The full focused Drawer suite passed 8/8 (log SHA-256 `6acc2230c682293174681508a2d14a7aa2a1a63878ed6651df57def18f573fe5`). An initial direct `window.focus_next` probe escaped the modal (log `c75138fded7425c4ada4bfda76d3f0844e230c0a246a6d383b1b01199b2315fa`); that API bypasses the Root's keyboard trap and does not demonstrate a Tab failure. Programmatic focus containment, native/browser keyboard, nested overlays and wider acceptance remain open.

- **R30 protected bundle incident, 2026-10-28:** The later local macOS release gate on `74a47b66` ran `package-app.sh` against `target/headstart/debug/RustUI Playground.app` despite the earlier automatic rejection of that bundle's cleanup. The prior recorded bundle executable hash was `9ecaa0ec2d93ef82c4bf3cd760afd30a3619ddb4777cdaeb334579d242ffc24f`; the current bundle executable is `daf9ed138046454d9940373c7137b31a70469ab6adc56ea7e0b2fb4bcbc681c6`, matching the release executable built by that gate. Thus the protected bundle was overwritten during packaging; the earlier app artifact was not preserved at that path. The other rejected bundle under `target/headstart/verification/debug/` remains protected. Do not run the macOS packaging gate or remove/overwrite either protected path until this incident and cleanup are resolved. The recorded automatic review rejection did not provide a reason in this ledger.

- **R16 retained TextEditor preedit, 2026-10-28:** A GPUI test-window regression drives the retained multiline input's marked-text handler and observes preedit preserved across a rerender. A distinct owner update replaces that preedit, and both retained input and owner retain the external value after the GPUI event queue runs. All 17 focused TextEditor tests passed (log SHA-256 `f0670dffefc47ae5601672033149309bd82e56597ae5d48753d52c6fc3dfbace`). This exercises real retained state and binding in a test window; it does not prove OS-generated IME, focus, subsequent commit, native GUI or browser behavior.

- **R16 retained TextField owner update, 2026-10-28:** A GPUI test-window regression now drives the retained input's marked-text handler, updates the RustUI owner to a distinct value, then simulates a subsequent commit after the GPUI event queue runs. The retained field and owner both contain the new value and then `externalé`. All 40 focused TextField tests passed (log SHA-256 `d370985f62c9a8ab129340c7ab0ecb3b6d43a8ab55cae024163c884f1f3f9a2b`). This exercises the real retained input and owner binding in a test window, not OS-generated IME events, focus retention, browser IME or current native GUI acceptance. The previously rejected disposable native bundle paths remain protected.

- **R22/R25 numeric preview transport, 2026-10-28:** Preview edits previously rejected exact integer JSON tokens such as `1` for Number properties despite the document codec accepting them. A focused regression reproduced `invalid_edit` (log SHA-256 `26a1df7ee3583830c5351067e476d70d6a3e86715dccb065fba235a218827f45`). The shared property decoder now applies the codec's exact-integer rule to raw edit values; preview updates, composition property edits and inserted-node properties preserve raw tokens until schema-aware decoding. The nine transport tests cover exact edits, lossy integers beyond f64 precision and i64 range, and atomic composition rejection (log `a95cf8a9a8e081fe3dc70af3aeda3cf24dc03b22eac22adfe20ccb65b6ef7177`). The preview suite passed 45/45 and Playground library passed 220/220 (logs `a95d5819049298b6f18efba485c9e189f4ae4d4b504019935ae17864ef28b6ce` and `e6b88b501b24d66c5b3ba9fbb52920182fad49c467b9c4aa95aaf55611bde767`). The optimized Web build passed with three notice gaps (log `52a0ca6055591d047cd87ba513b92322aa999044eed02c8e623317e07d828019`, manifest `4f33b657e49833bc8f882801ef31329de51372a8bbf9263e809c77c6984d0d30`; all 2,943 captured source hashes matched). In one Chromium reference-host session, `amount = 1` was accepted and rendered as `1.0`; a large invalid edit was rejected with the prior revision intact. Its schema also limits amount to 0–1, so that browser rejection alone does not isolate precision validation. The owned preview was disconnected and the tab and server closed. Native and broader browser acceptance remain open.

- **R31 notice source correspondence, 2026-10-28:** The vendored original manifest and Rust source match the recorded upstream revisions for `hexf-parse 0.2.1` (2/2 files, revision `4225763d744183d720f575ae96d04161b4d08ea0`) and `seahash 4.1.0` (7/7 files, revision `94b632aeac099031c373599313d5b5f0acbbaec0`). Exact URLs and hashes are retained in `target/headstart/evidence/notice-source-correspondence-20261028.json` (SHA-256 `2efe97769264f39236280b98f6f91e8b42cecf4d2468d151706b8a4c3c77599e`). This verifies those files only, not full package identity or permission-text coverage. Both packages remain unresolved notice gaps; licensing and release clearance remain open.

- **R22/R24 composition transport regression, 2026-10-28:** A Playground protocol test now mounts a valid composition and submits replacements with unknown document and node fields, asserting both return `invalid_document` and leave the mounted document and stamp unchanged. All seven focused preview transport tests pass with Headstart (log SHA-256 `7bd03555b0a410a7c48b67684749fb25d3b62a6d4353fa59c41ef4c7a66a1ecd`). This makes the previously observed bounded Chromium behavior repeatable at the transport layer; it does not close native, cross-browser or full composition acceptance.

- **R22/R24 browser strict composition import, 2026-10-28:** In one in-app Chromium session, the same-origin composition reference host returned `invalid_document` for unknown top-level and node fields, then mounted a valid draft. A subsequent unknown-node replacement was rejected; export returned the original document at the same generation/revision. The host disconnected and unmounted before the owned tab and server were closed. The tested bundle manifest SHA-256 was `e3be1de67539378e45b15830f0453d0718fb4100b6b99997c71cb887d46696e0`; all 2,943 captured source hashes matched commit `b7ebdb96`, including the composition decoder. This is bounded browser protocol evidence; native, cross-browser and broader composition acceptance remain open.

- **R22/R24 composition decoding, 2026-10-28:** The custom composition decoder now rejects unknown document and node fields, matching the public structures' existing strict schema instead of silently discarding input. A regression first failed on an unknown document field (log SHA-256 `ec302a848cd9f32bcef795247c7209e8f5d92b1fd2dd1aaf8be9c285cdc6bdb3`); after the fix it verifies both levels and all 45 preview tests pass (log `463ffecc3ea484777b6f234a6f84ec14ea91fa5acfe29d688701e47732d81b6d`). The optimized Web build and 18 IME-plan tests passed (log `39182b1424baa3e24dd8a777b0672017542b86bf29dd81916bc1a5e610e47503`, manifest `e3be1de67539378e45b15830f0453d0718fb4100b6b99997c71cb887d46696e0`; three notice gaps). This is codec/source and build evidence, not browser import or full composition acceptance.

- **R26 Web HTTP error cleanup, 2026-10-28:** The theme loader now starts cancelling a failed HTTP response body before reporting its status, without waiting for a stalled cancellation promise. A focused regression failed before the fix because cancellation was never initiated; all 15 theme-loader Node tests pass afterward. The optimized Web build and 18 IME-plan tests passed (log SHA-256 `d7c183c24679d2e893b4fc7bd98a4e3b981c25586ea85a961d40c1dc6f52fd82`, manifest `a06b4c1d5a66fd566c22136a849ac63caf3564061e45e14ddb0d6f0d8a3cb269`; three notice gaps). In one Chromium session a temporary harness using the packaged module, whose SHA-256 matched source `47a27f47e83eeb2ea19ca69a83a2a427d80bddc68f0dd1d92b110c9845347aaa`, observed prompt HTTP 503 reporting and initiated cancellation for a simulated stream whose cancellation never settled. The owned tab, server and temporary harness were removed. This is a bounded error-path check; real network stalls and wider browser acceptance remain open.

- **R30 current source and macOS release gates, 2026-10-28:** On clean commit `74a47b6681c25c5fd0fd571a25c3e08470d3cfb4`, `./scripts/verify-release.sh source` passed 598 Python tests plus JavaScript, package, locale, theme and provenance checks (log SHA-256 `771f52d5d2e61a3fe10086030566122927031c2babcaa443178e842ed65a8da9`). The GitHub [source job](https://github.com/abouzoubaa/rustui/actions/runs/40706570264) passed for the same commit; its macOS/Web job was skipped on push. Local `./scripts/verify-release.sh macos` passed 1,391 tests across 41 groups, the locked external-consumer check and optimized Playground packaging (log `56dea09e10ae6e1900d05e2450fa9ea89becbaff0e64d334cb8ce5a00b0e1a13`). The macro doctest group had zero cases, and five native dependency-notice gaps remain. These are source, compiled-test and packaging results; they do not establish current compiled CI, native interaction, accessibility, performance, licensing or distribution acceptance.

- **R13 supplementary story locale and alignment, 2026-10-28:** Remaining visible supplementary-story copy and counts now use English, French and Arabic catalogs. A direction-aware RustUI `HStack` places the story at logical start. Locale metadata validation and the catalog initialization suite passed (3/3; log SHA-256 `e70f1326c09b9edb143c559fb61deacff597216a796446ddebdd7a838cba2f8f`); the final optimized Web build and 18 IME-plan tests passed (log `eed81c4d57c2ebee0a33361dac8fafa69a1b3017004566d93663ee3bf9f01e31`, manifest `b83b3d6e21e788f518744ee0a50e7d644df7da56c5ec49cd1c81d4322fef49b7`; three notice gaps). In one Chromium session the final story appeared at the right in Arabic and left in English, scrolling exposed lower Link controls, and the Arabic pointer action updated its count to ١. Before the alignment change the French story displayed translated text, its pointer action updated to 1, and scrolling exposed its lower controls. This resolves the previous screenshot's apparent clipping for that viewport; native, keyboard, cross-browser and broader RTL acceptance remain open. The Inbox button's dynamic accessible name remains English; accessibility is deferred. The owned test tab and server were closed.

- **R13 supplementary Link story, 2026-10-28:** The enabled and disabled Link labels and activation count use English, French and Arabic catalogs; the disabled name follows the localized label. The Playground Headstart check and locale metadata validator passed. An initial `{ $count }` entry caused a runtime catalog assertion despite the metadata validator and Web build succeeding; it was corrected to the supported `{count}` syntax. The actual catalog initialization tests then passed 3/3 (log SHA-256 `f35f7b9ce3a34ca35200d893ee9d5b04e1622504ef5fa570fdb39d7807c45b09`), and the optimized Web build passed its 18 IME-plan tests (log `aee985ee4f5d1af24c50738b73e73fe1aacc0bccbe129699e289b250463c9615`, manifest `0c29de6cc9d97250563ee392d6006db1a01ef787921d26ed162490dc038124d7`; three notice gaps). In one Chromium session the corrected app started and showed the translated enabled Link in French and Arabic; the semantic tree exposed both translated links and the localized zero count. The remainder of this story still contains English text, and its lower content was clipped at the observed viewport. Native, interaction, wider RTL and cross-browser acceptance remain open. The owned browser tab and server were closed.

- **R13/R15 Toolbar story alignment, 2026-10-28:** The two standalone story actions now use direction-aware RustUI `HStack` wrappers. The preceding session observed them at the right in Arabic and at the left in English in one in-app Chromium session; both actions responded in Arabic. The final source passed the Playground Headstart check and the optimized Web build, including 18 IME-plan tests. Bundle manifest SHA-256: `60e484387799748dbbd2517daec8ae4e0d9c09ca8b6a5bf38637f8f3c1105019`; build log SHA-256: `ea0d6dc72cf4c03b9e3dccd07cd9818fd848b73374af29a19a51cba182ecdad7`. The local test server was stopped; browser inventory timed out during cleanup, so test-tab closure is unconfirmed. This is a story-specific bounded observation, not general Button RTL or native, keyboard, or cross-browser acceptance. Three Web dependency-notice gaps remain.

- **R13/R15 RTL Toolbar, 2026-10-28:** RustUI now passes its layout direction to the pinned Components Toolbar. The reproducible `gpui-toolbar-rtl-v1` patch mirrors the RTL row and reverses left/right focus traversal to match the visual direction; default LTR behavior is retained. Patch replay and altered-source rejection passed 2/2, the RustUI Toolbar Headstart suite passed 9/9 including GPUI test-window focus movement in both directions (log SHA-256 `05781ef4f1f3330b733f7034023543759d77803faefd3d4f1c38901629b79cd3`), and source release checks passed 598 Python tests and accompanying checks (log `48052eb92c3fd1a54c4dd2f1e7b9bf31f58320912498cc8e2ecb3fc83fb460dc`). The optimized Web build passed (manifest `911e5560b002340df1a625939708da5470b0564c6ca28fa65f3c0f2326107314`, build log `9f082719662f0c1963b00b0463230b324a8f33971cf6225eae2cb78fbc37d5ed`), with three notice gaps. In one Chromium session the Arabic Toolbar appeared at the right with mirrored item order; Increment and Reverse worked, while English remained left-to-right. The separate story action buttons remain left-aligned in Arabic. Browser physical-keyboard focus was not established; native, broader RTL, keyboard and screen-reader acceptance remain open. The test tab and server were closed.

- **R13 Toolbar story localization, 2026-10-28:** The existing Toolbar story's title, actions, status and help now resolve from English, French and Arabic catalogs; counts use numeric arguments so Arabic digits follow the active locale. Locale validation passed for all three catalogs, the existing toolbar-gallery tests passed 3/3, and the optimized Web build passed with three notice gaps. In one in-app Chromium session on the first bundle (manifest `3ceb241a117f74e4e77038d9036b4e3ba09bbeb82059e7b8a3bd816a6aae8b0b`), French and Arabic labels appeared and Increment and Reverse updated their respective statuses. After switching counts to numeric arguments, the final bundle (manifest `5bd538bfd5b789af37c7f5f9976cfed1c551c45c453148a6a9c343bbba43fcb7`) displayed Arabic ٠ and changed it to ١ on Increment. The Arabic toolbar controls appeared left-aligned within a right-to-left story; wider RTL layout, native, keyboard and cross-browser acceptance remain open. The test tab and owned server were closed. Build log SHA-256 `2207c6d112442084a623d808612fcf92d3a89300987c16d5bba041826760956a`.

- **R15 public DatePicker boundary ownership, 2026-10-28:** A Headstart GPUI test-window regression rendered the public RustUI `DatePicker`, updated its retained owner from `NaiveDate::MIN` to `MAX` and back, and verified the control's value and stable identity after each update. The focused test passed 1/1 (log SHA-256 `45d1638f18cfeb64c450fb77eb849ae81a424269160139e82fd28afdc8257eba`). This checks retained owner-to-control synchronization and the test-window semantic value; it does not exercise the popup or establish native/browser visual or input acceptance.

- **R15 DatePicker calendar date limits, 2026-10-28:** The versioned `gpui-base-calendar-date-limits-v1.patch` repairs the previously confirmed Chrono boundary panic without changing the pinned package version. Calendar grids use empty padding for dates outside the supported range, month/year navigation and available multi-month views stay within the range, and public month data omits unrepresentable padding. The patch script verifies package selection, preserved upstream checksum/license, exact source and patch hashes, and reversible replay; its two replay/tamper tests pass. The RustUI DatePicker Headstart suite passed 20/20, including actual GPUI grid rendering and disabled boundary navigation at both limits and Sunday/Monday/Saturday week starts (log SHA-256 `3e5cdae0eb2d1a3c4f65428bb1a08ccd7a6c58b9791ca1b32c7691f8e54b72a0`). Source release checks passed 596 Python tests and accompanying checks before the patch artifact was reformatted; exact replay and tamper tests passed again afterward (source log `57ddedfa0cf6970311dba7c789938f1dea9eb13de983fb3fa69d2e4fc2a64740`); the optimized Web build and 18 IME-plan tests passed (log `ac94f5135453d84d24bfa951113492cf203a1e31d25fd985d9ad9aec979abada`, manifest `4271f45710cf5fca95b44ea300569202df104b5819c9862333f795f6f268ec1c`). The Web build predates a subsequent test-only edit. Three Web notice gaps remain. This is coding, GPUI test-window and build evidence; native and Web visual/input acceptance at the limits remains open.

- **R15 DatePicker year-list boundary, 2026-10-28:** RustUI now clamps the year list around a selected date to Chrono's representable minimum and maximum years; the exclusive upper endpoint still includes the maximum year. Both focused Headstart year-range tests passed, including selected minimum and maximum dates. Log SHA-256: `2b55f94214f768403b9578de48cae30579e3f6e220a9fd7e33a88b1a254c71ba`. This checkpoint prevented invalid year-list entries only; the later calendar patch and its remaining acceptance are recorded above.

- **R15 DatePicker extreme-date defect, 2026-10-28:** A temporary GPUI test constructed the pinned Components `CalendarState`, selected the valid `chrono::NaiveDate::MAX`, and caught a panic from `gpui-base-0.7.3/src/calendar.rs:837`: generating December's grid unconditionally constructs the next January, outside Chrono's range. The probe passed only because it expected the panic; this is a confirmed defect, not an acceptance pass. Log SHA-256 `283596ac789d43e971aa822145f04e44fd149d4e7b49a89121eb00794c3f0f31` and the temporary test patch SHA-256 `5432676e84da25d972cde778293bb96ea82bc82ab2b3611b735395370bf6e595` are under `target/headstart/evidence/calendar-extreme-probe-20261028.*`. The temporary test was removed from source. The subsequent reproducible patch and its remaining acceptance are recorded above.

- **R14 Fluent reference traversal, 2026-10-28:** Catalog validation previously revisited shared Fluent reference chains exponentially and used recursive stack frames. It now traverses iteratively, retaining active-cycle detection and tracking completed references separately for caller and term-local scopes. The Headstart-profile i18n suite passed 107 tests/doctests, including a 100-message shared-reference graph, a cycle spanning a message and term, and a reference reached in both scopes. Log SHA-256: `3393830160015df849c65fef363751a1dcd990371217b7a5d9e0957850dd1f75`. This is a bounded source regression, not a performance benchmark or full locale acceptance.

- **R14 Fluent term validation, 2026-10-28:** Catalog validation previously accepted missing or recursive Fluent term references even though rendering those messages fails. It now follows term references for existence and cycles while keeping term-local variables out of the caller's argument contract. The new regression failed before the fix; the `rustui-i18n` Headstart-profile suite passed 105 tests/doctests, including different local term-variable names in English and French. Log SHA-256: `dd2aa4ab977b5ace2f4fe954db07edf27bd84fd95f609c3848e9e9f1b3fab78f`. The first default-profile suite attempt hit an incremental compiler ICE involving `icu_decimal`; reports are preserved under `target/headstart/evidence/fluent-ice-20261028/`. This is source-level validation, not native or browser locale acceptance.

- **R30 compiled CI, 2026-10-28:** GitHub Actions [run 40695765478](https://github.com/abouzoubaa/rustui/actions/runs/40695765478) passed the source job, trusted checkout verification, macOS checks and optimized package, and Web optimized bundle for commit `365882d1ca62a8dd7ed9a0cd4668b4ab8370d0b5`. The ephemeral macOS ARM64 runner archive matched the official GitHub release SHA-256, and all 9,390 extracted regular files matched it before registration. The runner exited successfully, removed its credentials and registration, and no runner remained registered. This is CI/build evidence for that commit, not native or browser interaction, license completeness, cross-machine reproduction, or signed distribution acceptance. The subsequent ledger-only commit does not change the tested product source.

- **R26 stalled stream cancellation, 2026-10-28:** A focused JavaScript regression reproduced an invalid UTF-8 response hanging when the response stream's cancellation promise never settled. The browser theme loader now starts cancellation without awaiting it, then propagates the original decode or size error and releases the reader lock. All 14 theme-loader tests pass, including the stalled-cancellation regression; this Node test uses a simulated stream and does not establish full reload acceptance.

- **R26 browser cancellation check, 2026-10-28:** On commit `4357f68a`, `./scripts/verify-release.sh web` passed with the existing Wasm Headstart root: 18 IME-plan tests passed and the optimized bundle was produced, with three dependency-notice gaps still reported. Log SHA-256: `1b78f2348b6d927f52fa078f30be06dbfb9c9fa60dc2a321e98707f627ea937a`; bundle manifest SHA-256: `22502614bf7a622d118af0908d2dcdfe4979f919e2a4e3b6ec786bdc31cc3375`. A temporary localhost harness imported the bundle's theme-loader module, whose SHA-256 matched the source (`9ae8fd8c8968cbd571ce944c812f988c9334f5d6dc06b9abde096e73d7cf6f73`). In one in-app Chromium session, a simulated stream containing invalid UTF-8 whose cancellation never settled returned the decoding error and initiated cancellation. The test tab and server were closed. This validates that narrow browser error path, not real network stalls or full reload acceptance.

- **R24/R25 Safari composition rendering, 2026-10-28:** In Safari 27.0 on macOS 27.0, the same-origin reference host mounted a `composition-column`, accepted insertion of a `composition-text` child, and visibly rendered `Safari child one`. A `set_property` edit was accepted at the current stamp (revision advanced from 1 to 2), and the rendered label changed to `Safari child updated`; the browser semantic tree also reflected both labels. The host reported disconnected and unmounted before the owned Safari window and localhost server were closed. The bundle manifest SHA-256 was `8eb9b4088974c4e3588a819bd924a6913dfb1cf7060b784194f154b96de66aa7`; all 2,943 captured source hashes matched the checkout. This is a bounded insert/property-edit observation, not full composition or cross-browser acceptance.

- **R24/R25 Safari preview transport, 2026-10-28:** In Safari 27.0 on macOS
  27.0, the same-origin reference host reported PASS for both canonical compiled
  preview and composition roundtrips, including stale-stamp rejection and
  disconnect/reconnect. A separately drafted composition mount was accepted,
  then explicitly disconnected and unmounted. The test used the Web bundle
  manifest SHA-256 `8eb9b4088974c4e3588a819bd924a6913dfb1cf7060b784194f154b96de66aa7`;
  all 2,943 captured source file hashes matched the current checkout. The
  separate Safari window and localhost server were closed. These are bounded
  protocol and mount observations; the rendered-child check is recorded above. Other browser
  versions, keyboard/IME and broader preview acceptance remain open.

- **R26 Safari strong-ETag watcher, 2026-10-28:** Safari 27.0 on macOS 27.0
  loaded the source-matched Web bundle (manifest SHA-256
  `8eb9b4088974c4e3588a819bd924a6913dfb1cf7060b784194f154b96de66aa7`;
  all 2,943 captured source hashes match, although its Git field records the
  parent because the build preceded commit `6dfbe56d`).
  In one localhost session, the first automatic theme request transferred
  821,287 body bytes and matching strong-ETag requests returned bodyless 304s.
  Changing the ETag and adding JSON whitespace caused a fresh 821,288-byte
  response followed by 304s. Stopping and restarting caused an unconditional
  first fetch; removing the ETag caused subsequent full fetches. The watcher,
  separate Safari window and owned server were stopped. Server log:
  `target/headstart/evidence/theme-strong-etag-safari-20261028.log` (SHA-256
  `1379916dd5969af22f95eb761410df9d1381e649c5c89f749dc82da3b68f4846`).
  This is bounded network and UI evidence, not a semantic theme-change test,
  browser-version matrix or full performance acceptance.

- **R26 Web watcher cache generation, 2026-10-28:** A deterministic test
  transport that ignored cancellation reproduced a stale request overwriting
  a cleared and reused theme cache. Cache requests and clears now invalidate
  older generations, preventing late responses from writing or clearing a
  newer cache. The reproduction failed before the fix and all 13 loader tests
  pass after it, including overlapping requests and stale `no-store` responses.
  The first Web build correctly rejected a source change during packaging; a
  stable retry succeeded (manifest SHA-256
  `8eb9b4088974c4e3588a819bd924a6913dfb1cf7060b784194f154b96de66aa7`).
  In one in-app Chromium session, that bundle fetched the theme once and
  continued with empty-body 304s; the test tab and server were closed. The
  browser check does not reproduce the late-response race. Test and browser
  logs are under `target/headstart/evidence/theme-cache-*-20261028.log`.

- **R26 Web automatic theme transfer, 2026-10-28:** The Playground watcher
  now keeps an owner-scoped response cache and conditionally revalidates only
  strong ETags. A matching 304 reuses the bounded prior response; absent or
  weak validators use full fetches, invalid 304 validation retries without a
  validator, and `no-store` clears the cache. Stop/unmount clears the cache;
  manual and inspector reloads remain unconditional. Ten JavaScript tests,
  the native owner-lifetime regression (1/1), formatting, and the optimized
  Web build passed. In one in-app Chromium session, the final bundle (manifest
  SHA-256 `85e68abcee56f3c8973e805c977b2c9d03c215e3b733408b7d5cf90c00b6d9f7`)
  transferred 821,287 response-body bytes initially, then zero for matching
  304s; a no-store 304 caused an unconditional 200 and later full fetches.
  In earlier builds, changing the strong ETag transferred changed data,
  and stopping/restarting the watcher caused a fresh first request. Browser
  server logs are `target/headstart/evidence/theme-strong-etag-verified-browser-20261028.log`
  and `theme-strong-etag-final-browser-20261028.log`. The test tabs and servers
  were closed. These are bounded local observations; server correctness,
  browser variants, CPU/memory measurements and wider performance acceptance
  remain open. Servers without a strong ETag still transfer the full theme.

- **R31 lockfile advisory refresh, 2026-10-28:** OSV batch queries for all
  834 distinct registry package/version pairs in lockfile SHA-256
  `553f12f725d6e74b6f20950308dca47c2d09df872d1286a2e9de8c9dfe30e27a`
  returned nine advisory matches with no query errors or paginated results:
  eight unmaintained-package notices and `GHSA-qwgh-2vcv-g2f7` for
  `block-buffer 0.10.4`, describing panic-related buffer corruption. The
  default-feature Playground reverse trees for `aarch64-apple-darwin` and
  `wasm32-unknown-unknown` both reported no active path to that version; an
  all-target tree includes a path through `gpui-pre-linux`. This does not clear
  other feature selections or future targets. The previously recorded
  `rustls-webpki 0.103.15` is no longer in the current lockfile, which has
  `0.103.16`. Raw query responses, advisory details and trees are retained in
  `target/headstart/evidence/osv-lockfile-20261028/`; report SHA-256 is
  `8b6ac3b3eec5903e973632f857724dfa01aa96e6e97355c319f7dba620486b3b`.
  Review of maintenance notices, license obligations and other feature
  selections remains open; this is not a clean security bill.

- **R13 Arabic catalog count wording, 2026-10-28:** The Playground header
  now labels the catalog and demo totals in Arabic without using an incorrect
  plural after the number 16. Locale validation, formatting, the focused
  catalog-summary test (1/1), and an optimized Web build passed. In one in-app
  Chromium session, the rebuilt bundle displayed `عدد صفوف الدليل: ٤٦ · عدد
  أمثلة إطار العمل: ١٦ · macOS 27`; its manifest SHA-256 is
  `5320af3fd0f78f0983411ce08936a9fbbbc37ba26d13dc8bd33ecedc95532b65`.
  The test tab and server were closed. Native, wider browser, and complete
  Arabic/RTL acceptance remain open.

- **R03 RustUI public doctest, 2026-10-28:** On clean `eead9002`, the offline
  Headstart workspace doctest run passed one `rustui-i18n` example; the other
  five crates had zero doctests. RustUI's crate documentation now includes a
  public `View`, `VStack` and `Text` composition example, and the focused
  `rustui` doctest passed 1/1. Logs:
  `target/headstart/evidence/doctest-workspace-eead9002-20261028.log` (SHA-256
  `c1d32496790c7acdfe6eee0d3a94f9e5958665f986917f8fd2ddd024a79fe870`)
  and `target/headstart/evidence/doctest-public-example-20261028.log`
  (SHA-256 `70d68cf3b1a96fce75a24c8912c3a9f00b7d43b85eb12e97d77b0e00b424c767`).
  One composition example does not establish complete API documentation or
  coverage of the remaining public examples.

- **R03/R30 workspace documentation build, 2026-10-28:** On clean commit
  `4c127c2c`, an offline Headstart `cargo doc --workspace --no-deps` in the
  persistent verification target succeeded on retry. The first attempt exited
  unsuccessfully, but its exact diagnostics could not be recovered from the
  compressed tool output; its cause is unverified. The successful log is
  `target/headstart/evidence/doc-workspace-4c127c2c-20261028.log` (SHA-256
  `ad7d16be6c9596a95d0b8a4eddee23475f4ddcc4143aead0b27ddd1d47846544`).
  It contains 1,256 warning headers, including many upstream deprecations,
  including 27 warnings attributed to the Playground library. Documentation
  completeness, examples, doctests, warning review and release acceptance
  remain open.

- **R31 notice provenance, 2026-10-28:** Upstream version tags for three
  unresolved notice packages were compared byte-for-byte against all vendored
  `.rs` files: `block 0.1.6` tag commit `47178790cfc9d4a8b092051d8b413b78bd31254a`
  matched 2/2 files; `malloc_buf 0.0.6` tag commit
  `a7811e5f4c6f9685dd57ad9bfe34e9bf6b0ba4f9` matched 1/1; and `mac 0.1.1`
  tag commit `66afc663b68a65633ea165c742b5a9c6734581c2` matched 8/8. The
  respective upstream tag roots contain no license file. These comparisons
  establish source correspondence only, not complete package identity or
  permission-text coverage; all three notice gaps remain open.

- **R26 Web reload error display, 2026-10-28:** A real-browser oversized
  response exposed a JavaScript stack and localhost URL in the visible reload
  status. RustUI's Playground now extracts the JavaScript error message and
  uses a generic fallback for unknown rejection values. Formatting, six focused
  loader tests and the optimized Web build passed. In the in-app Chromium
  browser, a local server advertising a 3 MiB uncompressed theme produced the
  concise `Reload error: Theme response exceeds 2 MiB`, without the stack or URL;
  the owned tab and server were closed. The tested bundle manifest SHA-256 is
  `6980e00c7b40b0bcb7a46ed9757468f74f970f56c3e027f3f2517faf9e788e58`.
  This is a bounded browser check, not broader browser or bandwidth acceptance.

- **R26 oversized Web theme responses, 2026-10-28:** The browser loader now
  cancels a response declaring an uncompressed body above 2 MiB before reading
  it. Compressed responses continue through the existing decoded-stream limit,
  since Content-Length describes encoded bytes; missing or unreliable lengths
  do not replace the stream limit. Six focused Node tests passed, including
  early cancellation, compressed headers, streaming limits and owner abort.
  The optimized Web package built successfully (manifest SHA-256
  `1abb6cfd895d88f18a58c8aa4b5b9de5a125a964994c43069d774713ce5237a5`),
  and the packaged loader hash matches the source. This does not reduce the
  repeated transfer of a valid theme or establish browser performance acceptance.

- **R26 cache validation investigation, 2026-10-28:** The browser loader
  deliberately uses `no-store` and automatic reload requests the approximately
  821 KiB theme every 350 ms. A local Python HTTP server experiment changed a
  file while preserving the same whole-second modification time: an
  `If-Modified-Since` request returned 304 even though an unconditional fetch
  returned the changed bytes. Switching blindly to conditional browser cache
  revalidation can therefore miss rapid edits on such servers. Preserve the
  current freshness behavior while investigating bandwidth improvements that
  work without reliable strong validators. This is a local protocol experiment,
  not a browser performance benchmark or release acceptance.

- **R26 Safari theme reload, 2026-10-28:** Safari 27.0 on macOS 27.0
  loaded a temporary copy of the Web bundle built from `ae736006` (manifest
  SHA-256 `5e344dfc43942cf3bb625a31c64a8fd09f29290e8e2f3cdbc3707a366c3ff59e`).
  The Divider foundation story reported successful Light reload, rejected
  deliberately malformed theme JSON with a parse error, and reported successful
  Dark reload after restoring the valid file. The probe changed only the
  temporary copy, verified its restored data against the packaged original,
  closed its separate Safari window, and stopped its localhost server. This
  does not establish inspector error handling, automatic polling, visual theme
  fidelity, or broader cross-browser acceptance.

- **R13/R20 owner-task visible localization, 2026-10-28:** Playground's owner-task title, controls and visible idle/loading/success/error states now resolve through English, French and Arabic catalogs; unknown result/error values remain interpolated. Locale validation, two focused tests, formatting and Web build passed (manifest SHA-256 `5e344dfc43942cf3bb625a31c64a8fd09f29290e8e2f3cdbc3707a366c3ff59e`). In one in-app browser session a restarted success remained visible and translated after English-to-French-to-Arabic switches; an Arabic error action displayed the translated failure. The existing live-region and accessible button descriptions remain English and are undone/deferred accessibility work. Native and wider locale/RTL/browser acceptance remain open.
- **R30 current native test integration, 2026-10-28:** On clean commit `ec114476`, locked/offline Headstart workspace library tests passed 1,326/1,326 across six result groups and integration tests passed 56/56 across 34 groups, with none ignored. Logs are `target/headstart/evidence/stage-integration-lib-ec114476-20261028.log` (SHA-256 `a379359b4ef17b5c64735d9d686794fa5f44cfbfdb379d831bf6d2574ed9e14b`) and `target/headstart/evidence/stage-integration-tests-ec114476-20261028.log` (SHA-256 `e8926e6eadc346859153637f16640a15032d289ea77a94095e5d2403e8d38dbd`). GitHub [run 40685391014](https://github.com/abouzoubaa/rustui/actions/runs/40685391014) passed source CI on the same commit; its compiled macOS/Web job was skipped. Native GUI, current compiled CI, notices, packaging/distribution and wider acceptance remain separate.
- **R09 recent-search initial state, 2026-10-28:** The Playground Text inputs story now starts its recent-search popover closed, so it does not cover the story on entry. Five focused search-field tests and formatting passed; the Web bundle built (manifest SHA-256 `f75b1c8c5f7b933b082ad13cdcbd680e5a5c8b7db9d0fed9f853fc1827bfc261`). In one in-app browser session the menu was initially closed, opened on its button, and selecting `Studio` updated the field and dismissed the menu. Native, keyboard and cross-browser acceptance remain open.
- **R16 Web IME attempt, 2026-10-28:** In the in-app browser, an automated Option+E in the focused owner-bound Name field inserted committed `e` into both field and owner instead of producing an observable marked preedit. This does not exercise external owner updates during composition and does not establish real-OS IME acceptance. The test tab and local server were closed.
- **R26 Web theme inspector reload, 2026-10-28:** The read-only inspector now uses the bounded browser theme loader on Web rather than re-reading the compile-time embedded theme. Reload errors retain the last valid summary and preview, and unmount cancels an outstanding request. Focused native inspector tests passed 2/2; formatting and the Web build passed. In one in-app browser on manifest `6030260f37da5bf06fdedc70ff4447fa70190233a47f2069fba56156876e4fa4`, a changed served `source.kind` appeared in the normalized preview, invalid JSON produced an error while preserving that preview, and restoring the asset cleared the error and restored the original value. A four-second delayed request disconnected after the story unmounted; server log SHA-256 `f5a807959062365d0eff26f8cd3e1f387c5db3e6b247645e9fced2f16b0b037a` is retained at `target/headstart/evidence/web-theme-inspector-delay-20261028.log`. After correcting the explanatory label, the Web bundle rebuilt (manifest `61c6b7f5f3259cbe7cf7d05c7aff2586f1a8085f566f0bdbc7eb29074bf6b735`); browser observations remain tied to the preceding bundle. Cross-browser, native GUI and broader R26 acceptance remain open.
- **R26 delayed appearance race, 2026-10-28:** A four-second Web response exposed a defect: selecting Dark in the Playground after starting a Light reload was overwritten by the late Light response. Successful RustUI theme activations now advance an `ActiveTheme` revision; manual browser reloads discard results if that revision changed after the request began. The focused GPUI test passed 1/1, including unchanged revision after failed activation; the native Playground check and Web build passed. In one in-app browser on manifest `3053d2137e76a5760525f0c951c0988b3d0c55ac625e47a91af2e50d3a40b222`, the original sequence stayed Dark and reported the reload superseded; a later uncontested Light reload applied. The owned tab/server were closed; server log SHA-256 `f1214317523fbe44dd923fa985354f1074fd04bebccbc3e6a3c92828bf7fffdf` is retained at `target/headstart/evidence/web-theme-revision-20261028.log`. This is one browser and one delayed-response case; wider concurrency and platform acceptance remain open.
- **R26 Web streamed response regression, 2026-10-28:** The theme loader's JavaScript suite now passes 5/5 cases, including decoding Arabic UTF-8 split across chunks and canceling after a response body has begun but stalled. This exercises a simulated stream; real-browser slow-body, cross-browser and broader performance acceptance remain open.
- **R26 Web request cancellation, 2026-10-28:** Browser theme requests now have a cancellation handle held by the Rust future; dropping the task aborts its fetch rather than waiting for timeout. Four JavaScript tests passed, including explicit cancellation and timeout, and the optimized Web bundle built (manifest SHA-256 `0d71ae07215ad060b88ff9a33aa93d9155639c4f6c2b8b7cfef810be001da318`). In one in-app browser session with a four-second local server delay, unmounting during manual reload and stopping automatic polling each caused the server to observe a disconnected request. A Light request replaced by Dark disconnected the first request and applied Dark when the second completed; a manual Light request also completed while polling was active. The tab and owned server were closed. Server log: `target/headstart/evidence/web-theme-delay-20261028.log` (SHA-256 `03fa518490eb468dad1b7503651dcdfa8a4748b08f3c7a02c8dd31f54355cbc1`). This is bounded one-browser evidence; cross-browser, slow-body and wider lifecycle/performance acceptance remain open.
- **R26 Web theme data reload, 2026-10-28:** Playground now packages the theme JSON and fetches it from the same origin without cache, limits responses to 2 MiB and requests to 10 seconds, and validates/activates through RustUI's public theme API. Weak ownership, request generations and manual-pending arbitration reject stale completions; polling uses the current appearance mode. Only already-locked `js-sys` and `wasm-bindgen-futures` versions were added as direct Web dependencies. JavaScript tests passed 3/3, the Web packaging fixture and native ownership test passed, and the optimized Web bundle built (manifest SHA-256 `c7863863c35ffa5d14b2737a6836f00747bba3455bf341ab81eda438e6838ed2`). In one in-app browser session, an earlier bundle with the same Rust implementation (manifest `20680d10b5370ce50cf6af41fe0850105f17ab415465d1f1977a32aaba609a6c`) applied theme data, retained the appearance on invalid JSON, recovered after repair, applied changed valid data, and remounted with polling stopped. The final bundle was separately observed applying Light. The served asset was restored byte-for-byte and the test tabs and servers closed. This is bounded browser evidence; delayed-response races, cross-browser, native event paths, polling bandwidth/performance and broader R26 acceptance remain open. At this checkpoint, dropped requests could continue at the network layer until timeout; the subsequent cancellation repair is recorded above.
- **R26 Web theme-reload diagnosis, 2026-10-28:** The earlier `3cfa60701359baf15400522a3d13f54a6eb8c507ebfd93d5c7ab2b86adbb36b6` Web bundle reported `Open error: could not read theme file: operation not supported on platform` in the Divider/Theme Layout story. It passed a filesystem path to `ThemeFile` and did not package a browser-fetchable theme. The subsequent Web data path and its bounded evidence are recorded above.
- **R30 source CI, 2026-10-28:** GitHub run [40680465446](https://github.com/abouzoubaa/rustui/actions/runs/40680465446) passed its source job on `4e3dc72c`; the macOS/Web build job was skipped by workflow policy. This is source CI evidence only.
- **R26 theme watcher owner lifecycle, 2026-10-28:** Playground's manual and automatic theme-reload actions now hold weak, unmount-invalidated leases. An escaped action can no longer retain watcher state or restart a watcher after unmount; a newly created lease is valid after the epoch changes. A focused ownership/epoch regression passed 1/1, the existing theme watcher tests passed 7/7, and formatting and the Web bundle build passed (manifest SHA-256 `3cfa60701359baf15400522a3d13f54a6eb8c507ebfd93d5c7ab2b86adbb36b6`). These are source and build checks; dispatched native/Web event-path acceptance and broader reload acceptance remain open.
- **R13/R35/R36 graph owner-edit errors, 2026-10-28:** `GraphError::localization_key` exposes the existing framework message mapping; the Playground retains the typed error and resolves it in the active catalog instead of storing an English string. Its focused graph tests passed 2/2, including the retained error, all three catalog values, and clearing after a successful edit; the existing RustUI graph localization/fallback test passed 1/1. Locale validation, workspace formatting, and a Web bundle build passed (manifest SHA-256 `5ac08a73293aa385c5a6e288eb23b5f0f2d5ecc6b591deb906628e84acd47594`). Direct graphical error and locale-switch acceptance remain open; example node/port data are still untranslated.
- **R13 native selected-date rendering, 2026-10-28:** The macOS 27 Playground package from the preceding successful release gate had executable SHA-256 `9ecaa0ec2d93ef82c4bf3cd760afd30a3619ddb4777cdaeb334579d242ffc24f`, matching the release executable. In Arabic, selecting 28 October 2026 showed the complete visual date `2026/10/28`; the native accessibility value was `28\u200f/10\u200f/2026`. Switching to French retained the selection and displayed `28/10/2026`. This is one native case, not broader RTL, locale, or platform acceptance. The owned process exited; automatic command review rejected removal of `target/headstart/debug/RustUI Playground.app`, so that bundle and the GUI-slot claim remain preserved pending cleanup. The earlier rejected bundle under `target/headstart/verification/debug/` is also preserved.
- **R30 external consumer and macOS release gate, 2026-10-28:** The first current-source macOS gate failed at the external consumer's `--locked --offline` check because its independent lockfile omitted dependencies added by the ICU calendar and editor changes. Regenerating it offline added five package records with versions, sources, and checksums matching the workspace lockfile, without changing existing locked versions. The rerun of `./scripts/verify-release.sh macos` exited 0: 1,378 tests passed across 41 result groups, the locked external-consumer check passed, and the optimized Playground package built. Log: `target/headstart/evidence/release-macos-consumer-lock-20261028.log` (SHA-256 `ae17e5dcbc672d03b4b5f010a514cf21e2287692dac5706bd3429996b0713748`). Five native notice gaps remain; this is not native interaction, visual, performance, or distribution acceptance.
- **R30 source release gate, 2026-10-28:** `./scripts/verify-release.sh source` exited 0 on clean source commit `ab5ea146`, including 594 Python tests and the accompanying source, patch-provenance, shell and Node checks. Log: `target/headstart/evidence/release-source-ab5ea146-20261028.log` (SHA-256 `40ff47e124ddf2b421ada45a3811615de8c4e6da423ab4fe0b7d0a56b711f2ee`). This source gate does not establish native or browser interaction, licensing, performance, or distribution acceptance.
- **R23–R25 current Web reference host, 2026-10-28:** A fresh Web bundle built from `d5fbdec7` (manifest SHA-256 `1468b7931f61865c4137e1e3c0d255a29d42b846fab53db4fea5d95902f243ff`). In one in-app browser session, its reference host reported PASS for both compiled-preview and composition canonical roundtrips, including edits/export, stale-stamp rejection, disconnect and reconnect; the preview case also dispatched an allowlisted action. This is bounded browser protocol evidence, not independent rendered-appearance, cross-browser, IME, or complete release acceptance.
- **R22/R23 preview transport boundary, 2026-10-28:** A new regression verifies exact 1 MiB UTF-8 message acceptance, rejection beyond the byte limit, the 128-byte identity boundary, and rejection of a sequence exceeding u64. All six focused Rust protocol tests, eight JavaScript bridge tests, and workspace formatting passed. These are source-level boundary checks; external host integration and broader release security acceptance remain open.
- **R20 stale completion race, 2026-10-28:** A deterministic GPUI TestAppContext regression lets an old task finish on the background executor, starts a replacement before the old foreground completion is delivered, and verifies that only the replacement result reaches the owner and the registry is cleaned up. All six owner-task tests and workspace formatting passed. This is a bounded simulated race, not native or exhaustive concurrency acceptance.
- **R20 independent owner tasks, 2026-10-28:** A GPUI TestAppContext regression starts two pending tasks under distinct identities, cancels the first, verifies only its future is dropped and the second remains Loading, then wakes the second and verifies its successful completion and registry cleanup. The focused Headstart test and workspace formatting passed. This is simulated runtime evidence; broader stale-completion races and native acceptance remain open.
- **R20 owner task release regression, 2026-10-28:** A GPUI TestAppContext test starts pending work on an independent component host, releases the last strong owner handle, processes GPUI deferred entity cleanup, and verifies the pending future is dropped. A retained action does not resurrect the owner or start new work. The focused Headstart test passed (1/1) and workspace formatting passed. This is simulated GPUI lifecycle evidence; native acceptance and broader concurrency cases remain open.
- **R20 owner task action regression, 2026-10-28:** A GPUI TestAppContext test now exercises actual owner actions: starting pending work, replacing it with a successful task, canceling another pending task, and delivering an error. Drop probes confirm replaced and canceled work is dropped, and successful/error completion empties the task registry. The focused Headstart test passed (1/1) and workspace formatting passed. This is simulated GPUI runtime evidence; owner release, native acceptance, and broader concurrency cases remain open.
- **R15 Web shortcut regression, 2026-10-28:** The Web input key bindings now come from a shared testable helper. A focused native GPUI keymap test passed for six representative undo, redo, movement, selection, and deletion bindings, including their Input-only context; the Web bundle and workspace formatting checks passed. GitHub reported successful checks for prior commit `3146f961`. This source test complements, but does not replace, physical macOS keyboard, cross-browser, RTL, and IME acceptance.
- **R15 Web macOS editing shortcuts, 2026-10-28:** RustUI Web application initialization now registers input-scoped Command-arrow navigation and selection, Option-arrow word movement, and Command/Option deletion actions alongside the existing Control bindings. The Web build, native Playground check, and workspace formatting check passed. In one in-app browser session Command+Left/Right, Option+Left, Option+Backspace, Command+Shift+Left, and Command+Backspace worked in the search field; Command+Down placed a CodeEditor edit at the end of its multiline buffer. No browser errors appeared during the check. Other registered combinations, physical macOS keyboard, other browsers, RTL and IME remain unverified. Web manifest SHA-256: `efebe92d0eee501d648562d1926bb92415dd227b7db3e3ce89a247c2fa6f1604`.
- **R15 Web Command undo/redo, 2026-10-28:** RustUI application initialization now installs input-scoped Command+Z and Command+Shift+Z on Web, where the underlying input defaults only exposed those bindings for native macOS. A Web bundle build, native Playground check, and workspace formatting check passed. In one in-app browser session Command+Z and Command+Shift+Z undid and redid an editor edit; Ctrl+Z still worked, and Command+Z also undid a Playground search-field edit. No browser errors were observed during the check. Other browsers and physical macOS keyboard acceptance remain open. Web manifest SHA-256: `b901daf458abf4fb5dedbc25098f495ef7ac2d84e0d8acacb767e3be4ec6974d`.
- **R15 Web CodeEditor input, 2026-10-28:** In one in-app browser session on Web bundle manifest `000a48fdbb791d0ac9fa932dd4b13f58e5602a30c8e5cadbe90d00f2085d9cf2`, typing a JSON opening brace inserted a matching pair, Backspace removed the pair, and a separate text insertion was undone with Ctrl+Z and restored with Ctrl+Y. No browser errors appeared during this check. This does not establish platform-specific Command shortcuts, real-OS IME, other browsers, or broad editing acceptance.
- **R15 Web syntax-context clock, 2026-10-28:** The version 3 Components runtime patch uses the existing Web-compatible clock for syntax-aware editing as well as highlighting. Exact patch replay/tamper checks, the focused native highlighting test (1/1), and a Web bundle build passed. In one browser comparison the earlier bundle did not insert a JSON opening brace and logged borrow errors; the fixed bundle inserted a matching `{}` pair and showed no browser errors during the check. This does not establish full editor, IME, multi-browser, or performance acceptance. Web manifest SHA-256: `000a48fdbb791d0ac9fa932dd4b13f58e5602a30c8e5cadbe90d00f2085d9cf2`.
- **R15 Web highlighter registry lifetime, 2026-10-28:** The Web language registry now uses reference-counted thread-local ownership rather than permanently leaking its allocation. The version 2 runtime patch replay/tamper test and Web bundle build passed. In one browser session, Rust and JSON retained syntax coloring after switching language and omitting/remounting the editor, with no console errors observed. This does not establish multi-threaded Web behavior or large-file performance. Web bundle manifest SHA-256: `adbbf0bb33d1dbe1c0a9f3ee440b9fd95eda04760e3bfdebad3084efe86e3ad5`.
- **R13/R35/R36 Graph story locale, 2026-10-28:** GraphView and NodeEditor Playground controls, selection and activation status, omission text, and keyboard instructions use English, French, and Arabic catalogs. Locale validation, workspace formatting, and the Playground library tests (212/212) passed; the Web bundle built. One browser session displayed French and Arabic controls and instructions; adding a node in French remained visible after switching to Arabic. Graph example node/port data and error messages are not translated by this change; native visual, wider RTL and keyboard acceptance remain open. Web bundle manifest SHA-256: `436b76275cba05d2413cdc68526091d8616fa95496ea7a694d9d6c5afe815dd4`.
- **R13/R15 CodeEditor locale state, 2026-10-28:** The Playground CodeEditor language, line-number, read-only, disabled, mounted, and bound-source labels now use the English, French, and Arabic catalogs. Locale validation, the workspace format check, and the Playground library tests (212/212) passed; the Web bundle built. In one browser session French and Arabic state labels rendered, and the Arabic line-number toggle changed its label from نعم to لا while the Rust source remained visible. Native visual, broader locale/RTL, and IME acceptance remain open. Web bundle manifest SHA-256: `0c3c37f724f97e2ee8f4d8fb863345f9d732cd3095a3c0b94cc988504b6dc9e5`.
- **R15 Web CodeEditor implementation, 2026-10-28:** RustUI enables the existing Rust and JSON grammar features on Web using Tree-sitter 0.27.1 and versioned Components patches; pinned GPUI/Components revisions are unchanged. The Web Playground release bundle built, and one local browser run showed colored Rust and JSON, preserved Rust coloring after an edit, and opened the input story without the initially observed unsupported-clock panic. The focused native Rust/JSON parser and styles test passed; patch replay/tamper checks passed; the source gate passed 594 Python tests and accompanying checks. GitHub run `40670047093` passed its source job on commit `4fa0030c`; its macOS/Web job was skipped because that workflow reserves those builds for trusted manual dispatch. A follow-up Rust 1.97.1 formatting pass fixed the two pre-existing workspace formatting differences; the workspace formatting check passed. Web diff preparation currently runs on the browser executor, with a thread-local language registry; large-file responsiveness, threaded Web behavior, native visual and broader editor acceptance remain open. Bundle manifest SHA-256: `4601d06cf4c5d7e48c2b95e21754ea7ebdb9a04684b33cdc65c916956d198a10`.
- **R15 Web CodeEditor probe, 2026-10-28:** An isolated `wasm32-unknown-unknown` Headstart build of the locked, vendored Tree-sitter 0.26.13 and Rust/JSON grammars failed before linking. Apple Clang has no WebAssembly target, and `tree-sitter-language` 0.1.9 explicitly rejects the Tree-sitter 0.26 Wasm libc integration as incompatible; adding its bundled headers alone cannot fix that incompatibility. The probe used `cc` 1.8.0 from the project lockfile and did not change project dependencies. A follow-up isolated probe used the locally installed Homebrew LLVM 22 compiler and a temporary Tree-sitter 0.27.1 package with the existing 0.1.9 language crate and Rust/JSON grammars. It compiled and linked for Wasm; running its module in Node parsed both samples and produced highlight-query captures. This establishes a candidate parser path only: Components integration, actual editor styling, browser testing, and release validation remain open.
- **R15 native CodeEditor highlighting, 2026-10-28:** Native RustUI enables Components' Rust and JSON grammar feature without changing its pinned version; the subsequent Web implementation is recorded above. The existing CodeEditor test parses both languages and verifies nonempty highlight-query styles; the Playground library passed 212/212 tests, the source gate passed 593 Python tests and accompanying checks, and a macOS Playground binary and Web bundle built successfully. Five new locked and vendored packages provide the parser and grammars; the two grammar license texts were captured from their exact recorded upstream revisions and their published manifests matched those revisions. Native notice collection captured notices for all five new packages and retained the same five pre-existing gaps; the Web report retained three. Native visual highlighting and broader editor acceptance remain open. Web manifest SHA-256: `b87a6a94ea2fb5e4f6e28f888de868591a501acb722c7067cc243979a5313baa`.

- **R13 TokenField visible add label, 2026-10-28:** RustUI now resolves a short visible label from the English, French, and Arabic runtime catalogs independently of the descriptive accessible name. Locale validation and the focused Headstart test passed. The source-bound Web Playground (manifest SHA-256 `4f5dfec40a32fe30effa2d0152c57abfb869672ad4386294f5c05fe6e9858da4`) showed `Ajouter` and `إضافة`; adding `Design` in Arabic and switching to English retained the token and displayed `Add`. This is bounded browser evidence; native and wider locale acceptance remain open.

- **R13 Cosmic layout validation, 2026-10-28:** The RustUI regression now passes 2/2 tests, covering Arabic-Indic digits and one-, two-, and four-digit mixed-direction prefixes. In an isolated copy using the same patched `shape.rs` and pinned sources, Cosmic passed 4/4 library tests and 30/30 enabled integration tests, including 14 ellipsis, 7 shaping/rendering, 3 wrapping, 4 decoration, one rich-text, and one font-weight test. The temporary manifest excluded the unavailable optional `vi` dependencies `cosmic_undo_2`, `modit`, and `syntect`, and the benchmark-only `criterion` dependency; the `vi` editor integration target therefore ran zero tests. This is not full-feature upstream acceptance. Logs: `/tmp/rustui-cosmic-lib-isolated.log`, `/tmp/rustui-cosmic-integration-isolated.log`, and `/tmp/rustui-cosmic-integration-remaining.log`.

- **R13 Arabic date glyph loss fixed, 2026-10-28:** Cosmic 0.19.0 treated an absent start position as an explicit zero position in no-wrap mixed-direction layout, omitting the first opposite-direction span and its width. The reproducible `cosmic-text-initial-bidi-span-v1.patch` fixes both cases without changing the version or removing ICU direction marks. A bundled-font regression passed for Latin and Arabic-Indic digits; exact patch replay/tamper passed. In the source-bound Web Playground (manifest SHA-256 `06516879c3b19b48e9b0776cb6f9cafeeb398e2b3f172af6b18e5a07c87e7201`), the previously missing day appeared after French-to-Arabic locale switching. The macOS binary built using a separate temporary incremental directory after the normal cache produced a `module_children` incremental ICE. The unmodified standalone suite did not resolve: optional `cosmic_undo_2` is absent from the vendor set. Isolated default-feature tests and their limitations are recorded above. Native GUI and broader RTL/mixed-direction acceptance remain open.

- **R13 Arabic date rendering diagnosis, 2026-10-28:** In two source-bound Web diagnostic builds, removing only ICU's U+200F direction marks made the selected date appear in full (`28/10/2026`), while preserving them and removing the picker's ellipsis styling still showed the incomplete `2026/10/`. Both diagnostic changes were reverted; the original ICU text and the verified v2 upstream patch are restored, and the canonical Web bundle was rebuilt. The marks affect bidirectional ordering, so dropping them is not an accepted fix. The subsequent Cosmic layout fix and regression are recorded above; native Arabic and wider RTL acceptance remain open.

- **R13 DatePicker calendar localization, 2026-10-28:** The retained RustUI picker now updates CLDR week start on locale changes and supplies ICU short date text, using the versioned `gpui-component-date-picker-localization-v2.patch`. The focused Rust test passed 1/1; patch replay/tamper passed 1/1; the source release gate passed 592 Python tests and its shell/Node checks; native and Web builds succeeded. Web manifest SHA-256 `68ccf6e0cd285046ef70c4aa8715ec3867ec56f6244c23bcbf895a6585bcbce2` captures the tested picker source. In the in-app browser, French started Monday and selected 28 October rendered `28/10/2026`; switching to English retained the value and rendered `10/28/26`. Arabic started Saturday, but the selected date appeared truncated as `2026/10/`; ICU's `ar-EG` formatter returned the full `٢٨\u200f/١٠\u200f/٢٠٢٦`. The subsequent Cosmic layout fix is recorded above; verify native Arabic before accepting this capability. Native GUI, broader keyboard, RTL and browser acceptance remain open. The older disposable native bundle remains because automatic command review rejected its removal.

- **R13 calendar week rules, 2026-10-28:** RustUI i18n exposes CLDR first-weekday information from the already-locked ICU 2.4 calendar data. A focused Headstart test passed for `en-US` Sunday, `fr-FR` Monday, `ar-EG` Saturday, and an explicit `u-fw-wed` override. The DatePicker integration and its remaining Arabic rendering defect are recorded above.

- **R05 popup readability, 2026-10-28:** RustUI composites the translucent menu tint over an opaque theme surface for Components popovers. The focused composition test passed 1/1 in Headstart, and the native build succeeded. In the disposable macOS Playground, the French calendar obscured underlying story text in both light and dark appearances. A source-bound Web bundle (manifest SHA-256 `4fe00d5bfa54502341d62643e6045acf498451b7aa9725ce358fbe0007f4021b`, theme file hash matched) showed an opaque French calendar and recent-search menu in the in-app browser. These are bounded visual observations, not full popup or platform acceptance. French week start and localized selected-date formatting remain open. The native app was quit; automatic command approval rejected removal of its disposable bundle and executable, which remain in `target/headstart/verification/debug/` for cleanup.

- **R13/R15 ComboBox locale and sizing:** public RustUI ComboBox now resolves visible and searchable placeholders from the active catalog with literal fallbacks, and exposes validated trigger/menu widths. The canonical Text inputs story starts empty to show the translated label and sizes both control and popup to 220 logical px. Focused Headstart ComboBox tests passed 16/16 on native wrapper commit `8289da2f`; the earlier Playground library suite passed 212/212, locale validation and `./scripts/cargo-headstart check -p rustui-playground` passed on macOS. Source-bound `3125a321` Web bundle showed French/Arabic names and popup search fields; at 1280×720 the Arabic popup and trigger stayed within the story pane and selecting Skia updated the trigger. Evidence: `target/headstart/evidence/combo-box-localized-web-3125a321-20261027.json`. Source-bound macOS release binary SHA-256 `4aeae64f14d3151ff623db2e9ecb5e4d5802ec6c0e0ffa1ead048035111ffaba` showed the 220 logical px Arabic trigger and popup inside the story pane, the full Arabic search prompt, and an AppKit selection reflected in the accessible value. The preceding build also showed the French search prompt. Native test app and GUI slot were cleaned. Evidence: `target/headstart/evidence/combo-box-native-8289da2f-20261027.json`. Safari, narrow viewport, keyboard/IME, screen reader, and broader locale and input acceptance remain open; other Text inputs controls still show untranslated labels. Coding/acceptance requirement totals remain provisional and unchanged.

- **R09/R13 Alert story:** the canonical Playground Alert story now resolves its heading, message, title, visible state, variant and action labels from the English/French/Arabic catalogs. `validate_locales.py` validated all three runtime catalog groups; `./scripts/cargo-headstart check -p rustui-playground` and 212/212 Playground library tests passed. A source-bound `40a059cb` Web bundle rendered all three locales in one in-app Chromium profile at 1280×720; French hide and Arabic show/warning actions changed the expected state. The matching native macOS 27 bundle showed the same locale/action behavior and named controls in the native accessibility tree. Bounded evidence: `target/headstart/evidence/alert-story-web-40a059cb-20261027.json` and `alert-story-native-40a059cb-20261027.json`. Safari, OS-version, wider viewport and VoiceOver acceptance remain open; VoiceOver remains user-deferred. Source CI passed on `40a059cb` (runs 40452390974 and 40452383687); compiled CI was skipped and must run on a release candidate.

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
- **Release state:** the provisional inventory has 37 requirements. The current
  recorded lower bounds are **4/37 coding complete** (R00–R02 and R34) and
  **3/37 acceptance complete** (R00–R02). R34's coding closure was omitted from
  the previous coding count; these bounds are not an exhaustive reclassification
  of all groups. Most other rows have substantial source implementation
  but retain an unverified contract or release gate. This milestone strengthens
  R28 and R30 without closing either. Five native and
  three Web dependency-notice gaps, supported OS/browser versions, wider
  functional/locale/RTL and workload acceptance, and signed distribution remain
  open. Accessibility and VoiceOver, Japanese and other languages are explicitly
  undone/deferred under the current release scope.

## Current checkpoint — 2026-10-27

### Card story English, French, Arabic copy

- **Narrow Web repair:** At 480×720 in the `09ffc399` bundle, the French
  `Mettre à jour le parent` control clipped at the right edge. Commit `e1fbcd24`
  splits the existing four actions into two rows, retaining their identities
  and callbacks. The `7adfba7c` Web bundle showed the full French label inside
  the viewport (right edge 337.5 of 480 CSS pixels), visible Arabic controls,
  and a working parent-count increment after the layout change. Source check,
  optimized Web build, and whitespace check passed. Before/after screenshots
  and source-bound evidence: `target/headstart/evidence/card-story-narrow-web-7adfba7c-20261027.json`.
  Other viewport sizes and Safari remain open.
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
| R26 | 9: live theme/data reload | Present, distinct from Rust source hot reload. The watcher ownership/epoch regression and seven watcher tests passed; these do not establish dispatched native/Web event-path acceptance. Invalid-update recovery and broader reload acceptance remain open. |
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

## Refresh 2026-10-28 — R13–R15 primary input label localization

Checkbox, Toggle, and Radio now expose `localized_label` through the public
RustUI API. The Text inputs Playground story supplies English, French, and
Arabic catalog entries; a missing entry retains the constructor label. This
slice is **coded 3/3** and has bounded **native and Web acceptance 3/3** for
visible labels and control names in those three locales. The focused Headstart
regression passed 3/3, rustfmt and `git diff --check` passed, and the current
Web release bundle built successfully. Its manifest was independently
recomputed with 773/773 bundle entries matching and source SHA-256
`e47294d7dc6cf0d208b8126b543fa7103bf2f8dfe76dd6fae3f84334b1467823`.

Native validation used the exact production-consolidation Playground bundle,
distinguished by its 16 framework demos from the retired c611 bundle with 10.
French rendered `Activé`, `Accepté`, `Sélectionné`; Arabic rendered `مفعّل`,
`مقبول`, `محدّد`. Native AX names matched, including the disabled Checkbox's
localized name. Clicking the Arabic Checkbox changed its value to on; switching
to French retained on. The disposable bundle and test processes were removed.
The same names appeared in the current Web bundle's semantic tree and rendered
Arabic story; its Checkbox also retained value 1 across the Arabic-to-French
switch. The local Web tab and server were closed.

This accepts only the primary input label slice. It does not close remaining
story copy, generic locale runtime, VoiceOver or screen-reader acceptance,
broader keyboard/IME behavior, or roadmap-wide production gates. The matching
Wasm Headstart toolchain was read-only from worktree 4adc; outputs stayed in
this worktree. A release candidate needs a fresh source-bound bundle after
further source or documentation changes.

## Refresh 2026-10-28 — R13 Checkbox reference locale slice

The Text inputs story's Checkbox reference heading, two visible labels, and distinct checked/unchecked control names now have English, French, and Arabic catalog entries. This focused slice is coded and accepted in native and Web Playground for those three locales; it does not change the roadmap-wide counts above. `validate_locales.py`, the Playground Headstart check, rustfmt, and `git diff --check` passed. The native bundle executable hash was `cdce492ecc152da758eeb574ecf05b357ee67e4e9a3fea8fa375147b14302b92`; its English, French, and Arabic names passed, and French and Arabic labels were visually observed. The exact source-bound native process was stopped and its disposable bundle and standalone executable were removed. The fresh Web bundle built with the matching Wasm Headstart root; manifest SHA-256 is `200d9c3ad158dc90d06e2127b4c263760caa4ddd1e7ed5f4369e33639f6ac5ff`. Browser observations showed the heading, labels, and checked/unchecked names in all three locales, with checked state retained across locale switches and no browser console errors. The validation tab and server were closed. Three Web dependency notice gaps remain open, alongside broader locale and release acceptance.

## Refresh 2026-10-28 — R13 Stepper Quantity locale slice

`StepperView::localized_label` now resolves its visible label through the active RustUI catalog, preserving the constructor label when the message is missing. Its increment and decrement names use the resolved label. The Playground Text inputs story supplies `Quantity`, `Quantité`, and `الكمية` in English, French, and Arabic. Focused Stepper Headstart tests, the Playground Headstart check, locale validation, rustfmt, and `git diff --check` passed. The source-bound native executable and bundle matched SHA-256 `2b59f58caaa0303775b5da49ec2a6c8bcc3ba9ce192703a8ce19524772fc177d`. Native observation showed French and Arabic visible labels and localized increment/decrement names; activating the Arabic increment changed value 1 to 2. The app was quit and its disposable bundle and standalone executable removed. The fresh Web bundle manifest SHA-256 was `c6dbe82590ca2b9a04fcdb8f0cec8b060cd5185d58388d09f31b24a663250687`; browser observation showed English, French, and Arabic action names, the Arabic visible label, and increment from 1 to 2, with no browser console errors. The validation tab and server were closed. This is a bounded coding and native/Web acceptance result for Stepper labels, not completion of the remaining locale or production gates.

## Refresh 2026-10-28 — R13 disabled Stepper locale slice

The Text inputs disabled Stepper now uses the existing `StepperView::localized_label` contract for `Count (disabled)`, `Compteur (désactivé)`, and `العدّاد (معطّل)`. Locale validation, rustfmt, `git diff --check`, and the Playground Headstart check passed. The source-bound native bundle and executable matched SHA-256 `aac1cd7e30960afd84d7b306da278a708bf3395d0ce2293e6f5336ecd2dbd93a`; native observations showed the Arabic visible label and disabled French/Arabic action names. The native app was quit and its disposable bundle and executable removed. The fresh Web manifest SHA-256 was `b5cb2f8f329bd10161b146b98836f3d84c75ee41571eca85f735985a09e45024`; the browser semantic tree showed both actions disabled with the expected English, French, and Arabic names, with no console errors. The Web tab and server were closed. Other Text inputs copy and roadmap acceptance remain open.

## Refresh 2026-10-28 — source CI and Picker locale

Pull request run 40653208356 failed two candidate snapshot tests because they relied on directory iteration order and on the first hashed file; its macOS/Web job was skipped by the workflow's manual-dispatch condition. Commit `5377df73` targets the intended inode during the hard-link race and reports link-count violations before general metadata changes. The focused snapshot suite passed 34 tests and the local source release gate passed 590 Python tests plus its shell checks. GitHub pull request run 40654729170 passed the source job on Linux for that commit; macOS/Web was skipped under the same dispatch condition.

The Text inputs story now localizes the Picker owner value, its Kotlin action, and the ColorPicker label in English, French, and Arabic. Locale validation and the Playground Headstart check passed. The native bundle matched executable SHA-256 `b79fa0f0e655e3b56dc8bc106cf7be32ca17665cc6e9a7c0835e74d96b13a65b`; French and Arabic labels appeared in its accessibility tree and the Picker value changed to Kotlin and survived a locale switch. The Web bundle manifest SHA-256 was `494b7badb4afcf3aa3a0790c6d90be2deb1e2533d94213cfb8101bc5dd38aab2`; the browser showed the three locale names and the same value update across a switch, with no console errors. The native app, disposable executables, browser tab, and server were cleaned up. These bounded checks do not close broader locale or release acceptance; three Web dependency notice gaps remain.

## Refresh 2026-10-28 — R13 DatePicker placeholder locale

`DatePickerView::localized_label` resolves the empty-date placeholder and default trigger name from the active catalog, retaining the constructor label as fallback and preserving explicit accessible-name overrides. The Text inputs story supplies English, French, and Arabic entries. Locale validation and 17 focused Headstart DatePicker tests passed; both native and Web Playground builds succeeded. The source-bound native bundle and executable matched SHA-256 `5ec6f28ff58a2c0951032eed983b70bd49734be1c9f4ae8f90f135d45d5d8ee1`. Native observations showed names in all three locales and an Arabic placeholder; selecting 2026-10-27 and switching to French retained the value. The Web manifest SHA-256 was `28fe91a152e39c7c8396d67cc584b783da00b11eb2ad119a3ef753a292d26743`; browser observations showed all three control names and the Arabic placeholder with no console errors. Disposable apps, executables, tab, and server were removed.

The calendar itself remains incomplete for French and Arabic: its popup showed English month and weekday names in both native and Web while Arabic was selected, and the selected date used a fixed `2026/10/27` format. Native observation also showed underlying story text visible through or over the calendar popup. Investigate these separately; this placeholder slice does not establish calendar localization, popup layering, or overall locale acceptance. Three Web dependency notice gaps remain.

Source investigation: the pinned Components `locales/ui.yml` contains 19 calendar month/weekday entries and none has French or Arabic translations; the existing RustUI adapter already selects `fr` and `ar`. The pinned date picker defaults to `%Y/%m/%d`; RustUI's ICU locale runtime has a date formatter, but the Components picker currently accepts a chrono format string. Prepare a reproducible upstream patch and preserve the pinned versions while addressing both paths; inspect the native popup layering independently.

## Refresh 2026-10-28 — R13 calendar month and weekday translations

The approved reproducible Components patch `gpui-component-calendar-fr-ar-v1.patch` adds French and Arabic values for all 19 calendar month and weekday keys without changing the pinned version or upstream checksum metadata. The exact source/patch verifier passed, and a temporary replay test passed apply, check, reverse, reapply, and rejection of an altered source. The source release gate passed 591 Python tests and its shell checks. Both Headstart Playground builds recompiled `gpui-component 0.7.3`. Native bundle and executable matched SHA-256 `1e2694afa5dd313f39a176be8102c67e2e77801fee02ee1b18f0482578de4d3b`; the Web manifest SHA-256 was `8acc90b8ba84aba58245258445cfd3c45313b578e2b656ec63dce5b446230234`. Native and Web observations showed French `octobre` and Arabic `أكتوبر` with translated weekday abbreviations; the browser reported no console errors. Test apps, executables, tab, and server were removed.

At this historical checkpoint, the calendar started weeks on Sunday in French, selected dates used the fixed `%Y/%m/%d` format, and underlying story text appeared through or over the popup on native and French Web. Later R13 calendar week-rule and locale-format work, and R05 popup-readability work, are recorded in the current inventory above; this paragraph does not describe their current status. Three Web dependency notice gaps and broader release acceptance remain open.

### Stage 9: pre-clone preview and composition text bounds (2026-10-29)

Preview encoding now counts serialized format, identifiers, property strings, and action references before cloning; composition encoding also counts graph, slot, and child identifiers. This is a conservative lower bound; the bounded JSON writer still enforces the complete output limit. Oversized action and child identifiers have focused regression coverage. Codec tests passed 14/14 and composition tests passed 14/14; the Web release driver passed with 18/18 host IME-plan tests and reported three outstanding Web dependency-notice gaps. Evidence: `target/headstart/evidence/preview-id-budget-codec-20261029.log` (SHA-256 `98dd935f9e2002d0fe8182aa4a954e126c3c9d129a87edaf9105df701d82cec3`), `preview-id-budget-composition-20261029.log` (`ba8fa55e3c18971eb4e4c62c5a97b106a03cc90b69355c0d0b18b4e8ce9fb093`), and `preview-id-budget-web-20261029.log` (`5d02f361b7c569788fdf32731fdc14c73764b1adfc942c43ee473557e7b80fd6`). This focused hardening is not a complete security or release acceptance audit.

### R26: app theme registry recovery check (2026-10-29)

A GPUI test exercises the public `ThemeFile::reload_if_changed` file path against the real app theme registry: an invalid update leaves the active theme revision unchanged, restoring the last good source avoids a redundant activation, and a subsequent valid changed source activates again. The focused Headstart test passed 1/1. This does not establish watcher dispatch or native/Web live-reload acceptance.

### R25: browser client serialization errors (2026-10-29)

The reference browser client rejects non-serializable commands through its promise interface rather than throwing synchronously. A focused Node regression covers circular references and BigInt, verifies that neither sends a message nor consumes the session sequence, and confirms a subsequent valid request succeeds. The preview transport test file passed 11/11 (corrected from the previously recorded 9/9). Browser runtime and broader external-consumer acceptance remain separate.

### R25: malformed browser reply handling (2026-10-29)

The reference client now ignores successful replies whose result is an array, retaining the pending request until a valid object reply arrives. The transport suite passed 12/12; log `target/headstart/evidence/preview-client-array-result-20261029.log`, SHA-256 `3786f9746694a71a9af5274ced18fc349db7822e3dc62cb6dcc68e611017d552`. This is a focused Node check, not full browser acceptance.

### R25/R28: Web release build with preview-client fixes (2026-10-29)

On clean source commit `eb20bf545dd68d57750c6643e171b0849a876a5a`, the Web release driver exited successfully and its host IME-plan suite passed 18/18. The bundled `reference-client.js` matches the source SHA-256 `a3f36b6c0790173ab7f167c81fae84d4b9d4fe865a50afd4b367dcad1231644a`. Build log `target/headstart/evidence/preview-replies-web-20261029.log` SHA-256 `0e13b4c404bd61870d3bf7a5ace2997845346cb885670d490564e5567c5023d1`; bundle manifest SHA-256 `c6c91e6e1860df8e9ea237ce4cd855a76613d1619f1c96f8be9795d347eb972a`. Dependency notices still have three gaps (`hexf-parse`, `mac`, `seahash`). This build does not establish interactive browser, IME, licensing, or release acceptance.

### R25: browser roundtrips with preview-client fixes (2026-10-29)

In an in-app Chromium session using the source-bound Web bundle from `eb20bf54` (manifest SHA-256 `c6c91e6e1860df8e9ea237ce4cd855a76613d1619f1c96f8be9795d347eb972a`), the reference host reported PASS for both preview and composition canonical roundtrips, including expected stale-stamp errors, disconnect, and reconnect. Each profile was disconnected; the owned tab and local server were closed. These checks do not exercise artificially malformed array replies in a browser, establish visual rendering, real OS IME, cross-browser behavior, or complete preview acceptance.

### R32/R13: Outline Web interaction and localization gap (2026-10-29)

On the `eb20bf54` Web bundle (manifest SHA-256 `c6c91e6e1860df8e9ea237ce4cd855a76613d1619f1c96f8be9795d347eb972a`), the Outline framework demo expanded Document and Section, selected Heading, retained its selection when collapsed and re-expanded, and cleared the selection and removed expansion ID after removing the Section branch. In the same in-app Chromium session, switching to French translated the surrounding Playground title and description but the Outline demo controls, status and node labels remained English. English/French/Arabic localization of the story remains open; native, keyboard, wider browser and RTL acceptance are separate. The owned browser tab and server were closed.

### R32/R13: localized Outline content (2026-10-29)

TreeNode now supports an optional catalog label with its literal fallback; labels are resolved before retained tree reconciliation, so locale changes update displayed labels without changing node IDs. The Outline demo controls, status, help and node labels have English, French and Arabic messages. Locale catalog validation passed; the Playground Headstart check passed, Tree tests passed 24/24 (`outline-locale-tree-20261029.log`, SHA-256 `51a3f810097de1a39552e2619800904ef118f939a4789564accc25b8996b478d`), and catalog tests passed 5/5 (`outline-locale-catalog-final-20261029.log`, SHA-256 `07a5b04fe686f03b8d3a0a3947321725a9b6208ace15b98b6deba17090617e4c`). An initial Web bundle failed startup because the new status interpolation used invalid syntax; the corrected catalogs passed those tests and the subsequent Web build, including 18/18 host IME-plan tests (`outline-locales-web-final-20261029.log`, SHA-256 `30989bdf6d916b967510eade4ba3c9e10d39da6dd4f4f615b31c39c8eb206173`; manifest `aee1954885b7cb0d743a2e15d44110b9f2e993e30a0b9552785228e810430932`). In one in-app Chromium session, switching from English to French and Arabic translated labels and retained selected Heading and expanded Document/Section. Arabic tree rows remained on the left; broader RTL, native, keyboard and browser acceptance remain open. The outer group accessible label remains literal; accessibility is deferred. The owned tab and server were closed. Three Web dependency-notice gaps remain.

### R13/R17/R32: shared Tree row direction (2026-10-29)

RustUI Tree rows now apply depth indentation on the right, reverse the marker/label order and use a left chevron for collapsed folders in RTL; LTR keeps left indentation and right chevrons. This shared adapter also serves Outline and does not modify vendored code. Tree tests passed 24/24 (`tree-rtl-suite-final2-20261029.log`, SHA-256 `be9f466e24609dc1299ffee7dab5dedd0a1a4132307ed7d03f0fb3c324d83230`). The Web release driver passed with 18/18 host IME-plan tests (`tree-rtl-web-20261029.log`, SHA-256 `fd04a65b1dfefbaa0914b2da8632b6d5672a0ac4eb170c07b8e52c0be2279c3f`; manifest `e2599477cea0edf99222ae7cc71566ce2b97e59d50610b7cd0d1577194d98b48`). In one in-app Chromium session, Arabic Outline rows and nesting appeared on the right and a nested item could be selected; switching to English retained selection and restored left-aligned rows. The owned tab and server were closed. RTL directional keyboard behavior, native, cross-browser and broader acceptance remain open; three Web notice gaps remain.

### R13/R17/R32: RTL Tree directional keys (2026-10-29)

The shared Tree adapter reverses left/right expansion actions in RTL through the existing Tree action path, preserving disabled-folder guards and LTR behavior. Outline tests passed 6/6, including a GPUI test-window regression that sends RTL keystrokes and verifies expansion binding updates (`outline-rtl-keyboard-suite-final-20261029.log`, SHA-256 `ea109c422e474563070d68ce02fabe918df10f3edc83b18f01e0eceab5994398`); Tree tests passed 24/24. A first incorrectly scoped Outline test command matched zero tests and was replaced by the correctly scoped run. The Web release build passed (`tree-rtl-keyboard-web-20261029.log`, SHA-256 `25b3e381282bd60e16ca7dd0d211c534079eed5f01ee220c3f9f2ae4001d9940`; manifest `76969e24c356de662ae7930659f83062dea440bdebc12e0fab411b13950b3641`). In one in-app Chromium session, Right collapsed and Left expanded the selected Arabic section; English Right expanded and Left collapsed. The Web build preceded the test-only regression addition. Native, other browsers, disabled-branch runtime keyboard and broader acceptance remain open; accessibility remains deferred. The owned tab and server were closed.

### R15/R32: disabled Outline branch in RTL (2026-10-29)

A GPUI test-window regression explicitly selects a disabled collapsed branch, sends Left, Right and Enter in Arabic, and verifies that the branch remains collapsed and the owner receives no expansion writes. The focused Headstart test passed 1/1 (`target/headstart/evidence/outline-disabled-rtl-20261029.log`, SHA-256 `8cfd37a7980f2e761e1bb25b480df8f1882843a423deb5926c5702643a7e56ab`). This is simulated event-path evidence, not native or browser keyboard acceptance; broader disabled-control coverage remains open.

### R17/R32: Tree and Outline integration checkpoint (2026-10-29)

On clean commit `be51d99282820dfae37dbc853ce5e55d7a1bba1e`, the locked offline Headstart Tree/Outline library suite passed 32/32, including the new RTL and disabled-branch regressions (`target/headstart/evidence/tree-outline-integration-be51-20261029.log`, SHA-256 `5dcc41554c56e98fa36756c3e217051026df54470a3bc7175976090d65bfe527`). GitHub Actions run `40926085934` passed its source job for that commit; its macOS/Web compiled job was skipped. This does not establish native or browser acceptance, compiled CI, or release readiness.

### R15/R32: disabled Arabic Outline in browser (2026-10-29)

In one in-app Chromium session on the `a847e83a` Tree RTL keyboard Web bundle (manifest SHA-256 `76969e24c356de662ae7930659f83062dea440bdebc12e0fab411b13950b3641`), clicking the disabled Arabic branch left it collapsed, with selection `none`, expansion `document`, and activation count zero. After selecting the enabled section, Down selected its heading and another Down wrapped to Document, skipping the disabled branch. The owned tab and server were closed. This bounded pointer and keyboard check does not prove native, other-browser, or broader disabled-control acceptance; accessibility is deferred.

### R13/R32: retained Outline locale regression (2026-10-29)

A GPUI test-window regression changes an existing Outline from English to French, checks the translated root label and literal fallback for a missing message, and verifies expansion and owner selection remain intact. The focused Headstart test passed 1/1 (`target/headstart/evidence/outline-locale-regression-20261029.log`, SHA-256 `447f4cad6843298a5ce4b54d98132eeacca744868f4e019277b0f451ccd12645`). This simulated test complements the bounded browser observations; native, wider RTL/browser and deferred accessibility acceptance remain open.

### R13/R17: localized Tree reconciliation story (2026-10-29)

The Tree reconciliation demo now resolves node labels, controls, status and guidance in English, French and Arabic while preserving stable IDs and controlled state. Locale validation passed and the Playground catalog suite passed 5/5 (`tree-story-locales-catalog-20261029.log`, SHA-256 `cbe0044bba6a1ac1bcdd5c95287bdeb1564a5716809dcd5c48c3c3a6f126551a`). The Web release build passed with 18/18 host IME-plan tests (`tree-story-locales-web-20261029.log`, SHA-256 `b7a9ce3f41c23a0ad8ab961c7f7bdce52c8184f7edbb9fb39dc4f740ead43ff6`; manifest `89579c4967ed8ce3ab6dc52b7917316c6f4de7d31a1cc5c44e40d1bd1232a797`). In one in-app Chromium session, French labels appeared, a nested leaf was selected, and switching to Arabic preserved the selection and expanded branches with right-aligned Arabic rows. The earlier basic Tree example on the same page still has English labels; native, wider browser and RTL acceptance remain open. The owned tab and server were closed; three Web notice gaps remain.

### R13/R17: localized basic Tree example (2026-10-29)

The basic Tree example now resolves its node labels and selection status in English, French and Arabic, while retaining stable selection IDs. Locale validation and 5/5 Playground catalog tests passed (`basic-tree-locales-catalog-20261029.log`, SHA-256 `09ba4e68b47dec14964566036c3ac94b24f7bcafdac0a00c6244782ba7330609`). The Web build passed with 18/18 host IME-plan tests (`basic-tree-locales-web-20261029.log`, SHA-256 `7a82d0066ad2124556c8237d2311843c07a4cc6ede56a7c00ee6f4d5d78214d7`; manifest `7a239ec5abb2376825cba8c2f4ecafc7554f507f1a92dce465018d0565ab84e0`). In one in-app Chromium session, French Tree labels appeared; selecting TextField then switching to Arabic preserved the selected `text-field` ID and displayed its Arabic label. The owned tab and server were closed. The generic layout-inspector text remains English; native, wider browser, RTL and accessibility acceptance remain open, with accessibility deferred. Three Web notice gaps remain.

### R09/R13: localized Playground story metadata (2026-10-30)

The layout summaries for all 16 story types, generic verification status and illustrative-source notice now resolve in English, French and Arabic. A catalog regression verifies every story summary key in all three runtime catalogs (1/1; `story-metadata-locale-test-20261030.log`, SHA-256 `cdf981f4850be16937b41b77461224c7ef1b0a63e0f472b9a55bc422d539c63a`); the catalog suite passed 5/5. The Web release build passed 18/18 host IME-plan tests (`story-metadata-locales-web-20261030.log`, SHA-256 `1eb1913ef4b45310e463985ffff28a49aab9194739d3ff11a876e50af6670c12`; manifest `12255561f1e5e5c5aa66aa65084b4080b2a7b11fb24a90437fa959e1e13cb666`). In one in-app Chromium session the Tree story showed its layout, verification and source notice in French and Arabic. This does not establish all story translations, native rendering, wider browser/RTL, or production acceptance; three Web notice gaps remain. The owned tab and server were closed.

### R10/R30: current workspace test checkpoint (2026-10-30)

On clean commit `bd0f9ea2838093058e5d48ccc3b536c99cc759f1`, `./scripts/cargo-headstart test --workspace --tests --locked --offline` passed 1,447 tests across 42 reported suites, with zero failures and zero ignored tests. Log: `target/headstart/evidence/workspace-integration-bd0f9ea2-20261030.log`, SHA-256 `cb9df2e1cd53ab5c32506e77c27999c437eddc1608ef5df627741ec230e99291`. This covers the command’s macOS host unit/integration test targets; it does not establish native GUI, browser, Web runtime, compiled CI, licensing, performance, or release acceptance.

### R03/R10: workspace documentation tests (2026-10-30)

On clean commit `f4766388`, locked offline Headstart workspace doctests passed 3/3 across six reported crate suites (`workspace-doc-f4766388-20261030.log`, SHA-256 `15a4584599f0886fe8a10bfeeea61af905a203e59cdbb9e16e3b5128045ec58f`). The small number of doctests does not establish comprehensive documentation or public API example coverage. TreeNode localized-label documentation now explains the literal fallback and stable-ID behavior and includes a public API example; the updated RustUI crate doctests passed 3/3, including that example (`tree-localized-doc-20261030.log`, SHA-256 `2b571e1531e86fd4dce6e048a02d9f8afa8b014c31e0f987f40abf2d81d26eab`). Native/browser acceptance and release gates remain open.

### R13/R33: Drawer story locale handling (2026-10-30)

The canonical Drawer story now resolves its controls, title, status, modal-conflict examples and note-field prompt in English, French and Arabic. Compiled-preview specimens retain caller-supplied trigger and title strings; user-edited note contents are not translated or replaced. The original demo’s initial editable note remains English sample content, and its outer accessible name remains literal (accessibility deferred). Locale validation and the catalog suite passed 5/5 (`drawer-locales-catalog-20261030.log`, SHA-256 `4fe4c9be8b1a8496ae727a74f7b6ce6d2a20ad627dc41c637a884cc72b4daba3`); the bounded preview specimen test passed 1/1 (`drawer-locales-specimen-20261030.log`, SHA-256 `845541ddc6ce88bbdd3f3ec0634dae95c061432ae727156e44475ae4d919047c`). The Web release driver passed 18/18 host IME-plan tests (`drawer-locales-web-20261030.log`, SHA-256 `3f3da3af3390763fd98d8523c28fc870bedc24dccbc8e0b9664b52dde111ebd8`; manifest `d33f09c08934f823965923caf0727c64e515254e0d73036a76e2dc949daf8176`). In one in-app Chromium session French controls and Drawer title appeared; edited notes survived closing, switching to Arabic and reopening, and the Arabic title updated after its owner action. The owned tab and server were closed. Native, wider RTL/browser and modal acceptance remain open; three Web notice gaps remain.

### R23/R25: embedded preview viewport (2026-10-30)

The browser reference host previously mounted a custom Drawer but placed it after the full developer control gallery, outside the 440-pixel iframe; wheel scrolling did not reach it. A trial ScrollView wrapper did not resolve the observation and was discarded. The Web reference profile now renders the owner bridge and mounted factory directly; the normal Playground compiled-preview story retains its developer controls. On parent `ae258388` plus this change, the Playground compiled-preview gallery suite passed 99/99 (`preview-compact-gallery-suite-20261030.log`, SHA-256 `eb0da7e6ff11cadefa6e45dac4d1f0c55ed1871e9f912e43f4f59573d6b2ea61`), and the Web release driver passed 18/18 host IME-plan tests (`preview-reference-compact-web-20261030.log`, SHA-256 `637af3f7d29b2a83320b865e7eb7f42e22dbd599e1f58e5e46e8fcaead131938`; manifest `e04dac68f14c40bdbb116be61d27d6ff7cc490c79b06040fcf25bf8c1969480b`). In one in-app Chromium session a custom Drawer trigger was visibly available in the iframe, opened its caller-provided title and notes, and closed; the reference host then passed the preview roundtrip, stale-stamp rejection, disconnect and reconnect. The earlier Drawer localization kept those custom labels intact. The owned tab and server were closed. Wider component, browser, native and accessibility acceptance remains open; three Web notice gaps remain.
