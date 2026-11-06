# ErrorState accessible-name contract

`ErrorState` always renders an alert with a name. Its visible title is the
default accessible name, and callers may replace that name with
`accessibility_label(...)`. Both inputs reject whitespace-only strings while
preserving any non-empty supplied text as-is. This prevents an explicitly
blank override or fallback title from creating a nameless alert.

`localized_accessibility_label(message_id)` resolves the accessible name from
the active render locale. A prior explicit label remains the fallback;
otherwise the validated visible title is used. Blank IDs are rejected, and
blank or unresolved catalog results use that fallback. Localization changes
only the alert name; title, detail, identity, and alert role remain intact.

Focused source tests cover a normal title, a non-empty override, explicit and
title fallbacks for localized names, blank title and override rejection, and
blank localization-key rejection. This does not establish native
accessibility-tree, screen-reader, or cross-platform acceptance.

## M606 — Catalog-backed accessible name

The focused ErrorState suite passed 7/7. `rustfmt --check` and scoped diff
checks passed. Evidence: `target/headstart/evidence/m606-error-state-localized-accessibility-tests-20261021.log`.
