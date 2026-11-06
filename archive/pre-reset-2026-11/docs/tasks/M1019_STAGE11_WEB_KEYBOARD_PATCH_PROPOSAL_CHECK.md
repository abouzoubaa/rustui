# M1019 — Stage 11: validate the Web keyboard patch proposal

The existing v3 proposal routes keydown and keyup through guarded window
listeners for events targeted at the GPUI canvas or its IME mirror. This closes
the focus-boundary gap that caused the v2 proposal to miss Space activation
when the canvas held focus. The isolated checker passed: exact patch and pinned
source hashes matched, the proposal applied without fuzz to a disposable copy,
postimage hashes and routing invariants matched, and rustfmt passed.

The proposal is not applied to the pinned GPUI source. Review and approval are
still required before applying it; browser interaction and accessibility
acceptance remain open afterward.
