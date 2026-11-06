# M976 — Stage 8 / keep the current Breadcrumb from navigating to itself

The final Breadcrumb item represents the current page. Rendering now omits its
navigation action while preserving actions on earlier crumbs.

Focused check: `breadcrumb::tests::final_breadcrumb_does_not_keep_a_navigation_action`
passed (1/1). Rustfmt and a scoped whitespace check passed.

Native keyboard and screen-reader acceptance remain open.
