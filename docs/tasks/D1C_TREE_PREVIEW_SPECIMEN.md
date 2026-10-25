# M301 — Stage 9 / D1c Tree preview specimen

## Status

Implemented. The focused regression passed; the locked/offline Playground
all-targets check passed. M302 records partial native interaction and
accessibility-tree observations; broader native acceptance remains open.

## Scope

Add an opt-in Tree specimen to the canonical compiled-preview gallery using
the public RustUI Tree component API. Its descriptor should expose only these
bounded scalar properties:

- `root_label` (proposed default `Project`, 1–80 characters)
- `src_label` (proposed default `src`, 1–80 characters)
- `readme_label` (proposed default `README.md`, 1–80 characters)
- `tests_label` (proposed default `tests`, 1–80 characters)
- `selected_id` (default `none`, restricted to `none`, `root`, `src`,
  `readme`, or `tests`)

The compiled factory constructs one fixed tree with unique hard-coded IDs
`root`, `src`, `readme`, and `tests` (three to four nodes total). Only labels
and the controlled selected ID come from the descriptor. Validate all labels
as nonblank before mount; the public `TreeNode` constructor panics on blank
labels. Fixed IDs avoid duplicate-ID panic. Bind selection to `selected_id`
through the existing generation/revision-stamped property-feedback path. Keep
event and action maps empty; selection is property feedback, not a preview
event. Do not serialize arbitrary recursive trees, item arrays, closures, or
executable behavior.

Add a visible `Preview Tree specimen` mount control and use the existing
preview host for property editing and descriptor import/export. The focused
regression should cover schema defaults and bounds, restricted selection
values, the fixed unique-ID fixture, empty event/action maps, descriptor
round-trip, stamped selection feedback, and rejection of invalid imported
values without replacing the mounted preview.

## Acceptance

Run the focused locked/offline Playground regression and a locked/offline
Playground all-targets check using an isolated Headstart target directory.
Record exact evidence paths only after those checks run. Format only touched
Rust files and inspect the scoped diff.

Expansion is internal Tree state and is not part of the descriptor. Native
expansion and selection behavior, focus and keyboard interaction,
accessibility-tree role/name and screen-reader behavior, localization, and
cross-platform behavior remain open acceptance gates. A source test or
successful build does not close them.

## Validation completed

- Focused regression: `target/headstart/evidence/m301-focused-test.log`
- Playground all-targets check: `target/headstart/evidence/m300-m301-playground-all-targets-check.log`
- Isolated target directory: `target/headstart/candidates/m300-validation/target`

## M302 — Stage 9 / D1c Tree specimen native interaction observation

On macOS 27, the disposable Playground mounted `tree-specimen` at generation
2. Its accessibility tree exposed an `outline` with named `row` entries
`Project`, `src`, `README.md`, and `tests`; selected state was reported, while
focused and expanded state were not. Clicking `src` updated the inspector's
`selected_id` to `src`. Down moved selection from `src` to `README.md` and then
`tests`, with stamped feedback accepted. Left and Right collapsed and restored
the root and `src` branches. No defect was observed in these exercised
interactions.

This is partial native acceptance. VoiceOver speech/focus announcements,
native focused/expanded accessibility states, broader keyboard behavior,
localization/RTL, visual comparison, and other platforms remain unverified.
The disposable bundle and standalone executable were removed after the run;
the pre-existing Playground process (PID 20036) was left untouched. Evidence:
`target/headstart/evidence/m302-tree-native-observation-20261020.json` and
`target/headstart/evidence/m302-tree-native-build.log`.
