# M599 — D1b TextField selection retention without blur

## Acceptance

In the canonical disposable Playground on macOS 27, focus a TextField containing
`abcdef`, select `cd`, trigger a parent rerender without blurring the field, and
type `x`. The field and its bound owner must both read `abxef` after the selected
range is replaced.

## Result

The native check passed. The selected range survived the delayed parent rerender,
and typing `x` produced `abxef` in both the field and its bound owner. The
accessibility tree did not expose generation, revision, or rerender counters, so
none are claimed. The canonical validation app was quit, its process was
confirmed absent, and only its disposable bundle was removed.

Evidence: `target/headstart/evidence/m599-d1b-textfield-selection-retention-20261021.json`.

This closes the focused no-blur selection-retention case only. Active IME
composition, VoiceOver, broader focus traversal, and cross-platform acceptance
remain open.
