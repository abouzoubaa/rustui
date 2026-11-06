# M965 — Disabled Link theme color

Stage 5 / component theme resolution.

Disabled Link controls now use the active platform theme's
`disabled_foreground`. Enabled Links continue to use the foreground resolved
from the Link component style.

The focused regression passed 1/1. Rust formatting and scoped whitespace
checks passed. Native visual and cross-platform theme acceptance remain open.
