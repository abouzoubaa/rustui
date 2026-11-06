# M788 — Reconcile TextEditor after canceled IME preedit

## Scope

When the owner value stayed unchanged during an IME composition, TextEditor preserved local marked text. If composition was canceled and GPUI cleared the marked range, the stale preedit could remain indefinitely because the owner value itself had not changed.

## Change

Track the prior marked-range state through GPUI's `EntityInputHandler`. On a marked-to-unmarked transition, restore the unchanged owner value. Continue applying real owner changes during composition and preserve ordinary local edits when no composition cancellation occurred.

## Evidence

`./scripts/cargo-headstart test --quiet --locked --offline -p rustui --lib text_editor` passed (12 tests). Scoped rustfmt and `git diff --check` passed.

Native IME composition and cross-platform acceptance remain open.
