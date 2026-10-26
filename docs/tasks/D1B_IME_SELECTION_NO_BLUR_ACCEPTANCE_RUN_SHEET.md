# D1b native TextField no-blur acceptance run sheet

## Scope

**Current user scope, 2026-10-26:** English, French and Arabic only. Japanese and
other languages/scripts, including case B below, are undone/deferred for later.
Do not enable Japanese or run case B under the current release scope. Preserve
this procedure for a later explicitly resumed language expansion. The temporary
Japanese keyboard/dictation setup was removed and original settings restored.
Ordinary focused-selection retention in case A remains in scope.

The current macOS 27 run covers ordinary selection, French dead-key composition,
Arabic keyboard entry, mixed-direction editing and locale retention in the
canonical RustUI Playground. Case B preserves the future Japanese procedure;
it is outside the current release scope. Source tests and the M599 observation
do not replace current native evidence.

The ordinary `Rerender preview parent` button transfers focus and cannot establish
no-blur behavior. Use the existing five-second delayed owner-task rerender. Its
control is now placed immediately above the live compiled preview, after the
specimen catalog, so it can be scheduled before focusing the actual mounted
TextField. This is a layout repair in the Playground; the owner task and field
implementation are unchanged. The rebuilt candidate's bounded native results
are recorded below; the old missing-trigger statement was stale.

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
- For current English/French/Arabic checks, use the existing ABC and Arabic
  input sources and restore the original configuration afterward. Do not enable
  Japanese. If case B is explicitly resumed later, first verify its Japanese
  input source and candidate UI; plain Japanese text is not composition evidence.
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

## External owner-value update in the Text inputs story — 2026-10-26

The public-API Playground Text inputs story now schedules a distinct owner
value after five seconds. This exercises the mounted Name `TextField`, its
binding, and the other controls sharing that binding. The native app was built
with `./scripts/cargo-headstart build -p rustui-playground --locked --offline`
on macOS 27, packaged as `dev.rustui.playground`, and bound to executable
SHA-256 `4add853a5a23e3cac1f445065acf3393123c87c27a3a15ad013e51a1b40bf5e3`.
The selected macOS keyboard source was ABC; the Playground locale was French.

- **Focused update:** Name was the focused accessibility element before and
  after the delayed callback. The owner, Name, Search, and Notes values all
  changed to `External owner update 3` without a focus change. This is a
  bounded no-blur external-value pass.
- **French dead-key update:** With Name focused, ABC Option+E showed a marked
  acute accent in the field while the owner stayed `External owner update 3`.
  Clicking the update button left Name focused. After the timer, the owner and
  all bound fields showed `External owner update 4`, Name remained focused,
  and pressing E produced `External owner update 4é` in the owner and fields.
  This is a bounded external-value synchronization and next-key pass. It does
  not establish candidate-window or Japanese IME behavior.
- An earlier attempt typed `été` across the timer boundary and ended with
  `External owner uptdate 2`. The exact keystroke/callback order was not
  captured, so this remains **inconclusive** and needs a controlled mixed-
  typing test before claiming that behavior.

The owned app process exited. The disposable bundle and standalone binary were
removed; shared Cargo caches remain. The GUI-slot claim was released. This
run's source check, build, package, native AX observations, and screenshot
inspection are distinct evidence; no saved screenshot or marked-range counter
was captured for the new case. Wider macOS input, browser IME, and the deferred
accessibility/VoiceOver and additional-language cases remain open.

## Evidence and cleanup

The current result is
`target/headstart/evidence/native-en-fr-ar-selection-20261026/acceptance.json`.
Use a separately dated result for any later test run.
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

- Current candidate: application code `a8290b2465aa18de987e55da27367357ad9fec85`,
  executable SHA-256
  `60cc18a69b09b3c02ac68b1a11f347cf6b82c226be8b4955f91e42dfbc422ba1`.
  **8/8 bounded native cases pass** on macOS 27 arm64. No component code was
  rewritten for these checks.
- Case A: `cd` remains selected across the delayed update, then `x` produces
  `abxef` in field/owner; generation 2 stays mounted, revision 6→7 and parent
  rerenders 0→1. Preparation took 494 ms of the five-second delay.
- French: Option+E marked `Caf´` survives the focused parent update, then E
  commits `Café` in field/owner; generation 2 stays mounted, revision 12→13 and
  rerenders 1→2. Preparation took 338 ms. This is Latin dead-key evidence,
  not Japanese candidate-UI acceptance.
- Arabic: existing Arabic keyboard keys produce `سلام`; Backspace produces
  `سلا`. Switching back to ABC and entering Latin/digits produces
  `سلام 123 Café`; Backspace removes the logical final `é`. The bound owner
  receives the exact value. English/French/Arabic switching retains generation
  2/revision 30 and the mixed value; root layout mirrors for Arabic.
- Full mixed-text selection remains focused/highlighted across another delayed
  update, then `x` replaces the whole value; generation 2 remains, revision
  30→31 and rerenders 2→3. Preparation took 327 ms. Observation intervals
  include automation and are not latency measurements.
- Owned PID 82483 is absent, disposable bundle removed, archive/caches retained,
  and enabled/selected/current keyboard configuration matches the original.
  Screenshots, full owner snapshots, checks and hashes accompany the result.
- These checks cover one preview TextField. Exhaustive bidirectional caret,
  typography, other control/browser behavior and external owner-value changes
  during active composition remain separate. Japanese/other languages/scripts,
  accessibility and VoiceOver are undone/deferred.

- M599 recorded `cd` replacement as `abxef` in field and owner after a
  five-second no-blur parent rerender. It did not expose generation, revision,
  or rerender counters.
- M147 tests cover owner-value reconciliation but do not establish native input
  acceptance. The current French marked-accent observation adds bounded native
  retention/commit evidence; it does not close the deferred Japanese case B.
- Lost focus or unobservable required behavior leaves the corresponding case
  open. A deferred language is not counted as accepted.

## Exact-path current candidate focus check — 2026-10-26

The disposable bundle from this writing worktree, SHA-256
`32382ba3031f31b7a5c2f927d673954a14042b50b237756188b7bb99ea6bbbaf`,
exposed the five-second owner update control on macOS 27. After scheduling it,
the Name field accepted `abc`; the timer then replaced its value with
`External owner update 3`. Typing `z` without another click changed the Name
field and shared binding to `External owner update 3z`. This is bounded focus
and subsequent typing evidence. It does not prove active OS IME composition or
precise ordering of the earlier `abc` keystrokes against the timer.

App-name lookup initially opened an older `c611` candidate with 10 demos.
That instance was quit and excluded. Binding the exact current bundle path
selected the 15-demo candidate. Both session-launched app processes were quit,
the disposable bundle and standalone executable were removed, and the GUI slot
was released. The raw observation and provenance record is at
`target/headstart/evidence/native-owner-update-current-20261026/result.json`.
