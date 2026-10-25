# P3D native visual evidence record

Use the JSON template for four screenshots of the 32-leaf native workload:
cached and uncached, each before and after its per-leaf mutation. Record a
separate screenshot and observation for every state, along with the exact build
record, launch plan, binary hash, platform, appearance, display, and scale.

Set `visual_capture: true` in each run config to use the capture-only native
layout. It arranges the same 32 leaves in eight columns by four rows, keeps each
210×220 logical-pixel card and all twelve text lines, and opens a 1700×1000
logical-pixel window. The regular process-comparison path remains four columns
by eight rows in a 900×520 window when this field is absent or false. Capture
configs must set `diagnostics_enabled: false`; the app rejects visual capture
configs that enable diagnostics or select a different leaf count. This mode is
for paired pixel review and does not produce performance measurements. It
records AX and native-drag activity but does not stop the capture-only mutation
when screenshot tooling activates AX; those observations remain separate from
native-input acceptance. Record the actual display and scale because the window
must fit on screen for all 32 cards to be visible.

For each capture, record whether all 32 values are visible and whether the
expected initial or mutated state is visible. Also record AX activation, native
drag activity, reviewer, and notes. Preserve failed and inconclusive attempts;
do not treat them as passes. Review pixel equivalence and input/accessibility
behavior separately. No visual or performance threshold is defined here.

Run the structural and hash check with:

```sh
python3 scripts/validate_p3d_visual_evidence.py <manifest.json>
```

The validator checks four-state coverage, provenance and screenshot file hashes,
and required observations. It does not inspect pixels or decide acceptance.


## M582 capture result — 2026-10-21

The four-state capture passed structural validation against
`target/headstart/evidence/p3d-visual-capture-20261021e2.json`. The current
`gpui_component::Root`-wrapped build displayed all 32 cards and their 12 lines
in cached and uncached modes. Initial captures show value `0`; post-mutation
captures show value `1`. Cached and uncached images are byte-identical within
each state. Text contrast appears low. CUA screenshot/AX observation activated
accessibility in both run records; neither reported native dragging. This is a
bounded capture and pixel comparison, not acceptance of native input,
VoiceOver, frame/GPU behavior, or general performance. The disposable app
bundle was removed after each mode and no benchmark process remains.
