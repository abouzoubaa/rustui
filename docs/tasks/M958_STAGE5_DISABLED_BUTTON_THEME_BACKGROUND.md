# M958 — Stage 5 / use the resolved disabled Button colors

The shared Button renderer now selects `disabled_background` and
`disabled_foreground` from the resolved theme for disabled Automatic and
Bordered buttons. The focused regression distinguishes disabled from enabled
background selection and passed 1/1. No vendored source changed. Native visual
and cross-platform theme acceptance remain open.
