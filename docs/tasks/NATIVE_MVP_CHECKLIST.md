# Proposed macOS developer-preview MVP checks

## Current scope — 2026-11-05

The user clarified that true native anchored popup windows are required for the
RustUI MVP. RustUI's existing `Popover` is an in-window overlay; unpatched pinned
GPUI 0.3.9 rejects `WindowKind::AnchoredPopup` on macOS. The approved v4 patch
adds native backend support in build paths that apply it. This adds one
acceptance check and three evidence milestones. Historical entries below retain their
dated totals; the latest detailed audit records 23/34 evidence
milestones (67.65%) and 2/11 acceptance checks (18.18%). See the current R15
evidence in `ROADMAP_REMAINING_AUDIT.md`.

## Current-candidate combined controls — 2026-11-04

The rebuilt public-API consumer passed a keyboard flow across TextField,
Checkbox, Toggle, Stepper, Slider, Picker, Radio, and Button. It verified bound
owner updates, Stepper lower-bound traversal, Slider adjustment, Picker and
Radio selection, action dispatch, and lock/disabled skip/re-enable behavior.
The disabled Checkbox ignored pointer activation. A first batched Checkbox key
attempt was inconclusive until focus was observed; the properly sequenced
keyboard retry passed. Exact observations and executable hash are in
`ROADMAP_REMAINING_AUDIT.md`. This remains partial evidence: other control/group
policies and the complete basic-controls check remain open. MVP stays 67.74%
(21/31 evidence milestones) and 2/10 acceptance checks (20%).

## Current-candidate native text editing — 2026-11-04

On the rebuilt combined-controls consumer, Tab focused TextField; selecting all
and replacing the text updated both the field and owner summary. Inserting at
the beginning updated both again. Tab moved to Checkbox and Shift-Tab returned
to TextField. Evidence: `ROADMAP_REMAINING_AUDIT.md`. This closes only the
bounded current-candidate editing check. MVP totals remain 21/31 evidence
milestones (67.74%) and 2/10 acceptance checks (20%); the complete basic-controls
workflow remains open.

## Latest native retest — 2026-11-04

R11/R15 Checkbox, Toggle, TextField, and Radio dynamic-disable recovery passed
in the earlier rebuilt native consumer run. A follow-up found and repaired
focus loss for dynamically disabled Picker and Slider. In a fresh consumer
build, each control received keyboard focus, Toggle lock disabled it, Tab
focused Toggle lock, and Space unlocked the controls. The five focused
GPUI Root regressions passed, including both new cases. See
`ROADMAP_REMAINING_AUDIT.md` for the failure, repair, test log, and executable
hash. These bounded paths do not complete the basic-controls workflow. Totals
remain 21/31 evidence milestones (67.74%) and 2/10 acceptance checks (20%).

## Earlier Checkbox-only result — 2026-11-04

R11/R15 dynamic-disabled Checkbox recovery now passes a Root-backed actual
Tab/Shift-Tab regression and a rebuilt native consumer check: after a focused
Checkbox is disabled, Tab reaches the enabled Toggle lock button and Space
unlocks the controls. See `ROADMAP_REMAINING_AUDIT.md`. This closes that
bounded defect only; the complete basic-controls workflow remains unchecked.
Overall totals after the popup-window scope update are 21/34 evidence
milestones (61.76%) and 2/11 acceptance checks (18.18%).

This is a provisional acceptance checklist for the macOS developer preview
discussed with the user. It does not replace the full production roadmap or
assert that the user has approved a publication strategy. Count a check only
when all of its stated evidence exists. The checks differ in size, so their
unweighted count is not a measure of code written or time remaining.

**Current count: 2/11 checks complete (18.18%).**

## Granular progress baseline after popup-window scope update — 2026-11-04

**MVP 61.76%: 21 of 34 evidence milestones below have been demonstrated.** This
unweighted count exposes progress inside the larger checks. It is not an
estimate of effort, calendar time, coding completion, or production readiness.
The milestones differ in size; some record evidence on an earlier revision
and need repetition on a release candidate. The 2/11 acceptance count above
remains separate and closes only when each complete requirement is satisfied.
Future updates should report both counts, the percentage rounded to two decimal
places, and the milestone that changed.

The scope is a proposed macOS developer preview. This does not remove the
broader production roadmap or decide the release strategy. Evidence references
below identify sections in `ROADMAP_REMAINING_AUDIT.md`.

1. **Separate local consumer: 3/3**
   - [x] Build an outside-checkout consumer using the public API (R03/R04 native source-consumer probe).
   - [x] Run its native button action (R03/R04 native source-consumer probe).
   - [x] Run a bound text input and observe the owner update (R03/R06 external consumer bound input).
2. **Repeatable local setup: 3/3**
   - [x] Provide a generator for a separate consumer workspace (R03/R04 repeatable local consumer setup).
   - [x] Refuse to overwrite an existing destination (same section).
   - [x] Build a generated fixture offline from a path containing spaces (same section).
3. **Window lifecycle: 2/3**
   - [x] Observe native root launch, close and owner unmount (R04/R05 native root close unmount).
   - [x] Observe cancellation of pending owner work on close (R04/R05 native pending task root close).
   - [ ] Verify actual platform window-open failure handling and broader lifecycle cases end to end. The existing error injection is partial evidence.
     Partial progress: on current candidate `71c8a4c7`, a fresh internal
     framework probe reached the real unsupported-popup backend error. The
     custom handler received it and the default handler logged it and quit;
     both runs exited successfully. This confirms that failure mode only.
     Other platform failure and combined lifecycle cases remain open. Logs
     and executable identity are in `ROADMAP_REMAINING_AUDIT.md`.
4. **Basic controls and input: 2/3**
   - [x] Demonstrate native text binding with owner state (R03/R06 external consumer bound input).
   - [x] Demonstrate Stepper bounds and enabled/disabled owner updates (R06/R15 native combined form coordination).
   - [ ] Complete the documented MVP control set, disabled states, keyboard navigation and ordinary text-editing workflow on a current candidate.
     Partial progress: R06 native consumer keyboard editing verifies selection,
     replacement, cursor editing and Tab navigation between the sample's field
     and button. R06/R15 native combined form keyboard flow additionally verifies
     disabled Stepper traversal and keyboard operation of Toggle and Stepper;
     R15 native disabled-control traversal checks disabled Checkbox, Toggle and
     Stepper in both Tab directions and re-enabled operation. R15 native disabled
     TextField owner update covers disable, external owner update and re-enable.
     R15 native Slider and Picker keyboard flow covers disabled traversal,
     keyboard adjustment and selection, and re-disabling. R11/R15 native Radio
     and Button disabled flow covers disabled clicks, traversal and re-enabling.
     R15 native owner-controlled Radio selection covers one exclusive-binding
     pattern; other control and group policies remain open. R11/R15 combined
     native controls exercises eight control types together with shared state,
     disabled pointer input and one re-enabled keyboard transition. The complete
     control set and keyboard workflow remain open. R15 Stepper boundary focus
     recovery now passes rendered keyboard regression and bounded native checks
     at both limits; broader controls and modal acceptance remain open.
5. **Navigation and overlays: 2/3**
   - [x] Demonstrate native unknown-route recovery (R17 native unknown-route keyboard recovery).
   - [x] Demonstrate a native Drawer interaction (R19/R33 native Drawer interaction).
   - [ ] Integrate and verify lost-focus recovery for Sheet and Dialog and complete the chosen navigation/overlay workflow.
6. **Theme and motion: 3/4**
   - [x] Observe native light and dark switching with preserved state in one story (R26 current native appearance and build diagnostic).
   - [x] Verify normal opacity scheduling and quiescence in the native probe (R08 native opacity scheduling).
   - [x] Verify the probe's reduced-motion scheduling (same section).
   - [ ] Verify the supported theme and ordinary-motion paths across the selected MVP workflow, including visual behavior.
7. **Trusted native CI: 2/3**
   - [ ] Make a trusted runner with the required macOS labels available.
   - [x] Run native-focused source checks for the selected candidate in trusted CI (R30 native source CI result). The full source job remains open.
   - [x] Run compiled macOS checks for that exact candidate in trusted CI (native-only smoke: workspace all-targets check and core tests, R30 native CI smoke result). The full release job remains open.
8. **Distribution: 2/3**
   - [x] Package a real native Playground with collected notices (R31 real Playground package reuse).
   - [x] Compare two bundles made from the same binary and notice collection on one host (R30/R31 native package comparison).
   - [ ] Select, document and independently verify the distribution workflow outside the development checkout.
9. **Dependency notices: 2/3**
   - [x] Collect an inventory for the native build (R31 real Playground package reuse).
   - [x] Determine whether the five missing-notice packages occur on normal macOS dependency paths (R31 native missing-notice dependency paths).
   - [ ] Resolve the five notice gaps and complete license sufficiency review.
10. **Supported macOS release: 0/3**
    - [ ] Decide and verify the supported macOS versions.
    - [ ] Decide and verify the release packaging route.
    - [ ] Verify signing and notarization if required by the selected route, or document why they do not apply.

11. **Native anchored popup windows: 2/3 evidence milestones complete**
    - [x] Implement macOS `WindowKind::AnchoredPopup` against the approved pinned GPUI source through a reproducible, versioned patch; preserve the pinned revision.
    - [x] Expose `open_anchored_popup` through RustUI and return GPUI's
      `AnyWindowHandle` for explicit close; retain the in-window `Popover`
      path for non-native use.
    - [ ] Verify current-candidate placement, dismissal, parent teardown, and owner-state updates in a native macOS app.

1. **Complete — separate local consumer:** an app outside the checkout builds
   using the public API and responds to native button and bound text input.
   Evidence: R03/R04 and R03/R06 entries in the remaining-work ledger.
2. **Complete — repeatable local setup:** a documented generator creates that
   workspace without replacing an existing directory; its fixture and an
   offline build from a path with spaces passed. Evidence: R03/R04 local
   consumer setup in the ledger and `examples/external-consumer/README.md`.
3. **Open — window and lifecycle:** current-candidate native launch, close,
   error handling, owner teardown and cancellation are verified end to end.
   Bounded native probes confirm root unmount, cancellation of one pending
   timer-backed task, suppression of one in-flight native result, and dispatch
   of an injected window-open error. Actual platform failure and broader
   lifecycle cases still need evidence.
4. **Open — basic controls and input:** the supported MVP controls, bindings,
   disabled states and ordinary keyboard/text editing pass a documented
   current-candidate native workflow. Bounded native checks cover Stepper
   endpoints and a combined TextField, Toggle and Stepper owner-update flow;
   the remaining control paths still need evidence.
5. **Open — navigation and overlays:** the chosen navigation and modal paths
   pass the native workflow, including recovery after focus loss. In
   particular, an unfocused Components Sheet or Dialog cannot yet regain
   keyboard focus through Tab.
6. **Open — theme and ordinary motion:** the supported light/dark theme and
   animation paths are verified on the current candidate.
7. **Open — current native CI:** the chosen source and compiled macOS checks
   pass on the exact candidate in the trusted CI environment.
8. **Open — distribution workflow:** the user-selected way to obtain RustUI is
   documented and verified independently of the development checkout. The
   local source workflow above does not satisfy an installed SDK requirement.
9. **Open — dependency notices and review:** the five native missing notice
   files and license sufficiency are resolved for the candidate. All five
   appear on normal macOS Playground dependency paths; linked-binary contents
   and the legal obligations remain to be determined.
10. **Open — supported macOS release:** the supported OS/version scope and
    package workflow are decided and verified; signing and notarization are
    verified if required for the chosen distribution route.
11. **Open — native anchored popup windows (2/3 evidence milestones):** RustUI's
    versioned macOS patch and public API/demo are implemented, but current-candidate
    native placement, dismissal, parent teardown, and owner-state updates remain
    unverified. See the R15 entries in `ROADMAP_REMAINING_AUDIT.md`.

Localization, Web/Wasm, accessibility and VoiceOver remain deferred by the
user. They are not counted as passed and remain required wherever the final
product scope calls for them. The production-readiness objective remains open.
