# Sheet Playground story

## Scope

Demonstrate the public trigger-driven `Sheet` API in the canonical RustUI
Playground. The story shows the upstream default side placement, visible title,
RustUI body, and explicit accessible name.

## Accessible name contract

`SheetView` applies the explicit literal or localized accessible name to the
RustUI semantic `dialog` node wrapping the body. The fallback order is explicit
name, visible title, then trigger label. Localized names resolve from the active
RustUI message catalog when lowering the story.

GPUI Components does not expose its Sheet host for RustUI to name. The semantic
node remains inside the body; this contract does not assign a role to the
upstream host or name its close button.

## Acceptance

- The existing Playground `sheet` catalog entry opens this public-API story.
- The story demonstrates an explicit accessible name without changing visible
  title or trigger text.
- Focused unit tests cover explicit literal and localized name selection and
  title/trigger fallback.
- The `rustui` package tests, Playground package check, and formatting check
  pass.
- These source checks do not establish native accessibility, screen-reader,
  locale-switching, focus, theme, RTL, or cross-platform acceptance.


## Bounded native observation

On 2026-10-19, the canonical `dev.rustui.playground` story opened a right-side
sheet. The native accessibility tree exposed a `dialog` named “Details and sheet
contents,” its visible title, and its body. Activating the upstream close
button removed that dialog and its contents from the tree. This does not verify
focus entry, containment or restoration, screen-reader speech, localized name
switching, theme, RTL, cross-platform behavior, or pixel comparison. Evidence:
`target/headstart/evidence/roadmap-m120-m123-native-20261019.json`. The app was
quit; its disposable bundle was removed and shared build artifacts were kept.
