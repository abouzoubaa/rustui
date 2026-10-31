# M855 — Stage 11 Web keyboard focus patch, v3

The user approved applying v3 on 2026-10-23. The current checker targets
`patches/proposed/gpui-pre-web-keyboard-focus-activation-v3.patch`; v2 is
historical. The v3 patch makes the canvas tabbable, routes keydown/keyup through
the browser window, and accepts only events targeted at the GPUI canvas or its
hidden IME mirror. Text composition remains attached to the IME mirror.

The exact patch is now versioned under `patches/approved/`, applied with the
hash-checking helper, and selected through the same-version `gpui-pre-web`
Cargo path override. The original vendor checksum metadata remains unchanged.
The optimized Playground WebAssembly bundle built successfully. In a fresh
browser session, the canvas had `tabindex=0`, keyboard focus reached Increment,
and Space changed the counter from 1 to 2. Screenshot and structured evidence:
`target/headstart/evidence/m855-stage11-web-space-activation-20261023.png` and
`.json`.

The focused Space-activation path is verified. Text entry/IME, screen-reader
output, broader keyboard navigation, and the remaining Stage 11 browser
acceptance are still open. See `docs/tasks/ROADMAP_REMAINING_AUDIT.md`.
