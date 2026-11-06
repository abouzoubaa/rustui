# M885 — Stage 5 TextField renderer theme combinations

## Change

Theme validation now requires `text_field/body` rules for every renderer-used
appearance and size combination: light/dark × small/large. Previously,
removing one combination still passed schema validation and failed later
during style resolution.

## Focused evidence

- `./scripts/cargo-headstart test -p rustui text_field_theme_requires_each_renderer_appearance_and_size_rule` passed (1 test).
- Scoped `git diff --check` passed.

## Acceptance boundary

This closes one required-rule coverage gap. Broader theme visual and
accessibility acceptance remains open.
