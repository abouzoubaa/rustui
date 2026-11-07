# Changelog

- T11b: Added RustUI-owned macOS GPUI platform crates with source-copy verification.
- T11a: Added the RustUI-owned GPUI core crate while preserving the pinned upstream source and attribution.

- Stable Rust 1.97.1 is the daily compiler; the Headstart script remains optional.
- Removed retired development-process instructions from repository documentation.
- T3: archived retired planning documents; native macOS is first, with Web/localization/accessibility deferred and Windows/Linux last.
- Playground packaging now refuses an existing app bundle unless --replace is supplied.

- Split the compiled preview gallery into focused modules without changing behavior.
- Split graph view into model, validation, rendering, and interaction modules without changing behavior.
- Added GPUI test-window coverage for Popover and Menu lifecycle behavior.
- Added ignored Dialog and Sheet tests; focus recovery and the strict background-focus guard remain unresolved pending an incomplete proposal.
- Promoted Dialog and Sheet focus recovery with active GPUI test-window coverage.
- Added validation and validity bindings for text, secure, and search fields, with a Playground story.
- Added the editable DataGrid design for owner approval.
- Added typed editable DataGrid cells with validation and basic navigation.
- Add stable view-only sorting to EditableDataGrid.
- Add editable grid navigation, conflict recovery, and accessibility metadata.
- Add typed theme tokens and navigation surface capabilities.
- Add keyed appear and disappear transitions using GPUI springs and inert focus scopes.
- Fix editable grid Tab and Shift-Tab focus exit at its edges.
- Expose validation descriptions and editable grid read-only, sort, and header semantics.
