# M825 — Stage 10 P3D screenshot identity for incomplete captures

The P3D evidence validator now checks screenshot file identity for screenshots
attached to `failed` and `inconclusive` states, as it already does for captured
states. Reusing one file for multiple states is rejected even when the captures
are incomplete. The focused regression assigns the same screenshot to failed
and inconclusive states and verifies rejection.

The focused regression passed. This is evidence-record structure validation; it
does not inspect pixels or establish native visual, interaction, accessibility,
or performance acceptance. Those P3D gates remain open.
