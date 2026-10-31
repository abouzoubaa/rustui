# M1191 — Stage 5 custom profile font family: native acceptance run sheet

## Status

**Preparation complete; native acceptance not run.** The adapter maps custom
profile families through unchanged (`Avenir Next` remains `Avenir Next`) and
maps `SF Pro`/`System` to `.SystemUIFont` on native. Existing evidence is a
focused source regression only; native visual and cross-platform acceptance
remain open. See `M1191_STAGE5_CUSTOM_PROFILE_FONT_FAMILY.md`.

## Preconditions

- Wait for Stage 6 to release the shared Playground validation slot and for the
  protected Playground processes to exit. Confirm the Mac is unlocked before
  any future GUI run. Recheck process/slot status immediately before launch.
- Reuse the existing single disposable Playground validation app and bundle
  identifier. Do not build or launch a second app for this check.
- Confirm `Avenir Next` is available to the native font system. If it is not,
  stop and record the missing-family limitation; do not accept a silent
  fallback as a pass.
- Keep `themes/macos27.json` unchanged. Make a disposable copy under
  `target/headstart/verification/` and use only that copy for the profile
  variation.

## Setup

1. Copy `themes/macos27.json` to
   `target/headstart/verification/m1191-custom-font-profile.json`.
2. Set `RUSTUI_THEME_RELOAD_PATH` to the absolute path of that disposable copy
   before launching the existing Playground. The Divider story contains the
   Foundation Additions theme-file controls; this control reads the environment
   path and can reload Light or Dark from that file.
3. Keep every other theme field identical. Use `SF Pro` for the baseline, then
   change only `typography.font_family` to `Avenir Next` for the custom-family
   observation. Do not edit the repository theme pack.
4. In the Playground, select the `divider` catalog story. Confirm the theme
   reload panel reports the file path and successful activation for each
   appearance before recording the sample.

## Checks and evidence

| Check | Action | Pass evidence |
|---|---|---|
| Native baseline | With `SF Pro`, activate Light and Dark from the reload panel. Capture the same visible Playground text in each mode. | Record selected mode, reload status, screenshot path, and baseline profile hash. |
| Custom family | Change only the disposable profile family to `Avenir Next`; allow auto-reload or request the panel reload. Repeat Light and Dark. | Record selected mode, successful reload status, exact profile family, screenshot path, and observed glyph/line-width differences against the matching baseline. |
| Family resolution | Confirm the system resolves `Avenir Next` as that family while the sample is rendered. | Record the native font-family resolution evidence. If the app or native accessibility output exposes no family identity, mark identity verification incomplete; screenshots alone do not prove the family name. |
| Stability | Switch between `SF Pro` and `Avenir Next` in both appearances without changing text, size, window, or story. | Each switch updates the rendered sample; neither appearance retains the prior family after reload. Record both directions and screenshots. |

Use identical visible text and window geometry for each paired capture. Record
the current HEAD, macOS version, disposable theme path/hash, selected family,
appearance, reload status, screenshot paths, and native font-resolution result.
Leave any unobserved family identity or rendering result incomplete.

## Pass and cleanup

- **Pass:** both theme families activate successfully in Light and Dark; the
  native system resolves the requested custom family; the rendered sample
  changes between the baseline and custom family in the same story; switching
  back restores the baseline; all paired evidence is recorded.
- **Pending:** slot/process preconditions are unmet, the family is unavailable,
  reload fails, the requested family cannot be independently resolved, or the
  visual change is not observable. Do not infer font identity from the theme
  document alone.
- After the run, quit the validation app with Cmd-Q, make one bounded process
  exit check, remove only this run's disposable Playground bundle and any
  standalone GUI executable it created, then verify no test app remains.
  Preserve evidence and shared Cargo caches.

This run sheet covers native macOS font acceptance. Web font selection is a
separate cross-platform check and remains open.
