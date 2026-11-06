# Existing macOS 27 SVG conversion and RustUI compatibility audit

Date: 2026-10-22  
Scope: read-only inspection of the existing Chorus SVG export set, its conversion script and theme packs, plus RustUI's theme loader/resolver/renderer declarations. No conversion script, build, unit test, app, Git command, Trail command, or production write was run. The only intended write for this audit is this report inside `hig/`.

## Executive finding

The conversion is a real, evidence-traced data pipeline—not just a list of guessed values. It pins source SVG SHA-256 hashes, uses explicit ownership contracts to identify source nodes, records selectors and per-property source traces, and generates a full appearance pack. The normalized component data in that pack matches the current production RustUI theme's 41 component IDs and 419 style rules.

That does **not** mean the full archival pack can be loaded by the current RustUI loader, that every extracted property affects rendering, or that the appearance is proven native macOS 27. The archival `rustui.macos27.json` has source-manifest and style provenance fields that the current strict Button schema rejects. The full pack is accepted by a separate Chorus UI Studio reader. Existing production JSON is the slimmed, runtime-shaped counterpart. Several extracted effects/path values are retained as reference metadata and are not painted by the RustUI renderer.

**Disposition:** preserve the source originals and complete pack; do not run the legacy generator against them or copy the full pack into a runtime location. Treat this as a useful but partially consumable conversion with a clear compatibility boundary. No replacement values are proposed in this audit.

## Source identity and file relationships

| Artifact | Verified identity | Meaning |
|---|---|---|
| SVG export root | `/Users/Abium/Developer/chorus/docs/ui/macos-27/` | 38 SVGs across root, Alert, Dialog, Text Styles, and Windows; approximately 976 MiB. |
| Converter | `/Users/Abium/Developer/chorus/scripts/derive_rustui_macos27_styles.py` | Deliberate offline migration tool. It names `apps/ui-studio/assets/appearance/rustui.macos27.json` as its pack output and `RUSTUI_APPEARANCE_COVERAGE.md` as its coverage output. Its header says normal runtime/builds do not read SVG sources. Not executed. |
| Full pack in Chorus | `/Users/Abium/Developer/chorus/apps/ui-studio/assets/appearance/rustui.macos27.json` | 2,028,537 bytes, SHA-256 `e558ea7aa45068ab6a18124451d4d82676be67017440d279d52937161dfc9683`. |
| Full pack copy in HIG | `/Users/Abium/Developer/rustui/hig/full.macos27.json` | Byte-identical to the Chorus pack above. It contains the source manifest and per-style provenance. |
| Current RustUI theme | `/Users/Abium/Developer/rustui/themes/macos27.json` | Byte-identical to `/Users/Abium/Developer/rustui/hig/macos27.json` and the preserved production snapshot: 820,587 bytes, SHA-256 `5a4a81adb6af226d59af916a4feadf575b23447b7cee2517882e9bdf2453e01e`. This is a distinct, reduced runtime-shaped file. |
| UI Studio input | `/Users/Abium/Developer/chorus/apps/ui-studio/assets/appearance/macos27.allui.json` | Allui appearance input named by the full pack's derivation metadata; not the same artifact as the RustUI full pack. |
| Existing coverage report | `/Users/Abium/Developer/chorus/apps/ui-studio/RUSTUI_APPEARANCE_COVERAGE.md` | Documents source rows, selected call sites, resolved-only rows, known projection gaps, and the no-fidelity-claim boundary. Read only. |
| Local provenance note | `/Users/Abium/Developer/chorus/docs/THEMING_CONTINUATION.md` | Says the files were exported with Figma text outlines off and records the intended typography sampling. It does not expose a Figma file URL/key. |

A read-only check compared all 38 SVGs to the SHA-256 entries in the full pack's source manifest: **38/38 matched**. This proves identity with the manifest, not authorship or completeness. The owner has now supplied the Figma URL [macOS 27 Community Copy](https://www.figma.com/design/gJ8x9MVIPCW7yIKFEjv34f/macOS-27--Community---Copy-) as the relevant macOS 27 UI Kit; the connected Figma app's page metadata was readable according to the parent task. This is a user-identified source, not yet an independently verified mapping from the 38 local SVGs to exact frames/exports in that file. No browser account search or re-export was performed for this audit. The link and local project naming do not independently authenticate it as an official Apple asset.

## Conversion quality and evidence

### What the converter does well

Code inspection shows the converter:

- Parses SVG XML and extracts authored colors/alpha, lengths/radii, borders, typography, filters, blend modes, vector path data and selected cursor geometry.
- Stores source file/label, selector, SVG node identity/group ancestry, source attribute, authored literal and affine transform alongside extracted values.
- Uses explicit per-component/per-part ownership contracts and checks unique node ownership, expected authored values, and source hashes. For paired `-1` variants it projects source-backed counterpart rows only when a semantic scope exists.
- Deliberately leaves unsupported composite forms and surfaces as gaps (for example sidebar/toolbar/list/tab/menu-bar bodies and the Stepper composite); it does not silently invent a single Stepper body from unrelated source primitives.
- Includes self-check code for ownership cases and documents that paths/filters/blend modes remain source data when the public RustUI renderer cannot represent them.

The pack records 41 components, 419 style rules, 1,358 style properties, 419 source references, and property-level evidence for the extracted declarations. Property type counts: foreground 346, typography 340, blend mode 193, background 118, width 78, height 78, vector path 60, corner radius 56, filter 55, border 18, border width 8, and cursor width/height/radius/color 2 each. All recorded transforms inspected here have identity linear terms (no scale/rotation in their linear component); translation is retained. These counts are structural, not feature coverage or confidence scores.

### Findings that still need design review

- **SVG unit is not established as native point size.** The extractor writes SVG user-unit values as `kind: length, value: {px: ...}`. The current runtime profile says `measure_unit: logical_px`, but a 38-file sample of Figma-export coordinates is not by itself proof that each exported rectangle is a native macOS control measurement at the runtime's logical scale. Record scale/source-context and confirm against official semantics or a real macOS 27 capture before elevating a value to normative.
- **Component canvas versus control geometry must be distinguished.** `Scrollbar - Horizontal.svg` has a 250×90 SVG artboard. The converter's exact ownership trace for the `scroll_view/scrollbar_horizontal` style selects the enclosing 249×89 rounded background rectangle, while the visible Thumb variants inside that artboard are 8×6, 15×6, or 29×6. That 249×89 rectangle is an export/container surface, not the scrollbar thumb's dimensions. The current scrollbar renderer capability does not consume width/height, so this does not affect its listed renderer fields today; nevertheless the width/height records must not be interpreted as native thumb dimensions. The future contract should model track/container, thumb and page/position states independently.
- **Paired appearance and state coverage is uneven.** The converter's existing coverage report identifies no dark Search Field sheet with the six text/focus/disabled variants; the full pack only has the corresponding light state sheet plus separate dark search examples. Several SVGs are complete window compositions with raster images, so their frame values should not be confused with primitive widget measurements. Static selector assignment from file/group labels requires review where no explicit native state is drawn.
- **Text family anomaly to resolve.** `Text Styles - Left Aligned - Light Vibrant.svg` contains a Title2 sample authored with `Inter`, while most comparable records use `SF Pro`. The converter preserves the literal source family; determine whether that row is intentional kit artwork, a fallback, or an export anomaly before treating it as the platform font role.
- **Artwork fidelity is partial.** SVG vectors and property evidence are preserved, but the converter skips embedded `<image>`/foreign-object payloads for primitive extraction, and not every filter/mask/clip/blend operation can become a runtime effect. SVG parsing cannot reconstruct vectors from its raster payloads. Source extraction quality and rendered visual equivalence are separate gates.

These are review flags, not assertions that the source converter selected the wrong node in every context. The explicit contracts and traces make such issues auditable; they do not make the authored source semantics self-evident.

## Current RustUI loader contract

Read-only schema/resolver inspection covered:

- `crates/rustui/src/theme/schema.rs` (`ThemeDocument`, Button schema, generic component records, length/color/typography/profile types)
- `crates/rustui/src/theme/loader.rs` (`parse_theme`, `load_theme`, typed serializer)
- `crates/rustui/src/theme/resolver.rs` (`resolve_component_style_with`, `validate_component_theme`, coverage calculation)
- `crates/rustui/src/theme/registry.rs` (`RENDERER_CAPABILITIES`, menu/table helpers)

Compatibility distinctions:

1. `ThemeDocument` is `#[serde(deny_unknown_fields)]`. The full archival pack has a top-level `source_manifest` key that is not in the current RustUI `ThemeDocument`; therefore the full file cannot be passed directly to current `parse_theme` as-is.
2. Button uses strict typed `ButtonStyleRule`, `ButtonStyleDefinition`, and `ButtonStyleProperties`. Full-pack provenance fields such as per-rule `sources` and style-level `evidence` are not accepted by these strict Button structs. By contrast, non-Button generic component/style records use flattened JSON metadata and can preserve arbitrary fields, but those fields remain metadata and are not magically interpreted by the renderer.
3. Current v1 style lengths are typed as `{kind:"length", value:{px:number}}`; colors as RGBA; typography as family/size/weight. Button allows only its known width/height/background/foreground/typography shape. Generic part rules validate selector matching, property types and resolver support. An unknown property rejected by the resolver is an error unless the caller explicitly handles it; adding a key to JSON does not add a rendering capability.
4. A structural comparison showed the full pack's `components` become exactly equal to the production theme's `components` when the full-pack `sources` and `evidence` fields are recursively stripped. At top level the full pack also has a separate source manifest and richer derivation/platform metadata. This is an offline structural normalization comparison, **not** a call to RustUI's parser and not a claim that the entire full file is runtime-loadable.
5. The current production theme is the loadable counterpart to preserve for runtime use. The application’s tests reference `themes/macos27.json`, but no loader test or build was run during this audit. Current source has parser tests for that file; their existence is not a newly observed pass.

**Compatibility verdict:** `full.macos27.json` / `rustui.macos27.json` = rich source/evidence tooling pack; not drop-in loader input. Current `themes/macos27.json` = runtime-shaped file with the same normalized component style data; do not replace it with the archival pack. A future provenance-preserving runtime format needs an explicit schema/version/migration decision, not unchecked extra keys.

## Data-to-renderer path: extracted does not mean consumed

RustUI’s common resolver interprets lengths, colors and typography. Unknown property names are rejected unless a caller lists them as handled extras; handled extras are stored opaquely in `extra_properties`. Renderer capabilities are declared per component and part. They are narrower than the extracted data.

- The full pack contains **308** filter/blend/vector-path property instances (55 + 193 + 60). These are evidence/artwork descriptions, not a promise of live filters, blend modes or vector rendering.
- `menu_background` explicitly allows `filter`, `blend_mode`, and `vector_path` through style resolution, then returns only a background color to its caller. It does not apply the recorded effect/path.
- The table-header helper allows `vector_path`, then returns only background/foreground colors; the path is explicitly treated as source artwork and is not drawn by that helper.
- Cursor geometry/color records remain data because GPUI Components owns the native cursor painting path, as documented by the coverage artifact.
- The existing UI Studio coverage file explicitly separates rendered call sites from resolved-only records and says “generated appearance fidelity unverified.” It also states the projection boundary: body colors/geometry/typography can be applied where public RustUI adapters expose them; filters, blend modes and vector paths remain exact catalog data.

Thus there are at least three distinct checks for every imported property: (a) source trace proves where it came from; (b) schema/resolver proves whether current JSON shape accepts/resolves it; (c) concrete consumer proves whether it changes actual rendering. A fourth macOS 27 visual/interaction comparison is required before claiming native fidelity.

## Representative conversion trace

| Family | Existing SVG → JSON evidence | Consumer finding | Status |
|---|---|---|---|
| Button | `Form.svg` and `Form-1.svg`, `button/body`; 67×24 SVG user units, black 8% surface, white 85% text, SF Pro 13/510. Evidence traces the background rectangle and text label separately. | Runtime capability declares width, height, background, foreground and typography. Strict Button schema accepts only the typed runtime rule, not the archival rule `sources` or style `evidence` metadata. Radius is not recorded in this Button style rule. | Core values compatible after provenance metadata is separated; visual and source-scale status still unverified. |
| Search field | `List.svg`, `search_field/list_search`; 130×36, radius 18, gray 17% surface, SF Pro 13/510, plus an SVG filter and `luminosity` blend. Evidence shows filter on `Fill + Shadow_5`, separate rectangle/text traces. | Capability names dimensions/radius/colors/type. It does not list filter or blend-mode as renderable; coverage reports effects unresolved. Dark 6-state sheet missing. | Extracted core style is useful; effect and state parity incomplete. |
| Table header | `List.svg`, `table/table_column_headers`; background/text/11-unit bold text and a vector path representing a one-unit separator. | Helper only returns header background and foreground. The vector path is explicitly “source artwork” and is not rendered by that helper. | Partial rendering; path is evidence only. |
| Scrollbar | `Scrollbar - Horizontal.svg`; root artboard 250×90; style owns outer 249×89 background rectangle; distinct thumb sizes/positions are nested under page/position groups. | Runtime scrollbar capability lists background, border and corner radius, not the extracted 249×89 dimensions. | Do not interpret artboard/container dimensions as native scrollbar/thumb dimensions; needs semantic subparts/states. |

The stated call-site and projection findings are source inspection of the existing project coverage artifact and RustUI code—not a fresh runtime observation in this turn.

## Safe next steps for future conversion work

1. Keep the 38 source exports immutable and retain their exact hashes/provenance. Compare the owner-identified Figma file's frames and exports against the local files and source manifest; establish export completeness before calling the collection complete.
2. Continue using ownership contracts, per-property traces and paired-variant checks, but add semantic shape roles (track/thumb/container; frame/control/label; material/content) so artboard geometry cannot be mistaken for a control.
3. For every emitted property write a target: accepted current-v1 field, preserved source metadata, or explicitly future-v2 design property. Never add an unknown field to a purported loadable v1 file.
4. Cross-check source dimensions against display scale and native macOS 27 observations. Keep any SVG literal that lacks this mapping labeled `source user units` or provisional; do not call it a measured native point value.
5. Use the per-part RustUI capability table and actual consumer code as the compatibility gate. Preserve unconsumed effects/artwork in a source/evidence pack, separately versioned from the runtime document. Add rendering support only in a separately approved production change.
6. Resolve the Dark Search Field state gap and the Title2 `Inter` source anomaly; inspect additional Figma exports before broadening the data set. Leave missing content marked missing.
7. After the user separately authorizes an implementation, test parser acceptance, resolver selection/precedence, serialization, consumer output, and paired light/dark interaction states. These checks were **not run** here; no test/build was run for this audit.

## Limits

- The owner-identified Figma URL is known, and its page metadata was readable through the connected Figma app per the parent task. Exact correspondence between that file's frames/exports and the local 38 SVGs, export completeness, and official provenance remain unverified.
- No macOS 27 runtime/reference image, API availability test, native measurement, or live RustUI render comparison was made. The codebase's fidelity gate remains `NOT DEMONSTRATED`.
- No conversion code was executed, no binary asset was edited, and no source, production theme, Chorus file, Git state, build output, or Trail cache was changed by this audit.
