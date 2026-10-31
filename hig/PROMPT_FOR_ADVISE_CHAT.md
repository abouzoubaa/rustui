# Ready-to-send prompt for “Advise on Apple HIG for RustUI”

Send this as a single message to the existing Codex conversation titled **Advise on Apple HIG for RustUI** (`01a15ffb-60a9-7a42-95e8-f2c7a4a8a1de`) after the parent reviews the asset inventory. This file is prepared only; it has not been sent.

---

You are continuing the existing RustUI Human Interface Guidelines and macOS theming study. First study the local material in `/Users/Abium/Developer/rustui/hig`, especially `HIG_THEME_STRATEGY.md`, `IMAGE_INVENTORY.json`, `README.md`, `macos27.json`, and `full.macos27.json`. The images have already been stored locally in `/Users/Abium/Developer/rustui/hig/rustui-images` and `/Users/Abium/Developer/rustui/hig/ui-images`; open and inspect those files directly as visual evidence. This is a local-file handoff; no image attachments were sent in this message. `IMAGE_INVENTORY.json` maps 86 stored unique images (46 under `rustui-images`, 40 under `ui-images`) to 77 source Library records plus 19 v1.1 archive members; nine byte-identical v1.1 items map to their v1.0 canonical file. All stored assets have SHA-256 and size verification recorded. Manual visual comparison covered only the seven version-sensitive #02, #16 and P28 candidates, not every image. Preserve the source IDs, checksums and version relationships. Do not invent or regenerate missing images. The initial user-provided reference image payload omitted from the source transcript is still unavailable and outside this generated-asset inventory.

Your task is to study the evidence and prepare a rigorous, traceable strategy for a complete macOS 27 theming system for RustUI and a substantially improved, maintainable `macos27.json`. This request is for analysis and planning, not implementation. Do not edit source, schema, theme JSON, tests, or generated code; do not delete or replace any of the current 41 component records. Return a strategy that is concrete enough to review and approve before any implementation begins.

## Evidence discipline

Separate and label: (1) explicit user requirements; (2) current behavior verified in source/tests; (3) earlier assistant proposals or claims; (4) illustrative generated screen concepts; (5) current official Apple guidance; and (6) unresolved assumptions. Cite repository paths and test names/locations for implementation claims. Validate Apple claims against current official Apple Developer/HIG material; record URL, access date, platform/version applicability and what the source actually supports. Do not treat generated screens as authoritative Apple specifications, and do not infer implementation support merely because a property appears in JSON or a mockup.

The source conversations are **RustUI Documents Search** (`6ad69904-a38c-83e8-a796-0d4f4db78de6`) and **Generate RustUI Screens** (`6ad6bcc8-8c24-83ea-b45c-3df8e7535503`). The supplied transcript has three omitted early assistant messages in Documents Search and omitted image content in the initial Screens user message; record this limit. The Documents Search conversation reports v1.1 documentation (135 pages), corrected to the real component model, and explicitly says no Rust compile/native UI/hot reload/GPU test was run. These are transcript reports, not proof of current source state.

The actual authoring model to preserve as the primary RustUI model is `#[component]`, `#[state(...)]`, ordinary Rust methods, `View::body`, constructors and `VStack!`/`HStack!`, method actions such as `self.action(Self::increment)`, root mounting through `RustUI`, and keyed independent children through `Component(key, initializer)`. Earlier `ViewCx`/`use_state` ideas were acknowledged as incorrect for the user's current implementation. Verify current details in source before relying on the conversation's claims.

## Baseline to verify

The local reduced `macos27.json` currently has 41 component records and 419 style entries; `full.macos27.json` is its broader source catalogue. Four reduced records are reported to be empty and others may be reference-only or partly consumed. The README documents a v1 adapter with selected appearance/profile metrics, mapped colors/body font size/radii/material opacity, button styling, toolbar metrics and scroll-view measurements; it says several other records are partial/reference-only, native Liquid Glass composition is unsupported, and increased-contrast appearance is missing. Re-derive these facts from code and tests. Preserve all 41 records. Theme data alone is not proof a renderer consumes it.

The conversations report 16 paired light/dark screen concepts covering Studio, component gallery, flexbox/responsive layouts, dashboards, data, settings/forms, Kanban, calendar, assets, editor, workflow, split/code workspace, command/search, setup, and multilingual/accessibility UI. They also report 40 dark Playground concepts covering controls, navigation, data, flexbox, dialogs/panels, typography, media, accessibility, localization, materials and widgets. Final versions were called out for screen 02, screen 16 and P28. The completed local inventory contains 19 paired-gallery candidates and 41 Playground candidates because corrected/alternate versions exist, plus 16 v1.0 and 19 v1.1 specification artworks (95 candidate artworks total, nine exact duplicates deduplicated to 86 files). Use the existing source/message mapping and final/alternate status in `IMAGE_INVENTORY.json`; inspect the stored images as design evidence, and explicitly mark uncertainty where visual classification has not been verified. Never silently discard variants.

## Required analysis and outputs

1. **Current implementation audit.** Trace loader, JSON model, validation, profile selection, token resolution, renderer/adapter calls and Playground consumption. Show what each profile field and each component property actually affects at runtime. Identify dead, reference-only, partial and unsupported data. Cite source/test evidence and state confidence.

2. **41-record component-by-component matrix.** Preserve every existing ID and show current JSON record, intended semantic contract, RustUI/GPUI/native owner, concrete renderer path, appearance/state/variant coverage, screen evidence, accessibility/keyboard/localization coverage, tests, runtime support category, gaps and recommendation. Map screen concepts to families and states; do not collapse distinct controls or omit records. For empty records, recommend explicit supported/placeholder/deferred status with rationale.

3. **Complete theming model.** Propose semantic token families and a clear resolution model for color roles, surfaces, text, accent, status, borders/separators, focus/selection/disabled, typography, spacing, dimensions/hit regions, radii, strokes, icons, materials/elevation, motion and window controls. Define light/dark, system accent, increased contrast, reduced transparency, reduced motion, inactive windows, larger text and platform/profile overrides, with fallbacks and contrast validation. Clearly divide serializable theme data from behavior that needs runtime/native APIs.

4. **Component contracts and interaction/accessibility.** Specify variants, state machines and composition for all major control/layout/panel/data/media families represented in the existing 41 records and screen inventory. Cover keyboard navigation, focus order and rings, shortcuts, hover/press/selected/mixed/disabled/invalid/loading/drag/expanded/error states, cancellation/destructive actions, accessibility roles/labels/values, VoiceOver, contrast and non-color cues, text scaling, reduced motion/transparency, RTL/bidi and locale-sensitive text. Mark each item supported, partially supported, reference-only, native-only or unsupported based on evidence.

5. **Layout/typography/localization.** Define typography styles, scaling, fallback, wrapping/truncation and baseline behavior; layout rules for intrinsic size, flex/grid/stack, constraints, grow/shrink, wrapping, overflow, scroll, split resizing and window adaptation. Include long localized strings, Arabic RTL mirroring, date/number localization and size stress. Treat the generated images as scenarios and aesthetic evidence, never authoritative metrics.

6. **Versioned schema proposal.** Before proposing edits, document actual current schema paths and consumers. Propose a backwards-compatible, versioned JSON structure with typed units/enums, stable semantic references, required/optional rules, cross-field constraints, alias/deprecation policy, unknown-key handling, deterministic precedence/fallback, diagnostics, migrations and generated documentation. Include concise JSON examples and a mapping from every current field/record to the proposed model. Keep the 41 records and 419 styles intact in the proposal; no destructive replacement and no actual file modification.

7. **Prioritized file-by-file strategy.** Provide phases/cards in dependency order with exact file paths, owner, purpose, scope, test types, measurable acceptance criteria, risks and unresolved decisions. Include schema validation, theme coverage lint, runtime consumption, light/dark/accessibility/locale variants, visual regression baselines, dynamic sizing, keyboard and native integration tests. Avoid promising partial GPU repaint or native Liquid Glass behavior without a demonstrated rendering path. Separate native composition and advanced rendering/material work into later gates requiring explicit review.

8. **Decision register and approval gates.** Distinguish decisions the user already made from recommendations, proposals and questions. Identify what needs explicit approval before implementation, especially schema breakage, new public API, dependencies, native platform bridges or advanced GPU/material work.

## Acceptance criteria for your response

- Traceable evidence, with current Apple sources and repository/test citations near each claim.
- A matrix covering exactly all 41 existing component IDs; report count reconciliation and do not lose any record.
- A complete list of implemented/partial/reference/unsupported claims backed by actual consumers.
- No claim that the generated images prove Apple requirements or working implementation.
- No theme/schema/repository edits and no test runs that mutate data. End with the recommended first approved implementation slice, but wait for approval before starting it.

---

