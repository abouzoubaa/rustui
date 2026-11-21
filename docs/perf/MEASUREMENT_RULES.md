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

## G5 latency track

Owner target: **“first eligible refresh” = zero refreshes missed between GPU completion and `presentedTime` beyond the fixed floor measured by the controls.** Keep damage off, use buffered logging, and keep every new behavior switch off by default.

Every latency report includes input → `presentedTime` median/p95/max; GPU-complete → presented median; missed refreshes median/p95 against the control floor; distinct versions shown per second; frames rendered but never shown; `nextDrawable` calls over 1 ms; and the display's observed refresh cadence. Report actual display-link timestamps/cadence separately from a screen's advertised maximum. Missing data is NOT MEASURED, never zero. GPU end time and completion-handler arrival are different timestamps and must remain distinct.

Use five matched off/on captures in the same alternating order and measured-window method above. Process CPU verdicts use the P rule. No latency improvement claim without `MTLDrawable.presentedTime`; a submit timestamp or callback opportunity is not a display endpoint. The AppKit control uses a decoded on-screen version marker and WindowServer frame display time because NSTextView has no Metal drawable; label that endpoint explicitly.

Establish and disclose the control floor before assessing missed refreshes. Keep the standalone immediate Metal control, fullscreen Direct control (confirmed by Metal HUD), and observed AppKit control separate. An unavailable Direct confirmation or unstable/unmeasured floor cannot be replaced by a guessed refresh count. Compare sparse ~10 keys/s and fast ~30 keys/s. G5-1 requires windowed median within one refresh of the standalone Metal control and p95 within two, zero lost edits and monotonic shown versions. Owner acquisition gate (2026-11-22): **no nextDrawable call > 2 ms (¼ refresh at 120 Hz) inside measured windows; all >1 ms calls listed with path, in-flight count and time since detach**. Keep warmup and measured-window counts separate. The >1 ms acquisition count remains mandatory alongside all acquisition durations; a capacity counter alone is not proof that the blocking `nextDrawable` API cannot wait.


## Machine setup

Owner-authorized setup, 2026-11-22, for this macOS measurement host. The following **107 existing RustUI build/cache/capture folders** each contain an empty `.metadata_never_index` marker, and native `tmutil isexcluded` reports every folder excluded. No sudo was used. Parent target/cache exclusions cover their descendants, including future output folders.

Exact folders configured:

```text
/Users/Abium/.cache/rustui-p0-20261121-target
/Users/Abium/.cache/rustui-target
/Users/Abium/.cache/rustui-target-f0-10-gpui-fast
/Users/Abium/.cache/rustui-target-f0-10-gpui-pre
/Users/Abium/.cache/rustui-target-f0-13-fast
/Users/Abium/.cache/rustui-target-f0-check-tree
/Users/Abium/.cache/rustui-target-f0-debug-tree
/Users/Abium/.cache/rustui-target-f0-disabled-family
/Users/Abium/.cache/rustui-target-f0-fast-check-disabledjson
/Users/Abium/.cache/rustui-target-f0-graph-glyph-fix
/Users/Abium/.cache/rustui-target-f0diag
/Users/Abium/.cache/rustui-target-f0diag-postpatch
/Users/Abium/.cache/rustui-target-fixture-kit09
/Users/Abium/.cache/rustui-target-fixture-kit8
/Users/Abium/.cache/rustui-target-fixture-kit9
/Users/Abium/.cache/rustui-target-r1
/Users/Abium/.cache/rustui-target/f0-13-captures
/Users/Abium/.cache/rustui-target/f0-13-captures/fast-after-collection.pixels
/Users/Abium/.cache/rustui-target/f0-13-captures/fast-after-editor.pixels
/Users/Abium/.cache/rustui-target/f0-13-captures/fast-after-grid.pixels
/Users/Abium/.cache/rustui-target/f0-13-captures/fast-after-keyed.pixels
/Users/Abium/.cache/rustui-target/f0-13-captures/fast-after-list.pixels
/Users/Abium/.cache/rustui-target/f0-13-captures/fast-before-collection-native1.pixels
/Users/Abium/.cache/rustui-target/f0-13-captures/fast-before-collection-native2.pixels
/Users/Abium/.cache/rustui-target/f0-13-captures/fast-before-collection-oracle.pixels
/Users/Abium/.cache/rustui-target/f0-13-captures/fast-before-collection-oracle2.pixels
/Users/Abium/.cache/rustui-target/f0-13-captures/fast-before-editor-oracle-v2.pixels
/Users/Abium/.cache/rustui-target/f0-13-captures/fast-before-grid-oracle-v2.pixels
/Users/Abium/.cache/rustui-target/f0-13-captures/fast-before-keyed-oracle-v2.pixels
/Users/Abium/.cache/rustui-target/f0-13-captures/fast-before-list-oracle-v2.pixels
/Users/Abium/.cache/rustui-target/f0-13-captures/fast-before-list-oracle.pixels
/Users/Abium/.cache/rustui-target/f0-13-captures/pre-before-collection-oracle.pixels
/Users/Abium/.cache/rustui-target/f0-13-captures/pre-before-editor-oracle-v2.pixels
/Users/Abium/.cache/rustui-target/f0-13-captures/pre-before-keyed-oracle-v2.pixels
/Users/Abium/.cache/rustui-target/f0-13-captures/pre-before-list-oracle.pixels
/Users/Abium/.cache/rustui-target/f0-14
/Users/Abium/.cache/rustui-target/f0-14/editor-applied-controlled.pixels
/Users/Abium/.cache/rustui-target/f0-14/editor-applied-pixel.pixels
/Users/Abium/.cache/rustui-target/f0-14/final-collection-pixel.pixels
/Users/Abium/.cache/rustui-target/f0-14/final-editor-pixel.pixels
/Users/Abium/.cache/rustui-target/f0-14/final-grid-pixel.pixels
/Users/Abium/.cache/rustui-target/f0-14/final-keyed-pixel.pixels
/Users/Abium/.cache/rustui-target/f0-14/finish-keyed-pixel.pixels
/Users/Abium/.cache/rustui-target/f0-14/finish-list-pixel.pixels
/Users/Abium/.cache/rustui-target/f0-14/grid-bounce-aware.pixels
/Users/Abium/.cache/rustui-target/f0-14/official-collection-pixel.pixels
/Users/Abium/.cache/rustui-target/f0-14/official-editor-pixel.pixels
/Users/Abium/.cache/rustui-target/f0-14/official-grid-pixel.pixels
/Users/Abium/.cache/rustui-target/f0-14/official-keyed-pixel.pixels
/Users/Abium/.cache/rustui-target/f0-14/official-list-pixel.pixels
/Users/Abium/.cache/rustui-target/f0-14/resume-editor-current-1.pixels
/Users/Abium/.cache/rustui-target/f0-14/resume-editor-current-2.pixels
/Users/Abium/.cache/rustui-target/f0-14/resume-editor-current-3.pixels
/Users/Abium/.cache/rustui-target/f0-14/resume-editor-reverted-1.pixels
/Users/Abium/.cache/rustui-target/f0-14/resume-editor-reverted-2.pixels
/Users/Abium/.cache/rustui-target/f0-14/resume-editor-reverted-3.pixels
/Users/Abium/.cache/rustui-target/f0-14/resume-keyed-pixel.pixels
/Users/Abium/.cache/rustui-target/f0-14/resume-list-pixel.pixels
/Users/Abium/.cache/rustui-target/fa-adoption
/Users/Abium/.cache/rustui-target/native-check
/Users/Abium/.codex/worktrees/4adc/rustui/target
/Users/Abium/.codex/worktrees/4adc/rustui/target/headstart/evidence
/Users/Abium/.codex/worktrees/6390/rustui/target
/Users/Abium/.codex/worktrees/6390/rustui/target/headstart/evidence
/Users/Abium/.codex/worktrees/6390/rustui/target/headstart/verification
/Users/Abium/.codex/worktrees/7ee6/rustui/target
/Users/Abium/.codex/worktrees/7ee6/rustui/target/headstart/evidence
/Users/Abium/.codex/worktrees/7ee6/rustui/target/headstart/verification
/Users/Abium/.codex/worktrees/8c17/rustui/target
/Users/Abium/.codex/worktrees/b76c/rustui/target
/Users/Abium/.codex/worktrees/b76c/rustui/target/headstart/evidence
/Users/Abium/.codex/worktrees/b76c/rustui/target/headstart/verification
/Users/Abium/.codex/worktrees/c611/rustui/target
/Users/Abium/.codex/worktrees/c611/rustui/target/headstart/evidence
/Users/Abium/.codex/worktrees/c611/rustui/target/headstart/verification
/Users/Abium/.codex/worktrees/m1291-p3e-manifest/rustui/target
/Users/Abium/.codex/worktrees/m781-font-assertions/rustui/target
/Users/Abium/.codex/worktrees/rustui-production-consolidation/rustui/target
/Users/Abium/.codex/worktrees/rustui-production-consolidation/rustui/target/headstart/evidence
/Users/Abium/.codex/worktrees/rustui-production-consolidation/rustui/target/headstart/performance
/Users/Abium/.codex/worktrees/rustui-production-consolidation/rustui/target/headstart/verification
/Users/Abium/.codex/worktrees/session-coordination/rustui/target
/Users/Abium/.codex/worktrees/stage3-m1215-hover-exit/rustui/target
/Users/Abium/.codex/worktrees/t11c-review-validation/rustui/target
/Users/Abium/.codex/worktrees/t11c-validation/rustui/target
/Users/Abium/Developer/rustui-p0/target
/Users/Abium/Developer/rustui-p0/target/headstart/g5-1
/Users/Abium/Developer/rustui-p0/target/headstart/g5-1/machine-setup
/Users/Abium/Developer/rustui-p0/target/headstart/g5-1/machine-setup/pairs-hold
/Users/Abium/Developer/rustui-p0/target/headstart/g5-1/machine-setup/pairs-screen
/Users/Abium/Developer/rustui-p0/target/headstart/g5-1/machine-setup/window
/Users/Abium/Developer/rustui-p0/target/headstart/p0
/Users/Abium/Developer/rustui-p0/target/headstart/performance
/Users/Abium/Developer/rustui-p0/target/headstart/performance/p0/log-allocations.trace
/Users/Abium/Developer/rustui-p0/target/headstart/px3
/Users/Abium/Developer/rustui-p0/target/headstart/px3b
/Users/Abium/Developer/rustui-p0/target/headstart/px4
/Users/Abium/Developer/rustui-px1/target
/Users/Abium/Developer/rustui-px1/target/headstart/performance
/Users/Abium/Developer/rustui-px1/target/headstart/px1
/Users/Abium/Developer/rustui-px1/target/headstart/px2
/Users/Abium/Developer/rustui/target
/Users/Abium/Developer/rustui/target/headstart/evidence
/Users/Abium/Developer/rustui/target/headstart/performance
/Users/Abium/Developer/rustui/target/headstart/px1
/Users/Abium/Developer/rustui/target/headstart/stage-g
/Users/Abium/Developer/rustui/target/headstart/verification
```

To undo an entry, delete its marker and remove its Time Machine exclusion; repeat for each configured folder:

```sh
folder='/Users/Abium/.cache/rustui-target'
rm "$folder/.metadata_never_index"
tmutil removeexclusion "$folder"
tmutil isexcluded "$folder"
```

Inherited or system Time Machine policy can still apply after a local exclusion is removed. The marker records the requested Spotlight setup; its presence alone is not a measured CPU improvement. Load is sampled again after setup.

### Capture retention

Cleanup uses a strict 72-hour cutoff and a committed report covering the capture. Compiled artifacts and tracked source/report files are retained. The path and size must be listed before deletion. Raw output with missing summary/provenance is retained.


### Temporary loaded-machine protocol

The normal background CPU limit remains **10%**, after excluding Codex/ChatGPT, its shells/agents and the measured app, using a matched top interval and normalizing process percentages by logical CPU count. Save the top five processes at each capture start.

For this owner-authorized G5-1 attempt only: wait up to 30 minutes, sampling every 60 seconds. If no sample reaches 10%, captures may proceed with **`RUSTUI_G5_BACKGROUND_LIMIT=25`** only while accepted start/end checks remain at or below 25%. Above 25%, stop and report. This opt-in does not change the default or the paired P verdict thresholds.

Every capture under that opt-in retains its measured background percentages and is labeled **“measured on a loaded machine (X% background)”**, with actual start/end values. The same build, five matched alternating pairs, normal warmup, 60-second window, damage-off and buffered-logging requirements still apply. Required latency and CPU diagnostics and the G5-1 merge/stop criteria remain unchanged. Restore all owner workloads temporarily stopped for measurement at the end.



### Owner-authorized G5-1 CPU-gate bypass

On November 22 the owner explicitly authorized bypassing the 10% background constraint for the G5-1 finish captures. `RUSTUI_G5_BACKGROUND_LIMIT=none` opts into that bypass for this run; the default remains 10%. It disables CPU-threshold rejection only: missing or inconsistent load samples still fail, and both start/end load plus the top five processes remain recorded. Each capture also saves the requested raw `top -l 1` summary. Every bypass result is labeled **measured on a loaded machine**, with actual background percentages. No app or system service is stopped for this exception.

The five alternating matched pairs, release build, 60-second measured windows after warmup, damage off, buffered logging, P-rule CPU verdict, presentedTime and ScreenCaptureKit endpoints, lost-edit proof, nextDrawable >1 ms reporting, and all G5-1 acceptance targets remain unchanged. The updated macOS build and standalone Metal control floor are recorded separately from older OS cohorts. The earlier 25% exception is not substituted for this explicit bypass.

The owner accepted G5-1 on November 22 under the revised measured-window 2 ms acquisition gate. The former 1 ms gate failed (18/27,025 calls; 16 warmup and two measured; maximum 1.297 ms with spare tracked capacity). The accepted results were measured on a loaded machine under the explicit CPU-gate bypass. A quiet-machine confirmation is deferred. The old measurements and verdicts remain in the G5-1 report.
