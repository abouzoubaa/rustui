# M1187 — Stage 3 stateful window-chrome preview

## Scope

Replace the static `window_chrome` title/content placeholders and unrelated
Toolbar review demo with a Playground-only owner-state composition. Its public
RustUI Button creates a sample document or closes it; both actions update the
visible title and body. The catalog now describes those behaviors.

The story explicitly identifies itself as illustrative. It does not create a
native titlebar, a system window, or system window actions. No public framework
API or native adapter was added.

## Verification

`./scripts/cargo-headstart test -p rustui-playground --lib stories::window_chrome_gallery::tests::document_actions_update_the_preview_title_and_body --locked --offline`
passed 1/1. The new module passed rustfmt, and scoped whitespace checks
passed.

Native visual, keyboard, accessibility, localization, and platform acceptance
remain open.

## 2026-10-24 accessibility source follow-up

The current document title and body are now exposed together through the
existing polite Status live-region semantics, so Create and Close transitions
have an announcement value. Native accessibility-tree and VoiceOver acceptance
remain open. At the time of this source follow-up, the focused state regression
was not rerun because the current Headstart wrapper stopped before compiling
the Playground crate with `E0463: can't find crate for rustui`; see the M1205
follow-up for that toolchain limitation.

## 2026-10-24 focused status-payload regression

The existing document-action regression now checks that the status value
contains the current title and body for the initial, closed, and newly created
document states. The focused check ran in a persistent isolated Headstart
target:

```sh
CARGO_TARGET_DIR=/Users/Abium/Developer/rustui/target/headstart/m1187-window-chrome-status-regression ./scripts/cargo-headstart test -p rustui-playground --locked --offline --lib stories::window_chrome_gallery::tests::document_actions_update_the_preview_title_and_body
```

Result: 1 passed, 0 failed; 182 filtered. This verifies the story's
source-level status payload only. Native accessibility-tree and VoiceOver
acceptance remain open.
