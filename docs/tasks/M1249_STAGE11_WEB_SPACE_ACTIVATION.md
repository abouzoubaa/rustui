# M1249 — Stage 11: apply M855 and verify Web Space activation

The user approved M855 v3 on 2026-10-23. The exact patch was versioned under
`patches/approved/`, applied with the preimage/postimage-checking helper, and
selected through the same-version `gpui-pre-web` 0.3.9 local Cargo path override.
The original vendor checksum metadata remains unchanged.

The optimized Playground WebAssembly bundle built successfully. Fresh-browser
acceptance verified canvas `tabindex=0`, keyboard focus on Increment, and Space
activation changing the counter from 1 to 2. Screenshot and structured evidence:
`target/headstart/evidence/m855-stage11-web-space-activation-20261023.png` and
`.json`.

This verifies the focused activation path only. Text entry/IME, screen-reader
output, broader keyboard navigation, navigation/assets, and font/glyph
acceptance remain open.
