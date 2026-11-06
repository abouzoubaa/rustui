# macOS 27 theming strategy brief for RustUI

> State update (2026-10-22): the opening description below records the pre-reconciliation HIG copy. `macos27.json` has since been reconciled to the production v1 baseline, including provenance, token sources, theme roles, component metrics/compositions, all 41 IDs and 419 styles. See `MACOS27_BASELINE_RECONCILIATION_2026-10-22.md` for hashes and exact path comparison; this note does not change runtime files.

Status: evidence and planning brief; no source, theme, or schema changes were made.

## Executive summary

RustUI already has a reduced `hig/macos27.json` alongside the larger `hig/full.macos27.json`. The reduced file contains 41 component records and 419 style entries. The local README describes a v1 runtime adapter that applies only selected profile metrics, mapped colors, body font size, radii, material opacity, button styles, toolbar metrics, and scroll-view SVG measurements. It explicitly calls several other component records partial or reference-only; native Liquid Glass composition is unsupported, and increased-contrast appearance is absent from v1. This is a meaningful starting point, but the file must not be mistaken for a complete system-wide macOS 27 theme contract.

The two Chorus conversations contribute complementary evidence: a proposed RustUI architecture/specification with an important correction to the real component model, and 56 reported screen concepts (16 light/dark gallery screens and 40 dark Playground widget screens) plus corrections to screens 02, 16, and P28. These images are design concepts, not Apple specifications and not proof of current implementation. The originals are now stored under `rustui-images` and `ui-images`; `IMAGE_INVENTORY.json` records source IDs, versions, SHA-256 values, file sizes, deduplication and paths. All stored files were hash-verified against the inventory. Visual review was limited to the version-sensitive #02, #16 and P28 candidates (three, two and two images respectively); the full collection was not visually inspected image by image.

The next step should be a traceable audit and strategy, not an immediate schema rewrite. Preserve all 41 current component records, map each to its RustUI renderer/capability and each desired macOS concept, define missing behavior explicitly, and only then propose a backwards-compatible schema evolution and phased implementation plan. Keep native integration and advanced rendering work behind separate explicit design/approval gates.

## Evidence and confidence

### User requirements in the conversations

- Keep RustUI's actual declarative component syntax as the primary public model: `#[component]`, `#[state(...)]`, ordinary methods, `View::body`, constructors and stack macros, method actions, root mounting, and keyed child hosting through `Component(key, initializer)`.
- Correct the earlier spec's builder-first `ViewCx` / `use_state` presentation; v1.1 was reported as correcting it. The earlier assistant's account is a transcript claim, not a fresh code audit here.
- Use the screen concepts as visual references spanning common controls, flexbox, application layouts, accessibility, localization/RTL, and macOS system-widget concepts.
- Produce a complete and robust strategy for macOS 27 theming and an improved `macos27.json`; retain the current component inventory and distinguish supported, partial, reference-only, and unsupported capabilities.
- User intent was to keep source images in `rustui-images` and other UI images in `ui-images` under `hig`, and to send a carefully scoped strategy prompt to the existing HIG advice chat after review.

### Claims and proposals made by prior assistants

- Documents Search reported a RustUI Master Specification v1.0 (113 pages) and later v1.1 (135 pages), with accompanying component/file/requirements inventories and diagrams. v1.1 was said to include 127 component concepts, 303 file destinations and 143 acceptance requirements. These numbers describe proposed documentation inventories, not verified code or approved contracts.
- The v1.1 response reports that three diagrams were added and seven revised. The `RustUI_Master_Specification_v1.1_Complete.zip` archive was downloaded and its 19 image members extracted and hashed. This work did not audit the full specification text or verify its reported contents against source code.
- Generate RustUI Screens reports 16 paired light/dark designs and 40 dark Playground designs. It names final corrections for #02, #16 and P28. The downloaded set contains 19 gallery PNG candidates (versions/corrections), 41 dark Playground PNG candidates (including the P28 correction), and 16 v1.0 artwork candidates. The inventory preserves these variants and records the transcript-designated finals. Only the seven version-sensitive candidates (#02, #16 and P28, including alternatives) received manual visual comparison; every stored file, including the others, received size and SHA-256 verification.
- Screens use visual language such as graphite surfaces, blue accents, glass/material concepts and native-looking controls. Treat those as aesthetic proposals only. They do not establish Apple's requirements or RustUI's support.
- Documents Search explicitly reported that the v1.1 documentation was checked but no Rust compilation, native UI, hot-reload or GPU test was run.

### Local repository evidence inspected

- `hig/README.md` is the local status note and records the current adapter boundaries and known gaps summarized above.
- `hig/macos27.json` parses as 41 component records and 419 styles: 37 records have non-empty styles and four are empty. The exact current component IDs are: `link`, `button`, `text`, `label`, `checkbox`, `divider`, `icon`, `image`, `slider`, `toggle`, `picker`, `stepper`, `scroll_view`, `text_field`, `search_field_examples`, `radio_button`, `combo_box`, `form_accessories`, `alert_dialog`, `save_dialog`, `menu`, `list`, `notification`, `window_chrome`, `toolbar`, `sidebar`, `text_style_guide`, `utility_window`, `button_group`, `color_picker`, `disclosure_control`, `form`, `menu_bar`, `menu_item`, `menu_separator`, `outline_view`, `search_field`, `segmented_control`, `tab_bar`, `table`, `window_controls`.
- Profile axes include default/light/dark appearance; profile members include dimensions, radii, borders, typography, spacing, materials, accents, window controls, accessibility, platform and measure unit. Runtime role mapping includes primary-to-accent; token-source mappings include `spacing.control_inline` and `radii.control`.
- A cached RustUI trail query reports `ThemeDocument::validate` checks format/version/framework/profile match, nonempty components, button semantics and platform snapshot; the loader deserializes then validates. Coverage code/tests expect 41 records and 419 style rules, classify renderer-mapped/reference-only/unresolved coverage, and explicitly identify zero-style records. The Playground theme inspector also reports 41 records and light/dark button resolution. These are trail-derived findings to confirm against the source files and tests during the follow-up audit.
- The trail query says native rendering is not implied by theme data: only actual renderer calls establish runtime support. Preserve that distinction in the strategy and in the resulting schema.

### Conversation coverage limits

The supplied full available text for both named conversations was reviewed. Three early assistant turns in RustUI Documents Search are explicitly omitted; the first user turn in Generate RustUI Screens has image content omitted. No other pages were available before/after the supplied transcript. The 16 gallery and 40 Playground counts come from the visible assistant summaries. The initial user-provided reference image and those omitted message payloads are not recovered by this analysis. Do not claim full image or full-message reconstruction.

## Current macos27.json assessment

The reduced theme is a curated application of the full catalogue, not yet an end-to-end coverage contract. The 41 records must remain present. Four records currently have no style entries; those entries need an explicit status/reason and owner rather than silent deletion. The other 37 records also require a capability audit: a non-empty style block is not evidence that a renderer consumes it.

Build a component-by-component matrix with at least these fields: current record and schema path; screen/design evidence; actual RustUI widget/renderer; consuming source location and runtime call path; supported properties; state/variant coverage; light/dark/profile behavior; platform bridge/native status; accessibility and interaction behavior; test/story coverage; gaps; confidence/source; proposed owner and phase. Keep evidence links stable and identify whether each conclusion comes from user instruction, current source, tests, a prior assistant, an illustrative image, or official Apple documentation.

Do not replace the file wholesale. Additive schema changes should be versioned, migration-aware, validated, and compatible with all current records. Preserve the full catalogue separately and make the reduced theme's relationship to it explicit.

## Required strategy study

### 1. Verify official platform guidance

Before making Apple-specific claims, check current official Apple Developer guidance and record title, URL, last-updated/access date, platform/version applicability, and the requirement derived from it. Start with the [macOS Human Interface Guidelines](https://developer.apple.com/design/human-interface-guidelines/designing-for-macos), [Color](https://developer.apple.com/design/human-interface-guidelines/color), [Materials](https://developer.apple.com/design/human-interface-guidelines/materials), [Typography](https://developer.apple.com/design/human-interface-guidelines/typography), [Accessibility](https://developer.apple.com/design/human-interface-guidelines/accessibility), [Dark Mode](https://developer.apple.com/design/human-interface-guidelines/dark-mode), [Windows](https://developer.apple.com/design/human-interface-guidelines/windows), [Toolbars](https://developer.apple.com/design/human-interface-guidelines/toolbars), [Sidebars](https://developer.apple.com/design/human-interface-guidelines/sidebars), and [SF Symbols](https://developer.apple.com/sf-symbols/). Confirm current naming/availability; do not infer a future macOS 27 rule from an older platform page. Label uncertain, unavailable, or unsupported guidance explicitly.

### 2. Complete visual and semantic model

Define semantic tokens rather than scattering literal values: foreground/background/surface/elevated/selection/semantic status roles; accent and control roles; separator, border, focus and disabled roles; typography families/styles/weights/line heights/tracking; spacing, sizing, hit regions, radii and strokes; icon metrics; material/elevation/opacity/blur; animation and transition roles. Specify aliasing, precedence, fallback, contrast checks, profile resolution and platform-snapshot handling. Include light/dark, system accent, increased contrast, reduced transparency, inactive/active windows, reduced motion, high text sizes and explicit profile overrides. Separate colors, metrics and behaviors that can be theme data from effects requiring runtime/native APIs.

For each component family, define all variants/states and composition rules: button/action; text/labels; text and search input; selection controls; slider/stepper/progress; menus/popups/segmented/picker; lists/outlines/tables/collections; tabs/navigation; toolbar/sidebar/window chrome; dialogs/alerts/sheets/popovers/tooltips; forms; color/font panels; symbols/images; scroll/split/grid/flex layouts; media/charts/maps/doc previews; drag/drop/selection; notifications/system widgets. Explicitly classify RustUI-owned, GPUI/GPUI Components-owned, native-system, partially bridged, reference-only and unsupported cases.

### 3. Interaction and accessibility contract

Specify hover/pressed/selected/mixed/disabled/focused/invalid/loading/busy/dragged/expanded/inactive/empty/error states, event semantics, keyboard focus order, tab/arrow navigation, Return/Escape, menu shortcuts, actions, cancellation, validation and destructive confirmation. Define accessibility roles/labels/value/state, VoiceOver behavior, full keyboard use, contrast, color-independent cues, larger text, reduced motion, reduced transparency, focus visibility and test evidence. Distinguish visual mockup state from implementation behavior.

### 4. Typography, localization and layout

Cover Dynamic Type or RustUI's equivalent scaling model, text styles, fallback fonts, truncation/wrapping, baseline alignment and content-size stress. Cover long strings, pluralization, locale-sensitive dates/numbers, bidi isolation and correct RTL mirroring (while retaining non-mirrored semantic icons where appropriate). Define flex/grid/stack sizing, intrinsic content, constraints, min/max, wrapping, overflow, split resizing, scroll behavior, window resizing, compact/wide cases, safe content insets and pixel/scale rounding. Treat the screen set as test scenarios, not normative metric sources.

### 5. JSON schema and runtime architecture

Inventory actual JSON paths and consumer code before proposing schema. Define schema version, required/optional properties, typed units, enums, token references, cross-field constraints, extension policy, unknown-key policy, default/fallback rules, migration chain, validation diagnostics, runtime profile selection and deterministic resolution. Consider schema namespaces for semantic tokens, component contracts, platform appearances/material capabilities, interaction/accessibility guidance and examples, but justify the final structure from current consumers. Preserve all 41 records and current 419 styles unless a reviewed migration proves otherwise; no file edits in this strategy phase.

### 6. Maintainability, performance and acceptance

Define ownership and provenance for each token/property, deprecation policy, documentation generation, version compatibility, linting, schema validation, coverage reports and contribution rules. Establish performance budgets for theme resolution, component/style lookup, allocations, layout, material/backdrop work, memory and reload. Avoid promising GPU partial repaint or native Liquid Glass behavior unless the render path demonstrates it. Define deterministic snapshots and visual regression coverage for every supported component/state/profile, plus accessibility, keyboard, locale/RTL, scaling, window resize, serialization/migration and runtime-consumption tests. Acceptance must fail if a JSON field is unconsumed or an unsupported feature is represented as implemented.

## Requested deliverables from the target HIG chat

Ask the target to produce, before implementation:

1. A source-grounded current-state audit with citations to repository files/tests and official Apple sources, plus explicit evidence confidence and known gaps.
2. A component-by-component coverage/gap matrix covering all 41 existing component records and all relevant screen families; no silent omissions.
3. A versioned, backwards-compatible target schema proposal for a more complete `macos27.json`, with illustrative JSON snippets, token taxonomy, ownership/consumption rules, validation and migrations. Do not edit the actual theme file yet.
4. A prioritized phased, file-by-file implementation strategy with test plan, acceptance criteria, dependencies, risks, unresolved product choices and separate approval gates for native/advanced rendering.
5. A mapping from each of the 56 reported screen concepts and identified variants to component families/states, while marking unavailable assets and uncertain versions instead of fabricating them.
6. A short decision register separating user requirements, verified current behavior, prior proposals, visual concepts, Apple guidance and open questions.

## Asset retrieval status

All 77 matched Library records were downloaded through their supported GET transfers: 76 original image/vector records and the v1.1 complete ZIP. Every downloaded record matches its Library byte-size metadata and has a recorded SHA-256. The archive contains 19 artwork members; each member was extracted and hashed. Across the 95 artwork candidates (19 paired-gallery records, 41 dark Playground records, 16 v1.0 specification assets and 19 v1.1 archive assets), nine v1.1 items are byte-identical to v1.0 and are stored once with provenance mappings. This leaves **86 unique image files**, split between **46 files in `rustui-images`** and **40 files in `ui-images`**. Every stored asset passed SHA-256 verification. Manual visual inspection covered only seven version-sensitive images: three #02 candidates, two #16 candidates and two P28 candidates; it was not a visual audit of all 86 files.

- `rustui-images/playground-dark`: 41 dark Playground screens.
- `rustui-images/component-and-layout-gallery`: 5 paired gallery candidates for component Playground #02 and flexbox/layout #03–04.
- `ui-images/screen-concepts`: 14 paired gallery records for the remaining screen concepts, including both #16 versions.
- `ui-images/spec-v1.0`: 16 original v1.0 specification artworks.
- `ui-images/spec-v1.1`: 10 distinct v1.1 specification artworks; nine unchanged v1.1 records map to the v1.0 files.

The component Playground #02 candidates were visually compared; the latest Library candidate is the redesigned 3-column gallery and is marked final per the conversation. The #16 candidate with the latest Library timestamp has corrected RTL mirroring and is marked final. The final P28 image corrects the progress/read-only state; its earlier predecessor is retained. These seven candidates were manually inspected; the other stored images were integrity-checked but not all visually reviewed. All alternatives remain in the folders and inventory, except exact byte duplicates, which are mapped to their canonical copy. The initial user-provided reference-image payload in the Screens conversation was omitted from the transcript and is not part of this generated-asset inventory.

`IMAGE_INVENTORY.json` maps each source Library ID and source chat/message to the original filename, category/version, stored path, size, SHA-256 and deduplication target. These are generated concepts, not Apple specifications or implementation evidence. The complete v1.1 ZIP was used for extraction and remains in the source Library; it was not copied into the image folders.

## Suggested phase gates

- Gate A — verify official Apple guidance and current repository consumers; reconcile evidence and unknowns.
- Gate B — agree on complete semantic token/component contracts and support classification.
- Gate C — review schema migration plan, validator coverage, backward compatibility and ownership.
- Gate D — implement a bounded additive slice only after explicit approval; run schema, component, accessibility and visual tests.
- Gate E — consider native composition or advanced materials/rendering separately with platform constraints, performance data and explicit approval.

## Evidence references

- Local project notes: `hig/README.md`.
- Reduced theme: `hig/macos27.json` (41 records; 419 styles).
- Full catalogue: `hig/full.macos27.json`.
- Asset metadata/status: `hig/IMAGE_INVENTORY.json`.
- Source conversation: [RustUI Documents Search](https://chatgpt.com/c/6ad69904-a38c-83e8-a796-0d4f4db78de6).
- Source conversation: [Generate RustUI Screens](https://chatgpt.com/c/6ad6bcc8-8c24-83ea-b45c-3df8e7535503).

