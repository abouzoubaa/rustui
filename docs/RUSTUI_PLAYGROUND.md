# RustUI Playground

`apps/rustui-playground` is the canonical component development and manual
acceptance application. Its application crate consumes the public `rustui` API;
GPUI and GPUI Components dependencies stay inside the framework implementation.
The `rustui-lab` package is a compatibility launcher that delegates to the same
Playground entry point.

## Launch

```sh
./scripts/cargo-headstart playground
./scripts/cargo-headstart dev
./scripts/cargo-headstart lab
```

`playground` is the standard alias. `dev` uses the shared Headstart profile. `lab`
keeps the previous command available while launching the same application. See
[Build workflow](BUILDING.md) for compiler, cache, and evidence rules.

## Shell and references

The native window is 1320 × 820, with a 250-pixel story sidebar and a 300-pixel
reference pane. Startup loads `themes/macos27.json` in Dark appearance; the shell
provides Light/Dark controls. Its catalog retains the exact 41 component keys from
the theme document and links to the corresponding macOS 27 SVG reference entries.
Catalog status records baseline API availability separately from full platform
widget support. A reference asset or baseline adapter is not visual acceptance.

## Live stories

The app currently provides 15 live RustUI story routes:

- Button
- Text
- Layout
- Input
- List
- Table
- Tree
- Tabs
- Breadcrumb
- Navigation
- Outline
- Drawer
- GraphView
- NodeEditor
- SplitView + ResizablePanel

The shell currently lists 46 catalog rows. Search matches story labels, titles,
and summaries; searching either `SplitView` or `ResizablePanel` opens the shared
public-API split gallery directly, as well as its original Stacks foundation tab.

The selected story can show RustUI source and recent event activity. Locale controls
switch among English, French, and Arabic. These views exercise the current public
RustUI APIs and do not expose GPUI types from the application crate.

## Current limits and verification

Layout inspection is partial. The performance panel and advanced component
inspectors remain incomplete. Each catalog row describes available API coverage;
it does not claim complete native platform behavior.

Full-application behavioral, visual, and accessibility acceptance remains pending
for this migration. The focused builds, story checks, and GUI reviews recorded below
cover only their stated scenarios; a compiled application or catalog entry alone is
not evidence that broader acceptance passed.


## Consolidation verification — 2026-10-18

The workspace all-target check, Playground executable build, five targeted tests,
formatting and migrated locale validation passed. Structural checks confirmed only
`rustui` as a direct application dependency, no direct GPUI API paths, unchanged
upstream lock identities, the original 41 catalog keys and unchanged 31 SVG files.

A native review window was opened and inspected through CUA. Button increment,
framework-story selection, child title/environment updates and keyed reorder were
exercised. Alpha retained count 1 and Beta retained count 0 after reorder. Preview
scrolling reached the child panels and source display. This is a smoke check, not
full interaction/accessibility or pixel-perfect visual acceptance. Theme switching,
locale switching and editing/removal acceptance remain pending in this record.

The canonical bundle is `target/headstart/debug/RustUI Playground.app`; the separate
review bundle under `target/headstart/verification/` runs the same built executable.
See the consolidation evidence in `target/headstart/evidence/` and the roadmap.

## Quit controls — M23 native review

The macOS Playground build succeeded. Native review confirmed that Cmd+Q and the
menu Quit command each terminate the owned application process; the corresponding
PIDs were absent in `ps` afterward. Theme and locale controls were present in the
reviewed window. See `target/headstart/milestone23-quit-gui.json` and the
Milestone 23 roadmap entry. Other platforms and closing the window were not
verified by this review.
## External measurement tools (Milestone 12)

Use `scripts/collect_runtime_metrics.py --pid PID` to observe a running
Playground and `scripts/summarize_runtime_metrics.py` to report its CPU/RSS and
sampling overhead. See `BUILDING.md` for commands and unique evidence filenames.
The collector adds no application dependencies and does not control the app.

The recorded five-sample, one-second idle smoke confirms tool operation only.
A deterministic W1 driver/reset, action completion records, repeated baseline,
and calibrated thresholds remain pending. Callback cadence is not presented FPS;
frame/cycle/main-thread timings remain unavailable. GPUI Fast remains disabled.

## W1 retained-child scenario

In Layout, use **Reset W1**, then click **Next W1 step** six times, inspecting
Alpha/Beta after each step. Expected counters are shown. The sequence increments
each child, changes and restores the title prefix, then reverses and restores
order. Reset clears child counters and notes by using fresh keys. Avoid other
manual controls during a run.

**Mark visual pass/fail** is an operator assertion, not automatic validation.
Dispatch records appear in the event panel and as stdout JSON lines. Capture
stdout from a terminal launch for persistent evidence. No cycle/frame timing is
measured; performance comparisons still need a calibrated repeated baseline.
See `tasks/P2D_W1_WORKLOAD.md` for the exact contract and current validation limits.

## W1 acceptance and baseline completed

Reset W1 now displays both children side by side so controls, counters and notes
fit together. Native visual checks passed, followed by three fresh launches with
three warm-up cycles and 30 measured cycles each: **90 cycles / 630 state
observations passed**. CPU/RSS calibration and its limits are recorded in
`tasks/P2E_W1_ACCEPTANCE_BASELINE.md`. This supersedes the earlier pending W1
acceptance/baseline notes; it does not establish blanket application acceptance.

## Current functional checkpoint — 2026-10-26

The production-consolidation run records 55 bounded Web functional checks across
Outline, Drawer, GraphView, NodeEditor and SplitView/ResizablePanel, plus 11 native
SplitView/ResizablePanel checks. These are named state/lifecycle/interaction cases,
not full component or application acceptance. The canonical
[remaining-work record](tasks/ROADMAP_REMAINING_AUDIT.md) links per-component
contracts, exact build-input and executable provenance, excluded attempts, and
cleanup records. Accessibility and VoiceOver are explicitly undone and deferred
by user direction; existing behavior and previously gathered evidence remain.
