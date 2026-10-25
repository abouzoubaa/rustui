# M901 — Stage 5 / require TreeView theme rules for both appearances

The TreeView renderer resolves `outline_view/outline_view`, but theme
validation did not require a rule for either appearance. Validation now
requires the part to resolve for both light and dark appearances. A focused
regression removes the dark rule and confirms rejection.

Focused check passed 1/1:

```text
./scripts/cargo-headstart test -p rustui outline_view_theme_requires_renderer_style_for_both_appearances -- --exact theme::schema::tests::outline_view_theme_requires_renderer_style_for_both_appearances
```

This closes a schema/runtime lookup gap only. Native visual, accessibility,
and cross-platform theme acceptance remain open.
