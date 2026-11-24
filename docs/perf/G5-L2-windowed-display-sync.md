# G5-L2 windowed display sync

The control is the merged G5-L1 baseline at `35be237d4`. G5-L3 remains a separate draft
after missing its latency targets. This experiment changes one layer
property, with `GPUI_FAST_NO_DISPLAY_SYNC=1` and the existing immediate
and coalescing switches enabled. Unset leaves the existing default.

The opt-in setting applies to the base and every GPUI composition
surface. Native and simple fullscreen, AppKit fullscreen transitions,
and synchronous transaction presents stay synchronized. The scheduling
and drawable order/capacity guards stay active. Apple documents that an
unsynchronized layer can show tearing.
[CAMetalLayer display sync](https://developer.apple.com/documentation/quartzcore/cametallayer/displaysyncenabled)

Measure on one release build with native Cocoa events, direct executable
launches, damage off, buffered logging, and the passive display link off
for the paired workloads. Use five alternating 60-second off/on pairs
after warmup. AppKit reference captures run in the same session using the
same external ScreenCaptureKit observer. AppKit does not expose an
`MTLDrawable.presentedTime` for its NSTextView. ScreenCaptureKit provides
the WindowServer display time.
[ScreenCaptureKit displayTime](https://developer.apple.com/documentation/screencapturekit/scstreamframeinfo/displaytime)

The moving vertical bar proof checks for a change in the bar's left or
right edge between scan rows in complete WindowServer capture frames.
It also records missing bars and observed capture cadence. Physical
panel scanout remains outside the capture method.
[ScreenCaptureKit frame status](https://developer.apple.com/documentation/screencapturekit/capturing-screen-content-in-macos)

## Moving-bar proof

The bar runs directly from the bare GPUI reference on the native Cocoa
pointer path. The passive observer was explicitly on for this proof.
ScreenCaptureKit requested up to 120 frames/s and the live proof checked
all 1,000 rows in the bar's interior-height band on each complete frame.
A synthetic input to the same detector confirmed that it accepts an
aligned bar and detects a deliberately shifted lower half.

| Arm | Complete frames checked | Missing bars | Split frames | Distinct x coordinates | SCK gap median | Passive tick median / p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Sync on | 6,327 | 0 | 0 | 900 | 9.01 ms | 8.33 / 8.33 ms |
| Sync off | 6,186 | 0 | 0 | 900 | 9.12 ms | 8.33 / 8.33 ms |

Both 60-second captures were measured on a loaded machine. Background
CPU was 17.78% → 19.88% with sync on and 17.60% → 18.58% with sync off.
The complete captures arrived below 120/s. The display-link observation
and capture delivery rates are reported separately.

For the owner visual check, watch the moving cyan bar on the physical
panel in a window with the switch off and on. Look for a horizontal
break in the bar during steady movement or the jump back across the
window. The ScreenCaptureKit proof covers WindowServer frames, and the
physical visual check remains pending.
## Matched latency and controls

| Workload | Arm | Captures | Handler → SCK | Handler → Metal | Native timestamp → SCK | Native timestamp → Metal |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| RustUI typing 10/s | Sync on | 5 | 13.85 / 20.77 / 39.65 | 24.26 / 32.20 / 40.39 | 15.13 / 22.48 / 40.28 | 26.90 / 33.73 / 44.22 |
| RustUI typing 10/s | Sync off | 5 | 8.88 / 19.19 / 34.90 | 8.82 / 31.46 / 46.50 | 9.90 / 20.79 / 36.25 | 9.89 / 32.23 / 47.62 |
| RustUI typing 30/s | Sync on | 5 | 13.83 / 20.81 / 29.13 | 24.22 / 30.83 / 40.78 | 15.08 / 22.52 / 37.36 | 26.39 / 32.98 / 48.90 |
| RustUI typing 30/s | Sync off | 5 | 8.49 / 16.10 / 32.20 | 6.79 / 21.56 / 44.97 | 9.41 / 17.95 / 48.02 | 7.77 / 22.67 / 57.07 |
| Bare GPUI Fast typing 10/s | Sync on | 5 | 11.41 / 15.51 / 28.52 | 22.98 / 26.78 / 35.16 | 12.34 / 16.74 / 31.15 | 23.65 / 27.58 / 43.14 |
| Bare GPUI Fast typing 10/s | Sync off | 5 | 7.14 / 15.33 / 28.63 | 6.19 / 24.31 / 40.26 | 8.24 / 16.80 / 30.89 | 7.28 / 26.73 / 40.92 |
| Bare GPUI Fast typing 30/s | Sync on | 5 | 11.67 / 16.97 / 38.60 | 23.00 / 27.51 / 40.72 | 12.66 / 17.99 / 40.19 | 23.82 / 28.37 / 43.87 |
| Bare GPUI Fast typing 30/s | Sync off | 5 | 5.62 / 13.20 / 30.00 | 4.07 / 11.35 / 39.13 | 6.94 / 14.68 / 52.88 | 5.31 / 12.99 / 62.01 |
| AppKit NSTextView typing 10/s | AppKit control | 5 | 9.24 / 14.16 / 34.64 | — | 11.58 / 15.95 / 40.84 | — |
| AppKit NSTextView typing 30/s | AppKit control | 5 | 8.28 / 13.95 / 33.38 | — | 9.64 / 15.42 / 41.89 | — |
| RustUI hover | Sync on | 5 | 15.28 / 21.01 / 32.42 | 24.22 / 30.92 / 44.62 | 15.82 / 21.67 / 33.36 | 25.15 / 31.32 / 45.56 |
| RustUI hover | Sync off | 5 | 9.43 / 14.80 / 33.07 | 7.00 / 13.13 / 45.11 | 9.78 / 15.13 / 33.22 | 7.29 / 13.45 / 45.27 |
| RustUI drag | Sync on | 5 | 14.90 / 21.07 / 34.66 | 24.13 / 30.37 / 45.86 | 15.40 / 21.43 / 36.54 | 24.63 / 30.56 / 46.04 |
| RustUI drag | Sync off | 5 | 8.65 / 13.69 / 31.26 | 6.14 / 11.70 / 38.33 | 9.02 / 13.96 / 31.48 | 6.39 / 12.24 / 38.41 |

All latency cells show median / p95 / max in ms.

| Workload | Arm | P | D lower envelope | Wait | S | GPU end → Metal |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| RustUI typing 10/s | Sync on | 4.97 / 7.65 | 3.88 / 3.88 | 4.86 / 10.11 | 11.82 / 13.05 | 20.05 / 24.76 |
| RustUI typing 10/s | Sync off | 4.79 / 7.17 | 1.52 / 1.55 | 2.84 / 12.23 | -0.94 / 13.70 | 4.04 / 25.17 |
| RustUI typing 30/s | Sync on | 4.49 / 7.50 | 3.39 / 3.62 | 5.70 / 10.99 | 11.41 / 12.78 | 20.32 / 24.70 |
| RustUI typing 30/s | Sync off | 4.71 / 7.08 | 1.63 / 1.65 | 2.20 / 9.45 | -1.55 / 7.81 | 2.17 / 17.18 |
| Bare GPUI Fast typing 10/s | Sync on | 2.28 / 4.39 | 4.29 / 4.87 | 4.22 / 9.29 | 11.71 / 12.85 | 20.50 / 24.46 |
| Bare GPUI Fast typing 10/s | Sync off | 2.20 / 3.33 | 2.32 / 2.41 | 2.39 / 10.58 | -0.97 / 12.10 | 3.94 / 22.66 |
| Bare GPUI Fast typing 30/s | Sync on | 2.34 / 3.55 | 3.87 / 4.30 | 5.27 / 10.62 | 11.58 / 12.65 | 20.18 / 24.99 |
| Bare GPUI Fast typing 30/s | Sync off | 2.25 / 3.08 | 2.31 / 2.39 | 0.81 / 8.35 | -1.52 / -0.85 | 1.43 / 9.48 |
| RustUI hover | Sync on | 3.17 / 7.97 | 3.95 / 4.12 | 7.99 / 12.37 | 9.46 / 12.01 | 21.21 / 26.15 |
| RustUI hover | Sync off | 3.52 / 4.87 | 1.72 / 1.78 | 4.57 / 9.69 | -2.40 / 3.59 | 3.89 / 9.98 |
| RustUI drag | Sync on | 2.45 / 8.58 | 3.87 / 4.13 | 8.50 / 12.29 | 9.12 / 12.12 | 21.36 / 25.83 |
| RustUI drag | Sync off | 1.96 / 3.30 | 1.52 / 1.55 | 5.27 / 10.01 | -2.71 / 3.35 | 4.20 / 9.59 |

D uses each capture’s minimum nonnegative GPU-end-to-SCK margin. S is signed, Metal minus SCK. Values are median / p95 ms.

| Workload | CPU s/s control → candidate (medians) | Five paired CPU changes | P verdict | Kernel task CPU control → candidate |
| --- | ---: | --- | --- | ---: |
| RustUI typing 10/s | 0.08202 → 0.06934 | -13.81% / -14.42% / -16.52% / -18.81% / -12.70% | GAIN | 11.95 / 13.10 → 43.90 / 45.40 |
| RustUI typing 30/s | 0.21569 → 0.20478 | +14.15% / -6.11% / -4.65% / -4.54% / -6.15% | INCONCLUSIVE | 13.15 / 13.80 → 50.20 / 51.80 |
| Bare GPUI Fast typing 10/s | 0.04040 → 0.03724 | -3.63% / -27.85% / +3.93% / -13.31% / -12.20% | INCONCLUSIVE | 12.30 / 13.60 → 46.75 / 47.60 |
| Bare GPUI Fast typing 30/s | 0.12531 → 0.11598 | -20.09% / -6.58% / +7.24% / +16.46% / -12.24% | INCONCLUSIVE | 12.20 / 13.80 → 51.65 / 53.70 |
| RustUI hover | 0.35768 → 0.34551 | -1.63% / -7.49% / -3.40% / -2.87% / -3.96% | INCONCLUSIVE | 20.10 / 22.00 → 84.30 / 88.80 |
| RustUI drag | 0.27831 → 0.23681 | -6.90% / -15.73% / -14.40% / -17.57% / -17.83% | GAIN | 20.90 / 22.20 → 84.90 / 86.20 |

Kernel CPU is median / p95 percent of one logical CPU from the `top` sampling intervals wholly inside the timed windows.

| Workload | Arm | SCK versions / delivered | SCK versions/s | Metal versions/s | SCK gaps median / p95 ms | Committed / no callback / invalid Metal time | Out of order SCK / Metal | Lost keys | Measured acquisition >1 / >2 ms |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| RustUI typing 10/s | Sync on | 2,925 / 3,000 | 9.75 | 9.75 | 102.13 / 111.36 | 2,920 / 0 / 0 | 0 / 0 | 0 | 0 / 0 |
| RustUI typing 10/s | Sync off | 2,935 / 3,000 | 9.78 | 9.78 | 104.06 / 116.55 | 2,930 / 0 / 0 | 0 / 0 | 0 | 0 / 0 |
| RustUI typing 30/s | Sync on | 8,804 / 9,000 | 29.34 | 29.34 | 33.47 / 43.31 | 8,799 / 0 / 0 | 0 / 0 | 0 | 2 / 0 |
| RustUI typing 30/s | Sync off | 8,748 / 9,000 | 29.15 | 29.17 | 34.66 / 42.76 | 8,749 / 0 / 0 | 0 / 0 | 0 | 0 / 0 |
| Bare GPUI Fast typing 10/s | Sync on | 3,000 / 3,000 | 10.00 | 10.00 | 100.11 / 108.95 | 3,005 / 0 / 5 | 0 / 0 | 0 | 0 / 0 |
| Bare GPUI Fast typing 10/s | Sync off | 3,000 / 3,000 | 10.00 | 10.00 | 100.09 / 109.04 | 3,005 / 0 / 5 | 0 / 0 | 0 | 4 / 0 |
| Bare GPUI Fast typing 30/s | Sync on | 8,991 / 9,000 | 29.97 | 29.97 | 33.36 / 42.33 | 8,997 / 0 / 5 | 0 / 0 | 0 | 0 / 0 |
| Bare GPUI Fast typing 30/s | Sync off | 8,994 / 9,000 | 29.98 | 29.98 | 33.34 / 40.47 | 8,999 / 0 / 5 | 0 / 0 | 0 | 3 / 0 |
| RustUI hover | Sync on | 29,695 / 30,110 | 98.97 | 99.20 | 9.21 / 15.25 | 29,773 / 0 / 15 | 0 / 0 | — | 0 / 0 |
| RustUI hover | Sync off | 28,845 / 30,007 | 96.15 | 98.83 | 9.28 / 17.78 | 29,919 / 0 / 274 | 0 / 0 | — | 6 / 1 |
| RustUI drag | Sync on | 30,554 / 30,939 | 101.83 | 102.46 | 9.20 / 14.32 | 30,738 / 0 / 0 | 0 / 0 | — | 5 / 0 |
| RustUI drag | Sync off | 29,396 / 30,798 | 97.98 | 101.96 | 9.28 / 17.39 | 30,751 / 0 / 164 | 0 / 0 | — | 3 / 1 |

SCK tracks its own first observation of each delivered version. Drawable callbacks without a valid timestamp are accounted for separately. Reference barcode and drawable counts can differ. The full acquisition list records path, in-flight count and detach age in `target/headstart/g5-l2/drawable-outliers.csv`.

## Capture load

Correction uses background CPU at start → after in percent. An asterisk marks a loaded-machine capture. GPUI entries show sync on / off. AppKit entries show its single reference capture per pair.

| Workload | Pair 1 | Pair 2 | Pair 3 | Pair 4 | Pair 5 |
| --- | --- | --- | --- | --- | --- |
| RustUI typing 10/s | 17.83→7.77\* / 19.04→24.25\* | 13.64→24.38\* / 15.88→22.15\* | 14.70→10.17\* / 16.68→9.79\* | 13.40→7.67\* / 14.52→17.70\* | 12.84→23.01\* / 15.51→9.00\* |
| RustUI typing 30/s | 12.80→19.00\* / 14.30→18.80\* | 16.06→17.71\* / 20.91→17.84\* | 12.93→18.25\* / 15.03→16.73\* | 15.54→17.15\* / 13.77→18.87\* | 13.23→15.62\* / 14.75→18.26\* |
| Bare GPUI Fast typing 10/s | 15.56→18.54\* / 20.04→12.22\* | 14.12→11.94\* / 14.36→17.92\* | 17.23→8.74\* / 14.77→11.71\* | 13.28→17.90\* / 20.46→11.10\* | 16.17→16.15\* / 13.38→17.94\* |
| Bare GPUI Fast typing 30/s | 17.64→17.77\* / 16.67→17.00\* | 15.26→18.85\* / 15.63→17.68\* | 13.92→17.88\* / 10.84→19.19\* | 17.55→11.40\* / 16.17→18.42\* | 18.24→10.43\* / 17.60→27.79\* |
| AppKit NSTextView typing 10/s | 13.13→8.58\* | 9.66→10.58\* | 13.53→10.24\* | 21.08→10.25\* | 9.83→11.75\* |
| AppKit NSTextView typing 30/s | 13.58→16.72\* | 16.84→11.72\* | 16.81→11.57\* | 18.03→12.43\* | 27.57→12.89\* |
| RustUI hover | 12.38→9.38\* / 16.81→19.23\* | 16.20→21.93\* / 13.34→11.64\* | 15.05→12.72\* / 18.32→24.32\* | 20.28→12.97\* / 16.63→17.61\* | 17.50→19.91\* / 13.19→15.12\* |
| RustUI drag | 15.73→12.60\* / 15.97→11.05\* | 16.24→13.38\* / 17.97→10.22\* | 16.87→13.21\* / 15.78→12.05\* | 15.72→13.78\* / 16.31→9.40\* | 15.82→19.91\* / 11.27→18.75\* |

## Outcomes

Sync off lowered the median on both clocks for all six GPUI workloads.
RustUI at 30 keys/s moved from 24.22 to 6.79 ms on Metal and from 13.83 to
8.49 ms on the shared ScreenCaptureKit clock. AppKit reached 8.28 ms median
on that clock. RustUI's shared-clock p95 remained higher than AppKit's
at both typing rates.

For RustUI typing at 30/s, median P was 4.49 → 4.71 ms. D shifted from
3.39 to 1.63 ms, and S from 11.41 to −1.55 ms. A negative S means the
recorded Metal timestamp preceded the captured WindowServer frame.
The clock-specific latency tables use their own observed endpoints.
The passive-link-off timing matrix measures content presentation gaps.
An independent physical refresh count and first-eligible-refresh floor
were unavailable in those windows.

Application CPU was GAIN for RustUI typing at 10/s and drag. The four
other workloads were INCONCLUSIVE. Median sampled `kernel_task` CPU
rose in every workload, including 20.10% → 84.30% of one core in hover
and 20.90% → 84.90% in drag. These are separate process observations.
Power was not measured here.

No edit was lost and neither recorded clock showed a version moving
backward. Every missing tagged GPUI version had a later valid Metal
version. All measured submitted drawables received a callback. Sync
off produced more callbacks without a usable Metal timestamp in hover
and drag. These frames have no separate Metal display observation.
ScreenCaptureKit versions/s fell from 98.97 to 96.15 in hover and from
101.83 to 97.98 in drag. The live bar proof observed aligned bar edges
in all complete compositor frames.

The acquisition log contains 23 measured calls over 1 ms and 42 warmup
calls over 1 ms. Two measured calls exceeded the 2 ms gate, both in
the experimental arm:

| Capture | Duration | Path | In flight / pool | Time since detach |
| --- | ---: | --- | ---: | ---: |
| RustUI hover, pair 2 | 4.196 ms | Immediate | 2 / 3 | 40,575.231 ms |
| RustUI drag, pair 3 | 2.187 ms | Immediate | 2 / 3 | 2,533.014 ms |

The windowed preflight recorded a disabled sync property during the
experimental run. The native-fullscreen typing preflight kept the
property enabled with the same switch. The candidate remains draft for
the owner's adoption decision and physical-panel visual check.
