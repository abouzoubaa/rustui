# G5-4 motion latching

## Method

RustUI Lab on an Apple M1 Max running macOS 27.3 (26C5047e), branched from
`b3d39a7990351cde8740688bfae742b9b116151f`. The release used Rust 1.98.0,
direct launch, the native Cocoa input path, `RUSTUI_G5_PROFILE=1`, the
buffered frame recorder and damage off. Metal layer telemetry confirmed
display sync on. The owner session was left pending.

The driver posts motion on an absolute 120 Hz or 240 Hz schedule. If it wakes
late, it posts the due events in order. Drag follows the same track in each arm.
Scroll uses signed pixel-wheel events around an interior viewport offset and
checks both the net delta and final viewport position. At 240 Hz macOS may
combine adjacent wheel events before AppKit calls the handler. Delivery counts
are diagnostic, and preserving the delta and viewport is the correctness check.

Input age is ScreenCaptureKit `displayTime` minus the Cocoa timestamp of the
newest input version encoded in the captured pixels. Handler latency uses the
same ScreenCaptureKit endpoint. For each complete decoded frame, the probe
matches the version to the draw and counts newer tagged events posted by draw
start. It also counts how many of those had no recorded handler entry by then.
This infers the Cocoa backlog from timestamps, including events merged by macOS.
The difference between adjacent decoded frames estimates missing 120 Hz slots
only when there was a newer input pending. ScreenCaptureKit can omit unchanged
frames, so this is a ScreenCaptureKit gap estimate. All
measured frame rows and acquisition outliers are saved locally under
`target/headstart/g5-4/`, with the release fingerprints and raw capture logs.

`RUSTUI_G5_LATCH_LATEST=1` drains the same-window native motion prefix before
the post-event wake and applies the last compatible pointer position and
summed compatible wheel deltas before scene construction. A button transition,
different window, incompatible wheel phase, direction reversal or delta-unit
change preserves event order. Windows with platform composition surfaces
retain synchronous event forwarding. The Cocoa queue path can also run with
the legacy display link.

`RUSTUI_G5_LATCH_DEADLINE=1` uses the per-workload baseline p95 for
handler→GPU end, plus 1 ms, to choose the latest permitted build start.
It aims 0.5 ms earlier to allow for main-thread timer delivery. If the callback
or timer is later than the cutoff, that update expires without starting a
build. The runtime logs late starts that were declined, late command submission
and GPU completion past the presentation target separately. The Metal display
link's CPU submit deadline and presentation target have different meanings.
Below macOS 14 the arm leaves legacy scheduling in place and emits an
unsupported marker.
[Apple's display-link update documentation](https://developer.apple.com/documentation/quartzcore/cametaldisplaylink/update/targettimestamp)

## Baseline before the candidate

Six 12-second diagnostic windows, after warmup. These are separate from the
matched comparisons. They were measured on a loaded machine, with 11.71–18.72%
background CPU across the capture boundaries. All six reached the intended
posting rate. The four fewer delivered wheel entries at 120 Hz and 1,443 fewer
at 240 Hz preserved the exact sum and final offset in these controls.

| Lab scene | Target event Hz | Posted Hz | Native/posted | SCK frames | Input age p50/p95 ms | Newer at draw p50/p95 | Still in Cocoa p50/p95 | P p95 ms | Final |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| drag | 120 | 120.00 | 1440/1440 | 1313 | 16.46 / 24.57 | 0 / 0 | 0 / 0 | 10.28 | [80, 80] |
| drag | 240 | 239.97 | 2880/2880 | 1317 | 19.65 / 28.90 | 1 / 2 | 1 / 2 | 8.80 | [80, 80] |
| scroll | 120 | 120.02 | 1436/1440 | 1313 | 19.39 / 27.68 | 0 / 1 | 0 / 1 | 9.48 | -192 px, delta -192 |
| scroll | 240 | 240.00 | 1437/2880 | 1321 | 18.35 / 27.50 | 2 / 3 | 2 / 3 | 9.27 | -384 px, delta -384 |
| hover | 120 | 120.01 | 1440/1440 | 1316 | 19.80 / 24.73 | 0 / 1 | 0 / 1 | 10.56 | [400, 407] |
| hover | 240 | 240.00 | 2880/2880 | 1323 | 19.08 / 27.96 | 1 / 2 | 1 / 2 | 10.11 | [400, 587] |


At 240 Hz, every workload showed newer posted events already waiting at draw
start at the median. At 120 Hz the median draw already used the freshest
posted version. The additional events posted between the draw and the SCK
frame explain why age can still be much larger than one input interval.

## Deadline arm

**REJECTED on this Mac.** Six five-second correctness captures displayed
zero new motion versions inside their measured windows. The best callback
lead was 8.29 ms, and even the smallest calibrated budget required
9.80 ms before the submit deadline. The shortfall occurred before any main-thread
delay timer could matter. The test did not start a late build. These captures
were measured on a loaded machine.

| Lab scene | Posted rate | Needed lead (ms) | Best callback lead (ms) | Callback misses | Motion versions displayed |
| --- | ---: | ---: | ---: | ---: | ---: |
| drag | 120 | 11.28 | 8.29 | 599 | 0 |
| drag | 240 | 9.80 | 8.28 | 599 | 0 |
| scroll | 120 | 10.48 | 8.28 | 595 | 0 |
| scroll | 240 | 10.27 | 8.28 | 594 | 0 |
| hover | 120 | 11.56 | 8.28 | 599 | 0 |
| hover | 240 | 11.11 | 8.29 | 599 | 0 |

Background CPU across those capture boundaries was 12.23–27.65%.
The deadline arm failed the display gate before paired timing. A five-pair
latency table and a CPU performance verdict for this arm are unavailable
because the arm displayed no measured motion. The final internal drag and
scroll state still advanced. The draft retains the diagnostic evidence. No 60-second deadline pairs were run.

## Latest-input matched captures

30 of 30 valid pairs. Each complete pair used one frozen release build,
two 60-second measured windows after warmup, and alternating order. The runner
sampled corrected background CPU before each pair and rechecked high load
after 45 seconds before the interruption and 30 seconds after resuming. The
capture boundary samples ranged from 3.12% to 33.91% background CPU. See
the individual pairs below for loaded-machine labels.

Each table entry is the median of the available capture quantiles. Counts sum
across the matched captures. Missed slots estimate gaps against a nominal
120 Hz. Several scroll and hover runs observed roughly 60 distinct version
updates per second, so these counts also reflect lower cadence. CPU verdicts
use the five paired changes in process CPU seconds per wall second. Input age
uses ScreenCaptureKit, with hardware-to-Metal results shown separately.

| Lab scene | Hz | Candidate | Valid pairs | SCK age off → on p50/p95 ms | Handler→SCK off → on p50/p95 ms | Missed slots off → on | Order violations SCK + Metal | Exact final pairs | CPU s/s off → on | P verdict |
| --- | ---: | --- | ---: | --- | --- | ---: | ---: | ---: | --- | --- |
| drag | 120 | latest | 5/5 | 18.96 / 27.94 → 18.98 / 27.94 | 16.67 / 25.35 → 16.83 / 25.67 | 276 → 344 | 0 | 5/5 | 0.2692 → 0.2692 | INCONCLUSIVE |
| drag | 240 | latest | 5/5 | 19.42 / 28.57 → 19.40 / 28.42 | 16.34 / 25.26 → 16.51 / 25.36 | 344 → 315 | 0 | 5/5 | 0.3231 → 0.3253 | INCONCLUSIVE |
| scroll | 120 | latest | 5/5 | 32.93 / 35.10 → 32.83 / 34.67 | 30.96 / 33.66 → 31.00 / 33.36 | 14420 → 14459 | 0 | 5/5 | 0.2708 → 0.2481 | INCONCLUSIVE |
| scroll | 240 | latest | 5/5 | 32.34 / 33.96 → 32.58 / 34.10 | 30.83 / 32.22 → 30.81 / 32.24 | 17975 → 17977 | 0 | 5/5 | 0.3073 → 0.3004 | INCONCLUSIVE |
| hover | 120 | latest | 5/5 | 37.90 / 40.14 → 37.44 / 39.12 | 31.90 / 33.28 → 31.60 / 33.41 | 17985 → 17994 | 0 | 5/5 | 0.3671 → 0.3589 | INCONCLUSIVE |
| hover | 240 | latest | 5/5 | 32.51 / 36.56 → 34.34 / 36.67 | 31.29 / 35.11 → 31.03 / 32.90 | 17990 → 17996 | 0 | 5/5 | 0.4101 → 0.4227 | INCONCLUSIVE |

| Scene | Hz | Candidate | Hardware→Metal p50/p95/max ms | GPU end→Metal p50 ms | SCK frame interval p50/p95 ms | Versions/s | Rendered, unshown | Acquisition calls >1 ms, all phases |
| --- | ---: | --- | --- | --- | --- | --- | --- | --- |
| drag | 120 | latest | 26.77 / 32.56 / 47.96 → 26.77 / 35.14 / 52.25 | 20.21 → 20.50 | 8.95 / 11.21 → 8.94 / 11.32 | 99.81 → 99.30 | 0 → 0 | 70 → 130 |
| drag | 240 | latest | 27.14 / 35.62 / 53.29 → 27.10 / 35.47 / 52.44 | 20.37 → 20.25 | 8.94 / 11.32 → 8.93 / 11.24 | 100.33 → 99.43 | 0 → 0 | 95 → 90 |
| scroll | 120 | latest | 51.01 / 52.31 / 68.89 → 50.88 / 51.75 / 67.91 | 43.43 → 43.44 | 16.64 / 18.62 → 16.66 / 18.56 | 59.92 → 59.91 | 19 → 19 | 49 → 50 |
| scroll | 240 | latest | 50.56 / 51.73 / 80.29 → 50.73 / 52.09 / 82.00 | 43.57 → 43.40 | 16.68 / 18.32 → 16.69 / 18.29 | 59.91 → 59.92 | 20 → 20 | 28 → 29 |
| hover | 120 | latest | 55.08 / 56.39 / 71.95 → 54.89 / 56.37 / 76.19 | 42.31 → 42.12 | 16.66 / 18.96 → 16.66 / 18.96 | 59.95 → 59.95 | 20 → 20 | 26 → 26 |
| hover | 240 | latest | 49.70 / 54.00 / 87.40 → 51.73 / 52.75 / 77.26 | 42.86 → 41.90 | 16.68 / 18.65 → 16.66 / 18.76 | 59.95 → 59.95 | 20 → 20 | 30 → 32 |

- drag 120 Hz latest, age p95 change -0.01 ms. Paired CPU changes -3.30%, +5.96%, +4.37%, -7.47%, -2.61%. Median -2.61%, range -7.47% to +5.96%. Newer events already posted at draw, p50/p95 0.0 / 1.0 → 0.0 / 1.0.
  - Pair 1. Loaded, peak background off/on 21.18% / 23.40%. Endpoints [80, 80] / [80, 80]. Native/posted 7200/7200 / 7200/7200. Deadline starts 0, misses {}.
  - Pair 2. Loaded, peak background off/on 32.55% / 33.91%. Endpoints [80, 80] / [80, 80]. Native/posted 7200/7200 / 7200/7200. Deadline starts 0, misses {}.
  - Pair 3. Loaded, peak background off/on 21.21% / 30.42%. Endpoints [80, 80] / [80, 80]. Native/posted 7200/7200 / 7200/7200. Deadline starts 0, misses {}.
  - Pair 4. Loaded, peak background off/on 21.25% / 22.56%. Endpoints [80, 80] / [80, 80]. Native/posted 7200/7200 / 7200/7200. Deadline starts 0, misses {}.
  - Pair 5. Loaded, peak background off/on 21.53% / 21.15%. Endpoints [80, 80] / [80, 80]. Native/posted 7200/7200 / 7200/7200. Deadline starts 0, misses {}.
- drag 240 Hz latest, age p95 change -0.14 ms. Paired CPU changes +1.18%, +1.75%, -4.20%, -1.70%, +0.98%. Median +0.98%, range -4.20% to +1.75%. Newer events already posted at draw, p50/p95 1.0 / 2.0 → 1.0 / 2.0.
  - Pair 1. Loaded, peak background off/on 24.24% / 21.81%. Endpoints [80, 80] / [80, 80]. Native/posted 14400/14400 / 14400/14400. Deadline starts 0, misses {}.
  - Pair 2. Loaded, peak background off/on 22.61% / 21.65%. Endpoints [80, 80] / [80, 80]. Native/posted 14400/14400 / 14400/14400. Deadline starts 0, misses {}.
  - Pair 3. Loaded, peak background off/on 21.65% / 30.49%. Endpoints [80, 80] / [80, 80]. Native/posted 14400/14400 / 14400/14400. Deadline starts 0, misses {}.
  - Pair 4. Loaded, peak background off/on 21.25% / 22.13%. Endpoints [80, 80] / [80, 80]. Native/posted 14400/14400 / 14400/14400. Deadline starts 0, misses {}.
  - Pair 5. Loaded, peak background off/on 25.94% / 21.68%. Endpoints [80, 80] / [80, 80]. Native/posted 14400/14400 / 14400/14400. Deadline starts 0, misses {}.
- scroll 120 Hz latest, age p95 change -0.43 ms. Paired CPU changes -26.85%, -1.51%, -9.26%, +0.19%, -7.19%. Median -7.19%, range -26.85% to +0.19%. Newer events already posted at draw, p50/p95 1.0 / 1.0 → 0.0 / 1.0.
  - Pair 1. Loaded, peak background off/on 22.56% / 28.38%. Endpoints -960 px, delta -960 / -960 px, delta -960. Native/posted 7189/7200 / 7185/7200. Deadline starts 0, misses {}.
  - Pair 2. Loaded, peak background off/on 21.83% / 22.55%. Endpoints -960 px, delta -960 / -960 px, delta -960. Native/posted 7105/7200 / 7155/7200. Deadline starts 0, misses {}.
  - Pair 3. Loaded, peak background off/on 17.84% / 18.72%. Endpoints -960 px, delta -960 / -960 px, delta -960. Native/posted 7172/7200 / 7149/7200. Deadline starts 0, misses {}.
  - Pair 4. Loaded, peak background off/on 20.19% / 17.44%. Endpoints -960 px, delta -960 / -960 px, delta -960. Native/posted 7129/7200 / 7165/7200. Deadline starts 0, misses {}.
  - Pair 5. Loaded, peak background off/on 17.81% / 17.50%. Endpoints -960 px, delta -960 / -960 px, delta -960. Native/posted 7186/7200 / 7150/7200. Deadline starts 0, misses {}.
- scroll 240 Hz latest, age p95 change +0.15 ms. Paired CPU changes -2.26%, +1.60%, +14.20%, -2.10%, -1.51%. Median -1.51%, range -2.26% to +14.20%. Newer events already posted at draw, p50/p95 2.0 / 3.0 → 2.0 / 3.0.
  - Pair 1. Loaded, peak background off/on 18.55% / 17.57%. Endpoints -1920 px, delta -1920 / -1920 px, delta -1920. Native/posted 7189/14400 / 7494/14400. Deadline starts 0, misses {}.
  - Pair 2. Loaded, peak background off/on 19.12% / 17.60%. Endpoints -1920 px, delta -1920 / -1920 px, delta -1920. Native/posted 7189/14400 / 7211/14400. Deadline starts 0, misses {}.
  - Pair 3. Loaded, peak background off/on 19.82% / 21.96%. Endpoints -1920 px, delta -1920 / -1920 px, delta -1920. Native/posted 7193/14400 / 7249/14400. Deadline starts 0, misses {}.
  - Pair 4. Loaded, peak background off/on 7.23% / 9.70%. Endpoints -1920 px, delta -1920 / -1920 px, delta -1920. Native/posted 7191/14400 / 9276/14400. Deadline starts 0, misses {}.
  - Pair 5. Loaded, peak background off/on 9.96% / 9.96%. Endpoints -1920 px, delta -1920 / -1920 px, delta -1920. Native/posted 7193/14400 / 7202/14400. Deadline starts 0, misses {}.
- hover 120 Hz latest, age p95 change -1.02 ms. Paired CPU changes -2.04%, -3.64%, -7.76%, +3.79%, -4.21%. Median -3.64%, range -7.76% to +3.79%. Newer events already posted at draw, p50/p95 1.0 / 1.0 → 1.0 / 1.0.
  - Pair 1. Loaded, peak background off/on 13.67% / 13.67%. Endpoints [400, 587] / [400, 587]. Native/posted 7200/7200 / 7200/7200. Deadline starts 0, misses {}.
  - Pair 2. Loaded, peak background off/on 18.67% / 7.65%. Endpoints [400, 587] / [400, 587]. Native/posted 7200/7200 / 7200/7200. Deadline starts 0, misses {}.
  - Pair 3. Loaded, peak background off/on 9.89% / 11.99%. Endpoints [400, 587] / [400, 587]. Native/posted 7200/7200 / 7200/7200. Deadline starts 0, misses {}.
  - Pair 4. Loaded, peak background off/on 15.85% / 11.26%. Endpoints [400, 587] / [400, 587]. Native/posted 7200/7200 / 7200/7200. Deadline starts 0, misses {}.
  - Pair 5. Loaded, peak background off/on 13.68% / 13.63%. Endpoints [400, 587] / [400, 587]. Native/posted 7200/7200 / 7200/7200. Deadline starts 0, misses {}.
- hover 240 Hz latest, age p95 change +0.11 ms. Paired CPU changes +6.36%, +13.10%, +2.45%, -1.76%, +1.00%. Median +2.45%, range -1.76% to +13.10%. Newer events already posted at draw, p50/p95 1.0 / 2.0 → 1.0 / 2.0.
  - Pair 1. Loaded, peak background off/on 8.74% / 8.74%. Endpoints [400, 227] / [400, 227]. Native/posted 14400/14400 / 14400/14400. Deadline starts 0, misses {}.
  - Pair 2. Loaded, peak background off/on 11.88% / 6.64%. Endpoints [400, 227] / [400, 227]. Native/posted 14400/14400 / 14400/14400. Deadline starts 0, misses {}.
  - Pair 3. Loaded, peak background off/on 7.04% / 8.40%. Endpoints [400, 227] / [400, 227]. Native/posted 14400/14400 / 14400/14400. Deadline starts 0, misses {}.
  - Pair 4. Loaded, peak background off/on 7.15% / 8.92%. Endpoints [400, 227] / [400, 227]. Native/posted 14400/14400 / 14400/14400. Deadline starts 0, misses {}.
  - Pair 5. Loaded, peak background off/on 10.73% / 15.73%. Endpoints [400, 227] / [400, 227]. Native/posted 14400/14400 / 14400/14400. Deadline starts 0, misses {}.


### Decision

**Latest-input REJECTED.** The largest input-age p95 improvement was 1.02 ms
(120 Hz hover), below the 2 ms gate. All six CPU verdicts were INCONCLUSIVE.
All 30 pairs preserved the exact endpoint, and the candidate recorded zero
order violations. Neither experimental arm is in the
active replay stacks. The single reversible patch remains in the draft as a
record of the tested experiment. Applying it in a disposable checkout and
building the release references restores both opt-in arms without changing
their defaults. The local traces include the per-frame CSV and the list of
all acquisition calls above 1 ms.

During the first 240 Hz scroll run, pair 4 failed the native-focus checks.
The control capture for pair 5 completed, and its candidate capture was
stopped when the owner needed the Mac. The reruns kept the original pair
order on the same frozen build. Only valid matched captures contribute to
the table.

The candidate release build, focused motion-merge test, exact patch replay
and candidate-mode source verification passed. The workspace suite ran
before the hover trace stamp fix (1,875 passed and 7 ignored across seven
groups). The stamp fix passed the release rebuild, candidate source
verification and both hover smoke runs.
