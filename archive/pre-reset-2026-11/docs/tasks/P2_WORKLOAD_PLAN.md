# P2: Bounded RustUI Playground workload and measurement plan

**Status: proposal only.** This document defines candidate workloads and a
measurement protocol for future review. It reports no measured performance,
does not authorize running a benchmark or implementing instrumentation, and
does not enable GPUI Fast, change a dependency, or approve a patch. GPUI Fast
remains disabled. Any upstream revision or patch requires its separate approval;
advanced GPU work and native composition remain outside this plan.

## Current Playground evidence and limits

The native Playground is `apps/rustui-playground`, bootstrapped from
`apps/rustui-playground/src/stories/mod.rs`. Its current stories and component implementations
are in `src/stories/component_catalog.rs` and `src/stories/canvas.rs`. In particular:

- **Layout / retained children:** `src/stories/child_component.rs` defines a child-owned
  counter and notes editor; `src/stories/canvas.rs` creates Alpha and Beta under
  separate component identities and passes a parent-updated title; `src/stories/mod.rs`
  exposes title changes, keyed row reorder, and Beta removal/restoration. This
  is the smallest existing stateful workload that exercises parent update,
  child-local update, stable identity and reorder.
- **Text editing:** the Input story in `src/stories/canvas.rs` includes retained text
  fields and a `TextEditor`; the Tabs and NavigationStack stories provide
  retained-editor lifecycle exercises documented in `docs/RUSTUI_LAB.md`.
- **Virtualized collections and scrolling:** List, Table and Tree stories are in
  `src/stories/canvas.rs`; the Playground shell and story preview use scroll views in
  `src/stories/mod.rs`. The present data is small, so this is not yet a large-data
  workload.
- **Idle:** no deterministic idle measurement mode or repeatable capture window
  is documented.
- **Drag:** no controlled drag workload is implemented in these stories. Do
  not infer one from historical GPUI Fast or UI Studio references. Include drag
  only if a future scoped Playground story actually supports the gesture and can
  capture it reproducibly.

The external process collector and summary tools are implemented (Milestone 12).
They measure process CPU/RSS and sampling overhead; app cycle/frame/main-thread
timing remains unavailable. `scripts/record_build_provenance.py` records declared
build facts, not proof that a sampled process loaded that binary.

## Proposed bounded workload set

Use existing Lab stories, not a new component catalog. Start with **W1** as the
first representative workload; add W2/W3 only after W1 is stable and the
measurement path is reviewed.

| ID | Scenario | Fixed action sequence | What it exercises |
| --- | --- | --- | --- |
| W1 | Layout: retained child update and reorder | Open Layout once; with Alpha and Beta present, increment each child once; alternate the parent title prefix on/off; reverse order and restore it. Repeat this fixed cycle. | Parent-driven child update, independent child state, keyed ordering, retained layout/render work. |
| W2 | Text editing and retention | Open Input; focus the Notes editor, replace a fixed short string, then repeat with a fixed insertion/deletion sequence. Separately use the existing Tabs retention exercise: edit Overview and Details, switch tabs, remove/restore Details. | Text editing, binding notifications, retained editor state and view lifecycle. |
| W3 | Virtualized list scroll and selection | Open List; scroll through a fixed sequence of viewport positions and select fixed source rows. Current catalog has seven rows; larger fixed data is a future harness extension, not current evidence. | Existing virtualized List path, scroll and selection updates. |
| W4 (conditional) | Active drag | Define only after a Playground story provides a deterministic draggable item and resettable geometry. Record pointer path, duration, and overlay/content state. | Drag invalidation/retention under an actually supported Lab interaction. |

W1 is the first candidate because its present implementation is bounded and
already combines state updates, child ownership, title changes and reorder. It
does not represent large collections, text shaping breadth, scrolling, or drag;
those require their own later scenario. Keep W1 data at exactly two children so
the first comparison does not conflate a new scale workload with child-retention
behavior.

## Proposed controlled environment

Initial platform: native macOS on one named Apple Silicon machine, plugged into
power, with the same display and refresh configuration for paired baseline and
candidate captures. Record machine model, macOS version, architecture, display
resolution/scale/refresh, power state, and relevant foreground/background
conditions. Do not pool results across machines or platforms.

For every run, record and hold constant:

- Playground viewport: window content size 1320 x 820 logical points, same display
  scale, window position and split/panel arrangement. If that exact size is not
  supported, record the chosen size and use it unchanged for both sides.
- Appearance: macos27 Light for initial acceptance; Dark is a separate scenario,
  not mixed into a sample.
- Locale/direction: English (en, left-to-right) initially; French and Arabic
  are separate follow-up scenarios because text and direction can alter layout.
- Input: fixed W1 sequence above, one native input method, no unrelated typing,
  pointer motion, window resizing, or application interaction during capture.
- Build: same RustUI workspace inputs, selected profile/features/flags, target
  triple, launcher/toolchain, and approved GPUI/Components snapshot. Capture
  exact source/worktree identity, dirty state, lockfile hash, compiler/Cargo
  versions, profile, effective flags and dependency identities.

Cold and warm **build** states are provenance labels only: define cold as a
separate empty candidate target directory and warm as a repeated build using the
same unchanged source/toolchain/configuration and target directory. Record build
duration and Cargo freshness separately if collected. These labels do not enter
runtime samples and do not establish runtime speed. Runtime launches should use
the same already-built executable and the same launch procedure for both sides;
record whether the OS/app process is at a first launch or a repeated launch.

## Correctness before performance

Before collecting a performance sample, manually establish the W1 baseline state
and verify the visible result: both children start with the expected titles,
their counters update independently, each title-prefix change leaves counter
and notes state intact, keyed reorder keeps each child's state with its identity,
and removal/restoration follows the documented child lifecycle. Record each
check as pass, fail, or not run, with reviewer and evidence reference. Any
correctness failure invalidates the performance comparison. Compilation alone
does not satisfy this check. No visual automation or GUI acceptance hook is
claimed to exist.

## Proposed runtime capture protocol

Instrumentation is a missing prerequisite. A future minimal harness should
measure externally or through a narrowly scoped Lab facility whose overhead is
measured and disclosed; it must not introduce a second reactive or rendering
runtime. Prefer native macOS process metrics and GPUI-owned frame signals if
available through reviewed APIs. Do not claim frame time/FPS data until a real
frame boundary source has been identified and validated for the pinned stack.

For each scenario and each baseline/candidate executable:

1. Launch, wait until the window and selected story are visually settled, then
   run five unrecorded warm-up cycles.
2. Capture 30 repetitions of the fixed action cycle, with a one-second quiet
   interval between repetitions. Keep the same capture duration and reset state
   before each repetition. W1 cycle timing begins immediately before the first
   increment and ends after the final restored order has rendered and settled.
3. Repeat the paired comparison in alternating order (baseline/candidate,
   candidate/baseline) across at least three fresh app-process launches. Keep
   each raw repetition; do not discard outliers without a recorded cause.
4. Report per-process and per-repetition values, sample count, median, p95 and
   spread. For paired comparison, report candidate/baseline ratios and their
   uncertainty; do not rely on a single best run.

Metrics, where the capture mechanism supports them:

- **Primary:** end-to-end W1 cycle wall time and main-thread busy time during
  the cycle. Report median and p95 per cycle. A main-thread attribution method
  must be named; otherwise mark unavailable.
- **Frame responsiveness:** frame interval median/p95 and missed/long-frame
  count only when a validated frame callback/profiler source exists. Do not
  substitute wall time for frame time or label a result as FPS without observed
  presented frames.
- **CPU:** process CPU time and utilization over the same fixed active and idle
  windows, separately labeled. Identify the collection API and interval.
- **Memory:** resident/physical memory at settled idle and peak during the action
  window, using one named native process metric. Record sampling interval and
  avoid comparing incompatible definitions.
- **Idle:** after settling for 60 seconds, collect a further 60-second idle
  window. Report process CPU and memory only if available; no synthetic idle
  frames are assumed.

If frame/main-thread instrumentation or reliable memory sampling is unavailable,
mark that metric `unavailable` and continue only with the defined measurable
metrics. If fewer than 30 valid repetitions or three launches complete, mark the
comparison `inconclusive`, retain raw data, and do not assert a pass. If run
variance is high (coefficient of variation above 10% for the primary measure),
inspect conditions and repeat the paired capture; if it remains above 10%, label
inconclusive rather than relaxing the threshold. Report confidence intervals
from paired launch-level samples (bootstrap 95% interval is a proposed method);
if the interval crosses the acceptance boundary, result is inconclusive.

## Proposed initial acceptance thresholds

These values are **PROPOSED — NEED BASELINE CALIBRATION AND APPROVAL**. They are
not measured facts or an existing CI gate. First collect an unchanged official
path baseline under this protocol, inspect repeatability and metric validity,
then approve or revise the thresholds before comparing any optimization.

- Correctness: zero observed state, identity, content, or visual correctness
  failures for the defined sequence; any failure rejects the candidate regardless
  of speed.
- Primary W1 cycle time: require at least a 10% median reduction to claim a
  meaningful improvement; reject a candidate for a median regression greater
  than 5% or a p95 regression greater than 10%. Values between those boundaries
  are inconclusive unless a later approved workload decision defines otherwise.
- Main-thread busy time: no more than 5% median regression; a claimed
  improvement requires at least 10% median reduction and an uncertainty interval
  that excludes no change.
- Frame responsiveness, if validly measurable: no more than 5% p95 frame interval
  regression and no increase in long-frame count. Missing instrumentation means
  no frame acceptance claim.
- Memory: no more than 5% increase in settled-idle resident memory and no more
  than 10% increase in peak resident memory. Also record absolute bytes; ratios
  alone can hide small baselines.
- Idle CPU: no more than 5% increase in mean process CPU over the fixed idle
  window; if the measured baseline is near zero or the metric resolution is too
  coarse, classify it as inconclusive.

Apply thresholds to paired candidate/baseline results with the uncertainty rule
above. A metric that is noisy, unavailable, methodologically incompatible, or
not sampled remains inconclusive. Do not average unlike scenarios into one score.

## Evidence and implementation boundary

Keep two distinct evidence records:

1. **Build/source provenance:** source tree and dirty state, lockfile and
   dependency identities, compiler/toolchain, flags/profile/features, build
   command/log, and cold/warm Cargo artifact freshness. Existing provenance and
   reuse scripts can support parts of this record, within their documented
   limits.
2. **Runtime run record:** workload ID/version and action parameters, correctness
   result, machine/display/appearance/locale, executable/source identity, launch
   and warm-up details, metric methods, all raw samples, summaries, exclusions,
   uncertainty and outcome. The process schema/collector now exists; workload cycle/completion evidence remains pending.

Raw evidence should be written under an isolated run identity, for example
`target/headstart/performance/<run-id>/`, without overwriting prior results. For
upstream candidates, pair the same workload with P3's isolated source and target
under `target/headstart/candidates/<candidate-id>/`; do not reuse accepted vendor
files or the normal target cache for candidate work. This is a proposed path,
not an existing pipeline.

The next implementation is limited to deterministic W1 scenario/reset entry and
workload completion/correctness records in Playground stories. The external
process collector is already implemented. Frame and main-thread timings remain
unavailable following the pinned GPUI hook review. Add no FPS claim, large dataset,
drag story, new dependency, or RGPE caching based on this plan alone.

This document defines a bounded proposal for later review. It does not authorize
implementation, a GUI run, profiling, benchmarking, dependency fetch, or patch
validation. P3 must use the accepted version of this workload plan and its
calibrated thresholds before any controlled candidate comparison.


## Milestone 12 implementation update — 2026-10-18

This update supersedes earlier statements that the process collector/schema are
missing or a comment-only scaffold. The external collector and per-run summarizer
now exist: `scripts/collect_runtime_metrics.py`, `scripts/runtime_metrics.py`, and
`scripts/summarize_runtime_metrics.py`. The old future-collector scaffold is gone.
Seventeen Python tests passed; a five-sample idle smoke is recorded. This is not
a W1 baseline and provides no frame, cycle, or main-thread timing.

The canonical bootstrap is `apps/rustui-playground/src/lib.rs`, with stories under
`apps/rustui-playground/src/stories/`. Lab is a compatibility launcher. The current
window is 1320 x 820 logical points (correcting the earlier 1280 x 800 proposal).
Use one fixed, recorded viewport for future paired measurements.

Next implement W1 reset and a deterministic action sequence with completion and
correctness records. The proposed 30-cycle/three-launch protocol, presentation
timing, calibrated baseline, and numerical acceptance thresholds remain pending.
Proposed thresholds are not approved gates. See `P2C_GPUI_TIMING_HOOKS.md` before
selecting any future timing adapter; keep it inside RustUI's backend boundary.

## Milestone 13 scenario-runner update

W1 now has Reset and manually paced Next Step controls in Playground Layout, with
expected values, parent-state checks, stdout dispatch records, and explicit
operator visual results. See `P2D_W1_WORKLOAD.md`. This supersedes the proposed
scenario/reset implementation above. The six actions use props-driven increment
commands rather than native input simulation. It is not an automated workload or
measured completion boundary. Native acceptance, automatic scheduling, child
readback, repeated baseline, and calibrated thresholds remain pending.

## Milestone 14 calibration completed

The bounded W1 compact-view workload now has native visual acceptance, a saved
CUA driver, three fresh-launch runs of 30 cycles each, idle controls, and a
CPU/RSS repeatability calibration. See `P2E_W1_ACCEPTANCE_BASELINE.md`. This
supersedes earlier statements that W1 automation, native acceptance or the initial
CPU/RSS baseline are missing. The actual protocol uses three warm-up cycles per
launch; proposed numerical regression thresholds have not become approved gates.
Frame/presentation, GPU and main-thread timings remain outside this calibration.
