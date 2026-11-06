# M887 — Stage 3 Form Accessories partial catalog route

## Change

The catalog now marks `form_accessories` as Partial and routes it to the
existing Input story, which demonstrates related field, selection, and value
controls. The catalog caveat states that there is no dedicated FormAccessory
specimen.

## Focused evidence

- `./scripts/cargo-headstart test -p rustui-playground form_accessories_catalog_item_reaches_partial_input_story` passed (1 test, 129 filtered).
- Scoped `git diff --check` passed.

## Acceptance boundary

This provides related controls in the Playground; it does not claim a dedicated
FormAccessory specimen, native visual behavior, or accessibility acceptance.
