# C6: Link

## API

- `components::basic::link::{Link, LinkView}` creates a link from visible text
  and a URL.
- `.accessibility_label(text)` can provide a more descriptive accessible name;
  the visible label remains unchanged.
- `.disabled(true)` prevents pointer and keyboard activation and removes the
  link from tab traversal.
- `.id(identity)` supplies a stable element identity. Without an explicit ID,
  the fallback uses both label and URL. Repeated links with the same label and
  URL should receive distinct explicit IDs.

Activation delegates the captured URL to `gpui::App::open_url`, which uses the
platform's default browser. The URL is passed through unchanged; RustUI does not
guess URL schemes or add URL filtering.

## Implementation and interaction semantics

The wrapper uses the pinned GPUI Components Button's Link presentation, which
provides the AccessKit Link role and the button base's focus and keyboard
behavior. Enter and Space both activate the link, matching that pinned Button
behavior. This is not strict HTML anchor keyboard behavior.

Disabled state is passed to the pinned Button and also sets its tab stop to
false. The base Button prevents disabled pointer and keyboard activation; its
accessibility test confirms that disabled controls do not expose the Click
action. This note makes no claim that an AccessKit disabled property is set.

## Pinned Link audit

The pinned GPUI Components 0.7.3 `Link` stores `disabled` but does not read it
while rendering. It always installs its click handler, which opens `href` with
`cx.open_url` and invokes the optional callback. Its rendered `div` has no Link
role, focus/tab stop, or keyboard handler. Its tests confirm that disabled
links still open and invoke their callback, while Enter and Space do not
activate. Source: `vendor/gpui-component-0.7.3/src/link.rs`.

The RustUI wrapper uses the pinned Button Link variant rather than inheriting
those gaps. No dependency, vendor source, shared export, or manifest change is
part of C6.

## Verification boundary

Focused unit tests cover defaults, accessible-name override, explicit identity,
fallback identity stability and uniqueness across label/URL changes, and the
disabled builder state.

## Native acceptance (M19)

The native Playground review passed enabled-link mouse activation twice, disabled
mouse activation without changing the count or browser, and keyboard focus traversal
that focused the enabled link while skipping the disabled link. Enter and Space each
activated the focused link; Space behavior is inherited from the pinned Button Link
presentation. A local fixture opened in Safari and showed the expected title.
Evidence: `target/headstart/milestone19-links-gui.json`.

External navigation and screen-reader announcement behavior were not tested. This
evidence accepts the exercised native interaction slice only.
