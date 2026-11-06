# M777 — Stage 8 collision-safe Toolbar action identities

Toolbar action button IDs now encode the toolbar and item IDs with explicit
lengths. This prevents distinct pairs such as `("toolbar:a", "b")` and
`("toolbar", "a:b")` from colliding after string concatenation.

The focused collision regression passed (1 passed, 752 filtered). A scoped
rustfmt check still reports pre-existing formatting differences in the dirty
Toolbar file; those unrelated hunks were preserved. Native keyboard and
accessibility acceptance remain open.
