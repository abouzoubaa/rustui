# M1181 — Stage 5 / choose contrasting accent-button foreground

The theme adapter supplied `primary_text` as the foreground for filled accent
buttons. In the macOS 27 light and dark profiles, this fell below the
adapter's black/white contrast choice for the accent surface.

`ButtonPalette::parse` now uses the existing `contrasting_foreground(accent)`
helper for `accent_foreground`. The focused regression resolves both
appearances and checks the palette mapping; it passed 1/1:

```sh
./scripts/cargo-headstart test -p rustui --lib adapters::themes::tests::accent_foreground_uses_contrasting_color_in_both_appearances
```

This is source-level palette evidence. Native visual and accessibility
acceptance remain open.
