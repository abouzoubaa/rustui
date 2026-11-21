# Stage P measurement rules

Owner rule, 2026-11-21. Applies to PX-3 and all Stage P tickets. PX-1 also has its explicit observer-overhead requirement: phase profiling enabled must stay within 5% of phase profiling disabled across five paired captures.

## Method

Use one release build with an off-by-default candidate switch. For each affected scene, run five matched off/on pairs, alternating **off→on, on→off, off→on, on→off, off→on**. Measure 60 seconds after normal warmup. Keep damage off and use PX-1 buffered logging, with identical logging flags in both arms. Do not compile, profile, or run other load during timing.

Capture `getrusage(RUSAGE_SELF)` at both boundaries of the measured window, after warmup. The deciding metric is the user + system CPU delta divided by the number of attempted presents in that window. Include all process threads. Exclude warmup, post-window cleanup and the buffered exit flush.

For each pair, calculate `(on − off) / off × 100`. Negative means less process CPU per attempted present. Report both arm medians; all five paired changes; and the median, minimum and maximum paired change. Preserve unrounded values for verdicts.

PX-1's observer proof necessarily compares `RUSTUI_FRAME_PROFILE=0` versus `1`, with `RUSTUI_FRAME_LOG=buffered` in both arms. That required phase-flag comparison is separate from the general candidate rule of keeping logging flags identical. Apply the owner verdict thresholds to its process-CPU changes as well as reporting the explicit 5% observer limit.

## Verdict per scene

| Verdict | Rule on all five paired changes |
|---|---|
| GAIN | All five ≤ −3% |
| REGRESSION | All five > +3% |
| NEUTRAL | All five within −3% … +3%, inclusive |
| INCONCLUSIVE | Anything else |

A ticket is kept if `scripts/check.sh` is green and no affected scene is REGRESSION. Report verdicts as measured; NEUTRAL and INCONCLUSIVE are never called a gain.

Also report draw + present elapsed CPU-phase milliseconds per attempted present, key/mouse/append-to-screen median and p95 delay once PX-2 is available, and frames over 8.3 ms. These diagnostics do not change the verdict. PX-2 uses `MTLDrawable.presentedTime`, not submission time; unavailable screen-delay metrics must be identified as not measured.

The Stage G CPU gate is separate and unchanged: PASS if all five paired draw + present changes ≤3%; FAIL if all five >3%; otherwise INCONCLUSIVE. Do not substitute this elapsed-phase metric for Stage P's process CPU metric.
