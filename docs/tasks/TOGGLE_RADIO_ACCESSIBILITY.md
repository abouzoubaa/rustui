# Toggle and Radio accessible-name validation

Toggle and Radio use their visible label as the default accessible name and
allow an explicit override. Both now reject whitespace-only visible labels
and overrides, matching the source-level guard on Checkbox. Non-empty values
are preserved as supplied.

Focused unit tests verify the default-name contract and both rejection paths
for each control; raw output is recorded in
`target/headstart/evidence/toggle-radio-accessibility-m177-20261020.log`. Native
role/name/value, focus, keyboard activation, grouping, screen-reader
announcement, and cross-platform behavior remain separate acceptance work.

## 2026-10-21 bounded Toggle source audit

The Toggle adapter reads the current checked binding and resolves the current
visible label and accessible-name override on each render. Its change handler
captures the binding writer for that render; the pinned GPUI Base `Switch`
omits focus tracking and the change handler when disabled. RustUI also marks
the synthetic Switch node disabled. No stale retained-value/name path or
disabled-activation source defect was found. The focused locked/offline Toggle
suite passed 4/4; evidence:
`target/headstart/evidence/toggle-stage7-retention-audit-20261021.log`.

Native keyboard activation/focus, live AX role/name/value and disabled state,
screen-reader announcements, visual behavior, and cross-platform acceptance
remain open.

## M368 — Disabled accessibility state

The RustUI Toggle and Radio adapters now set the AccessKit disabled state when
`.disabled(true)` is supplied. The pinned GPUI Base controls suppress focus and
activation but do not set this state themselves. A focused source regression
verifies the AccessKit disabled flag; native accessibility-tree and
screen-reader acceptance remain pending.

The first locked/offline `rustui` all-targets check using the default shared
target reported unresolved `rustui_i18n` formatting re-exports in
`crates/rustui/src/localization/mod.rs`. The exports are correct in
`crates/rustui-i18n/src/lib.rs`; follow-up diagnosis traced the mismatch to a
stale `rustui_i18n` rlib in the shared target. Keep the initial failure as a
cache-specific observation in
`target/headstart/evidence/toggle-radio-accessibility-check-m177-20261020.log`.
The same locked/offline package check passes with the existing isolated target
and two existing example warnings. Passing output:
`target/headstart/evidence/toggle-radio-accessibility-check-isolated-m177-20261020.log`.


## Radio localized accessible name

`RadioView::localized_accessibility_label` accepts a nonblank RustUI message ID and resolves it from the active catalog when rendered. The explicit `.accessibility_label(...)` value is retained as fallback; when no explicit name is set, the visible label is used. Blank or unresolved catalog results use that fallback. Focused source tests cover localized resolution, fallback behavior, and blank-ID rejection. Native accessibility-tree and screen-reader verification remains separate acceptance work.
