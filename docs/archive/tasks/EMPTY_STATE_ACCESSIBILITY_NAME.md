# EmptyState accessible-name contract

`EmptyState` uses its visible title as the default Status name. An explicit
`accessibility_label(...)` override must contain a non-whitespace character and
is preserved exactly as supplied. A blank visible title is allowed only when a
nonblank explicit name is provided; the resolved name is checked again at
render time.

`localized_accessibility_label(...)` resolves a message key through the active
locale. A blank key is rejected. Blank or unresolved catalog results (including
a result equal to the key) fall back to the prior explicit label, or the visible
title when no explicit label was set. Localization changes only the accessible
name; the visible title remains unchanged.

Focused source tests cover rejection of whitespace-only overrides and keys,
exact preservation of nonblank text, blank-title fallback behavior, explicitly
named blank-title states, and localized-name resolution/fallback. The locked/offline RustUI library suite passed 474/474
(`target/headstart/evidence/roadmap-rustui-library-tests-followup-20261021.log`),
and the locked/offline Playground all-targets check passed
(`target/headstart/evidence/roadmap-playground-all-targets-followup-20261021.log`).
Formatting and scoped whitespace checks passed. Native accessibility-tree,
screen-reader, and cross-platform acceptance remain open.

## M607 — Catalog-backed accessible name

`EmptyState::localized_accessibility_label(message_id)` resolves through the
active render locale and preserves the prior explicit label or visible title as
fallback. Blank keys and blank or key-echo results use the fallback without
changing visible content. The focused suite passed 9/9; `rustfmt --check` and
scoped diff checks passed. Native accessibility-tree, screen-reader, and
cross-platform acceptance remain open. Evidence:
`target/headstart/evidence/m607-empty-state-localized-accessibility-tests-20261021.log`.
