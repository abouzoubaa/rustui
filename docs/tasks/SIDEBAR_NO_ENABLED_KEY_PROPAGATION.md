# Stage 8 — propagate Sidebar navigation keys when no row is enabled

The focused Sidebar Tree handler previously stopped Up, Down, Home, and End
even when the Sidebar was empty or every row was disabled. In that state there
is no local navigation target, so those keys now remain available to ancestor
handlers. Enabled rows continue to make the navigation keys local; Enter keeps
its existing handling.

The `navigation_keys_propagate_when_sidebar_has_no_enabled_rows` regression
covers empty, all-disabled, and enabled cases; the focused locked/offline test
passed 1/1. Scoped `git diff --check` passed. Whole-file `rustfmt --check`
still reports unrelated pre-existing formatting differences elsewhere in the
shared `sidebar.rs` file. Native keyboard routing, per-row focus, and
accessibility acceptance remain open.
