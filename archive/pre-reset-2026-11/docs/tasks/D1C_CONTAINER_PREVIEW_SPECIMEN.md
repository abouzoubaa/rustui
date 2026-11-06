# M359 — Stage 9 / D1c Container preview specimen

## Scope

The canonical compiled-preview gallery registers an opt-in
`container-specimen` using the public RustUI `Container` builder. Its schema
exposes a bounded `label` and a `surface` choice restricted to `background`,
`sidebar`, or `panel`. Padding and border presentation remain fixed in the
compiled factory. The schema and mounted action map contain no events or
actions; descriptors cannot encode layout code or callbacks.

## Validation

- The focused gallery test verifies the surface allowlist, empty event/action
  maps, and descriptor import/export for label and surface.
- The gallery test module passed 51/51 tests through
  `./scripts/cargo-headstart test -p rustui-playground --lib
  stories::compiled_preview_gallery::tests --locked --offline`.
- The locked/offline Playground all-targets check passed.
- No GUI acceptance ran. Native appearance, theme, accessibility, and
  cross-platform checks remain open.

Evidence: `target/headstart/evidence/d1c-layout-specimens-m359-tests-20261021.log`
and `target/headstart/evidence/d1c-layout-specimens-m359-check-20261021.log`.

## M727 — expose bounded padding

The inspector now exposes `padding` from 0 through 32 logical pixels, default
12. The factory passes the imported value to the public `Container::padding`
builder. The focused locked/offline descriptor regression verifies the
bounds, default, and import/export round-trip; it passed 1/1. No GUI
acceptance ran; native appearance, theme, accessibility, and cross-platform
checks remain open.
