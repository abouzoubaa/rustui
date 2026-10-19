# M56: Focus-scoped keyboard shortcuts

## Contract

`KeyboardShortcuts(id, child)` attaches single-stroke command/control shortcuts
to the focused descendant path of `child`. It does not register application-wide
keys, request focus, or add a tab stop. The ID must be nonempty and unique among
sibling wrappers. Definitions use GPUI's `Keystroke::parse` and matching rules;
whitespace sequences, character-simulation syntax, duplicate normalized chords,
and chords without command or control are rejected. `secondary` resolves to the
platform's command or control modifier.

`.shortcut(chord, action)` and `.shortcut_enabled(chord, action, enabled)` accept
RustUI component `Action` or `AppAction`. Invalid definitions panic at view
construction; `try_shortcut_enabled` returns `KeyboardShortcutError`. An
`AppAction` that requires a `ClickEvent` cannot be installed. `AppAction::new`,
owner actions, and compositions of those actions are keyboard compatible.

GPUI dispatches keymap actions before the wrapper's bubble listener. A handled
native binding therefore wins. The wrapper ignores held repeats, events marked
`prefer_character_input`, and incomplete IME keystrokes. It stops propagation
only after matching and running an enabled shortcut. It does not claim to
replace native editor shortcuts or handle multi-stroke sequences.

## Validation

The M56 unit suite passed as part of 167 passing tests, zero failures across 12
binaries in `target/headstart/milestone56-57-tests.log`. The M56/M57 Headstart
build passed in `target/headstart/milestone56-57-build.log`; the reuse report
confirms all three pinned dependencies were fresh in
`target/headstart/milestone56-57-reuse-report.json`.

Bounded native Playground checks are recorded in
`target/headstart/milestone56-keyboard-gui.json`. With focus inside the scope,
Cmd-Shift-K and Cmd-Shift-J activated their counters; disabling the optional
shortcut suppressed its action while typing continued. The outside field accepted
ordinary typing without triggering either scoped shortcut. Native Cmd-Z edited
the text without changing shortcut counters. The reviewed M56/M57 bundle SHA-256
is `ede2a1525425256ab1d665c44165ebe50148bf0cf8fc62f7dcc03dc44f03c322`.

Acceptance is bounded: no competing built-in binding was defined, so dispatch
precedence conflicts were not tested. AppAction execution, nested scopes, repeat
suppression, real IME, sibling retained-state isolation, and full platform keyboard
and screen-reader behavior remain unverified. This is a scoped single-chord API,
not an application menu keymap.
