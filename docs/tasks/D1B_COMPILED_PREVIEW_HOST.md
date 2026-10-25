# D1b local compiled preview host

## Contract

`rustui::preview::PreviewRegistry<C>` pairs D1a component schemas with explicitly registered, ordinary compiled Rust factories. `C` is typed local owner context, such as RustUI bindings and owner actions. The registry rejects event schemas with payload properties. Its catalog records explicit action targets; descriptors can reference only those targets. Factories, closures, component state, bindings and GPUI entities are never serialized.

`PreviewHost<C>` is a value stored in an ordinary RustUI component. GPUI remains the owner, scheduler and renderer. The host validates descriptors with the existing D1a validator and invokes factories returning existing `AnyView` values. It has no transport, task loop, reactive runtime or dependency additions.

A `PreviewStamp` contains preview ID, generation and revision. Mount validates before replacing a current specimen and advances generation. Edits and binding feedback require the exact current stamp, apply the D1a typed property batch atomically, and advance revision. Failed validation leaves both values and stamp unchanged. Teardown also requires the exact stamp. A subsequent mount advances generation, including when the same preview/component IDs are reused.

A `PreviewSignal` carries only `EventId` plus the stamp. `dispatch` checks the exact current stamp and resolves the event's explicitly allowlisted `ActionRef`. The owner then invokes its compiled handler. Signals carry no arbitrary payload values; typed binding feedback is a direct local method call through D1a property validation.

`render_factory()` captures an immutable render description and shared immutable registry. It cannot mount, edit or clone the live host. Captured callbacks remain bound to that description's stamp and are checked against the owner's live host before mutation. The captured registry contains ordinary compiled factories, never serialized code.

Rendering uses existing `Keyed` retention scopes, with preview ID plus generation as identity. A property revision or parent rerender keeps that scope and stable control IDs. Teardown/remount changes the scope. Native edit-state preservation still requires GUI evidence; the scope design alone does not prove selection, caret or IME acceptance.

## Narrow owner APIs

`OwnerAction::action_with(value, handler)` captures an ordinary typed value and routes its mutation through the existing GPUI owner updater. The Playground captures the immutable signal using this method.

`Binding::new_with(current, context, read, write)` captures typed context. The PreviewHost API still requires an exact stamp for every edit. The Playground’s retained TextField uses a mount-scoped shared revision lease: accepted feedback advances the lease so the retained InputState can keep editing, and each remount receives a new lease. A callback retained from an older mount therefore stays stale; callbacks sharing the current mount lease intentionally use its latest accepted revision. Ordinary existing actions and bindings retain their existing APIs and behavior.

Callers must dispatch signals and feedback through the live owner host. Resolving an action on a separately constructed host or applying a binding directly outside this validation path cannot establish stale-callback rejection. There is no mutable host clone API.

## Playground specimen

The Layout story includes a dedicated `compiled_preview_gallery` component. It registers representative Text, Button and TextField factories through the public `rustui::preview` API. Controls select a specimen, apply a typed inspector property edit, rerender the parent, teardown, and remount through the specimen selector. Visible status includes owner text, mount stamp, activation count and parent rerender count. Rejection probes exercise stale callback stamps and a wrong property type.

The Button's `activate` event maps only to the explicitly allowlisted `increment` target. TextField edits feed the owner value back to the current descriptor through the captured-stamp binding. Its native control has one stable explicit ID within the mounted preview generation.

## Focused coverage and acceptance status

Source tests cover current action resolution, revision advancement, stale action/edit/binding rejection, atomic wrong-type rejection, teardown, remount generation changes and rejection of unregistered action targets. The core captured binding test accepts one matching-context write and rejects the next stale write without changing the owner value. Workspace `--lib` tests after M127 passed 335 tests across six result groups (`target/headstart/roadmap-workspace-library-tests-20261019.log`);

Formatting was run on the implementation files. The selected packages' locked/offline all-targets check passed, and the Playground build and packaging succeeded. Workspace library tests passed 335 tests across six result groups (`target/headstart/roadmap-workspace-library-tests-20261019.log`); doctests were not part of that run. Native Playground checks confirmed the three specimens mount; a typed inspector edit and two successive typed TextField edits update the owner value in the same mount (generation 1, revisions 0→14→29); a 201-character entry is rejected at the schema’s 200-character limit with both owner and field showing the accepted 200-character value and validation feedback; the recorded parent-rerender sequence reports a generation advance, but source shows that action only increments a render counter; do not attribute that transition or value preservation to a parent-only rerender; the allowlisted Button action fires once; teardown/remount advances generations; and stale callbacks and wrong property types are rejected. NavigationSplit's catalog route exposed the `Workspace navigation` outline and named `Files details`/`Settings details` containers after selection changes. M128 observed caret-position retention across parent rerender plus keyboard blur/refocus. A separate selection-retention observation in the consolidated native record confirms that `cd` remained selected through focus traversal and a parent rerender, and typing replaced that range in both the field and owner. M599 separately verifies ordinary selection retention while focus remains in the field across a no-blur parent rerender. Active IME composition, VoiceOver, and other-platform behavior remain unverified. The complete native observations and required disposable-app cleanup are recorded in `target/headstart/evidence/d1b-preview-host-acceptance-20261019.json`.

## M128 native caret-position check

In the canonical macOS Playground, the TextField was set to `abcdef`, and
the caret was positioned between `c` and `d` with Home followed by three
Right presses. The Rerender preview parent action was activated through
keyboard focus traversal. After returning to the field through Tab traversal
and typing `X`, the owner and field both showed `abcXdef` (generation 1,
revision 13; one parent rerender). This directly observes insertion-position
retention across the action and blur/refocus sequence. Selection retention
through focus traversal and rerender is recorded separately below. This does
not establish retention while focus remains in the field, IME composition,
VoiceOver, or behavior on other platforms. Evidence:
`target/headstart/evidence/d1b-preview-host-acceptance-20261019.json`.


## Native selection-retention follow-up (2026-10-19)

On macOS 27 in Layout > Stacks, set the compiled TextField to `abcdef` and select `cd` with Home, Right twice, then Shift+Right twice. Shift+Tab to Reject wrong preview type and Tab back; the selection remained visible. Activating Rerender preview parent left the range selected; typing `x` replaced it, producing `abxef` in both the field and owner. Generation remained 1 and revision reached 4; the parent-rerender count advanced from 1 to 2 during the measured action. Evidence is appended to `target/headstart/evidence/d1b-preview-host-acceptance-20261019.json`. This is one bounded macOS observation. Retention while focus remains in the field, IME composition, VoiceOver interaction, automated/race cases, and other-platform behavior remain open.


## M147 — TextField preedit reconciliation (2026-10-20)

A parent rerender could previously call `InputState::set_value` while an
operating-system IME had marked preedit text that had not yet reached the owner
binding. `TextField` now retains the last owner value beside its `InputState`
and preserves marked text only while the owner still matches that value. A real
external owner update remains authoritative during composition; normal committed
change writeback is unchanged. Six focused TextField tests and the locked/offline
all-targets check passed. This is source-level coverage, not native IME acceptance:
active composition, no-blur rerender, and native focus retention remain open.
Evidence: `target/headstart/evidence/d1b-ime-rerender-m147-20261020.json`.

### Native IME acceptance procedure

The existing Playground `Rerender preview parent` control is a focusable
button. Reaching it by click or keyboard traversal can blur the TextField and
commit or cancel the operating-system composition first, so that interaction
cannot establish the M147 no-blur rerender gate. Do not count ordinary typed
Japanese text or a rerender after leaving the field as active-composition
evidence.

The native probe must keep the compiled TextField focused while triggering a
parent rerender through a separate owner action, then record that the marked
preedit and candidate UI remain visible and that the owner value has not yet
been replaced by the preedit. Commit the composition and verify that the
committed string reaches both the retained field and owner. In a separate
active-composition run, update the owner externally and verify that the field
adopts that value instead of preserving stale preedit. Record the active macOS
input source, exact steps, visible field/owner values, focus state, generation,
revision, and app build identity. Enable the Japanese input source before the
run; if it is unavailable, leave native IME acceptance open rather than
treating source tests as a substitute.

### M271 no-blur probe trigger

The canonical compiled-preview gallery now includes a delayed owner-task trigger
for the no-blur case. Start the five-second delay, refocus the compiled
TextField, begin Japanese IME composition, and let the parent update fire while
the field remains focused. The delayed task is owned by the same Playground
component and reports completion in the existing preview status. It awaits the
GPUI background-executor timer through `OwnerTask::start_with_executor`, so the
delay does not block an executor worker. This adds a test affordance only; it
does not establish native IME acceptance. Record composition visibility, owner
value, commit propagation, focus, generation, revision, and app identity using
the M147 procedure above. The post-fix isolated locked/offline check is
`target/headstart/evidence/m271-d1b-no-blur-timer-check-20261020.log`.

### Pinned input-state source trace (2026-10-20)

The selected GPUI Components `InputState` exposes `marked_text_range`; its
underlying GPUI input handler reports the active marked range. The pinned
`InputBaseState::set_value` implementation replaces all content, resets the
selection, and clears undo history. RustUI must therefore avoid calling it
while unchanged-owner preedit is marked, and it must avoid calling it for an
unchanged value when an ordinary text selection is present. M147's predicate
does both; when composition has been canceled and the mark is gone, an
unchanged owner remains authoritative and replaces any divergent retained
value. Focused source regressions cover cancellation and the no-op path that
leaves selection state alone. There is no pinned-API blocker for this source
boundary. M599 verifies ordinary selection retention across a focused no-blur parent rerender. Active IME acceptance remains open: the direct `Rerender preview parent` button moves focus, while the M271 delayed owner-task trigger provides the no-blur probe path documented above; that trigger does not itself establish runtime composition retention.


### Live input-source availability audit (2026-10-21)

The macOS input-source inventory currently contains ABC and Arabic keyboard
layouts, with no Japanese input source enabled. The
`/System/Library/Input Methods/JapaneseIM.app` bundle is absent. The canonical Playground bundle was also
absent when checked. No system settings were changed, and the unrelated
`/private/tmp/RustUIPlayground.app` process was left untouched. Run the M147
acceptance after a Japanese input source is made available and the canonical
Playground is rebuilt. This environment audit does not replace native IME
acceptance.

### M722 — validate allowed string lengths

Component schema registration now rejects allowed string values that violate
the property's `min_length` or `max_length`, using the same Unicode character
count as imported property values. The focused locked/offline RustUI regression
passed 1/1 and covers both bounds with no default value, isolating enum
validation.

### Pinned input event boundary (2026-10-24 source audit)

The pinned `gpui-base` `InputBaseState::replace_and_mark_text_in_range`
updates the marked range and notifies the input state without emitting
`InputEvent::Change`. RustUI's retained TextField subscription therefore does
not write marked preedit into the preview owner. A committed text replacement
clears the marked range and emits `Change`; RustUI then writes the committed
value through the current binding. Programmatic `set_value` suppresses
`Change`, resets selection, and clears undo history. RustUI skips that reset
while marked text is active and the owner still matches its prior snapshot, but
applies a changed owner value even during composition. Relevant source:
`vendor/gpui-base-0.7.3/src/input/base/state.rs:1032-1059,4137-4373,4376-4506`
and `crates/rustui/src/components/input/text_field.rs:336-345,422-485,502-515`.
On macOS, `NSTextInputClient`'s `setMarkedText`, `insertText`, and `unmarkText`
callbacks route to those three `InputHandler` operations; AppKit queries the
same handler for marked and selected ranges. The GPUI input element registers
its handler only while its focus handle is focused, and the window restores and
registers that handler for each rendered frame. Candidate bounds are derived
from the marked range and selection. Source:
`vendor/gpui-pre-macos-0.3.9/src/window.rs:3370-3390,3437-3480`,
`vendor/gpui-pre-0.3.9/src/window.rs:3444-3494,5391-5404`,
`vendor/gpui-base-0.7.3/src/input/base/element.rs:3024-3032`, and
`vendor/gpui-pre-0.3.9/src/platform.rs:2129-2142`.
This establishes event and reconciliation semantics from source only. It does
not establish that native focus, marked text, or candidate UI survive the
delayed rerender; those still require the screenshots, accessibility snapshots,
and source-linked app identity listed in the run sheet.

### Cancellation after preedit replaces a selection (2026-10-24 source fix)

When marked preedit replaces a non-empty selection, an empty marked-text
replacement can cancel the preedit after GPUI has already removed the selected
text. RustUI then reconciles the unchanged owner string through `set_value`,
which restores the text but resets the caret to the end. TextField now retains
the last unmarked selection interval and reapplies it after this cancellation
reconciliation, using the public `InputState::selected_range` and
`set_selected_range` APIs. This restores the selected interval; the public setter
does not expose selection direction. The change is RustUI-only and has not been
compiled or exercised through native IME in this run. The native acceptance gate
below remains open. The restore path is at
`crates/rustui/src/components/input/text_field.rs:435-485`; the pinned
selection-range API is at
`vendor/gpui-base-0.7.3/src/input/base/state.rs:3128-3164`.

### Native IME run sheet and current preflight (2026-10-24)

This is a readiness record, not an acceptance result. Checked 2026-10-24 at
04:49 UTC on macOS 27.0 (build 26A5425a). Enabled input sources are ABC and
Arabic; Character Palette and Press and Hold are also listed as non-keyboard
input methods. Japanese Kana and Romaji, Korean, SCIM, TCIM, and TYIM bundles
are installed under `/System/Library/Input Methods`, but no CJK source is
enabled. The main-checkout debug bundle exists at
`target/headstart/debug/RustUI Playground.app` (bundle ID
`dev.rustui.playground`, version `0.1.0`, build `1`, modified 2026-10-21 at
21:59:55 UTC); its executable SHA-256 is
`cfc31e421f01b9ef1c4ae31192479652b211ab3c06ad38c7a4d4408b65bb21b6`. No
provenance currently links that executable hash to the current worktree source
snapshot. Three Playground instances were already running across the main
checkout and worktrees, so the shared GUI slot was occupied. No app was launched
and no input-source settings were changed for this preflight. Native IME runs
remain blocked until an approved CJK source is enabled, the shared GUI slot is
available, and the tested app has source-linked build identity.

This current inventory supersedes the 2026-10-21 note above about the Japanese
bundle being absent. That check found no bundle at the `JapaneseIM.app` path;
this inventory found the separate Kana and Romaji bundles. Existing evidence
does not close the native gate: the M147 record sets `native_ime_reproduction`
to `false`, M599 covers ordinary selection retention, and the M271 log is a
source/build check rather than an IME run.

Once available, fill one record per run; use the screenshot for preedit and
candidate visibility, and the accessibility tree for focused control name,
field value, owner summary, preview stamp, and rerender status. There is no
Playground IME event log or accessibility mark-range oracle. Internal
`marked_text_range` is source state, not runtime acceptance evidence.

| Record | Run A: rerender and commit | Run B: rerender and cancel | Run C: external owner update |
| --- | --- | --- | --- |
| Date/time and operator | _fill_ | _fill_ | _fill_ |
| macOS version/build and active input source | _fill_ | _fill_ | _fill_ |
| App bundle ID, executable SHA-256, source/build provenance | _fill_ | _fill_ | _fill_ |
| Exact steps and any focus transitions | _fill_ | _fill_ | _fill_ |
| Initial focus, field/owner values, generation/revision, rerender count | _fill_ | _fill_ | _fill_ |
| Preedit text and candidate UI before delayed update | _fill_ | _fill_ | _fill_ |
| Focus, values, stamp, count/status after delayed update | _fill_ | _fill_ | _fill_ |
| Final action and focus/field/owner values | commit: _fill_ | cancel route: _fill_ | owner-write action: _fill_ |
| Screenshot/accessibility evidence paths and cleanup result | _fill_ | _fill_ | _fill_ |
| Result and failure notes | _fill_ | _fill_ | _fill_ |

**Pass criteria:** In A and B, the field remains focused and preedit/candidate
UI remains visible across the delayed parent rerender; the owner keeps its last
committed value while preedit is marked; a parent-only rerender advances the
rerender count without changing preview generation or revision; then commit
writes the committed text to field and owner and advances the preview revision,
or cancellation restores the starting field and owner values without committing
preedit. Record the actual native cancellation route: an empty marked
replacement clears preedit, while `unmark_text` alone does not restore the prior
text. In C, specify a focus-preserving owner-write action before running; the
M271 timer only rerenders and cannot perform this update. The current canonical
gallery exposes the ordinary `Rerender preview parent` button and
`Schedule no-blur parent rerender (5 seconds)` task, but neither writes the
owner value. No focus-preserving owner-write trigger is currently exposed by
the gallery, so C is not runnable with its existing controls. Record the actual
trigger and how it preserves TextField focus before attempting C; do not count
the inspector or either rerender control as a substitute without evidence that
the owner update occurs while focus remains. The new owner value must replace
preedit in the field while focus remains. Do not mark a run passed if focus or
composition is lost before its required checkpoint. Source:
`apps/rustui-playground/src/stories/compiled_preview_gallery.rs:7389-7409`.

### Live host decision refresh (2026-10-24 07:45 UTC)

Read-only check on macOS 27.0 (build 26A5425a). `AppleEnabledInputSources`
contains ABC and Arabic keyboard layouts, plus Character Palette and
Press-and-Hold input methods; `AppleSelectedInputSources` contains ABC and
Press-and-Hold. No Japanese or other CJK keyboard source is enabled. The Kana
and Romaji typing input-method bundles remain installed under
`/System/Library/Input Methods`.

The shared GUI slot remains occupied by three Playground processes: PID 10507
from the main checkout, PID 23568 from worktree `c611`, and PID 36564 from
worktree `66c9` (validation app). No app was launched, stopped, or interacted
with. The main-checkout executable remains timestamped 2026-10-21 21:59:55 UTC
with SHA-256
`cfc31e421f01b9ef1c4ae31192479652b211ab3c06ad38c7a4d4408b65bb21b6`; it
predates this source change, and the existing record has no provenance linking
it to the current source snapshot.

**Decision/prerequisites:** the host owner must free the shared GUI slot and
enable/select an installed Japanese Kana or Romaji input source; then build the
Playground from the current source and record source-linked executable
provenance before runs A or B. Run C also requires a focus-preserving external
owner-write trigger in the gallery; the current delayed task only rerenders.
This check changed no input-source settings and performed no UI interaction.
