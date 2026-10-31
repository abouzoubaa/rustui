# Stage 11 Web Accessibility Bridge — Patch Proposal v1

**Status: proposal only.** This document defines a reviewable, versioned patch
proposal. No executable patch was generated or applied, and no dependency or
RustUI integration change is authorized by this proposal.

## Finding and ownership

RustUI already writes roles, labels, descriptions, values, and live-region
metadata into GPUI AccessKit nodes. GPUI core builds complete `TreeUpdate`
snapshots, but the WebAssembly path cannot currently deliver them:

- GPUI registers accessibility activation and action callbacks only under
  `cfg(not(target_family = "wasm"))` in
  `vendor/gpui-pre-0.3.9/src/window.rs` around lines 1666–1707.
- GPUI emits per-frame updates only while its accessibility state is active;
  the Wasm path never activates it. `Window::handle_a11y_action` is also
  excluded on Wasm.
- `PlatformWindow` already defines `a11y_init`, `a11y_tree_update`, and
  `a11y_update_window_bounds`, with no-op defaults. `WebWindow` implements none
  of them. Its canvas and hidden IME textarea do not represent rendered GPUI
  controls in the DOM.

This is a GPUI Web host gap, with a small GPUI-core Wasm enablement change.
RustUI should continue supplying accurate AccessKit semantics, but it should
not own a second semantic tree or a Playground-only DOM overlay.

## Proposed patch boundary

Patch ID: `gpui-pre-web-accessibility-bridge-v1`.

Apply after the already approved `gpui-pre-web-keyboard-focus-activation-v3`
patch. Keep both packages at their current versions and pinned source revision.
The smallest proposed source scope is:

1. `vendor/gpui-pre-0.3.9/src/window.rs`: enable the existing AccessKit
   callback setup and action dispatch path on Wasm. Keep
   `accessibility_force_disabled` respected. Keep tree generation active in the
   browser because this host has no reliable browser API for detecting screen
   reader attachment.
2. `vendor/gpui-pre-web-0.3.9/src/window.rs`: implement the three existing
   `PlatformWindow` accessibility hooks. On initialization, activate GPUI's
   callback path and retain its action callback. On each update, reconcile the
   current full snapshot into browser DOM nodes keyed by AccessKit node ID.
   Reuse existing DOM nodes in place, update their attributes and bounds, and
   remove stale IDs; do not rebuild the entire focused subtree on every frame.
3. `vendor/gpui-pre-web-0.3.9/Cargo.toml`: add the `web-sys` `Node` feature if
   required for DOM hierarchy operations. Use the existing `gpui::accesskit`
   re-export and existing `web-sys` dependency; do not add an adapter crate.

Map AccessKit hierarchy and supported roles, names, descriptions, values, live
regions, focus, and states such as disabled, checked, selected, and expanded to
DOM/ARIA semantics. Position semantic proxy elements from AccessKit bounds,
converting to browser CSS coordinates and accounting for canvas offset, device
scale, viewport changes, clipping, and removed nodes. Keep the visual canvas
renderer authoritative for drawing. The DOM mirror owns only browser semantic
exposure and action routing; it must not create a second RustUI state or
rendering path.

Forward supported browser focus and activation events through the retained
`A11yCallbacks.action` callback as AccessKit `Focus` and `Click` requests. GPUI
core remains responsible for focus and action behavior. Prevent one browser
activation from being delivered both through the DOM mirror and the existing
canvas input path. Keep the IME textarea private and outside the semantic tree.

The tree builder currently sends complete snapshots, so this proposal does not
need `accesskit_consumer` merely to apply updates. A keyed in-place DOM
reconciliation can consume the existing `TreeUpdate` directly. No new crate or
version is proposed; adding a `web-sys` feature does not select a new package
revision. If implementation review finds that an additional crate is necessary,
that is a scope change requiring a separate dependency and license review.

## Pinned source, patch order, and licensing

The package is Longbridge's `gpui-pre` 0.3.9 packaging of official Zed revision
`93d745668805029476e6d7fbb19652116fed51de`; it is not a Zed-published release.
`gpui-pre-web` is Apache-2.0. Preserve its `LICENSE-APACHE`, upstream
attribution, registry provenance, and pristine `.cargo-checksum.json`. Preserve
the same upstream license and checksum metadata for the touched `gpui-pre` core
package. No third-party implementation code is proposed for copying.

| Source | Required patch preimage | Provenance |
| --- | --- | --- |
| `gpui-pre 0.3.9/src/window.rs` | SHA-256 `44f32319b9385fa6dbc15c36b3edea00a26208309b37283ae4f9bd4bead674f8` | Pristine source file; package source checksum |
| `gpui-pre-web 0.3.9/src/window.rs` | SHA-256 `3779ba8d5647009b80d0ea52dc7823806640741d3a1cfa9a0a7752ec35fe7dd8` | Approved M855 v3 postimage; mandatory prior patch |
| `gpui-pre-web 0.3.9/src/window.rs` pristine base | SHA-256 `eda543e585912fcb012e2a3559d8fb2e9ad7570ba58746747af380d044085cb8` | Original package checksum entry; not the v1 patch preimage after M855 |
| `gpui-pre-web 0.3.9/Cargo.toml` | SHA-256 `f51c51c22a0b75567b66c2a8078d14612545ecbeef002ca9ec1e07799bd198ff` | Original package checksum entry; only changed if adding the `Node` feature |
| `gpui-pre-web 0.3.9/Cargo.toml.orig` | SHA-256 `e3e8f27d5859dc6d925ef7b84e7d3d181c32ca6891b7961b0d08544a6d1c42da` | Original manifest provenance; update only if the normalized manifest feature is added |

The original `gpui-pre-web` registry checksum is
`aedd55b99dc77e28df8202b0ea868aa5c90fda4aefff539ea21087d7c2ff4bfb`; its
pristine `.cargo-checksum.json` SHA-256 is
`aad9503f554e462d582461d008ea4bc0dda00021812ccf6b876734b55757579e`. The
ordered patch series must record M855 v3 before this proposal and include hashes
for the new patch and all touched preimages/postimages. Preserve the source
checksum metadata unchanged.

## Focused validation gates

Run candidate checks only in a fresh temporary copy created from the pinned
package sources. When an executable v1 diff is prepared, apply and verify M855
v3 first, verify the preimages above, then apply that diff with zero fuzz. Never
validate by applying it to `vendor/`. Record the executable diff hash and its
postimage hashes in the proposal before any approval review.

Renderer-level checks should cover a representative nested tree snapshot and a
subsequent snapshot with changed state, focus, bounds, and removed nodes. Assert
stable DOM identity for unchanged AccessKit IDs, correct role/name/state
attributes, correct parent order, stale-node removal, and one-to-one action
routing. Include a disabled control that does not activate. Verify that
`accessibility_force_disabled` still suppresses the bridge and that enabling the
bridge does not change GPUI's non-Web platform behavior.

Browser acceptance should use the current Playground Button story and a fresh
browser accessibility snapshot:

- Expose separate named `Increment` and `Reset` buttons, disabled `Unavailable`,
  and understandable `Count` text with the expected order and state.
- Tab to a semantic control and activate it with Enter and Space. Each action
  must reach GPUI once; Increment changes the count from 0 to 1 and the updated
  value is visible in the next accessibility snapshot.
- Verify focus remains associated with the matching semantic node after state
  changes, resize, and device-scale changes. No stale, duplicated, or misplaced
  controls may remain in the browser tree.
- Preserve M855 canvas keyboard focus/activation, pointer hit testing, and the
  text/IME path. The hidden IME textarea must not appear as a user-facing
  control; ensure the canvas is not the only accessibility representation of
  the story controls.

Separately test actual screen-reader speech in the target browser/OS, including
role, name, disabled state, focus movement, and the changed count. A browser tree
snapshot is not screen-reader acceptance. Stage 11 remains incomplete until
these browser and screen-reader gates run and pass.

## Risks, fallback, and approval

Browser accessibility tree generation will run on each Web frame because the
host cannot reliably detect screen-reader attachment. Measure its cost in a
later approved candidate. Role/state mapping, DOM focus, browser action
translation, geometry, and interactions with the M855 canvas input route all
need renderer and browser coverage.

Fallback: keep the pinned sources and dependency graph unchanged and leave the
browser accessibility blocker open. Remove this patch only after an upstream
Web adapter provides equivalent semantics and passes the same acceptance gates.
Do not change upstream revisions, apply this patch to `vendor/`, or integrate
RustUI changes without separate user approval and the repository's reproducible,
versioned patch process.

**Evidence in this proposal:** source and patch metadata inspection only. No
patch was generated, applied, compiled, or browser-tested for this proposal.
