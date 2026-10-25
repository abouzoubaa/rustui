# macOS 27 visual interaction fidelity acceptance gate — 2026-10-22

## Acceptance objective

Subsequent macOS 27 theme design is accepted only when it demonstrates visual and interaction fidelity against observed macOS 27 controls and official macOS 27 guidance/APIs. Generated images and local artwork are supplementary concept references. They do not establish native appearance, behavior, measurements, or compliance.

The authority order is:

1. **Observed macOS 27 runtime behavior** on a recorded OS build, device/display, scale, appearance, and relevant settings.
2. **Official Apple macOS 27 guidance and API declarations**, including exact API names and supported availability.
3. **Measured native values** captured from the target runtime with a documented method and uncertainty.
4. **Provisional RustUI design choices** used only where native observation or an official semantic API does not prescribe the detail.
5. **Unknown / unverified** where evidence is missing. Leave the property unresolved; do not supply invented native values.

Generated images may help explore composition, but remain supplementary and cannot override levels 1–3. Catalogue coverage, a candidate JSON record, a static checker, or a screenshot of a generated design cannot pass this gate.

## Required provenance for each important decision

For each important color, metric, material, state, or interaction decision, record a row in the relevant HIG design ledger with:

- Stable decision ID; component and semantic part; property or behavior being decided.
- Evidence class: `measured native value`, `official semantic API`, `provisional RustUI choice`, or `unknown / unverified`.
- Source artifact or official URL, exact OS/API availability, capture or measurement method, and evidence date.
- Confidence and limitations. A measured value must identify the runtime configuration and measurement uncertainty; an API-backed semantic behavior must identify the symbol and availability; a provisional choice must be labeled as RustUI-owned.
- Current theme JSON field, runtime consumer (if any), candidate contract, and whether it is presently unimplemented or unverified.

Do not convert a visual estimate into an exact point, radius, color, opacity, blur, or animation value. If the target value cannot be observed or sourced, record `unknown / unverified` and keep the design decision provisional.

## Required comparison and interaction coverage

For each applicable component family, collect native macOS 27 evidence and compare the candidate RustUI rendering and behavior in a controlled, recorded configuration. Include:

- **Appearance and window state:** Light and Dark; system appearance changes; active and inactive windows; selected/unselected and enabled/disabled states; accent choices including Graphite only where the OS exposes them.
- **Accessibility environment:** Increase Contrast / Show Borders and Reduce Transparency where exposed by the target OS; verify the resulting control, edge, focus, and material treatments rather than inferring them from color tokens.
- **Sizing and typography:** native control sizes, text styles, truncation/wrapping, localized expansion, display scale, and constrained/resized layouts. Record measured values separately from semantic API behavior.
- **Input and accessibility:** keyboard-only and full-keyboard-access navigation, visible focus, activation and cancellation, VoiceOver names/roles/values/state, text composition and selection where applicable, RTL direction, and focus entry/return for overlays.
- **Native integration:** verify host-owned windows, titlebars, controls, menus, panels, popovers, notifications, Quick Look, sharing, Widgets, and other system surfaces through their actual native API path. A RustUI approximation must be labeled as such.
- **Materials and motion:** observe native Glass, scroll-edge, background, and window-corner behavior only on macOS 27; verify fallback when transparency or contrast settings change. Opacity/radius data alone is not evidence of native material support.

Each comparison record must identify the native reference capture, RustUI capture, OS/build, hardware/display/scale, appearance, accent, active/inactive state, accessibility settings, interaction steps, and reviewer outcome. State-specific captures must be paired; do not compare unrelated states or generated images against native screenshots as if they shared provenance.

## Gate decision

Mark a design slice **demonstrated** only after every applicable comparison above has native evidence, a per-decision evidence class and provenance, and a recorded pass/fail outcome for both visible treatment and interaction behavior. Unresolved properties remain explicitly unverified; they cannot be averaged away by complete catalogue coverage.

If macOS 27 runtime or reference access is missing, set the slice status to **`NOT DEMONSTRATED — macOS 27 reference evidence unavailable`**, list the missing captures/APIs, and keep measured native values unknown. Do not substitute the local macOS 26.1 SDK, generated images, reports, or assumed API availability for macOS 27 evidence.

## Current status

**Not demonstrated.** The current HIG set has item-level coverage and static provenance checks, but this acceptance record contains no macOS 27 runtime captures, interaction observations, VoiceOver results, or measured native values. The macOS 26.1 SDK previously identified for this workspace is not a macOS 27 reference. No fidelity/compliance claim is made from the catalogues or static JSON validation.
