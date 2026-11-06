# M1258 — Stage 10: bound P3G cleanup traversal depth

`PinnedDir.remove_tree` used recursive traversal, so a sufficiently deep
candidate tree could exhaust the Python call stack or directory descriptors
during cleanup.

Cleanup now walks an explicit stack with a constant number of active directory
descriptors, independent of nesting depth. It verifies directory identity
when opening children, while ascending to remove them, and before removing the
root entry. The existing no-follow behavior and replacement-directory
rejection remain in place.

The 1,100-level low-file-descriptor cleanup regression and the focused
filesystem suite passed **22 tests**; Python bytecode compilation passed. This
closes a local cleanup resource gap only. No candidate was executed; candidate
correctness, GUI/performance acceptance, independent review, approval, and
promotion remain open.
