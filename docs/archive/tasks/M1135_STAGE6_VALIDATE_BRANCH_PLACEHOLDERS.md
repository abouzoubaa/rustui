# M1135 — validate placeholders within expression branches

## Change

Catalog validation now retains the argument names used within each plural or
select branch and compares them against the corresponding source branch. A
translation can no longer move a required placeholder between branches while
preserving the same global placeholder set and expression shape.

## Focused evidence

```text
./scripts/cargo-headstart test -p rustui-i18n --lib detects_placeholders_moved_between_expression_branches
1 passed
```

The regression failed before the fix and passed afterward. The scoped diff
check passed. Full CLDR, Fluent/ICU, and native localization acceptance remain
open.
