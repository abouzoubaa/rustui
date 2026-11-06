# Changelog

- T11c: Added RustUI-owned GPUI Base and Component sources and the patched block source.

- T11b: Added RustUI-owned macOS GPUI platform crates with source-copy verification.
- T11a: Added the RustUI-owned GPUI core crate while preserving the pinned upstream source and attribution.

- Stable Rust 1.97.1 is the daily compiler; the Headstart script remains optional.
- Removed retired development-process instructions from repository documentation.
- T3: archived retired planning documents; native macOS is first, with Web/localization/accessibility deferred and Windows/Linux last.
- Playground packaging now refuses an existing app bundle unless --replace is supplied.

- Split the compiled preview gallery into focused modules without changing behavior.
- Split graph view into model, validation, rendering, and interaction modules without changing behavior.
