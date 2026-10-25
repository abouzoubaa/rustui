# RustUI macOS 27 theming strategy — evidence study

**Study date:** 2026-10-22 (UTC)  
**Scope:** Analysis and strategy only. No source, theme, schema, test, setting, or existing report file was modified. No build or test was run.  
**Target:** `/Users/Abium/Developer/rustui`

## Executive summary

RustUI already has a versioned appearance loader and runtime theme registry, but the present contract is a constrained adapter, not a complete app-wide semantic theming system. `themes/macos27.json` is format version 1; `ThemeDocument` strictly deserializes its envelope, stores each component record as arbitrary JSON, validates typed records and resolves styles, and rejects unknown envelope fields. Runtime activation applies a selected profile to GPUI Components and consumes selected component styles through RustUI renderers. Presence in JSON is not runtime support.

The local working copy `hig/macos27.json` has **41 records / 419 style rules**: 37 nonempty records and four empty records (`toolbar`, `sidebar`, `menu_bar`, `tab_bar`). RustUI's runtime capability registry lists **27 mapped records** and explicitly classifies **14 as reference-only**. Among mapped records, actual consumption is selective; all should be treated as *partial theme contracts* until property-level coverage and behavior tests prove otherwise. The three empty mapped records (`toolbar`, `sidebar`, `tab_bar`) remain unresolved for component-rule coverage; toolbar/sidebar/tab behavior is partly supplied through the profile/GPUI theme, not their empty record. The empty `menu_bar` is reference-only and has no renderer. The repository exposes a coverage report that distinguishes applied, uncovered, reference-only and unresolved zero-style data, but I did not run the Playground or tests to obtain a fresh runtime report.

Recommended direction: retain v1 and all 41 records/419 rules as source data; introduce a typed v2 envelope with stable semantic tokens, explicit component contract/status, property ownership and coverage policy, deterministic appearance/accessibility resolution, migration diagnostics, and legacy preservation. Roll out in stages, starting with schema/coverage and one bounded component family. Do not imply native Liquid Glass, AppKit panel/window ownership, full accessibility behavior, or GPU partial repaint from JSON values alone.

## Evidence labels and limitations

- **User requirement:** The approved delegated prompt requests a complete strategy, actual RustUI model, complete audit and all 41 records, no implementation, no redownload/recreation, and new uniquely named Markdown/CSV deliverables only. It also states the originals/hash verification and prior three-family visual-review scope.
- **Verified source:** The repository facts below come from the current files named inline. The working copy under `hig/` is a curated file; the running Playground embeds `themes/macos27.json`, so the coverage tool does not automatically validate the working copy.
- **Earlier proposal/report:** `hig/HIG_THEME_STRATEGY.md` and `hig/README.md` preserve prior findings and conversation reports. They are treated as leads; claims were rechecked where noted.
- **Visual concept:** 86 local unique originals were read in ten contact-sheet batches. These communicate intended scenarios and aesthetic exploration, not Apple requirements, implementation support, or pixel-accurate platform metrics. Only #02, #16 and P28 alternatives were individually called out in the preexisting notes as version-sensitive detailed comparisons; the rest received broad thumbnail-level inspection, not asset-by-asset semantic annotation.
- **Official Apple guidance:** Current Apple Developer/HIG pages were checked on 2026-10-22. The live pages are JavaScript-rendered; the browser index supplied snippets and titles but did not expose full page Markdown. Conclusions below are limited to those supported excerpts and stable public documentation titles/URLs. Apple's pages are living HIG pages and do not constitute a verified macOS 27-specific pixel spec. The study cannot certify a macOS 27-only rule where Apple does not state one.
- **Unknown / not verified:** No test/build was run; no runtime coverage report was refreshed; no native AppKit integration, accessibility tree, VoiceOver, actual macOS 27 display, GPU/material effect, performance, or live theme reload was exercised. The initial user-provided image payload omitted from the cited conversation transcript remains unavailable and outside the 86-file inventory. Three early assistant turns in the Documents Search conversation are also omitted as documented in the brief. Those gaps are not reconstructed.

## Current code and consumer audit

### File path and data flow

1. `crates/rustui/src/theme/loader.rs`: `parse_theme` deserializes JSON to `ThemeDocument`, then calls `validate`; `load_theme` reads disk; `serialize_theme` validates and normalizes output. Tests include `validated_theme_serialization_round_trips_the_typed_document`, `serialization_preserves_arbitrary_json_theme_records`, and `parser_accepts_typed_component_records_with_intentionally_empty_styles`.
2. `crates/rustui/src/theme/schema.rs`: `ThemeDocument` is `deny_unknown_fields`; `format_version` is currently `u8` and must equal `THEME_FORMAT_VERSION == 1`. The envelope is typed, but `source`, `token_sources`, `derivation`, and each component value use `serde_json::Value`; `theme_roles` is `BTreeMap<String,String>`. `validate` checks format, framework name/version, profile/snapshot ID, nonempty components, button contract, typed component parts/styles, profile values, and resolves button in both light and dark. `select_profile(platform,version)` exists, but activation uses `platform_snapshot` directly (see below). Thus presence of `select_profile` does not establish automatic profile discovery or selection.
3. `crates/rustui/src/theme/resolver.rs`: button and generic component selectors resolve by appearance/material/interaction/text presence/size/orientation/chrome/modality; specificity is the count of non-`any` dimensions. Equal-specificity overlapping rules are rejected. Generic style properties are parsed/resolved; this is not equivalent to renderer use. `ThemeCoverageReport` checks declared renderer capabilities against each rule and reports uncovered properties, reference records and zero-style records.
4. `crates/rustui/src/theme/registry.rs`: declares the 27 `MAPPED_COMPONENTS`, 14 `REFERENCE_ONLY_COMPONENTS`, and property-by-part `RENDERER_CAPABILITIES`. `activate_theme` validates, resolves button/table-header/menu-background styles, computes coverage, calls `apply_profile` on the `platform_snapshot`, then stores an `ActiveTheme` containing mode/profile/button/report/document. `table_header_colors` deliberately resolves `vector_path` but comments that source artwork is still unresolved coverage; menu resolution similarly handles `filter`, `blend_mode`, `vector_path` outside declared renderer support. This is a concrete example of “the resolver can parse a field” differing from “the capability report says the renderer applies it.”
5. `crates/rustui/src/adapters/themes.rs`: `apply_profile` changes GPUI Components' global `Theme` mode and maps selected profile colors, font family/body size, md/lg radii and material opacity to GPUI roles. It hardcodes other translations, and current adapter fields are not a general semantic token engine. It validates positive finite sizes and opacity in `[0,1]`. `apply_profile` does not construct an AppKit `NSVisualEffectView`/native Liquid Glass surface.
6. `crates/rustui/src/components/theme.rs` plus each component renderer: components look up active theme and style capabilities. Explicit resolver calls and tests exist in button, link, text, label, checkbox, radio, slider, text field, menu, etc. Actual renderer read paths, not mere files or JSON records, establish runtime consumption.
7. `apps/rustui-playground/src/lib.rs`, `stories/theme_inspector.rs`, `stories/component_catalog.rs`: the Playground embeds/loads `themes/macos27.json`, displays appearance and coverage information, and catalog metadata says its first 41 entries mirror that file. The coverage report is not currently pointed at the isolated `hig/macos27.json` working copy as a verification target (`hig/README.md`).

### Profile field boundary

The profile snapshot includes `id`, version range, `appearance.default/light/dark`, 20 named appearance colors, dimensions, radii, borders, typography/font styles and sizes, spacing, materials, accent variants, window-control colors, accessibility hints, platform and `measure_unit`. `apply_profile` explicitly reads: appearance colors via `ButtonPalette`, body font size, md/lg radii and `materials.material_opacity`; it sets GPUI theme palette roles from mapped values. Component renderers separately consume selected dimensions/metrics/style roles. Examples are toolbar height in `components/navigation/toolbar.rs`, scroll measurements in the scroll renderer, button metric references/role mapping in the button resolver, and two token-source paths used by button (`spacing.control_inline`, `radii.control`).

Do not label every profile field “applied.” Specifically, sidebar/toolbar blur radii and opacities, many dimensions, most typography styles, window control colors, accessibility contrast/focus/reduced-motion fields, and accent alternatives are not proven to affect their corresponding native platform behavior just because they are serialized. Some GPUI roles may receive semantically similar colors, but the code does not provide a general rule mapping all profile fields. Increased contrast, reduced transparency, reduced motion, inactive-window adaptation, text-scale adaptation and system accent discovery are not runtime axes in this v1 contract.

### Actual RustUI component model

The required authoring model is source-compatible with `#[component]`, `#[state(...)]`, ordinary Rust methods, `View::body`, constructor/stack macros, and actions bound to an owning component. `crates/rustui/src/view.rs` defines `View::body(&self) -> impl View` and `Component(key, initializer)` as the retained child with its own owner/lifecycle; `crates/rustui/tests/component_state.rs` and `examples/view_composition.rs` demonstrate the macro/state shape. The style strategy should not replace this with builder-first `ViewCx`/`use_state` concepts, which the earlier report itself says were wrong for the current API.

### 41-record component audit

“Partial” below means at least one path is registered/consumed, not that the entire visual, interaction, platform, or accessibility contract is implemented. A `reference-only` row can still describe a component family that has a differently named RustUI implementation; it means this particular catalogue record has no declared runtime mapping. The record and style counts are read directly from the working copy. Recommendations are proposals.

| ID | Current record / styles | Runtime category and verified boundary | Proposed semantic owner / next step |
|---:|---|---|---|
| 1 | `link` / 1 | Partial — mapped; foreground/typography capability. | RustUI Link; semantic link + hover/focus/visited/disabled. |
| 2 | `button` / 2 | Partial — strongest v1 contract; typed metrics/roles, selector rules, GPUI palette and button resolver. | RustUI action contract plus GPUI rendering; add variants, busy/destructive, focus/keyboard/a11y contract. |
| 3 | `text` / 4 | Partial — two opaque appearance parts mapped; vibrant parts not in the declared capability list. | RustUI Text; role/style tokens and scale/wrap policy. |
| 4 | `label` / 4 | Partial — opaque light/dark text styles mapped; vibrant paths unclassified. | RustUI Label; semantic label association separate from text appearance. |
| 5 | `checkbox` / 2 | Partial — background/foreground/typography style mapping; indeterminate/disabled semantics require behavior review. | RustUI Checkbox; checked/mixed/disabled/focus/invalid and accessible value. |
| 6 | `divider` / 4 | Partial — two parts and background are mapped; exact direction/contrast behavior incomplete. | RustUI decorative separator; orientation and semantic/hidden behavior. |
| 7 | `icon` / 9 | Partial — generic `icon` part width/height; other named icon parts are not declared renderer capabilities. | RustUI Icon / platform symbol bridge; symbol source, rendering mode, RTL mirroring, accessibility. |
| 8 | `image` / 4 | Partial — `image` width/height/radius; plural `images` artwork not mapped. | RustUI Image; crop/fit/decorative vs informative labels, loading/error. |
| 9 | `slider` / 2 | Partial — size dimensions mapped; track/thumb semantics and keyboard/value feedback are not established by theme data. | RustUI Slider; min/max/step/value/focus/increments and non-color value. |
| 10 | `toggle` / 2 | Partial — body bounds/radius/background mapped. | RustUI Toggle; on/off/mixed/disabled, label/state and keyboard. |
| 11 | `picker` / 2 | Partial — body geometry/colors/typography mapped; option menu behavior is separate. | RustUI Picker; popup/menu semantics, selected state and arrow/keyboard rules. |
| 12 | `stepper` / 10 | Partial — stepper buttons/separator/glyph paths mapped. | RustUI Stepper; bounds, increments, min/max, repeat and accessibility value. |
| 13 | `scroll_view` / 2 | Partial — horizontal/vertical scrollbar background/border/radius capability; SVG metrics are used. | RustUI scroll view; overlay visibility, track/thumb behavior, input, reduced motion. |
| 14 | `text_field` / 12 | Partial — body styles in small/large variants; `search`/control-specific artwork is not a complete input contract. | RustUI TextField; caret/selection/IME/validation/placeholder/secure semantics. |
| 15 | `search_field_examples` / 2 | Reference-only — example composition, not a renderer. | Preserve as reference; map tests to SearchField family, never mark record supported. |
| 16 | `radio_button` / 2 | Partial — geometry/color/typography mapped. | RustUI Radio; grouping, selected/focus/keyboard, label, disabled. |
| 17 | `combo_box` / 2 | Partial — body geometry/colors/typography mapped. | RustUI ComboBox; editable vs choice contract, popup and typeahead semantics. |
| 18 | `form_accessories` / 6 | Reference-only — artwork composition, not a RustUI control. | Preserve evidence; map desired accessory parts to IconButton/menu/disclosure contracts. |
| 19 | `alert_dialog` / 4 | Reference-only — composition record; registry notes RustUI `AlertView` is inline. | RustUI alert vs AppKit dialog owner decision; cancel/destructive/action order. |
| 20 | `save_dialog` / 4 | Reference-only — system save-panel composition; no native panel consumer. | Native AppKit integration gate; don't skin to imply system panel. |
| 21 | `menu` / 2 | Partial — background applied to GPUI popover; artwork filter/blend/vector are special-cased but reported as unresolved coverage. | GPUI menu owner; menu roles, keyboard/shortcut, checked/mixed/submenu/disabled states. |
| 22 | `list` / 2 | Reference-only — record describes column header while RustUI `ListView` has no header part (registry reason). | RustUI ListView separate contract; preserve record as non-applicable unless re-scoped. |
| 23 | `notification` / 2 | Partial — mapped body style properties; delivery/persistence/announcement semantics are outside theme. | RustUI notification or OS notification integration decision; severity and actions. |
| 24 | `window_chrome` / 8 | Reference-only — composition artwork; window chrome owned by host/window system. | AppKit/GPUI native boundary; document unsupported until bridge exists. |
| 25 | `toolbar` / 0 | Partial via profile only — mapped name but zero style rules; toolbar height is read by toolbar renderer. | RustUI toolbar; tokenize metrics, overflow/customization and accessibility behavior. |
| 26 | `sidebar` / 0 | Partial via GPUI profile roles only — mapped name but zero component rules. | GPUI/RustUI navigation; active/inactive sidebar + source list selection contract. |
| 27 | `text_style_guide` / 265 | Reference-only — typography source catalogue, not runtime component; profile exposes selected font sizes/styles separately. | Reference data; derive typed typography scale and generated docs, avoid runtime record pretending to render. |
| 28 | `utility_window` / 2 | Reference-only — window composition, no renderer mapping. | Window-system/native owner gate. |
| 29 | `button_group` / 6 | Reference-only — composed control, not runtime component in registry. | Consider grouped action contract after standalone buttons/segmented component. |
| 30 | `color_picker` / 6 | Partial — color wheel/swatches/crayons geometry/background capabilities. | RustUI ColorPicker + native color panel distinction; gamut, value, keyboard and contrast. |
| 31 | `disclosure_control` / 4 | Partial — disclosure part foreground/typography capability. | RustUI Disclosure; expanded/collapsed, arrow direction/RTL, keyboard and accessible state. |
| 32 | `form` / 4 | Reference-only — source composition, no runtime form renderer record. | RustUI composition contract; labels/errors/help/focus-order and responsive layout. |
| 33 | `menu_bar` / 0 | Reference-only — no styles or RustUI menu-bar renderer. | OS-owned menu bar; native command/action integration gate. |
| 34 | `menu_item` / 4 | Partial — foreground/typography capability. | GPUI menu item; shortcut, checked/mixed, submenu, disabled, hover/focus and separator. |
| 35 | `menu_separator` / 4 | Reference-only — record mismatch: popup separators owned by GPUI; source record names list/menu-bar separators. | Keep reference-only; choose a separate semantic separator for menu if required. |
| 36 | `outline_view` / 4 | Partial — width/height/background part mapped. | RustUI Tree/Outline; disclosure, selection, hierarchy, keyboard, row accessibility. |
| 37 | `search_field` / 6 | Partial — only `list_search` declared; toolbar/save-context parts unmapped. | RustUI SearchField; query/clear/submit/suggestions/empty results and typeahead. |
| 38 | `segmented_control` / 8 | Reference-only — no RustUI segmented-control renderer. | New public component/API decision; preserve multiselect vs single-select distinction. |
| 39 | `tab_bar` / 0 | Partial via GPUI theme roles only — mapped name but no component style rules. | RustUI TabView/GPUI; selection, overflow, close action, keyboard and badge roles. |
| 40 | `table` / 4 | Partial — table rows and headers have declared properties; header resolver special-cases vector artwork and coverage flags it unresolved. | RustUI Table/DataGrid; sortable headers, selection, row/column navigation, density. |
| 41 | `window_controls` / 8 | Reference-only — titlebar composition; no system window-control renderer. | AppKit/window manager owns controls; native integration gate, never imitate as supported. |

The 14 reference-only IDs are `list`, `search_field_examples`, `form_accessories`, `alert_dialog`, `save_dialog`, `window_chrome`, `text_style_guide`, `utility_window`, `button_group`, `form`, `menu_bar`, `menu_separator`, `segmented_control`, `window_controls`. The other 27 are in the source mapped list. Four empty records reconcile as above. “Mapped” describes registry membership, not full support. Exact covered/uncovered rule counts should be captured from a read-only report against the chosen theme at implementation Gate A; this study intentionally does not execute the test suite or claim a fresh count beyond 41/419.

## Official Apple guidance checked

Apple HIG is a living platform guide. Access date for every URL below: **2026-10-22**. Apple's public pages were searchable and their titles/short claims verified, but their JavaScript Markdown links could not be fetched by the research tool; cited claims below are limited to the returned page descriptions/snippets and should be rechecked during implementation design review. These pages are not a macOS 27-versioned pixel specification.

| Official source | What it supports for this strategy | Limit |
|---|---|---|
| [Designing for macOS](https://developer.apple.com/design/human-interface-guidelines/designing-for-macos) | macOS interaction context includes keyboard shortcuts and keyboard-only work styles; customization of colors/fonts appears in the page description. | No numeric token values or exhaustive control-state table verified. |
| [Color](https://developer.apple.com/design/human-interface-guidelines/color) | System-defined colors adapt to backgrounds, appearance, vibrancy and accessibility settings. Prefer semantic/system color roles and explicit accessible variants. | Does not establish this theme's literal hex values as Apple values. |
| [Materials](https://developer.apple.com/design/human-interface-guidelines/materials) | Materials allow background color to pass through and help hierarchy/place recognition. | Alpha/blur values in a JSON file do not create platform material composition. |
| [Typography](https://developer.apple.com/design/human-interface-guidelines/typography) | Typography supports legibility, hierarchy and brand/style expression. | Does not validate the catalogue's point sizes or guarantee scaling in RustUI. |
| [Accessibility](https://developer.apple.com/design/human-interface-guidelines/accessibility) | Apple discusses contrast standards including WCAG and APCA; accessibility needs review beyond a color palette. | A profile contrast threshold alone does not prove end-to-end accessibility. |
| [Dark Mode](https://developer.apple.com/design/human-interface-guidelines/dark-mode) | Dark Mode is a system appearance with a dark palette intended for low-light viewing. | Current RustUI mode toggle is not evidence of following system appearance automatically. |
| [Windows](https://developer.apple.com/design/human-interface-guidelines/windows) | macOS windows have frame/body and can often be moved/resized by edges. | Does not establish current RustUI window chrome/native ownership. |
| [Toolbars](https://developer.apple.com/design/human-interface-guidelines/toolbars) | Toolbars group controls along an edge of a view. | Customization, overflow and native integration require local implementation evidence. |
| [Sidebars](https://developer.apple.com/design/human-interface-guidelines/sidebars) | Sidebar icons use accent by default; macOS system accent changes affect apps. | RustUI does not currently prove system accent discovery from serialized alternatives. |
| [SF Symbols](https://developer.apple.com/design/human-interface-guidelines/sf-symbols) | Multiple symbol rendering modes exist (monochrome, hierarchical, palette, multicolor). | Availability/version/licensing and actual rendering remain a runtime/platform concern. |
| [Adopting Liquid Glass](https://developer.apple.com/documentation/technologyoverviews/adopting-liquid-glass) | Apple's overview mentions preferred appearance and accessibility settings such as Reduce Transparency. | Current RustUI adapter is not a native Liquid Glass implementation. |
| [AppKit `NSWorkspace.accessibilityDisplayShouldReduceTransparency`](https://developer.apple.com/documentation/appkit/nsworkspace/accessibilitydisplayshouldreducetransparency) | There is a native API for the macOS Reduce Transparency setting. | The RustUI theme adapter shown does not call it. |

## Proposed target contract: additive schema v2

Do not modify the current file in this study. Use a new format version and typed structures; migrate only after implementation approval. Preserve all 41 current IDs and their 419 style definitions in an archived/imported `legacy_v1` namespace until every value has a reviewed disposition. Do not silently drop unrecognized values. v2 should make unsupported/reference-only status explicit instead of treating opaque data as implied support.

### Resolution model

Resolve deterministically in this order: explicit per-component state override → component variant/state rule → semantic token reference → appearance (`light`/`dark`) → accessibility variant (`increasedContrast`, `reduceTransparency`) → system accent choice → profile/platform range → declared base fallback. `system` appearance means ask runtime; never choose default dark solely because a profile's sample says dark. Inactive window is a runtime signal, not a static theme mode. Reduced motion and text scale are environment/runtime inputs. A deterministic resolved snapshot records every input and fallback chosen.

Recommended token namespaces: `color` (canvas/window, content/surface/elevated, text primary/secondary/tertiary, accent/contrast foreground, selection, focus, separator/border, disabled, success/warning/error/info, overlay); `type` (role, family, weight, line-height, tracking, scaling behavior); `space`; `size` (control bounds/hit region distinct from visible glyph); `radius`; `stroke`; `icon`; `material` (named semantic capability plus fallback, not just opacity); `elevation`; `motion` (duration/curve plus reduce-motion behavior); `component` (variants, states, parts, semantic token refs and property support disposition). Every token includes stable id, typed value/unit, provenance, owner, supported platform range, and optional accessibility override.

### Illustrative envelope (not a file edit)

```json
{
  "format": "rustui-appearance",
  "format_version": 2,
  "framework": {"name": "RustUI", "version": "0.1.0"},
  "profile": {"id": "macos27", "platform": "macos", "min_version": 27},
  "compatibility": {"accepts": [1, 2], "unknown_keys": "preserve_extensions"},
  "environment": {"appearance": "system", "accent": "system",
    "contrast": "system", "transparency": "system", "motion": "system",
    "text_scale": "system"},
  "tokens": {"color": {"surface.canvas": {"light": "#F5F5F5", "dark": "#1E1E1E"},
    "text.primary": {"light": "#1A1A1A", "dark": "#FFFFFF"}},
    "space": {"control.inline": {"value": 12, "unit": "pt"}}},
  "components": {"button": {"contract_version": 1, "status": "partial",
    "owner": "rustui", "parts": {"body": {"states": ["rest", "hover", "pressed",
      "focused", "disabled", "busy"], "styles": [{"when": {"appearance": "light",
      "variant": "primary"}, "background": {"ref": "color.accent.primary"},
      "foreground": {"ref": "color.text.onAccent"}}]}}}},
  "legacy_v1": {"preserved": true}
}
```

Conventions: metric values require units (`pt` logical points or a named dimension scale); opacity is bounded `[0,1]`; no scalar blur/radius can assert a material exists. Enum values are versioned. Token references must resolve, must be type-compatible, and cannot cycle. Selectors use a typed set of axes, overlap is either rejected or given explicit priority; avoid implicit “last matching rule wins.” Unknown root keys are preserved only under an explicit extension namespace and surfaced in diagnostics; unknown enum/property names never silently affect runtime. Runtime capability manifests list per renderer: part, state axes, consumed properties, platform owner and fallback. “Supported” is computed from capabilities plus contract requirements, not hand-entered marketing status.

### Legacy mapping and dispositions

- `format`, `format_version`, `framework`, `profile` → `format`, `format_version`, `framework`, structured `profile`; `profile_snapshot.id/platform/min/max_version` remains version range.
- `platform_snapshot.appearance.default/light/dark` → `tokens.color` appearance branches plus environment default; `default: dark` retained only as legacy sample, not inferred system behavior.
- `appearance` colors (`window_background`, `sidebar_background`, `content_background`, `elevated_panel_background`, `toolbar_background`, row state fills, separator/border, text roles, accent/status/disabled/focus/badge) → semantic `color.*` IDs. Aliases must resolve to semantic tokens, not duplicate literals.
- `dimensions`, `radii`, `borders`, `spacing`, `typography` → typed `size.*`, `radius.*`, `stroke.*`, `space.*`, and `type.*` with explicit logical units and scaling policy. Keep touch/hit area separate from drawn size.
- `materials` → semantic `material.*` descriptor with `nativeCapability`, fallback surface, reduce-transparency fallback and runtime owner. Existing blur/opacity fields migrate as legacy hints only.
- `accents` → semantic accent palette, with `selected: system`/explicit selection plus system accent runtime input; don't imply all named accent values are queried from the OS.
- `window_controls` → host-owned native capability metadata; RustUI imitation is unsupported unless a custom window owns and draws the entire chrome.
- `accessibility` → contrast policy, focus appearance policy, motion/transparency/text-scale environment behavior plus machine-checkable constraints. Existing numeric targets can seed audit thresholds but must not be called Apple mandates without a cited standard/version.
- `token_sources.spacing.control_inline`, `token_sources.radii.control`, `theme_roles.primary` → typed alias/reference tables with cycle/target validation.
- `components.*` all 41 keys remain; translate parts/styles/metadata into contracts and keep record IDs. Mark 14 as `reference_only` with reason; 27 as renderer-mapped but per-property `partial` until consumer coverage is exhaustive. Explicitly mark four empty rules as `deferred` or `reference_only` with owner/reason.
- `source`/`derivation` → structured provenance including source URI/id/checksum, authored/generated status, derivation revision, and license/redistribution note. Keep v1 values in `legacy_v1` exactly during migration; preserve image source IDs/hashes/version relationships.

### Rules, diagnostics and compatibility

Maintain a JSON Schema (structural type/range constraints) and a Rust semantic validator (cross-references, selector overlap, contrast pairs, capability coverage, profile ranges, extension policy). Validation should report JSON pointer, code, severity, message, suggested fix and whether runtime ignores the item. Fail CI for missing token references, ambiguous selectors, unsupported properties claimed `supported`, missing light/dark required roles, unsafe contrast against defined thresholds, or empty renderer-owned contracts. Warnings for deprecated aliases/legacy values include removal version. v1 loader remains available for an agreed deprecation window; migration `v1 -> v2` is deterministic, writes a new candidate rather than modifying the input, and produces a mapping report showing preserved/unmapped values. v2 readers should reject unsupported major versions and preserve extension data under a namespace. Generate reference docs and capability tables from the same typed definitions to avoid drift.

## Component behavior and design-system contracts

Theme JSON may select visual values. Runtime owns state transitions, focus, hit-testing, pointer/key dispatch, accessibility tree, localization, resize policy, native control APIs and effect rendering. Treat these as component contracts and test obligations, not decorative style keys.

- **Actions and links:** primary/secondary/quiet/destructive variants; rest/hover/pressed/focused/disabled/busy; click/Return/Space as appropriate, Escape cancellation for transient flows; distinguish link navigation from button action; visible focus and accessible name/value.
- **Text and input:** text role and scale, wrap/ellipsis policy, selection/caret/IME/composition, placeholder contrast, secure field semantics, invalid/warning/help/error text associations; validation must not be color-only.
- **Choice/value controls:** checkbox mixed state, radio group arrow navigation, switch on/off, picker/menu choice, slider/stepper keyboard increments and min/max, text/numeric/date editor locale formats; report accessible value, range and disabled state.
- **Navigation/menus:** list/tree/table/tab/toolbar/sidebar ownership, selected/expanded/focused/drag target/empty/loading/error row states, typeahead and shortcuts, checked/mixed/submenu state, focus return and dismissal. Define whether arrow keys move focus or selection per pattern and document it.
- **Dialogs/overlays:** modal owner, initial focus, focus containment/return, Escape vs cancel, default and destructive actions, outside-click policy, announcements, reduced-motion transition. Native save/open panels remain OS-owned.
- **Feedback/media/data:** progress determinate/indeterminate/read-only, busy announcement cadence, status icon+text, skeleton, notification severity, chart/table non-color encoding, image alt/decorative distinction, media keyboard and captions, drag/drop copy/move feedback and cancellation.
- **Accessibility runtime axes:** implement system contrast, reduce transparency, reduce motion, text size and accent query through host APIs or explicit runtime settings. Theme variants alone cannot observe OS changes. Use focus shape/width/offset and non-color selection cues; test VoiceOver names, roles, values and state.
- **Localization/RTL:** use locale-aware formatting, long-string expansion and pluralization; separate content direction from layout direction; mirror directional navigation glyphs but not semantic symbols such as play/check; bidi-isolate embedded code/numbers/identifiers. Test Arabic and mixed Arabic/Latin across dialog, menus, tables and fields. Images demonstrate scenarios only; they do not prove string correctness.
- **Layout/typography:** intrinsic size, min/max, grow/shrink, wrap, overflow, baseline, scroll, split resize and window minimum size belong to layout/runtime. Define units and scale rounding once. Use semantic body/label/caption/title roles with font fallback, text scaling, line-height, truncation/accessibility full value. Stress narrow/wide windows, large text, localization expansion, empty data and multiple displays.

Performance work must measure cold parse/validation, cached resolution, allocations, layout/render invalidation, theme activation, full-window refresh and material/backdrop cost independently. Do not promise GPU partial repaint or reusable blur unless a renderer path and measurements prove it. Prefer prevalidated immutable resolved token tables, stable IDs, no per-frame JSON parse, and one invalidation transaction per theme activation.

## Visual evidence map

The asset inventory contains 77 original Library records plus 19 archive members; nine archive candidates are exact duplicates of originals, resulting in **86 unique files** (46 `rustui-images`, 40 `ui-images`). The inventory maps filenames, source IDs, message IDs, checksums, sizes, hashes, target paths, deduplication/version relationships. Files were already present; none was downloaded, regenerated or changed during this study. Contact-sheet inspection was used to keep image context manageable.

- Paired gallery concepts: #01 Studio; #02 component Playground (latest redesigned three-column candidate marked final in inventory; earlier candidates retained); #03–04 flexbox/layout; #05 analytics dashboard; #06 data explorer; #07 settings/forms; #08 Kanban; #09 calendar; #10 asset browser; #11 Notes editor; #12 workflow studio; #13 split editor; #14 command center; #15 setup; #16 multilingual/accessible settings (latest mirrored RTL candidate marked final; older retained). These support review scenarios for hierarchy, light/dark, layout, controls and localization, not exact specs.
- Dark Playground candidate set spans P01 buttons/actions; P02 checkbox/radio/switch; P03 segmented/pickers; P04 text fields/validation; P05 Search, Tokens and Combo Boxes; P06 slider/stepper/levels; P07 date/time; P08 Menus, Pop-ups and Pull-downs; P09 toolbar customization; P10 sidebar/column browser; P11 tabs/path; P12 list/outline; P13 tables/tree; P14 asset grid; P15 flexbox; P16 wrapping/adaptive layout; P17 grids/stacks; P18 split/scrolling; P19 rule editor; P20 popovers/tooltips; P21 alerts/dialogs/sheets; P22 window anatomy; P23 open/save panels; P24 print/export; P25 color well/panel; P26 typography; P27 rich text/rulers; P28 progress/feedback (corrected read-only progress candidate marked final); P29 symbols/images; P30 media; P31 charts; P32 maps; P33 document/Quick Look; P34 drag/drop/selection; P35 system previews/sharing; P36 keyboard/focus; P37 accessibility appearance; P38 localization/RTL; P39 materials; P40 Desktop Widgets and System Controls. Historical alternates remain; not every title maps one-to-one to an official HIG control.
- The 16 v1.0 and 19 v1.1 archive artwork candidates include architecture, authoring, state, identity, reload, transaction, damage, performance, roadmap, history and Playground wireframes. These illustrate an earlier documentation/authoring proposal, not current RustUI runtime implementation. Nine duplicates map to original v1.0 files. The initial user reference image was unavailable.

## Phased implementation roadmap (approval required before any phase starts)

| Phase / files | Owner and scope | Tests / measurable acceptance | Risks and gate |
|---|---|---|---|
| A. Reproducible audit: `crates/rustui/src/theme/registry.rs`, `resolver.rs`, component renderer files, `apps/rustui-playground/src/stories/theme_inspector.rs`, `hig/README.md` (new report only) | Theme/runtime owner: produce machine-readable coverage from both the actual embedded theme and selected candidate; record every field/property consumer and status. | Read-only generated report lists 41 IDs/419 rules; no component disappears; every rule is applied, reference-only, intentionally deferred, or explicitly uncovered with property and source path; exact counts reconcile. | Current coverage treats matching selectors/capabilities; it may not verify every state or use. Gate A review before schema choice. |
| B. Schema contract: `crates/rustui/src/theme/schema.rs`, `loader.rs`, `resolver.rs`, new `schemas/rustui-appearance-v2.schema.json`, `hig/macos27.json` only after approval | Theme API owner: additive typed v2 format, v1 compatibility, migration/report, provenance, diagnostics. | Schema + semantic validator test all valid/invalid cases; legacy 41/419 round-trip; migration reports every field; no silent unknown-property or token loss; selector ambiguity and references fail with JSON pointers. | Public format/API/dependency compatibility. Gate B approves v2 fields, unknown-key policy and deprecation duration. |
| C. Semantic tokens and palette: `crates/rustui/src/theme/tokens.rs`, `adapters/themes.rs`, `registry.rs`; palette fixtures | Theme + GPUI adapter owners: token resolution and explicit capabilities, system appearance/accent/accessibility environment adapters where available. | Unit tests for every appearance/contrast/transparency/accent fallback; pairwise contrast audit and deterministic snapshot; activation resolves once and no per-frame JSON parsing. | OS API availability and high-contrast contract need review; no effect claims without real native path. |
| D. Bounded control slice: `components/theme.rs`, `components/basic/button.rs`, `components/input/text_field.rs`, `components/input/checkbox.rs`, `components/input/slider.rs`, `themes/macos27.json` | Component owners: choose one initial slice (recommend Button + TextField after B), add all parts/states with actual consumers. | Keyboard-only operation; focus visible; VoiceOver role/name/value; light/dark/contrast; long text/large text; screenshots for supported variants; component coverage report marks all required properties consumed. | Avoid one mega-PR. Gate D separately authorizes actual implementation; scope API and behavior review. |
| E. Navigation/data/overlays: `components/navigation/*`, `components/data/*`, `components/overlay/*`, stories and fixtures | Family owners: migrate list/tree/table/menu/tab/sidebar/dialog one family at a time, separating GPUI and native ownership. | Arrow/typeahead/selection/focus-return tests; empty/loading/error/disabled/expanded states; locale and RTL screenshots; table sort/row/column accessibility semantics. | Existing catalogue conflates source artwork with component boundaries; must retain IDs and label unsupported records. |
| F. Layout/localization/accessibility: `layout/*`, `accessibility/*`, `interaction/{focus,keyboard,drag_drop}.rs`, `apps/rustui-playground/src/stories/*` | Layout/a11y owners: dynamic text, constraints, locale/direction, system settings bridge and window adaptation. | Screen matrix covers narrow/medium/wide, min window, Arabic/mixed bidi, long strings, large text, Reduce Transparency/Motion, keyboard and accessibility tree snapshots. | Native behavior/API gaps; use explicit feature gates rather than visual fallback claims. |
| G. Visual regression/performance: dedicated deterministic fixtures/baselines under `apps/rustui-playground` and test resources; `crates/rustui` perf harnesses | Graphics/QA owners: capture stable host renderer outputs and performance budgets. | Baseline per supported component × state × appearance × contrast mode × scale; device/OS/build metadata; stable capture tolerance; cold/warm parse, resolve, layout and render timings; no unjustified GPU claims. | Must baseline only implemented contracts; reviewed diff workflow and intentional baseline update policy. |
| H. Native materials/chrome: separate macOS adapter/module and capability manifest (new paths proposed only after design) | macOS platform owner: native materials, titlebar/window controls, open/save/color panels, menu bar as native APIs. | macOS 27 runtime integration test matrix for system accent, appearance, inactive windows, reduced transparency, VoiceOver, window resize; benchmark backdrop cost; capability fallback evidence. | Distinct native dependency/entitlement/API design gate. Requires explicit approval; v1 JSON alpha/blur values do not satisfy this phase. |

### Shared acceptance criteria

- All 41 existing IDs and 419 v1 style entries are retained through migration; every one has a disposition and provenance trail.
- Coverage claims are generated from executable consumer declarations and renderer probes, not manually labeled in JSON; no “supported” field can outpace actual consumers.
- Every required semantic token has light and dark resolution and declared system/accessibility fallback; contrast calculations identify text/background pair, algorithm/version, threshold source, and exception rationale.
- Keyboard, focus, accessibility tree, locale/bidi, large-text, resize and reduced-motion/transparency checks are required for applicable families, not only snapshots.
- Visual baselines record platform build, renderer, scale, font set, window size, theme revision and environment axes. Artifacts are RustUI captures; generated concepts and Apple HIG pictures do not become normative baselines.
- Theme activation is atomic and diagnostics explain ignored/deprecated/unsupported values; measured budgets are set from baseline before optimization.

## Decision register and recommended first slice

**Already authorized/user-stated:** analysis-only; no repository/theme/schema/test edits or test/build run; preserve actual RustUI model and all component records; inspect official Apple guidance; use verified local image originals, do not redownload/recreate; save new uniquely named Markdown/CSV only under `hig`; report limitations and wait for separate implementation approval.

**Verified current behavior:** format v1 strict envelope; arbitrary JSON component payloads; 27 registry-mapped / 14 reference-only; 41 records and 419 styles in working copy; selected profile/component properties feed GPUI and RustUI adapters; native Liquid Glass composition is not established by the adapter; four empty component records and initial report caveats are documented above.

**Recommended, not yet approved:** additive typed v2, preserve v1 records under a legacy namespace during migration, use runtime capability manifests and property coverage, system appearance/accent/accessibility inputs with named fallbacks, implement a small Button/TextField slice only after schema/contract review, reserve native/window/material work for a separate gate.

**Open decisions before implementation:** Is the intended contract a RustUI theme API or only an app-local Playground theme? Should a semantic profile default to `system` or retain a named explicit default? What compatibility/deprecation window is required for v1 and public APIs? Which RustUI/GPUI components own each record (especially list vs table, alerts vs native dialogs, menu bar, tab/segmented control)? Which OS accessibility settings are in v1 runtime scope? Are native AppKit bridges and dependencies approved? What performance targets and baseline host/OS are acceptable? Is every artwork's full-screen visual inventory annotation required before component design approval?

**Recommended first approved implementation slice:** after a Gate A coverage report and Gate B approval of schema policy, migrate one vertical Button + TextField slice from v1 to typed semantic color/type/space tokens, wire only properties demonstrably consumed, and add appearance × focus/disabled/invalid × keyboard/a11y tests plus deterministic Playground captures. Do not begin it in this study; separate approval is required.

## Source locations inspected

- Theme: `crates/rustui/src/theme/{schema,loader,resolver,registry,tokens,watcher}.rs` and `crates/rustui/src/adapters/themes.rs`.
- Renderers: `crates/rustui/src/components/{theme.rs,basic/{button,link,text,label,divider,icon,image}.rs,input/{checkbox,radio,slider,toggle,picker,stepper,text_field,combo_box,color_picker}.rs,navigation/{toolbar,sidebar,tab_view,disclosure}.rs,data/{list,table,outline}.rs,overlay/{menu,alert}.rs}`.
- Model and API examples: `crates/rustui/src/view.rs`, `crates/rustui/tests/component_state.rs`, `crates/rustui/examples/view_composition.rs`.
- Playground: `apps/rustui-playground/src/{lib.rs,catalog.rs,stories/theme_inspector.rs}`.
- Local evidence: `hig/{README.md,HIG_THEME_STRATEGY.md,IMAGE_INVENTORY.json,macos27.json,full.macos27.json,PROMPT_FOR_ADVISE_CHAT.md}`.
- Useful existing test/function evidence (not run in this study): `ThemeDocument::validate`; loader tests named above; resolver tests `coverage_reports_unapplied_artwork_and_reference_only_records`, `parser_rejects_ambiguous_equal_specificity_component_selectors`, `parser_rejects_non_button_style_rules_that_reference_undeclared_parts`; adapters tests `theme_metrics_require_positive_finite_f32_values`, `theme_material_opacity_requires_unit_interval_f32_values`; registry test `scroll_view_track_border_and_radius_are_counted_as_applied`.


## macOS 27 API claim register addendum

See [MACOS27_APPLE_API_CLAIM_REGISTER_2026-10-22.md](MACOS27_APPLE_API_CLAIM_REGISTER_2026-10-22.md) for current official-source checks of AppKit refresh/toolbar/menu APIs, WidgetKit controls, Liquid Glass and Show Borders claims, and corrected or unresolved API names. Key boundaries: the WidgetKit/`ControlWidget` system-control preview is native extension territory; `ViewAnnotation` is documented under App Intents Testing, not as a general SwiftUI annotation API; the alleged Launchpad replacement and a new toolbar-minimization feature remain unverified. The exact owner-supplied 16-category inventory was not present in the accessible HIG files or handoff text, so it is not reconstructed here.
