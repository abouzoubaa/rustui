# G5-L8 CPU per keystroke

## Method

One 60-second native Cocoa capture per app after warmup, with 10 posted
X/Backspace keys/s. The Lab and AppKit editors loaded the same 10,000
lines and used Menlo 13 pt. The single-line controls began empty.
The apps launched directly. GPUI ran with display sync and damage off,
visible-line reuse on, and buffered logging. For the two editors, an
additional capture ran under Samply with all threads sampled on-CPU at
2,000 Hz. The thread enumeration and process snapshot came from that
same profiled run. The separate plain captures show the accounting
without sampling overhead.

CPU is user plus system time from `getrusage(RUSAGE_SELF)` for the app
process. External observer and WindowServer CPU are outside that count.
Thread rows
come from the start/end `task_threads` and `thread_info` snapshots.
A thread first observed at the end contributes its lifetime CPU.
Threads that disappeared and snapshot skew remain in the process
residual. Thread labels use Mach names and the dominant sampled stack.
A pool worker that handles multiple types of work stays in its dominant
group. The function tables use each sampler’s CPU weights. The Mach CPU
counter is shown separately for each thread.

The frame-window estimate uses the first matching GPUI version from
native handler entry through Metal GPU end. The AppKit window ends at
NSTextView draw return. Thread sample-CPU overlap supplies the fraction
of each measured thread’s CPU assigned to the windows. This is a time
partition. Background work can overlap a frame without contributing to
it, and unjoined keys have no frame window. Outside-frame CPU is the
process total less the estimate. The AppKit and GPUI frame endpoint
differences limit direct comparison of those two columns.

| Arm | CPU ms/key total | Main | Background | Frame-window CPU | Outside-frame CPU | Idle CPU s/s focused / unfocused |
| --- | --- | --- | --- | --- | --- | --- |
| RustUI CodeEditor, 10,000 lines | 9.706 | 3.473 | 6.233 | 3.949 | 5.757 | 0.00797 / 0.00051 |
| AppKit NSTextView, 10,000 lines | 5.538 | 4.243 | 1.295 | 2.255 | 3.283 | 0.01070 / 0.00054 |
| RustUI TextField | 4.300 | 2.991 | 1.310 | — | — | 0.00596 / 0.00049 |
| Bare GPUI Fast | 3.405 | 2.125 | 1.280 | — | — | 0.00065 / 0.00059 |

The CodeEditor spent **5.955 ms/key on five workers** whose sampled
stacks were dominated by highlighting and parsing. This explains most of
its higher process CPU. In the profiled runs, AppKit used more main-thread
CPU and much less worker CPU. The plain runs retained the same total CPU
ordering. Background load differed between the captures, which prevents
a sampler-overhead estimate from these runs.

No dedicated Metal/driver thread was identified in the measured CPU
deltas. Driver work on the main thread stays in the main-thread column.

The estimated **5.757 ms/key outside CodeEditor frame windows** includes
Tree-sitter injection-query traversal. The closest sampled framework
function was `SyntaxHighlighter::compute_injection_layers_observed`
at 3.152 ms/key outside the windows. It runs the query over the syntax
tree from `draw_first_background::parse_scoped` on background workers.
The PX-3 flag was on. The profile establishes that the full-query fallback
ran. Which reuse predicate failed per edit remains unmeasured.
The AppKit outside-window samples include its event loop, Core Animation
transaction flushing and NSTextView invalidation.

## Process and thread accounting

| Arm | Capture | Keys | Wall seconds | CPU ms/key | Main ms/key | Highlighter / parse threads | Metal / driver threads | Other threads | Process minus thread sum | Frame windows joined |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| RustUI CodeEditor, 10,000 lines | Samply | 600 | 60.03 | 9.706 | 3.473 | 5.955 | 0.000 | 0.277 | <0.001 | 590 |
| RustUI CodeEditor, 10,000 lines | Plain | 600 | 60.04 | 10.944 | 3.977 | — | — | — | +0.052 | — |
| AppKit NSTextView, 10,000 lines | Samply | 599 | 60.00 | 5.538 | 4.243 | 0.000 | 0.000 | 1.263 | +0.032 | 599 |
| AppKit NSTextView, 10,000 lines | Plain | 599 | 60.00 | 7.074 | 5.331 | — | — | — | +0.389 | — |

<details><summary>RustUI CodeEditor, 10,000 lines per-thread counter deltas</summary>

| Thread ID | Name | User ms/key | System ms/key | Total ms/key |
| --- | --- | --- | --- | --- |
| 4097183 | Main | 3.1667 | 0.3063 | 3.4730 |
| 4097267 | unnamed | 1.3668 | 0.1406 | 1.5073 |
| 4097214 | unnamed | 1.1817 | 0.1311 | 1.3128 |
| 4097264 | unnamed | 1.0897 | 0.1413 | 1.2310 |
| 4097250 | unnamed | 1.0998 | 0.1247 | 1.2245 |
| 4097265 | unnamed | 0.5992 | 0.0806 | 0.6798 |
| 4097216 | com.apple.NSEventThread | 0.1361 | 0.1412 | 0.2773 |

</details>

<details><summary>AppKit NSTextView, 10,000 lines per-thread counter deltas</summary>

| Thread ID | Name | User ms/key | System ms/key | Total ms/key |
| --- | --- | --- | --- | --- |
| 4098391 | Main | 3.7981 | 0.4448 | 4.2429 |
| 4098415 | com.apple.NSEventThread | 0.2002 | 0.2312 | 0.4314 |
| 4098413 | unnamed | 0.1814 | 0.1135 | 0.2948 |
| 4098411 | unnamed | 0.1774 | 0.1045 | 0.2819 |
| 4098433 | unnamed | 0.1592 | 0.0959 | 0.2552 |

</details>

The sampled and plain typing runs had loaded-machine snapshots.
Corrected background CPU at the run boundaries appears below.

| Run | Start | End |
| --- | ---: | ---: |
| profile-editor | 14.79% | 12.94% |
| profile-appkit | 12.51% | 9.57% |
| plain-editor | 3.24% | 18.73% |
| plain-appkit | 3.58% | 16.84% |
| plain-single | 7.79% | 17.19% |
| plain-bare | 4.15% | 27.64% |
| wait-v2-editor | 4.37% | 16.77% |
| wait-v2-single | 17.70% | 16.32% |
| wait-v2-bare | 4.21% | 16.67% |

## Handler CPU and waiting

Handler wall and thread CPU clocks bracket the measured Cocoa `sendEvent:` call. Waiting is wall minus main-thread CPU, so it includes both blocked time and descheduling. The blocked-stack observer ran separately. It sampled the main thread only when the kernel reported a waiting state and then verified the key tag still matched. The observer can miss short waits. Each instrumented key is compared with
its own thread CPU and Cocoa wall interval. Table counts for the plain
and observer arms come from separate captures.

| Arm | Keys | Wall median / p95 (ms) | On-CPU median / p95 (ms) | Waiting median / p95 (ms) | Plain waits >0.2 ms | Observer waits with a stack |
| --- | --- | --- | --- | --- | --- | --- |
| RustUI CodeEditor, 10,000 lines | 600 | 1.317 / 2.386 | 1.235 / 1.844 | 0.033 / 0.760 | 99 | 32 / 75 |
| RustUI TextField | 600 | 1.046 / 1.936 | 0.993 / 1.530 | 0.030 / 0.393 | 62 | 28 / 65 |
| Bare GPUI Fast | 600 | 0.747 / 1.085 | 0.706 / 1.007 | 0.020 / 0.200 | 31 | 10 / 20 |

Each blocked-stack cell shows distinct keys / samples. One key can appear in more than one row.

| Blocking stack | CodeEditor | TextField | Bare GPUI |
| --- | --- | --- | --- |
| CoreFoundation preferences / cfprefsd reply | 1 / 4 | 1 / 7 | 1 / 6 |
| WindowServer cursor hiding | 0 / 0 | 1 / 18 | 1 / 7 |
| WindowServer key-focus lookup | 31 / 1311 | 27 / 1126 | 9 / 421 |

Observer keys above 0.2 ms without a stack:

- RustUI CodeEditor, 10,000 lines: 35 without a poll during the handler, 8 only runnable observed.

- RustUI TextField: 7 only runnable observed, 30 without a poll during the handler.

- Bare GPUI Fast: 10 without a poll during the handler.

Representative blocking stacks:

- CoreFoundation preferences / cfprefsd reply. `mach_msg2_trap` (libsystem_kernel.dylib) → `mach_msg2_internal` (libsystem_kernel.dylib) → `mach_msg_overwrite` (libsystem_kernel.dylib) → `mach_msg` (libsystem_kernel.dylib) → `_dispatch_mach_send_and_wait_for_reply` (libdispatch.dylib) → `dispatch_mach_send_with_result_and_wait_for_reply` (libdispatch.dylib) → `xpc_connection_send_message_with_reply_sync` (libxpc.dylib) → `__103-[CFPrefsSearchListSource synchronouslySendSystemMessage:andUserMessage:andDirectMessage:replyHandler:]_block_invoke_2` (CoreFoundation) → `-[_CFXPreferences withConnectionForRole:andUserIdentifier:performBlock:]` (CoreFoundation) → `__103-[CFPrefsSearchListSource synchronouslySendSystemMessage:andUserMessage:andDirectMessage:replyHandler:]_block_invoke` (CoreFoundation)

- WindowServer cursor hiding. `mach_msg2_trap` (libsystem_kernel.dylib) → `mach_msg2_internal` (libsystem_kernel.dylib) → `mach_msg_overwrite` (libsystem_kernel.dylib) → `mach_msg` (libsystem_kernel.dylib) → `SLSObscureCursor` (SkyLight) → `MacPlatform::hide_cursor_until_mouse_moves` (`crates/rustui-gpui-fast-macos/src/platform.rs`) → `Window::dispatch_event` (`crates/rustui-gpui-fast/src/window.rs`).

- WindowServer key-focus lookup. `mach_msg2_trap` (libsystem_kernel.dylib) → `mach_msg2_internal` (libsystem_kernel.dylib) → `mach_msg_overwrite` (libsystem_kernel.dylib) → `mach_msg` (libsystem_kernel.dylib) → `SLPSGetKeyFocusProcess` (SkyLight) → `CurrentProcessHasKeyFocus` (HIToolbox) → `ShowSourceIndicator` (HIToolbox) → `TSMProcessRawKeyEventWithOptionsAndCompletionHandler` (HIToolbox) → `__84-[NSTextInputContext _handleEvent:options:allowingSyntheticEvent:completionHandler:]_block_invoke_2.801` (AppKit) → `__204-[NSTextInputContext tryTSMProcessRawKeyEvent_orSubstitution:dispatchCondition:setupForDispatch:furtherCondition:doubleSpaceSubstitutionCondition:doubleSpaceSubstitutionWork:dispatchTSMWork:continuation:]_block_invoke.781` (AppKit)

The stacks expose WindowServer focus and cursor requests and preference
lookups. Exact Mach destination ports were unavailable.
`CLOCK_THREAD_CPUTIME_ID` establishes the waiting residual, and runnable
polls alone leave its cause unknown.

## Idle with no input

The no-input windows ran without ScreenCaptureKit or a stack sampler. Mach interrupt and platform-idle wakeup rates are reported separately. Timer-bin counts and logged drawing sources can overlap those counters.

| Arm | Focus | Key / active at both boundaries | CPU s/s | Interrupt wakes/s | Platform-idle wakes/s | Timer-bin 1 / 2 wakes/s | Wall window (s) | Caret events | GPUI draws / callbacks |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| RustUI CodeEditor, 10,000 lines | focused | True/True → True/True | 0.00797 | 2.74 | 0.05 | 0.00 / 0.00 | 60.12 | 114 | 114 / 0 |
| RustUI CodeEditor, 10,000 lines | unfocused | False/True → False/True | 0.00051 | 0.45 | 0.00 | 0.00 / 0.00 | 61.87 | 0 | 0 / 0 |
| AppKit NSTextView, 10,000 lines | focused | True/True → True/True | 0.01070 | 5.70 | 0.16 | 0.36 / 0.00 | 60.83 | 54 | — / — |
| AppKit NSTextView, 10,000 lines | unfocused | False/True → False/True | 0.00054 | 0.50 | 0.02 | 0.00 / 0.00 | 62.22 | 0 | — / — |
| RustUI TextField | focused | True/True → True/True | 0.00596 | 2.69 | 0.03 | 0.00 / 0.00 | 60.26 | 114 | 114 / 0 |
| RustUI TextField | unfocused | False/True → False/True | 0.00049 | 0.40 | 0.00 | 0.00 / 0.00 | 61.99 | 0 | 0 / 0 |
| Bare GPUI Fast | focused | True/True → True/True | 0.00065 | 0.50 | 0.00 | 0.00 / 0.00 | 62.32 | 0 | 0 / 0 |
| Bare GPUI Fast | unfocused | False/True → False/True | 0.00059 | 0.48 | 0.02 | 0.00 / 0.00 | 62.32 | 0 | 0 / 0 |

The focused CodeEditor and TextField each recorded 114 accepted
`BlinkCursor::schedule` timer ticks and 114 CPU draws. The GPUI frame
callback recorder counted zero display-link-driven callbacks. NSTextView
recorded 54 insertion-point draws in its focused window. All caret events
stopped when the benchmark window lost key status. The passive display
link was off. No typing or highlight work was deliberately scheduled in
these idle windows. Residual wakeups could not be assigned individually.

## Sampled functions by thread

Function times below are Samply CPU ms per delivered key. Inclusive rows overlap within the call tree. A library name is shown when source lines are unavailable. Threads without a positive sample have no function ranking.

### RustUI CodeEditor, 10,000 lines

Leading sampled self-time functions outside the joined frame windows:

- `ts_query_cursor__advance` (vendor/tree-sitter-0.27.1/src/query.c:4033), 1.094 ms/key.

- `ts_tree_cursor_child_iterator_next` (vendor/tree-sitter-0.27.1/src/tree_cursor.c:65), 0.976 ms/key.

- `mach_msg2_trap` (libsystem_kernel.dylib), 0.873 ms/key.

- `ts_tree_cursor_current_status` (vendor/tree-sitter-0.27.1/src/tree_cursor.c:506), 0.699 ms/key.

- `__workq_kernreturn` (libsystem_kernel.dylib), 0.687 ms/key.

Nearest sampled framework frames outside those windows:

- `<gpui_component::highlighter::highlighter::SyntaxHighlighter>::compute_injection_layers_observed` (crates/rustui-gpui-component/src/highlighter/highlighter.rs:894), 3.152 ms/key.

- `_DPSBlockUntilNextEventMatchingListInMode` (AppKit), 0.703 ms/key.

- `_NSEventThread` (AppKit), 0.213 ms/key.

- `<ropey::rope::Rope as gpui_base::input::rope_ext::RopeExt>::offset_to_point` (crates/rustui-gpui-base/src/input/base/rope_ext.rs:313), 0.210 ms/key.

- `<gpui_component::highlighter::input_adapter::TreeSitterInputHighlighter>::finish_update::{closure#0}::{closure#0}::{closure#0}` (crates/rustui-gpui-component/src/highlighter/input_adapter.rs:110), 0.183 ms/key.

<details><summary>rustui-lab (4097183), 3.473 Mach CPU ms/key</summary>

| Rank | Self function | Self ms/key | Inclusive function | Inclusive ms/key |
| --- | --- | --- | --- | --- |
| 1 | `mach_msg2_trap` (libsystem_kernel.dylib) | 1.3228 | `start` (dyld) | 3.4730 |
| 2 | `_platform_memmove` (libsystem_platform.dylib) | 0.1591 | `main` (rustui-lab) | 3.4730 |
| 3 | `_xzm_xzone_malloc_tiny` (libsystem_malloc.dylib) | 0.0480 | `-[NSApplication run]` (AppKit) | 3.4730 |
| 4 | `gpui::text_system::line::paint_line` (crates/rustui-gpui-fast/src/text_system/line.rs:585) | 0.0467 | `rustui_lab::main` (apps/rustui-lab/src/main.rs:6) | 3.4730 |
| 5 | `objc_msgSend` (libobjcMsgSend1.dylib) | 0.0388 | `std::sys::backtrace::__rust_begin_short_backtrace::<fn(), ()>` (library/std/src/sys/backtrace.rs:166) | 3.4730 |
| 6 | `objc_msgSend` (libobjcMsgSend.dylib) | 0.0353 | `<gpui_macos::platform::MacPlatform as gpui::platform::Platform>::run` (crates/rustui-gpui-fast-macos/src/platform.rs:565) | 3.4730 |
| 7 | `_xzm_free_main` (libsystem_malloc.dylib) | 0.0255 | `std::rt::lang_start_internal` (library/std/src/rt.rs:171) | 3.4730 |
| 8 | `__CF_IS_OBJC` (CoreFoundation) | 0.0222 | `rustui_lab::g0::run_cli` (apps/rustui-lab/src/g0.rs:581) | 3.4730 |
| 9 | `__workq_kernreturn` (libsystem_kernel.dylib) | 0.0210 | `std::rt::lang_start::<()>::{closure#0}` (library/std/src/rt.rs:206) | 3.4730 |
| 10 | `core::slice::sort::unstable::quicksort::quicksort::<u128, <u128 as core::cmp::PartialOrd>::lt>` (library/core/src/slice/sort/unstable/quicksort.rs:31) | 0.0207 | `-[NSApplication(NSEventRouting) nextEventMatchingMask:untilDate:inMode:dequeue:]` (AppKit) | 2.1731 |

</details>

<details><summary>Thread <4097267> (4097267), 1.507 Mach CPU ms/key</summary>

| Rank | Self function | Self ms/key | Inclusive function | Inclusive ms/key |
| --- | --- | --- | --- | --- |
| 1 | `ts_query_cursor__advance` (vendor/tree-sitter-0.27.1/src/query.c:4033) | 0.3420 | `_pthread_wqthread` (libsystem_pthread.dylib) | 1.5065 |
| 2 | `ts_tree_cursor_child_iterator_next` (vendor/tree-sitter-0.27.1/src/tree_cursor.c:65) | 0.3130 | `_dispatch_client_callout` (libdispatch.dylib) | 1.2612 |
| 3 | `__workq_kernreturn` (libsystem_kernel.dylib) | 0.2339 | `_dispatch_worker_thread2` (libdispatch.dylib) | 1.2591 |
| 4 | `ts_tree_cursor_current_status` (vendor/tree-sitter-0.27.1/src/tree_cursor.c:506) | 0.2173 | `_dispatch_root_queue_drain` (libdispatch.dylib) | 1.2591 |
| 5 | `ts_tree_cursor_goto_sibling_internal` (vendor/tree-sitter-0.27.1/src/tree_cursor.c:316) | 0.0967 | `_dispatch_queue_override_invoke` (libdispatch.dylib) | 1.2590 |
| 6 | `ts_tree_cursor_goto_first_child_internal` (vendor/tree-sitter-0.27.1/src/tree_cursor.c:195) | 0.0575 | `gpui_apple::dispatcher::trampoline` (rustui-lab) | 1.2590 |
| 7 | `ts_subtree_get_changed_ranges` (vendor/tree-sitter-0.27.1/src/get_changed_ranges.c:413) | 0.0358 | `_dispatch_queue_override_invoke.cold.3` (libdispatch.dylib) | 1.2590 |
| 8 | `<ropey::rope::Rope>::_line_to_byte_idx` (vendor/ropey-2.0.0-beta.1/src/shared_impl.rs:947) | 0.0306 | `<async_task::raw::RawTask<core::pin::Pin<alloc::boxed::Box<dyn core::future::future::Future<Output = core::option::Option<(tree_sitter::Tree, alloc::vec::Vec<gpui_component::highlighter::highlighter::InjectionLayer>, alloc::vec::Vec<gpui_base::input::display_map::folding::FoldRange>, core::option::Option<alloc::sync::Arc<gpui_component::highlighter::highlighter::draw_first_cache::Cache>>)>> + core::marker::Send>>, core::option::Option<(tree_sitter::Tree, alloc::vec::Vec<gpui_component::highlighter::highlighter::InjectionLayer>, alloc::vec::Vec<gpui_base::input::display_map::folding::FoldRange>, core::option::Option<alloc::sync::Arc<gpui_component::highlighter::highlighter::draw_first_cache::Cache>>)>, <scheduler::executor::BackgroundExecutor>::schedule_with_priority::{closure#0}, scheduler::RunnableMeta>>::run` (vendor/async-task-4.7.1/src/raw.rs:557) | 1.2580 |
| 9 | `iterator_descend` (rustui-lab) | 0.0226 | `<gpui_component::highlighter::input_adapter::TreeSitterInputHighlighter>::finish_update::{closure#0}::{closure#0}::{closure#0}` (crates/rustui-gpui-component/src/highlighter/input_adapter.rs:110) | 1.2571 |
| 10 | `<ropey::rope::Rope>::_byte_to_line_idx` (vendor/ropey-2.0.0-beta.1/src/shared_impl.rs:926) | 0.0180 | `<tree_sitter::QueryMatches<gpui_component::highlighter::highlighter::TextProvider, &[u8]> as streaming_iterator::StreamingIterator>::advance` (vendor/tree-sitter-0.27.1/binding_rust/lib.rs:3615) | 1.0378 |

</details>

<details><summary>Thread <4097214> (4097214), 1.313 Mach CPU ms/key</summary>

| Rank | Self function | Self ms/key | Inclusive function | Inclusive ms/key |
| --- | --- | --- | --- | --- |
| 1 | `ts_tree_cursor_child_iterator_next` (vendor/tree-sitter-0.27.1/src/tree_cursor.c:65) | 0.2744 | `_pthread_wqthread` (libsystem_pthread.dylib) | 1.3120 |
| 2 | `ts_query_cursor__advance` (vendor/tree-sitter-0.27.1/src/query.c:4033) | 0.2717 | `_dispatch_client_callout` (libdispatch.dylib) | 1.0731 |
| 3 | `__workq_kernreturn` (libsystem_kernel.dylib) | 0.2345 | `_dispatch_worker_thread2` (libdispatch.dylib) | 1.0711 |
| 4 | `ts_tree_cursor_current_status` (vendor/tree-sitter-0.27.1/src/tree_cursor.c:506) | 0.1770 | `_dispatch_root_queue_drain` (libdispatch.dylib) | 1.0711 |
| 5 | `ts_tree_cursor_goto_sibling_internal` (vendor/tree-sitter-0.27.1/src/tree_cursor.c:316) | 0.0840 | `gpui_apple::dispatcher::trampoline` (rustui-lab) | 1.0711 |
| 6 | `ts_tree_cursor_goto_first_child_internal` (vendor/tree-sitter-0.27.1/src/tree_cursor.c:195) | 0.0494 | `_dispatch_queue_override_invoke.cold.3` (libdispatch.dylib) | 1.0710 |
| 7 | `ts_subtree_get_changed_ranges` (vendor/tree-sitter-0.27.1/src/get_changed_ranges.c:413) | 0.0303 | `_dispatch_queue_override_invoke` (libdispatch.dylib) | 1.0710 |
| 8 | `<ropey::rope::Rope>::_line_to_byte_idx` (vendor/ropey-2.0.0-beta.1/src/shared_impl.rs:947) | 0.0270 | `<async_task::raw::RawTask<core::pin::Pin<alloc::boxed::Box<dyn core::future::future::Future<Output = core::option::Option<(tree_sitter::Tree, alloc::vec::Vec<gpui_component::highlighter::highlighter::InjectionLayer>, alloc::vec::Vec<gpui_base::input::display_map::folding::FoldRange>, core::option::Option<alloc::sync::Arc<gpui_component::highlighter::highlighter::draw_first_cache::Cache>>)>> + core::marker::Send>>, core::option::Option<(tree_sitter::Tree, alloc::vec::Vec<gpui_component::highlighter::highlighter::InjectionLayer>, alloc::vec::Vec<gpui_base::input::display_map::folding::FoldRange>, core::option::Option<alloc::sync::Arc<gpui_component::highlighter::highlighter::draw_first_cache::Cache>>)>, <scheduler::executor::BackgroundExecutor>::schedule_with_priority::{closure#0}, scheduler::RunnableMeta>>::run` (vendor/async-task-4.7.1/src/raw.rs:557) | 1.0709 |
| 9 | `ts_query__pattern_map_search` (vendor/tree-sitter-0.27.1/src/query.c:1241) | 0.0220 | `<gpui_component::highlighter::input_adapter::TreeSitterInputHighlighter>::finish_update::{closure#0}::{closure#0}::{closure#0}` (crates/rustui-gpui-component/src/highlighter/input_adapter.rs:110) | 1.0701 |
| 10 | `iterator_descend` (rustui-lab) | 0.0216 | `<tree_sitter::QueryMatches<gpui_component::highlighter::highlighter::TextProvider, &[u8]> as streaming_iterator::StreamingIterator>::advance` (vendor/tree-sitter-0.27.1/binding_rust/lib.rs:3615) | 0.8878 |

</details>

<details><summary>Thread <4097264> (4097264), 1.231 Mach CPU ms/key</summary>

| Rank | Self function | Self ms/key | Inclusive function | Inclusive ms/key |
| --- | --- | --- | --- | --- |
| 1 | `ts_query_cursor__advance` (vendor/tree-sitter-0.27.1/src/query.c:4033) | 0.2740 | `_pthread_wqthread` (libsystem_pthread.dylib) | 1.2306 |
| 2 | `__workq_kernreturn` (libsystem_kernel.dylib) | 0.2353 | `_dispatch_client_callout` (libdispatch.dylib) | 0.9861 |
| 3 | `ts_tree_cursor_child_iterator_next` (vendor/tree-sitter-0.27.1/src/tree_cursor.c:65) | 0.2154 | `_dispatch_worker_thread2` (libdispatch.dylib) | 0.9821 |
| 4 | `ts_tree_cursor_current_status` (vendor/tree-sitter-0.27.1/src/tree_cursor.c:506) | 0.1681 | `_dispatch_root_queue_drain` (libdispatch.dylib) | 0.9821 |
| 5 | `ts_tree_cursor_goto_sibling_internal` (vendor/tree-sitter-0.27.1/src/tree_cursor.c:316) | 0.0732 | `gpui_apple::dispatcher::trampoline` (rustui-lab) | 0.9820 |
| 6 | `ts_tree_cursor_goto_first_child_internal` (vendor/tree-sitter-0.27.1/src/tree_cursor.c:195) | 0.0430 | `_dispatch_queue_override_invoke.cold.3` (libdispatch.dylib) | 0.9820 |
| 7 | `iterator_descend` (rustui-lab) | 0.0328 | `_dispatch_queue_override_invoke` (libdispatch.dylib) | 0.9820 |
| 8 | `ts_subtree_get_changed_ranges` (vendor/tree-sitter-0.27.1/src/get_changed_ranges.c:413) | 0.0267 | `<async_task::raw::RawTask<core::pin::Pin<alloc::boxed::Box<dyn core::future::future::Future<Output = core::option::Option<(tree_sitter::Tree, alloc::vec::Vec<gpui_component::highlighter::highlighter::InjectionLayer>, alloc::vec::Vec<gpui_base::input::display_map::folding::FoldRange>, core::option::Option<alloc::sync::Arc<gpui_component::highlighter::highlighter::draw_first_cache::Cache>>)>> + core::marker::Send>>, core::option::Option<(tree_sitter::Tree, alloc::vec::Vec<gpui_component::highlighter::highlighter::InjectionLayer>, alloc::vec::Vec<gpui_base::input::display_map::folding::FoldRange>, core::option::Option<alloc::sync::Arc<gpui_component::highlighter::highlighter::draw_first_cache::Cache>>)>, <scheduler::executor::BackgroundExecutor>::schedule_with_priority::{closure#0}, scheduler::RunnableMeta>>::run` (vendor/async-task-4.7.1/src/raw.rs:557) | 0.9819 |
| 9 | `<ropey::rope::Rope>::_line_to_byte_idx` (vendor/ropey-2.0.0-beta.1/src/shared_impl.rs:947) | 0.0218 | `<gpui_component::highlighter::input_adapter::TreeSitterInputHighlighter>::finish_update::{closure#0}::{closure#0}::{closure#0}` (crates/rustui-gpui-component/src/highlighter/input_adapter.rs:110) | 0.9811 |
| 10 | `iterator_advance` (rustui-lab) | 0.0197 | `<tree_sitter::QueryMatches<gpui_component::highlighter::highlighter::TextProvider, &[u8]> as streaming_iterator::StreamingIterator>::advance` (vendor/tree-sitter-0.27.1/binding_rust/lib.rs:3615) | 0.7901 |

</details>

<details><summary>Thread <4097250> (4097250), 1.224 Mach CPU ms/key</summary>

| Rank | Self function | Self ms/key | Inclusive function | Inclusive ms/key |
| --- | --- | --- | --- | --- |
| 1 | `ts_query_cursor__advance` (vendor/tree-sitter-0.27.1/src/query.c:4033) | 0.2797 | `_pthread_wqthread` (libsystem_pthread.dylib) | 1.2237 |
| 2 | `ts_tree_cursor_child_iterator_next` (vendor/tree-sitter-0.27.1/src/tree_cursor.c:65) | 0.2398 | `_dispatch_client_callout` (libdispatch.dylib) | 1.0009 |
| 3 | `__workq_kernreturn` (libsystem_kernel.dylib) | 0.2119 | `_dispatch_worker_thread2` (libdispatch.dylib) | 0.9993 |
| 4 | `ts_tree_cursor_current_status` (vendor/tree-sitter-0.27.1/src/tree_cursor.c:506) | 0.1740 | `_dispatch_root_queue_drain` (libdispatch.dylib) | 0.9993 |
| 5 | `ts_tree_cursor_goto_sibling_internal` (vendor/tree-sitter-0.27.1/src/tree_cursor.c:316) | 0.0761 | `gpui_apple::dispatcher::trampoline` (rustui-lab) | 0.9992 |
| 6 | `ts_tree_cursor_goto_first_child_internal` (vendor/tree-sitter-0.27.1/src/tree_cursor.c:195) | 0.0419 | `_dispatch_queue_override_invoke.cold.3` (libdispatch.dylib) | 0.9992 |
| 7 | `<ropey::rope::Rope>::_line_to_byte_idx` (vendor/ropey-2.0.0-beta.1/src/shared_impl.rs:947) | 0.0294 | `_dispatch_queue_override_invoke` (libdispatch.dylib) | 0.9992 |
| 8 | `ts_subtree_get_changed_ranges` (vendor/tree-sitter-0.27.1/src/get_changed_ranges.c:413) | 0.0242 | `<async_task::raw::RawTask<core::pin::Pin<alloc::boxed::Box<dyn core::future::future::Future<Output = core::option::Option<(tree_sitter::Tree, alloc::vec::Vec<gpui_component::highlighter::highlighter::InjectionLayer>, alloc::vec::Vec<gpui_base::input::display_map::folding::FoldRange>, core::option::Option<alloc::sync::Arc<gpui_component::highlighter::highlighter::draw_first_cache::Cache>>)>> + core::marker::Send>>, core::option::Option<(tree_sitter::Tree, alloc::vec::Vec<gpui_component::highlighter::highlighter::InjectionLayer>, alloc::vec::Vec<gpui_base::input::display_map::folding::FoldRange>, core::option::Option<alloc::sync::Arc<gpui_component::highlighter::highlighter::draw_first_cache::Cache>>)>, <scheduler::executor::BackgroundExecutor>::schedule_with_priority::{closure#0}, scheduler::RunnableMeta>>::run` (vendor/async-task-4.7.1/src/raw.rs:557) | 0.9991 |
| 9 | `iterator_descend` (rustui-lab) | 0.0184 | `<gpui_component::highlighter::input_adapter::TreeSitterInputHighlighter>::finish_update::{closure#0}::{closure#0}::{closure#0}` (crates/rustui-gpui-component/src/highlighter/input_adapter.rs:110) | 0.9982 |
| 10 | `ts_query__pattern_map_search` (vendor/tree-sitter-0.27.1/src/query.c:1241) | 0.0153 | `<tree_sitter::QueryMatches<gpui_component::highlighter::highlighter::TextProvider, &[u8]> as streaming_iterator::StreamingIterator>::advance` (vendor/tree-sitter-0.27.1/binding_rust/lib.rs:3615) | 0.8267 |

</details>

<details><summary>Thread <4097265> (4097265), 0.680 Mach CPU ms/key</summary>

| Rank | Self function | Self ms/key | Inclusive function | Inclusive ms/key |
| --- | --- | --- | --- | --- |
| 1 | `ts_query_cursor__advance` (vendor/tree-sitter-0.27.1/src/query.c:4033) | 0.1578 | `_pthread_wqthread` (libsystem_pthread.dylib) | 0.6797 |
| 2 | `__workq_kernreturn` (libsystem_kernel.dylib) | 0.1375 | `_dispatch_client_callout` (libdispatch.dylib) | 0.5385 |
| 3 | `ts_tree_cursor_child_iterator_next` (vendor/tree-sitter-0.27.1/src/tree_cursor.c:65) | 0.1201 | `_dispatch_worker_thread2` (libdispatch.dylib) | 0.5385 |
| 4 | `ts_tree_cursor_current_status` (vendor/tree-sitter-0.27.1/src/tree_cursor.c:506) | 0.0948 | `_dispatch_root_queue_drain` (libdispatch.dylib) | 0.5373 |
| 5 | `ts_tree_cursor_goto_sibling_internal` (vendor/tree-sitter-0.27.1/src/tree_cursor.c:316) | 0.0394 | `gpui_apple::dispatcher::trampoline` (rustui-lab) | 0.5372 |
| 6 | `ts_tree_cursor_goto_first_child_internal` (vendor/tree-sitter-0.27.1/src/tree_cursor.c:195) | 0.0208 | `_dispatch_queue_override_invoke.cold.3` (libdispatch.dylib) | 0.5371 |
| 7 | `iterator_descend` (rustui-lab) | 0.0141 | `_dispatch_queue_override_invoke` (libdispatch.dylib) | 0.5371 |
| 8 | `<ropey::rope::Rope>::_byte_to_line_idx` (vendor/ropey-2.0.0-beta.1/src/shared_impl.rs:926) | 0.0129 | `<gpui_component::highlighter::input_adapter::TreeSitterInputHighlighter>::finish_update::{closure#0}::{closure#0}::{closure#0}` (crates/rustui-gpui-component/src/highlighter/input_adapter.rs:110) | 0.5371 |
| 9 | `ts_subtree_get_changed_ranges` (vendor/tree-sitter-0.27.1/src/get_changed_ranges.c:413) | 0.0113 | `<async_task::raw::RawTask<core::pin::Pin<alloc::boxed::Box<dyn core::future::future::Future<Output = core::option::Option<(tree_sitter::Tree, alloc::vec::Vec<gpui_component::highlighter::highlighter::InjectionLayer>, alloc::vec::Vec<gpui_base::input::display_map::folding::FoldRange>, core::option::Option<alloc::sync::Arc<gpui_component::highlighter::highlighter::draw_first_cache::Cache>>)>> + core::marker::Send>>, core::option::Option<(tree_sitter::Tree, alloc::vec::Vec<gpui_component::highlighter::highlighter::InjectionLayer>, alloc::vec::Vec<gpui_base::input::display_map::folding::FoldRange>, core::option::Option<alloc::sync::Arc<gpui_component::highlighter::highlighter::draw_first_cache::Cache>>)>, <scheduler::executor::BackgroundExecutor>::schedule_with_priority::{closure#0}, scheduler::RunnableMeta>>::run` (vendor/async-task-4.7.1/src/raw.rs:557) | 0.5371 |
| 10 | `<ropey::rope::Rope>::_line_to_byte_idx` (vendor/ropey-2.0.0-beta.1/src/shared_impl.rs:947) | 0.0083 | `<tree_sitter::QueryMatches<gpui_component::highlighter::highlighter::TextProvider, &[u8]> as streaming_iterator::StreamingIterator>::advance` (vendor/tree-sitter-0.27.1/binding_rust/lib.rs:3615) | 0.4401 |

</details>

<details><summary>com.apple.NSEventThread (4097216), 0.277 Mach CPU ms/key</summary>

| Rank | Self function | Self ms/key | Inclusive function | Inclusive ms/key |
| --- | --- | --- | --- | --- |
| 1 | `mach_msg2_trap` (libsystem_kernel.dylib) | 0.2585 | `_pthread_start` (libsystem_pthread.dylib) | 0.2773 |
| 2 | `_kernelrpc_mach_vm_deallocate_trap` (libsystem_kernel.dylib) | 0.0015 | `_CFRunLoopRunSpecificWithOptions` (CoreFoundation) | 0.2773 |
| 3 | `SLS::TokenizedCoding::ReadDataProvider::with_slice_b(unsigned long, void (std::__1::shared_ptr<SLS::TokenizedCoding::ReadDataProvider>&) block_pointer)` (SkyLight) | 0.0010 | `_NSEventThread` (AppKit) | 0.2773 |
| 4 | `_xzm_xzone_malloc_tiny` (libsystem_malloc.dylib) | 0.0010 | `__CFRunLoopRun` (CoreFoundation) | 0.2767 |
| 5 | `_os_log_impl_flatten_and_send` (libsystem_trace.dylib) | 0.0009 | `mach_msg2_trap` (libsystem_kernel.dylib) | 0.2585 |
| 6 | `SLS::TokenizedCoding::Storage::valid() const` (SkyLight) | 0.0008 | `mach_msg2_internal` (libsystem_kernel.dylib) | 0.2585 |
| 7 | `SLEventGetIntegerValueField` (SkyLight) | 0.0007 | `mach_msg` (libsystem_kernel.dylib) | 0.2566 |
| 8 | `__kdebug_trace64` (libsystem_kernel.dylib) | 0.0007 | `mach_msg_overwrite` (libsystem_kernel.dylib) | 0.2566 |
| 9 | `-[NSObject init]` (libobjc.A.dylib) | 0.0007 | `__CFRunLoopServiceMachPort` (CoreFoundation) | 0.2252 |
| 10 | `_CFArrayReplaceValues` (CoreFoundation) | 0.0006 | `__CFRunLoopDoSource1` (CoreFoundation) | 0.0514 |

</details>

### AppKit NSTextView, 10,000 lines

Leading sampled self-time functions outside the joined frame windows:

- `mach_msg2_trap` (libsystem_kernel.dylib), 1.112 ms/key.

- `__workq_kernreturn` (libsystem_kernel.dylib), 0.448 ms/key.

- `kevent_id` (libsystem_kernel.dylib), 0.252 ms/key.

- `objc_msgSend` (libobjcMsgSend1.dylib), 0.052 ms/key.

- `objc_msgSend` (libobjcMsgSend.dylib), 0.046 ms/key.

Nearest sampled framework frames outside those windows:

- `_DPSBlockUntilNextEventMatchingListInMode` (AppKit), 0.977 ms/key.

- `stepTransactionFlush` (AppKit), 0.358 ms/key.

- `_NSEventThread` (AppKit), 0.252 ms/key.

- `-[NSTextView setNeedsDisplayInRect:avoidAdditionalLayout:]` (AppKit), 0.101 ms/key.

- `-[NSTextView(NSTextCheckingClient) annotatedSubstringForProposedRange:actualRange:]` (AppKit), 0.062 ms/key.

<details><summary>latency-appkit-display (4098391), 4.243 Mach CPU ms/key</summary>

| Rank | Self function | Self ms/key | Inclusive function | Inclusive ms/key |
| --- | --- | --- | --- | --- |
| 1 | `mach_msg2_trap` (libsystem_kernel.dylib) | 1.0359 | `main` (main.swift:219) | 4.2432 |
| 2 | `kevent_id` (libsystem_kernel.dylib) | 0.3342 | `start` (dyld) | 4.2432 |
| 3 | `objc_msgSend` (libobjcMsgSend1.dylib) | 0.1238 | `-[NSApplication run]` (AppKit) | 4.2432 |
| 4 | `objc_msgSend` (libobjcMsgSend.dylib) | 0.1123 | `-[NSApplication(NSEventRouting) nextEventMatchingMask:untilDate:inMode:dequeue:]` (AppKit) | 2.4608 |
| 5 | `_xzm_xzone_malloc_tiny` (libsystem_malloc.dylib) | 0.0520 | `-[NSApplication(NSEventRouting) _nextEventMatchingEventMask:untilDate:inMode:dequeue:]` (AppKit) | 2.4608 |
| 6 | `__kdebug_trace64` (libsystem_kernel.dylib) | 0.0441 | `_DPSNextEvent` (AppKit) | 2.4309 |
| 7 | `__CF_IS_OBJC` (CoreFoundation) | 0.0316 | `_DPSBlockUntilNextEventMatchingListInMode` (AppKit) | 2.4288 |
| 8 | `__CFStringHash` (CoreFoundation) | 0.0316 | `ReceiveNextEventCommon` (HIToolbox) | 2.4280 |
| 9 | `mk_timer_arm` (libsystem_kernel.dylib) | 0.0257 | `_BlockUntilNextEventMatchingListInMode` (HIToolbox) | 2.4280 |
| 10 | `_xzm_free_main` (libsystem_malloc.dylib) | 0.0255 | `RunCurrentEventLoopInMode` (HIToolbox) | 2.4090 |

</details>

<details><summary>com.apple.NSEventThread (4098415), 0.431 Mach CPU ms/key</summary>

| Rank | Self function | Self ms/key | Inclusive function | Inclusive ms/key |
| --- | --- | --- | --- | --- |
| 1 | `mach_msg2_trap` (libsystem_kernel.dylib) | 0.3992 | `_pthread_start` (libsystem_pthread.dylib) | 0.4314 |
| 2 | `_kernelrpc_mach_vm_deallocate_trap` (libsystem_kernel.dylib) | 0.0049 | `_NSEventThread` (AppKit) | 0.4313 |
| 3 | `__kdebug_trace64` (libsystem_kernel.dylib) | 0.0018 | `_CFRunLoopRunSpecificWithOptions` (CoreFoundation) | 0.4312 |
| 4 | `<deduplicated_symbol>` (SkyLight) | 0.0016 | `__CFRunLoopRun` (CoreFoundation) | 0.4299 |
| 5 | `_platform_memmove` (libsystem_platform.dylib) | 0.0015 | `mach_msg2_internal` (libsystem_kernel.dylib) | 0.3992 |
| 6 | `_xzm_xzone_malloc_tiny` (libsystem_malloc.dylib) | 0.0013 | `mach_msg2_trap` (libsystem_kernel.dylib) | 0.3992 |
| 7 | `CFAllocatorDeallocate` (CoreFoundation) | 0.0013 | `mach_msg` (libsystem_kernel.dylib) | 0.3981 |
| 8 | `_NSBufferCGEventForGameController` (AppKit) | 0.0013 | `mach_msg_overwrite` (libsystem_kernel.dylib) | 0.3981 |
| 9 | `_kernelrpc_mach_port_deallocate_trap` (libsystem_kernel.dylib) | 0.0012 | `__CFRunLoopServiceMachPort` (CoreFoundation) | 0.3248 |
| 10 | `__bsdthread_ctl` (libsystem_kernel.dylib) | 0.0010 | `__CFRunLoopDoSource1` (CoreFoundation) | 0.1032 |

</details>

<details><summary>Thread <4098413> (4098413), 0.295 Mach CPU ms/key</summary>

| Rank | Self function | Self ms/key | Inclusive function | Inclusive ms/key |
| --- | --- | --- | --- | --- |
| 1 | `__workq_kernreturn` (libsystem_kernel.dylib) | 0.2079 | `_pthread_wqthread` (libsystem_pthread.dylib) | 0.2929 |
| 2 | `__psynch_cvwait` (libsystem_kernel.dylib) | 0.0217 | `__workq_kernreturn` (libsystem_kernel.dylib) | 0.2079 |
| 3 | `mach_msg2_trap` (libsystem_kernel.dylib) | 0.0045 | `_dispatch_lane_invoke` (libdispatch.dylib) | 0.0850 |
| 4 | `iokit_user_client_trap` (IOKit) | 0.0038 | `_dispatch_workloop_worker_thread` (libdispatch.dylib) | 0.0850 |
| 5 | `objc_msgSend` (libobjcMsgSend.dylib) | 0.0030 | `_dispatch_root_queue_drain_deferred_wlh` (libdispatch.dylib) | 0.0850 |
| 6 | `kevent_id` (libsystem_kernel.dylib) | 0.0026 | `_dispatch_lane_serial_drain` (libdispatch.dylib) | 0.0831 |
| 7 | `start_wqthread` (libsystem_pthread.dylib) | 0.0019 | `_dispatch_client_callout` (libdispatch.dylib) | 0.0824 |
| 8 | `__CFStringHash` (CoreFoundation) | 0.0018 | `_dispatch_call_block_and_release` (libdispatch.dylib) | 0.0389 |
| 9 | `objc_msgSendSuper2` (libobjcMsgSend.dylib) | 0.0016 | `-[IATextInputActionsAnalytics mergeOrConsumeAction:]` (InputAnalytics) | 0.0386 |
| 10 | `__CFBasicHashRehash` (CoreFoundation) | 0.0013 | `-[IATextInputActionsAnalytics flushAndSetLastAction:]` (InputAnalytics) | 0.0366 |

</details>

<details><summary>Thread <4098411> (4098411), 0.282 Mach CPU ms/key</summary>

| Rank | Self function | Self ms/key | Inclusive function | Inclusive ms/key |
| --- | --- | --- | --- | --- |
| 1 | `__workq_kernreturn` (libsystem_kernel.dylib) | 0.1990 | `_pthread_wqthread` (libsystem_pthread.dylib) | 0.2806 |
| 2 | `__psynch_cvwait` (libsystem_kernel.dylib) | 0.0177 | `__workq_kernreturn` (libsystem_kernel.dylib) | 0.1990 |
| 3 | `objc_msgSend` (libobjcMsgSend.dylib) | 0.0049 | `_dispatch_workloop_worker_thread` (libdispatch.dylib) | 0.0817 |
| 4 | `mach_msg2_trap` (libsystem_kernel.dylib) | 0.0045 | `_dispatch_root_queue_drain_deferred_wlh` (libdispatch.dylib) | 0.0816 |
| 5 | `__CF_IS_OBJC` (CoreFoundation) | 0.0043 | `_dispatch_lane_invoke` (libdispatch.dylib) | 0.0808 |
| 6 | `iokit_user_client_trap` (IOKit) | 0.0038 | `_dispatch_lane_serial_drain` (libdispatch.dylib) | 0.0804 |
| 7 | `___CFBasicHashFindBucket_Linear` (CoreFoundation) | 0.0018 | `_dispatch_client_callout` (libdispatch.dylib) | 0.0792 |
| 8 | `kevent_id` (libsystem_kernel.dylib) | 0.0018 | `_dispatch_call_block_and_release` (libdispatch.dylib) | 0.0458 |
| 9 | `start_wqthread` (libsystem_pthread.dylib) | 0.0013 | `-[IATextInputActionsAnalytics mergeOrConsumeAction:]` (InputAnalytics) | 0.0454 |
| 10 | `__CFSearchStringROM` (CoreFoundation) | 0.0012 | `-[IATextInputActionsAnalytics flushAndSetLastAction:]` (InputAnalytics) | 0.0443 |

</details>

<details><summary>Thread <4098433> (4098433), 0.255 Mach CPU ms/key</summary>

| Rank | Self function | Self ms/key | Inclusive function | Inclusive ms/key |
| --- | --- | --- | --- | --- |
| 1 | `__workq_kernreturn` (libsystem_kernel.dylib) | 0.1791 | `_pthread_wqthread` (libsystem_pthread.dylib) | 0.2546 |
| 2 | `__psynch_cvwait` (libsystem_kernel.dylib) | 0.0193 | `__workq_kernreturn` (libsystem_kernel.dylib) | 0.1791 |
| 3 | `mach_msg2_trap` (libsystem_kernel.dylib) | 0.0054 | `_dispatch_workloop_worker_thread` (libdispatch.dylib) | 0.0755 |
| 4 | `objc_msgSend` (libobjcMsgSend.dylib) | 0.0038 | `_dispatch_lane_invoke` (libdispatch.dylib) | 0.0753 |
| 5 | `iokit_user_client_trap` (IOKit) | 0.0035 | `_dispatch_root_queue_drain_deferred_wlh` (libdispatch.dylib) | 0.0753 |
| 6 | `__CF_IS_OBJC` (CoreFoundation) | 0.0023 | `_dispatch_lane_serial_drain` (libdispatch.dylib) | 0.0751 |
| 7 | `kevent_id` (libsystem_kernel.dylib) | 0.0017 | `_dispatch_client_callout` (libdispatch.dylib) | 0.0741 |
| 8 | `___CFBasicHashFindBucket_Linear` (CoreFoundation) | 0.0017 | `_dispatch_call_block_and_release` (libdispatch.dylib) | 0.0363 |
| 9 | `_platform_memmove` (libsystem_platform.dylib) | 0.0014 | `-[IATextInputActionsAnalytics mergeOrConsumeAction:]` (InputAnalytics) | 0.0360 |
| 10 | `_xzm_free_main` (libsystem_malloc.dylib) | 0.0013 | `-[IATextInputActionsAnalytics flushAndSetLastAction:]` (InputAnalytics) | 0.0347 |

</details>

<details><summary>Thread <4098412> (4098412), — Mach CPU ms/key</summary>

| Rank | Self function | Self ms/key | Inclusive function | Inclusive ms/key |
| --- | --- | --- | --- | --- |
| 1 | `__workq_kernreturn` (libsystem_kernel.dylib) | 0.0082 | `_pthread_wqthread` (libsystem_pthread.dylib) | 0.0184 |
| 2 | `__psynch_cvwait` (libsystem_kernel.dylib) | 0.0021 | `_dispatch_workloop_worker_thread` (libdispatch.dylib) | 0.0102 |
| 3 | `__psynch_cvbroad` (libsystem_kernel.dylib) | 0.0016 | `_dispatch_lane_serial_drain` (libdispatch.dylib) | 0.0101 |
| 4 | `_CFStringCheckAndGetCharacters` (CoreFoundation) | 0.0009 | `_dispatch_lane_invoke` (libdispatch.dylib) | 0.0101 |
| 5 | `_platform_memmove` (libsystem_platform.dylib) | 0.0008 | `_dispatch_root_queue_drain_deferred_wlh` (libdispatch.dylib) | 0.0101 |
| 6 | `iokit_user_client_trap` (IOKit) | 0.0008 | `_dispatch_client_callout` (libdispatch.dylib) | 0.0084 |
| 7 | `objc_msgSend` (libobjcMsgSend.dylib) | 0.0008 | `__workq_kernreturn` (libsystem_kernel.dylib) | 0.0082 |
| 8 | `_CFGetTSD` (CoreFoundation) | 0.0007 | `_dispatch_call_block_and_release` (libdispatch.dylib) | 0.0045 |
| 9 | `__45-[NSDictionary descriptionWithLocale:indent:]_block_invoke` (CoreFoundation) | 0.0004 | `-[IATextInputActionsAnalytics mergeOrConsumeAction:]` (InputAnalytics) | 0.0045 |
| 10 | `strlen` (dyld) | 0.0003 | `-[IATextInputActionsAnalytics flushAndSetLastAction:]` (InputAnalytics) | 0.0045 |

</details>

<details><summary>Thread <4098434> (4098434), — Mach CPU ms/key</summary>

| Rank | Self function | Self ms/key | Inclusive function | Inclusive ms/key |
| --- | --- | --- | --- | --- |
| 1 | `__workq_kernreturn` (libsystem_kernel.dylib) | 0.0052 | `_pthread_wqthread` (libsystem_pthread.dylib) | 0.0066 |
| 2 | `__psynch_cvwait` (libsystem_kernel.dylib) | 0.0005 | `__workq_kernreturn` (libsystem_kernel.dylib) | 0.0052 |
| 3 | `customRR_retain` (libobjc.A.dylib) | 0.0002 | `_dispatch_lane_serial_drain` (libdispatch.dylib) | 0.0014 |
| 4 | `0x255eb25bc` (source unavailable) | 0.0001 | `_dispatch_lane_invoke` (libdispatch.dylib) | 0.0014 |
| 5 | `-[NSXPCInterface _respondsToRemoteSelector:]` (Foundation) | 0.0001 | `_dispatch_workloop_worker_thread` (libdispatch.dylib) | 0.0014 |
| 6 | `objc_msgSend` (libobjcMsgSend2.dylib) | 0.0001 | `_dispatch_root_queue_drain_deferred_wlh` (libdispatch.dylib) | 0.0014 |
| 7 | `AGX::RenderContext<AGX::G13::Encoders, AGX::G13::Classes, AGX::G13::ObjClasses, AGX::G13::CommandEncoding>::encodeAndEmitRenderState(AGX::G13::VertexProgramVariant const*, AGX::G13::FragmentProgramVariant const*, AGX::G13::FixedLayoutUserArgumentTable*, AGX::G13::FixedLayoutUserArgumentTable*, AGX::RenderDirtyBits<AGX::G13::Classes>&, bool)` (AGXMetalG13X) | 0.0001 | `_dispatch_client_callout` (libdispatch.dylib) | 0.0013 |
| 8 | `AGX::Mempool<16u, 0u, true, 0u, 134217728u, AGX::ImageStateEncoderGen6<kAGXRevG13>::ImageStateFields, AGX::ImageStateEncoderGen6<kAGXRevG13>::ImageStateFields, unsigned long long>::addToResourceList(std::__1::array<AGX::Mempool<16u, 0u, true, 0u, 134217728u, AGX::ImageStateEncoderGen6<kAGXRevG13>::ImageStateFields, AGX::ImageStateEncoderGen6<kAGXRevG13>::ImageStateFields, unsigned long long>::ChunkInfo, 3ul>&&, MTLResourceList*, bool*)` (AGXMetalG13X) | 0.0001 | `CA::CG::Queue::Flush::callback(void*)` (QuartzCore) | 0.0006 |
| 9 | `__psynch_cvbroad` (libsystem_kernel.dylib) | 0.0001 | `invocation function for block in CA::CG::Queue::Flush::callback(void*)` (QuartzCore) | 0.0006 |
| 10 | `iokit_user_client_trap` (IOKit) | 0.0001 | `CA::CG::AccelQueue::flush_all_renderers(CA::CG::Queue::FlushMode)` (QuartzCore) | 0.0006 |

</details>

<details><summary>Thread <4098438> (4098438), — Mach CPU ms/key</summary>

| Rank | Self function | Self ms/key | Inclusive function | Inclusive ms/key |
| --- | --- | --- | --- | --- |
| 1 | `__workq_kernreturn` (libsystem_kernel.dylib) | 0.0033 | `_pthread_wqthread` (libsystem_pthread.dylib) | 0.0054 |
| 2 | `0x18debd130` (source unavailable) | 0.0007 | `__workq_kernreturn` (libsystem_kernel.dylib) | 0.0033 |
| 3 | `__psynch_cvwait` (libsystem_kernel.dylib) | 0.0005 | `_dispatch_lane_serial_drain` (libdispatch.dylib) | 0.0021 |
| 4 | `objc_msgSend` (libobjcMsgSend3.dylib) | 0.0005 | `_dispatch_lane_invoke` (libdispatch.dylib) | 0.0021 |
| 5 | `-[NSString quotedStringRepresentation]` (Foundation) | 0.0002 | `_dispatch_workloop_worker_thread` (libdispatch.dylib) | 0.0021 |
| 6 | `CA::OGL::GlyphCache::flush(unsigned long, CGGlyphBitmap const* const*, CGGlyphPosition const*)` (QuartzCore) | 0.0001 | `_dispatch_client_callout` (libdispatch.dylib) | 0.0021 |
| 7 | `iokit_user_client_trap` (IOKit) | 0.0000 | `_dispatch_root_queue_drain_deferred_wlh` (libdispatch.dylib) | 0.0021 |
| 8 | `free` (libsystem_malloc.dylib) | 0.0000 | `-[IATextInputActionsAnalytics flushAndSetLastAction:]` (InputAnalytics) | 0.0014 |
| 9 | — | — | `_dispatch_call_block_and_release` (libdispatch.dylib) | 0.0014 |
| 10 | — | — | `-[IATextInputActionsAnalytics mergeOrConsumeAction:]` (InputAnalytics) | 0.0014 |

</details>

<details><summary>Thread <4098436> (4098436), — Mach CPU ms/key</summary>

| Rank | Self function | Self ms/key | Inclusive function | Inclusive ms/key |
| --- | --- | --- | --- | --- |
| 1 | `_dispatch_alloc_continuation_free` (libdispatch.dylib) | 0.0000 | `_pthread_exit` (libsystem_pthread.dylib) | 0.0000 |
| 2 | — | — | `_dispatch_cache_cleanup` (libdispatch.dylib) | 0.0000 |
| 3 | — | — | `_dispatch_alloc_continuation_free` (libdispatch.dylib) | 0.0000 |
| 4 | — | — | `_pthread_wqthread_exit` (libsystem_pthread.dylib) | 0.0000 |
| 5 | — | — | `_pthread_wqthread` (libsystem_pthread.dylib) | 0.0000 |
| 6 | — | — | `_pthread_tsd_cleanup` (libsystem_pthread.dylib) | 0.0000 |

</details>

## Diagnostic controls

The local latency references use `RUSTUI_G5_L8_CPU=1` for process,
thread and native-handler CPU clocks. The separate blocked-stack capture
also sets `RUSTUI_G5_L8_WAIT_STACKS=1`. For no-input windows, set
`RUSTUI_G5_L8_IDLE=focused` or `RUSTUI_G5_L8_IDLE=unfocused` and turn off
the ScreenCaptureKit observer. The measurements used the L7 matched
workloads and frozen release binaries with the local sync-off, L4
phase and optional GPUI Kit caret-count probes. The temporary GPUI
patches were reversed after building those binaries.
