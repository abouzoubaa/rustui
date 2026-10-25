# M83 — Collection keyboard decision tests

## Scope

Add a focused unit test for the existing pure keyboard-decision helpers used by `Collection`. The test checks arrow/Home/End selection boundaries, Escape clearing, Enter confirmation gating, and ignored character-like or unknown keys.

This milestone adds regression coverage only; it does not change keyboard runtime behavior.

## Validation

- `./scripts/cargo-headstart test -p rustui keyboard_decision_helpers_cover_navigation_escape_confirmation_and_ignored_input --lib`: 1 passed.
- `./scripts/cargo-headstart check -p rustui --all-targets`: passed.
- Logs: `target/headstart/milestone83-collection-keyboard-tests.log` and `target/headstart/milestone83-collection-keyboard-check.log`.

The check reports existing warnings in benchmark examples and the pinned `block` dependency's future-incompatibility notice.

## Remaining acceptance

The test calls pure helpers. It does not dispatch a native key event or verify live focus, modifier filtering, IME/character-input filtering, accessibility-tree behavior, screen-reader announcements, or platform-specific behavior. The runtime handler already guards focus, modifiers, `prefer_character_input`, and active IME input; those event-path behaviors remain unverified in a running Playground.

## M84 — bounded native keyboard check

The Playground's existing Collection catalog demo was used without adding a second story. Native interaction selected `record-0001` after clicking `record-0000` and pressing Down; Up/Home/End navigation worked; Escape cleared selection; Return without a selection left the confirmation count at 0; Return with `record-0000` selected incremented it to 1. The native accessibility tree exposed the list as `Collection gallery records` and rows as selectable items with names such as `Record 0000`.

Evidence: `target/headstart/milestone84-collection-keyboard-gui.json`; binary SHA-256 is recorded there. The app was quit and its app-specific build outputs were removed after the check. Screen-reader speech, IME event filtering, modified shortcuts, cross-platform behavior, and performance remain unverified.

## M610 — Collection keyboard event eligibility

Extracted the existing key-event guard into `keyboard_event_eligible` and kept
the handler behavior unchanged. The focused regression covers the plain
focused path and rejects unfocused input, character-preferred input, active
IME, and Control, platform, Alt, or Shift modifiers.

Validation: the Collection test group passed 10/10 and
`./scripts/cargo-headstart check -p rustui --all-targets` passed with existing
example warnings and the pinned `block` future-incompatibility notice. Logs:
`target/headstart/evidence/m610-collection-keyboard-filter-tests-20261021.log`
and `target/headstart/evidence/m610-collection-keyboard-filter-check-20261021.log`.

This proves the pure guard decisions and their use in the handler; it does not
prove native IME delivery, modified-key dispatch, screen-reader behavior, or
cross-platform acceptance.
