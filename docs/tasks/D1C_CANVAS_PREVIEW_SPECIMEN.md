# M320 — Stage 9 / D1c Canvas preview specimen

## Scope

Add an opt-in compiled-preview specimen for the public RustUI `Canvas` API. The
descriptor exposes one `composition` string restricted to `basic` or
`overlap`. Both choices build fixed vector geometry inside the compiled factory;
the descriptor cannot provide drawing commands, coordinates, colors, callbacks,
or executable behavior. The schema has no events and mounted descriptors have
no actions.

## Acceptance

- One focused Playground regression checks the one-property schema and default,
  fixed allowed values, empty event/action maps, accepted feedback with revision
  advancement, descriptor export/import, and rejection of an arbitrary
  composition without mutating the mounted snapshot.
- Run the focused locked/offline Playground library test and locked/offline
  Playground all-targets check through `./scripts/cargo-headstart`, using the
  isolated target directory `target/headstart/diagnostics/d1c-canvas-m319/`.
  This directory name was reserved before the specimen was assigned M320.
- Run Rust formatting and scoped `git diff --check`.
- No GUI test is part of this source-only slice. Native drawing, accessibility,
  visual comparison, and other-platform acceptance remain open.

## Validation

- Focused regression passed 1/1.
- Locked/offline Playground all-targets check passed.
- Rust formatting and scoped `git diff --check` passed.
- No GUI run was performed; native and broader acceptance remain open.

Evidence:
`target/headstart/diagnostics/d1c-canvas-m319/focused-test.log` and
`target/headstart/diagnostics/d1c-canvas-m319/all-targets-check.log`.

## Boundaries

Keep both drawing compositions fixed in the compiled factory. Do not serialize
arbitrary scenes, geometry, colors, callbacks, or executable behavior. Do not
change Canvas APIs, upstream revisions, vendored sources, shared roadmap,
completion gates, file registry, or dependencies.
