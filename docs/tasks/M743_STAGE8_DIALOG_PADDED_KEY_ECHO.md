# M743 — Stage 8 / ignore padded Dialog localization key echoes

Dialog's localized accessible-name resolver now treats a translation whose
trimmed value equals its message key as unresolved. This prevents padded key
echoes from leaking localization identifiers into the Dialog's accessible
name; the resolver continues to the visible title, trigger, or stable identity.

Regression: `padded_unresolved_localized_accessible_name_falls_back_to_visible_title`.
Validation: `./scripts/cargo-headstart test --locked --offline -p rustui --lib
padded_unresolved_localized_accessible_name_falls_back_to_visible_title` passed
(1 passed, 736 filtered out). Scoped rustfmt and `git diff --check` passed.

This is source-level evidence only. Native accessibility-tree, screen-reader,
locale-switching, RTL, and cross-platform acceptance remain open; refer to the
Stage 8 C1 Dialog and Sheet gate in `ROADMAP_COMPLETION_GATES.md`.
