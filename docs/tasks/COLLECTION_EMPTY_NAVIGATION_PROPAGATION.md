# Stage 8 — propagate collection navigation with no rows

`CollectionState::keyboard` stops propagation whenever `next_selection` returns
`Some`. For an empty ID list, Up, Down, Home, and End previously returned
`Some(None)`, which cleared owner selection and consumed the key despite there
being no row to navigate to. `next_selection` now leaves those keys unhandled;
Escape still clears selection when requested.

The focused regression passed 1/1:

```text
./scripts/cargo-headstart test -p rustui --lib components::data::collection::tests::keyboard_decision_helpers_cover_navigation_escape_confirmation_and_ignored_input --locked --offline
```

Scoped formatting and `git diff --check` passed. Native keyboard propagation
and accessibility acceptance remain open.
