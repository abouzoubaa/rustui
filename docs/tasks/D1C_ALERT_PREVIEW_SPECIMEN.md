# D1c Alert preview specimen

The canonical compiled-preview gallery includes an opt-in `alert-specimen` built with the public RustUI `Alert` API. Its message is bounded to 1–200 characters and visibility is a Boolean property. The `dismiss` event has an empty payload and maps to an allowlisted `dismiss` action. The Dismiss alert button dispatches the stamped signal; its owner action writes visibility through the existing Boolean binding feedback path.

The focused locked/offline Playground library regression passed 1/1. It checks the bounded schema, signal-only event mapping, dismissal feedback, descriptor export/import, and preservation of the mounted preview when a whitespace-only message is imported. Rustfmt and scoped diff checks passed. Source-test evidence: `target/headstart/evidence/d1c-alert-specimen-test-20261020.log`.

## Bounded native acceptance — 2026-10-20

The canonical Playground rendered the Warning card with title `Connection issue` and message `Connection lost`. Pressing Dismiss hid it and changed the bound visibility to false. Export included that false value; importing it restored the hidden state in generation 7. Turning visibility on in the inspector showed the card again, and the visible Dismiss alert button hid it once more.

The macOS accessibility snapshot exposed the inspector’s message field, the visible switch, and the Dismiss alert button, but it did not retain raw `AXRole`/`AXSubrole` attributes or confirm the rendered title/message as Alert descendants. The pinned AccessKit macOS backend maps `Role::Alert` to `AXGroup` with `AXApplicationAlert` subrole, so the CUA container label alone cannot confirm or refute the mapping. VoiceOver was not captured. The specimen fixes the Warning variant, so other variants were not checked. Native role/subrole, announcement, and accessible-content acceptance remain open.

Evidence: `target/headstart/evidence/d1c-alert-native-acceptance-20261020.json`. The disposable validation app and executable were removed; the pre-existing Playground process PID 20036 was untouched. No screenshot file was retained.

## M258 — Native Alert AX retest and control reachability

The specimen mount controls were in one horizontal row that clipped after the visible pane. They now use a vertical stack, making every specimen control reachable in the Playground. A fresh native run mounted `alert-specimen` at generation 2; the macOS AX snapshot exposed one `container Connection issue` with help text `Connection lost`, plus `Close` and `Dismiss alert` controls. It did not report an explicit Alert role, and VoiceOver was not tested. Accessible content is now present; Alert-role and spoken-announcement acceptance remain open. Evidence: `target/headstart/evidence/d1c-alert-native-ax-retest-20261020.json`. The disposable process and bundle were removed; shared Playground PID 20036 was untouched.
