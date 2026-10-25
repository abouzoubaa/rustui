# RustUI macOS 27 HIG working set

This folder keeps the production-equivalent RustUI appearance v1 baseline, the broader design catalogue, visual evidence and non-runtime v2 design candidates together. JSON presence and illustrative images do not establish renderer support or Apple platform requirements.

## Files

- [`macos27.json`](macos27.json) editable HIG copy of the v1 production appearance theme. It currently matches the production baseline byte-for-byte and retains all 41 component IDs, 419 style rules, provenance, metrics, compositions, token sources and theme roles.
- [`macos27.production-baseline.2026-10-22.json`](macos27.production-baseline.2026-10-22.json) read-only snapshot of `themes/macos27.json` at reconciliation time.
- [`macos27.hig-pre-design-baseline.2026-10-22.json`](macos27.hig-pre-design-baseline.2026-10-22.json) read-only snapshot of the earlier HIG copy before production-only metadata was restored.
- [`macos27.design-v2-candidate.json`](macos27.design-v2-candidate.json) design-only inventory and additive contract proposal. It is not accepted by the current loader and does not change runtime support. Its first batch maps P05 Search, Tokens and Combo Boxes; P08 Menus, Pop-ups and Pull-downs; and P40 Desktop Widgets and System Controls to their provenance-tracked visual concepts.
- [`full.macos27.json`](full.macos27.json) broader source catalogue with design provenance and reference data.
- [`MACOS27_BASELINE_RECONCILIATION_2026-10-22.md`](MACOS27_BASELINE_RECONCILIATION_2026-10-22.md) records the baseline hashes, value reconciliation and preserved counts.
- [`MACOS27_COMPONENT_GAP_CATALOG_2026-10-22.md`](MACOS27_COMPONENT_GAP_CATALOG_2026-10-22.md) separates 31 source-present unthemed components, 5 absent reusable patterns, 11 native integration candidates and 9 composition examples.
- [`MACOS27_VISUAL_REFERENCE_MAP_2026-10-22.csv`](MACOS27_VISUAL_REFERENCE_MAP_2026-10-22.csv) maps each of the 86 unique local image files to source IDs, checksums and design-only evidence labels.
- [`IMAGE_INVENTORY.json`](IMAGE_INVENTORY.json) preserves Library and archive provenance, hashes, deduplication and verification status.
- [`MACOS27_THEME_STUDY_2026-10-22.md`](MACOS27_THEME_STUDY_2026-10-22.md) is the current source-grounded design study. It distinguishes implementation evidence, open questions, visual concepts and platform guidance.
- [`MACOS27_DESIGN_VALIDATION_2026-10-22.md`](MACOS27_DESIGN_VALIDATION_2026-10-22.md) records static validation results for the design-only artifacts; it is not a build or runtime test report.
- [`MACOS27_APPLE_API_CLAIM_REGISTER_2026-10-22.md`](MACOS27_APPLE_API_CLAIM_REGISTER_2026-10-22.md) separates macOS 27 API facts, corrected names, and unverified platform claims from RustUI runtime support.
- [`OWNER_COMPONENT_REFERENCE_2026-10-22.md`](OWNER_COMPONENT_REFERENCE_2026-10-22.md) is the source inventory; [`MACOS27_OWNER_COMPONENT_COVERAGE_2026-10-22.csv`](MACOS27_OWNER_COMPONENT_COVERAGE_2026-10-22.csv) contains one exact source row per named bullet across its 16 numbered sections.
- [`MACOS27_OWNER_CLAIM_COVERAGE_2026-10-22.csv`](MACOS27_OWNER_CLAIM_COVERAGE_2026-10-22.csv) separately tracks all eight appearance requirements, 11 version-sensitive claim families, and nine supplied source links; these are not added to the 148 component rows.
- [`MACOS27_OWNER_RECONCILIATION_2026-10-22.md`](MACOS27_OWNER_RECONCILIATION_2026-10-22.md) explains the row mapping, design contract families, Apple claim dispositions, and non-runtime boundaries. [`validate_owner_component_coverage.py`](validate_owner_component_coverage.py) runs HIG-local static checks.
- [`MACOS27_VISUAL_FIDELITY_ACCEPTANCE_2026-10-22.md`](MACOS27_VISUAL_FIDELITY_ACCEPTANCE_2026-10-22.md) is the acceptance gate for subsequent design work: native macOS 27 observations and official guidance are authoritative; generated images are supplementary; missing runtime/reference access remains an explicit verification gap.

## Runtime field boundary

| Data | Runtime status | Boundary |
| --- | --- | --- |
| `platform_snapshot` | Validated; selected values applied | Supplies the appearance palette and profile metrics. GPUI Components receives mapped colors, body font size, radii, and material opacity; RustUI components read selected profile metrics. A profile field being present does not mean every renderer consumes it. |
| `components.button` | Applied | Typed button resolver reads its metrics, role mapping, light/dark rules, and style properties. |
| `components.toolbar.metrics` | Applied | Toolbar renderer reads its height metric. |
| `components.scroll_view.svg_measurements` | Applied | Scrollbar renderer reads thumb measurements and opacity. |
| Other component records | Partially applied or reference-only | Component renderers read the fields declared by their style capabilities. The remaining records preserve design coverage; use the coverage report to distinguish them from applied styles. |
| `token_sources.spacing.control_inline`, `token_sources.radii.control` | Applied | Resolve the button's horizontal padding and corner radius. |
| `theme_roles.primary` | Applied | Button resolver checks that the primary role maps to the platform accent. |
| `source`, `derivation` | Present production provenance metadata | Restored from the production baseline (`macos27_design_reference`; RustUI ownership; SVG source-data policy; consolidation method and preserved counts). This records provenance but does not establish renderer support. |
| HIG-only interaction guidance | Not represented by theme data | Keyboard commands, window behavior, menu-bar behavior, customization, and accessibility workflows require component/runtime behavior and review. |
| Native Liquid Glass composition | Unsupported by the current GPUI theme adapter | The profile's material opacity can tint supported surfaces; it does not create native Liquid Glass or blur composition. |

## HIG mapping

Reviewed against Apple's living Human Interface Guidelines on 2026-10-21.
These are design principles and platform patterns, not a pixel-level macOS 27
specification.

| Apple guidance | RustUI theme connection | Work needed beyond the JSON |
| --- | --- | --- |
| [Designing for macOS](https://developer.apple.com/design/human-interface-guidelines/designing-for-macos): spacious work, multiple windows/apps/displays, keyboard and pointer input, resizable windows, customization | Profile dimensions, toolbar/sidebar/tab records, component styles | Review resize behavior, focus and keyboard navigation, menu commands, toolbar customization, and inactive-window appearance in Playground. |
| [Color](https://developer.apple.com/design/human-interface-guidelines/color): consistent meaning, light/dark variants, increased contrast, status clarity | Light/dark palette and semantic color roles in the profile | Add an explicit increased-contrast design and verify contrast. The current v1 profile has no separate increased-contrast appearance. |
| [Materials](https://developer.apple.com/design/human-interface-guidelines/materials): depth, hierarchy, and separation of controls from content | Material opacity and surface colors | Do not describe opacity as Liquid Glass support; validate actual platform material behavior separately. |
| [Typography](https://developer.apple.com/design/human-interface-guidelines/typography): readable type and adaptable text | Profile typography and component typography rules | Check truncation, larger text, localization expansion, and window resizing in real components. |
| [Accessibility](https://developer.apple.com/design/human-interface-guidelines/accessibility): clear, perceivable, operable experiences | Theme contrast, focus, and disabled-state roles | Validate keyboard operation, focus visibility, VoiceOver semantics, and increased contrast; these are not guaranteed by colors alone. |
| [Design principles](https://developer.apple.com/design/human-interface-guidelines/design-principles): familiarity, agency, flexibility, simplicity, craft | Guides consistency of component states and hierarchy | Use these principles during interaction and visual reviews; they are not token fields. |

## Visual references

Use the full catalogue's SVG provenance and Apple's HIG examples as review
references, then make RustUI-owned annotated captures for the Playground review
set. Keep those captures outside the runtime JSON and record their source and
appearance/state. This folder does not redistribute Apple's HIG images.

Useful review states include light and dark appearance, increased contrast,
focused/hovered/pressed/disabled controls, window resizing, and keyboard-only
navigation. Increased contrast is a review target, not a mode represented in
the current theme schema.
