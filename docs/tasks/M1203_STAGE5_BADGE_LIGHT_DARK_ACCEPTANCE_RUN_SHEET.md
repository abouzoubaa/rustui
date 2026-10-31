# M1203 — Stage 5 Badge Light/Dark visual acceptance run sheet

## Status

**Preparation complete; native visual acceptance not run.** The active Badge
count renderer in pinned GPUI Components 0.7.3 hard-codes a white foreground
([`badge.rs:120-127`](../../vendor/gpui-component-0.7.3/src/badge.rs#L120)). RustUI's M1203
wrapper maps the active profile's `badge_background` token to the Badge, but
does not currently map a text foreground (`crates/rustui/src/components/basic/badge.rs`,
[`BadgeView::into_element:214-225`](../../crates/rustui/src/components/basic/badge.rs#L214)). Runtime visual acceptance must wait until the
approved Badge foreground implementation is available in this checkout.

Do not treat this run sheet, the theme token regression, or the ratios below as
a visual pass. Do not start a Playground while another session owns the shared
validation app or while a prior Playground process remains active.

## Source-backed setup

- Use the existing Playground compiled preview Badge specimen, whose defaults
  are label `Inbox`, count `7`, and dot disabled. Its controls support changing
  the label, count, and dot (`apps/rustui-playground/src/stories/compiled_preview_gallery.rs`,
  [`mount_badge_specimen:5602-5604`](../../apps/rustui-playground/src/stories/compiled_preview_gallery.rs#L5602)
  and [`badge_specimen_round_trips_bounded_count_and_dot_mode:10058-10123`](../../apps/rustui-playground/src/stories/compiled_preview_gallery.rs#L10058)).
- Use the active macOS 27 theme profile. Its Light values are
  `badge_background #E6E6E6` and `primary_text #1A1A1A`; its Dark values are
  `badge_background #444444` and `primary_text #FFFFFF`
  ([Light:27273-27282, Dark:27295-27304](../../themes/macos27.json#L27273)).
- RustUI resolves the Badge background from the selected profile and mode in
  `profile_badge_background` and applies it when an active theme exists
  ([`profile_badge_background:54-60`](../../crates/rustui/src/components/basic/badge.rs#L54)
  and [`into_element:214-225`](../../crates/rustui/src/components/basic/badge.rs#L214)). The existing regression
  `badge_background_uses_the_profile_token_for_both_appearances` checks both
  appearances and checks that the selected value differs from the error token
  ([test:244-262](../../crates/rustui/src/components/basic/badge.rs#L244)).
- Pinned Badge count rendering currently uses white text
  ([renderer:120-127](../../vendor/gpui-component-0.7.3/src/badge.rs#L120)). The foreground expectations
  below therefore require the approved foreground override to be present.

## Run procedure

1. **Preflight:** confirm the approved Badge foreground implementation is
   available and built into the Playground under test. Confirm the shared
   validation slot is reserved for this run and no other Playground session
   owns it. Record the app build/revision and macOS version. If either check
   fails, stop; leave the run status pending.
2. Open the existing Playground and the compiled preview Badge specimen. Keep
   the label `Inbox`, count `7`, and dot disabled for the baseline.
3. Select/activate Light through the supported appearance control and confirm
   the active theme mode is Light. Record the rendered badge background and
   count foreground, then inspect that `7` is legible at the actual rendered
   size. If the Playground exposes no supported way to select and confirm both
   modes, stop and record that limitation; do not infer the rendered mode from
   the system setting alone.
4. Select/activate Dark and confirm the active theme mode is Dark. Repeat the
   same observations.
5. Return to Light and Dark once each, confirming the values track the active
   appearance rather than remaining fixed after the first render.
6. Exercise a multi-digit count such as `42` in each appearance. Confirm the
   count remains centered, unclipped, and legible; restore the baseline state.
7. Save screenshots of the Light and Dark baseline states and record measured
   foreground/background colors and contrast ratios. Quit the app and complete
   the repository's disposable-app cleanup procedure after the run.

## Expected observations

Contrast values use WCAG sRGB relative luminance on the opaque source colors;
confirm the actual rendered/composited colors during the run.

| Appearance | Expected background | Expected count foreground | Expected contrast | Visual result |
|---|---|---|---:|---|
| Light | `#E6E6E6` | `#1A1A1A` (active `primary_text`) | **13.94:1** | Count `7` and `42` clearly legible; background is neutral gray, not the error red. |
| Dark | `#444444` | `#FFFFFF` (active `primary_text`) | **9.74:1** | Count `7` and `42` clearly legible; background is dark gray, not the error red. |

The current white-only Badge foreground would yield **1.25:1** on the Light
background, which fails the project's 4.5:1 body-text threshold. It yields
**9.74:1** on the Dark background. These are source-color calculations, not
observations of the rendered app. The 4.5:1 threshold is defined by
[`Accessibility::min_body_contrast`](../../crates/rustui/src/theme/schema.rs#L2263).

## Pass and stop criteria

- **Pass:** both appearances render their expected background and active
  primary text colors; actual contrast is at least 4.5:1; both single- and
  multi-digit counts are legible and unclipped; screenshots and environment
  details are recorded; app cleanup is complete.
- **Stop / pending:** foreground implementation is unavailable, profile or
  appearance selection cannot be confirmed, contrast is below 4.5:1, count
  placement is clipped or illegible, or the shared Playground is occupied.
- A passing token mapping regression is supporting source-level evidence only;
  it does not satisfy this visual acceptance run.

## Current preflight evidence

At preparation time, three Playground processes were present (PIDs 10507,
23568, and 36564). No new app was launched. Recheck process and shared-slot
ownership immediately before any future run; do not terminate another
session's process as part of this preparation task.

The existing M1203 task note records the focused background-token test as
previously passed. It was not rerun while preparing this run sheet.
