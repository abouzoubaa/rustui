# L1: RustUI Lab acceptance roadmap

Historical snapshot taken before L2/L3a. The story browser is now searchable and
scrollable (`L2_STORY_BROWSER.md`); `theme_selector.rs` now implements Light/Dark
controls for macos27 (`L3A_APPEARANCE_CONTROLS.md`). GUI acceptance remains pending.

## Current state and evidence

The Lab is a native GPUI window with ten selectable stories. `main.rs` owns the
root `Lab` state, locale activation, and one startup theme. `canvas.rs` renders
the Button, Text, stacks, input controls, List, Table, Tree, Tabs, Breadcrumb,
and NavigationStack stories. The single-line status in `test_runner.rs` says
interaction and visual verification is pending for every story; compilation is
not GUI acceptance.

Evidence paths:

- `apps/rustui-lab/src/main.rs`: story switching, counters, button property
  controls, startup theme/localization activation, event summary.
- `apps/rustui-lab/src/canvas.rs`: rendered stories and the Action/Binding values
  passed into them.
- `apps/rustui-lab/src/code_preview.rs`: displayed API fragments; these are
  static strings, not code compiled or evaluated by the Lab.
- `apps/rustui-lab/src/properties_panel.rs`: Button style/disabled properties.
- `apps/rustui-lab/src/locale_selector.rs` and
  `apps/rustui-lab/locales/{en,fr,ar}/lab.ftl`, plus `locales/{en,fr,ar}/rustui.ftl`:
  locale setup and catalogs.
- `apps/rustui-lab/src/theme_selector.rs`: explicitly planned placeholder; no
  runtime selector behavior.
- `themes/macos27.json`: the one theme parsed and activated at launch.
- `apps/rustui-lab/src/layout_inspector.rs`, `event_inspector.rs`, and
  `test_runner.rs`: current summary/status displays, not an automated or GUI
  acceptance harness.

No existing `graphify-out/graph.json` was found during this bounded inspection.
No GUI acceptance run was performed.

## Snippet status

The preview strings are **display-only fragments**. They omit the surrounding
component and imports, and several snippets refer to generated state bindings
whose declarations are not shown. They must not be described as independently
executable examples. The Breadcrumb preview previously used undefined
`go_overview` and `go_components` names. It now follows the compiled canvas:
`StoryCanvas` owns passed `Action` values and clones them into the two actionable
items; the current page is supplied as state. Other preview snippets should be
kept aligned with `canvas.rs` as that file evolves. For example, the Button
preview is an API-shaped fragment; the actual canvas receives an `Action` and
uses `self.increment.clone()`.

## Work breakdown

Priority order is acceptance evidence first, then missing capabilities. Candidate
files are proposals for follow-up work, not authorization to change them.

| Priority | Work item | Candidate files and dependency | Risk |
| --- | --- | --- | --- |
| P0 | Run and record a repeatable native-window acceptance pass for all ten stories, interactions, and failures. | Existing `apps/rustui-lab/src/main.rs`, `canvas.rs`, `test_runner.rs`; requires a runnable macOS GUI session and a written observation record. | High: compile success cannot establish native event/focus/render behavior. |
| P1 | Verify hover, keyboard focus, activation, and disabled behavior for buttons, inputs, tabs, breadcrumb actions, and navigation links; fix only observed defects. | Primarily `apps/rustui-lab/src/canvas.rs`; consult upstream GPUI/GPUI Components behavior before adding RustUI APIs. Depends on the P0 GUI run. | High: platform focus and hover details can differ; do not infer support from API presence. |
| P1 | Verify locale switching and translated labels for English, French, and Arabic, including Arabic logical direction and layout. | `apps/rustui-lab/src/locale_selector.rs`, `apps/rustui-lab/locales/{en,fr,ar}/lab.ftl`, `locales/{en,fr,ar}/rustui.ftl`, and if needed `main.rs`/`canvas.rs`. Depends on P0 and current localization runtime. | Medium-high: catalog completeness does not prove rendered translation or RTL layout. |
| P1 | Decide and implement the user-visible theme acceptance surface: startup theme validation, plus an interactive selector only if in scope. | For startup-only acceptance: `main.rs` and `themes/macos27.json`. For a selector: `theme_selector.rs`, `main.rs`, likely new explicit state/wiring in `main.rs`; first settle supported runtime theme set and loading contract. | High: `theme_selector.rs` is only a planned placeholder; planned theme metadata is not runtime support. |
| P1 | Verify route push/back behavior, destination content, and repeated transitions in NavigationStack. | `apps/rustui-lab/src/canvas.rs`, with route state wired from `main.rs`; depends on P0 and GPUI native interaction. | Medium-high: path binding and declared destinations alone do not prove back-control behavior. |
| P2 | Make displayed snippets clearly illustrative or provide independently compilable examples and keep them synchronized with the rendered stories. | `apps/rustui-lab/src/code_preview.rs`, possibly `main.rs` for an explicit display label; synchronize against `canvas.rs`. | Medium: static strings can drift and imply executable APIs they do not establish. |
| P2 | Replace generic pending status with evidence-backed per-story acceptance results after GUI work is complete. | `apps/rustui-lab/src/test_runner.rs` and the acceptance record from P0. | Medium: premature status changes risk overstating unrun checks. |

## Manual acceptance criteria

Record OS/build identity, story, input steps, observed result, and any failure.
Mark a criterion passed only after performing it in the native window.

- **Hover and focus:** use pointer and keyboard only to inspect hover feedback,
  tab order, visible focus indication, keyboard activation, and focus retention
  for each interactive control class. Check controls nested in Tabs, Breadcrumb,
  and NavigationStack.
- **Disabled:** confirm the fixed `Unavailable` button and disabled tab cannot
  activate or mutate state. Toggle the main Button story's disabled control and
  confirm both action buttons stop changing the count until re-enabled.
- **Locale:** select English, French, and Arabic. Verify all localized Lab and
  component labels update, no keys are exposed, the Arabic text is legible, and
  directional layout is appropriate. Record untranslated static story content
  separately; catalog validity alone is not enough.
- **Theme:** verify the launch theme resolves and visibly applies. If theme
  switching is accepted as scope, change between each supported runtime theme
  and verify colors/contrast across every story, including disabled and focused
  controls. Do not count a placeholder selector or planned theme file as support.
- **Navigation:** from the root, open details and inspector, use back controls
  to return one level at a time, repeat transitions, and verify displayed
  destination matches the current path. Also activate both Breadcrumb actions
  and verify current-page text and event summary change.
- **Stories/state:** switch repeatedly among all ten stories; confirm selection,
  button count/reset, editable controls, collection selections, tabs, and event
  summary remain coherent and do not crash or display stale content.

Completion requires recorded native-window evidence for each criterion and
resolution or explicit acceptance of observed defects. A compile or source
inspection is useful supporting evidence, not a substitute.
