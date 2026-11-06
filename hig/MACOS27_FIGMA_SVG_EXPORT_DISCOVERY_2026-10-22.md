# macOS 27 UI SVG export discovery

Date: 2026-10-22

## Scope and result

Read-only discovery was limited to the likely Chorus project at `/Users/Abium/Developer/chorus`, especially `docs/ui/`, and its directly relevant theming notes. The SVG set found is:

`/Users/Abium/Developer/chorus/docs/ui/macos-27/`

There is no local `.fig` source file or `figma.com/file` / `figma.com/design` source URL in the searched UI documentation. The 38 SVGs therefore appear to be a partial local export set, consistent with the owner's statement that only some exports are present. No Chorus file was changed, and no bulk conversion was attempted.

### User-identified Figma source

The owner supplied this Figma design link as the macOS 27 UI Kit: [macOS 27 Community Copy](https://www.figma.com/design/gJ8x9MVIPCW7yIKFEjv34f/macOS-27--Community---Copy-). The file key in the URL is `gJ8x9MVIPCW7yIKFEjv34f`. The connected Figma app returned page metadata, according to the parent task's read-only verification. This establishes the owner's selected candidate source and an account-readable Figma page. It does **not** independently prove that every SVG in the local 38-file export set came from this exact page, that the local exports cover all relevant frames/variants, or that the file is an official Apple asset. No export-to-frame/hash correspondence was established in this discovery.

## Inventory

There are **38 SVG files** across five groups, totaling **1,023,766,129 bytes (~976 MiB)**:

| Group | Files | Coverage in filenames |
|---|---:|---|
| Root of `macos-27/` | 17 | Menubar (2), Utility Window (2), List (2), Form (2), Notifications (2), Light/Dark Examples, Search Field, horizontal/vertical scrollbar, Clear, Magnifying Glass |
| `Alert/` | 4 | Side-by-side and Window Modal variants |
| `Dialog/` | 4 | Save Large/Small variants |
| `Text Styles/` | 5 | Header and light/dark opaque/vibrant type samples |
| `Windows/` | 8 | Default titlebar/sidebar, expanded toolbar/title, unified toolbar/title, unified compact toolbar/title variants |

The `-1` suffix variants are retained as separately named exports. I did not infer that they are redundant or discard them.

## What the SVGs expose

Representative exports were inspected as XML: `Text Styles/Header.svg`, `Search Field.svg`, `Menubar.svg`, `Windows/Unified Toolbar + Title.svg`, and `Utility Window.svg`. Their markup exposes extractable design data:

- Canvas `width`, `height`, `viewBox`, and positioned geometry (`rect`, `path`, groups and transforms).
- Text content, `SF Pro` font family, font size, weight, letter spacing, baseline/coordinates, and fill. For example, `Menubar.svg` carries 13-unit labels with recorded weights, and `Text Styles/Header.svg` contains 56/28-unit heading/body samples.
- Explicit fill/stroke colors, opacity, shape radii, and named SVG groups/IDs.
- Some effects as SVG filters and backdrop-blur markup (`filter`, `foreignObject`, `backdrop-filter`, and Figma-specific `data-figma-bg-blur-radius`).

These are source-export coordinates and values. The 1512×982 artboards and 3024×1964 embedded desktop wallpapers do **not** establish native macOS point dimensions or display scale by themselves. Typography values can be recovered, but their source status and scaling must be reconciled with official macOS 27 evidence and native observations before treating them as normative.

The exports are mixed vector/raster: large full-window examples embed PNG data in `<image>` nodes (some examples contain 26 embedded images; window canvases commonly contain a 3024×1964 wallpaper image). Other focused component sheets, such as `Search Field.svg`, are mostly geometry and filters. Thus SVG parsing can recover editable vector geometry and style attributes but cannot recover original vector details from embedded raster pixels.

## Provenance found in local documentation

`/Users/Abium/Developer/chorus/docs/THEMING_CONTINUATION.md` says the measured values came from the Apple kit in `docs/ui/macos-27/`, that the SVGs were re-exported with “Outline text” off, and records this type ramp: Body 13/510, Title1 22, Title2 17, Caption1 10, with weights 400/510/590/700/860.

`/Users/Abium/Developer/chorus/docs/ui/ADABRAKA_UI_CLAIMS_STUDY_2026-09.md` describes the directory as a real design kit with window-style SVGs, dialogs, forms, lists, menu bar, notifications, and type-ramp reference.

Those documents establish local project provenance and intended use, but the scanned files do not expose the originating Figma URL or key. The owner-provided link above now identifies the candidate source, but its frame/export relationship to the local SVGs has not been checked. The “macOS 27” folder label, project notes, and user identification do not independently authenticate every drawing as an official Apple asset.

## Limits and next source needed

This discovery confirms useful structured design evidence is available locally, including typography, geometry, colors, states represented in named groups, and some effects. The Figma candidate is now identified by the owner, but the 38 SVGs have not been matched to that file's frames/exports and the export set's completeness remains unknown. It does not establish that every value is native macOS 27 behavior or that unseen frames/components are absent. Preserve SVG variants and embedded image provenance; do not replace the existing theme or bulk-convert these values until frame correspondence, completeness, scale, and official/native evidence are reconciled.
