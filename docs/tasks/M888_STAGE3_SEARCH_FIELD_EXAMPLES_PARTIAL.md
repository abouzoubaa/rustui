# M888 — Stage 3 Search Field examples partial coverage

## Change

The `search_field_examples` catalog row now explains that it routes to the
shared Text inputs story. It marks the coverage Partial and identifies the
six Dark Search Field reference combinations that the story does not
reproduce.

## Focused evidence

- `./scripts/cargo-headstart test -p rustui-playground search_field_examples_routes_to_input_story_with_reference_gaps --lib` passed (1 test).

## Acceptance boundary

At M888, the story did not implement the six named Dark Search Field
combinations. M910 adds those state/text combinations to the canonical Input
gallery. Native dark appearance, focus visuals, and visual/accessibility
acceptance remain open; see `M910_STAGE3_DARK_SEARCH_FIELD_MATRIX.md`.
