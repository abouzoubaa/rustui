# List row accessible names

`ListView` uses each item string as the accessible label for its virtualized row. Its constructor now rejects empty and whitespace-only labels before they reach rendering, while preserving nonblank strings verbatim. Focused RustUI unit tests cover rejection and preservation.

Validation: both focused locked/offline RustUI unit tests passed (2/2); `rustfmt --edition 2024 --check crates/rustui/src/components/data/list.rs` and the scoped `git diff --check` passed. This is source-level validation. Native accessibility-tree and screen-reader acceptance remain part of the broader collection acceptance work.
