# M1009 — ignore IME candidate keys in Sidebar navigation

Sidebar's outer tree key handler already skipped character-preferred and
modified keystrokes, but it did not skip IME composition. Arrow keys used to
move through IME candidates could therefore trigger Sidebar selection. Add an
IME eligibility check and regression coverage for focused, composing,
character-preferred, modified, and unfocused events.

Focused test passed 1/1:

```text
./scripts/cargo-headstart test -p rustui sidebar_keyboard_navigation_ignores_ime_candidate_keys --lib
```

Native IME and assistive technology acceptance remain open.
