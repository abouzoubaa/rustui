# M359 — Stage 9 / D1c SplitView preview specimen

## Scope

The canonical compiled-preview gallery registers an opt-in
`split-view-specimen` using the public RustUI `SplitView` and `SplitPanel`
builders. Its schema exposes `horizontal` or `vertical` orientation and two
bounded panel labels. Panel IDs, size ranges, and initial sizes are compiled
locally. The schema and mounted action map contain no events or actions, and
descriptors cannot supply views or callbacks.

## Validation

- The focused gallery test verifies the orientation allowlist, empty
  event/action maps, and descriptor import/export for orientation and both
  panel labels.
- The gallery test module passed 51/51 tests through
  `./scripts/cargo-headstart test -p rustui-playground --lib
  stories::compiled_preview_gallery::tests --locked --offline`.
- The locked/offline Playground all-targets check passed.
- No GUI acceptance ran. Native drag/keyboard resizing, focus, screen-reader,
  appearance, and cross-platform checks remain open.

Evidence: `target/headstart/evidence/d1c-layout-specimens-m359-tests-20261021.log`
and `target/headstart/evidence/d1c-layout-specimens-m359-check-20261021.log`.

## M721 — Stage 9 inspector initial-size control

The SplitView specimen now exposes `first_size` in the preview inspector, using
the public `SplitPanel::initial_size` builder. The value defaults to 180 and is bounded to the existing panel range of 120–360. The focused
descriptor regression verifies the customized value survives import and export;
native rendering and drag acceptance remain open.
