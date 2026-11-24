# G5-F1 — native input scheduling

The branch starts from G5-3's merge into `g5-integration`. It carries the L1 diagnostic probe for native input and a separate reversible scheduler candidate. `GPUI_FAST_NATIVE_SCHEDULER=1` opts into the scheduler on top of immediate presentation. The fix uses the same scene, per-surface acquisition, order, and capacity guards as G5-3.

## Reproduction

The initial release control used the L1 runner to post 50 native key events to the RustUI editor after its normal warmup. The deadline was 10 seconds from process launch. Letters and backspaces alternate. PX-3 was on, damage was off, and the frame recorder was buffered.

| Immediate / draw-first / coalescing / any-change | Before G5-F1 |
| --- | --- |
| 0 / 0 / 0 / 0 | Exit 0 in 8.295 s |
| 1 / 1 / 1 / 1 | Timed out at 10 s |
| 1 / 0 / 0 / 0 | Exit 0 in 8.218 s |
| 0 / 1 / 0 / 0 | Exit 0 in 8.190 s |
| 0 / 0 / 1 / 0 | Exit 0 in 8.214 s |
| 0 / 0 / 0 / 1 | Exit 0 in 8.169 s |

The last three features depend on immediate presentation, so those single-switch controls leave their dependent feature inactive. The runner also exercises each dependent switch with immediate presentation enabled.

With the fix switched off, the native trace reproduced the timeout. One wake entered the draw path with dirty state, input sequence 4, scene generation 13, and the mask for AppKit event dispatch, key handling, and text insertion. The old draw-first deferred effect runs during this stack. The old demand path then feeds frame requests back into the immediate queue. App effect flushing requests another frame when dirty state, a pending presentation, or next-frame callbacks remain. Because any-change intercepts generic scheduled demand, this can queue more immediate work without a new edit. The trace records repeated requests with the same input and generation, including requests while the scene is clean. In the diagnostic trace, the input sequence counts GPUI platform inputs including key releases. The separate L1 version identifies one posted measured input.

The candidate queues one wake after the outer AppKit boundary, including text callbacks nested inside native event handling. Generic frame requests go to the normal display link. Dirty transitions from outside immediate frame effects may request an any-change draw and still pass per-surface coalescing. Binding or colour notifications use ordinary ticks and retain the existing visible-change checks. The per-window watchdog routes work to normal ticks for 250 ms if more than three immediate wakes for the same input reach the draw path within a refresh interval.

## Checks and measurement

Build with `./scripts/build_g5_refs.sh` before running the driver. Use a fresh output directory per invocation.

```sh
python3 scripts/run_g5_f1.py reproduce --output target/headstart/g5-f1/reproduce
python3 scripts/run_g5_f1.py proof --output target/headstart/g5-f1/proof
python3 scripts/run_g5_f1.py pairs --output target/headstart/g5-f1/pairs
```

`GPUI_FAST_TRACE_WAKES=1` is for untimed reproduction. It records wake source, dirty state at draw entry, scene generation, input sequence, and the AppKit callback mask. The live trace samples long repeats at powers of two after the first 48 entries for an input so a hung process leaves evidence without an unbounded log. Cause codes distinguish native input, binding, frame effects, colour result, external invalidation, generic demand, coalescing timer, and drawable completion. Event callbacks can contain nested key, insert-text, or marked-text work.

The paired driver runs 60-second windows after warmup, alternating the first arm, with trace and screen capture off. The keyboard driver posts tagged Core Graphics events to the app. Hover uses tagged Core Graphics mouse moves. Drag enqueues target-window NSEvents through AppKit, which preserves the window association. AppKit reconstructs the drag event before dispatch and can discard its event number and object association. The driver also joins it by the recorded event timestamp. Motion sends one sample per wake and leaves late samples unposted instead of replaying a burst. The capture reports the actual number posted. The reported timestamp starts at the synthetic event's creation. The handler starts at AppKit delivery. Both endpoints use actual Metal `presentedTime`. The raw result distinguishes events coalesced before GPUI delivery from delivered states superseded by a later displayed version, and reports versions without a later display separately. The CPU verdict uses process CPU seconds per measured wall second under the P rule. Loaded captures include the measured background CPU.

The release proof completed all 50-key RustUI controls within 10 seconds, including all-on, individual switches, and each feature with its immediate prerequisite. Bare typing, hover, and drag also completed. There were zero lost edits, order violations, and watchdog fallbacks. Drag delivered all 530 posted samples and joined 513 versions to a presentation. Each other delivered version had a later presented successor.

Local `check.sh` passed 1,880 tests across all seven groups (1,873 passed, 7 ignored). Candidate source verification passed 732 tests. Both stacks passed with 93 ordered patches, one alternative and 161 exact postimages. The synthetic draw-first editor checks passed all four cases with the candidate switches enabled. The focused scheduler and coalescing proof passed eight Rust tests, including nested callback wake coalescing and watchdog recovery. The standalone Fast Cargo test command could not resolve its independent offline dependency set. The source fixture compiles the actual scheduler module and runs its tests.

The 40 matched captures finished on the frozen release build at commit
`237c5daa6ba47a662c8f2365dddcde63f4d311ce`. Each measured window lasted
60.001–60.105 seconds. All captures were measured on a loaded machine.
The start/end background CPU snapshots ranged from 4.57% to 21.23%.
The runner retained the top five processes for every capture.

All captures had zero out-of-order presentations, lost keyboard edits, and
watchdog fallbacks. Every delivered version without a matching
presentation had a later version that superseded it. Eleven posted hover
moves were coalesced before delivery, six off and five on. The driver
records each superseded version and the later version that appeared.

### Merge decision

PR #175 remains draft. Typing at 10 keys/s was a CPU REGRESSION under the
P rule. Each of its five pairs used 9.28–13.72% more CPU with the candidate.
The candidate removed the stall while keeping the slower display-link
presentation path for most typing frames. At 10 and 30 keys/s, 52 and 40
direct drawable acquisitions were recorded across the five candidate runs
at each rate, compared with 2,893 and 8,563 matched typing presentations.
The candidate logged 161 and 190 rendering-link recreations, respectively.
These counters describe path use. They do not attribute the CPU increase
to a single function. Hover and drag improved on both delay and CPU.

Candidate source CI passed on the measured commit. The G5-L1 measurement
ticket remains blocked on the merge gate.


## Native release proof

| workload / switches | posted | delivered | joined to presentation | delivered versions superseded | lost edits | order violations | watchdog |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 0000 | 50 | 50 | 47 | 3 | 0 | 0 | 0 |
| 0001 | 50 | 50 | 47 | 3 | 0 | 0 | 0 |
| 0010 | 50 | 50 | 47 | 3 | 0 | 0 | 0 |
| 0100 | 50 | 50 | 47 | 3 | 0 | 0 | 0 |
| 1000 | 50 | 50 | 47 | 3 | 0 | 0 | 0 |
| 1001 | 50 | 50 | 48 | 2 | 0 | 0 | 0 |
| 1010 | 50 | 50 | 47 | 3 | 0 | 0 | 0 |
| 1100 | 50 | 50 | 47 | 3 | 0 | 0 | 0 |
| 1111 | 50 | 50 | 48 | 2 | 0 | 0 | 0 |
| bare-typing | 50 | 50 | 50 | 0 | 0 | 0 | 0 |
| lab-hover | 523 | 522 | 503 | 19 | — | 0 | 0 |
| lab-drag | 530 | 530 | 513 | 17 | — | 0 | 0 |

## Matched native captures

| scene | event timestamp → presented median/p95 ms, off → on | handler → presented median/p95 ms, off → on | CPU s/s off → on | P verdict | five CPU changes % |
| --- | --- | --- | --- | --- | --- |
| typing-10 | 50.73/53.68 → 52.83/57.13 | 48.78/52.17 → 51.31/55.63 | 0.0976 → 0.1094 | REGRESSION | +11.13, +12.03, +9.28, +13.72, +11.96 |
| typing-30 | 47.82/57.10 → 46.92/51.75 | 46.66/54.84 → 46.05/49.22 | 0.2410 → 0.2290 | INCONCLUSIVE | -7.48, -6.50, -3.02, -0.69, -5.90 |
| hover | 45.66/51.33 → 26.97/31.02 | 45.30/49.19 → 26.36/30.50 | 0.3888 → 0.3510 | GAIN | -3.39, -12.05, -9.25, -16.39, -10.17 |
| drag | 46.81/50.96 → 24.31/32.61 | 46.63/49.20 → 24.10/32.39 | 0.3081 → 0.2725 | GAIN | -19.21, -12.08, -9.97, -14.39, -12.78 |

| capture | background CPU | delivered/posted | joined/received | superseded | missing without later frame | order / lost / watchdog |
| --- | --- | --- | --- | ---: | ---: | --- |
| drag-10-1-off | measured on a loaded machine (13.24% background at start; 15.77% after) | 6249/6249 | 6232/6249 | 17 | 0 | 0 / — / 0 |
| drag-10-1-on | measured on a loaded machine (11.49% background at start; 14.38% after) | 6337/6337 | 6283/6337 | 54 | 0 | 0 / — / 0 |
| drag-10-2-off | measured on a loaded machine (17.43% background at start; 16.72% after) | 6269/6269 | 6254/6269 | 15 | 0 | 0 / — / 0 |
| drag-10-2-on | measured on a loaded machine (11.33% background at start; 14.39% after) | 6344/6344 | 6287/6344 | 57 | 0 | 0 / — / 0 |
| drag-10-3-off | measured on a loaded machine (15.30% background at start; 18.29% after) | 6219/6219 | 6200/6219 | 19 | 0 | 0 / — / 0 |
| drag-10-3-on | measured on a loaded machine (11.77% background at start; 16.83% after) | 6256/6256 | 6115/6256 | 141 | 0 | 0 / — / 0 |
| drag-10-4-off | measured on a loaded machine (14.35% background at start; 16.05% after) | 6198/6198 | 6171/6198 | 27 | 0 | 0 / — / 0 |
| drag-10-4-on | measured on a loaded machine (11.61% background at start; 19.23% after) | 6277/6277 | 6149/6277 | 128 | 0 | 0 / — / 0 |
| drag-10-5-off | measured on a loaded machine (14.80% background at start; 18.14% after) | 6195/6195 | 6171/6195 | 24 | 0 | 0 / — / 0 |
| drag-10-5-on | measured on a loaded machine (15.89% background at start; 18.35% after) | 6275/6275 | 6185/6275 | 90 | 0 | 0 / — / 0 |
| hover-10-1-off | measured on a loaded machine (14.22% background at start; 16.50% after) | 6132/6134 | 6039/6132 | 93 | 0 | 0 / — / 0 |
| hover-10-1-on | measured on a loaded machine (11.06% background at start; 14.92% after) | 6307/6308 | 6151/6307 | 156 | 0 | 0 / — / 0 |
| hover-10-2-off | measured on a loaded machine (10.42% background at start; 16.84% after) | 6109/6110 | 6100/6109 | 9 | 0 | 0 / — / 0 |
| hover-10-2-on | measured on a loaded machine (14.53% background at start; 13.16% after) | 6325/6326 | 6223/6325 | 102 | 0 | 0 / — / 0 |
| hover-10-3-off | measured on a loaded machine (6.06% background at start; 16.52% after) | 6109/6110 | 6100/6109 | 9 | 0 | 0 / — / 0 |
| hover-10-3-on | measured on a loaded machine (4.57% background at start; 16.18% after) | 6315/6316 | 6182/6315 | 133 | 0 | 0 / — / 0 |
| hover-10-4-off | measured on a loaded machine (10.03% background at start; 15.70% after) | 6042/6043 | 6026/6042 | 16 | 0 | 0 / — / 0 |
| hover-10-4-on | measured on a loaded machine (6.85% background at start; 14.76% after) | 6319/6320 | 6207/6319 | 112 | 0 | 0 / — / 0 |
| hover-10-5-off | measured on a loaded machine (19.28% background at start; 14.92% after) | 6101/6102 | 6094/6101 | 7 | 0 | 0 / — / 0 |
| hover-10-5-on | measured on a loaded machine (8.22% background at start; 13.51% after) | 6314/6315 | 6191/6314 | 123 | 0 | 0 / — / 0 |
| typing-10-1-off | measured on a loaded machine (6.29% background at start; 13.92% after) | 600/600 | 577/600 | 23 | 0 | 0 / 0 / 0 |
| typing-10-1-on | measured on a loaded machine (12.89% background at start; 14.15% after) | 600/600 | 577/600 | 23 | 0 | 0 / 0 / 0 |
| typing-10-2-off | measured on a loaded machine (11.93% background at start; 15.07% after) | 600/600 | 574/600 | 26 | 0 | 0 / 0 / 0 |
| typing-10-2-on | measured on a loaded machine (13.96% background at start; 18.64% after) | 600/600 | 583/600 | 17 | 0 | 0 / 0 / 0 |
| typing-10-3-off | measured on a loaded machine (11.18% background at start; 15.04% after) | 600/600 | 574/600 | 26 | 0 | 0 / 0 / 0 |
| typing-10-3-on | measured on a loaded machine (11.01% background at start; 17.26% after) | 600/600 | 576/600 | 24 | 0 | 0 / 0 / 0 |
| typing-10-4-off | measured on a loaded machine (15.32% background at start; 16.06% after) | 600/600 | 575/600 | 25 | 0 | 0 / 0 / 0 |
| typing-10-4-on | measured on a loaded machine (11.21% background at start; 15.35% after) | 600/600 | 579/600 | 21 | 0 | 0 / 0 / 0 |
| typing-10-5-off | measured on a loaded machine (12.82% background at start; 16.58% after) | 600/600 | 574/600 | 26 | 0 | 0 / 0 / 0 |
| typing-10-5-on | measured on a loaded machine (11.97% background at start; 15.68% after) | 600/600 | 578/600 | 22 | 0 | 0 / 0 / 0 |
| typing-30-1-off | measured on a loaded machine (13.11% background at start; 16.03% after) | 1800/1800 | 1712/1800 | 88 | 0 | 0 / 0 / 0 |
| typing-30-1-on | measured on a loaded machine (11.28% background at start; 12.66% after) | 1800/1800 | 1713/1800 | 87 | 0 | 0 / 0 / 0 |
| typing-30-2-off | measured on a loaded machine (14.75% background at start; 13.82% after) | 1800/1800 | 1712/1800 | 88 | 0 | 0 / 0 / 0 |
| typing-30-2-on | measured on a loaded machine (15.85% background at start; 15.70% after) | 1800/1800 | 1714/1800 | 86 | 0 | 0 / 0 / 0 |
| typing-30-3-off | measured on a loaded machine (15.25% background at start; 16.94% after) | 1800/1800 | 1713/1800 | 87 | 0 | 0 / 0 / 0 |
| typing-30-3-on | measured on a loaded machine (14.96% background at start; 12.80% after) | 1800/1800 | 1711/1800 | 89 | 0 | 0 / 0 / 0 |
| typing-30-4-off | measured on a loaded machine (20.64% background at start; 14.74% after) | 1800/1800 | 1722/1800 | 78 | 0 | 0 / 0 / 0 |
| typing-30-4-on | measured on a loaded machine (15.10% background at start; 21.23% after) | 1800/1800 | 1713/1800 | 87 | 0 | 0 / 0 / 0 |
| typing-30-5-off | measured on a loaded machine (14.64% background at start; 14.71% after) | 1800/1800 | 1712/1800 | 88 | 0 | 0 / 0 / 0 |
| typing-30-5-on | measured on a loaded machine (14.24% background at start; 15.01% after) | 1800/1800 | 1712/1800 | 88 | 0 | 0 / 0 / 0 |

The reported medians and p95s are medians of the five capture quantiles for each arm. Hover and drag request up to 120 pointer samples per second.

### Measured drawable waits above 2 ms

| Capture | nextDrawable ms | path | in flight before call | ms since detach |
| --- | ---: | --- | ---: | ---: |
| drag-10-2-on | 2.760 | immediate | 2.0 | 2530.581 |
| drag-10-2-on | 2.715 | immediate | 2.0 | 53135.107 |
