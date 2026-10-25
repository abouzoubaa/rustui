# D1b native TextField no-blur acceptance run sheet

## Scope

Run two bounded macOS 27 checks in the canonical RustUI Playground: ordinary
selection retention and active Japanese IME composition across a parent rerender
while the compiled-preview TextField stays focused. Source tests and the M599
selection observation are useful context; they do not replace recording this
run against the current validation app.

The ordinary `Rerender preview parent` button transfers focus and cannot be used
for either no-blur check. A five-second delayed owner-task rerender is the
approved focus-preserving trigger. However, the current worktree's
`compiled_preview_gallery.rs` exposes only the focus-moving button; it does not
contain **Schedule no-blur parent rerender (5 seconds)**. Do not start either
no-blur case with this build. The case steps below apply only when that trigger
is present in the exact app under test and has been verified to run without
moving focus. Otherwise record the trigger as unavailable and leave both
no-blur steps unrun. M599 remains the recorded pass for ordinary selection
retention; the native IME gate remains open. That historical result does not add
this trigger to the current build.

## Before the run

- Claim `@shared/playground-gui-slot` in the shared roadmap claim ledger and
  verify this session is its sole active owner. If it is occupied, do not launch
  the app.
- Use the canonical disposable validation app and bundle identifier
  `dev.rustui.playground.validation`; do not create another app. Record the
  bundle path, executable SHA-256, macOS version, and input source in the result.
- Confirm a Japanese input source is enabled and can show its candidate UI. If
  it is unavailable, run the selection check only and leave native IME
  acceptance open; ordinary Japanese text entry is not composition evidence.
- Confirm the tested app exposes the focus-preserving delayed owner-task
  rerender. If not, stop before the no-blur steps; do not substitute the
  focus-moving button, a click, or keyboard focus traversal. M599 remains the
  selection result; leave native IME acceptance open.
- Start from the compiled-preview gallery with its TextField mounted. Record
  the initial owner value, field value, mount generation, revision, and parent
  rerender count. Use the same mounted generation throughout each case.

## A. Selection retention while focused

1. Set the compiled TextField and owner to `abcdef`.
2. Focus the field and select exactly `cd` (Home, Right twice, Shift+Right
   twice). Record the selected range and confirm the field owns focus.
3. Schedule the verified focus-preserving five-second owner-task rerender. Do
   not click or use focus traversal after returning to the TextField. Keep focus
   in the field until the delayed update has fired; record the rerender count
   and focus state.
4. Type `x` while the field remains focused.

**Pass:** the selection survives the parent update; typing replaces `cd`, and
both field and owner show `abxef`. Mount generation is unchanged, the parent
rerender count advances once, and the accepted edit advances revision. Record
the observed counters; if the UI does not expose a counter, mark it unavailable
rather than infer it.

## B. Active IME composition while focused

1. Reset the mounted field and owner to a known value, such as `base`. Focus the
   TextField and start Japanese composition. Before the delayed update fires,
   record the active input source, visible marked preedit, candidate UI, field
   focus, field value, owner value, generation, revision, and rerender count.
2. Schedule the verified focus-preserving five-second owner-task rerender while
   composition is active. Keep the field focused and composition uncommitted
   until the delayed parent update has fired. Record the same observations
   immediately after it fires.
3. Commit the composition without leaving the field. Record the committed text
   and verify that field and owner converge to that text.
4. If a separate owner-update trigger can run without moving focus, repeat with
   active composition and apply a distinct external owner value during
   composition. Verify the field adopts that external value rather than stale
   preedit. Record its trigger and observations. If no focus-preserving owner
   update is available, mark this subcase open; do not use a click or focus
   traversal and call it no-blur evidence.

**Pass for the rerender case:** marked preedit and candidate UI remain visible
through the parent update; focus and mount generation remain unchanged; owner
value remains the pre-composition value and revision does not advance for
uncommitted preedit. Committing then updates both field and owner, with revision
advancing for accepted committed input. Record the observed values and counters;
do not infer unavailable accessibility-tree counters.

## Evidence and cleanup

Save one result at
`target/headstart/evidence/d1b-ime-selection-no-blur-acceptance-20261025.json`.
Include app identity and hash, OS/input source, exact trigger sequence, before
and after observations for each case, pass/open result, and cleanup results.
Capture enough visible evidence to show the selection or marked preedit,
candidate UI, focus state, field value, and owner value. Keep selection and IME
results as separate cases so one cannot stand in for the other.

After the GUI run, quit the validation app, verify its process is absent, remove
the disposable app bundle and standalone test executable, preserve source and
evidence, and release the GUI-slot claim only after cleanup. Do not touch the
standard Playground package.

## Existing evidence and limits

- M599 recorded `cd` replacement as `abxef` in field and owner after a
  five-second no-blur parent rerender. It did not expose generation, revision,
  or rerender counters.
- M147 tests cover owner-value reconciliation but do not exercise a native
  input method or focused TextField. Native active-composition retention and
  commit propagation remain open until this run is recorded.
- A missing Japanese input source, unavailable focus-preserving trigger, lost
  focus, or unobservable required behavior leaves the corresponding native gate
  open.
