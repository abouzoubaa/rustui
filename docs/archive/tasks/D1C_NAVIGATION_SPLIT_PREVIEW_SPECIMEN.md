# M312 — Stage 9 / D1c NavigationSplitView preview specimen

## Scope

The canonical compiled-preview gallery includes an opt-in specimen built with
the public `NavigationSplitView` and `NavigationSplitItem` APIs. Its descriptor
stores only `selected_id`, restricted to `none`, `files`, or `settings`.
Navigation labels and detail content are fixed in the compiled factory; the
Files and Settings details are static `Text` views. The schema has no events,
and descriptors have no actions.

Selection uses the existing generation/revision-stamped string binding path.
Invalid selection imports must be rejected without changing the mounted
snapshot. The specimen uses a named navigation outline and a fixed viewport
because NavigationSplitView requires bounded width and height.

## Acceptance

- One focused Playground regression covers the one-property schema and `none`
  default, accepted selection feedback and revision advancement, normalized
  descriptor export/import, and invalid-selection rejection preserving the
  mounted stamp/document.
- Run the focused locked/offline Playground library test and locked/offline
  Playground all-targets check through `./scripts/cargo-headstart`, using an
  isolated target directory under `target/headstart/diagnostics/`.
- Run Rust formatting and `git diff --check` for the scoped changes.
- Native rendering, selection, keyboard/focus, accessibility/VoiceOver,
  localization/RTL, visual comparison, and other-platform acceptance remain
  open. Source tests and compilation do not close those gates.

## Boundaries

Keep item IDs fixed and unique, and keep their detail content compiled into the
gallery factory. Do not serialize arbitrary destinations, views, callbacks, or
executable behavior. Do not change upstream revisions or vendored sources.

## Validation

The focused regression and isolated Playground all-targets check passed. Their
logs are recorded at:

- `target/headstart/diagnostics/d1c-navigation-split-m312-focused-test.log`
- `target/headstart/diagnostics/d1c-navigation-split-m312-all-targets-check.log`

No GUI run was performed; native acceptance remains open.
