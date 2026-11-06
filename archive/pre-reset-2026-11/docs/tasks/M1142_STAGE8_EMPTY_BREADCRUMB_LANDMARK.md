# M1142 — Stage 8: omit an empty Breadcrumb navigation landmark

An empty Breadcrumb no longer creates a navigation landmark without any
navigation content. The rendering helper and production path both return the
empty state for a Breadcrumb with no items.

Focused regression: `empty_breadcrumb_has_no_navigation_landmark_content`;
`rtk proxy ./scripts/cargo-headstart test -p rustui --lib empty_breadcrumb_has_no_navigation_landmark_content`
exited 0.

Broader native keyboard and screen-reader acceptance remains open.
