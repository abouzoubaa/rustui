# M801 Stage 8 Sidebar arrows from a disabled selected row

When the current selection points at a row that has since become disabled, Sidebar Up/Down navigation must use that row's current position as the search origin and skip disabled rows in the requested direction. The selection lookup now finds the row by ID without requiring it to be enabled; the existing directional search then selects the nearest enabled row.

Focused verification passed:

```sh
./scripts/cargo-headstart test -p rustui arrows_from_a_disabled_selection_move_to_the_nearest_enabled_row --lib
```

This closes the source-level keyboard-navigation defect. Native focus and screen-reader acceptance remain separate gates.
