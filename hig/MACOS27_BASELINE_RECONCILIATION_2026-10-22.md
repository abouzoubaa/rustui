# macOS 27 theme baseline reconciliation — 2026-10-22

All writes in this batch are confined to `hig/`. Two read-only JSON baselines were created before the editable copy changed:

- `macos27.production-baseline.2026-10-22.json`: byte-for-byte copy of `themes/macos27.json`; 820,587 bytes; SHA-256 `5a4a81adb6af226d59af916a4feadf575b23447b7cee2517882e9bdf2453e01e`.
- `macos27.hig-pre-design-baseline.2026-10-22.json`: byte-for-byte copy of pre-edit `hig/macos27.json`; 811,905 bytes; SHA-256 `903c483647ee2c9cf371d98e8c1f051c6ff58ee8ff21b1903e5bd15fa1ee5e3e`.

Both are mode `0444` (read-only). Production `themes/macos27.json` was read only. Before reconciliation both documents had 41 IDs / 419 rules. Recursive comparison found **zero value differences at shared JSON paths** and **96 production-only paths**; no HIG-only values were lost. The earlier HIG document was a deliberate reduced subset; its documented removal of source/composition metadata has now been reconciled.

## Restored into editable HIG copy

- All 41 production `components.*.metrics` values and composition arrays (component count: 40 records have one or both fields).
- Full `source` and `derivation` provenance objects.
- All 14 production theme roles, retaining HIG `primary: accent`.
- Full token-source families for `colors`, `dimensions`, `radii`, `spacing`, and `typography`, retaining HIG aliases `spacing.control_inline: space_2` and `radii.control: md`.
- All original component part/style values; **41 component IDs and 419 style entries remain**.

## Exact root-level additions

| Path | Production reference | Pre-edit HIG copy | Resolution |
|---|---|---|---|
| `source` | `kind`, `ownership`, `svg_policy` | `{}` | Restored production provenance. |
| `derivation` | method + preserved-count metadata | `{}` | Restored production provenance. |
| `theme_roles` | 14 roles | only `primary` | Unioned; shared `primary` value is identical. |
| `token_sources` | 5 namespaces and role aliases | `spacing.control_inline`, `radii.control` | Unioned; both shared aliases identical. |

## Component metadata paths restored

- `components.alert_dialog`: composition, metrics
- `components.button_group`: composition, metrics
- `components.checkbox`: metrics
- `components.color_picker`: composition, metrics
- `components.combo_box`: composition, metrics
- `components.disclosure_control`: composition, metrics
- `components.divider`: metrics
- `components.form`: composition, metrics
- `components.form_accessories`: composition, metrics
- `components.icon`: metrics
- `components.image`: metrics
- `components.label`: metrics
- `components.link`: metrics
- `components.list`: composition, metrics
- `components.menu`: composition, metrics
- `components.menu_bar`: composition, metrics
- `components.menu_item`: composition, metrics
- `components.menu_separator`: composition, metrics
- `components.notification`: composition, metrics
- `components.outline_view`: composition, metrics
- `components.picker`: metrics
- `components.radio_button`: composition, metrics
- `components.save_dialog`: composition, metrics
- `components.scroll_view`: metrics
- `components.search_field`: composition, metrics
- `components.search_field_examples`: composition, metrics
- `components.segmented_control`: composition, metrics
- `components.sidebar`: composition, metrics
- `components.slider`: metrics
- `components.stepper`: metrics
- `components.tab_bar`: composition, metrics
- `components.table`: composition, metrics
- `components.text`: metrics
- `components.text_field`: metrics
- `components.text_style_guide`: composition, metrics
- `components.toggle`: metrics
- `components.toolbar`: composition
- `components.utility_window`: composition, metrics
- `components.window_chrome`: composition, metrics
- `components.window_controls`: composition, metrics

## Post-merge verification

`hig/macos27.json` now hashes exactly to the production baseline because its previous differences consisted solely of omitted paths. The pre-edit HIG baseline preserves the old reduced artifact independently. The editable file is therefore a faithful verified v1 starting point with all prior HIG choices reconciled, ready for additive sandbox design work. No claim is made that this confirms a runtime load or renderer coverage report; no Rust build/test was run.
