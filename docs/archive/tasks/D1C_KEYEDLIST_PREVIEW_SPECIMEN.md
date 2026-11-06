# M299 — Stage 9 / D1c KeyedList preview specimen

## Status

Implemented. The focused regression passed; the locked/offline Playground
all-targets check passed. Native GUI acceptance was not run.

## Scope

Add an opt-in KeyedList specimen to the canonical compiled-preview gallery
using the public RustUI `components::data::keyed_list::{KeyedList,
KeyedListItem}` API. Its descriptor should expose only bounded scalar
properties:

- `first_label` (proposed default `First item`, 1–80 characters)
- `second_label` (proposed default `Second item`, 1–80 characters)
- `selected_id` (default `first`, restricted to `first` or `second`)

The compiled factory constructs exactly two fixed `KeyedListItem` rows with
stable IDs `first` and `second`; only their labels and controlled selected ID
come from the descriptor. Bind selection to `selected_id` through the existing
generation/revision-stamped property-feedback path. Use the public default
height (240 logical pixels), keep search disabled, and keep event and action
maps empty. Do not serialize arbitrary item arrays, closures, or executable
behavior.

Add a visible `Preview KeyedList specimen` mount control and use the existing
preview host for property editing and descriptor import/export. The focused
regression should cover schema defaults and bounds, the restricted selection
values, empty event/action maps, descriptor round-trip, stamped selection
feedback, and rejection of blank live label feedback and invalid imported
values while preserving the mounted preview. Whitespace-only labels are
rejected at the shared feedback boundary because `KeyedListItem::new` asserts
on blank labels.

## Acceptance

Run the focused locked/offline Playground regression and a locked/offline
Playground all-targets check using an isolated Headstart target directory.
Record exact evidence paths only after those checks run. Format only touched
Rust files and inspect the scoped diff.

Native row selection and identity behavior, keyboard/focus, accessibility
roles/names and screen-reader behavior, localization, and cross-platform
behavior remain open acceptance gates. A source test or successful build does
not close them.

Validation completed:

- Focused regression: `target/headstart/evidence/m299-focused-test.log`
- Playground all-targets check: `target/headstart/evidence/m300-m301-playground-all-targets-check.log`
- Isolated target directory: `target/headstart/candidates/m299-validation/target`

## M305 native selection observation (2026-10-20)

On macOS 27, the native tree exposed the two rows First item and Second item, with the first row selected initially. Clicking Second item changed stamped `selected_id` feedback to `second` at revision 1, and the second row became selected in AX. VoiceOver, keyboard/focus, localization/RTL, visual comparison, and other platforms remain open. The disposable app bundle and standalone executable were removed. Evidence: `target/headstart/evidence/m305-keyedlist-native-observation-20261020.json`; build: `target/headstart/evidence/m304-sidebar-native-build.log`.
