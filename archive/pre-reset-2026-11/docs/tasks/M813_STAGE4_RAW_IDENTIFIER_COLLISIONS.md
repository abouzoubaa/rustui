# M813: raw identifier collision checks

## Scope

Treat Rust raw identifiers as their underlying identifier when checking
generated component helper and accessor collisions.

## Change

The component macro now normalizes the `r#` token spelling before collision
checks. This catches `r#action` conflicting with the generated `action` helper
and fields such as `value` conflicting with a generated accessor from
`r#value_binding`.

## Evidence

- `./scripts/cargo-headstart test -p rustui-macros generated_state_accessors_reject_raw_identifier_name_collisions --lib`
  — 1 focused test passed.
- Rustc independently confirmed `fn action` and `fn r#action` are duplicate
  definitions.

Broader macro integration acceptance remains open.
