# G5-L5 screen refresh and keep-warm

The built-in display was left in its existing configuration. This experiment
uses the same opt-in Cocoa helper for RustUI Lab, the bare GPUI reference, and
the AppKit NSTextView. It intercepts native input in the existing benchmark
shim. The production GPUI scheduler stays on the existing render path. With
`RUSTUI_G5_KEEP_WARM=4` or `250ms`, the helper wakes one view-bound
`CADisplayLink` and requests `min = max = preferred` at the current
screen’s `maximumFramesPerSecond`. The link never builds a scene or
presents a drawable. The four-refresh arm counts callbacks after the latest
input. The 250 ms arm re-arms its pause timer on every input. macOS <14
records `unsupported` and skips the keep-warm link. Unset keeps it off.

`RUSTUI_MAIN_LAG=1` adds a 4 ms CFRunLoop timer in common modes. It tracks
the expected fire time on the host clock, skips already elapsed fire times,
and buffers lateness over 2 ms. A stall overlaps a key when its
expected-to-actual timer interval intersects the native event timestamp
through the decoded ScreenCaptureKit update. This is a main-thread lag
indicator and does not assign the stall to a particular function.

## Timing method

Each matched block targets 60 seconds at 10 native Cocoa key events/s after
warmup, direct executable launch, damage off and the passive link off.
RustUI and bare GPUI use the temporary display-sync-disabled measurement
build. AppKit uses its ordinary NSTextView and the same native helper and
ScreenCaptureKit observer. A single off capture within each block serves both
comparisons. Off precedes both candidates in odd blocks and follows both
candidates in even blocks. The paired CPU verdict uses process CPU per
attempt for the GPUI arms, with CPU per second alongside it. AppKit has no
GPUI present attempt, so its verdict uses CPU per second.

P is native handler entry to Metal GPU end. The 8.333 ms miss threshold
retains the existing G5 definition. The per-key
CSV records consecutive ScreenCaptureKit sample intervals, NSScreen’s last
framebuffer update timestamp and maximum rate, and an active keep-warm
link’s callback interval where available. An unchanged window may produce
an idle sample or repeat a display timestamp. A gap in SCK samples is a
compositor observation. NSScreen timestamps describe framebuffer updates,
and the display-link range is a request. These observations do not
independently establish the physical panel’s instantaneous scanout rate.
SCK's WindowServer display time is the common endpoint. NSTextView exposes
no matching Metal GPU end, so its P and GPU-to-SCK miss rate are unavailable.

- [ScreenCaptureKit frame status](https://developer.apple.com/documentation/screencapturekit/scframestatus/idle)
- [NSScreen refresh information](https://developer.apple.com/documentation/appkit/nsscreen)
- [Apple display-link rate range](https://developer.apple.com/documentation/quartzcore/cadisplaylink/preferredframeraterange)

## Quiet-machine gate

Spotlight was sampled every 15 seconds using process CPU from
`ps` (100% means one logical core). The sum included `mds`, `mds_stores`
and `mds_worker`. The gate finished after 827.8 seconds.
The final sum was 0.0%. The longest sampled interval below 5%
was 120.4 seconds, against the required
120 seconds.

The first block cleared the Spotlight gate.

All 15 final pre-block checks met the 5% corrected background threshold
(3.51–4.82%). The paired checks exclude Codex and its process tree under
the existing measurement rules. Separate per-capture snapshots exceeded
5% on 15/45 starts
and 45/45 end samples.
The maximum end sample was 28.88%.
The quiet label describes the pre-block check. The load snapshots for each
arm are saved alongside its capture.

## Results

| Arm | Miss rate | Handler → SCK median / p95 | CPU verdict |
| --- | ---: | ---: | --- |
| RustUI 10,000-line editor, off | 33.31% | 8.78 / 19.43 ms | Control |
| RustUI 10,000-line editor, 4 refreshes | 32.63% | 8.80 / 19.25 ms | REGRESSION |
| RustUI 10,000-line editor, 250 ms | 29.50% | 10.43 / 24.79 ms | REGRESSION |
| Bare GPUI Fast, off | 36.17% | 7.70 / 16.46 ms | Control |
| Bare GPUI Fast, 4 refreshes | 31.70% | 7.89 / 16.50 ms | REGRESSION |
| Bare GPUI Fast, 250 ms | 46.00% | 10.77 / 20.37 ms | REGRESSION |
| AppKit NSTextView, off | Unavailable | 8.80 / 15.54 ms | Control |
| AppKit NSTextView, 4 refreshes | Unavailable | 8.62 / 14.92 ms | INCONCLUSIVE |
| AppKit NSTextView, 250 ms | Unavailable | 11.12 / 14.37 ms | REGRESSION |

### What the refresh traces show

In the RustUI off arm, the sampled NSScreen interval before the key was
slower than the 120 Hz band for 51.0% of hit keys and 38.0% of missed
keys. In bare GPUI the corresponding shares were 31.1% and 33.1%. The
slower-update category requires a recent 8.83–25 ms interval. Older or
longer gaps are unobserved for this classification because the sampler
may have missed unchanged refreshes.

With 250 ms keep-warm, the requested link provided an interval at
837 of 856 RustUI misses and 1,360 of 1,380 bare misses. In both sets the
observed link interval was 8.33 ms at median and p95. No measured GPUI
acquisition used an attached rendering CAMetalDisplayLink. These results
do not establish that lower physical panel refresh caused the misses.

The 250 ms arm reduced RustUI’s miss rate after GPU completion, while
RustUI’s P p95 increased from 6.71 to 9.65 ms and its end-to-end p95
increased from 19.43 to 24.79 ms. The miss rate is conditional on when
the GPU finished. The 250 ms arm worsened end-to-end p95 in bare GPUI too.

The main-thread lag timer overlapped 496 of 980 missed RustUI keys
in the off arm and 930 of 1,962 hits. In bare GPUI it overlapped 53 of
1,085 misses and 55 of 1,915 hits. The overlaps identify coincident
main-thread delays, including cases where the key still hit its window.
The counts of distinct overlapping timer intervals are below.

RustUI had 58, 55 and 98 keys without a GPU-to-SCK join in the off,
four-refresh and 250 ms arms respectively. Bare GPUI joined every key.
The miss rates exclude unjoined keys. The GPUI workload reported no lost
edits or out-of-order presentations.

### Per-arm diagnostics

- **RustUI 10,000-line editor, off.** 5 blocks. Median process CPU 0.07855 s/s (8.047 ms/attempt). P 4.40 / 6.71 ms. 487 late timer intervals overlapped missed keys. 0 display-link pauses.
  - hit: 1962 keys. SCK bracketing interval 17.14 / 34.39 ms, active link interval —. NSScreen prior interval counts {'other_interval': 92, 'slower_update': 1001, 'near_120_update': 752, 'unobserved': 117}.
  - miss: 980 keys. SCK bracketing interval 21.03 / 40.95 ms, active link interval —. NSScreen prior interval counts {'slower_update': 372, 'near_120_update': 545, 'unobserved': 36, 'other_interval': 27}.
- **RustUI 10,000-line editor, 4 refreshes.** 5 blocks. Median process CPU 0.08410 s/s (8.551 ms/attempt). P 4.42 / 6.80 ms. 454 late timer intervals overlapped missed keys. 2877 display-link pauses. Paired CPU/attempt changes +6.26%, +6.07%, +8.82%, +7.69%, +6.11% (median +6.26%).
  - hit: 1984 keys. SCK bracketing interval 17.61 / 34.13 ms, active link interval 8.33 / 8.33 ms. NSScreen prior interval counts {'slower_update': 1039, 'near_120_update': 785, 'other_interval': 65, 'unobserved': 95}.
  - miss: 961 keys. SCK bracketing interval 20.81 / 41.50 ms, active link interval —. NSScreen prior interval counts {'slower_update': 378, 'near_120_update': 515, 'other_interval': 38, 'unobserved': 30}.
- **RustUI 10,000-line editor, 250 ms.** 5 blocks. Median process CPU 0.08794 s/s (9.078 ms/attempt). P 4.81 / 9.65 ms. 587 late timer intervals overlapped missed keys. 5 display-link pauses. Paired CPU/attempt changes +11.51%, +13.06%, +21.70%, +12.56%, +16.36% (median +13.06%).
  - hit: 2046 keys. SCK bracketing interval 21.34 / 37.83 ms, active link interval 8.33 / 8.33 ms. NSScreen prior interval counts {'other_interval': 103, 'slower_update': 1013, 'near_120_update': 830, 'unobserved': 100}.
  - miss: 856 keys. SCK bracketing interval 21.83 / 36.91 ms, active link interval 8.33 / 8.33 ms. NSScreen prior interval counts {'slower_update': 320, 'near_120_update': 477, 'unobserved': 38, 'other_interval': 21}.
- **Bare GPUI Fast, off.** 5 blocks. Median process CPU 0.04753 s/s (4.745 ms/attempt). P 2.14 / 3.11 ms. 54 late timer intervals overlapped missed keys. 0 display-link pauses.
  - hit: 1915 keys. SCK bracketing interval 19.57 / 33.43 ms, active link interval —. NSScreen prior interval counts {'near_120_update': 1195, 'slower_update': 596, 'other_interval': 84, 'unobserved': 40}.
  - miss: 1085 keys. SCK bracketing interval 21.49 / 37.86 ms, active link interval —. NSScreen prior interval counts {'slower_update': 359, 'near_120_update': 686, 'other_interval': 22, 'unobserved': 18}.
- **Bare GPUI Fast, 4 refreshes.** 5 blocks. Median process CPU 0.05492 s/s (5.483 ms/attempt). P 2.20 / 3.11 ms. 78 late timer intervals overlapped missed keys. 2997 display-link pauses. Paired CPU/attempt changes +9.06%, +27.25%, +17.56%, +15.33%, +16.37% (median +16.37%).
  - hit: 2049 keys. SCK bracketing interval 20.51 / 37.41 ms, active link interval —. NSScreen prior interval counts {'near_120_update': 968, 'slower_update': 945, 'unobserved': 77, 'other_interval': 59}.
  - miss: 951 keys. SCK bracketing interval 22.99 / 40.56 ms, active link interval 8.33 / 8.33 ms. NSScreen prior interval counts {'slower_update': 313, 'near_120_update': 585, 'unobserved': 29, 'other_interval': 24}.
- **Bare GPUI Fast, 250 ms.** 5 blocks. Median process CPU 0.05773 s/s (5.764 ms/attempt). P 2.29 / 6.40 ms. 237 late timer intervals overlapped missed keys. 5 display-link pauses. Paired CPU/attempt changes +24.10%, +30.39%, +27.57%, +20.27%, +18.37% (median +24.10%).
  - hit: 1620 keys. SCK bracketing interval 19.21 / 33.19 ms, active link interval 8.33 / 8.33 ms. NSScreen prior interval counts {'near_120_update': 887, 'other_interval': 66, 'slower_update': 604, 'unobserved': 63}.
  - miss: 1380 keys. SCK bracketing interval 30.23 / 40.98 ms, active link interval 8.33 / 8.33 ms. NSScreen prior interval counts {'slower_update': 445, 'near_120_update': 830, 'other_interval': 60, 'unobserved': 45}.
- **AppKit NSTextView, off.** 5 blocks. Median process CPU 0.05607 s/s. P and miss-conditioned lag unavailable. 0 display-link pauses.
  - no_gpu_clock: 2995 keys. SCK bracketing interval 8.96 / 10.59 ms, active link interval —. NSScreen prior interval counts {'slower_update': 8, 'unobserved': 3, 'near_120_update': 2984}.
- **AppKit NSTextView, 4 refreshes.** 5 blocks. Median process CPU 0.06589 s/s. P and miss-conditioned lag unavailable. 2990 display-link pauses. Paired CPU/s changes +15.23%, +21.20%, +22.50%, +11.37%, +2.06% (median +15.23%).
  - no_gpu_clock: 2995 keys. SCK bracketing interval 8.93 / 10.10 ms, active link interval 8.33 / 8.33 ms. NSScreen prior interval counts {'slower_update': 10, 'near_120_update': 2984, 'unobserved': 1}.
- **AppKit NSTextView, 250 ms.** 5 blocks. Median process CPU 0.07670 s/s. P and miss-conditioned lag unavailable. 5 display-link pauses. Paired CPU/s changes +20.43%, +41.09%, +45.30%, +16.49%, +34.26% (median +34.26%).
  - no_gpu_clock: 2995 keys. SCK bracketing interval 8.83 / 10.19 ms, active link interval 8.33 / 8.33 ms. NSScreen prior interval counts {'slower_update': 6, 'unobserved': 2, 'near_120_update': 2987}.

Per-key intervals, correlation windows and classifications are saved in
`target/headstart/g5-l5/per-key.csv`. `analysis.json` contains the numeric
summaries, CPU-per-second paired changes and per-capture counts. The
release binaries were frozen before timing and the sync-off patch was
reversed from the checkout. The branch carries the benchmark helper,
without a GPUI Fast or Kit runtime patch.

## Functional checks

A local native fixture passed with the same helper on the built-in display.
The off arm created no display link. The four-refresh arm stopped twice
after exactly four callbacks. The 250 ms arm paused once, after the second
input’s deadline. Both candidates requested 120/120/120. A deliberate
main-thread stall was detected in all three arms. These smoke runs did
not enter the latency or CPU table. The helper compiled for a macOS 13
deployment target with the macOS 14 API use guarded at runtime.
