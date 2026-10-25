# M315 — Stage 9 / D1c Collection preview specimen

## Scope

The canonical compiled-preview gallery includes an opt-in specimen of the
public `Collection` and `CollectionItem` APIs. The factory owns a fixed set of
three stable item IDs and labels. The descriptor stores only `selected_id`,
restricted to `none`, `first`, `second`, or `third`. The schema has no events,
and descriptors have no actions.

Selection maps the fixed IDs to the public `Binding<Option<Identity>>` API and
uses the gallery's generation/revision-stamped property feedback. Unknown IDs
are rejected. Item content, IDs, labels, and callbacks remain compiled into the
factory; the descriptor cannot define rows or executable behavior.

## Acceptance

- One focused Playground regression covers the one-property schema and `none`
  default, the fixed item fixture, accepted selection feedback and revision
  advancement, rejection of unknown item IDs without snapshot mutation,
  descriptor export/import, and invalid-selection import preserving the mounted
  stamp/document.
- Run the focused locked/offline Playground library test and locked/offline
  Playground all-targets check through `./scripts/cargo-headstart`, using an
  isolated target directory under
  `target/headstart/diagnostics/d1c-collection-m314/`. This directory name was
  reserved by the live validation before the milestone number was corrected;
  it is still isolated and the evidence paths below identify the M315 work.
- Run Rust formatting and `git diff --check` for the scoped changes.
- Native rendering, selection interaction, keyboard/focus, accessibility and
  VoiceOver, localization/RTL, visual comparison, and other-platform
  acceptance remain open. Source tests and compilation do not close those
  gates.

## Boundaries

Keep item IDs, labels, row values, and factory behavior fixed in the compiled
gallery. Do not serialize arbitrary items, views, callbacks, or executable
behavior. Do not change Collection APIs, upstream revisions, vendored sources,
or dependencies.

## Validation

The focused regression passed 1/1, and the isolated locked/offline Playground
all-targets check passed. Rust formatting and `git diff --check` passed. Logs:

- `target/headstart/diagnostics/d1c-collection-m314/focused-test.log`
- `target/headstart/diagnostics/d1c-collection-m314/all-targets-check.log`

The directory retains its original `m314` name because it was reserved by the
live isolated build before this specimen was assigned M315. No GUI run was
performed; native acceptance remains open.
