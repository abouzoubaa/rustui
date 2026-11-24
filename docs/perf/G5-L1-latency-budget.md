# G5-L1 native-input latency budget

The measurement base is `g5-integration` at
`953409e7848077cbdfb36ee009bf2d0f49bde1cc`, including G5-F1 and G5-F2.
This ticket measures the existing candidate scheduler. It changes no product
scheduling defaults.

## Method

Build once with `./scripts/build_g5_refs.sh` using pinned Rust 1.98.0 release
and line-table symbols. Keep damage off, frame logging buffered, and PX-3
enabled for RustUI typing. Capture after warmup on the built-in display of an
M1 Max Mac on macOS 27.3 (26C5047e). The runner records background CPU at both
ends, the app and helper hashes, and launch configuration. Each pair uses the
same executables with alternating order.

Native typing diagnostics post tagged Core Graphics key events through Cocoa
and alternate X and Backspace. Every delivered event is recorded at
`NSApplication.sendEvent:`. The `NSEvent.timestamp` on these events is a
synthetic event timestamp. Physical typing uses the opt-in `--physical`
runner option and needs a person at the keyboard. Hover uses mouse events
addressed to the RustUI window and delivered through Cocoa.

The probe joins events by the visible version and records native handler
entry, first matching CPU draw start and end, drawable acquisition, command
buffer commit, Metal GPU end, GPU completion-callback arrival,
ScreenCaptureKit `displayTime`, and Metal `presentedTime`. Mach ticks use the
recorded timebase. The AppKit NSTextView control exposes its barcode through
ScreenCaptureKit. Its Core Animation presentation does not expose an
`MTLDrawable`, so its Metal clock and Metal stages are unavailable.

For matched versions, in milliseconds:

- Input delivery = native handler entry − `NSEvent.timestamp`.
- P = Metal GPU end − native handler entry. Completion callback lag is
  recorded separately.
- D estimate = smallest nonnegative GPU-end → WindowServer
  `displayTime` margin observed within the capture.
- Wait = GPU-end → WindowServer `displayTime` − D estimate.
- S = `presentedTime` − WindowServer `displayTime`.

Handler → WindowServer = P + D + Wait. Handler → Metal = P + D + Wait + S.
Add input delivery to obtain native timestamp → display on each clock.
Check the identities per version. Independently computed stage medians need
not add up to the median total. D describes the observed lower envelope.
It cannot identify an internal WindowServer deadline by itself. Keep signed
clock differences and explicitly report version match coverage.

The initial 60-second direct-launch diagnostics joined both clocks for 599
of 600 bare GPUI typing versions, 576 of 600 RustUI typing versions, and
6,056 of 6,124 RustUI hover versions. AppKit captured 599 of 599 on the SCK
clock. The paired captures below use the same external ScreenCaptureKit
observer for both launch routes. The direct diagnostics used in-process
capture and are kept separate from the paired comparison.

## Matched launch comparison

The same executable inside each bundle ran directly and through
`open -n -W App.app`. A separate, locally authorized companion decoded the
same ScreenCaptureKit window barcode in both routes and supplied the
synthetic test keys. This gave the benchmark apps identical diagnostics
without altering their macOS privacy settings. AppKit's comparison uses the
common ScreenCaptureKit endpoint. The Metal clock is available for the two
GPUI apps. The fixed comparison period is the measured passive-link interval,
approximately 8.33 ms. Per-route columns pool versions. Delta columns report
the median of the paired capture differences, LaunchServices minus direct.

| Workload | Pairs | SCK median / p95 Δ (ms) | Metal median / p95 Δ (ms) | CPU s/s paired changes | P verdict |
| --- | ---: | ---: | ---: | --- | --- |
| RustUI typing 10/s | 5 | -0.09 / +0.57 | -0.02 / -0.01 | +14.64% / +12.82% / +21.25% / +15.96% / +13.89% | REGRESSION |
| Bare GPUI Fast typing 10/s | 5 | +0.39 / +0.30 | -0.01 / -0.05 | +3.10% / -5.26% / +11.47% / +0.16% / +1.99% | INCONCLUSIVE |
| AppKit NSTextView typing 10/s | 5 | +0.12 / +0.99 | — | +39.65% / +7.37% / +4.44% / +19.85% / +28.95% | REGRESSION |
| RustUI hover | 5 | -0.17 / -0.05 | -0.01 / -0.01 | +21.22% / +35.64% / +23.34% / +26.35% / +25.91% | REGRESSION |

| Workload and route | Native timestamp → SCK | Native timestamp → Metal | Handler → SCK | Handler → Metal | Median CPU s/s | SCK versions / delivered |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| RustUI typing 10/s, direct | 17.46 / 28.06 / 37.57 | 28.39 / 39.41 / 49.18 | 12.78 / 21.23 / 29.41 | 24.01 / 32.32 / 40.00 | 0.08903 | 2,892 / 3,000 |
| RustUI typing 10/s, LaunchServices | 19.09 / 28.74 / 45.38 | 30.35 / 40.11 / 56.72 | 12.72 / 21.77 / 34.00 | 24.00 / 32.33 / 46.46 | 0.10668 | 2,884 / 3,000 |
| Bare GPUI Fast typing 10/s, direct | 15.97 / 21.09 / 31.72 | 27.52 / 32.12 / 39.79 | 12.28 / 14.50 / 25.47 | 23.93 / 24.07 / 32.42 | 0.06040 | 3,000 / 3,000 |
| Bare GPUI Fast typing 10/s, LaunchServices | 16.39 / 20.16 / 38.81 | 27.80 / 32.01 / 47.90 | 12.52 / 14.83 / 26.82 | 23.94 / 24.02 / 35.91 | 0.05940 | 3,000 / 3,000 |
| AppKit NSTextView typing 10/s, direct | 14.25 / 19.18 / 48.87 | — | 9.49 / 13.70 / 41.09 | — | 0.05650 | 2,995 / 2,995 |
| AppKit NSTextView typing 10/s, LaunchServices | 14.99 / 20.15 / 60.05 | — | 11.31 / 14.86 / 44.59 | — | 0.06772 | 2,995 / 2,995 |
| RustUI hover, direct | 20.72 / 25.71 / 46.36 | 29.83 / 35.34 / 53.88 | 15.86 / 23.55 / 41.53 | 24.06 / 32.41 / 49.08 | 0.35966 | 29,798 / 30,269 |
| RustUI hover, LaunchServices | 20.43 / 25.86 / 38.22 | 30.21 / 35.36 / 48.77 | 15.67 / 23.44 / 33.93 | 24.06 / 32.41 / 40.77 | 0.45245 | 28,545 / 28,913 |

All latency triples above are median / p95 / max in ms.

| Workload and route | Input delivery | P | D estimate | Wait | S | Handler → SCK | Handler → Metal |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| RustUI typing 10/s, direct | 4.00 / 8.35 | 5.78 / 8.20 | 4.15 / 5.08 | 3.12 / 9.17 | 11.37 / 12.90 | 12.78 / 21.23 | 24.01 / 32.32 |
| RustUI typing 10/s, LaunchServices | 4.22 / 8.38 | 6.14 / 8.35 | 4.14 / 5.02 | 2.61 / 9.65 | 11.59 / 12.99 | 12.72 / 21.77 | 24.00 / 32.33 |
| Bare GPUI Fast typing 10/s, direct | 3.58 / 8.17 | 2.86 / 4.83 | 6.45 / 6.71 | 3.05 / 6.02 | 11.64 / 12.86 | 12.28 / 14.50 | 23.93 / 24.07 |
| Bare GPUI Fast typing 10/s, LaunchServices | 3.85 / 7.97 | 3.04 / 5.93 | 6.53 / 6.89 | 2.70 / 4.96 | 11.41 / 12.91 | 12.52 / 14.83 | 23.94 / 24.02 |
| AppKit NSTextView typing 10/s, direct | 3.13 / 8.61 | — | — | — | — | 9.49 / 13.70 | — |
| AppKit NSTextView typing 10/s, LaunchServices | 3.31 / 6.91 | — | — | — | — | 11.31 / 14.86 | — |
| RustUI hover, direct | 3.47 / 8.15 | 4.92 / 8.32 | 3.79 / 4.22 | 7.95 / 13.27 | 9.10 / 12.09 | 15.86 / 23.55 | 24.06 / 32.41 |
| RustUI hover, LaunchServices | 2.97 / 8.15 | 5.49 / 8.61 | 4.13 / 4.21 | 6.77 / 12.83 | 9.34 / 12.02 | 15.67 / 23.44 | 24.06 / 32.41 |

Table uses median / p95 in ms, on versions with both clocks for GPUI. The
D column pools each capture's lower-envelope value over its matched
versions. There were 70,119 paired-clock version matches, zero negative
GPU-end → SCK margins, and zero formula residual at the stored precision.
No new standalone control floor was measured for this setup. Missed
refreshes beyond that floor were not measured.

| GPUI workload and route | Handler → draw start | CPU draw duration | Drawable acquired → commit | GPU end → callback | GPU end → Metal | Metal versions shown/s |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| RustUI typing 10/s, direct | 2.63 / 5.45 | 1.07 / 1.87 | 0.12 / 0.28 | 0.23 / 0.34 | 18.73 / 24.74 | 9.64 |
| RustUI typing 10/s, LaunchServices | 2.83 / 5.25 | 1.54 / 2.12 | 0.18 / 0.32 | 0.22 / 0.34 | 18.38 / 24.96 | 9.61 |
| Bare GPUI Fast typing 10/s, direct | 1.13 / 3.49 | 0.19 / 0.38 | 0.20 / 0.40 | 0.33 / 0.50 | 21.06 / 22.65 | 10.00 |
| Bare GPUI Fast typing 10/s, LaunchServices | 1.15 / 4.89 | 0.19 / 0.39 | 0.19 / 0.34 | 0.34 / 0.57 | 20.88 / 22.05 | 10.00 |
| RustUI hover, direct | 2.43 / 5.35 | 1.25 / 3.05 | 0.09 / 0.33 | 0.28 / 0.44 | 20.44 / 26.80 | 99.80 |
| RustUI hover, LaunchServices | 2.17 / 4.48 | 2.43 / 3.60 | 0.17 / 0.43 | 0.29 / 0.54 | 19.55 / 26.53 | 95.58 |

## ProMotion passive-link comparison

Bare GPUI Fast ran directly, with ScreenCaptureKit disabled. Compare the
passive CADisplayLink off/on and leave the render path switched to G5-F2.
The sparse target is 60 edits, one every 2 seconds, in a 120-second window.
Use the Metal clock in both arms. Drawable gaps measure the gap between two
observed presentations. With the passive link off, the physical refresh
interval between draws cannot be observed by this harness.

| Workload | Passive link | Captures | Handler → Metal | Native timestamp → Metal | Drawable gaps | Observed passive cadence | Render-link recreations / acquisitions while attached |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| typing10 | off | 5 | 23.54 / 27.30 / 32.33 | 24.44 / 28.69 / 37.63 | 100.00 / 108.33 | — | 0 / 0 |
| typing10 | on | 5 | 23.64 / 23.96 / 32.33 | 28.02 / 32.40 / 41.15 | 100.00 / 100.00 | 8.33 / 8.33 | 0 / 0 |
| sparse | off | 5 | — | — | — / — | — | 0 / 0 |
| sparse | on | 5 | — | — | — / — | 8.33 / 8.33 | 0 / 0 |

Latency columns show median / p95 / max. Gap and cadence columns show the
median across the captures' median and p95 values. Times are in ms.

| Workload | Metal median / p95 Δ on−off (ms) | CPU s/s paired changes | P verdict | Edit counts per arm |
| --- | ---: | --- | --- | --- |
| typing10 | +0.74 / -2.56 | +48.13% / +49.32% / +122.59% / +29.75% / +22.30% | REGRESSION | 600, 600, 600, 600, 600 / 600, 600, 600, 600, 600 |
| sparse | — / — | +387.57% / +479.58% / +539.44% / +428.13% / +409.44% | REGRESSION | 60, 60, 60, 60, 60 / 60, 60, 60, 60, 60 |

## Capture conditions and acquisition diagnostics

An asterisk means measured on a loaded machine. Each number is corrected
background CPU at start → after, in percent. Limit 10%. Capture logs retain
the top processes.

| Direct / LaunchServices | Pair 1 | Pair 2 | Pair 3 | Pair 4 | Pair 5 |
| --- | --- | --- | --- | --- | --- |
| lab-typing | 4.80→14.62\* / 10.91→17.40\* | 9.01→29.19\* / 11.80→19.29\* | 32.10→17.40\* / 11.48→19.90\* | 13.93→11.01\* / 5.56→16.43\* | 7.99→20.38\* / 12.27→17.59\* |
| bare-typing | 10.97→11.94\* / 7.52→8.10 | 10.57→10.02\* / 7.77→9.69 | 6.53→10.01\* / 7.26→9.86 | 9.20→16.86\* / 9.64→15.36\* | 11.25→15.40\* / 5.26→15.47\* |
| appkit-typing | 8.63→8.86 / 11.12→9.37\* | 3.40→8.63 / 10.29→12.15\* | 12.22→11.71\* / 9.74→10.97\* | 12.16→7.58\* / 8.67→9.89 | 12.13→10.50\* / 7.83→10.11\* |
| lab-hover | 11.16→16.33\* / 10.09→18.87\* | 12.10→16.26\* / 11.67→13.68\* | 6.51→13.02\* / 15.29→10.48\* | 21.03→26.83\* / 17.96→38.65\* | 23.40→16.71\* / 15.97→10.47\* |

| Passive off / on | Pair 1 | Pair 2 | Pair 3 | Pair 4 | Pair 5 |
| --- | --- | --- | --- | --- | --- |
| typing10 | 15.75→11.32\* / 10.20→4.11\* | 10.19→55.51\* / 11.63→15.50\* | 47.75→20.48\* / 20.13→5.32\* | 16.90→6.60\* / 11.68→4.45\* | 11.12→12.42\* / 21.64→4.13\* |
| sparse | 13.03→8.80\* / 11.23→6.59\* | 15.91→5.06\* / 10.09→3.56\* | 6.09→3.69 / 6.69→8.30 | 4.99→5.19 / 10.96→5.87\* | 9.66→5.34 / 5.41→4.76 |

| GPUI workload and route | Measured >1 ms / >2 ms | Warmup >1 ms | Drain >1 ms | Out of order | Lost edits | Versions without a later valid Metal timestamp |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| lab-typing, direct | 0 / 0 | 3 | 0 | 0 | 0 | 0 |
| lab-typing, LaunchServices | 0 / 0 | 4 | 0 | 0 | 0 | 0 |
| bare-typing, direct | 0 / 0 | 6 | 0 | 0 | 0 | 0 |
| bare-typing, LaunchServices | 0 / 0 | 3 | 0 | 0 | 0 | 0 |
| lab-hover, direct | 4 / 2 | 4 | 0 | 0 | — | 0 |
| lab-hover, LaunchServices | 31 / 6 | 1 | 0 | 0 | — | 0 |
| typing10, passive off | 0 / 0 | 2 | 0 | 0 | 0 | 0 |
| typing10, passive on | 0 / 0 | 4 | 0 | 0 | 0 | 0 |
| sparse, passive off | 0 / 0 | 5 | 0 | — | 0 | 300 |
| sparse, passive on | 0 / 0 | 6 | 0 | — | 0 | 298 |

Of the GPUI command buffers committed in measured windows, each produced
a presentation callback. The table separates missing callbacks from
callbacks with a zero or invalid Metal timestamp. A callback alone cannot
prove the frame reached the display.

| Workload and route | Committed drawables | No callback | Callback without valid `presentedTime` |
| --- | ---: | ---: | ---: |
| Bare typing, direct | 3,005 | 0 | 5 |
| Bare typing, LaunchServices | 3,005 | 0 | 5 |
| RustUI typing, direct | 2,887 | 0 | 0 |
| RustUI typing, LaunchServices | 2,879 | 0 | 0 |
| RustUI hover, direct | 29,955 | 0 | 16 |
| RustUI hover, LaunchServices | 28,688 | 0 | 15 |
| Passive off, 10/s | 3,014 | 0 | 9 |
| Passive on, 10/s | 3,005 | 0 | 10 |
| Passive off, sparse | 305 | 0 | 305 |
| Passive on, sparse | 305 | 0 | 303 |

All logged acquisition calls over 1 ms:

| Workload / route / pair | Phase | Call ms | Path | In flight | Time since detach (ms) |
| --- | --- | ---: | --- | ---: | ---: |
| lab-typing / direct / 1 | warmup | 1.141 | non-immediate | 0 | — |
| lab-typing / direct / 3 | warmup | 1.240 | non-immediate | 0 | — |
| lab-typing / direct / 4 | warmup | 1.170 | non-immediate | 0 | — |
| lab-typing / LaunchServices / 1 | warmup | 1.115 | non-immediate | 0 | — |
| lab-typing / LaunchServices / 2 | warmup | 1.638 | non-immediate | 0 | — |
| lab-typing / LaunchServices / 3 | warmup | 1.232 | non-immediate | 0 | — |
| lab-typing / LaunchServices / 5 | warmup | 1.131 | non-immediate | 0 | — |
| bare-typing / direct / 2 | warmup | 1.082 | non-immediate | 0 | — |
| bare-typing / direct / 2 | warmup | 1.255 | immediate | 0 | 0.09 |
| bare-typing / direct / 3 | warmup | 1.081 | non-immediate | 0 | — |
| bare-typing / direct / 4 | warmup | 2.775 | non-immediate | 0 | — |
| bare-typing / direct / 5 | warmup | 1.109 | non-immediate | 0 | — |
| bare-typing / direct / 5 | warmup | 1.828 | immediate | 0 | 0.09 |
| bare-typing / LaunchServices / 1 | warmup | 1.002 | non-immediate | 0 | — |
| bare-typing / LaunchServices / 2 | warmup | 1.106 | non-immediate | 0 | — |
| bare-typing / LaunchServices / 4 | warmup | 1.021 | non-immediate | 0 | — |
| lab-hover / direct / 2 | warmup | 1.190 | non-immediate | 0 | — |
| lab-hover / direct / 2 | measured | 4.215 | immediate | 2 | 9594.11 |
| lab-hover / direct / 2 | measured | 13.692 | immediate | 2 | 13068.03 |
| lab-hover / direct / 3 | warmup | 1.071 | non-immediate | 0 | — |
| lab-hover / direct / 3 | measured | 1.173 | immediate | 2 | 2392.86 |
| lab-hover / direct / 4 | warmup | 1.029 | non-immediate | 0 | — |
| lab-hover / direct / 4 | measured | 1.819 | immediate | 2 | 3107.40 |
| lab-hover / direct / 5 | warmup | 1.115 | non-immediate | 0 | — |
| lab-hover / LaunchServices / 1 | measured | 1.336 | immediate | 1 | 4976.55 |
| lab-hover / LaunchServices / 1 | measured | 1.020 | immediate | 2 | 8926.40 |
| lab-hover / LaunchServices / 1 | measured | 1.289 | immediate | 2 | 11118.10 |
| lab-hover / LaunchServices / 1 | measured | 1.016 | immediate | 2 | 12978.44 |
| lab-hover / LaunchServices / 1 | measured | 1.042 | immediate | 2 | 20711.54 |
| lab-hover / LaunchServices / 1 | measured | 1.227 | immediate | 2 | 22343.89 |
| lab-hover / LaunchServices / 1 | measured | 1.668 | immediate | 2 | 35386.85 |
| lab-hover / LaunchServices / 1 | measured | 6.083 | immediate | 1 | 37409.43 |
| lab-hover / LaunchServices / 1 | measured | 1.093 | immediate | 2 | 45311.51 |
| lab-hover / LaunchServices / 1 | measured | 1.007 | immediate | 2 | 50511.42 |
| lab-hover / LaunchServices / 2 | warmup | 1.056 | non-immediate | 0 | — |
| lab-hover / LaunchServices / 2 | measured | 2.023 | immediate | 2 | 15132.28 |
| lab-hover / LaunchServices / 2 | measured | 1.044 | immediate | 2 | 22347.01 |
| lab-hover / LaunchServices / 2 | measured | 1.308 | immediate | 1 | 28646.98 |
| lab-hover / LaunchServices / 2 | measured | 1.006 | immediate | 2 | 52731.92 |
| lab-hover / LaunchServices / 2 | measured | 1.210 | immediate | 2 | 62171.60 |
| lab-hover / LaunchServices / 3 | measured | 1.246 | immediate | 2 | — |
| lab-hover / LaunchServices / 3 | measured | 2.881 | immediate | 2 | — |
| lab-hover / LaunchServices / 3 | measured | 1.128 | immediate | 2 | — |
| lab-hover / LaunchServices / 4 | measured | 1.159 | immediate | 2 | — |
| lab-hover / LaunchServices / 4 | measured | 1.454 | immediate | 1 | — |
| lab-hover / LaunchServices / 4 | measured | 1.703 | immediate | 2 | — |
| lab-hover / LaunchServices / 4 | measured | 4.296 | immediate | 1 | — |
| lab-hover / LaunchServices / 4 | measured | 1.001 | immediate | 2 | — |
| lab-hover / LaunchServices / 5 | measured | 1.059 | immediate | 1 | 2340.63 |
| lab-hover / LaunchServices / 5 | measured | 4.260 | immediate | 2 | 2347.84 |
| lab-hover / LaunchServices / 5 | measured | 5.839 | immediate | 1 | 8106.48 |
| lab-hover / LaunchServices / 5 | measured | 1.211 | immediate | 2 | 11008.25 |
| lab-hover / LaunchServices / 5 | measured | 1.410 | immediate | 2 | 19742.17 |
| lab-hover / LaunchServices / 5 | measured | 1.276 | immediate | 1 | 25264.83 |
| lab-hover / LaunchServices / 5 | measured | 1.026 | immediate | 2 | 27142.21 |
| lab-hover / LaunchServices / 5 | measured | 1.102 | immediate | 2 | 35614.23 |
| typing10 / passive off / 2 | warmup | 1.169 | immediate | 0 | 0.10 |
| typing10 / passive off / 5 | warmup | 1.030 | non-immediate | 0 | — |
| typing10 / passive on / 1 | warmup | 1.006 | non-immediate | 0 | — |
| typing10 / passive on / 3 | warmup | 2.269 | immediate | 0 | 0.10 |
| typing10 / passive on / 4 | warmup | 1.234 | non-immediate | 0 | — |
| typing10 / passive on / 5 | warmup | 2.281 | non-immediate | 0 | — |
| sparse / passive off / 1 | warmup | 1.122 | non-immediate | 0 | — |
| sparse / passive off / 2 | warmup | 1.047 | non-immediate | 0 | — |
| sparse / passive off / 4 | warmup | 1.073 | non-immediate | 0 | — |
| sparse / passive off / 4 | warmup | 1.153 | immediate | 0 | 0.10 |
| sparse / passive off / 5 | warmup | 1.112 | non-immediate | 0 | — |
| sparse / passive on / 1 | warmup | 1.088 | non-immediate | 0 | — |
| sparse / passive on / 2 | warmup | 1.030 | non-immediate | 0 | — |
| sparse / passive on / 3 | warmup | 1.195 | non-immediate | 0 | — |
| sparse / passive on / 4 | warmup | 1.068 | non-immediate | 0 | — |
| sparse / passive on / 5 | warmup | 1.042 | non-immediate | 0 | — |
| sparse / passive on / 5 | warmup | 2.082 | immediate | 0 | 0.09 |

## Findings

- Across five pairs per workload, neither launch route added a refresh to
  median or p95 latency on either available clock. The largest median paired p95
  increase was 0.99 ms for AppKit on the SCK clock. The median matched
  passive interval was 8.33 ms. The key-window process-activity experiment
  was conditional on a full-refresh launch gap, so it was not run. The
  [Apple Developer Forums observation](https://developer.apple.com/forums/thread/841189)
  is a separate test on a different setup.
- The shared SCK clock puts direct-launch AppKit at 9.49 ms median after
  handler entry, bare GPUI Fast at 12.28 ms, and RustUI typing at 12.78 ms.
  The Metal timestamp for bare GPUI is typically another 11.64 ms later
  than SCK, and for RustUI typing it is 11.37 ms later. Comparisons to
  AppKit use SCK for all three apps.
- Most of the direct RustUI typing median remains after GPU completion.
  GPU end → Metal was 18.73 ms median, compared with 5.78 ms for handler →
  GPU end on samples with both clocks. RustUI hover also has a sizable
  and more variable WindowServer wait estimate, 7.95 ms median and
  13.27 ms p95. These observations separate the GPU work from subsequent
  frame timing. They do not establish a calibrated count of missed
  physical refreshes for this capture setup.
- At 10 keys/s, removing the passive link increased handler → Metal p95
  by a median 2.56 ms over five pairs. The paired median was 0.74 ms
  faster with the passive link off, and native timestamp → Metal was
  3.58 ms faster in the pooled median. Passive observation raised
  measured process CPU in all pairs. Because it changes the workload,
  keep its state explicit in comparisons.
- At one key per two seconds, all ten captures delivered and applied all
  60 edits and observed zero rendering-link recreations or acquisitions
  while attached. Metal returned `presentedTime == 0` for 300 of 300
  tagged edits with the passive link off and 298 of 300 with it on. The
  two positive samples were in separate passive-on runs. These captures
  do not support a latency or physical refresh comparison at the sparse
  rate. The passive-on callback clock still reported 8.33 ms.
- Pointer replay was sampled once per wake. LaunchServices delivered
  fewer RustUI hover updates in the same 60 seconds, so the hover CPU
  increase does not come from processing more delivered inputs.
  Background load varied between and within arms. CPU verdicts describe
  these captures and do not isolate the cause of the differences.

Physical-key captures remain outstanding. All typing numbers in this report
are from posted events delivered through the native Cocoa path. No callback
timestamp was substituted for a missing display timestamp.
