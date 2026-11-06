# C8: Empty and Error states

## API

- `components::feedback::empty_state::{EmptyState, EmptyStateView}` creates an
  empty-content message from a title.
- `components::feedback::error_state::{ErrorState, ErrorStateView}` creates an
  error alert from a title.
- Both support `.detail(text)`, `.id(identity)`, and
  `.accessibility_label(text)`. The accessible name defaults to the visible
  title. The stable fallback identity includes title and optional detail; use
  explicit distinct IDs when repeated states with equal text can appear in one
  view tree.
- `EmptyState` exposes a Status role; `ErrorState` exposes an Alert role.
  Titles use the current foreground theme color and semibold text. Supporting
  detail uses the muted foreground color; error title uses the theme danger
  color.

These are presentation components only. Callers compose retry or other actions
with an ordinary RustUI `Button`; the state components do not own async state or
action dispatch.

## Verification boundary

Focused unit tests cover title defaults, optional detail, explicit identity,
accessible-name overrides, and fallback identity differences. The M22 native
Review showed the empty-state title and detail, with AX name “Saved items are
empty” and Help “Saved items will appear here.” The error state showed its red
title and detail, with AX name “Item loading failed” and Help “Check your
connection and retry.” A retry action count changed from 0 to 1.

The final unit/integration run passed 77 tests across 12 binaries; see
`target/headstart/milestone21-22-tests-final.log`. Native inspection records
the observed accessibility attributes and pixels only. VoiceOver announcement
and announcement cadence were not verified.
