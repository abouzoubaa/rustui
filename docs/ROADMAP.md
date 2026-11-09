# RustUI roadmap after 0.1.0

## Stages and releases

- A — finish Phase 1 (items 10, 11, 18) → 0.2.0
- B — theme engine rework (professional, robust; owner-provided materials) → 0.3.0
- C — components: chat bubbles + message list, diff viewer, Markdown/rich text, dock/panel layout, command palette, charts/plots, status bar, time picker, accordion, hover card, kbd, form layout, settings panel, rating, pagination, carousel, attachments, description list, async images + SVG → 0.4.0
- D — macOS desktop app features: native menu bar + keyboard shortcut system, clipboard, open/save dialogs, multi-window + window state restore, custom title bar, undo/redo, settings persistence, system notifications, menu-bar extra, drag out to Finder, trackpad gestures → 0.5.0
- E — developer experience: Rust hot reload (research spike first: function hot-patching; fallback: fast rebuild + relaunch with restored state), API docs, getting-started guide, examples, project template, test helpers for apps → 0.6.0
- F — performance, GPUI Fast track (Phase 3) → 0.7.0
- G — damage tracking, inside RustUI (Phase 4) → 0.8.0
- After G: start the UI Studio rewrite on RustUI and finish Chorus, in parallel with the stages below.
- H — localization: French, Arabic, RTL, IME, bidi patches, locale catalog live reload (Phase 2) → 0.9.0
- I — accessibility settings (increase contrast, reduce transparency, text scaling, owner VoiceOver check) + app shipping (code signing, notarization, DMG, auto-update, crash reporting, logging) → 0.10.0
- J — deferred platform 1 (owner starts it explicitly)
- K — iOS (new GPUI platform backend)
- L — Windows
- M — Android (new GPUI platform backend)
- N — Linux → 1.0

Each stage is split into tickets of at most two hours before work starts.

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
10. Done (G10): gallery modules done (103 tests and identical registration-order dump); test locations done (5 Popover and 4 Menu tests); file-drop description done (11 tests, formerly 9); cell-header relation done (actual header NodeIds; 32 grid tests, formerly 31; two GPUI patches promoted).
11. 11a done (S11): old stash, owner presync backup and merged T11 reviewed; no USEFUL recovery items. See [STASH_REVIEW.md](STASH_REVIEW.md). Remaining 11b: review `wip/native-macos-popups`.

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

19. see Stages J–N.
