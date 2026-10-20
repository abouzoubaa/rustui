# Sheet Playground story

## Scope

Demonstrate the public trigger-driven `Sheet` API in the canonical RustUI
Playground. The story shows the upstream default side placement, localized
trigger and visible title, RustUI body, and explicit localized accessible name.

## Localization API

`SheetView::localized_trigger` and `SheetView::localized_title` use the same
active-catalog message-key contract as `DialogView`. Literal `.title(...)`
and the `Sheet(...)` constructor remain supported. The accessible-name fallback
uses the localized title, then the localized trigger. The Playground catalogs
provide the story's three message keys in English, French, and Arabic.

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
- The story demonstrates localized trigger, title, and accessible name in all
  three declared Playground catalogs.
- Focused unit tests cover explicit literal and localized name selection and
  title/trigger fallback; a Playground test checks all three Sheet story keys
  exist in each declared locale catalog.
- The `rustui` package tests, Playground package check, and formatting check
  pass.
- Source checks do not establish native locale-switching, screen-reader, focus,
  theme, RTL, or cross-platform acceptance.

## Localization source verification

`SheetView::localized_trigger` and `SheetView::localized_title` mirror the
existing Dialog message-key API. Focused Sheet tests cover message-key storage
and accessible-name fallback; the Playground catalog test checks that English,
French, and Arabic each provide the story's trigger, title, and name keys.
Native locale switching and rendered translation remain unverified.


## Bounded native observation

On 2026-10-19, the canonical `dev.rustui.playground` story opened a right-side
sheet. The native accessibility tree exposed a `dialog` named “Details and sheet
contents,” its visible title, and its body. Activating the upstream close
button removed that dialog and its contents from the tree. This does not verify
focus entry, containment or restoration, screen-reader speech, localized name
switching, theme, RTL, cross-platform behavior, or pixel comparison. Evidence:
`target/headstart/evidence/roadmap-m120-m123-native-20261019.json`. The app was
quit; its disposable bundle was removed and shared build artifacts were kept.

## M136 — bounded keyboard focus return

On 2026-10-19, the canonical macOS Playground `sheet` story was opened using
keyboard focus on **Open sheet** followed by Return. The open accessibility tree
exposed the named `dialog`, title, and body. Escape dismissed those nodes and a
visible focus ring remained on **Open sheet**. Pressing Return immediately,
without Tab or pointer input, reopened the sheet. The accessibility API still
reported the Playground window as focused, so this demonstrates the keyboard
return path but does not identify the focused child. Focus containment,
VoiceOver, theme/locale, RTL, and cross-platform behavior remain unverified.
Screenshots were inspected during the live probe but were not persisted as image
files. Evidence: `target/headstart/evidence/sheet-focus-return-m136-20261019.json`.

## M143 — bounded keyboard focus containment

On 2026-10-20, the canonical macOS 27 Playground story was opened from keyboard
focus on **Open sheet**. Tab moved through the Sheet close control, body field,
and body action, then returned to the close control; Shift+Tab from the action
returned to the body field. Escape removed the Sheet nodes and restored the
visible trigger focus ring. An immediate Return reopened the Sheet. The native
accessibility API continued to report window focus, so child focus ownership
and VoiceOver speech remain unverified. Theme/locale switching, RTL, and
cross-platform behavior remain open. Screenshots were inspected live but were
not persisted. The disposable app process was terminated after UI automation
stopped responding during cleanup; its bundle and standalone executable were
moved to Trash, and shared build caches were preserved. Evidence:
`target/headstart/evidence/sheet-focus-containment-m143-20261020.json`.


## M151 — native French localization observation

On macOS 27, the Playground locale was switched to French before selecting the Sheet story. The trigger rendered as **Ouvrir le panneau**. Opening it exposed the title **Détails** and a native `dialog` named **Détails et contenu du panneau**, with the body field and action present. This verifies one French native-rendering path; it does not establish Arabic/RTL, VoiceOver speech, child-focus ownership, locale changes while open, theme, or cross-platform behavior. Evidence: `target/headstart/evidence/sheet-localized-native-m151-20261020.json`.
