# M991 — Stage 10 / enforce portable P3D artifact paths

P3D artifact validation now rejects Windows drive-qualified paths, backslash
paths, and dot/traversal segments under its portable relative-path contract.

Focused check: malformed drive and backslash paths were rejected by the new
regression (1/1).

No candidate was run. Review, approval, and promotion remain open.
