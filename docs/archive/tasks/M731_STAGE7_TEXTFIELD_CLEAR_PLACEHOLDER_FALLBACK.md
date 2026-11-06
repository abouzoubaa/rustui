# M731 — Stage 7 TextField clear-name fallback

`TextField` permits an empty placeholder. The generated clear-button accessible name previously interpolated that placeholder directly, producing `Clear ` for an empty value and a whitespace-only name for whitespace input. The shared fallback helper now returns `Clear text field` when the placeholder has no non-whitespace characters, while preserving `Clear {placeholder}` for nonblank placeholders.

Regression: `clear_button_accessible_name_stays_meaningful_without_a_placeholder` covers both empty and whitespace-only placeholders. `./scripts/cargo-headstart test --locked --offline -p rustui --lib clear_button_accessible_name_stays_meaningful_without_a_placeholder` passed (1 passed, 734 filtered). Scoped rustfmt and diff whitespace checks passed. This is source-level evidence only; native accessibility-tree, screen-reader, locale-switching, and cross-platform acceptance remain open.
