# M1208 — Stage 3 report Offset reference-action activation

The Offset story's visible “Reference action” button now updates owner state
and changes its label to “Reference action · selected” after activation. The
fixed layout slot and initial label remain intact.

Verification: `./scripts/cargo-headstart test -p rustui-playground --lib
reference_action_reports_activation_in_its_owner_state` passed (1 passed, 0
failed; 179 filtered). This is state-test evidence only; no GUI acceptance was
run.

## 2026-10-24 accessibility source audit

The story passes its changing visible label directly to `Button`. RustUI
Buttons use that visible label as the accessible-name fallback and forward the
resolved name to GPUI, so the selected wording is present in the source
accessibility contract without an additional status region. This does not
verify the native accessibility tree or VoiceOver announcement after a click.
