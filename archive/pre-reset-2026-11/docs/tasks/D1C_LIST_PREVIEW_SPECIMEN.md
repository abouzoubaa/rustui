# M307 — Stage 9 / D1c public List preview specimen

## Scope

The compiled-preview gallery exposes an opt-in specimen built with the public
RustUI `List` API. Its fixture is fixed to three rows: First item, Second item,
and Third item. The serializable descriptor stores only `selected_id`, bounded
to `none`, `row-0`, `row-1`, or `row-2`; row content remains compiled into the
gallery factory. There are no events or actions.

Selection uses the existing generation/revision-stamped property feedback
bridge. Invalid and stale selection requests are rejected without changing the
mounted snapshot. This is a Playground preview fixture, not a complete List
acceptance claim.

## Verification

- Focused specimen regression passed **1/1**. It covers schema and default
selection, row fixture, stamped selection updates, stale/invalid input
rejection, and descriptor round-trip. The first run exposed three unrelated
generic default properties being added during mount; the List-specific mount
branch now initializes only the declared selection property. The failed
diagnostic is preserved at
`target/headstart/evidence/d1c-list-preview-focused-test.log`; the passing
rerun is at
`target/headstart/evidence/d1c-list-preview-focused-test-followup.log`.
- The locked/offline Playground all-targets check passed:
  `target/headstart/evidence/d1c-list-preview-playground-all-targets-check.log`.
- Scoped whitespace check passed for
  `apps/rustui-playground/src/stories/compiled_preview_gallery.rs`.

Native rendering and selection, keyboard/focus, accessibility/VoiceOver,
localization/RTL, visual comparison, and other-platform acceptance remain
open.

## M310 — Native observation

On macOS 27, the compiled List specimen visibly rendered First item, Second
item, and Third item. Its accessibility tree reported a `list` with three
selectable row containers. Clicking Second item changed `selected_id` from
`none` to `row-1`; the preview stamp advanced from generation 3/revision 0 to
generation 3/revision 1, and the accessibility tree reported Second item as
selected. This is partial native acceptance. VoiceOver, keyboard/focus,
localization/RTL, visual comparison, and other-platform acceptance remain
open. Evidence: `target/headstart/evidence/m310-list-native-observation-20261020.json`
and `target/headstart/evidence/m309-preview-controls-native-build.log`.
