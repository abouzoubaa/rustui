# L1: RustUI Playground acceptance roadmap

RustUI Playground is the canonical component development application. The
`rustui-lab` executable is a compatibility launcher that delegates to
Playground; it does not own a second story implementation. Playground currently
has a searchable component catalog and ten framework demos. Native acceptance
is partial and remains open.

## Current state and evidence

`apps/rustui-playground/src/lib.rs` owns the Playground shell and its
interactions. `component_catalog.rs` supplies the catalog and framework demo
registry; rendered demos live under `apps/rustui-playground/src/stories/`.
`target/headstart/evidence/playground-l1-native-20261021.json` records the
bounded native observations made in the M56 Review app. It records partial
results only; it does not close this acceptance roadmap.
The follow-up no-match catalog search and clear-query recovery observation is
recorded in `target/headstart/evidence/m535-l1-catalog-search-native-20261021.json`;
small-window scrolling and the other listed native gates remain open.

The Playground build completed before the native pass
(`target/headstart/evidence/m534-playground-build.log`). Build success is
supporting compile evidence, not GUI acceptance.

## Snippet status

The displayed source fragments are illustrative strings, not independently
compiled examples. The Breadcrumb fragment follows the current story wiring:
the Playground owns the passed `Action` values and clones them into actionable
items, while the current page is supplied as state. Keep other fragments
aligned with the rendered stories as those stories change.

## Work breakdown

Priority order is native acceptance evidence first, then fixes for defects that
were actually observed. Candidate files are paths for follow-up, not claims of
acceptance.

| Priority | Work item | Candidate files and dependency | Risk |
| --- | --- | --- | --- |
| P0 | Run and record a repeatable native-window acceptance pass for all ten framework demos, interactions, and failures. | `apps/rustui-playground/src/lib.rs`, `component_catalog.rs`, and `stories/`; requires a runnable macOS GUI session and a written observation record. | High: compile success cannot establish native event, focus, or render behavior. |
| P1 | Verify hover, keyboard focus, activation, and disabled behavior for buttons, inputs, tabs, breadcrumb actions, and navigation links; fix only observed defects. | Relevant files under `apps/rustui-playground/src/stories/`; consult pinned GPUI and GPUI Components behavior before adding RustUI APIs. Depends on P0. | High: platform focus and hover details can differ; do not infer support from API presence. |
| P1 | Verify locale switching and translated labels for English, French, and Arabic, including Arabic logical direction and layout. | `apps/rustui-playground/src/lib.rs`, `stories/locale_selector.rs`, `stories/localization_gallery.rs`, and locale catalogs. Depends on P0 and current localization runtime. | Medium-high: catalog completeness does not prove rendered translation or RTL layout. |
| P1 | Verify the visible Light/Dark appearance controls and startup theme behavior. | Playground shell and theme adapter; use only runtime-supported themes. | High: planned theme metadata does not establish runtime support. |
| P1 | Verify route push/back behavior, destination content, and repeated transitions in NavigationStack. | `apps/rustui-playground/src/stories/foundation_demos.rs` and navigation story modules. Depends on P0 and native interaction. | Medium-high: path bindings and declared destinations do not prove back-control behavior. |
| P2 | Keep displayed source fragments labeled as illustrative and their coverage aligned with rendered stories. | `apps/rustui-playground/src/stories/code_preview.rs` and story registry. | Medium: static strings can drift and imply executable APIs they do not establish. |
| P2 | Replace generic pending labels only with evidence-backed per-story results after the corresponding GUI checks. | Playground story status text and the acceptance record from P0. | Medium: premature status changes risk overstating unrun checks. |

## Manual acceptance criteria

Record OS/build identity, demo, input steps, observed result, and any failure.
Mark a criterion passed only after performing it in the native Playground
window. Keep each bounded result distinct from checks that remain open.

- **Hover and focus:** use pointer and keyboard to inspect hover feedback, tab
  order, visible focus, keyboard activation, and focus retention for each
  interactive control class. Check controls nested in Tabs, Breadcrumb, and
  NavigationStack.
- **Disabled:** confirm the fixed Unavailable button and disabled tab cannot
  activate or mutate state. Toggle the main Button demo's disabled control and
  confirm both action buttons stop changing the count until re-enabled. Inspect
  native accessibility state as well as pointer behavior.
- **Locale:** select English, French, and Arabic. Verify localized Playground
  labels update, no keys are exposed, Arabic text is legible, and directional
  layout is appropriate. Record untranslated static demo content separately;
  catalog validity alone is not enough.
- **Theme:** verify the launch theme resolves and visibly applies. Exercise the
  Light and Dark controls and verify colors/contrast across every demo,
  including disabled and focused controls. Do not count planned theme files as
  runtime support.
- **Navigation:** from the root, open details and inspector, use back controls
  to return one level at a time, repeat transitions, and verify displayed
  destination matches the current path. Activate both Breadcrumb actions and
  verify current-page text and event summary change.
- **Stories/state:** switch repeatedly among all ten framework demos; confirm
  selection, button count/reset, editable controls, collection selections,
  tabs, and event summary remain coherent and do not crash or display stale
  content. Exercise catalog search, no-match recovery, and scrolling.
- **Small-window scrolling (open):** resize the canonical window from 1320 x 820 to 960 x 560 points. Exercise the catalog sidebar to its final catalog row and framework demo. In the `Stacks` demo, select the `Grid` tab and scroll the shell preview to the embedded Scroll Gallery; verify its vertical, horizontal, and both-axis panes scroll independently. Exercise the references pane when it has overflow. Verify each region can return to the top without clipping; record the window size, region, last reachable item, and any defect. Keep this gate open until native evidence is recorded.

Completion requires recorded native-window evidence for each criterion and
resolution or explicit acceptance of observed defects. Compile or source
inspection is supporting evidence, not a substitute.


## M547 repeat preflight
A fresh disposable-app attempt could not establish a 960×560 logical window from CUA, so it stopped before the Scroll Gallery probe. This leaves the prior System Events window-size readback intact and establishes no pane endpoint result. Cleanup removed only the disposable app. Evidence: `target/headstart/evidence/stage3-l1-small-window-resize-blocker-20261021.json`.
