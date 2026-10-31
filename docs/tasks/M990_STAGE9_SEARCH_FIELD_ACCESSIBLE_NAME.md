# M990 — Stage 9 / name SearchField when its placeholder is blank

SearchField now uses `Search` as the accessible-name fallback when its
placeholder is blank or whitespace-only. Nonblank placeholders remain the
accessible name.

Focused check: `components::input::search_field::tests::blank_placeholder_still_identifies_search_field`
passed (1/1). Rustfmt and scoped whitespace checks passed.

Native screen-reader acceptance remains open.
