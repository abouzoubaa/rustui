# M894 — Stage 11 / route Web keyboard events across focus targets

The v3 GPUI Web patch proposal keeps pointer focus on the hidden IME textarea
and routes keydown/keyup from a guarded window listener when the event target
is the canvas or that textarea. Its focused checker passed: it verifies the
proposal hash, exact patch application to disposable copies, pinned source
hashes, routing invariants, and rustfmt.

The proposal is not applied. It has not been compiled with GPUI or accepted in
a browser. Keyboard activation, text entry, IME, and accessibility runtime
acceptance remain open. Explicit review approval is still required before
applying the upstream patch.
