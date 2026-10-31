# M863 — Stage 9 / route the Menu Item catalog entry

## Contract

Selecting the implemented `menu_item` catalog entry opens the existing Menu
gallery rather than the pending placeholder.

## Implementation

`catalog_demo("menu_item")` now resolves to `menu_gallery::demo()`, which
shows enabled and disabled menu actions. No new menu implementation or story
was introduced.

## Evidence

- `./scripts/cargo-headstart test -p rustui-playground --lib menu_item_catalog_entry_reaches_menu` passed 1/1.
- `git diff --check -- apps/rustui-playground/src/stories/mod.rs` passed.

This closes one source-level catalog route. Native interaction,
accessibility, and broader Stage 9 acceptance remain open.
