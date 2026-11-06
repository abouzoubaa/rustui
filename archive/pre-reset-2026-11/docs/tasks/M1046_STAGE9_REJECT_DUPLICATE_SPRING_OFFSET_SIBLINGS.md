# M1046 — Stage 9 / reject duplicate SpringOffset sibling identities

`SpringOffset` uses its identity to name both GPUI spring animations, but its
retention scope was missing from `RetainedEntities`' duplicate sibling identity
guard. Two sibling offsets with the same ID could therefore share animation
state. The scope is now included in the existing guard, matching
`SpringOpacity` and other identity-bearing scopes.

The focused `duplicate_spring_offset_siblings_are_rejected` regression passed
1/1 with the locked offline Headstart test command. Native animation timing,
retargeting, and broader lifecycle acceptance remain open.
