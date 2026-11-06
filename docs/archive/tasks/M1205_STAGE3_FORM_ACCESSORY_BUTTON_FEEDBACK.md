# M1205 — Stage 3 make Form Accessories buttons report activation

The Form Accessories Button, Detail, and Detail Info controls now update the
story's owner state and show the last activated action in the preview. They no
longer use empty callbacks.

The focused state regression covers each button.

Verification: `./scripts/cargo-headstart test -p rustui-playground
form_accessory_buttons_report_the_activated_reference_action --lib` passed
(1 passed, 0 failed; 178 filtered). Formatting passed for the story file.
This is source/test evidence, not GUI acceptance.

## 2026-10-24 accessibility source follow-up

The last-activated feedback now uses the existing polite Status live-region
semantics, so Button, Detail, and Detail Info activation updates have an
accessible announcement value as well as visible text. Native accessibility-
tree and VoiceOver acceptance remain open.

At the time of this source follow-up, the focused regression was attempted, but
Cargo stopped before compiling the story with `E0463: can't find crate for
rustui` in the current Headstart sysroot. That run did not verify the source
change.

## 2026-10-24 focused initial-status regression

The existing action-feedback regression now also checks the initial feedback
string, `Last activated: None`, before the Button, Detail, and Detail Info
transitions. The story passes this value to its existing polite Status
live-region builder. The focused check ran in a persistent isolated Headstart
target:

```sh
CARGO_TARGET_DIR=/Users/Abium/Developer/rustui/target/headstart/m1205-live-region-initial-status ./scripts/cargo-headstart test -p rustui-playground --locked --offline --lib form_accessory_buttons_report_the_activated_reference_action
```

Result: 1 passed, 0 failed; 182 filtered. This is source-level feedback
string coverage only; native accessibility-tree and VoiceOver acceptance
remain open.
