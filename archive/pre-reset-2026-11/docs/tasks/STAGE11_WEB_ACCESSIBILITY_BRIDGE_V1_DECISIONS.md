# Stage 11 Web Accessibility Bridge v1 — Decision Aid

**Status: decision aid only.** This brief records minimal recommendations for four contracts left open by the proposal. It does not approve a patch, accept browser accessibility, or authorize changes to pinned sources. The proposed owner remains the GPUI Web host; RustUI continues to provide AccessKit semantics.

## 1. Role mapping and fallback

**Options**

- Map every AccessKit role to a corresponding HTML element or ARIA role. This can provide rich semantics, but risks inaccurate mappings and browser-specific behavior.
- Map only reviewed roles; expose unsupported nodes as generic containers while retaining supported name and state properties. This is predictable but loses semantics for unsupported roles.
- Omit unsupported nodes. This avoids false semantics but can hide useful names or structure.

**Recommendation:** Maintain a small, explicit mapping for roles with clear browser equivalents. Preserve supported labels, descriptions, values, states, and parent/child order. Use a non-interactive generic container for unsupported structural roles; do not invent an interactive role or action. Expand the mapping only alongside browser-tree evidence.

**Evidence:** The proposal identifies role/state mapping as an open renderer-level risk. GPUI's AccessKit builder emits a complete `TreeUpdate` (`vendor/gpui-pre-0.3.9/src/window/a11y.rs:544-581`); AccessKit nodes are the semantic source, rather than a second RustUI tree.

## 2. DOM focus and keyboard routing

**Options**

- Make the canvas the only tab stop and expose a passive semantic mirror. This preserves the current keyboard target, but does not make individual semantic controls naturally focusable.
- Make semantic proxies focusable and route their focus/actions into GPUI. This supports browser navigation but requires the bridge to own focus synchronization.
- Make both canvas and proxies independently tabbable. This is simple to prototype but creates duplicate or confusing tab stops.

**Recommendation:** Make semantic proxies the browser-facing focus targets and translate supported focus changes into AccessKit `Focus` actions. Keep the hidden IME mirror private. Do not add a second canvas tab stop for the same controls; preserve canvas-directed input for non-semantic canvas interaction. Specify focus restoration after tree updates before implementation.

**Evidence:** M855 v3 filters Web keyboard events to the canvas or hidden IME mirror (`vendor/gpui-pre-web-0.3.9/src/events.rs:214-219`) and makes the canvas tabbable. GPUI's activation callback setup and `handle_a11y_action` are gated off on Wasm (`vendor/gpui-pre-0.3.9/src/window.rs:1669-1707`, `:7033-7040`).

## 3. Duplicate action delivery

**Options**

- Let both the semantic DOM and canvas event path observe activation. This is easy to wire but can dispatch twice.
- Dispatch from the semantic target once and prevent that same activation from reaching the canvas path. This needs explicit event ownership but has a clear exactly-once rule.
- Route all input through the canvas and infer semantic actions there. This avoids two dispatch paths but weakens native browser interaction with semantic proxies.

**Recommendation:** Give a semantic proxy activation one owner: translate it once to the matching AccessKit action, consume that browser event before it can trigger the canvas path, and ignore unsupported actions. Keep text entry on the existing private IME path. Validate one action per Enter, Space, and pointer activation in the later renderer/browser gate.

**Evidence:** The proposal calls for forwarding supported focus and click actions and preventing duplicate delivery. The current Web keyboard filter only accepts the canvas and IME mirror (`vendor/gpui-pre-web-0.3.9/src/events.rs:214-219`); GPUI provides the existing action callback path, but excludes its handler on Wasm (`vendor/gpui-pre-0.3.9/src/window.rs:7033-7040`).

## 4. Bounds, resize, and device scale

**Options**

- Position proxies once from initial bounds. This is minimal but leaves stale geometry after layout or viewport changes.
- Reconcile proxy bounds from each complete tree update and refresh the canvas-to-CSS transform on resize and device-scale changes. This adds update work but keeps controls aligned.
- Use `a11y_update_window_bounds` as the sole geometry update channel. This fits the platform abstraction, but the pinned GPUI path does not currently call that hook.

**Recommendation:** Derive proxy geometry from AccessKit node bounds on each tree update; recompute the canvas-to-CSS transform when the host viewport or device scale changes, and remove proxies for nodes absent from the next snapshot. Treat `a11y_update_window_bounds` as an integration point only if the Web host actually wires and calls it. Confirm coordinate units and clipping against the pinned renderer before implementation.

**Evidence:** GPUI exposes `a11y_update_window_bounds` in `vendor/gpui-pre-0.3.9/src/platform.rs:1387-1393`; the pinned packages have no call site beyond the trait default. The AccessKit builder finalizes full snapshots at `vendor/gpui-pre-0.3.9/src/window/a11y.rs:544-581`.

## Decision boundary

These are recommendations for an owner decision, not accepted contracts. No source, vendor, dependency, proposal, patch, or checksum file was changed. The existing proposal's renderer and browser acceptance gates remain necessary; Stage 11 accessibility is not complete on the basis of this document.
