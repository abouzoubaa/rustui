# M792 — Stage 8 Sidebar disabled Enter activation

`Sidebar` uses Enter on the currently selected enabled destination to write the
same selection back through its binding. That write is the activation path for
the controlled sidebar. The previous guard treated every nonempty selection as
activatable, including a destination that had become disabled while still
selected.

`should_update_selection` now allows the same-value Enter write only when the
selected ID is present and enabled in the current sidebar rows. Navigation
selection changes remain unchanged. The existing disabled-current regression
now checks that Enter does not request an owner write, while the enabled Enter
test preserves the activation case.

Focused evidence: `./scripts/cargo-headstart test -p rustui --lib
components::navigation::sidebar::tests::disabled_current_and_all_disabled_rows_do_not_activate_or_clear_owner
--offline` passed (1 test). This source-level check does not establish native
keyboard or screen-reader acceptance.
