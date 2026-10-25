# M862 — Stage 9 / route the Text Style Guide catalog item

## Contract

Selecting the implemented `text_style_guide` catalog entry opens the existing
Text framework story rather than the pending placeholder.

## Implementation

`story_for` maps `text_style_guide` to the existing `StoryId::Text`; no new
story or component implementation is introduced.

## Evidence

- `./scripts/cargo-headstart test -p rustui-playground --lib text_style_guide_catalog_item_reaches_text_framework_story` passed 1/1.
- `git diff --check -- apps/rustui-playground/src/lib.rs` passed.

This closes one source-level catalog route. Native preview interaction,
accessibility, and broader Stage 9 acceptance remain open.
