# G5-3 — RustUI input-to-screen latency

Readiness and merge follow the measured gates, current candidate-mode source CI and review checks in PR #173. This ticket uses the existing `codex/g5-3-rustui-latency` branch from g5-integration. The previous G5-3 results remain in the PR history and the local G5-3 report; they are not relabelled as new measurements.

## G5-3d finish — final measurements

Completed on 2026-11-24 without runtime, patch, dependency or switch changes. The four missing typing-30 captures are complete: pair 4 on→off, pair 5 off→on. One new 60-second sanity capture per remaining scene ran on the same frozen post-guard release executable and refresh observer. Both hashes match the original freeze and the completed capture set. Typing-10 and typing-30 pairs 1–3 date from November 23; typing-30 pairs 4–5 and the four sanity scenes date from November 24. Each window follows normal warmup, damage off, PX-3 on for both typing arms, identical buffered logging and getrusage user+system CPU over measured wall time. No compilation or profiling overlapped recording.

Typing comparisons below use five same-build matched off/on pairs; other rows compare the G5-3c candidate median with one final sanity capture. Single sanity captures receive no P verdict. Input starts at the in-process GPUI dispatch marker in `apps/rustui-lab/src/g0.rs`, before `dispatch_keystroke`/`dispatch_event`; it does not include an NSEvent hardware timestamp or OS event delivery. The endpoint is `MTLDrawable.presentedTime`. Hardware-to-handler delivery, ScreenCaptureKit displayTime and observer-free ProMotion behavior are separate G5-L1 measurements.

| workload | baseline | latency before → after ms | process CPU before → after s/s | CPU result |
| --- | --- | --- | --- | --- |
| Typing 10/s | same-build off → on | 49.342 → 24.795 | 0.101494 → 0.106130 | INCONCLUSIVE |
| Typing 30/s | same-build off → on | 46.044 → 24.881 | 0.206444 → 0.226282 | INCONCLUSIVE |
| Hover | G5-3c → final sanity | 29.720 → 29.999 | 0.295655 → 0.294806 | Sanity: no >5% rise (-0.287%) |
| Drag | G5-3c → final sanity | 31.121 → 31.339 | 0.186415 → 0.189249 | Sanity: no >5% rise (+1.520%) |
| Collection | G5-3c → final sanity | 31.120 → 31.496 | 0.392447 → 0.390259 | Sanity: no >5% rise (-0.557%) |
| Log | G5-3c → final sanity | 24.716 → 24.681 | 0.064836 → 0.063441 | Sanity: no >5% rise (-2.152%) |

| typing workload | five paired process CPU/s changes % | median / min / max % | P verdict |
| --- | --- | --- | --- |
| Typing 10/s | +2.304, +1.198, +10.170, +5.422, -3.263 | +2.304 / -3.263 / +10.170 | INCONCLUSIVE |
| Typing 30/s | +9.609, -1.174, +13.286, +12.032, +5.990 | +9.609 / -1.174 / +13.286 | INCONCLUSIVE |

**Measured gates:** PASS. Neither typing rate is REGRESSION, and both medians are below 26 ms. Both CPU verdicts remain INCONCLUSIVE, not gains; typing-30 still has predominantly positive paired CPU changes. No sanity CPU rise exceeds 5%; the largest sanity latency increase is 0.375 ms. Candidate captures have zero lost edits, presentation-order violations and capacity violations.

**Machine/load:** 17/24 captures are labelled **measured on a loaded machine (X% background)**; snapshot background spans 3.920%–22.770%. Top-five process snapshots accompany each local capture. All 40,570 GPU records across warmup, measurement and drain use whole-frame modes with damage off ({'direct': 40570}). There are 3 measured acquisitions over 2 ms. All >1 ms calls, paths, in-flight counts and detach ages remain listed in the completed capture details. The capacity guard is not claimed to eliminate these known waits.

**Validation:** `scripts/check.sh` exits 0: **1,880 total across seven groups — 1,873 passed, 7 ignored** (1,461+7 ignored; 11; 86; 3; 40; 272; 0). Explicit candidate-mode source verification exits 0, including 730 script tests. Both stacks pass 91 ordered patches plus one alternative and 156 exact postimages. No runtime code changed during this finish. Readiness/merge requires green candidate-mode source CI for the report commit and no unanswered reviews; those live results are tracked in PR #173. All behavior switches remain off by default and candidate patches remain unpromoted.

The remaining CPU work is not described as solved: the fold worker still copies the cached list, and the editor sorts/reinstalls it and rebuilds projection. The installed-state-safe fold-list experiment and the separate native/owner string-comparison experiment below remain unimplemented and unmeasured. Cache-applied-result counts are not actual reinstall counts. The passive refresh observer remained enabled in these measurements; whether it changes ProMotion behavior remains a separate measurement question.

## G5-3d owner decisions

Owner accepts G5-3c Collection's +3.69 ms versus G5-3b (31.12 ms median, approximately 15 ms below the pre-G5 control) with CPU GAIN. That latency change no longer blocks adoption. Source CI on g5-integration and PRs targeting it uses explicit candidate verification: approved state, hashed candidate chain, exact reverse. Main and release verification stay approved-only. Candidate promotion waits for the owner's approval when G5 merges to main.

## G5-3d frozen-build outcome (historical pre-guard)

All **24 requested captures** completed on the same frozen release build: five alternating matched pairs per typing rate, plus one 60-second candidate sanity capture each for hover, drag, Collection and log. The timing build is commit 1705496a6. Both typing CPU verdicts are **INCONCLUSIVE**, never GAIN. Both median latency targets pass on that measured build. The follow-on custom-lowering correctness guard was added only after all captures finished; these timings are not silently relabelled as a measurement of that later code. The completed post-guard results are reported below.

| workload | median off / on total CPU s/s | five paired changes % | median / min / max % | P verdict | off → on median display delay ms | on p95 / max ms |
| --- | --- | --- | --- | --- | --- | --- |
| typing-10 | 0.112557 / 0.112428 | -0.115, -10.684, +0.783, -3.544, +7.705 | -0.115 / -10.684 / +7.705 | INCONCLUSIVE | 49.341 → 24.810 | 31.106 / 35.904 |
| typing-30 | 0.208859 / 0.230827 | +21.609, -7.997, +13.859, +3.612, +17.713 | +13.859 / -7.997 / +21.609 | INCONCLUSIVE | 45.982 → 25.109 | 31.224 / 35.986 |

The sanity captures have no paired verdict. Their comparison with G5-3c's five candidate captures is diagnostic, not a same-build CPU claim:

| scene | G5-3c median → current single median ms | G5-3c median CPU s/s → current single CPU s/s | order / lost native edits / capacity violations | measured acquisitions >2 ms |
| --- | --- | --- | --- | --- |
| hover | 29.720 → 30.218 | 0.295655 → 0.297252 | 0 / 0 / 0 | 0 |
| drag | 31.121 → 31.408 | 0.186415 → 0.194826 | 0 / 0 / 0 | 2 |
| acceptance | 31.120 → 30.073 | 0.392447 → 0.394643 | 0 / 0 / 0 | 0 |
| log | 24.716 → 24.685 | 0.064836 → 0.061085 | 0 / 0 / 0 | 0 |

**Machine:** 22/24 captures are labelled **measured on a loaded machine (X% background)**; snapshot background ranged from 2.980% to 24.740%. Exact start/end loads, top-five process lists and raw captures remain local. There was no compilation or profiling during recording. Executable and refresh-observer hashes match before/after. All candidate captures have zero lost edits, order violations and capacity violations. There are 3 measured acquisitions over 2 ms, retained as known outliers; all >1 ms calls are listed below.

**Correctness follow-up after timing:** `body()` alone cannot prove a custom `into_element()` implementation's visible output. The guard now records whether the actual component entered the default body-lowering path, using a scoped, nesting-safe, type-and-instance witness without dereferencing stored addresses. Custom render overrides keep their owner notification; explicit supported native primitives retain their exact descriptor proof. A focused custom-render test covers a native editor plus an additional owner-derived label. Default-body, changed geometry, external observer, dynamic binding-target, normalization, IME and undo tests remain enabled. This is a correctness guard, not the owner's next CPU experiment.

**Next experiment 1 — NOT IMPLEMENTED / NOT MEASURED:** the copied fold vector in `crates/rustui-gpui-component/src/highlighter/draw_first_background.rs` still reaches `apply_highlighter_fold_candidates_quiet` in `crates/rustui-gpui-base/src/input/base/state.rs`. That path snapshots folded ranges and visible gutter markers, reinstalls candidates, and rebuilds the fold projection. Cache-vector identity alone is insufficient: incremental editing can clear candidates or shift installed arrows. Add an opt-in reuse path only when the list currently installed in the editor is proven correct for the current document/fold-map state; keep normal updates for arrow removal, blank-line shifts, wrapping, folding and changed visible colours. Compare blank-line insert/delete, paste, undo and edits within existing folds against a full scan, including the installed arrows/fold projection. Count actual fold-list reinstalls; the current `g53_fold_cache` applied-result count is not a reinstall count. Measure this change alone with G5 draw-first and PX-3 enabled in both arms, damage off, buffered logging, five alternating matched pairs at 10 and 30 keys/s, total CPU/s by the P rule and median presentedTime delay ≤26 ms. No gain is claimed here.

**Next experiment 2 — separate, NOT IMPLEMENTED / NOT MEASURED:** if CPU work remains, remove the whole-document string materialization used only by the native/owner equality check in `crates/rustui/src/components/advanced/code_editor.rs`. Test an exact comparison against the existing text storage; retain the real binding payload, setters/validation, observer guards and authoritative-owner behavior. Run its own pairs rather than combining it with fold-install reuse. There is insufficient time in this unchanged two-hour timebox to implement, oracle-test, build and independently time these experiments.

## G5-3d implementation and method

The new timebox runs from 01:36:28 to 03:36:28 UTC on 2026-11-23. Every runtime behavior remains opt-in, with damage off and PX-3 enabled in both typing arms. Candidate and control use one release build, Rust 1.98.0, buffered logging, identical diagnostic flags, normal warmup and 60-second measured windows. Typing at 10 and 30 keys/s each gets five matched pairs in alternating order; hover, drag, Collection and log each get one candidate sanity capture. A single sanity capture receives no P verdict.

The Kit candidate stores fold candidates against an explicit syntax-tree revision, separate from paint and colour application. When nonstructural edits leave line coordinates unchanged, the cache shares the previous fold-range vector and shifts only byte anchors; it does not repeat row lookups or sorting. Known edits shift previous byte endpoints. With no structural changed ranges, newline/covered-endpoint hazards absent, it reuses those candidates without traversing the tree. Structural or newline edits rebuild the intersecting top-level named subtrees and retain the rest. Unknown edits, missing cache or revision discontinuities prime a full result. Async results must match both the current text and revision, including undo back to identical text. Fold oracles compare with full recomputation after inserts, deletes, paste, undo, delimiter/block creation and removal, newline thresholds, malformed/empty input and edits beside unrelated functions. An unchanged revision visits zero nodes; a local structural change in a 500-function document visits less than one tenth of a full traversal.

Owner-notification dedup is conservative. It compares exact supported view descriptions before and after a native binding effect, confirms that the current native text revision was actually painted and equals the owner value, and consults GPUI's owner observers and cross-window read tracking. Only a native editor or supported Container around it qualifies; for nested hosts, GPUI retained records must also prove that no other view read the owner. Unsupported descriptions, different geometry/properties, normalized/rejected values, unpainted edits, external observers and other-window readers retain normal notifications. Binding/validation callbacks still run. A suppressed notification silently updates the retained owner marker and latest binding writer, so subsequent edits cannot restore stale owner text or write to a previous binding target. Colours retain their visible-span change check. Frame-cause masks and suppressed-owner/fold-visit counts are captured separately.

The source CI selector is tested for G5-targeted PRs, integration pushes, main, manual runs and misleading branch names. It invokes the explicit candidate source verifier only for the G5 integration context. That verifier reuses the live source gate, anchors Kit candidate inputs to approved checksums, validates manifest post-hashes and proves exact reverse restoration, including absence for new files. The release verifier itself is unchanged.

## G5-3d interim fold-row remapping result

The first complete five-pair 10 keys/s set, before exact fold-vector reuse, was **REGRESSION** in total process CPU/s: **+8.003%, +10.690%, +16.472%, +7.938%, +27.593%** (median +10.690%, min +7.938%, max +27.593%). Candidate median display delay was **24.793 ms**, versus **49.343 ms** control. Zero syntax nodes were visited by applied fold results, but the cache still rebuilt/sorted fold rows. The refinement removes that work for structurally unchanged edits with unchanged line positions; the oracle also asserts shared fold-vector identity. This interim set is preserved separately and is not combined with the final retest. The following 30 keys/s control was interrupted before a complete capture; it is excluded and has no verdict. The earlier interrupted capture before the nested-owner fix also has no verdict.

## G5-3c outcome (historical)

**NOT COMPLETE — PR #173 stays draft, with every switch off by default.** All 60 measured captures finished: five matched off/on pairs for each of the six workloads, alternating off→on / on→off / off→on / on→off / off→on. Both typing CPU verdicts are REGRESSION. Collection also misses the maximum 2 ms latency increase relative to G5-3b. No exception or adoption decision is made here.

| workload | five process CPU/s paired changes % | median change % | P verdict | off → on median display delay ms | G5-3b on median ms | increase from G5-3b ms | latency margin |
| --- | --- | --- | --- | --- | --- | --- | --- |
| typing 10/s | +28.731, +35.614, +44.163, +20.916, +47.017 | +35.614 | REGRESSION | 49.344 → 24.813 | 23.080 | +1.733 | PASS |
| typing 30/s | +35.553, +30.827, +39.681, +32.258, +34.261 | +34.261 | REGRESSION | 45.766 → 24.403 | 23.084 | +1.319 | PASS |
| hover | -6.363, -6.512, -4.111, -4.688, -3.874 | -4.688 | GAIN | 45.745 → 29.720 | 30.143 | -0.423 | PASS |
| drag | -8.763, -14.876, -15.608, -8.241, -17.313 | -14.876 | GAIN | 45.744 → 31.121 | 30.826 | +0.295 | PASS |
| F0 Collection | -5.118, -5.944, -4.011, -6.486, -3.350 | -5.118 | GAIN | 45.769 → 31.120 | 27.430 | +3.690 | FAIL |
| log | -4.850, +1.517, -6.312, -11.175, -14.005 | -6.312 | INCONCLUSIVE | 47.247 → 24.716 | 24.623 | +0.093 | PASS |

G5-3b's log reference has only three captures; it is disclosed as historical context, not converted into a five-pair verdict. G5-3c's log set has all five pairs. Quantiles in this report are medians of capture quantiles, not pooled-input quantiles. P verdicts use unrounded changes in getrusage user+sys CPU seconds per measured wall second; CPU per attempt is diagnostic.

Aggregate admission budgets in the candidate arm (measured-window totals divided by edits or provably changed presentation slots; individual-edit maxima are NOT VERIFIED):

| workload | CPU draws | GPU frames | edits | CPU draws/edit | GPU frames/edit | CPU draws/visible version | admission target |
| --- | --- | --- | --- | --- | --- | --- | --- |
| typing-10 | 5782 | 5569 | 3000 | 1.9273 | 1.8563 | N/A | PASS |
| typing-30 | 17222 | 16449 | 9000 | 1.9136 | 1.8277 | N/A | PASS |
| hover | 28921 | 28921 | N/A | N/A | N/A | 1.0010 | PASS |
| drag | 29502 | 29502 | N/A | N/A | N/A | 1.0007 | PASS |
| acceptance | 27984 | 27984 | N/A | N/A | N/A | 1.0007 | PASS |

**Machine load:** 58/60 captures crossed the quiet threshold at start or end. Those captures are labelled **measured on a loaded machine (X% background)** in the per-capture table; X ranged from 1.44% to 27.45% across all snapshots. Top-five process lists are retained locally with every capture. No compilation, profiling or heavy analysis overlapped the timed matrix. The executable and refresh-observer hashes match before and after it. All 233,570 GPU records across measured, warmup and drain phases assert Direct with damage off.

**Correctness:** all candidate captures have zero out-of-order presentations; both typing arms have zero lost native edits; all captures have zero capacity violations. Collection control pair 2 has two lower-version presentations: version 7 at presentedTime 60601.87909308335 s, followed by version 6 at 60601.88742641668 s and again at 60601.89575962501 s. These are later refreshes (8.333/16.667 ms), not equal-time callback ambiguity. The control has the G5-1 switch off; this baseline defect is retained in the results rather than attributed to the candidate guard. Five measured acquisitions exceed 2 ms (three in candidate drag, two in candidate Collection). All 83 calls over 1 ms, including warmup and drain, are listed below with path, pre-call in-flight/pool and detach age. No reserve or acquisition change was attempted.

**Validation in this session:** `scripts/check.sh` exits 0: **1,877 total across seven groups — 1,870 passed, 7 ignored** (groups: 1,458+7 ignored; 11; 86; 3; 40; 272; 0). The current opt-in editor run passes 15 tests, including local text before binding, actual IME commit and undo. The complete Kit oracle fixture passes 27 tests serially, including the original PX-3 oracles and six worker/cache tests. Metal copy/clear/retained/Skip/magenta Partial proofs pass. Native hover old/new-row pixels and the F0 Collection pixel/capture check pass. Both patch stacks pass: 85 ordered patches plus one alternative, 150 exact postimages. Explicit integration source verification exits 0 with 729 script tests. The approved-only `./scripts/verify-release.sh source` exits 1 because it correctly rejects applied integration Kit candidates; the release gate and CI command were not weakened or changed. Consequently approved-only source CI is not green and no merge is permitted.

Remaining implementation work is explicit: real owner binding changes still notify even when the local editor already paints the value; only identical owner-value writes are deduplicated. Removing those notifications without a dependency contract could leave sibling owner UI stale. Highlight capture queries are scoped, but the worker still extracts full-document folds on every edit, as the diagnostic profiles below confirm. These requirements are not marked complete. Collection's extra latency also needs another measured admission change; the current results do not establish the reason for that delay.

## G5-3c residual typing CPU profiles

After all 60 timed captures completed, one 60 s release Samply capture per arm ran at 30 keys/s, PX-3 on, damage off, buffered logging and the same runtime switches. No compilation or other profiling/analysis ran during recording. The existing P0 analyzer resolves symbol sidecars, clips CPU-weighted samples to the normal post-warmup G0 window and includes all app threads. These two profiles are diagnostic and receive no P verdict. Background load was not separately sampled for these profiles; they provide function-cost evidence, not another latency or CPU acceptance comparison. The executable stayed unchanged. Unresolved leaf CPU is 8.97% off and 5.72% on.

Full-tree `extract_fold_ranges_in_range` grows from **4.143 to 60.410 sampled CPU ms/s** (+56.267), while total sampled process CPU grows from **187.735 to 245.835 ms/s** (+58.100). This identifies fold traversal as the largest measured candidate-specific remaining cost. Inclusive time overlaps its tree-cursor children and must not be summed with them. One capture per arm does not establish stable rankings or a projected saving.

| function, self CPU | off ms/s | on ms/s | increase ms/s | group |
| --- | --- | --- | --- | --- |
| `ts_tree_cursor_child_iterator_next` | 9.533 | 44.934 | +35.401 | Other dependencies |
| `__workq_kernreturn` | 29.284 | 60.030 | +30.747 | macOS system |
| `ts_tree_cursor_goto_first_child_internal` | 1.491 | 10.543 | +9.052 | Other dependencies |
| `ts_tree_cursor_goto_sibling_internal` | 2.416 | 10.300 | +7.884 | Other dependencies |
| `ts_query_cursor__advance` | 7.722 | 10.963 | +3.241 | Other dependencies |
| `ts_tree_cursor_current_status` | 5.038 | 6.941 | +1.903 | Other dependencies |
| `gpui_component::highlighter::input_adapter::extract_fold_ranges_in_range::collect` | 0.096 | 1.983 | +1.887 | GPUI Kit |
| `ts_tree_cursor_current_node` | 0.240 | 1.288 | +1.048 | Other dependencies |
| `ts_tree_cursor_goto_next_sibling` | 0.014 | 0.992 | +0.978 | Other dependencies |
| `<smol_str::Repr>::new` | 0.145 | 0.851 | +0.706 | Other dependencies |

**Follow-up, not implemented here:** avoid the full fold-tree walk in `crates/rustui-gpui-component/src/highlighter/draw_first_background.rs` (`parse_scoped`, fold extraction) when one known edit leaves structural changed ranges empty; reuse the existing incrementally shifted fold candidates. Structural edits need bounded recomputation and a fold-map oracle. The cost is in the Kit candidate, so any fix remains a hashed reversible patch. Validate final folds/colours and rerun all six five-pair workloads before accepting it. Expected saving is NOT VERIFIED. No new runtime change was made after the matrix froze; G5-4 was not started.

## G5-3c implementation

The new two-hour timebox started at 23:34:48 UTC on 2026-11-22 and ends at 01:34:48 UTC on 2026-11-23. The complete 30-pair matrix did not meet all gates; PR #173 remains draft. All G5-3b measurements below remain historical; G5-3c captures appear in their own section at the end.

The hashed, reversible `gpui-fast-prebuild-visible-colours-v1-fast` candidate moves admission ahead of CPU scene construction in both runtime drawing paths. Each renderer/surface owns a refresh budget and an early-build lease. The late renderer check consumes that lease, rather than charging scene construction time a second time. Blocked work stays dirty and uses the existing GPUI executor timer. Overlay admission remains independent of the base layer. Tests cover independent surfaces, pending work, and a scene taking 3 ms inside an 8 ms interval.

Under draw-first, identical binding writes no longer notify the owner. Real owner-value changes still notify: the owner can render the same field elsewhere, and suppressing those notifications would leave sibling UI stale. Consequently the stronger request to suppress every owner frame when native text already matches is **not implemented**. Colour completion compares old and new spans over the editor's actual last visible range and notifies only when those spans changed. Draw-cause diagnostics accumulate input/binding/colour bits; mixed causes are reported explicitly, with zero classified as other.

The worker queries the bounded recent viewport and edited lines separately, retaining immutable previous spans when edits and structural changes cannot affect the viewport. It does not match the entire 10,000-line document on each edit. Newly revealed, uncached ranges use the current parsed snapshot's query when requested; this scroll fallback runs on the main thread. Incremental parsing and fold extraction still run on the background executor. The worker now carries PX-3's pinned Rust/grammar/query/generation negative-injection proof, with a single known edit, no structural changed ranges and no old/new parse errors required for reuse. Unknown whole-text replacements discard the old tree. The final-colour oracle covers insertion, deletion, paste, undo-style history, IME text, and distant viewports in a 10,000-line document after a whole-text multiline-comment replacement.

The two launcher failures came from toolchain-plumbing tests invoking the real checkout's approved-only Kit verifier. Their fake compiler setup was valid; the checkout contains integration-only candidate Kit files. Those tests now copy the unchanged launcher and rustdoc wrapper into a temporary fixture, like the existing launcher integration tests. This preserves approved-only default verification and exact candidate checks. Release verification and CI configuration are unchanged. Current CI's approved-only source command still rejects the applied Kit candidate; the explicit integration source command is reported separately.

The original Kit oracle suite has process-global language-registry mutation tests; parallel execution exposed a reuse-counter race. Its complete suite is run serially. A missing theme setup in the new binding test was corrected before the final checks. No benchmark verdict is inferred from these diagnostic test failures.

## Reserve arm retired

The owner dropped `GPUI_FAST_DRAWABLE_RESERVE_TWO`: cadence fell to 76–78 visible versions/s and CPU regressed. A 5.602 ms acquisition occurred with only 1/3 slots occupied. Those observations do not support pool saturation as the explanation for the waits. The reserve patch, its stack entries and its launcher arm are removed. All acquisitions over 2 ms remain known; this ticket adds no acquisition workaround.

## G5-3b implementation and method (historical)

Every new behavior stays off by default. `RUSTUI_DRAW_FIRST=1` prioritizes a completed Kit edit ahead of queued binding/owner/change-listener effects, after the editing entity has been returned to GPUI. It never draws while borrowing that entity. The CodeEditor continues painting the local Kit text while the owner is stale, including a committed IME value. Validation that rejects an edit remains before mutation; queued validation/change listeners run after the prioritized draw.

The same switch moves tree-sitter parsing, injections and highlight capture matching to GPUI's background executor using an immutable text/tree/query snapshot. Only a matching latest-text result can apply. Painting reads an indexed immutable capture cache and resolves current theme styles on the main thread. Pending edits translate visible cached spans without rewriting all document captures, preserving existing text's span positions until the new colours arrive. The first frame can use the preceding span colours. The worker currently matches the full document per edit: its total CPU cost is included in the paired verdict, and the async worker does not reuse PX-3's negative-injection proof.

`GPUI_FAST_IMMEDIATE_COALESCE=1` limits submissions separately per renderer/surface to one platform refresh interval and skips an already-submitted scene generation. Dirty changes declined for the current interval are retried on GPUI's executor; order and source/capacity guards remain per surface. Scene generations distinguish a genuinely new colour update from a keepalive presentation of the same scene. `GPUI_FAST_IMMEDIATE_ANY_CHANGE=1` routes idle timer/append/async invalidation through the same immediate path. Live resize retains its synchronous full-render route.

The off arm uses the normal display-link path (G5-1 immediate presentation and all three new switches off). The on arm enables G5-1 plus all three new switches. This compares total CPU against the original non-immediate baseline, so an improvement over an already-regressed immediate arm cannot conceal a regression. PX-3 is on in both typing arms. All six workloads use one pinned Rust 1.98.0 release build, damage off, buffered logging and identical diagnostics, five matched 60-second pairs after normal warmup, alternating off→on / on→off / off→on / on→off / off→on. No build, profiling or heavy analysis overlaps timing. Corrected background CPU and the top five processes are saved at each capture; a failed quiet check is labelled loaded rather than discarded under the owner's bypass.

The deciding CPU metric for G5-3b is total process CPU seconds per measured wall second. CPU per attempt remains diagnostic because coalescing changes the attempt count. Each capture takes getrusage user+sys snapshots after warmup. Input-to-screen uses MTLDrawable.presentedTime; the passive display-link observer supplies the actual observed refresh cadence. Missed-refresh figures retain the disclosed two-refresh G5-1 control floor. No new control floor or quiet-machine confirmation is claimed.

Workloads: typing 10/s and 30/s (10,000-line Rust editor, PX-3 on), hover, drag, F0 Collection, log. A Kit document-revision witness additionally checks every typing edit, so a matching net final string cannot conceal a lost insert/delete pair. Scroll-offset witnesses discount Collection presentations with no changed scroll position; unrendered intermediate offsets are not fabricated.

## G5-3b keystroke trace (historical)

The first diagnostic trace corrects the starting hypothesis: before the first CPU draw, Kit already held the new local text, while the RustUI owner still held the previous value. The first draw did not acquire/submit a drawable because a display link was reattached during that draw. Binding propagation followed; a later display-link draw submitted. This is a frame-source handoff delay, not evidence that the first draw read the owner's old text. The final paint witness records Kit's actual text length and document revision, independently of the input-version label.

Exact baseline, candidate and bare-example timestamps are filled from the final frozen-build traces below. Short diagnostic captures receive no P verdict.

## G5-3b validation (historical)

`scripts/check.sh`: exit 0, 1,876 total across seven groups (1,869 passed, 7 ignored). Ten editor tests also pass with the opt-in enabled, including drawing before binding effects, IME commit and undo. Five worker/cache colour oracles pass. The Metal copy/clear/retained/Skip/magenta Partial proofs, native hover pixel check and F0 Collection capture check pass. Both stacks pass: 84 ordered patches plus one alternative, 150 exact postimages.

The verifier/measurement focused suite passes 36 tests, including candidate tampering, default rejection, Cargo anchoring and exact restoration with created files absent. Explicit Kit candidate checking passes. The integration source flow runs 728 tests and fails with two errors in unchanged `test_cargo_headstart` cases: `test_explicit_rustc_is_preserved_and_default_stays_headstart` and `test_tool_rustdoc_restores_library_path_and_preserves_caller_override`; both report the unchanged launcher exiting 1. This ticket does not modify that launcher or its tests. The approved-only release source command rejects the applied candidate as required. Source CI is not green, and no merge is made.

There are 56 retained 60-second captures: five matched pairs for each of the first five workloads, three for log. Runtime hashes before/after are identical; windows range from 60.001409 to 60.105747 seconds. All 247,164 raw GPU records are Direct. Recorded state/version checks show no lost edits or capacity violations. Strictly later presentedTime comparisons show zero backwards versions; one equal-time slot remains explicitly ambiguous. Two measured acquisitions exceed 2 ms and are listed below. Corrected starting background load ranges from 5.98% to 27.95%; each capture retains its own quiet/loaded label.

Acceptance fails. Both typing medians improve to approximately 23.08 ms, but every complete workload has a total-process CPU REGRESSION. Typing 10/s submits 8,570 GPU frames for 3,000 edits, so the no-extra-frame target is not met. Hover and drag also miss the 30 ms median target. Log has only three pairs and receives no P verdict; its remaining two pairs were not started within the timebox. PR #173 stays draft; all new behavior remains off by default.

## Approved-only Kit verification

The default `python3 scripts/apply_gpui_kit_09_stack.py --check` accepts approved bytes only. The integration-only path is explicit:

```sh
python3 scripts/apply_gpui_kit_09_stack.py --check --candidate-stack patches/candidate-fast-latency/latency-stack.json
python3 scripts/check_source_candidates.py --candidate-stack patches/candidate-fast-latency/latency-stack.json
```

A pinned source/Cargo-checksum catalogue from the approved v0.5.0 commit covers every file in the two affected Kit crates (Base and Component) and their owned mirrors. All existing approved-stack checks remain in place. Existing approved patch hashes remain authoritative where historical Cargo metadata predates an approved patch; previously unlisted unchanged files require their pre-candidate Cargo hash. No candidate post-image is added to the approved catalogue. Verification reconstructs the approved bytes in a temporary fixture, replays the hashed candidate chain, and reverses it again, including absence of created files. The explicit integration entrypoint reuses every current source-mode check verbatim, supplying the candidate option only to the Kit check. The release verifier and CI are not changed to opt in automatically. Consequently approved-only source verification must reject this applied, unpromoted candidate checkout; its rejection is reported rather than bypassed.

## Equal-time presentation callbacks

During typing 30/s pair 3 off, drawable callbacks for versions 48 and 49 had exactly the same `presentedTime` (43453.11828391668 seconds). Converting each callback clock separately introduced an 83 ns ordering difference. The recorder's initial assertion stopped on that artificial inversion. The saved capture was retained and resumed without a runtime change or recapture. Order now compares strictly later presentedTime values; equal-time callbacks are reported as ambiguous. Visible-version throughput counts one presentation slot per refresh, and excludes overlapping possible-version sets that cannot prove a new visible change. It is a conservative rate, not a pixel claim about which tied drawable won. All saved captures are analyzed with this same corrected method.

## G5-3b matched captures

Quantiles below are medians of the five capture quantiles; maxima are maxima across captures. Input delays use MTLDrawable.presentedTime. Each capture has its own measured background load below. No unavailable callback is counted as physical proof that pixels were absent.

| workload | arm | captures / 5 | input→display median / p95 / max ms | visible versions/s | unmatched / rendered | order violations | lost native edits | >2 ms measured acquisitions | CPU s/s | CPU ms/attempt | draw+present ms/attempt | frames >8.3 ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| typing-10 | off | 5 | 49.254/58.105/70.679 | 9.146 | 5/2741 | 0 | 0 | 0 | 0.111 | 12.151 | 2.140 | 0 |
| typing-10 | on | 5 | 23.080/27.476/35.482 | 9.972 | 0/8570 | 0 | 0 | 0 | 0.328 | 3.412 | 0.564 | 0 |
| typing-30 | off | 5 | 45.769/49.914/68.632 | 26.201 | 6/7902 | 0 | 0 | 0 | 0.201 | 7.678 | 1.807 | 0 |
| typing-30 | on | 5 | 23.084/33.249/35.541 | 29.965 | 0/25193 | 0 | 0 | 0 | 0.746 | 2.549 | 0.404 | 0 |
| hover | off | 5 | 45.768/49.796/54.212 | 92.830 | 2867/35981 | 0 | 0 | 0 | 0.302 | 2.520 | 1.512 | 1 |
| hover | on | 5 | 30.143/36.379/43.312 | 101.437 | 12/30451 | 0 | 0 | 0 | 0.406 | 1.109 | 0.897 | 0 |
| drag | off | 5 | 45.773/49.690/57.704 | 93.411 | 2621/35963 | 0 | 0 | 0 | 0.212 | 1.771 | 0.456 | 1 |
| drag | on | 5 | 30.826/39.663/50.019 | 100.231 | 9/30095 | 0 | 0 | 2 | 0.240 | 0.729 | 0.300 | 1 |
| acceptance | off | 5 | 45.782/49.447/63.067 | 99.184 | 48/35951 | 0 | 0 | 0 | 0.420 | 3.502 | 3.015 | 1374 |
| acceptance | on | 5 | 27.430/34.867/54.099 | 100.000 | 9/30043 | 0 | 0 | 0 | 0.613 | 2.356 | 2.170 | 271 |
| log | off | 3 | 45.794/49.824/57.724 | 9.566 | 3/1726 | 0 | 0 | 0 | 0.069 | 7.200 | 2.996 | 1 |
| log | on | 3 | 24.623/28.978/32.728 | 9.566 | 3/1726 | 0 | 0 | 0 | 0.066 | 6.915 | 2.973 | 1 |

## Paired CPU gate

Deciding metric: getrusage user+sys seconds divided by measured wall seconds. The five pairs alternate off→on, on→off, off→on, on→off, off→on. Negative is less CPU. GAIN: all five ≤−3%; REGRESSION: all five >+3%; NEUTRAL: all five within inclusive ±3%; otherwise INCONCLUSIVE. Incomplete sets have no verdict. CPU per attempt is diagnostic only for this ticket.

| workload | five total-process CPU/s paired changes % | median / min / max % | verdict | five CPU/attempt changes %, diagnostic |
| --- | --- | --- | --- | --- |
| typing-10 | 210.338, 179.364, 173.201, 198.567, 192.393 | 192.393/173.201/210.338 | REGRESSION | -70.300, -73.595, -74.271, -71.532, -72.082 |
| typing-30 | 247.669, 337.505, 293.833, 273.192, 255.529 | 273.192/247.669/337.505 | REGRESSION | -68.838, -59.564, -64.361, -66.964, -68.064 |
| hover | 30.872, 34.138, 33.753, 37.202, 36.968 | 34.138/30.872/37.202 | REGRESSION | -57.335, -55.994, -55.992, -56.017, -54.923 |
| drag | 12.901, 15.004, 14.823, 9.245, 8.396 | 12.901/8.396/15.004 | REGRESSION | -58.605, -57.941, -58.598, -60.284, -61.730 |
| acceptance | 51.875, 46.649, 46.512, 42.958, 45.825 | 46.512/42.958/51.875 | REGRESSION | -33.815, -32.902, -32.595, -31.878, -32.119 |
| log | -3.375, -12.053, -3.908 | -3.908/-12.053/-3.375 | INCOMPLETE | -3.368, -12.049, -3.914 |

## Frame and refresh diagnostics

Throughput is a conservative count of distinct presentedTime slots with provably changed version sets. Multiple callbacks with different versions at exactly the same display time are ambiguous, not an inferred ordering.

The same input version can legitimately acquire later colours. Scene generations distinguish those updates from presenting an already-submitted unchanged scene. The single-refresh rule is per renderer/surface, using the platform refresh interval; it does not claim to read WindowServer private queue state.

| workload | arm | CPU draws / GPU frames / inputs | GPU→display median ms | missed refreshes median / p95 | gaps median / p95 / max ms | same-scene submissions skipped | next-interval deferrals | equal-time ambiguous slots |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| typing-10 | off | 2995/2741/3000 | 38.309 | 2.000/2.000 | 108.333/116.668/133.332 | 0 | 0 | 0 |
| typing-10 | on | 28980/8570/3000 | 20.894 | 0.000/0.000 | 16.667/79.166/83.333 | 0 | 20270 | 0 |
| typing-30 | off | 8995/7902/9000 | 38.439 | 2.000/2.000 | 41.666/41.667/66.666 | 0 | 0 | 1 |
| typing-30 | on | 88210/25193/9000 | 21.269 | 0.000/0.000 | 8.333/16.667/25.000 | 0 | 62423 | 0 |
| hover | off | 29681/35981/36000 | 39.001 | 2.000/2.000 | 8.333/16.666/25.000 | 0 | 0 | 0 |
| hover | on | 110196/30451/36000 | 21.417 | 0.000/1.000 | 8.333/16.667/25.000 | 0 | 79745 | 0 |
| drag | off | 29633/35963/36000 | 39.695 | 2.000/2.000 | 8.333/16.666/25.000 | 0 | 0 | 0 |
| drag | on | 99392/30095/36000 | 22.063 | 0.000/1.000 | 8.333/16.667/25.001 | 0 | 69297 | 0 |
| acceptance | off | 30034/35951/36000 | 38.317 | 2.000/2.000 | 8.333/8.333/25.000 | 0 | 0 | 0 |
| acceptance | on | 78377/30043/36000 | 20.657 | 0.000/0.000 | 8.333/16.667/41.666 | 0 | 48334 | 0 |
| log | off | 1726/1726/1800 | 36.772 | 2.000/2.000 | 104.166/108.334/116.667 | 0 | 0 | 0 |
| log | on | 1726/1726/1800 | 20.070 | 0.000/0.000 | 104.166/108.333/116.666 | 0 | 0 | 0 |

## Observed-refresh missed slots

These labels classify observable trace states, with the inherited two-refresh control floor. “WindowServer late” means GPU-ready and handed off but not observed in the eligible slot; it does not establish compositor causality. Observer gaps are disclosed rather than filled in.

| workload | arm | slots | no new input | frame not ready | no drawable free | presented but replaced | WindowServer late | observer gaps |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| hover | off | 35999 | 5 | 15 | 0 | 3222 | 4905 | 2 |
| hover | on | 36001 | 5 | 4005 | 0 | 16 | 1551 | 1 |
| drag | off | 36002 | 5 | 16 | 0 | 2979 | 4965 | 0 |
| drag | on | 35999 | 6 | 2623 | 0 | 14 | 3288 | 3 |

## Per-capture process CPU and load

Each loaded row means **measured on a loaded machine (X% background)**; X is the start→end background percentage shown. Corrected background excludes Codex, its agents/shells and the measured app. Raw top-five process lists remain with each local capture.

| workload | pair | arm | attempts | wall s | user / sys / total CPU s | CPU s/s | CPU ms/attempt | actual refresh Hz | background start→end % | label |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| typing-10 | 1 | off | 552 | 60.059 | 5.235/1.092/6.327 | 0.105 | 11.462 | 120.000 | 5.980→7.100 | quiet |
| typing-10 | 1 | on | 5768 | 60.060 | 18.523/1.113/19.636 | 0.327 | 3.404 | 120.000 | 10.190→16.800 | loaded |
| typing-10 | 2 | on | 5768 | 60.061 | 18.574/1.106/19.680 | 0.328 | 3.412 | 120.000 | 11.040→18.210 | loaded |
| typing-10 | 2 | off | 545 | 60.042 | 5.903/1.139/7.042 | 0.117 | 12.922 | 120.000 | 17.850→14.490 | loaded |
| typing-10 | 3 | off | 543 | 60.066 | 6.090/1.111/7.201 | 0.120 | 13.261 | 120.000 | 11.270→16.380 | loaded |
| typing-10 | 3 | on | 5760 | 60.005 | 18.600/1.053/19.653 | 0.328 | 3.412 | 120.000 | 7.230→13.480 | loaded |
| typing-10 | 4 | on | 5776 | 60.106 | 18.767/1.162/19.929 | 0.332 | 3.450 | 120.000 | 19.680→20.960 | loaded |
| typing-10 | 4 | off | 550 | 60.025 | 5.431/1.235/6.666 | 0.111 | 12.120 | 120.000 | 18.030→28.510 | loaded |
| typing-10 | 5 | off | 551 | 60.095 | 5.450/1.245/6.695 | 0.111 | 12.151 | 120.000 | 27.950→20.220 | loaded |
| typing-10 | 5 | on | 5768 | 60.066 | 18.520/1.048/19.568 | 0.326 | 3.392 | 120.000 | 16.440→24.720 | loaded |
| typing-30 | 1 | off | 1560 | 60.028 | 10.640/2.238/12.878 | 0.215 | 8.255 | 120.000 | 25.080→14.210 | loaded |
| typing-30 | 1 | on | 17400 | 60.011 | 42.493/2.266/44.759 | 0.746 | 2.572 | 120.000 | 16.670→15.620 | loaded |
| typing-30 | 2 | on | 17366 | 60.005 | 42.719/1.817/44.536 | 0.742 | 2.565 | 120.000 | 9.060→19.890 | loaded |
| typing-30 | 2 | off | 1606 | 60.040 | 8.442/1.744/10.186 | 0.170 | 6.342 | 120.000 | 19.170→20.540 | loaded |
| typing-30 | 3 | off | 1592 | 60.035 | 9.399/1.980/11.380 | 0.190 | 7.148 | 120.000 | 18.330→22.630 | loaded |
| typing-30 | 3 | on | 17593 | 60.036 | 42.853/1.965/44.818 | 0.747 | 2.547 | 120.000 | 16.520→8.610 | loaded |
| typing-30 | 4 | on | 17727 | 60.017 | 43.002/1.965/44.967 | 0.749 | 2.537 | 120.000 | 13.950→24.250 | loaded |
| typing-30 | 4 | off | 1569 | 60.007 | 10.008/2.040/12.047 | 0.201 | 7.678 | 120.000 | 16.030→25.320 | loaded |
| typing-30 | 5 | off | 1575 | 60.036 | 10.421/2.151/12.572 | 0.209 | 7.982 | 120.000 | 21.930→20.390 | loaded |
| typing-30 | 5 | on | 17530 | 60.023 | 42.747/1.939/44.686 | 0.744 | 2.549 | 120.000 | 15.220→17.070 | loaded |
| hover | 1 | off | 7196 | 60.002 | 13.382/5.254/18.636 | 0.311 | 2.590 | 120.000 | 17.500→16.730 | loaded |
| hover | 1 | on | 22073 | 60.002 | 21.419/2.970/24.389 | 0.406 | 1.105 | 120.000 | 6.440→15.730 | loaded |
| hover | 2 | on | 21948 | 60.008 | 21.608/2.734/24.342 | 0.406 | 1.109 | 120.000 | 14.750→21.790 | loaded |
| hover | 2 | off | 7200 | 60.005 | 13.108/5.038/18.146 | 0.302 | 2.520 | 120.000 | 13.480→7.300 | loaded |
| hover | 3 | off | 7196 | 60.001 | 13.116/5.043/18.159 | 0.303 | 2.523 | 120.000 | 15.130→16.260 | loaded |
| hover | 3 | on | 21873 | 60.008 | 21.305/2.985/24.290 | 0.405 | 1.111 | 120.000 | 14.450→15.400 | loaded |
| hover | 4 | on | 22443 | 60.002 | 21.473/2.948/24.422 | 0.407 | 1.088 | 120.000 | 13.110→16.880 | loaded |
| hover | 4 | off | 7195 | 60.005 | 12.954/4.847/17.801 | 0.297 | 2.474 | 120.000 | 9.200→23.860 | loaded |
| hover | 5 | off | 7194 | 60.009 | 13.017/4.791/17.808 | 0.297 | 2.475 | 120.000 | 19.540→16.030 | loaded |
| hover | 5 | on | 21859 | 60.008 | 21.519/2.872/24.391 | 0.406 | 1.116 | 120.000 | 14.460→18.640 | loaded |
| drag | 1 | off | 7192 | 60.008 | 6.245/6.491/12.736 | 0.212 | 1.771 | 120.000 | 14.980→17.320 | loaded |
| drag | 1 | on | 19614 | 60.003 | 8.514/5.864/14.377 | 0.240 | 0.733 | 120.000 | 14.410→14.160 | loaded |
| drag | 2 | on | 19651 | 60.009 | 8.462/5.859/14.321 | 0.239 | 0.729 | 120.000 | 14.230→16.440 | loaded |
| drag | 2 | off | 7186 | 60.003 | 6.186/6.265/12.451 | 0.208 | 1.733 | 120.000 | 18.450→15.280 | loaded |
| drag | 3 | off | 7197 | 60.008 | 6.260/6.397/12.657 | 0.211 | 1.759 | 120.000 | 13.380→20.110 | loaded |
| drag | 3 | on | 19961 | 60.011 | 8.656/5.878/14.534 | 0.242 | 0.728 | 120.000 | 18.160→16.420 | loaded |
| drag | 4 | on | 19791 | 60.003 | 8.669/5.865/14.533 | 0.242 | 0.734 | 120.000 | 22.650→20.720 | loaded |
| drag | 4 | off | 7195 | 60.003 | 6.546/6.758/13.304 | 0.222 | 1.849 | 120.000 | 13.570→15.060 | loaded |
| drag | 5 | off | 7193 | 60.004 | 6.517/6.670/13.188 | 0.220 | 1.833 | 120.000 | 14.440→23.060 | loaded |
| drag | 5 | on | 20375 | 60.008 | 8.526/5.770/14.296 | 0.238 | 0.702 | 120.000 | 17.360→26.220 | loaded |
| acceptance | 1 | off | 7191 | 60.010 | 9.759/15.503/25.262 | 0.421 | 3.513 | 120.000 | 17.180→17.310 | loaded |
| acceptance | 1 | on | 16499 | 60.002 | 13.894/24.468/38.362 | 0.639 | 2.325 | 120.000 | 13.250→14.360 | loaded |
| acceptance | 2 | on | 15706 | 60.008 | 13.763/23.244/37.007 | 0.617 | 2.356 | 120.000 | 14.060→16.110 | loaded |
| acceptance | 2 | off | 7186 | 60.007 | 9.866/15.368/25.234 | 0.421 | 3.512 | 120.000 | 21.660→15.360 | loaded |
| acceptance | 3 | off | 7194 | 60.010 | 9.688/15.420/25.108 | 0.418 | 3.490 | 120.000 | 18.050→20.190 | loaded |
| acceptance | 3 | on | 15637 | 60.010 | 13.904/22.882/36.786 | 0.613 | 2.353 | 120.000 | 21.570→21.650 | loaded |
| acceptance | 4 | on | 15093 | 60.003 | 13.056/22.949/36.005 | 0.600 | 2.386 | 120.000 | 18.700→23.660 | loaded |
| acceptance | 4 | off | 7192 | 60.002 | 9.910/15.275/25.185 | 0.420 | 3.502 | 120.000 | 20.360→23.570 | loaded |
| acceptance | 5 | off | 7188 | 60.008 | 10.251/14.754/25.005 | 0.417 | 3.479 | 120.000 | 24.240→21.690 | loaded |
| acceptance | 5 | on | 15442 | 60.009 | 13.526/22.939/36.465 | 0.608 | 2.361 | 120.000 | 20.470→28.030 | loaded |
| log | 1 | off | 575 | 60.012 | 3.037/1.104/4.140 | 0.069 | 7.200 | 120.000 | 22.740→19.510 | loaded |
| log | 1 | on | 575 | 60.017 | 2.996/1.004/4.001 | 0.067 | 6.958 | 120.000 | 19.660→19.640 | loaded |
| log | 2 | on | 575 | 60.003 | 2.982/0.995/3.976 | 0.066 | 6.915 | 120.000 | 20.760→15.600 | loaded |
| log | 2 | off | 576 | 60.105 | 3.430/1.099/4.529 | 0.075 | 7.863 | 120.000 | 13.090→16.520 | loaded |
| log | 3 | off | 575 | 60.004 | 3.105/1.007/4.112 | 0.069 | 7.152 | 120.000 | 13.950→15.500 | loaded |
| log | 3 | on | 576 | 60.104 | 3.029/0.929/3.958 | 0.066 | 6.872 | 120.000 | 13.630→18.270 | loaded |

## Known acquisitions over 2 ms

All phases are listed. No acquisition workaround is adopted from these numbers. Each entry gives phase, path, duration, pre-call live/pool, attached state and milliseconds since display-link detach.

- typing-10 pair 2 off: warmup, non-immediate, 4.339375 ms, 0/3, attached=False, detach age=N/A ms.

- hover pair 1 off: warmup, non-immediate, 4.045583 ms, 0/3, attached=False, detach age=N/A ms.

- hover pair 3 off: warmup, non-immediate, 4.582125 ms, 0/3, attached=False, detach age=N/A ms.

- drag pair 2 on: measured, immediate, 2.039416 ms, 1/3, attached=False, detach age=N/A ms.

- drag pair 2 on: measured, immediate, 2.634208 ms, 2/3, attached=False, detach age=N/A ms.

- acceptance pair 5 off: warmup, non-immediate, 4.066167 ms, 0/3, attached=False, detach age=N/A ms.

- acceptance pair 5 on: warmup, non-immediate, 2.059958 ms, 0/3, attached=False, detach age=N/A ms.

## Single-keystroke timestamps

Diagnostic traces only; these short recordings have no P verdict. All values are milliseconds after the first handled key, measured with mach_absolute_time. Presented delay uses the drawable display time. “Not instrumented” is retained where an older trace did not have the final paint witness.

Bare G5-1 immediate: key tick 1017892846413, timebase 125/3; RustUI G5-1 immediate diagnostic: key tick 970769878496, timebase 125/3; RustUI new opt-in: key tick 1017392123632, timebase 125/3; RustUI new opt-in: submitted draw 38 painted local length 248395, document revision 2.

| stage | Bare G5-1 immediate | RustUI G5-1 immediate diagnostic | RustUI new opt-in |
| --- | --- | --- | --- |
| Kit local text applied | Not instrumented / not applicable | 0.303542 | 0.302125 |
| Highlighter update returns | Not instrumented / not applicable | 1.478875 | 0.309625 |
| Kit Change queued | Not instrumented / not applicable | 1.591375 | 0.493792 |
| Prioritized edit-frame callback | Not instrumented / not applicable | Not instrumented / not applicable | 0.503250 |
| RustUI Change listener / binding starts | Not instrumented / not applicable | 3.200542 | 3.920667 |
| Binding write / owner notify returns | Not instrumented / not applicable | 3.243583 | 3.981750 |
| Background colours applied | Not instrumented / not applicable | Not instrumented / not applicable | 40.000417 |
| Bare local text applied | 1.480042 | Not instrumented / not applicable | Not instrumented / not applicable |
| First CPU draw | 0.028250 | 1.640042 | 0.508167 |
| First GPU command committed | 4.039125 | 5.101667 | 2.876625 |
| GPU-completed callback | 6.856833 | 6.755125 | 4.191917 |
| Drawable display time | Not instrumented / not applicable | Not instrumented / not applicable | 20.743750 |

In the older RustUI immediate trace, the local Kit text had changed before the first CPU draw, while the owner value was old. A display-link recreate and an acquisition attempt with attached=true occurred during that draw; it produced no GPU commit. The later display-link draw submitted. The final opt-in trace separately witnesses the actual local text revision in the submitted paint. Binding/owner effects occur after the prioritized CPU draw attempt; if admission declines, this implementation does not hold those effects until a successful submission. That stronger guarantee remains unproven.

## Findings requiring another iteration

The per-surface cadence check runs inside MetalRenderer, after Window::draw has built the CPU scene. It therefore limits submissions while still paying for declined CPU draws. Scene generations also advance for binding/colour notifications of the same input version; this does not satisfy the requested semantic-version deduplication. The new colour worker generates captures for the full 10,000-line document per edit and always notifies on application. Those are avoidable new costs identified in the implementation; this cohort did not profile individual function costs. Lower CPU per attempt must not be called a gain when total CPU per second regresses. No exception or adoption is recommended.

## G5-3c matched captures

Quantiles below are medians of the five capture quantiles; maxima are maxima across captures. Input delays use MTLDrawable.presentedTime. Each capture has its own measured background load below. No unavailable callback is counted as physical proof that pixels were absent.

| workload | arm | captures / 5 | input→display median / p95 / max ms | visible versions/s (lower bound) | unmatched / rendered | order violations | lost native edits | >2 ms measured acquisitions | CPU s/s | CPU ms/attempt | draw+present ms/attempt | CPU draw+present >8.3 ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| typing-10 | off | 5 | 49.344/61.728/66.518 | 9.201 | 5/2756 | 0 | 0 | 0 | 0.104 | 11.316 | 2.110 | 0 |
| typing-10 | on | 5 | 24.813/31.205/35.934 | 9.973 | 2/5569 | 0 | 0 | 0 | 0.146 | 7.843 | 1.866 | 0 |
| typing-30 | off | 5 | 45.766/49.877/67.856 | 26.210 | 6/7879 | 0 | 0 | 0 | 0.211 | 8.034 | 2.024 | 0 |
| typing-30 | on | 5 | 24.403/30.953/41.411 | 29.972 | 2/16449 | 0 | 0 | 0 | 0.283 | 5.161 | 1.115 | 0 |
| hover | off | 5 | 45.745/49.774/54.089 | 90.377 | 3316/35992 | 0 | 0 | 0 | 0.316 | 2.636 | 1.601 | 0 |
| hover | on | 5 | 29.720/38.976/48.710 | 96.588 | 15/28921 | 0 | 0 | 0 | 0.296 | 3.048 | 1.894 | 1 |
| drag | off | 5 | 45.744/49.672/55.584 | 91.818 | 3215/35988 | 0 | 0 | 0 | 0.218 | 1.817 | 0.451 | 2 |
| drag | on | 5 | 31.121/40.433/49.621 | 98.281 | 8/29502 | 0 | 0 | 3 | 0.186 | 1.905 | 0.518 | 0 |
| acceptance | off | 5 | 45.769/49.814/80.718 | 102.027 | 68/35960 | 2 | 0 | 0 | 0.413 | 3.442 | 2.943 | 1709 |
| acceptance | on | 5 | 31.120/37.699/78.123 | 92.443 | 7/27984 | 0 | 0 | 2 | 0.392 | 4.239 | 3.653 | 2375 |
| log | off | 5 | 47.247/49.814/57.723 | 9.567 | 5/2878 | 0 | 0 | 0 | 0.071 | 7.376 | 3.308 | 2 |
| log | on | 5 | 24.716/29.000/33.081 | 9.567 | 5/2878 | 0 | 0 | 0 | 0.065 | 6.765 | 3.276 | 0 |

## Paired CPU gate

Deciding metric: getrusage user+sys seconds divided by measured wall seconds. The five pairs alternate off→on, on→off, off→on, on→off, off→on. Negative is less CPU. GAIN: all five ≤−3%; REGRESSION: all five >+3%; NEUTRAL: all five within inclusive ±3%; otherwise INCONCLUSIVE. Incomplete sets have no verdict. CPU per attempt is diagnostic only for this ticket.

| workload | five total-process CPU/s paired changes % | median / min / max % | verdict | five CPU/attempt changes %, diagnostic |
| --- | --- | --- | --- | --- |
| typing-10 | 28.731, 35.614, 44.163, 20.916, 47.017 | 35.614/20.916/47.017 | REGRESSION | -36.887, -32.687, -28.231, -40.692, -26.796 |
| typing-30 | 35.553, 30.827, 39.681, 32.258, 34.261 | 34.261/30.827/39.681 | REGRESSION | -35.010, -37.428, -32.718, -36.928, -35.710 |
| hover | -6.363, -6.512, -4.111, -4.688, -3.874 | -4.688/-6.512/-3.874 | GAIN | 16.179, 15.617, 20.759, 19.334, 18.607 |
| drag | -8.763, -14.876, -15.608, -8.241, -17.313 | -14.876/-17.313/-8.241 | GAIN | 11.147, 4.366, 2.485, 12.893, 0.093 |
| acceptance | -5.118, -5.944, -4.011, -6.486, -3.350 | -5.118/-6.486/-3.350 | GAIN | 23.133, 21.812, 22.231, 18.148, 25.313 |
| log | -4.850, 1.517, -6.312, -11.175, -14.005 | -6.312/-14.005/1.517 | INCONCLUSIVE | -4.864, 1.510, -6.305, -11.181, -13.991 |

## Frame and refresh diagnostics

Throughput is a conservative count of distinct presentedTime slots with provably changed version sets. Multiple callbacks with different versions at exactly the same display time are ambiguous, not an inferred ordering.

The same input version can legitimately acquire later colours. Scene generations distinguish those updates from presenting an already-submitted unchanged scene. The single-refresh rule is per renderer/surface, using the platform refresh interval; it does not claim to read WindowServer private queue state.

| workload | arm | CPU draws / GPU frames / inputs | GPU→display median ms | missed refreshes median / p95 | gaps median / p95 / max ms | same-scene submissions skipped | next-interval deferrals | equal-time ambiguous slots |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| typing-10 | off | 2995/2756/3000 | 38.336 | 2.000/2.000 | 108.333/124.998/125.001 | 0 | 0 | 0 |
| typing-10 | on | 5782/5569/3000 | 20.340 | 0.000/0.000 | 16.669/100.000/108.337 | 0 | 0 | 0 |
| typing-30 | off | 8995/7879/9000 | 38.089 | 2.000/2.000 | 41.666/41.667/66.666 | 0 | 0 | 0 |
| typing-30 | on | 17222/16449/9000 | 20.724 | 0.000/0.000 | 16.667/33.333/41.667 | 0 | 0 | 0 |
| hover | off | 29206/35992/36000 | 38.977 | 2.000/2.000 | 8.333/16.666/16.690 | 0 | 0 | 0 |
| hover | on | 28921/28921/36000 | 22.028 | 0.000/1.000 | 8.333/16.667/25.002 | 0 | 0 | 0 |
| drag | off | 29539/35988/36000 | 39.694 | 2.000/2.000 | 8.333/16.666/16.667 | 0 | 0 | 0 |
| drag | on | 29502/29502/36000 | 22.309 | 0.000/1.000 | 8.333/16.667/25.004 | 0 | 0 | 0 |
| acceptance | off | 30442/35960/36000 | 38.328 | 2.000/2.000 | 8.333/8.333/41.666 | 0 | 0 | 0 |
| acceptance | on | 27984/27984/36000 | 22.065 | 0.000/1.000 | 8.333/16.667/58.333 | 0 | 0 | 0 |
| log | off | 2878/2878/3000 | 36.496 | 2.000/2.000 | 104.166/108.333/116.666 | 0 | 0 | 0 |
| log | on | 2878/2878/3000 | 20.073 | 0.000/0.000 | 104.166/108.333/116.665 | 0 | 0 | 0 |

## Observed-refresh missed slots

These labels classify observable trace states, with the inherited two-refresh control floor. “WindowServer late” means GPU-ready and handed off but not observed in the eligible slot; it does not establish compositor causality. Observer gaps are disclosed rather than filled in.

| workload | arm | slots | no new input | frame not ready | no drawable free | presented but replaced | WindowServer late | observer gaps |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| hover | off | 36003 | 6 | 16 | 0 | 3713 | 5060 | 1 |
| hover | on | 36001 | 5 | 3846 | 0 | 20 | 3237 | 0 |
| drag | off | 36002 | 5 | 15 | 0 | 3633 | 4782 | 2 |
| drag | on | 36004 | 7 | 3480 | 0 | 11 | 3026 | 1 |

## Per-capture process CPU and load

Each loaded row means **measured on a loaded machine (X% background)**; X is the start→end background percentage shown. Corrected background excludes Codex, its agents/shells and the measured app. Raw top-five process lists remain with each local capture.

| workload | pair | arm | attempts | wall s | user / sys / total CPU s | CPU s/s | CPU ms/attempt | actual refresh Hz | background start→end % | label |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| typing-10 | 1 | off | 544 | 60.042 | 5.940/1.091/7.031 | 0.117 | 12.924 | 120.000 | 6.720→2.160 | quiet |
| typing-10 | 1 | on | 1109 | 60.010 | 7.687/1.359/9.046 | 0.151 | 8.157 | 120.000 | 5.880→1.440 | quiet |
| typing-10 | 2 | on | 1115 | 60.035 | 7.210/1.283/8.493 | 0.141 | 7.617 | 120.000 | 3.650→16.000 | measured on a loaded machine (3.650%→16.000% background) |
| typing-10 | 2 | off | 554 | 60.095 | 5.184/1.085/6.269 | 0.104 | 11.316 | 120.000 | 12.960→16.750 | measured on a loaded machine (12.960%→16.750% background) |
| typing-10 | 3 | off | 555 | 60.079 | 4.965/1.100/6.065 | 0.101 | 10.928 | 120.000 | 23.410→15.830 | measured on a loaded machine (23.410%→15.830% background) |
| typing-10 | 3 | on | 1115 | 60.088 | 7.457/1.288/8.745 | 0.146 | 7.843 | 120.000 | 9.770→20.100 | measured on a loaded machine (9.770%→20.100% background) |
| typing-10 | 4 | on | 1119 | 60.065 | 6.710/1.197/7.907 | 0.132 | 7.067 | 120.000 | 16.270→22.130 | measured on a loaded machine (16.270%→22.130% background) |
| typing-10 | 4 | off | 549 | 60.080 | 5.448/1.093/6.541 | 0.109 | 11.915 | 120.000 | 26.410→19.790 | measured on a loaded machine (26.410%→19.790% background) |
| typing-10 | 5 | off | 554 | 60.103 | 4.991/1.059/6.050 | 0.101 | 10.921 | 120.000 | 7.240→15.260 | measured on a loaded machine (7.240%→15.260% background) |
| typing-10 | 5 | on | 1111 | 60.016 | 7.556/1.326/8.882 | 0.148 | 7.994 | 120.000 | 7.780→17.050 | measured on a loaded machine (7.780%→17.050% background) |
| typing-30 | 1 | off | 1575 | 60.015 | 10.494/2.159/12.653 | 0.211 | 8.034 | 120.000 | 10.030→11.550 | measured on a loaded machine (10.030%→11.550% background) |
| typing-30 | 1 | on | 3285 | 60.014 | 14.727/2.424/17.151 | 0.286 | 5.221 | 120.000 | 12.470→14.510 | measured on a loaded machine (12.470%→14.510% background) |
| typing-30 | 2 | on | 3285 | 60.023 | 14.876/2.388/17.264 | 0.288 | 5.255 | 120.000 | 23.240→21.140 | measured on a loaded machine (23.240%→21.140% background) |
| typing-30 | 2 | off | 1571 | 60.018 | 10.909/2.286/13.195 | 0.220 | 8.399 | 120.000 | 11.030→15.500 | measured on a loaded machine (11.030%→15.500% background) |
| typing-30 | 3 | off | 1587 | 60.017 | 9.877/2.128/12.005 | 0.200 | 7.565 | 120.000 | 12.360→15.110 | measured on a loaded machine (12.360%→15.110% background) |
| typing-30 | 3 | on | 3295 | 60.022 | 14.399/2.371/16.770 | 0.279 | 5.090 | 120.000 | 11.730→15.080 | measured on a loaded machine (11.730%→15.080% background) |
| typing-30 | 4 | on | 3291 | 60.026 | 14.583/2.401/16.984 | 0.283 | 5.161 | 120.000 | 7.640→19.380 | measured on a loaded machine (7.640%→19.380% background) |
| typing-30 | 4 | off | 1569 | 60.009 | 10.667/2.171/12.838 | 0.214 | 8.182 | 120.000 | 10.520→16.300 | measured on a loaded machine (10.520%→16.300% background) |
| typing-30 | 5 | off | 1577 | 60.031 | 10.459/2.195/12.654 | 0.211 | 8.024 | 120.000 | 19.480→14.680 | measured on a loaded machine (19.480%→14.680% background) |
| typing-30 | 5 | on | 3293 | 60.024 | 14.614/2.374/16.987 | 0.283 | 5.159 | 120.000 | 14.370→16.650 | measured on a loaded machine (14.370%→16.650% background) |
| hover | 1 | off | 7198 | 60.001 | 13.509/5.288/18.797 | 0.313 | 2.611 | 120.000 | 9.920→16.320 | measured on a loaded machine (9.920%→16.320% background) |
| hover | 1 | on | 5802 | 60.007 | 12.844/4.759/17.603 | 0.293 | 3.034 | 120.000 | 7.720→18.320 | measured on a loaded machine (7.720%→18.320% background) |
| hover | 2 | on | 5821 | 60.005 | 12.947/4.794/17.741 | 0.296 | 3.048 | 120.000 | 6.490→17.010 | measured on a loaded machine (6.490%→17.010% background) |
| hover | 2 | off | 7199 | 60.006 | 13.589/5.388/18.977 | 0.316 | 2.636 | 120.000 | 17.520→15.160 | measured on a loaded machine (17.520%→15.160% background) |
| hover | 3 | off | 7198 | 60.010 | 13.654/5.432/19.086 | 0.318 | 2.652 | 120.000 | 8.210→15.240 | measured on a loaded machine (8.210%→15.240% background) |
| hover | 3 | on | 5715 | 60.004 | 13.363/4.937/18.300 | 0.305 | 3.202 | 120.000 | 7.890→13.650 | measured on a loaded machine (7.890%→13.650% background) |
| hover | 4 | on | 5748 | 60.001 | 13.205/4.964/18.169 | 0.303 | 3.161 | 120.000 | 9.720→15.690 | measured on a loaded machine (9.720%→15.690% background) |
| hover | 4 | off | 7197 | 60.004 | 13.753/5.310/19.064 | 0.318 | 2.649 | 120.000 | 12.040→14.580 | measured on a loaded machine (12.040%→14.580% background) |
| hover | 5 | off | 7200 | 60.009 | 13.061/5.129/18.190 | 0.303 | 2.526 | 120.000 | 14.130→11.630 | measured on a loaded machine (14.130%→11.630% background) |
| hover | 5 | on | 5835 | 60.006 | 12.772/4.712/17.484 | 0.291 | 2.996 | 120.000 | 8.860→16.340 | measured on a loaded machine (8.860%→16.340% background) |
| drag | 1 | off | 7191 | 60.010 | 5.974/7.089/13.064 | 0.218 | 1.817 | 120.000 | 7.290→21.440 | measured on a loaded machine (7.290%→21.440% background) |
| drag | 1 | on | 5902 | 60.001 | 5.745/6.172/11.917 | 0.199 | 2.019 | 120.000 | 12.690→10.080 | measured on a loaded machine (12.690%→10.080% background) |
| drag | 2 | on | 5871 | 60.011 | 5.428/5.759/11.187 | 0.186 | 1.905 | 120.000 | 7.710→15.510 | measured on a loaded machine (7.710%→15.510% background) |
| drag | 2 | off | 7198 | 60.010 | 6.001/7.140/13.142 | 0.219 | 1.826 | 120.000 | 11.560→14.260 | measured on a loaded machine (11.560%→14.260% background) |
| drag | 3 | off | 7200 | 60.008 | 6.024/6.860/12.884 | 0.215 | 1.790 | 120.000 | 8.230→16.520 | measured on a loaded machine (8.230%→16.520% background) |
| drag | 3 | on | 5929 | 60.008 | 5.343/5.530/10.874 | 0.181 | 1.834 | 120.000 | 12.840→14.610 | measured on a loaded machine (12.840%→14.610% background) |
| drag | 4 | on | 5851 | 60.008 | 5.614/6.151/11.765 | 0.196 | 2.011 | 120.000 | 16.960→21.400 | measured on a loaded machine (16.960%→21.400% background) |
| drag | 4 | off | 7198 | 60.003 | 5.926/6.895/12.820 | 0.214 | 1.781 | 120.000 | 7.030→14.110 | measured on a loaded machine (7.030%→14.110% background) |
| drag | 5 | off | 7201 | 60.006 | 5.993/7.205/13.199 | 0.220 | 1.833 | 120.000 | 14.660→14.400 | measured on a loaded machine (14.660%→14.400% background) |
| drag | 5 | on | 5949 | 60.009 | 5.330/5.584/10.914 | 0.182 | 1.835 | 120.000 | 10.530→20.820 | measured on a loaded machine (10.530%→20.820% background) |
| acceptance | 1 | off | 7191 | 60.002 | 9.650/15.104/24.754 | 0.413 | 3.442 | 120.000 | 8.820→16.160 | measured on a loaded machine (8.820%→16.160% background) |
| acceptance | 1 | on | 5542 | 60.011 | 9.160/14.331/23.491 | 0.391 | 4.239 | 120.000 | 6.350→14.200 | measured on a loaded machine (6.350%→14.200% background) |
| acceptance | 2 | on | 5552 | 60.005 | 9.137/14.412/23.549 | 0.392 | 4.241 | 120.000 | 19.970→15.230 | measured on a loaded machine (19.970%→15.230% background) |
| acceptance | 2 | off | 7190 | 60.001 | 9.695/15.341/25.035 | 0.417 | 3.482 | 120.000 | 5.490→13.560 | measured on a loaded machine (5.490%→13.560% background) |
| acceptance | 3 | off | 7194 | 60.002 | 9.852/14.693/24.546 | 0.409 | 3.412 | 120.000 | 17.520→18.490 | measured on a loaded machine (17.520%→18.490% background) |
| acceptance | 3 | on | 5650 | 60.008 | 9.154/14.409/23.563 | 0.393 | 4.171 | 120.000 | 4.620→14.530 | measured on a loaded machine (4.620%→14.530% background) |
| acceptance | 4 | on | 5692 | 60.001 | 9.358/14.153/23.510 | 0.392 | 4.130 | 120.000 | 6.460→14.310 | measured on a loaded machine (6.460%→14.310% background) |
| acceptance | 4 | off | 7192 | 60.006 | 9.723/15.420/25.143 | 0.419 | 3.496 | 120.000 | 8.830→13.860 | measured on a loaded machine (8.830%→13.860% background) |
| acceptance | 5 | off | 7193 | 60.004 | 9.772/14.658/24.430 | 0.407 | 3.396 | 120.000 | 5.680→18.180 | measured on a loaded machine (5.680%→18.180% background) |
| acceptance | 5 | on | 5548 | 60.007 | 9.170/14.443/23.613 | 0.394 | 4.256 | 120.000 | 7.830→15.900 | measured on a loaded machine (7.830%→15.900% background) |
| log | 1 | off | 575 | 60.008 | 3.016/1.073/4.089 | 0.068 | 7.111 | 120.000 | 10.380→14.870 | measured on a loaded machine (10.380%→14.870% background) |
| log | 1 | on | 576 | 60.104 | 2.891/1.006/3.897 | 0.065 | 6.765 | 120.000 | 11.040→16.640 | measured on a loaded machine (11.040%→16.640% background) |
| log | 2 | on | 576 | 60.104 | 3.016/1.024/4.041 | 0.067 | 7.015 | 120.000 | 17.710→14.520 | measured on a loaded machine (17.710%→14.520% background) |
| log | 2 | off | 575 | 60.004 | 2.931/1.042/3.974 | 0.066 | 6.911 | 120.000 | 8.800→14.170 | measured on a loaded machine (8.800%→14.170% background) |
| log | 3 | off | 576 | 60.104 | 3.148/1.139/4.288 | 0.071 | 7.444 | 120.000 | 8.930→16.410 | measured on a loaded machine (8.930%→16.410% background) |
| log | 3 | on | 575 | 60.003 | 2.977/1.034/4.010 | 0.067 | 6.975 | 120.000 | 11.110→13.980 | measured on a loaded machine (11.110%→13.980% background) |
| log | 4 | on | 576 | 60.100 | 2.803/0.970/3.773 | 0.063 | 6.551 | 120.000 | 8.280→14.190 | measured on a loaded machine (8.280%→14.190% background) |
| log | 4 | off | 576 | 60.104 | 3.119/1.130/4.248 | 0.071 | 7.376 | 120.000 | 9.050→16.160 | measured on a loaded machine (9.050%→16.160% background) |
| log | 5 | off | 576 | 60.103 | 3.232/1.158/4.390 | 0.073 | 7.622 | 120.000 | 11.270→14.190 | measured on a loaded machine (11.270%→14.190% background) |
| log | 5 | on | 575 | 60.008 | 2.802/0.968/3.770 | 0.063 | 6.556 | 120.000 | 14.750→27.450 | measured on a loaded machine (14.750%→27.450% background) |

## Acquisitions over 1 ms (all phases)

All calls over 1 ms in all phases are listed; the measured-window >2 ms count remains in the summary table. No acquisition workaround is adopted from these numbers. Each entry gives phase, path, duration, pre-call live/pool, attached state and milliseconds since display-link detach.

- typing-10 pair 1 on: warmup, immediate, 1.024958 ms, 0/3, attached=False, detach age=N/A ms.

- typing-10 pair 2 on: warmup, non-immediate, 1.212083 ms, 0/3, attached=False, detach age=N/A ms.

- typing-10 pair 2 off: warmup, non-immediate, 1.159375 ms, 0/3, attached=False, detach age=N/A ms.

- typing-10 pair 3 off: warmup, non-immediate, 1.235084 ms, 0/3, attached=False, detach age=N/A ms.

- typing-10 pair 3 on: warmup, non-immediate, 1.061667 ms, 0/3, attached=False, detach age=N/A ms.

- typing-10 pair 4 on: measured, immediate, 1.218833 ms, 1/3, attached=False, detach age=N/A ms.

- typing-10 pair 4 off: warmup, non-immediate, 1.172000 ms, 0/3, attached=False, detach age=N/A ms.

- typing-10 pair 5 on: warmup, non-immediate, 1.030917 ms, 0/3, attached=False, detach age=N/A ms.

- typing-10 pair 5 on: measured, immediate, 1.054625 ms, 1/3, attached=False, detach age=N/A ms.

- typing-30 pair 1 on: warmup, non-immediate, 1.145792 ms, 0/3, attached=False, detach age=N/A ms.

- typing-30 pair 2 on: measured, immediate, 1.035042 ms, 1/3, attached=False, detach age=N/A ms.

- typing-30 pair 3 off: warmup, non-immediate, 1.123667 ms, 0/3, attached=False, detach age=N/A ms.

- typing-30 pair 3 on: warmup, non-immediate, 1.050542 ms, 0/3, attached=False, detach age=N/A ms.

- typing-30 pair 4 on: warmup, non-immediate, 1.217625 ms, 0/3, attached=False, detach age=N/A ms.

- typing-30 pair 4 on: measured, immediate, 1.347292 ms, 1/3, attached=False, detach age=N/A ms.

- typing-30 pair 4 off: warmup, non-immediate, 1.044042 ms, 0/3, attached=False, detach age=N/A ms.

- typing-30 pair 5 off: warmup, non-immediate, 1.017375 ms, 0/3, attached=False, detach age=N/A ms.

- typing-30 pair 5 on: warmup, non-immediate, 1.078541 ms, 0/3, attached=False, detach age=N/A ms.

- hover pair 1 on: warmup, non-immediate, 1.160042 ms, 0/3, attached=False, detach age=N/A ms.

- hover pair 1 on: measured, immediate, 1.148542 ms, 1/3, attached=False, detach age=N/A ms.

- hover pair 1 on: measured, immediate, 1.354708 ms, 2/3, attached=False, detach age=N/A ms.

- hover pair 2 on: warmup, non-immediate, 1.001583 ms, 0/3, attached=False, detach age=N/A ms.

- hover pair 2 on: measured, immediate, 1.096417 ms, 2/3, attached=False, detach age=N/A ms.

- hover pair 2 on: measured, immediate, 1.587084 ms, 2/3, attached=False, detach age=N/A ms.

- hover pair 3 off: warmup, non-immediate, 1.078208 ms, 0/3, attached=False, detach age=N/A ms.

- hover pair 3 on: measured, immediate, 1.352792 ms, 1/3, attached=False, detach age=N/A ms.

- hover pair 3 on: measured, immediate, 1.306375 ms, 2/3, attached=False, detach age=N/A ms.

- hover pair 3 on: measured, immediate, 1.811125 ms, 1/3, attached=False, detach age=N/A ms.

- hover pair 3 on: measured, immediate, 1.411459 ms, 2/3, attached=False, detach age=N/A ms.

- hover pair 4 on: measured, immediate, 1.258500 ms, 2/3, attached=False, detach age=N/A ms.

- hover pair 4 off: warmup, non-immediate, 1.259916 ms, 0/3, attached=False, detach age=N/A ms.

- hover pair 5 on: warmup, non-immediate, 1.005834 ms, 0/3, attached=False, detach age=N/A ms.

- hover pair 5 on: measured, immediate, 1.568750 ms, 2/3, attached=False, detach age=N/A ms.

- hover pair 5 on: measured, immediate, 1.101500 ms, 2/3, attached=False, detach age=N/A ms.

- drag pair 1 off: warmup, non-immediate, 1.004875 ms, 0/3, attached=False, detach age=N/A ms.

- drag pair 1 on: warmup, non-immediate, 1.084875 ms, 0/3, attached=False, detach age=N/A ms.

- drag pair 1 on: measured, immediate, 1.201250 ms, 2/3, attached=False, detach age=N/A ms.

- drag pair 1 on: measured, immediate, 1.004875 ms, 2/3, attached=False, detach age=N/A ms.

- drag pair 1 on: measured, immediate, 1.051083 ms, 2/3, attached=False, detach age=N/A ms.

- drag pair 1 on: measured, immediate, 1.204417 ms, 2/3, attached=False, detach age=N/A ms.

- drag pair 1 on: measured, immediate, 1.058333 ms, 2/3, attached=False, detach age=N/A ms.

- drag pair 1 on: measured, immediate, 1.029959 ms, 2/3, attached=False, detach age=N/A ms.

- drag pair 1 on: measured, immediate, 1.660208 ms, 2/3, attached=False, detach age=N/A ms.

- drag pair 1 on: measured, immediate, 1.036625 ms, 1/3, attached=False, detach age=N/A ms.

- drag pair 2 on: measured, immediate, 2.308417 ms, 2/3, attached=False, detach age=N/A ms.

- drag pair 2 on: measured, immediate, 2.163000 ms, 2/3, attached=False, detach age=N/A ms.

- drag pair 2 on: measured, immediate, 1.317875 ms, 2/3, attached=False, detach age=N/A ms.

- drag pair 2 off: warmup, non-immediate, 1.094708 ms, 0/3, attached=False, detach age=N/A ms.

- drag pair 3 on: warmup, non-immediate, 1.060959 ms, 0/3, attached=False, detach age=N/A ms.

- drag pair 3 on: measured, immediate, 1.058000 ms, 1/3, attached=False, detach age=N/A ms.

- drag pair 3 on: measured, immediate, 1.360667 ms, 2/3, attached=False, detach age=N/A ms.

- drag pair 4 on: measured, immediate, 2.387292 ms, 2/3, attached=False, detach age=N/A ms.

- drag pair 4 on: measured, immediate, 1.162208 ms, 2/3, attached=False, detach age=N/A ms.

- drag pair 4 on: measured, immediate, 1.453125 ms, 2/3, attached=False, detach age=N/A ms.

- drag pair 4 on: measured, immediate, 1.059417 ms, 2/3, attached=False, detach age=N/A ms.

- drag pair 4 off: warmup, non-immediate, 1.092291 ms, 0/3, attached=False, detach age=N/A ms.

- drag pair 5 off: warmup, non-immediate, 1.041500 ms, 0/3, attached=False, detach age=N/A ms.

- drag pair 5 on: measured, immediate, 1.160709 ms, 1/3, attached=False, detach age=N/A ms.

- drag pair 5 on: measured, immediate, 1.034333 ms, 2/3, attached=False, detach age=N/A ms.

- drag pair 5 on: measured, immediate, 1.151458 ms, 2/3, attached=False, detach age=N/A ms.

- drag pair 5 on: measured, immediate, 1.801083 ms, 2/3, attached=False, detach age=N/A ms.

- acceptance pair 1 off: warmup, non-immediate, 1.079167 ms, 0/3, attached=False, detach age=N/A ms.

- acceptance pair 1 on: warmup, non-immediate, 3.675333 ms, 0/3, attached=False, detach age=N/A ms.

- acceptance pair 1 on: measured, immediate, 1.098208 ms, 1/3, attached=False, detach age=N/A ms.

- acceptance pair 2 on: warmup, non-immediate, 1.073417 ms, 0/3, attached=False, detach age=N/A ms.

- acceptance pair 3 off: warmup, non-immediate, 3.033959 ms, 0/3, attached=False, detach age=N/A ms.

- acceptance pair 3 on: warmup, non-immediate, 1.020291 ms, 0/3, attached=False, detach age=N/A ms.

- acceptance pair 3 on: measured, immediate, 1.234084 ms, 2/3, attached=False, detach age=N/A ms.

- acceptance pair 3 on: measured, immediate, 1.313833 ms, 2/3, attached=False, detach age=N/A ms.

- acceptance pair 4 on: warmup, non-immediate, 4.726917 ms, 0/3, attached=False, detach age=N/A ms.

- acceptance pair 4 off: warmup, non-immediate, 1.034542 ms, 0/3, attached=False, detach age=N/A ms.

- acceptance pair 5 off: warmup, non-immediate, 1.222041 ms, 0/3, attached=False, detach age=N/A ms.

- acceptance pair 5 on: warmup, non-immediate, 1.058125 ms, 0/3, attached=False, detach age=N/A ms.

- acceptance pair 5 on: measured, immediate, 2.876625 ms, 2/3, attached=False, detach age=N/A ms.

- acceptance pair 5 on: measured, immediate, 4.316833 ms, 2/3, attached=False, detach age=N/A ms.

- log pair 1 off: warmup, non-immediate, 1.040875 ms, 0/3, attached=False, detach age=N/A ms.

- log pair 2 on: warmup, non-immediate, 1.036500 ms, 0/3, attached=False, detach age=N/A ms.

- log pair 2 off: warmup, non-immediate, 1.090292 ms, 0/3, attached=False, detach age=N/A ms.

- log pair 3 on: warmup, non-immediate, 1.328916 ms, 0/3, attached=False, detach age=N/A ms.

- log pair 4 on: warmup, non-immediate, 1.090416 ms, 0/3, attached=False, detach age=N/A ms.

- log pair 4 on: measured, immediate, 1.711000 ms, 0/3, attached=False, detach age=N/A ms.

- log pair 4 off: warmup, non-immediate, 1.127958 ms, 0/3, attached=False, detach age=N/A ms.

- log pair 5 off: warmup, non-immediate, 1.025292 ms, 0/3, attached=False, detach age=N/A ms.

## Scene construction causes

Cause bits are accumulated before each CPU draw: input=1, binding=2, changed visible colour=4; zero means other. A draw can consume multiple causes, so cause totals overlap. Exact mask counts are retained to avoid attributing a mixed draw to only one cause.

| workload | arm | input | binding | colour | other | masks | CPU builds deferred before draw |
| --- | --- | --- | --- | --- | --- | --- | --- |
| typing-10 | off | 2756 | 2995 | 0 | 0 | 2:239, 3:2756 | 0 |
| typing-10 | on | 3000 | 2995 | 0 | 0 | 1:2787, 2:2782, 3:213 | 16585 |
| typing-30 | off | 7879 | 8995 | 0 | 0 | 2:1116, 3:7879 | 0 |
| typing-30 | on | 9000 | 8995 | 0 | 0 | 1:8227, 2:8222, 3:773 | 53016 |
| hover | off | 29206 | 0 | 0 | 0 | 1:29206 | 0 |
| hover | on | 28921 | 0 | 0 | 0 | 1:28921 | 94187 |
| drag | off | 29539 | 0 | 0 | 0 | 1:29539 | 0 |
| drag | on | 29502 | 0 | 0 | 0 | 1:29502 | 102281 |
| acceptance | off | 30442 | 0 | 0 | 0 | 1:30442 | 0 |
| acceptance | on | 27984 | 0 | 0 | 0 | 1:27984 | 87584 |
| log | off | 0 | 0 | 0 | 2878 | 0:2878 | 0 |
| log | on | 0 | 0 | 0 | 2878 | 0:2878 | 0 |

## G5-3d matched captures

Typing quantiles are medians of five capture quantiles; each other scene is one sanity capture. Maxima are maxima across available captures. Input delays use MTLDrawable.presentedTime. Each capture has its own measured background load below. No unavailable callback is counted as physical proof that pixels were absent.

| workload | arm | captures | input→display median / p95 / max ms | visible versions/s (lower bound) | unmatched / rendered | order violations | lost native edits | >2 ms measured acquisitions | CPU s/s | CPU ms/attempt | draw+present ms/attempt | CPU draw+present >8.3 ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| typing-10 | off | 5 | 49.341/62.356/68.325 | 9.099 | 5/2730 | 0 | 0 | 0 | 0.113 | 12.348 | 2.114 | 0 |
| typing-10 | on | 5 | 24.810/31.106/35.904 | 9.971 | 0/2999 | 0 | 0 | 1 | 0.112 | 11.257 | 2.616 | 0 |
| typing-30 | off | 5 | 45.982/50.302/68.674 | 26.044 | 5/7823 | 0 | 0 | 0 | 0.209 | 8.009 | 1.941 | 0 |
| typing-30 | on | 5 | 25.109/31.224/35.986 | 29.973 | 0/9001 | 0 | 0 | 0 | 0.231 | 7.697 | 1.804 | 0 |
| hover | on | 1 | 30.218/38.633/44.619 | 98.612 | 2/5922 | 0 | 0 | 0 | 0.297 | 3.012 | 1.856 | 1 |
| drag | on | 1 | 31.408/40.916/49.049 | 98.397 | 2/5909 | 0 | 0 | 2 | 0.195 | 1.978 | 0.548 | 0 |
| acceptance | on | 1 | 30.073/37.973/50.879 | 93.917 | 1/5640 | 0 | 0 | 0 | 0.395 | 4.199 | 3.604 | 449 |
| log | on | 1 | 24.685/29.008/32.636 | 9.567 | 1/576 | 0 | 0 | 0 | 0.061 | 6.374 | 3.081 | 1 |

## Paired CPU gate

Deciding metric: getrusage user+sys seconds divided by measured wall seconds. The five pairs alternate off→on, on→off, off→on, on→off, off→on. Negative is less CPU. GAIN: all five ≤−3%; REGRESSION: all five >+3%; NEUTRAL: all five within inclusive ±3%; otherwise INCONCLUSIVE. Incomplete sets have no verdict. CPU per attempt is diagnostic only for this ticket.

| workload | five total-process CPU/s paired changes % | median / min / max % | verdict | five CPU/attempt changes %, diagnostic |
| --- | --- | --- | --- | --- |
| typing-10 | -0.115, -10.684, 0.783, -3.544, 7.705 | -0.115/-10.684/7.705 | INCONCLUSIVE | -8.837, -20.000, -7.908, -12.209, -0.866 |
| typing-30 | 21.609, -7.997, 13.859, 3.612, 17.713 | 13.859/-7.997/21.609 | INCONCLUSIVE | 6.866, -21.119, 0.063, -10.846, 2.373 |
| hover | none | NOT MEASURED/NOT MEASURED/NOT MEASURED | SANITY ONLY — no P verdict |  |
| drag | none | NOT MEASURED/NOT MEASURED/NOT MEASURED | SANITY ONLY — no P verdict |  |
| acceptance | none | NOT MEASURED/NOT MEASURED/NOT MEASURED | SANITY ONLY — no P verdict |  |
| log | none | NOT MEASURED/NOT MEASURED/NOT MEASURED | SANITY ONLY — no P verdict |  |

## Frame and refresh diagnostics

Throughput is a conservative count of distinct presentedTime slots with provably changed version sets. Multiple callbacks with different versions at exactly the same display time are ambiguous, not an inferred ordering.

The same input version can legitimately acquire later colours. Scene generations distinguish those updates from presenting an already-submitted unchanged scene. The single-refresh rule is per renderer/surface, using the platform refresh interval; it does not claim to read WindowServer private queue state.

| workload | arm | CPU draws / GPU frames / inputs | GPU→display median ms | missed refreshes median / p95 | gaps median / p95 / max ms | same-scene submissions skipped | next-interval deferrals | equal-time ambiguous slots |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| typing-10 | off | 2995/2730/3000 | 38.310 | 2.000/2.000 | 108.333/124.999/133.333 | 0 | 0 | 0 |
| typing-10 | on | 2999/2999/3000 | 19.847 | 0.000/0.000 | 108.332/116.666/116.670 | 0 | 0 | 0 |
| typing-30 | off | 8995/7823/9000 | 38.328 | 2.000/2.000 | 41.666/41.669/66.666 | 0 | 0 | 2 |
| typing-30 | on | 9001/9001/9000 | 20.726 | 0.000/0.000 | 33.333/41.667/50.000 | 0 | 0 | 0 |
| hover | on | 5922/5922/7200 | 21.548 | 0.000/1.000 | 8.333/16.667/25.000 | 0 | 0 | 0 |
| drag | on | 5909/5909/7200 | 22.257 | 0.000/1.000 | 8.333/16.667/25.000 | 0 | 0 | 0 |
| acceptance | on | 5640/5640/7200 | 21.256 | 0.000/1.000 | 8.333/16.667/41.666 | 0 | 0 | 0 |
| log | on | 576/576/600 | 20.136 | 0.000/0.000 | 100.003/108.333/108.336 | 0 | 0 | 0 |

## Observed-refresh missed slots

These labels classify observable trace states, with the inherited two-refresh control floor. “WindowServer late” means GPU-ready and handed off but not observed in the eligible slot; it does not establish compositor causality. Observer gaps are disclosed rather than filled in.

| workload | arm | slots | no new input | frame not ready | no drawable free | presented but replaced | WindowServer late | observer gaps |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| hover | on | 7199 | 1 | 723 | 0 | 3 | 555 | 1 |
| drag | on | 7200 | 1 | 691 | 0 | 3 | 601 | 0 |

## Per-capture process CPU and load

Each loaded row means **measured on a loaded machine (X% background)**; X is the start→end background percentage shown. Corrected background excludes Codex, its agents/shells and the measured app. Raw top-five process lists remain with each local capture.

| workload | pair | arm | attempts | wall s | user / sys / total CPU s | CPU s/s | CPU ms/attempt | actual refresh Hz | background start→end % | label |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| typing-10 | 1 | off | 547 | 60.007 | 5.703/1.052/6.754 | 0.113 | 12.348 | 120.000 | 14.630→3.860 | measured on a loaded machine (14.630%→3.860% background) |
| typing-10 | 1 | on | 600 | 60.074 | 5.642/1.112/6.754 | 0.112 | 11.257 | 120.000 | 8.900→10.220 | measured on a loaded machine (8.900%→10.220% background) |
| typing-10 | 2 | on | 600 | 60.063 | 5.699/1.095/6.794 | 0.113 | 11.324 | 120.000 | 3.920→4.600 | quiet |
| typing-10 | 2 | off | 537 | 60.016 | 6.564/1.037/7.601 | 0.127 | 14.155 | 120.000 | 4.560→5.560 | quiet |
| typing-10 | 3 | off | 548 | 60.020 | 5.557/1.047/6.605 | 0.110 | 12.052 | 120.000 | 2.980→14.100 | measured on a loaded machine (2.980%→14.100% background) |
| typing-10 | 3 | on | 600 | 60.048 | 5.542/1.117/6.659 | 0.111 | 11.099 | 120.000 | 12.530→20.530 | measured on a loaded machine (12.530%→20.530% background) |
| typing-10 | 4 | on | 600 | 60.081 | 5.490/1.079/6.569 | 0.109 | 10.949 | 120.000 | 14.400→15.750 | measured on a loaded machine (14.400%→15.750% background) |
| typing-10 | 4 | off | 546 | 60.070 | 5.781/1.029/6.810 | 0.113 | 12.472 | 120.000 | 11.420→14.420 | measured on a loaded machine (11.420%→14.420% background) |
| typing-10 | 5 | off | 552 | 60.078 | 5.267/1.044/6.311 | 0.105 | 11.433 | 120.000 | 6.590→13.610 | measured on a loaded machine (6.590%→13.610% background) |
| typing-10 | 5 | on | 599 | 60.006 | 5.671/1.118/6.789 | 0.113 | 11.334 | 120.000 | 10.260→14.510 | measured on a loaded machine (10.260%→14.510% background) |
| typing-30 | 1 | off | 1582 | 60.023 | 9.620/1.996/11.616 | 0.194 | 7.343 | 120.000 | 8.830→14.620 | measured on a loaded machine (8.830%→14.620% background) |
| typing-30 | 1 | on | 1800 | 60.015 | 12.060/2.064/14.124 | 0.235 | 7.847 | 120.000 | 13.200→14.380 | measured on a loaded machine (13.200%→14.380% background) |
| typing-30 | 2 | on | 1800 | 60.011 | 10.422/1.823/12.245 | 0.204 | 6.803 | 120.000 | 10.490→16.390 | measured on a loaded machine (10.490%→16.390% background) |
| typing-30 | 2 | off | 1544 | 60.039 | 11.229/2.086/13.316 | 0.222 | 8.624 | 120.000 | 14.280→19.830 | measured on a loaded machine (14.280%→19.830% background) |
| typing-30 | 3 | off | 1582 | 60.024 | 10.079/2.090/12.169 | 0.203 | 7.692 | 120.000 | 11.360→15.820 | measured on a loaded machine (11.360%→15.820% background) |
| typing-30 | 3 | on | 1800 | 60.020 | 11.793/2.061/13.854 | 0.231 | 7.697 | 120.000 | 13.220→14.430 | measured on a loaded machine (13.220%→14.430% background) |
| typing-30 | 4 | on | 1801 | 60.021 | 11.308/1.970/13.278 | 0.221 | 7.373 | 120.000 | 10.580→17.640 | measured on a loaded machine (10.580%→17.640% background) |
| typing-30 | 4 | off | 1550 | 60.033 | 10.796/2.022/12.818 | 0.214 | 8.269 | 120.000 | 24.740→20.480 | measured on a loaded machine (24.740%→20.480% background) |
| typing-30 | 5 | off | 1565 | 60.013 | 10.443/2.091/12.534 | 0.209 | 8.009 | 120.000 | 21.310→15.140 | measured on a loaded machine (21.310%→15.140% background) |
| typing-30 | 5 | on | 1800 | 60.030 | 12.608/2.151/14.759 | 0.246 | 8.199 | 120.000 | 20.530→15.400 | measured on a loaded machine (20.530%→15.400% background) |
| hover | 1 | on | 5922 | 60.003 | 12.982/4.854/17.836 | 0.297 | 3.012 | 120.000 | 11.060→14.440 | measured on a loaded machine (11.060%→14.440% background) |
| drag | 1 | on | 5909 | 60.002 | 5.648/6.042/11.690 | 0.195 | 1.978 | 120.000 | 6.930→15.020 | measured on a loaded machine (6.930%→15.020% background) |
| acceptance | 1 | on | 5640 | 60.010 | 9.477/14.206/23.683 | 0.395 | 4.199 | 120.000 | 7.610→14.750 | measured on a loaded machine (7.610%→14.750% background) |
| log | 1 | on | 576 | 60.104 | 2.734/0.937/3.671 | 0.061 | 6.374 | 120.000 | 8.520→15.270 | measured on a loaded machine (8.520%→15.270% background) |

## Acquisitions over 1 ms (all phases)

All calls over 1 ms in all phases are listed; the measured-window >2 ms count remains in the summary table. No acquisition workaround is adopted from these numbers. Each entry gives phase, path, duration, pre-call live/pool, attached state and milliseconds since display-link detach.

- typing-10 pair 1 off: warmup, non-immediate, 4.180375 ms, 0/3, attached=False, detach age=N/A ms.

- typing-10 pair 2 on: warmup, non-immediate, 1.047500 ms, 0/3, attached=False, detach age=N/A ms.

- typing-10 pair 2 on: measured, immediate, 2.160458 ms, 1/3, attached=False, detach age=N/A ms.

- typing-10 pair 2 off: warmup, non-immediate, 1.203708 ms, 0/3, attached=False, detach age=N/A ms.

- typing-10 pair 4 on: warmup, non-immediate, 2.910042 ms, 0/3, attached=False, detach age=N/A ms.

- typing-10 pair 4 on: measured, immediate, 1.042834 ms, 1/3, attached=False, detach age=N/A ms.

- typing-10 pair 4 off: warmup, non-immediate, 1.177125 ms, 0/3, attached=False, detach age=N/A ms.

- typing-10 pair 5 on: warmup, non-immediate, 1.068042 ms, 0/3, attached=False, detach age=N/A ms.

- typing-10 pair 5 on: measured, immediate, 1.340917 ms, 1/3, attached=False, detach age=N/A ms.

- typing-30 pair 1 on: warmup, non-immediate, 1.025417 ms, 0/3, attached=False, detach age=N/A ms.

- typing-30 pair 2 on: warmup, non-immediate, 1.102542 ms, 0/3, attached=False, detach age=N/A ms.

- typing-30 pair 2 on: measured, immediate, 1.195583 ms, 1/3, attached=False, detach age=N/A ms.

- typing-30 pair 2 off: warmup, non-immediate, 1.245000 ms, 0/3, attached=False, detach age=N/A ms.

- typing-30 pair 3 off: warmup, non-immediate, 1.007792 ms, 0/3, attached=False, detach age=N/A ms.

- typing-30 pair 3 on: warmup, non-immediate, 1.561209 ms, 0/3, attached=False, detach age=N/A ms.

- typing-30 pair 3 on: measured, immediate, 1.931000 ms, 1/3, attached=False, detach age=N/A ms.

- typing-30 pair 4 on: warmup, non-immediate, 1.131083 ms, 0/3, attached=False, detach age=N/A ms.

- typing-30 pair 4 on: measured, immediate, 1.274875 ms, 1/3, attached=False, detach age=N/A ms.

- typing-30 pair 4 off: warmup, non-immediate, 1.006625 ms, 0/3, attached=False, detach age=N/A ms.

- typing-30 pair 5 off: warmup, non-immediate, 1.231333 ms, 0/3, attached=False, detach age=N/A ms.

- hover pair 1 on: warmup, non-immediate, 1.271208 ms, 0/3, attached=False, detach age=N/A ms.

- hover pair 1 on: measured, immediate, 1.159583 ms, 1/3, attached=False, detach age=N/A ms.

- hover pair 1 on: measured, immediate, 1.278292 ms, 2/3, attached=False, detach age=N/A ms.

- drag pair 1 on: warmup, non-immediate, 1.267250 ms, 0/3, attached=False, detach age=N/A ms.

- drag pair 1 on: measured, immediate, 2.322083 ms, 1/3, attached=False, detach age=N/A ms.

- drag pair 1 on: measured, immediate, 6.508042 ms, 2/3, attached=False, detach age=N/A ms.

- drag pair 1 on: measured, immediate, 1.496416 ms, 2/3, attached=False, detach age=N/A ms.

- drag pair 1 on: measured, immediate, 1.376584 ms, 2/3, attached=False, detach age=N/A ms.

- drag pair 1 on: measured, immediate, 1.071208 ms, 2/3, attached=False, detach age=N/A ms.

- acceptance pair 1 on: warmup, non-immediate, 1.267917 ms, 0/3, attached=False, detach age=N/A ms.

- acceptance pair 1 on: measured, immediate, 1.340250 ms, 2/3, attached=False, detach age=N/A ms.

- log pair 1 on: warmup, non-immediate, 1.110667 ms, 0/3, attached=False, detach age=N/A ms.

## Scene construction causes

Cause bits are accumulated before each CPU draw: input=1, binding=2, changed visible colour=4; zero means other. A draw can consume multiple causes, so cause totals overlap. Exact mask counts are retained to avoid attributing a mixed draw to only one cause.

| workload | arm | input | binding | colour | other | masks | CPU builds deferred before draw |
| --- | --- | --- | --- | --- | --- | --- | --- |
| typing-10 | off | 2732 | 2995 | 0 | 0 | 2:263, 3:2732 | 0 |
| typing-10 | on | 2999 | 224 | 0 | 0 | 1:2775, 3:224 | 15202 |
| typing-30 | off | 7823 | 8995 | 0 | 0 | 2:1172, 3:7823 | 0 |
| typing-30 | on | 9000 | 1070 | 0 | 1 | 0:1, 1:7930, 3:1070 | 46256 |
| hover | on | 5922 | 0 | 0 | 0 | 1:5922 | 19987 |
| drag | on | 5909 | 0 | 0 | 0 | 1:5909 | 20878 |
| acceptance | on | 5640 | 0 | 0 | 0 | 1:5640 | 17500 |
| log | on | 0 | 0 | 0 | 576 | 0:576 | 0 |

## G5-3d post-guard retest — complete

This is the completed post-guard frozen-build set, superseding the partial table from November 23. The missing pairs and all four new sanity scenes finished November 24. Exact per-capture dates are retained in the driver logs; raw logs are not committed. Observed clock endpoints are unchanged. No incomplete set is assigned a verdict and no INCONCLUSIVE result is called a gain.

### Completed capture details

Typing quantiles are medians of five capture quantiles; each other scene is one sanity capture. Maxima are maxima across available captures. Input delays use MTLDrawable.presentedTime. Each capture has its own measured background load below. No unavailable callback is counted as physical proof that pixels were absent.

| workload | arm | captures | input→display median / p95 / max ms | visible versions/s (lower bound) | unmatched / rendered | order violations | lost native edits | >2 ms measured acquisitions | CPU s/s | CPU ms/attempt | draw+present ms/attempt | CPU draw+present >8.3 ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| typing-10 | off | 5 | 49.342/62.042/66.519 | 9.193 | 5/2770 | 0 | 0 | 0 | 0.101 | 11.020 | 2.086 | 0 |
| typing-10 | on | 5 | 24.795/30.706/35.901 | 9.975 | 0/3000 | 0 | 0 | 0 | 0.106 | 10.633 | 2.513 | 0 |
| typing-30 | off | 5 | 46.044/49.860/68.787 | 26.107 | 7/7870 | 0 | 0 | 0 | 0.206 | 7.941 | 1.999 | 0 |
| typing-30 | on | 5 | 24.881/31.411/39.357 | 29.970 | 0/9000 | 0 | 0 | 1 | 0.226 | 7.548 | 1.716 | 0 |
| hover | on | 1 | 29.999/38.971/45.605 | 95.546 | 2/5737 | 0 | 0 | 1 | 0.295 | 3.083 | 1.962 | 0 |
| drag | on | 1 | 31.339/40.370/50.164 | 97.474 | 2/5854 | 0 | 0 | 0 | 0.189 | 1.940 | 0.548 | 0 |
| acceptance | on | 1 | 31.496/37.778/54.172 | 92.291 | 1/5541 | 0 | 0 | 1 | 0.390 | 4.226 | 3.635 | 472 |
| log | on | 1 | 24.681/28.993/29.062 | 9.565 | 1/575 | 0 | 0 | 0 | 0.063 | 6.621 | 3.105 | 1 |

## Paired CPU gate

Deciding metric: getrusage user+sys seconds divided by measured wall seconds. The five pairs alternate off→on, on→off, off→on, on→off, off→on. Negative is less CPU. GAIN: all five ≤−3%; REGRESSION: all five >+3%; NEUTRAL: all five within inclusive ±3%; otherwise INCONCLUSIVE. Incomplete sets have no verdict. CPU per attempt is diagnostic only for this ticket.

| workload | five total-process CPU/s paired changes % | median / min / max % | verdict | five CPU/attempt changes %, diagnostic |
| --- | --- | --- | --- | --- |
| typing-10 | 2.304, 1.198, 10.170, 5.422, -3.263 | 2.304/-3.263/10.170 | INCONCLUSIVE | -5.649, -6.809, 2.043, -2.875, -10.635 |
| typing-30 | 9.609, -1.174, 13.286, 12.032, 5.990 | 9.609/-1.174/13.286 | INCONCLUSIVE | -4.955, -14.082, 0.017, -1.304, -7.526 |
| hover | none | NOT MEASURED/NOT MEASURED/NOT MEASURED | SANITY ONLY — no P verdict |  |
| drag | none | NOT MEASURED/NOT MEASURED/NOT MEASURED | SANITY ONLY — no P verdict |  |
| acceptance | none | NOT MEASURED/NOT MEASURED/NOT MEASURED | SANITY ONLY — no P verdict |  |
| log | none | NOT MEASURED/NOT MEASURED/NOT MEASURED | SANITY ONLY — no P verdict |  |

## Frame and refresh diagnostics

Throughput is a conservative count of distinct presentedTime slots with provably changed version sets. Multiple callbacks with different versions at exactly the same display time are ambiguous, not an inferred ordering.

The same input version can legitimately acquire later colours. Scene generations distinguish those updates from presenting an already-submitted unchanged scene. The single-refresh rule is per renderer/surface, using the platform refresh interval; it does not claim to read WindowServer private queue state.

| workload | arm | CPU draws / GPU frames / inputs | GPU→display median ms | missed refreshes median / p95 | gaps median / p95 / max ms | same-scene submissions skipped | next-interval deferrals | equal-time ambiguous slots |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| typing-10 | off | 2995/2770/3000 | 38.302 | 2.000/2.000 | 108.333/124.998/133.332 | 0 | 0 | 0 |
| typing-10 | on | 3000/3000/3000 | 19.641 | 0.000/0.000 | 108.332/116.666/116.670 | 0 | 0 | 0 |
| typing-30 | off | 8995/7870/9000 | 38.242 | 2.000/2.000 | 41.666/41.667/66.666 | 0 | 0 | 0 |
| typing-30 | on | 9000/9000/9000 | 20.777 | 0.000/0.000 | 33.333/41.667/50.000 | 0 | 0 | 0 |
| hover | on | 5737/5737/7200 | 22.211 | 0.000/1.000 | 8.333/16.667/25.004 | 0 | 0 | 0 |
| drag | on | 5854/5854/7200 | 22.496 | 0.000/1.000 | 8.333/16.667/25.001 | 0 | 0 | 0 |
| acceptance | on | 5541/5541/7200 | 22.182 | 0.000/1.000 | 8.333/16.667/33.333 | 0 | 0 | 0 |
| log | on | 575/575/600 | 20.210 | 0.000/0.000 | 104.166/108.333/112.500 | 0 | 0 | 0 |

## Observed-refresh missed slots

These labels classify observable trace states, with the inherited two-refresh control floor. “WindowServer late” means GPU-ready and handed off but not observed in the eligible slot; it does not establish compositor causality. Observer gaps are disclosed rather than filled in.

| workload | arm | slots | no new input | frame not ready | no drawable free | presented but replaced | WindowServer late | observer gaps |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| hover | on | 7197 | 1 | 796 | 0 | 3 | 665 | 3 |
| drag | on | 7201 | 1 | 722 | 0 | 3 | 626 | 0 |

## Per-capture process CPU and load

Each loaded row means **measured on a loaded machine (X% background)**; X is the start→end background percentage shown. Corrected background excludes Codex, its agents/shells and the measured app. Raw top-five process lists remain with each local capture.

| workload | pair | arm | attempts | wall s | user / sys / total CPU s | CPU s/s | CPU ms/attempt | actual refresh Hz | background start→end % | label |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| typing-10 | 1 | off | 553 | 60.074 | 5.166/1.066/6.232 | 0.104 | 11.269 | 120.000 | 5.910→14.780 | measured on a loaded machine (5.910%→14.780% background) |
| typing-10 | 1 | on | 600 | 60.113 | 5.333/1.047/6.380 | 0.106 | 10.633 | 120.000 | 6.360→5.490 | quiet |
| typing-10 | 2 | on | 600 | 60.049 | 5.290/1.051/6.341 | 0.106 | 10.568 | 120.000 | 4.280→6.380 | quiet |
| typing-10 | 2 | off | 553 | 60.100 | 5.203/1.068/6.271 | 0.104 | 11.340 | 120.000 | 3.920→7.330 | quiet |
| typing-10 | 3 | off | 556 | 60.076 | 4.832/1.031/5.863 | 0.098 | 10.546 | 120.000 | 6.010→21.490 | measured on a loaded machine (6.010%→21.490% background) |
| typing-10 | 3 | on | 600 | 60.048 | 5.387/1.069/6.457 | 0.108 | 10.761 | 120.000 | 13.900→7.320 | measured on a loaded machine (13.900%→7.320% background) |
| typing-10 | 4 | on | 600 | 60.021 | 5.378/1.044/6.422 | 0.107 | 10.704 | 120.000 | 18.640→7.650 | measured on a loaded machine (18.640%→7.650% background) |
| typing-10 | 4 | off | 553 | 60.045 | 5.068/1.026/6.094 | 0.101 | 11.020 | 120.000 | 12.710→8.300 | measured on a loaded machine (12.710%→8.300% background) |
| typing-10 | 5 | off | 555 | 60.084 | 5.025/1.063/6.088 | 0.101 | 10.969 | 120.000 | 8.150→9.040 | quiet |
| typing-10 | 5 | on | 600 | 60.005 | 4.907/0.974/5.882 | 0.098 | 9.803 | 120.000 | 6.620→9.810 | quiet |
| typing-30 | 1 | off | 1560 | 60.007 | 10.357/2.032/12.388 | 0.206 | 7.941 | 120.000 | 6.690→8.600 | quiet |
| typing-30 | 1 | on | 1800 | 60.039 | 11.637/1.949/13.586 | 0.226 | 7.548 | 120.000 | 7.660→10.060 | measured on a loaded machine (7.660%→10.060% background) |
| typing-30 | 2 | on | 1800 | 60.014 | 10.984/1.860/12.844 | 0.214 | 7.135 | 120.000 | 8.030→5.810 | quiet |
| typing-30 | 2 | off | 1565 | 60.018 | 10.770/2.227/12.997 | 0.217 | 8.305 | 120.000 | 3.980→17.750 | measured on a loaded machine (3.980%→17.750% background) |
| typing-30 | 3 | off | 1589 | 60.012 | 9.585/2.041/11.626 | 0.194 | 7.317 | 120.000 | 4.070→22.550 | measured on a loaded machine (4.070%→22.550% background) |
| typing-30 | 3 | on | 1800 | 60.018 | 11.254/1.919/13.172 | 0.219 | 7.318 | 120.000 | 12.670→15.290 | measured on a loaded machine (12.670%→15.290% background) |
| typing-30 | 4 | on | 1800 | 60.026 | 11.724/1.879/13.603 | 0.227 | 7.557 | 120.000 | 22.770→11.910 | measured on a loaded machine (22.770%→11.910% background) |
| typing-30 | 4 | off | 1586 | 60.036 | 10.052/2.092/12.144 | 0.202 | 7.657 | 120.000 | 18.220→14.940 | measured on a loaded machine (18.220%→14.940% background) |
| typing-30 | 5 | off | 1570 | 60.023 | 10.877/2.246/13.124 | 0.219 | 8.359 | 120.000 | 17.090→8.360 | measured on a loaded machine (17.090%→8.360% background) |
| typing-30 | 5 | on | 1800 | 60.040 | 11.962/1.952/13.914 | 0.232 | 7.730 | 120.000 | 14.950→17.210 | measured on a loaded machine (14.950%→17.210% background) |
| hover | 1 | on | 5737 | 60.002 | 13.071/4.619/17.689 | 0.295 | 3.083 | 120.000 | 13.020→17.670 | measured on a loaded machine (13.020%→17.670% background) |
| drag | 1 | on | 5854 | 60.006 | 5.677/5.679/11.356 | 0.189 | 1.940 | 120.000 | 9.360→12.430 | measured on a loaded machine (9.360%→12.430% background) |
| acceptance | 1 | on | 5541 | 60.006 | 9.423/13.995/23.418 | 0.390 | 4.226 | 120.000 | 17.870→12.490 | measured on a loaded machine (17.870%→12.490% background) |
| log | 1 | on | 575 | 60.008 | 2.822/0.985/3.807 | 0.063 | 6.621 | 120.000 | 15.180→11.200 | measured on a loaded machine (15.180%→11.200% background) |

## Acquisitions over 1 ms (all phases)

All calls over 1 ms in all phases are listed; the measured-window >2 ms count remains in the summary table. No acquisition workaround is adopted from these numbers. Each entry gives phase, path, duration, pre-call live/pool, attached state and milliseconds since display-link detach.

- typing-10 pair 1 off: warmup, non-immediate, 1.065500 ms, 0/3, attached=False, detach age=N/A ms.

- typing-10 pair 1 on: warmup, non-immediate, 1.069250 ms, 0/3, attached=False, detach age=N/A ms.

- typing-10 pair 2 on: warmup, non-immediate, 1.014000 ms, 0/3, attached=False, detach age=N/A ms.

- typing-10 pair 2 on: measured, immediate, 1.086750 ms, 1/3, attached=False, detach age=N/A ms.

- typing-10 pair 2 off: warmup, non-immediate, 1.148292 ms, 0/3, attached=False, detach age=N/A ms.

- typing-10 pair 3 off: warmup, non-immediate, 1.078166 ms, 0/3, attached=False, detach age=N/A ms.

- typing-10 pair 3 on: warmup, non-immediate, 1.202583 ms, 0/3, attached=False, detach age=N/A ms.

- typing-10 pair 3 on: measured, immediate, 1.222458 ms, 1/3, attached=False, detach age=N/A ms.

- typing-10 pair 4 on: warmup, non-immediate, 1.061208 ms, 0/3, attached=False, detach age=N/A ms.

- typing-10 pair 4 on: measured, immediate, 1.253166 ms, 1/3, attached=False, detach age=N/A ms.

- typing-10 pair 4 off: warmup, non-immediate, 3.070542 ms, 0/3, attached=False, detach age=N/A ms.

- typing-10 pair 5 off: warmup, non-immediate, 1.062708 ms, 0/3, attached=False, detach age=N/A ms.

- typing-10 pair 5 on: warmup, non-immediate, 1.072958 ms, 0/3, attached=False, detach age=N/A ms.

- typing-10 pair 5 on: measured, immediate, 1.244083 ms, 1/3, attached=False, detach age=N/A ms.

- typing-30 pair 1 off: warmup, non-immediate, 1.286583 ms, 0/3, attached=False, detach age=N/A ms.

- typing-30 pair 1 on: warmup, non-immediate, 1.054625 ms, 0/3, attached=False, detach age=N/A ms.

- typing-30 pair 2 on: warmup, non-immediate, 1.049458 ms, 0/3, attached=False, detach age=N/A ms.

- typing-30 pair 2 off: warmup, non-immediate, 1.042083 ms, 0/3, attached=False, detach age=N/A ms.

- typing-30 pair 3 off: warmup, non-immediate, 1.044959 ms, 0/3, attached=False, detach age=N/A ms.

- typing-30 pair 3 on: measured, immediate, 2.048000 ms, 1/3, attached=False, detach age=N/A ms.

- typing-30 pair 4 on: warmup, non-immediate, 1.106084 ms, 0/3, attached=False, detach age=N/A ms.

- typing-30 pair 5 off: warmup, non-immediate, 1.128208 ms, 0/3, attached=False, detach age=N/A ms.

- typing-30 pair 5 on: warmup, non-immediate, 1.189125 ms, 0/3, attached=False, detach age=N/A ms.

- typing-30 pair 5 on: measured, immediate, 1.451000 ms, 1/3, attached=False, detach age=N/A ms.

- hover pair 1 on: measured, immediate, 2.760541 ms, 1/3, attached=False, detach age=N/A ms.

- drag pair 1 on: measured, immediate, 1.454292 ms, 2/3, attached=False, detach age=N/A ms.

- acceptance pair 1 on: warmup, non-immediate, 1.137500 ms, 0/3, attached=False, detach age=N/A ms.

- acceptance pair 1 on: measured, immediate, 1.216708 ms, 1/3, attached=False, detach age=N/A ms.

- acceptance pair 1 on: measured, immediate, 2.652958 ms, 2/3, attached=False, detach age=N/A ms.

## Scene construction causes

Cause bits are accumulated before each CPU draw: input=1, binding=2, changed visible colour=4; zero means other. A draw can consume multiple causes, so cause totals overlap. Exact mask counts are retained to avoid attributing a mixed draw to only one cause.

| workload | arm | input | binding | colour | other | masks | CPU builds deferred before draw |
| --- | --- | --- | --- | --- | --- | --- | --- |
| typing-10 | off | 2770 | 2995 | 0 | 0 | 2:225, 3:2770 | 0 |
| typing-10 | on | 3000 | 217 | 0 | 0 | 1:2783, 3:217 | 15197 |
| typing-30 | off | 7872 | 8995 | 0 | 0 | 2:1123, 3:7872 | 0 |
| typing-30 | on | 9000 | 1034 | 0 | 0 | 1:7966, 3:1034 | 46134 |
| hover | on | 5737 | 0 | 0 | 0 | 1:5737 | 18720 |
| drag | on | 5854 | 0 | 0 | 0 | 1:5854 | 20053 |
| acceptance | on | 5541 | 0 | 0 | 0 | 1:5541 | 16788 |
| log | on | 0 | 0 | 0 | 575 | 0:575 | 0 |
