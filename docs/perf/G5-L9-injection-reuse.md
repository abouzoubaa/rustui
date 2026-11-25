# G5-L9 draw-first injection reuse

## Method

Five matched 60-second release captures per arm after warmup in the
RustUI Lab CodeEditor, using the 10,000-line L8 document at 10 posted
X/Backspace keys per second through Cocoa. Direct launch, display sync
off, passive display link off, damage off, prepaint reuse on and
buffered frame logging. Both arms used `RUSTUI_HL_FASTPATH=1` and
`RUSTUI_G5_L9_LOG=1`. Only `RUSTUI_G5_L9_SCOPED=0/1` varied. Pair order
was off/on, on/off, off/on, on/off, off/on. The release measurement
build included the local L2 sync-off and L4 frame timing probes.
The base was the G5-L8 merge into `g5-integration`.

Process CPU is measured-window user plus system time from
`getrusage(RUSAGE_SELF)` divided by native keys. The Stage P verdict
uses the five paired changes. Main and worker CPU use matched Mach
thread snapshots. Worker IDs come from the injection records, so the
worker column includes all CPU on identified parsing threads and
excludes other background threads. The injection stage is elapsed
time per key, including discovery, changed-range mapping and any
injected-layer parsing. Samply measures on-CPU time separately.

Handler latency ends at ScreenCaptureKit `displayTime` for the first
decoded version. The miss rate uses the L4 proxy, `SCK − GPU end ≥
8.333 ms`, among matched versions with non-negative margins. The
quiet-machine check used 5% corrected background CPU before each pair.
Capture labels use corrected background CPU at both boundaries.

## Candidate

The original PX-3 switch and the pinned Rust grammar, literal query
and registry-generation checks still apply. The scoped path also
requires `RUSTUI_G5_L9_SCOPED=1`. It maps cached matches through the
pending edits, combines syntax changes with text-only changes, and
runs the two pinned patterns around affected macro roots. An already
proven empty result still uses PX-3’s original single-edit, error-free,
structurally unchanged path. The query index retains all matches so a
later match can enter the parser’s
512-layer limit when an earlier match disappears. Unaffected layers
keep their parsed trees.

The first snapshot runs the full query. A new language initializes
its own state. Missing tree/index/edit mappings, or incomplete query
iteration, are explicit full-query fallback reasons. Grammars and
queries outside PX-3’s pin use the existing highlighter. The buffered
diagnostic records every failed legacy predicate for each applied
snapshot, even if an earlier predicate already failed. More than one
predicate can fail on the same snapshot.

The oracles compare ranges, injected syntax trees and styles with a
fresh full recomputation. They cover insertion, deletion, paste,
undo, edits inside and outside injections, creation and removal of a
macro, temporary syntax errors, accumulated pending edits and an
edit that opens a slot at the layer cap. The same-tree empty-result
test also verifies that a subsequent macro creation takes the scoped
query route.

## Paired results

The first scoped-query implementation reached 7.166 ms/key. The
final build also retained the empty-result shortcut and shared the
changed-range calculation between the diagnostic and scoped query.
All five pairs were repeated on the final build.

CPU and injection columns are medians across captures. Latency uses the
median of the five capture medians and the median of their p95s. Miss
rate pools joined keys across captures.

| Arm | Process CPU ms/key | Main ms/key | Workers ms/key | Injection stage ms/key | Handler → SCK median / p95 (ms) | Miss rate |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| PX-3 | 11.150 | 4.035 | 6.545 | 4.482 | 8.800 / 18.850 | 33.92% |
| Scoped | 6.871 | 3.953 | 2.537 | 0.259 | 8.719 / 18.748 | 32.62% |

**GAIN** on process CPU per key. Paired changes were -37.45%, -41.64%, -36.91%, -37.46%, -39.89% (median -37.46%).

The candidate median met 7 ms/key. Captures 3 and 4 individually exceeded it.
Every candidate capture kept the entire injection stage below 0.3 ms/key.
Aggregate ScreenCaptureKit median and p95 stayed within 0.11 ms of the control.
All ten captures exceeded 5% at one or both background-load boundaries.

| Pair | Process ms/key off → on | Main ms/key off → on | Workers ms/key off → on | Injection ms/key off → on | Handler → SCK median/p95 off → on (ms) | Miss rate off → on | Background off | Background on |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- | --- |
| 1 | 10.986 → 6.871 | 3.991 → 3.953 | 6.125 → 2.536 | 4.381 → 0.259 | 8.667 / 18.601 → 8.756 / 19.396 | 32.82% → 31.09% | loaded, 3.68 / 24.79% | loaded, 20.31 / 17.05% |
| 2 | 11.080 → 6.467 | 4.036 → 3.701 | 6.411 → 2.351 | 4.451 → 0.254 | 8.800 / 19.047 → 9.101 / 19.271 | 35.10% → 37.61% | loaded, 5.77 / 16.72% | loaded, 4.18 / 16.69% |
| 3 | 11.150 → 7.035 | 4.012 → 4.051 | 6.766 → 2.544 | 4.536 → 0.259 | 9.009 / 18.494 → 8.542 / 18.748 | 33.96% → 32.14% | loaded, 4.95 / 16.86% | loaded, 4.32 / 17.10% |
| 4 | 11.277 → 7.052 | 4.035 → 4.086 | 6.545 → 2.562 | 4.482 → 0.273 | 9.000 / 18.850 → 8.719 / 18.465 | 35.48% → 31.52% | loaded, 4.52 / 17.10% | loaded, 5.33 / 17.04% |
| 5 | 11.269 → 6.774 | 4.067 → 3.864 | 6.657 → 2.537 | 4.556 → 0.269 | 8.491 / 20.599 → 8.520 / 18.595 | 32.26% → 30.80% | loaded, 5.50 / 17.15% | loaded, 3.32 / 17.74% |

- PX-3 handled 3,000 native inputs. The trace recorded 2,899 applied results and 2,930 non-negative SCK/GPU margins.
- The scoped arm handled 3,000 native inputs. The trace recorded 2,927 applied results and 2,946 non-negative SCK/GPU margins.
- 24 worker/capture IDs were absent at the final Mach snapshot. Their CPU remains in the process total, so the worker column is a lower bound.

## Reuse predicates and full-query fallbacks

The first full PX-3 capture contained 600 native inputs and 578 applied injection records.

| Failed legacy predicate | Count |
| --- | ---: |
| One mapped edit required | 21 |
| PX-3 switch disabled | 0 |
| Language/query outside pinned eligibility | 0 |
| Previous result nonempty or unknown | 0 |
| Grammar differs from pinned Rust | 0 |
| Language registry generation changed | 0 |
| Query identity changed | 0 |
| New syntax tree has an error | 0 |
| No mapped old syntax tree | 0 |
| Old syntax tree has an error | 0 |
| Nonempty structural changed ranges | 290 |

| Injection route | First 60 s PX-3 | Five PX-3 captures | Five scoped captures |
| --- | ---: | ---: | ---: |
| Legacy empty reuse | 278 | 1399 | 0 |
| Legacy full query | 300 | 1500 | 0 |
| No mapped tree | 0 | 0 | 0 |
| Missing index or edit map | 0 | 0 | 0 |
| Scoped query | 0 | 0 | 1500 |
| Incomplete scoped query | 0 | 0 | 0 |
| Mapped syntax tree lost | 0 | 0 | 0 |
| Scoped empty reuse | 0 | 0 | 1427 |

The control took the full query on 300 records. Nonempty structural
ranges accounted for 290 records, and the single-edit requirement failed
on 21 records. Eleven records failed both checks.
The previous-empty, pinned grammar and query predicates passed throughout
this capture. The candidate handled both structural edits and empty
proof reuse without full queries in the measured window.

Fallback counts cover measured-window results. Startup and language-load
work before the measurement window is omitted.

## Remaining worker CPU

The separate 60-second candidate Samply run sampled all threads on-CPU
at 2,000 Hz and recorded 600 native keys. Corrected background CPU
was 3.85% at the start and 13.51% at the end. Stage buckets are
exclusive and use sampled thread CPU deltas.

Across the identified workers, the profile attributed 0.906 ms/key to
the draw-first highlight cache, 0.144 to fold updates and 0.003 to
stacks naming the scoped injection code. Highlight-cache samples
include Tree-sitter query traversal and changed-range iteration.
Another 1.120 ms/key fell in other stacks, dominated by samples ending
at the Grand Central Dispatch worker return `__workq_kernreturn`.
Samply associates a thread CPU delta with a sampled stack, so that
weight does not isolate time spent inside the syscall. Host parsing
did not resolve as a separate bucket in this recording. The per-edit
injection timer covers the entire stage, including work that the
sampler could not attribute to an injection symbol.

| Worker ID | Mach CPU ms/key | Sampled ms/key | Parse | Folds | Draw-first cache | Injection | Other |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 4246242 | 0.4956 | 0.4956 | 0.0000 | 0.0255 | 0.2183 | 0.0027 | 0.2490 |
| 4246246 | 0.4372 | 0.4372 | 0.0000 | 0.0345 | 0.1938 | 0.0000 | 0.2090 |
| 4246200 | 0.4053 | 0.4053 | 0.0000 | 0.0279 | 0.1566 | 0.0000 | 0.2208 |
| 4246201 | 0.3976 | 0.3976 | 0.0000 | 0.0236 | 0.1569 | 0.0000 | 0.2171 |
| 4246247 | — | 0.2921 | 0.0000 | 0.0214 | 0.1251 | 0.0003 | 0.1453 |
| 4246245 | 0.0984 | 0.0984 | 0.0000 | 0.0061 | 0.0375 | 0.0000 | 0.0547 |
| 4246204 | — | 0.0242 | 0.0000 | 0.0036 | 0.0085 | 0.0000 | 0.0120 |
| 4246248 | — | 0.0231 | 0.0000 | 0.0011 | 0.0097 | 0.0000 | 0.0123 |

<details><summary>Worker 4246242 top functions</summary>

| Rank | Self function and file | Self ms/key | Inclusive function and file | Inclusive ms/key |
| --- | --- | ---: | --- | ---: |
| 1 | `__workq_kernreturn` (libsystem_kernel.dylib) | 0.2398 | `_pthread_wqthread` (libsystem_pthread.dylib) | 0.4923 |
| 2 | `ts_subtree_get_changed_ranges` (vendor/tree-sitter-0.27.1/src/get_changed_ranges.c) | 0.0272 | `_dispatch_client_callout` (libdispatch.dylib) | 0.2484 |
| 3 | `ts_query_cursor__advance` (vendor/tree-sitter-0.27.1/src/query.c:4033) | 0.0247 | `_dispatch_root_queue_drain` (libdispatch.dylib) | 0.2475 |
| 4 | `<ropey::rope::Rope>::_line_to_byte_idx` (vendor/ropey-2.0.0-beta.1/src/shared_impl.rs:947) | 0.0246 | `_dispatch_worker_thread2` (libdispatch.dylib) | 0.2475 |
| 5 | `iterator_descend` (vendor/tree-sitter-0.27.1/src/get_changed_ranges.c:246) | 0.0206 | `gpui_apple::dispatcher::trampoline` (rustui-lab) | 0.2466 |

</details>

<details><summary>Worker 4246246 top functions</summary>

| Rank | Self function and file | Self ms/key | Inclusive function and file | Inclusive ms/key |
| --- | --- | ---: | --- | ---: |
| 1 | `__workq_kernreturn` (libsystem_kernel.dylib) | 0.1973 | `_pthread_wqthread` (libsystem_pthread.dylib) | 0.4370 |
| 2 | `<ropey::rope::Rope>::_line_to_byte_idx` (vendor/ropey-2.0.0-beta.1/src/shared_impl.rs:947) | 0.0287 | `_dispatch_client_callout` (libdispatch.dylib) | 0.2306 |
| 3 | `ts_subtree_get_changed_ranges` (vendor/tree-sitter-0.27.1/src/get_changed_ranges.c) | 0.0267 | `_dispatch_root_queue_drain` (libdispatch.dylib) | 0.2304 |
| 4 | `iterator_advance` (rustui-lab) | 0.0174 | `_dispatch_worker_thread2` (libdispatch.dylib) | 0.2304 |
| 5 | `iterator_descend` (vendor/tree-sitter-0.27.1/src/get_changed_ranges.c:246) | 0.0171 | `_dispatch_queue_override_invoke` (libdispatch.dylib) | 0.2304 |

</details>

<details><summary>Worker 4246200 top functions</summary>

| Rank | Self function and file | Self ms/key | Inclusive function and file | Inclusive ms/key |
| --- | --- | ---: | --- | ---: |
| 1 | `__workq_kernreturn` (libsystem_kernel.dylib) | 0.2086 | `_pthread_wqthread` (libsystem_pthread.dylib) | 0.4039 |
| 2 | `ts_subtree_get_changed_ranges` (vendor/tree-sitter-0.27.1/src/get_changed_ranges.c) | 0.0231 | `__workq_kernreturn` (libsystem_kernel.dylib) | 0.2086 |
| 3 | `<ropey::rope::Rope>::_line_to_byte_idx` (vendor/ropey-2.0.0-beta.1/src/shared_impl.rs:947) | 0.0193 | `_dispatch_client_callout` (libdispatch.dylib) | 0.1874 |
| 4 | `ts_query_cursor__advance` (vendor/tree-sitter-0.27.1/src/query.c:4033) | 0.0182 | `_dispatch_root_queue_drain` (libdispatch.dylib) | 0.1846 |
| 5 | `iterator_advance` (rustui-lab) | 0.0136 | `_dispatch_worker_thread2` (libdispatch.dylib) | 0.1846 |

</details>

<details><summary>Worker 4246201 top functions</summary>

| Rank | Self function and file | Self ms/key | Inclusive function and file | Inclusive ms/key |
| --- | --- | ---: | --- | ---: |
| 1 | `__workq_kernreturn` (libsystem_kernel.dylib) | 0.2079 | `_pthread_wqthread` (libsystem_pthread.dylib) | 0.3971 |
| 2 | `<ropey::rope::Rope>::_line_to_byte_idx` (vendor/ropey-2.0.0-beta.1/src/shared_impl.rs:947) | 0.0205 | `__workq_kernreturn` (libsystem_kernel.dylib) | 0.2079 |
| 3 | `ts_subtree_get_changed_ranges` (vendor/tree-sitter-0.27.1/src/get_changed_ranges.c) | 0.0168 | `_dispatch_client_callout` (libdispatch.dylib) | 0.1822 |
| 4 | `iterator_descend` (vendor/tree-sitter-0.27.1/src/get_changed_ranges.c:246) | 0.0160 | `_dispatch_root_queue_drain` (libdispatch.dylib) | 0.1806 |
| 5 | `ts_tree_cursor_child_iterator_next` (vendor/tree-sitter-0.27.1/src/tree_cursor.c:65) | 0.0154 | `_dispatch_worker_thread2` (libdispatch.dylib) | 0.1806 |

</details>

<details><summary>Worker 4246247 top functions</summary>

| Rank | Self function and file | Self ms/key | Inclusive function and file | Inclusive ms/key |
| --- | --- | ---: | --- | ---: |
| 1 | `__workq_kernreturn` (libsystem_kernel.dylib) | 0.1397 | `_pthread_wqthread` (libsystem_pthread.dylib) | 0.2919 |
| 2 | `<ropey::rope::Rope>::_line_to_byte_idx` (vendor/ropey-2.0.0-beta.1/src/shared_impl.rs:947) | 0.0208 | `_dispatch_client_callout` (libdispatch.dylib) | 0.1479 |
| 3 | `ts_query_cursor__advance` (vendor/tree-sitter-0.27.1/src/query.c:4033) | 0.0181 | `_dispatch_root_queue_drain` (libdispatch.dylib) | 0.1470 |
| 4 | `iterator_descend` (vendor/tree-sitter-0.27.1/src/get_changed_ranges.c:246) | 0.0167 | `_dispatch_worker_thread2` (libdispatch.dylib) | 0.1470 |
| 5 | `ts_subtree_get_changed_ranges` (vendor/tree-sitter-0.27.1/src/get_changed_ranges.c) | 0.0165 | `_dispatch_queue_override_invoke.cold.3` (libdispatch.dylib) | 0.1469 |

</details>

<details><summary>Worker 4246245 top functions</summary>

| Rank | Self function and file | Self ms/key | Inclusive function and file | Inclusive ms/key |
| --- | --- | ---: | --- | ---: |
| 1 | `__workq_kernreturn` (libsystem_kernel.dylib) | 0.0519 | `_pthread_wqthread` (libsystem_pthread.dylib) | 0.0983 |
| 2 | `<ropey::rope::Rope>::_byte_to_line_idx` (vendor/ropey-2.0.0-beta.1/src/shared_impl.rs:926) | 0.0062 | `__workq_kernreturn` (libsystem_kernel.dylib) | 0.0519 |
| 3 | `ts_subtree_get_changed_ranges` (vendor/tree-sitter-0.27.1/src/get_changed_ranges.c) | 0.0043 | `_dispatch_client_callout` (libdispatch.dylib) | 0.0439 |
| 4 | `ts_query_cursor__advance` (vendor/tree-sitter-0.27.1/src/query.c:4033) | 0.0034 | `_dispatch_root_queue_drain` (libdispatch.dylib) | 0.0437 |
| 5 | `<ropey::rope::Rope>::_line_to_byte_idx` (vendor/ropey-2.0.0-beta.1/src/shared_impl.rs:947) | 0.0030 | `_dispatch_worker_thread2` (libdispatch.dylib) | 0.0437 |

</details>

<details><summary>Worker 4246204 top functions</summary>

| Rank | Self function and file | Self ms/key | Inclusive function and file | Inclusive ms/key |
| --- | --- | ---: | --- | ---: |
| 1 | `__workq_kernreturn` (libsystem_kernel.dylib) | 0.0115 | `_pthread_wqthread` (libsystem_pthread.dylib) | 0.0242 |
| 2 | `iterator_descend` (vendor/tree-sitter-0.27.1/src/get_changed_ranges.c:246) | 0.0034 | `_dispatch_client_callout` (libdispatch.dylib) | 0.0122 |
| 3 | `ts_tree_cursor_child_iterator_next` (vendor/tree-sitter-0.27.1/src/tree_cursor.c:65) | 0.0017 | `<gpui_component::highlighter::input_adapter::TreeSitterInputHighlighter>::finish_update::{closure#0}::{closure#0}::{closure#0}` (crates/rustui-gpui-component/src/highlighter/input_adapter.rs:110) | 0.0121 |
| 4 | `<ropey::rope::Rope>::_byte_to_line_idx` (vendor/ropey-2.0.0-beta.1/src/shared_impl.rs:926) | 0.0014 | `_dispatch_root_queue_drain` (libdispatch.dylib) | 0.0121 |
| 5 | `<ropey::rope::Rope>::_line_to_byte_idx` (vendor/ropey-2.0.0-beta.1/src/shared_impl.rs:947) | 0.0012 | `gpui_component::highlighter::input_adapter::draw_first_background::parse_scoped` (crates/rustui-gpui-component/src/highlighter/draw_first_background.rs:54) | 0.0121 |

</details>

<details><summary>Worker 4246248 top functions</summary>

| Rank | Self function and file | Self ms/key | Inclusive function and file | Inclusive ms/key |
| --- | --- | ---: | --- | ---: |
| 1 | `__workq_kernreturn` (libsystem_kernel.dylib) | 0.0116 | `_pthread_wqthread` (libsystem_pthread.dylib) | 0.0231 |
| 2 | `<ropey::rope::Rope>::_line_to_byte_idx` (vendor/ropey-2.0.0-beta.1/src/shared_impl.rs:947) | 0.0025 | `__workq_kernreturn` (libsystem_kernel.dylib) | 0.0116 |
| 3 | `ts_subtree_get_changed_ranges` (vendor/tree-sitter-0.27.1/src/get_changed_ranges.c) | 0.0020 | `_dispatch_client_callout` (libdispatch.dylib) | 0.0109 |
| 4 | `ts_tree_cursor_current_status` (rustui-lab) | 0.0012 | `_dispatch_root_queue_drain` (libdispatch.dylib) | 0.0109 |
| 5 | `ts_query_cursor__add_state` (rustui-lab) | 0.0011 | `_dispatch_worker_thread2` (libdispatch.dylib) | 0.0109 |

</details>

## Verification scope

All three focused Kit oracles ran in an isolated fixture. The fixture
excluded an existing fold test whose `parser(&"rust".into())` call fails
to compile for a `&str` parameter. The candidate did not modify that
test.
