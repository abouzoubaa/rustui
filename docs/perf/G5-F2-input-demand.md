# G5-F2 — native input owns its frame demand

The G5-F1 fix stopped RustUI from stalling during native typing. Sparse typing still showed most edits through the display link. Its CPU cost also rose in all five pairs. This ticket keeps the G5-F1 safety guards and adds one opt-in switch, `GPUI_FAST_INPUT_SCOPED_DEMAND=1`.

## G5-F1 trace and accounting

The earlier estimate of 2,841 non-immediate presentations subtracted all 52 drawable acquisitions from 2,893 shown versions. It mixed acquisition calls, including calls outside the measured edit and repeated or unshown frames, with first presentations. Joining each first-visible version through its draw, attempt, submission and Metal presented callback identifies 14 immediate and **2,879 scheduled** presentations.

| First visible path or preceding trace observation | Versions |
| --- | ---: |
| Immediate submission | 14 |
| Scheduled, link already attached at input | 2,796 |
| Scheduled, link recreation between handler and submission | 54 |
| Scheduled, CPU admission deferred before the first shown draw | 29 |

The counts use mutually exclusive observations. Admission deferral takes precedence over recreation, followed by an attached source at the input snapshot. The old captures cannot identify the exact caller of each frame request.

A paired diagnostic run posted 20 keys at 10/s with G5-F1 active in both arms. The trace timestamps Cocoa `sendEvent:`, the GPUI key handler, the platform text service's `handleEvent:` call, and nested `insertText:`/marked text callbacks. This is the actual text interpretation entry point on this path. The probe records invalidate/notify versus refresh, the binding, colour or caret source when it is known, the post-event wake, frame admission, and Metal-link lifecycle. Unmarked demand remains “other.” The version barcode is unnecessary for the Metal presented-time join.

With F2 off, 19 of 20 versions appeared. All 19 were scheduled and one link was recreated inside the measured window. The GPUI key-up handler rebuilt the dirty scene for keyboard routing 19 times. The post-event wakes then found a clean scene. GPUI still had `needs_present` set because the rebuild prepared the scene for key dispatch without submitting it. G5-F1 tested only scene dirtiness on the immediate path, so the display link presented the prepared scene.

The first F2 preflight also exposed a clean-wake loop. A generic tick raised during the wake could enqueue another wake long after the dirty transition that started it. The final candidate declines that self-requeue, checks `needs_present` separately, and submits the prepared scene through the immediate path. Genuine follow-up callbacks and dirty changes still use admission. In the final diagnostic run 19 shown versions used the immediate path and zero links were recreated. Eighteen wakes explicitly saw a prebuilt scene. The median handler-to-presented time was 51.48 ms with F2 off and 27.87 ms with F2 on in these short trace runs. The 60-second captures below determine the gate.

## Candidate

In the native input boundary, frame requests join the single per-window post-event wake before binding or colour cause routing can send them to the display link. The hold starts on the first native input and extends from the last native input, including at sparse typing rates. During the hold, scheduled demand and callbacks use the same immediate submission path and per-surface admission. A normal display-link callback already in flight yields to that wake. When the native key handler has already built the scene, the wake can submit it without rebuilding it. The existing watchdog can still suspend immediate routing, and force-render requests retain their renderer recovery path.

## Initial sanity decision

PR #175 initially remained draft. Both typing rates met the delay, immediate share,
link-recreation, CPU, and correctness targets. The hover sanity run also met
both targets. Drag reached 23.65 ms median, but process CPU increased
**5.29%** against its same-build F1 control (0.268523 → 0.282721 CPU s/s).
That exceeds the 5% sanity limit. The separate first drag capture was
also above the historical F1 median by 9.0%. The matched retest below supersedes that sanity-based merge decision.

Two hover captures were excluded after Core Graphics targeted the process
but AppKit reported window ID 0 for every delivered mouse move. The native
proof directly showed that GPUI consumed none of them. The valid hover sanity
comparison used the opt-in `--target-hover` driver in both arms. It queues
an NSEvent addressed to the benchmark window. Both arms share the same
posting method, including timestamp and version lookup. The new helper was
compiled after the typing pairs; those pairs kept their original frozen
helper. The RustUI release executable is unchanged across both sets.

Local `check.sh` passed 1,880 tests across seven groups (1,873 passed,
7 ignored). Candidate source verification passed all 732 tests. Both stacks
passed with 94 ordered patches, one alternative and 161 exact postimages.
GitHub source CI passed for the measured runtime commit `de40273e8`.


## Paired release results

Both arms use the native Cocoa driver on one pinned release build with damage off, PX-3 on for typing and the same buffered diagnostics. The control has the immediate switches off. The candidate enables immediate presentation, draw-first, coalescing, any-change, F1, and F2. Each typing rate runs five 60-second pairs in alternating order. Hover and drag each use a 60-second F1-on control and a 60-second F2-on candidate on the same release executable. Both arms use the same native input helper for that workload.

| Workload | event timestamp → presented median/p95 ms | handler → presented median/p95 ms | CPU s/s | CPU verdict / sanity delta |
| --- | --- | --- | --- | --- |
| Typing 10/s | 49.11/53.66 → 27.80/34.64 | 48.17/52.32 → 26.52/31.99 | 0.0872 → 0.0902 | INCONCLUSIVE |
| Typing 30/s | 47.56/55.71 → 26.67/34.36 | 46.54/54.80 → 24.31/31.03 | 0.2391 → 0.2284 | INCONCLUSIVE |
| Hover | 24.15/28.86 → 24.09/29.39 | 24.05/28.66 → 23.98/28.95 | 0.3453 → 0.3395 | Sanity -1.69% |
| Drag | 23.91/28.11 → 23.88/27.95 | 23.69/27.78 → 23.65/27.68 | 0.2685 → 0.2827 | Sanity +5.29% |

| Typing rate | Five CPU/s changes % | Candidate immediate share | Maximum link recreations in any burst |
| --- | --- | --- | ---: |
| 10/s | +25.28, +3.47, +7.66, -5.92, -5.89 | 2894/2894 (100.00%) | 0 |
| 30/s | -3.33, +0.66, -3.22, -4.49, -7.88 | 8602/8602 (100.00%) | 0 |

Typing medians and p95 values are medians of the five corresponding capture quantiles. The immediate share uses actual presentations with a native edit tag inside the measured window, including repeated presents. A burst is a maximal input sequence with consecutive gaps below 250 ms. A link recreation counts if it falls between the first input and 250 ms after the last input in that burst.

24 of 24 captures were measured on a loaded machine. Across all start and end samples, background CPU ranged from 3.23% to 61.02%. Logs record the top five processes and the separate samples for each capture.

| Capture | Background CPU | Posted/delivered/shown | Superseded delivered versions | Lost edits / order / watchdog |
| --- | --- | --- | ---: | --- |
| drag-sanity-f0 | measured on a loaded machine (4.96% background at start; 17.41% after) | 6183/6183/6167 | 16 | — / 0 / 0 |
| drag-sanity-f1 | measured on a loaded machine (16.27% background at start; 16.54% after) | 6180/6180/6168 | 12 | — / 0 / 0 |
| hover-sanity-f0 | measured on a loaded machine (5.69% background at start; 16.10% after) | 6227/6227/6202 | 25 | — / 0 / 0 |
| hover-sanity-f1 | measured on a loaded machine (11.67% background at start; 15.64% after) | 5928/5928/5921 | 7 | — / 0 / 0 |
| typing-10-1-off | measured on a loaded machine (3.23% background at start; 29.35% after) | 600/600/575 | 25 | 0 / 0 / 0 |
| typing-10-1-on | measured on a loaded machine (24.55% background at start; 4.78% after) | 600/600/578 | 22 | 0 / 0 / 0 |
| typing-10-2-off | measured on a loaded machine (13.46% background at start; 13.47% after) | 600/600/575 | 25 | 0 / 0 / 0 |
| typing-10-2-on | measured on a loaded machine (5.36% background at start; 15.21% after) | 600/600/579 | 21 | 0 / 0 / 0 |
| typing-10-3-off | measured on a loaded machine (12.92% background at start; 53.40% after) | 600/600/580 | 20 | 0 / 0 / 0 |
| typing-10-3-on | measured on a loaded machine (61.02% background at start; 32.14% after) | 600/600/578 | 22 | 0 / 0 / 0 |
| typing-10-4-off | measured on a loaded machine (20.92% background at start; 5.88% after) | 600/600/575 | 25 | 0 / 0 / 0 |
| typing-10-4-on | measured on a loaded machine (37.82% background at start; 21.10% after) | 600/600/579 | 21 | 0 / 0 / 0 |
| typing-10-5-off | measured on a loaded machine (5.79% background at start; 15.88% after) | 600/600/575 | 25 | 0 / 0 / 0 |
| typing-10-5-on | measured on a loaded machine (12.13% background at start; 11.58% after) | 600/600/577 | 23 | 0 / 0 / 0 |
| typing-30-1-off | measured on a loaded machine (16.66% background at start; 12.62% after) | 1800/1800/1713 | 87 | 0 / 0 / 0 |
| typing-30-1-on | measured on a loaded machine (12.36% background at start; 13.22% after) | 1800/1800/1711 | 89 | 0 / 0 / 0 |
| typing-30-2-off | measured on a loaded machine (15.07% background at start; 16.06% after) | 1800/1800/1710 | 90 | 0 / 0 / 0 |
| typing-30-2-on | measured on a loaded machine (9.77% background at start; 15.39% after) | 1800/1800/1707 | 93 | 0 / 0 / 0 |
| typing-30-3-off | measured on a loaded machine (14.40% background at start; 16.31% after) | 1800/1800/1710 | 90 | 0 / 0 / 0 |
| typing-30-3-on | measured on a loaded machine (8.74% background at start; 13.56% after) | 1800/1800/1749 | 51 | 0 / 0 / 0 |
| typing-30-4-off | measured on a loaded machine (19.95% background at start; 13.70% after) | 1800/1800/1713 | 87 | 0 / 0 / 0 |
| typing-30-4-on | measured on a loaded machine (21.17% background at start; 13.53% after) | 1800/1800/1716 | 84 | 0 / 0 / 0 |
| typing-30-5-off | measured on a loaded machine (17.67% background at start; 23.42% after) | 1800/1800/1713 | 87 | 0 / 0 / 0 |
| typing-30-5-on | measured on a loaded machine (17.25% background at start; 21.50% after) | 1800/1800/1725 | 75 | 0 / 0 / 0 |

Every unmatched delivered version had a later presented successor. The native driver did not report any lost keyboard edit, order violation, or watchdog fallback.

### Drawable acquisitions
0 measured acquisitions exceeded 2 ms.

| Capture | Phase | Wait ms | Path | In flight | Time since detach ms |
| --- | --- | ---: | --- | ---: | ---: |
| drag-sanity-f0 | warmup | 1.128 | non-immediate | 0.0 | — |
| drag-sanity-f0 | measured | 1.109 | immediate | 2.0 | 2603.850 |
| drag-sanity-f1 | measured | 1.402 | immediate | 2.0 | 2466.290 |
| hover-sanity-f0 | warmup | 1.047 | non-immediate | 0.0 | — |
| hover-sanity-f1 | warmup | 1.017 | non-immediate | 0.0 | — |
| typing-10-1-off | warmup | 3.995 | non-immediate | 0.0 | — |
| typing-10-1-on | warmup | 1.104 | non-immediate | 0.0 | — |
| typing-10-3-off | warmup | 1.026 | non-immediate | 0.0 | — |
| typing-10-4-on | warmup | 1.079 | non-immediate | 0.0 | — |
| typing-10-4-on | warmup | 1.023 | immediate | 0.0 | 0.665 |
| typing-10-5-on | warmup | 1.127 | non-immediate | 0.0 | — |
| typing-30-2-off | warmup | 1.064 | non-immediate | 0.0 | — |
| typing-30-2-on | warmup | 1.121 | non-immediate | 0.0 | — |
| typing-30-3-on | warmup | 1.002 | non-immediate | 0.0 | — |
| typing-30-4-on | warmup | 1.010 | non-immediate | 0.0 | — |
| typing-30-5-off | warmup | 1.009 | non-immediate | 0.0 | — |
| typing-30-5-on | warmup | 1.049 | non-immediate | 0.0 | — |


## Drag finish — five matched pairs

The retest held G5-F1 active in both arms and toggled only
`GPUI_FAST_INPUT_SCOPED_DEMAND`. It used the same release executable and
native Cocoa helper as the original drag sanity comparison. Each capture
measured 60 seconds after warmup. Pair order alternated F1→F2, F2→F1,
F1→F2, F2→F1, F1→F2. All captures were loaded. No order violations,
lost edits or watchdog fallbacks occurred. All delivered versions that did
not appear had a later presented successor.

| Metric | F1 | F2 |
| --- | ---: | ---: |
| Handler→presentedTime median / p95 (ms) | 23.66 / 27.56 | 23.65 / 27.63 |
| Native event timestamp→presentedTime median / p95 (ms) | 23.89 / 27.82 | 23.87 / 27.93 |
| Total process CPU s/s | 0.27937 | 0.28158 |

Five paired CPU changes were **+0.79%, +3.89%, +2.81%, −0.11%, −4.82%**.
The P verdict is **INCONCLUSIVE**. The updated merge rule accepts that
verdict and the candidate median below 28 ms, subject to source CI and
reviews.

| Capture | CPU s/s | Handler→presented median / p95 (ms) | Background CPU |
| --- | ---: | ---: | --- |
| 1 off | 0.27937 | 23.65 / 27.41 | measured on a loaded machine (5.27% background at start; 15.11% after) |
| 1 on | 0.28158 | 23.69 / 27.69 | measured on a loaded machine (15.80% background at start; 15.61% after) |
| 2 on | 0.27829 | 23.59 / 27.24 | measured on a loaded machine (11.54% background at start; 17.81% after) |
| 2 off | 0.26786 | 23.55 / 27.10 | measured on a loaded machine (14.99% background at start; 16.78% after) |
| 3 off | 0.27246 | 23.66 / 27.56 | measured on a loaded machine (14.31% background at start; 17.15% after) |
| 3 on | 0.28011 | 23.70 / 27.75 | measured on a loaded machine (12.19% background at start; 16.18% after) |
| 4 on | 0.28530 | 23.65 / 27.53 | measured on a loaded machine (12.85% background at start; 14.53% after) |
| 4 off | 0.28560 | 23.76 / 27.78 | measured on a loaded machine (19.36% background at start; 14.94% after) |
| 5 off | 0.29620 | 23.75 / 27.62 | measured on a loaded machine (12.28% background at start; 17.79% after) |
| 5 on | 0.28193 | 23.64 / 27.63 | measured on a loaded machine (12.64% background at start; 15.73% after) |

| Capture | Phase | nextDrawable (ms) | Path | In flight | Time since detach (ms) |
| --- | --- | ---: | --- | ---: | ---: |
| 1 off | warmup | 2.223 | immediate | 0.0 | 0.222 |
| 1 off | measured | 2.638 | immediate | 2.0 | 2497.874 |
| 1 off | measured | 1.861 | immediate | 1.0 | 37458.372 |
| 1 on | warmup | 1.184 | non-immediate | 0.0 | — |
| 2 on | measured | 2.582 | immediate | 1.0 | 2980.629 |
| 2 off | warmup | 4.237 | non-immediate | 0.0 | — |
| 2 off | measured | 2.411 | immediate | 2.0 | 2554.354 |
| 3 off | warmup | 1.131 | non-immediate | 0.0 | — |
| 3 off | warmup | 1.438 | immediate | 0.0 | 0.238 |
| 3 off | measured | 1.694 | immediate | 2.0 | 2822.937 |
| 3 on | warmup | 1.196 | non-immediate | 0.0 | — |
| 4 on | warmup | 1.115 | non-immediate | 0.0 | — |
| 4 on | measured | 1.153 | immediate | 1.0 | 2412.474 |
| 4 off | measured | 1.039 | immediate | 2.0 | 2778.831 |
| 5 on | warmup | 1.047 | non-immediate | 0.0 | — |
| 5 on | measured | 1.081 | immediate | 2.0 | 2445.307 |
