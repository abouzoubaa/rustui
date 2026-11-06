# M70 TapRegion pointer interaction

`TapRegion(id, child, action)` wraps any RustUI `View` in a native GPUI `div` and attaches a primary-click listener. `action` accepts the same component or application `ControlAction` forms as `Button`. An empty or whitespace-only ID is rejected. `.enabled(false)` leaves the child rendered and skips listener registration. Each region has a typed `RetentionScope::Tap` and a distinct native `tap` ID namespace; use distinct stable IDs for sibling regions.

GPUI owns hit testing and click dispatch. In the pinned GPUI source, `div.on_click` registers listeners (`vendor/gpui-pre-0.3.9/src/elements/div.rs:591-597`), and the native mouse-up handler invokes primary-click listeners during the bubble phase (`div.rs:3089-3121`). TapRegion does not call `stop_propagation` or `prevent_default`. A descendant control can handle or intercept its own click according to GPUI's native behavior, so nesting interactive controls should be reviewed in the actual app.

The wrapper adds no focus target, keyboard activation contract, or button accessibility role. It is a pointer region, not a substitute for an accessible control. Provide a `Button` or another keyboard-accessible path for essential actions. `apps/rustui-playground/src/stories/tap_gallery.rs` has keyboard-equivalent buttons, disabled-pointer control, two same-ID editors isolated by different Tap scopes, and a right-region removal/restoration control. Removing that region leaves the gallery component owner mounted.

## Acceptance

1. Build and test the public wrapper and scope contracts.
2. In the Playground, click both outer regions and confirm only their corresponding owner counters change. Edit both same-ID fields and confirm each retains its own value across parent updates and right-region removal/restoration.
3. Disable pointer clicks and confirm region clicks no longer change counts while children remain rendered. Re-enable and confirm they work again.
4. Activate the keyboard-equivalent Buttons and confirm the same owner actions run.
5. Inspect nested editor click behavior and any GPUI propagation effects in the native app. Source implementation and compilation alone do not establish native or accessibility acceptance.

## Verification status

The final M69–M71 workspace suite passed 236 tests across 12 binaries, and the build succeeded with all three pinned GPUI dependencies fresh. The locked/offline all-targets check exited 0. Bounded native pointer acceptance is complete and recorded in `target/headstart/milestone70-tap-region-gui.json`. Click bubbling, independent child inputs, disabled wrappers, alternative button actions, and removal/restoration were observed. Keyboard-only activation of the alternative Buttons, exact pixels, RTL, full accessibility speech, and performance remain unverified. TapRegion adds no focus target or button role.