# M882 — Stage 11 Web keyboard focus patch review

## Original v2 finding

The v2 proposal added canvas keyboard listeners and made the canvas tabbable, but pointer interaction focuses the sibling hidden IME textarea. Events from that textarea do not bubble through the canvas. The isolated v2 checker did not exercise browser focus or keyboard behavior.

## Follow-up — M894 v3 proposal

The v3 proposal routes keyboard events from both the canvas and hidden IME textarea through a guarded window listener. Its isolated checker passes source/hash, exact disposable patch, routing-invariant, and rustfmt checks. It remains unapplied pending review and explicit approval; GPUI compilation and browser keyboard, text-entry, IME, and accessibility acceptance remain open. See `docs/tasks/M894_STAGE11_WEB_KEYBOARD_ROUTING_PROPOSAL.md`.
