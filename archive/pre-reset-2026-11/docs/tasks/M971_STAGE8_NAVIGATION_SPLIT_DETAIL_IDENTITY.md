# M971 — NavigationSplit detail semantics identity

Stage 8 / navigation accessibility identity.

Each NavigationSplit detail-region semantics identity now includes both the
owning split's identity and its selected route. The fields are length-prefixed
so different owner/route pairs cannot collide through delimiter placement.

The focused identity regression passed 1/1 and the scoped whitespace check
passed. Native accessibility/VoiceOver, visible focus, breakpoint, RTL, and
cross-platform acceptance remain open. Divider keyboard resizing and divider
accessibility remain constrained by the pinned NavigationSplit public API.
