# M803 — Stage 5 Checkbox and Radio renderer requirements

Theme validation rejects Checkbox style rules missing `background` or
`foreground`, and Radio style rules missing either color role or the renderer's
`width`, `height`, and `corner_radius` geometry. These fields are indexed by
the renderers and must be rejected before rendering rather than causing a
panic.

Focused regressions cover missing Checkbox `background` and Radio
`corner_radius`. The command
`./scripts/cargo-headstart test -p rustui theme_requires_renderer --lib --locked --offline`
exited successfully. Native visual and accessibility acceptance remain open.
