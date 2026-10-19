# M35: Animated opacity modifier

`AnimatedOpacity::new(id, child, from, to, duration)` is a one-shot public view
modifier that delegates frame scheduling and interpolation timing to the pinned
GPUI animation element. It validates both opacity endpoints as finite values in
`0..=1` and rejects a zero duration.

The animation runs on mount. Its `Identity` must be unique among sibling
animated opacity modifiers in the composition scope. Opacity endpoints and
duration are fixed while mounted; changing them does not retarget the running
animation. Use a new identity to replay it.

The modifier honors GPUI's app-level `reduce_motion` setting. When enabled,
GPUI renders one-shot animations at their end state without scheduling animation
frames. This does not claim OS reduced-motion preference propagation.

The modifier adds no RustUI scheduler, retained animation state, spring or
transition API. M35 validation is recorded in
`target/headstart/milestone34-35-tests-accepted.log`: 97 tests passed across 12
binaries, with 0 failures. `target/headstart/milestone35-opacity-gui.json` records
normal and reduced-motion captures at the full-opacity endpoint after the UI
settled. Intermediate frames were not captured because CUA waits for the UI to
settle. The separate corrected M35 probe accepted normal progression and
reduced-motion stability of RustUI host lowerings, with the exact counts in
`target/headstart/opacity-probe-verification-31323-1792319071087036000.json`.
The initial failed probe remains in
`target/headstart/opacity-probe-verification-19330-1792317490354016000.json`.
These checks do not establish intermediate pixel alpha, GPUI or presented-frame
counts, GPU behavior, presentation timing, or OS reduced-motion preference
propagation.
