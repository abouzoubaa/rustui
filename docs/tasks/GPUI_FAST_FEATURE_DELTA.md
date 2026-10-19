# GPUI Fast feature delta against the pinned RustUI stack

Source audit and validation status, 2026-10-18. The original audit was read-only. The exact quad-only V1 patch was validated in an isolated candidate and remains unpromoted. Its earlier development-profile measurements and bounded native visual observation remain recorded in `docs/tasks/QUAD_SORT_PATCH_PROPOSAL.md`; resize acceptance is still pending. A provenance-verified Release-profile batched comparison now shows large dense/reverse gains at 1,024 and 16,384 quads, but 6–11% regressions for every tested 131,072-quad workload. These are CPU workload measurements and establish no frame/GPU/whole-application speedup. V1 remains unpromoted. A concrete V2 replacement proposal and patch are prepared; exact approval is pending, and V2 has not been applied, built, tested, or measured. GPUI Fast remains disabled; active vendor and dependency resolution are unchanged.

Validation evidence includes the corrected isolated harness (9 passed: 7 algorithm, 2 Scene integration), successful baseline/candidate production builds, and the upstream full-library test build failure (23 compile errors; zero quad tests executed). Release ratios, source scope, provenance checks, and V2 identities are recorded in `docs/tasks/QUAD_SORT_PATCH_PROPOSAL.md`. The hardened batched-comparator test suite exited 0 (`target/headstart/candidates/quad-stable-sort-20261018-a/evidence/batched-comparator-tests.log`).

## Verified reference and comparison boundary

The public reference is
[longbridge/gpui-fast at fa133a1dd970532dce4ffaee8577438971e5dec9](https://github.com/longbridge/gpui-fast/tree/fa133a1dd970532dce4ffaee8577438971e5dec9).
Its recursive GitHub tree and pinned source files were read through the GitHub
connector. Source blobs were requested as base64 for exact code inspection;
returned blob identities were checked against tree entries for the principal
integration files. No repository was cloned, dependency source installed, or
vendor byte changed. Advertised performance numbers were not used as evidence.

[UPSTREAM:5–10](https://github.com/longbridge/gpui-fast/blob/fa133a1dd970532dce4ffaee8577438971e5dec9/UPSTREAM#L5-L10)
declares Zed base `93d745668805029476e6d7fbb19652116fed51de` and an unchanged
import commit `89ed5c77614b643c5ed1601900ce22b3c2c5cb8e`. The declared Zed base
matches RustUI's approved `gpui-pre = 0.3.9` snapshot. The former statement that
these used different Zed revisions was inaccurate. Longbridge packaging and Fast
modifications remain different source trees; a shared declared base does not
make their current files interchangeable.

The comparison target is the actual local `vendor/gpui-pre-0.3.9/src/` source,
plus RustUI's current cache adapter. GPUI Fast is research input and is not the
dependency or upstream source of truth. Findings concern inspected source paths,
not runtime acceptance or every backend in the fork.

## Actual feature delta

| Feature | Current pinned GPUI / RustUI | Verified Fast implementation and remaining delta |
| --- | --- | --- |
| Retained view/subtree cache | `view.rs:243–251` exposes `Entity::cached(style)` with definite-size layout. `prepaint_view` at `515–566` tests bounds, mask, text style, dirty view and refresh, then replays cached ranges. RustUI `ComponentView::cached_fixed` uses this only when typed environment is empty, native drag is absent and AX is inactive. | [fast/retained.rs](https://github.com/longbridge/gpui-fast/blob/fa133a1dd970532dce4ffaee8577438971e5dec9/crates/gpui/src/fast/retained.rs#L559-L643) adds default retained records and reuse for ordinary views, context checks including opacity/composition surface, dependency-sensitive reuse, and retained layout. This is much broader than exposing the already-existing explicit fixed-size cache. |
| Dependency tracking | The existing `app/entity_map.rs` tracks accessed entity IDs, while window/entity invalidation and notification support the explicit cache. RustUI owner mutation/props reconciliation notify GPUI hosts; typed environment refresh still makes the cache adapter fall back. | [fast/dependencies.rs:254–270,460–560](https://github.com/longbridge/gpui-fast/blob/fa133a1dd970532dce4ffaee8577438971e5dec9/crates/gpui/src/fast/dependencies.rs#L460-L560) records/replays entity, global, versioned-state and offset dependencies. It distinguishes notifications, updates and writes during rendering, including behavior inside notified ancestors. This requires hooks into App/EntityMap/global/state/scroll operations, not a standalone memoization wrapper. |
| Incremental layout across frames | `taffy.rs:55–104` clears the Taffy tree and allocates ordinary/measured nodes. Taffy's own computation caches do not imply that GPUI retains those nodes across ordinary frame clears. Explicit fixed-size cached views avoid rebuilding their subtree through a different path. | [fast/layout.rs:972–1088](https://github.com/longbridge/gpui-fast/blob/fa133a1dd970532dce4ffaee8577438971e5dec9/crates/gpui/src/fast/layout.rs#L972-L1088) claims stable layout keys, reuses nodes, applies changed styles/children, and retains measured-node inputs. `replay_pending_measures` and measurement logs recheck arbitrary closure results before deciding whether to dirty layout. Parent detachment, duplicate keys and rem/scale/style fingerprints are part of the mechanism. |
| Text measurement and shaping | `elements/text.rs:634–654` requests a measured-layout closure. `text_system/line_layout.rs` already has current/previous frame shaped-line caches, font-generation invalidation, and nonallocating hash-based probes. Text/layout caching is therefore not wholly missing here. | [fast/text.rs:194–252](https://github.com/longbridge/gpui-fast/blob/fa133a1dd970532dce4ffaee8577438971e5dec9/crates/gpui/src/fast/text.rs#L194-L252) carries measurements into retained layout nodes and distinguishes shape changes from decoration changes. `carry_over_line_layouts` at `556–584` preserves still-held lines across frames; `RecentShapes` at `732–852` adds bounded recent shaping reuse. These add invalidation and lifetime rules beyond the current shaped-line cache. |
| Draw-data / scene reuse | Native `Window::reuse_prepaint`, `reuse_paint` and `Scene::replay` already replay ranges and input/dispatch state. `Scene::finish` still sorts primitive vectors through stable `sort_by_key`. | Fast retains additional subtree records and replays accessibility operations/actions and native window-control hitboxes, e.g. [retained.rs:1180–1191](https://github.com/longbridge/gpui-fast/blob/fa133a1dd970532dce4ffaee8577438971e5dec9/crates/gpui/src/fast/retained.rs#L1180-L1191), plus shifted scroll-layer ranges. [fast/bounds_tree.rs:424–500](https://github.com/longbridge/gpui-fast/blob/fa133a1dd970532dce4ffaee8577438971e5dec9/crates/gpui/src/fast/bounds_tree.rs#L424-L500) reuses draw-order assignments from prior bounds insertions. [fast/scene.rs](https://github.com/longbridge/gpui-fast/blob/fa133a1dd970532dce4ffaee8577438971e5dec9/crates/gpui/src/fast/scene.rs#L41-L162) optimizes primitive sorting. No separate `DrawData`/`draw_data` core API was found in the inspected scene/window/fast paths; these concrete mechanisms should be named instead. |
| Scroll-layer retention | Current overflow scrolling, clipping and handle state exist. This is not a retained raster/tile layer that can move without replaying child prepaint/paint. | [fast/layers/paint.rs:1–15,41–71](https://github.com/longbridge/gpui-fast/blob/fa133a1dd970532dce4ffaee8577438971e5dec9/crates/gpui/src/fast/layers/paint.rs#L1-L71) implements Bypass/Repaint/Composite decisions and 512-device-pixel tiles. Policy checks jumps, resize, changed dependencies, work budgets and animation. It integrates scenes, atlas tiles, input, clipping and composition/backend paths; this is not a small RustUI scroll wrapper. |
| Virtual list row reuse | Current `uniform_list` renders its visible range; variable `list` caches item heights/focus and virtualizes. RustUI LazyGrid additionally retains keyed component owners. These do not by themselves retain rendered/prepainted rows between scrolling frames. | [fast/layers/lists.rs:850–880,1252–1295](https://github.com/longbridge/gpui-fast/blob/fa133a1dd970532dce4ffaee8577438971e5dec9/crates/gpui/src/fast/layers/lists.rs#L850-L880) plans row runs, renders only required runs, and carries row prepaint/paint/layout records. It bypasses unsupported flipped/invalid-size cases and disables list layers under active AX (`83–84`). Its row dependency/invalidation logic is substantial and tied to layers. |

### Important interaction limits in the actual reference

- `Window::reusable_record` in Fast `retained.rs:578–587` rejects active native
  drag. `fast/layers/mod.rs:125–132` also rejects active drag. **Selective cache
  retention during drag is not demonstrated by this reference** and must not be
  described as an already-implemented feature ready to copy.
- Fast `fast/a11y.rs:190–212` detects a stale accessibility recording and replays
  recorded prepaint operations; paint actions are also replayed. RustUI's
  conservative AX fallback remains required because the pinned implementation
  does not thereby gain that Fast replay infrastructure. Disabling the guard is
  not an implementation of accessibility-compatible caching.
- Retaining GPUI entity reads does not observe arbitrary interior mutation,
  external state or application semantics automatically. Fast's notification,
  write-generation, global and state-version rules must be evaluated as a whole.

## First isolated code feature: quad-only stable draw-order sorting

**Candidate: a CPU-side stable draw-order counting/permutation sort for quads
only in an isolated patched GPUI candidate.** This is a real Fast implementation
mechanism with a narrow semantic reference and no extra crate requirement. It
does not require the retained/dependency/layout/layer system or native GPU
composition. It was subsequently approved for isolated validation. V1 remains unpromoted after the mixed Release-profile results. A complete V2 replacement proposal is awaiting approval and has not been applied, built, tested, or benchmarked.

Why this slice:

- The pinned `Scene::finish` at `scene.rs:151–162` currently uses
  `self.quads.sort_by_key(|quad| quad.order)`.
- Fast's `sort_by_draw_order` at `fast/scene.rs:56–115` detects already ordered
  input, uses stable counting when the maximum draw order is no more than four
  times the primitive count, and otherwise sorts packed order/index keys. A
  gather pass clones primitives into retained scratch vectors and swaps them.
- A quad-only first patch preserves the existing comparator and equal-order
  insertion order exactly. It leaves all other primitive kinds and sprite
  sorting untouched. The generic helper can be expanded only after this bounded
  case is accepted; no full `fast` module should be transplanted.
- Fast sprite sorting is **not** an equivalent drop-in comparator: its
  `sort_sprites` uses `(order, texture_id.index, tile_id.0, original_index)`.
  The pinned implementation uses `(order, tile_id)`. Changing sprite batching
  deserves its own semantic review, especially across atlas textures and alpha
  blending. It is excluded from this first patch.

Concrete future candidate shape:

1. The exact versioned candidate patch was approved for isolated validation and
   is recorded at `patches/proposed/gpui-pre-quad-stable-sort.patch`. Its only
   source target is the isolated candidate package's `src/scene.rs`; active
   vendor and dependency resolution remain unchanged.
2. Add private reusable scratch storage to `Scene` and a private quad-order
   helper called from `Scene::finish`. Keep stock `sort_by_key` as the explicit
   fallback/reference. Standard `Vec`, integer keys and existing types suffice;
   no dependency or feature addition is justified.
3. Guard index/length conversion rather than truncating `usize` to `u32`.
   Bound counting storage by both order spread and a reviewed absolute memory
   budget. Use the ordinary stable sort when either bound fails. Preserve the
   pristine package checksum metadata; do not rewrite it for patched bytes.
4. Evaluate correctness before performance and retain an independent switch or
   candidate variant that restores stock sorting. Do not silently enable a
   candidate in the active RustUI dependency graph.

### Correctness checks and risks

- Compare the complete ordered quad sequence against stable `sort_by_key`,
  including original insertion identity and all quad payload fields: empty,
  singleton, already sorted, reverse, duplicate-heavy, sparse orders,
  `u32::MAX`, randomized deterministic inputs, and counting/fallback boundaries.
- Repeat `Scene::clear`/reuse cycles and alternate large/small/empty scenes to
  expose stale scratch state. Confirm `paint_operations`, bounds-tree assignment,
  layer-stack behavior, batching and all non-quad vectors remain unchanged.
- Native review must cover overlapping translucent quads, clipping, borders,
  shadows, text/images mixed with quads and nested layers in the same scene.
  Equal-order changes can alter blending even when counts match.
- The gather path clones/stores primitives and keeps scratch capacity. It may
  increase memory or regress small/already-sorted workloads; stdlib stable sort
  already handles some ordered inputs efficiently. Dense-order hypotheses do
  not establish a benefit. Capture allocation/memory and CPU variance separately;
  no frame/GPU/presentation improvement follows from CPU sorting alone.
- This first slice intentionally does not change Fast's bounds-tree replay,
  sprite grouping, path copying, retained views, text measurement, layout or
  scroll layers. Each has distinct correctness and integration requirements.

## Authority gates and immediate stop conditions

Current instructions permit this audit and report, not patch application.
`AGENTS.md` requires an explicitly approved versioned internal patch process;
`patches/README.md` records no installed patches. The proposed upstream
validation pipeline remains a design with unresolved source-closure/promotion
decisions. Before source work, approve the exact patch scope, isolated snapshot
and target layout, checksum/license preservation, workload/reviewer ownership,
and failure/removal policy. Build, native verification and any performance run
must be explicitly included in the subsequent task.

Dependency revision changes and patch promotion need their own approval. The
quad-sort candidate changes CPU preprocessing; it does not imply approval for
advanced GPU work, scroll raster layers, native composition, renderer/backend
replacement, or adopting the Fast fork. If evaluation requires such work, stop
at the boundary and propose that phase separately.

## Local source evidence fingerprints

These SHA-256 values were read from the pristine local files during the audit:

| `vendor/gpui-pre-0.3.9/src/` path | SHA-256 |
| --- | --- |
| `scene.rs` | `3addca59aab12ca0e00acb1e39d652863a8904146db2e28192275660aa5b8880` |
| `view.rs` | `0d70c1445e8f4db723eaa52fb3321850dc7daaf1c08b5385e57cfca3e525cd47` |
| `taffy.rs` | `dd561fcc6b693cfcd15b343b5e2935b9cff2ee8fb373d5e7166ca71ff1f74880` |
| `elements/text.rs` | `3ffcc422356924ae4d6eef864421f56588be65cd931c8fe35d7e5e6438c23346` |
| `text_system/line_layout.rs` | `a78d230d050bbf17bd968612da1d84f4c8aaefc39f82100e11b2b3a6990d45e4` |

No Cargo, tests, benchmark, native check or patch application ran as part of the
initial source audit. Later isolated candidate preparation and validation are
tracked in `docs/tasks/QUAD_SORT_PATCH_PROPOSAL.md`: the upstream library-test
build failed before quad tests. The corrected isolated harness passed 9 tests and baseline/candidate production builds succeeded. Five development-profile pairs and five Release-profile batched pairs are recorded in `docs/tasks/QUAD_SORT_PATCH_PROPOSAL.md`. The optimized results are mixed by workload size; V1 remains unpromoted and V2 review is pending.
