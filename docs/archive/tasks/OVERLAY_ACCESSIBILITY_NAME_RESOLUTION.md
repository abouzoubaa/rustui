# Dialog and Sheet accessible-name resolution

Dialog and Sheet literal accessibility-label overrides reject whitespace-only
values. At render time, both components now resolve the first nonblank name
from the explicit accessibility label, title, and trigger, so a blank localized
message key or blank title cannot erase a usable fallback. If every candidate
resolves blank, both components use their validated identity as the accessible
name rather than panicking or emitting an empty name.

Focused component tests cover blank-candidate fallback and identity fallback
when all resolved candidates are blank. Dialog and Sheet also reject blank
localized title keys, matching their existing trigger and accessible-name key
validation. Both reject whitespace-only literal titles, so a blank heading
cannot be accepted as a visible title.
This is source-level validation; native host and screen-reader acceptance remain
open.

## M454 — localized title-key validation (2026-10-21)

Focused locked/offline Dialog and Sheet regressions passed 2/2 in
`target/headstart/evidence/overlay-localized-title-key-validation-20261021.log`.
Rust 1.97.1 formatting passed. Native Accessibility Inspector and VoiceOver
acceptance across open, focus, dismissal, and trigger return remain open.
