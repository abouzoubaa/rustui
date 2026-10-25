# M903 — Stage 3 / interactive Form Accessories specimen

The `form_accessories` catalog row previously had no dedicated Playground
specimen. It now routes to `FormAccessoriesGallery`, composed from the public
RustUI API. The filter field applies a trimmed, case-insensitive match to three
illustrative items; a blank query restores the full list. Detail, information,
and action controls expose their toggled states with explanatory content.

Focused checks passed:

- `./scripts/cargo-headstart test -p rustui-playground form_accessories_catalog_routes_to_its_public_api_composition --lib` — 1 passed.
- `./scripts/cargo-headstart test -p rustui-playground filter_matches_case_insensitively_and_blank_query_restores_all_items --lib` — 1 passed.
- `rustfmt --edition 2024 --check apps/rustui-playground/src/stories/form_accessories_gallery.rs` — passed.

The catalog remains `Partial`: this specimen uses a small illustrative item
set and does not cover every macOS 27 reference combination. Native visual and
accessibility acceptance remains open.
