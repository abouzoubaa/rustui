# RustUI roadmap after 0.1.0

Work one item at a time; each item becomes a ticket in docs/TICKETS.md before work starts.

## Phase 0 Hygiene

1. Delete retired claim scripts: `scripts/roadmap_claims.py`, `scripts/roadmap_session_coordinator.py`, and `scripts/tests/test_roadmap_claims.py`.
2. Verify third-party licenses and notices.
3. Use a single GPUI source: owned crates only; retarget patch checks and remove duplicate vendor copies.
4. Delete merged branches.

## Phase 1 Quality debt (0.2.0)

6. Triage proposed patches (P0) and promote them.
7. Address disabled-control focus (R15).
8. Audit async cancellation (R20).
9. Review framework tools (R22–R26): preview/inspector, composition editing, and live theme reload.
10. Address small gaps: cell-header relation, `.accept()` description, `include!()` gallery, and test file locations.
11. Review parked work: `wip/stash-2026-10-26` and a decision on `wip/native-macos-popups`.

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

18. Publish to crates.io.
19. Support other platforms (deferred).
