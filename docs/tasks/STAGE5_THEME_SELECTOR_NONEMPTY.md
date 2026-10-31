# Stage 5 theme selectors must be nonempty

Component theme style selectors use `any` for an unconstrained field and exact
string equality for a constrained field. A blank or whitespace-only selector
can never match a runtime query, yet schema validation previously accepted it.

`ThemeDocument::validate` now rejects blank values across the eight selector
fields for component style rules. The focused regression mutates a parsed
TextField rule and confirms validation reports the component, rule index, and
field. This is source-level schema validation; theme visual, interaction,
accessibility, and cross-platform acceptance remain separate.

The focused regression passed 1/1, and the locked/offline RustUI library check
passed:

```text
./scripts/cargo-headstart test -p rustui --lib component_theme_rejects_empty_selector_values --locked --offline
./scripts/cargo-headstart check -p rustui --lib --locked --offline
```

Scoped diff checks passed. The file-wide rustfmt check still reports existing
formatting differences outside this change.
