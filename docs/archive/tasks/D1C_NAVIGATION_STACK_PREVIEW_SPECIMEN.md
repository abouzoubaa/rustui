# M313 — Stage 9 / D1c NavigationStack preview specimen

## Scope

The canonical compiled-preview gallery includes an opt-in specimen built with
the public `NavigationStack`, `NavigationDestination`, and `NavigationLink`
APIs. Its descriptor stores only `selected_id`, restricted to `none` and
`details`. The root link and the single `details` destination are fixed in the
compiled factory; destination content is static `Text`. The schema has no
events, and descriptors have no actions.

The preview maps `none` to an empty navigation path and `details` to the single
route `details`. Writes accept only those two canonical paths and use the
existing generation/revision-stamped feedback bridge. Arbitrary routes,
multi-route paths, views, callbacks, and executable descriptor data are not
serialized. Invalid route imports must be rejected without changing the
mounted snapshot.

## Acceptance

- One focused Playground regression covers the one-property schema and `none`
  default, root-link push to `details`, Back to `none`, rejection of unsupported
  paths, descriptor export/import for both states, and invalid-route import
  preserving the mounted stamp/document.
- Run the focused locked/offline Playground library test and locked/offline
  Playground all-targets check through `./scripts/cargo-headstart`, using an
  isolated target directory under
  `target/headstart/diagnostics/d1c-navigation-stack-m313/`.
- Run Rust formatting and `git diff --check` for the scoped changes.
- Native rendering, push/back interaction, keyboard/focus, accessibility and
  VoiceOver, localization/RTL, visual comparison, and other-platform
  acceptance remain open. Source tests and compilation do not close those
  gates.

## Boundaries

Keep the `details` route and content fixed in the compiled gallery factory. Do
not serialize arbitrary destinations, path arrays, views, callbacks, or
executable behavior. Do not change navigation component APIs, upstream
revisions, vendored sources, or dependencies.

## Validation

The focused locked/offline Playground regression passed 1/1, and the combined
locked/offline RustUI and Playground all-targets check passed. `rustfmt --check`
and `git diff --check` passed. No GUI run was performed; native acceptance
remains open. Evidence: `target/headstart/diagnostics/d1c-navigation-stack-m313/focused-test.log`
and `target/headstart/diagnostics/d1c-navigation-stack-m313/all-targets-check.log`.
## M440 — whitespace-only stack identity

`NavigationStackView::id` rejects an identity containing only spaces and a tab.
The focused `blank_stack_identity_is_rejected` regression passed 1/1
(`target/headstart/evidence/navigation-stack-blank-identity-m440-test.log`).
This source test does not change the existing native acceptance limits above.
