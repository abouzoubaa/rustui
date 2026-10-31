# M770 — Stage 8 / give implicit NavigationStacks distinct retained identities

An implicit `NavigationStack` now allocates its base identity through the
render context's retained-identity slot, matching the existing List, Tree,
TabView, Button, and Slider pattern. Sibling stacks no longer share the static
`navigation-stack` retention scope, while an explicitly supplied `.id(...)`
continues to bypass implicit allocation.

The focused regression verifies sibling separation and stable identity at the
same slot. Its locked/offline test passed (1/1); scoped formatting and
`git diff --check` passed. Native navigation interaction and cross-platform
acceptance remain open.
