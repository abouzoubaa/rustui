# M1210 — Stage 3 report Window Controls button selection

## 2026-10-24 accessibility source follow-up

The selected-control feedback now uses the existing polite Status live-region
semantics, so Close, Minimize, and Zoom selection changes have an accessible
announcement value as well as visible text. Native accessibility-tree and
VoiceOver acceptance remain open.

The focused regression was attempted with the locked/offline Headstart wrapper.
A narrow stage-1 build of `library/test` and `library/proc_macro` completed, but
the focused check still failed to compile with `E0463: can't find crate for
rustui`. The file-wide rustfmt check also reports existing formatting
differences outside the new live-region lines. No broader test suite or further
toolchain build was run after that failure.

The Window Controls illustration now tracks which Close, Minimize, or Zoom
button was selected and displays that value in the preview. Previously all
three actions shared the same active-state toggle, which was already active by
default and gave no visible response. The illustration continues to state
that clicks do not control native system chrome.

Verification: `./scripts/cargo-headstart test -p rustui-playground
window_control_buttons_reflect_owner_active_state --lib` passed (1 passed, 0
failed; 179 filtered). This is state-test evidence only; no GUI acceptance was
run.

## 2026-10-24 isolated focused rerun

The earlier attempt recorded above did not reach the test assertions because
the shared target contained dependency metadata from a different compiler. The
same focused regression was rerun with a persistent isolated target:

```sh
CARGO_TARGET_DIR=/Users/Abium/Developer/rustui/target/headstart/m1210-live-region-regression ./scripts/cargo-headstart test -p rustui-playground --locked --offline --lib window_control_buttons_reflect_owner_active_state
```

Result: 1 passed, 0 failed; 182 filtered. Cargo rebuilt into the isolated
target, leaving the shared source files and shared target untouched. This is
focused state-test evidence only; native accessibility-tree and VoiceOver
acceptance remain open.

## 2026-10-24 generated-action source regression

A focused regression now dispatches the generated Close, Minimize, and Zoom
owner actions from inactive state and verifies notification, active state, and
the matching visible feedback value:

```sh
CARGO_TARGET_DIR=/Users/Abium/Developer/rustui/target/headstart/m1210-live-region-regression \
  ./scripts/cargo-headstart test -p rustui-playground --locked --offline --lib \
  window_control_generated_actions_update_selection_feedback
```

Result: 1 passed, 0 failed; 186 filtered. This verifies owner-action behavior;
it does not activate the rendered GPUI Button callback. Native accessibility
tree and VoiceOver acceptance remain open.
