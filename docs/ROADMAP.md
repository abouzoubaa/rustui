# RustUI roadmap after 0.1.0

Work one item at a time; each item becomes a ticket in docs/TICKETS.md before work starts.

## Phase 0 Hygiene

1. Remove retired coordination scripts.
2. Verify third-party licenses and notices.
3. Reconsidered (G0 plan): GPUI duplication stays, guarded by verify_owned_gpui_sources.py; Git-consumer download size is solved by item 18 (crates.io), not by deleting files. Headstart aliases removed in G11; target/headstart directory name kept.
4. Delete merged branches.

## Phase 1 Quality debt (0.2.0)

6. Triage proposed patches (P0) and promote them.
7. Done (R15/R15b): disabled-control focus, dropdown/popover re-enable, popup-menu disabled accessibility and live selection recovery; see docs/DISABLED_AUDIT.md.
8. Done (R20): async task ownership, cancellation, generation-safe restart, error recovery and owner release verified for both public starters; see docs/ASYNC_AUDIT.md.
9. Done (T9R/R26): framework tools reviewed and native live theme reload verified with deterministic rendered-window, recovery and unmount tests; the external adapter remains deferred per docs/TOOLS_REVIEW.md.
10. G10: gallery modules done (103 tests and identical registration-order dump); test locations pending; file-drop description pending; cell-header relation pending.
11. Review parked work: `wip/stash-2026-10-26` and a decision on `wip/native-macos-popups`.

<!-- Preserve item 18's number when rendered as Markdown. -->

18. Publish to crates.io (moved up: also fixes Git-consumer download size).

## Phase 2 Localization

12. Complete French and Arabic catalogs, RTL acceptance, and IME/dead-key checks.

## Phase 3 Performance (GPUI Fast track, 0.3.0)

13. Add benchmarks for scrolling, typing, and dragging.
14. Run measured experiments and adopt only on a clear gain: (a) retained subtree cache and dependency tracking; (b) layout kept across frames; (c) scroll layers; (d) virtual list row reuse; (e) draw-data reuse.

Reference: [GPUI Fast feature delta](archive/tasks/GPUI_FAST_FEATURE_DELTA.md).

## Phase 4 Damage tracking (separate project)

15. Add per-widget damage tracking, starting with append-only panels.
16. Add a drag overlay surface via `raw_window_handle` and `wgpu`.

## Phase 5 Chorus integration (ongoing)

17. Turn gaps found while using RustUI in Chorus into tickets.

## Phase 6 Distribution (1.0)

19. Support other platforms (deferred).
