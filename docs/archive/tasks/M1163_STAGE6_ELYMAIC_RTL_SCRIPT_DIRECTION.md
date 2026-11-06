# M1163 — Stage 6 Elymaic script direction

The locale direction resolver now recognizes the valid BCP 47 script subtag
`Elym` as right-to-left. Before this change, an explicit Elymaic script fell
through the partial RTL script set and resolved to left-to-right.

## Focused evidence

- `./scripts/cargo-headstart test -p rustui-i18n explicit_script_overrides_language_direction` passed (1 passed, 72 filtered).

This extends the explicit-script direction list only. Full CLDR coverage and
native RTL GUI acceptance remain separate work.
