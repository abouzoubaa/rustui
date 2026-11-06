# L3a: RustUI Lab appearance controls

## Implementation

The Lab now exposes Light and Dark appearance buttons through
`apps/rustui-lab/src/theme_selector.rs`. Both activate an `AppearanceMode` on
the same parsed `themes/macos27.json` `ThemeDocument`; this adds appearance
selection for one theme and does not claim support for other planned theme
packs.

The parsed document is cached in a `OnceLock<Arc<ThemeDocument>>` and shared by
startup activation and both `AppAction` handlers. Each handler reports an
activation error to stderr rather than ignoring it. `activate_theme` updates
the GPUI Components global Theme through `Theme::update`; the vendored
`vendor/gpui-component-0.7.3/src/theme/mod.rs` documents that this write path
refreshes every window. No extra repaint API or framework change was needed.

The current story selection, searchable catalog, canvas bindings, demo actions,
and locale controls remain wired. The controls use the existing bordered-button
style and 8-point spacing.

## Pending manual checks

- Switch Light and Dark repeatedly with each story selected; verify the open
  window refreshes and colors match the selected appearance.
- Verify button hover, pressed, disabled, and focus colors in both appearances.
- Check locale selection, especially Arabic direction, after each appearance
  change.
- Confirm an activation failure is visible in the application's stderr output.

No tests, Cargo commands, builds, or GUI acceptance were run for this change.
The appearance and error-reporting checks remain pending.
