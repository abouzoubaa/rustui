# M881 — Stage 4 view-builder documentation attributes

## Change

The view-builder macro now hoists `cfg_attr` onto a generated child-builder
binding only when the attribute contains a `cfg` condition. Documentation-only
attributes remain attached to the child expression instead of an intermediate
generated `let` binding.

## Focused evidence

- `./scripts/cargo-headstart test -p rustui-macros docs_only_cfg_attr_stays_on_the_child_expression --lib` passed (1 test).

## Acceptance boundary

This confirms macro expansion for the covered documentation attribute. A
downstream compiler-warning probe was not available, so warning behavior is
not claimed verified. Broader macro and workspace acceptance remains open.
