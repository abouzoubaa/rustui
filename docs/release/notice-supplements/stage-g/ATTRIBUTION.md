# Stage G source attribution

These unmodified source snapshots are preserved for the Stage G ports under `patches/candidate-fast-damage/upstream-sources/`. Their exact repository, revision, upstream path, local path, SHA-256 and license are pinned in `SOURCES.json` there. These reference snapshots are not compiled into RustUI. When ported code enters a package, its corresponding attribution must also be attached to that package in the supplement manifest so the release collector includes it.

## GPUI Fast planner

Source: [longbridge/gpui-fast at 2002081b4a61249a1cef2b8a552f6ba0998d9f52](https://github.com/longbridge/gpui-fast/tree/2002081b4a61249a1cef2b8a552f6ba0998d9f52), `crates/gpui_windows/src/fast/partial.rs`.

Copyright 2022 - 2025 Zed Industries, Inc. Copyright 2026 Longbridge. Licensed under Apache-2.0; the repository's verbatim license and notices are in [gpui-fast-LICENSE](gpui-fast-LICENSE).

## Havunen retained frame and Metal renderer

Source: [Havunen/gpui-ce PR #29](https://github.com/Havunen/gpui-ce/pull/29), pinned to its head commit `28045891bfe87768a2c8e0aefd43403939844318` at retrieval on 2026-11-20:

- `crates/gpui_render/src/damage.rs`
- `crates/gpui_render/src/retained.rs`
- `crates/gpui_apple/src/metal_renderer.rs`

Attribution: Havunen and the gpui-ce contributors; the Apple crate retains Copyright 2022 - 2025 Zed Industries, Inc. Licensed under Apache-2.0, as declared by the workspace and Apple package. The repository's verbatim license is [gpui-ce-LICENSE.md](gpui-ce-LICENSE.md); the Apple crate's copyright notice and full license are [gpui-ce-apple-LICENSE-APACHE](gpui-ce-apple-LICENSE-APACHE).

Apache-2.0 permits copying and modification subject to its terms. Preserve these license/copyright notices, mark RustUI modifications prominently in ports, and retain any applicable upstream notices. The source snapshots remain unmodified.

Finityno and Zed PRs are idea references only; no code from those PRs is copied into the Stage G source snapshots.
