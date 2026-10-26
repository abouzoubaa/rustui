# D1b native TextField no-blur acceptance run sheet

## Scope

**Current user scope, 2026-10-26:** English, French and Arabic only. Japanese and
other languages/scripts, including case B below, are undone/deferred for later.
Do not enable Japanese or run case B under the current release scope. Preserve
this procedure for a later explicitly resumed language expansion. The temporary
Japanese keyboard/dictation setup was removed and original settings restored.
Ordinary focused-selection retention in case A remains in scope.

Run two bounded macOS 27 checks in the canonical RustUI Playground: ordinary
selection retention and active Japanese IME composition across a parent rerender
while the compiled-preview TextField stays focused. Source tests and the M599
selection observation are useful context; they do not replace recording this
run against the current validation app.

The ordinary `Rerender preview parent` button transfers focus and cannot establish
no-blur behavior. Use the existing five-second delayed owner-task rerender. Its
control is now placed immediately above the live compiled preview, after the
specimen catalog, so it can be scheduled before focusing the actual mounted
TextField. This is a layout repair in the Playground; the owner task and field
implementation are unchanged. Runtime acceptance for the rebuilt candidate must
still be recorded; the old missing-trigger statement was stale.

Schedule the delayed update first, then focus the live field and prepare the
selection or composition before five seconds expires. Capture the prepared state
before the update and do not change focus until the update has fired. Do not use
the schema inspector's `text` property editor as the mounted runtime TextField.

## Before the run

- Claim `@shared/playground-gui-slot` in the shared roadmap claim ledger and
  verify this session is its sole active owner. If it is occupied, do not launch
  the app.
- Use the canonical disposable validation app and bundle identifier
  `dev.rustui.playground`; do not create another app. Record the
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

1. Mount the runtime TextField and set its value and owner to `abcdef`.
2. Schedule the adjacent five-second rerender. Immediately focus the runtime
   field and select exactly `cd` (Home, Right twice, Shift+Right twice), before
   the delayed task completes. Record the selection, focus and starting counters.
3. Keep focus in that field without clicking or traversing until the delayed
   update fires. Record the updated rerender count, unchanged mount generation,
   selected range and focus state.
4. Type `x` while the field remains focused.

**Pass:** typing replaces `cd`; field and owner show `abxef`. Mount generation
is unchanged, the rerender count advances once, and the accepted edit advances
revision. Record observed counters; mark unavailable counters as unavailable.

## B. Active IME composition while focused

1. Reset the same mounted field and owner to a known value, such as `base`.
2. Schedule the adjacent five-second rerender, immediately focus the runtime
   TextField and begin Japanese composition before the task completes. Record
   the input source, marked preedit, candidate UI, focus, field/owner values,
   generation, revision and rerender count while composition is uncommitted.
3. Keep that field focused and composition uncommitted through the delayed
   parent update. Record the same observations after the task fires.
4. Commit without leaving the field; verify field and owner converge and record
   the revision advance.
5. If a separate owner-value update can run without moving focus, repeat with
   active composition and apply a distinct external owner value. Otherwise leave
   this subcase open. A click or focus traversal is not no-blur evidence.

**Pass for the rerender case:** marked preedit and candidate UI survive the
parent update, with unchanged focus and mount generation. Owner value and
revision stay at their pre-composition values until commit. Commit then updates
field and owner and advances revision. Do not infer counters from source alone.

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
