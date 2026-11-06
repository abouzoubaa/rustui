# RustUI Playground

The canonical development app for the new RustUI framework. Its only direct Rust
dependency is `rustui`; rendering, native controls and window setup go through the
framework public API. The application contains no direct GPUI imports or Render
implementations.

The 1320 × 820 window retains the 250px catalog sidebar, 300px reference panel,
macOS 27 theme, copied SVG references, search and Light/Dark actions. Ten migrated
framework demonstrations live under `src/stories/`, including inputs, collections,
navigation and retained child state/props/environment. Unsupported catalog entries
remain explicit. This is a source migration, not a pixel-perfect acceptance claim.

The native app bundle stores references in `Contents/Resources/references`;
source-checkout runs use `apps/rustui-playground/references`. The current
reference set is about 727 MB, so packaging copies the full catalog.

Run `./scripts/cargo-headstart playground` from the repository root. For a native
macOS bundle, build with `./scripts/cargo-headstart build -p rustui-playground
--locked`, then run `apps/rustui-playground/package-app.sh`. The old `rustui-lab`
package is a thin launcher for this same implementation.

See [the Playground guide](../../docs/RUSTUI_PLAYGROUND.md) and the consolidation
record in [the roadmap](../../docs/IMPLEMENTATION_ROADMAP.md).

## WebAssembly browser smoke

The canonical Playground has a browser start entry. With the matching `wasm-bindgen` CLI 0.2.129 installed, run `./scripts/build-playground-web.sh` from the repository root, then serve the generated `target/headstart/playground-web` directory with the printed Python HTTP server command. The browser smoke page uses GPUI's full-viewport canvas. Runtime and accessibility acceptance are still pending.
