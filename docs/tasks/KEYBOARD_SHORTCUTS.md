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
precedence conflicts were not tested. Nested scopes, repeat suppression, real IME,
sibling retained-state isolation, and full platform keyboard and screen-reader
behavior remain unverified. This is a scoped single-chord API, not an application
menu keymap.

## M56 AppAction follow-up

The Playground keyboard gallery includes Secondary-Shift-A. It increments
the visible primary counter and composes the owner `AppAction` with the public
`AppAction::refresh_windows()`. The isolated locked/offline Playground library
suite passed 41/41, including compilation of this scenario. In the bounded native
check, one press with focus inside the scope produced `Primary hits: 1 · Optional
hits: 0`. Typing continued (`inside` became `insidex`). From the outside field,
typing `z` worked and Secondary-Shift-A left the primary count at 1. The check
observed the owner update; it did not independently measure the refresh call.
Evidence: `target/headstart/evidence/roadmap-validation-m251-playground-lib.log`
and `target/headstart/evidence/m253-keyboard-appaction-native-20261020.json`.

The locked/offline Playground build passed with
`CARGO_TARGET_DIR=target/headstart/roadmap-validation-20261020` (exit 0). The
disposable `dev.rustui.playground.validation` process, bundle, and standalone
executable were removed after the check. Existing `dev.rustui.playground` PID
20036 was observed and left untouched. No screenshot or accessibility capture
file was retained. Other M56 limits above remain open.
