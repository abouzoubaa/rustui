# X1b: WebAssembly browser host

## Browser font defaults

RustUI selects the pinned GPUI Web platform's `EmojiAndCjk` browser fallback
policy at construction. Loaded fonts remain preferred. This repairs missing
eligible horizontal CJK glyphs without adding font assets or changing the pinned
packages. The upstream fallback renders eligible graphemes independently and is
approximate; it does not establish complete script shaping, vertical text,
variation-sequence coverage or cross-browser font fidelity. The public font
family policy still does not guarantee that a caller-named system face exists.

The Web bootstrap registers Inter Variable, Noto Sans, Noto Sans Arabic, Noto
Sans Hebrew, and Fira Mono Medium before component initialization. Inter and
Noto Sans Hebrew were added at M781; the other assets were copied unchanged
from `vendor/cosmic-text-0.19.0/fonts/`, with their licenses included in
`crates/rustui/assets/fonts/`.

GPUI's global fallback list contains Noto Sans, providing a fallback for
`.SystemUIFont` when the Web backend's IBM Plex Sans alias is unavailable.
Components' automatic monospace alternate list does not include Fira Mono,
so the owned Web bootstrap explicitly selects `Fira Mono` using the public
`Theme::global_mut(cx).mono_font_family` field immediately after component
initialization. Wasm selects Inter as its sans family; native font selection
is unchanged. Caller `on_launch` customization runs afterward and retains
control over its theme. This adds no dependency or vendored source change.
Only the Medium monospace face is bundled; browser weight selection and glyph
coverage remain unverified.

## Selected target

The Stage 11 target is a WebAssembly application hosted in a browser. The existing RustUI Playground remains the single canonical application.

## Implemented host path

Before GPUI Components initialize, the Wasm runtime registers Inter Variable,
Noto Sans, Noto Sans Arabic, Noto Sans Hebrew, and Fira Mono assets. It selects
Inter and Fira Mono as the Web sans and monospace defaults through the public
Components theme API. Native builds continue using platform fonts.

- The Playground binary exports a `wasm_bindgen(start)` entry point on Wasm and calls the existing public `rustui_playground::run()` function.
- `RustUI::run` initializes GPUI's web panic hook and browser logging on Wasm. Its existing platform factory selects GPUI's Web backend for `wasm32-unknown-unknown`.
- The Playground supplies the existing GPUI Kit asset source using its Wasm default endpoint; native builds continue to use the embedded unit source.
- The direct `wasm-bindgen` dependency is target-specific and pinned to the version already present in `Cargo.lock`. No GPUI or GPUI Components revision changed.
- The package script rejects `wasm-bindgen` CLI versions other than the locked 0.2.129 before building; a simulated 0.2.128 CLI was rejected before the build command.
- `apps/rustui-playground/web/index.html` provides the browser shell. `scripts/build-playground-web.sh` builds the optimized Wasm binary, runs `wasm-bindgen --target web`, and copies the shell and SVG references. It also packages the font license files under `licenses/fonts/` for the font assets embedded in Wasm. It discovers `rust-lld` from the installed Rust 1.97.1 host toolchain when needed.
- The package guard also requires the shell to import and invoke the generated module initializer in an executable module script. Its focused regression rejects an apparent bootstrap that exists only inside a multiline JavaScript template literal, preserving the prior bundle on rejection.

## Focused evidence

The locked workspace sources type-check for the selected target with the existing Headstart Wasm standard library:

```sh
CARGO_TARGET_DIR=/Users/Abium/Developer/rustui/target/headstart/x1a-wasm \
RUSTUI_HEADSTART_ROOT=/Users/Abium/Developer/rustui/target/headstart/wasm-root \
./scripts/cargo-headstart check -p rustui-playground \
  --target wasm32-unknown-unknown --locked --offline
```

Result: passed. Log: `target/headstart/evidence/m627-stage11-playground-wasm-check-20261022.log`.

The release packaging script also completed and produced the browser bundle in `target/headstart/playground-web` (34 MB Wasm plus generated JavaScript and declarations). Log: `target/headstart/evidence/m627-stage11-web-release-bundle-build-20261022.log`. The matching `wasm-bindgen` CLI 0.2.129 was installed only under `target/headstart/tools`.

## M688 browser first-render evidence

The optimized bundle was rebuilt with the generated Headstart stage1 sysroot that already contains `wasm32-unknown-unknown` libraries. The local tool-root links and Cargo outputs stayed under `target/headstart/`; no installed toolchain or dependency revisions changed. The focused check passed using `target/headstart/wasm-sysroot-check-fira`, and the release bundle was published to `target/headstart/playground-web`.

After a cache-bypassing reload in Chrome at `http://127.0.0.1:8765/`, the Playground header, catalog, selected Button story, and canvas content rendered. The HTML shell, generated JavaScript, and Wasm requests returned 200. The only console error was the unbundled `favicon.ico` request returning 404. At that time the reference pane lacked `Form.svg` and `Form-1.svg`; M703 later packaged them. Browser input, selected-story asset fetching, browser accessibility output, and full font weight/glyph coverage were not checked in this run. Structured observation: `target/headstart/evidence/m688-web-first-render.json`.

## Remaining acceptance

X1 browser support still needs pointer and keyboard interaction, selected-story asset-fetch checks, browser accessibility observations, and broader glyph and platform coverage. First render does not close those gates.

## M703 browser reference links

The Web bundle now publishes the Playground's existing SVG reference directory
under `references/`, and the browser references pane opens those files through
RustUI links instead of attempting native filesystem access. Reference lookup
uses the packaged basename, matching the existing flat reference bundle layout
for nested catalog paths. A focused path regression was added. Browser
fetch/navigation, keyboard interaction, accessibility, and the remaining Stage
11 acceptance checks are still open.


## M711 embedded Playground sample image

The Primitive gallery now names its sample image as an application asset. RustUI's `with_embedded_asset` helper composes the compile-time PNG bytes with the existing GPUI Kit asset source, keeping GPUI asset-source implementation inside the framework and preserving the Kit source for other assets. The focused PNG-signature and nested-path regression passed, as did the locked/offline `rustui-playground` check for `wasm32-unknown-unknown`. This does not close browser pointer, keyboard, accessibility, or broader glyph acceptance.

## M715 browser document locale metadata

The Playground locale selector now updates the browser document `lang` and `dir` after a locale change, using RustUI's locale direction policy. The focused English/French/Arabic direction regression and locked/offline `wasm32-unknown-unknown` check passed. Browser screen-reader behavior remains unverified.

## M781 Web font coverage

The Web runtime also registers the licensed Inter variable font and Noto Sans
Hebrew, and selects Inter for the Wasm sans family. The focused registration
test and locked/offline Playground Wasm check passed. Browser weight selection,
glyph rendering, accessibility, and full Stage 11 acceptance remain open.

## 2026-10-22 refreshed browser package assets

`./scripts/build-playground-web.sh` completed successfully in about 19 seconds.
The generated package now contains `references/Form.svg` and
`references/Form-1.svg`, confirming the existing reference-copy packaging path.
This is packaging evidence only; browser fetch, input, and accessibility
acceptance remain open.

## M786 keyboard focus boundary

Browser pointer activation, the packaged Form SVG link, and Arabic RTL layout
were observed. Space did not activate the selected Button after explicitly
focusing the GPUI canvas (`tabindex=-1`); the pinned host routes keyboard input
through its hidden IME textarea. No RustUI-only fix was verified. Keyboard and
screen-reader acceptance remain open pending an approved host-level change.

## 2026-10-23 focused browser follow-up

The existing `target/headstart/playground-web` bundle was older than the current
shell and packaging script. `./scripts/build-playground-web.sh` rebuilt it
successfully in 23.77 seconds. The packaged `index.html` SHA-256 matches the
source shell (`972b5570…d9001`); the package contains 31 SVG references and the
Fira Mono, Inter, and Noto Sans license files. The local server returned HTTP
200 for `/`, `/references/Form.svg`, and an encoded-space SVG path. This proves
current packaging and static serving, not a fresh browser load.

Before the rebuild, Chrome showed the older Playground bundle. Keyboard text
entry in the catalog search and pointer selection of the Image row worked, and
the image specimen rendered. This observation is not current-bundle acceptance.
After rebuilding, Chrome DevTools could not attach because its profile was
already running; CUA's Chrome binding moved to a separate user page, so no
further browser navigation was attempted. In the earlier Playground snapshot,
the browser accessibility tree exposed the page as generic HTML content with an
image and text-entry area, without the canvas controls. Fresh-bundle pointer
and keyboard checks, selected-story network evidence, browser accessibility

## 2026-10-23 shell loading-status accessibility
Removed `aria-busy` from `<body>` while the loading message is inside that same subtree. A busy ancestor can defer live-region announcements, so the `role="status"` loading message could be suppressed. The focused Web packaging test now checks that the status region and loading text remain present without a body-level busy state. `sh scripts/tests/test_build_playground_web.sh` passed. This source-level fix does not establish browser screen-reader acceptance; the remaining runtime acceptance above is still open.

## 2026-10-24 rebuilt-bundle browser follow-up

The current Playground Web bundle was rebuilt successfully with
`./scripts/build-playground-web.sh` (release build: 23.92 seconds). The packaged
`index.html` SHA-256 matches the source shell. A fresh in-app browser tab loaded
the Playground canvas and displayed the Button story. The browser asset
inventory observed the generated JavaScript and Wasm; it did not observe an SVG
reference request during this page state. The local server returned HTTP 200
for `/`, `/rustui_playground.js`, `/rustui_playground_bg.wasm`, and
`/references/Form.svg`; the SVG check was a direct request, not a story-triggered
browser fetch.

The browser accessibility snapshot exposed a generic page image and a text-entry
area, without the visible Playground controls. Typing into the focused text
area did not change the story, so current-bundle keyboard interaction was not
verified. Pointer activation was not verified. The browser console showed the
WebAssembly-threading fallback warning and no captured errors. The DevTools
connector could not attach because its Chrome profile was already running. The
visible Arabic labels provide only a small glyph sample; broader glyph and
font-weight coverage remains open. Stage 11 acceptance remains incomplete.

## 2026-10-24 Web focus-boundary follow-up

In a fresh in-app browser tab, clicking the center of the Web canvas left the
GPUI hidden `TEXTAREA` as `document.activeElement`. Pressing Tab from that text
entry area moved DOM focus to `BODY`; no visible Playground focus change was
observed. This did not focus a specific canvas control, so it is a bounded
focus-boundary observation rather than a control-keyboard acceptance result.

## 2026-10-24 existing-bundle pointer and reference follow-up

A fresh Codex in-app browser tab loaded `http://127.0.0.1:8765/`. The served
WebAssembly bundle files were timestamped 04:45 UTC, before the Menu Bar story
source at 05:12 UTC; this run does not exercise M1194. Pointer
activation on Increment changed the visible count from 0 to 1. Selecting the
Image catalog row rendered its embedded purple/cyan sample image. Following
that selected story's `Open List.svg` link opened
`/references/List.svg` in a new tab; its accessibility snapshot exposed the
Finder reference text. These checks establish pointer activation, selected
story rendering, and packaged SVG navigation for this browser state.

The Playground accessibility snapshot still exposes only a generic page image
and text-entry area. Space left the count at 1, and the following Tab showed no
visible control focus. This is a bounded unsuccessful keyboard observation,
not a full keyboard traversal. Screen-reader behavior, network status capture,
full font/glyph coverage, and cross-browser/platform acceptance remain open.

In a second fresh in-app browser tab, selecting Arabic visibly mirrored the
catalog and references pane and rendered Arabic coverage labels. Increment
still changed the count from 0 to 1 in that RTL layout. This visual and pointer
sample does not establish document-level `lang`/`dir`, full script coverage,
keyboard, or screen-reader acceptance.

