# M1138 — preserve ColorPicker preview channels

## Change

When a ColorPicker preview's `has_color` property is cleared, its serialized
HSLA channels remain in the descriptor. Re-enabling the color now reads those
stored channels from the mounted snapshot instead of replacing them with
defaults.

## Focused evidence

```text
./scripts/cargo-headstart test -p rustui-playground color_picker_preview_uses_bounded_channels_and_round_trips
1 passed
```

The regression covers clearing, descriptor export/import, re-enabling the
color, and restoration of the original custom channels. Broader UI Studio
composition and native acceptance remain open.
