# M883 — Stage 3 menu separator catalog route

## Change

The Playground catalog now identifies `MenuItem::separator()` as a partial
implementation and routes `menu_separator` to the existing Menu gallery. The
catalog no longer reports the component as missing when the API and a usable
demo already exist.

## Focused evidence

- `./scripts/cargo-headstart test -p rustui-playground menu_separator_catalog_entry_reaches_menu_demo` passed (1 test, 127 filtered).

## Acceptance boundary

This routes the existing separator API into the Playground. It does not claim
broader native menu behavior or accessibility acceptance.
