# Controlled Outline

`Outline(id, nodes, selection, expanded)` is a single-column document hierarchy
over the existing Tree retained engine. `nodes` contains public `TreeNode` values;
`selection` is `Binding<Option<String>>` and `expanded` is
`Binding<HashSet<String>>`. It adds controlled expansion to the existing stable-ID
selection behavior. It does not add columns, editing, drag/drop or a new runtime.

## State and ownership

- Identity is explicit, stable and nonblank. Node IDs are nonblank and unique
  across the hierarchy. Labels must be nonblank. Invalid construction panics at
  the same checks as Tree. An explicit nonblank `.accessibility_label(...)` is
  required before render; missing or blank labels panic with a specific message.
- The expansion binding is authoritative on every render, including changes with
  identical node declarations. Per-node `TreeNode::expanded` values are ignored.
  Only IDs of current branches are retained; missing IDs and leaf IDs normalize
  away through a guarded owner update. Expansions beneath collapsed ancestors
  remain in the binding and survive reopening the ancestor.
- Selection is owner-controlled by stable ID, using Tree's existing invalid-ID
  normalization. A hidden selected descendant stays in the owner binding without
  opening its ancestors. The native Tree has no selected row while that ID is
  hidden. Keyboard navigation from that state selects an enabled visible row.
- Disabled branches cannot be expanded by user keyboard/pointer interaction.
  Owner bindings can still declare disabled-row selection/expansion, matching
  Tree's existing programmatic-state contract.
- Owner reorders and relabels retain one GPUI TreeState for the same identity.
  Observer callbacks always use the current owner Binding. Queued normalization
  and interaction updates check their expected owner state to preserve a newer
  owner change; identical values do not notify again.
- Removing the view drops its retained observer and TreeState ownership. A stale
  externally retained TreeState cannot update the removed owner's bindings.
  Unmount itself does not clear either binding. Remount initializes from current
  owner state; it does not resurrect stale internal expansion.

`.height(f32)` rejects nonfinite values and clamps finite values to at least one
logical pixel, matching Tree. `.on_activate(...)` requires an action that can run
from keyboard input and activates enabled leaves only.

## Keyboard and accessibility

The existing Tree input path handles enabled visible-row Up/Down traversal,
Left/Right expansion and Enter activation. Disabled-row and disabled-folder
guards remain in that shared path. Ordinary Home/End select the first/last
enabled visible row; collapsed descendants and disabled edges are skipped.
Modified shortcuts and active character/IME input keep their existing handling.

The required accessible label names an outer `Group` that contains the semantic
Tree and rows. The approved, reproducible Tree series now supplies the inner
Tree name and one-based row hierarchy levels; rendered regressions verify them.
Native AX, VoiceOver, Web accessibility, focus display, locale/RTL and visual
acceptance require separate runtime checks on the declared macOS/Web targets.
Source code and test-support event simulation do not establish that acceptance.

## Canonical Playground and checks

The `outline_view` catalog route opens a dedicated public-API Outline story.
It includes collapse/expand-all, hidden-heading selection, branch removal and
restoration, reorder/relabel, a disabled branch and unmount/remount. The existing
Tree story remains separate. The `outline-specimen` compiled-preview factory uses
the public Outline API with editable labels, selected ID, and four expansion
states (`collapsed`, `root`, `nested`, `all`). Its fixed-node descriptor preserves
hidden selection and nested expansion across export/import. Existing schema and
revision checks reject invalid properties, blank labels and stale binding writes.

Focused regressions are in `crates/rustui/src/components/data/outline_tests.rs`:
controlled expansion and invalid-ID normalization, expected-owner guards, actual
rendered physical-key routing to the latest binding, unchanged-declaration owner
updates, hidden selection, retained state, removed observer and remount.

Validation uses the existing consolidation cache:

```sh
CARGO_TARGET_DIR=target/headstart/verification/production-consolidation ./scripts/cargo-headstart test --profile headstart --locked --offline -p rustui --lib components::data::tree::
CARGO_TARGET_DIR=target/headstart/verification/production-consolidation ./scripts/cargo-headstart test --profile headstart --locked --offline -p rustui-playground --lib
CARGO_TARGET_DIR=target/headstart/verification/production-consolidation ./scripts/cargo-headstart check --profile headstart --locked --offline -p rustui-playground --all-targets
```

On 2026-10-25 the focused Tree/Outline suite passed 25 tests, including two
rendered-keyboard Outline tests. Logs: `/tmp/rustui-outline-tree-tests-20261025.log`.
Playground passed 189 library tests and all-target checking; logs are
`/tmp/rustui-outline-playground-tests-20261025.log` and
`/tmp/rustui-outline-all-targets-20261025.log`.
That 2026-10-25 checkpoint did not include native or browser acceptance.

On 2026-10-26 the shared adapter's focused suite passed **30/30** tests. Its
Home/End regression uses rendered Outline keyboard events and checks controlled
owner selection, disabled edges, collapsed descendants, modified shortcuts and
an all-disabled tree. Rebuilt native Tree Home/End checks pass through real
CUA keyboard input; Outline-specific native acceptance remains separate.
Evidence: `target/headstart/evidence/native-tree-home-end-20261026/`.

## Bounded Web functional acceptance — 2026-10-26

Current optimized Web source `e50b6ff0` passes **12/12 functional checks** in one
Chromium profile at 1280×720: collapse, hidden selection, expansion, stable-ID
reorder/relabel, unmount/remount, removal normalization, restoration without
resurrecting removed state, pointer selection, Home/End and Enter activation.
Evidence: `target/headstart/evidence/web-outline-drawer-20261026/acceptance.json`.
This establishes the named state/interaction cases only. Native, broader browser,
visual/locale/RTL and workload acceptance remain open. Accessibility and VoiceOver
are explicitly undone and deferred by user direction.

## Bounded native functional acceptance — 2026-10-26

The reproducible native executable `d857e6b5b687951bcb20fd4fff0e1be2abecf7e2eb3b0c8ff696186e8acbffcf` passes **13/13 bounded functional checks** on macOS 27 arm64 at 1320×820 logical pixels. Application/runtime inputs are unchanged from the controlled release build at `4e00ad14`. Checks cover initial owner state, parent selection, collapsed hidden-selection retention, expansion, pointer selection, arrow navigation, stable IDs through reorder/relabel, removed-ID normalization, restoration and unmount/remount.

Evidence: `target/headstart/evidence/native-outline-drawer-20261026/`, including full state records, a rendered snapshot, preflight hashes and verified cleanup. Incorrect initial harness coordinates/count formatting were corrected and excluded; no component defect is claimed. The owned app and disposable bundle are removed. Broader visual, locale/RTL, focus and workload acceptance remain open. Accessibility and VoiceOver are undone/deferred.
