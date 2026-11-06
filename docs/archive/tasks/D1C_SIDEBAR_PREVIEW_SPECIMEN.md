# M298 — Stage 9 / D1c Sidebar preview specimen

## Status

Implemented. The focused regression and locked/offline all-targets check
passed. Native GUI acceptance was not run.

## Scope

Add an opt-in Sidebar specimen to the canonical compiled-preview gallery using
the public RustUI `components::navigation::sidebar::{Sidebar, SidebarItem}`
API. Its descriptor should expose only bounded scalar properties:

- `home_label` (proposed default `Home`, 1–80 characters)
- `settings_label` (proposed default `Settings`, 1–80 characters)
- `selected_id` (default `home`, restricted to `home` or `settings`)

The compiled factory constructs exactly two fixed `SidebarItem` rows with IDs
`home` and `settings`; only their labels and controlled selected value come
from the descriptor. Bind selection to `selected_id` using the existing
generation/revision-stamped property-feedback path. Keep event and action maps
empty; selection is property feedback, not a preview event. Do not serialize
arbitrary item arrays, closures, or executable behavior.

Add a visible `Preview Sidebar specimen` mount control and use the existing
preview host for property editing and descriptor import/export. The focused
regression should cover schema defaults and bounds, the restricted selection
values, empty event/action maps, descriptor round-trip, stamped selection
feedback, and rejection of an invalid imported `selected_id` without
replacing the mounted preview.

## Acceptance

Run the focused locked/offline Playground regression and a locked/offline
Playground all-targets check using an isolated Headstart target directory.
Record exact evidence paths only after those checks run. Format only touched
Rust files and inspect the scoped diff.

Both live property feedback and import reject whitespace-only `home_label` and
`settings_label` before mounting or updating the descriptor, avoiding the
public constructor's nonblank-label assertion. The focused regression covers
both blank-label cases and verifies the mounted snapshot is preserved.

Native selected-row updates, focus behavior, Up/Down/Home/End/Enter keyboard
navigation, accessibility-tree role/name and screen-reader behavior,
localization, and cross-platform behavior remain open acceptance gates. A
source test or successful build does not close them.

Validation completed:

- Focused regression: `target/headstart/evidence/m298-focused-test.log`
- Playground all-targets check: `target/headstart/evidence/m300-m301-playground-all-targets-check.log`
- Isolated target directory: `target/headstart/candidates/m299-validation/target`

The focused regression covers schema defaults and label bounds, the restricted
selection values, empty event/action maps, descriptor round-trip, stamp-guarded
selection feedback, and rejection of an imported invalid `selected_id` while
preserving the mounted preview.

## M304 native observation and layout correction (2026-10-20)

The first macOS 27 preview mounted an empty outline. The focus-scope wrapper lacked a definite height even though the frame and upstream Sidebar requested full-height layout. Adding `.size_full()` to the wrapper restored the two fixture rows inside the 240×240 frame. The AX tree exposed Home and Settings; clicking Settings changed stamped `selected_id` feedback to `settings` at revision 1. This remains partial native acceptance: VoiceOver, focus announcements, full keyboard navigation, localization/RTL, visual comparison, and other platforms are open. The locked/offline Playground build passed and the disposable app artifacts were removed. Evidence: `target/headstart/evidence/m304-sidebar-native-build.log` and `target/headstart/evidence/m304-sidebar-native-observation-20261020.json`.
