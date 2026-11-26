# G-R0 damage rebaseline on G5

The port starts at G5 integration commit `b3d39a7990351cde8740688bfae742b9b116151f`.
It imports [PR #163](https://github.com/abouzoubaa/rustui/pull/163) at
`ad2fc39e75f1a2195488c803e6db92692526b07b`, including replay provenance,
preflight and the signature cache, per-frame encoding metrics, Direct
hysteresis, and the opt-in 8-frame / 30-frame planner back-off. The prior
results are preserved in [G0 baseline](G0-baseline.md) and
[PX-2 telemetry](PX-2-G2-telemetry.md). A single integration candidate applies
after G5 in both stacks. Damage and planner back-off remain off by default.

## Conflict resolutions

| Overlap | Resolution |
| --- | --- |
| Stack order, patch manifests and Cargo checksums | Kept the G5 stack and its approved prefix, then appended one damage candidate with fresh hashes. Replayed the candidate to update the owned mirrors. |
| Core `fast/mod.rs` and `gpui.rs` | Added replay provenance and its exports alongside G5 modules. Kept one copy of the PX-2 exports. |
| Core `window.rs`, present entry and end-of-paint hooks | Kept G5's presentation scope and paint sequence. The PX-2 hooks were already present alongside G5's native and per-surface probes. |
| PX-2 `input_latency.rs` and Apple `presented.rs` | Kept G5's native timestamp and display attribution in the Core bridge. The Apple actual-display callback already matched the #163 copy. |
| Apple `fast/mod.rs` | Preserved the G5 pacing, presentation-order, latency and late-drawable modules. Imported the damage modules under `fast/damage`. |
| `damage/frame.rs`, off-path, atlas status and priming | Ported early rejection, Direct hysteresis, cached signatures and back-off from #163. Kept distinct `atlas_lost`, `verify_off_path` and `prime` reasons. |
| Metal draw, Skip and instance submission | Preserved G5's source, capacity, transaction, density and order checks. Restored Skip and instance counters. Direct draws can use the same pre-acquisition instance setup in both arms, then complete through the damage state machine. |
| Metal verification and submission order | Reference buffers are encoded first and commit only after the primary passes the order guard and commits. A rejected primary clears unsubmitted damage state. The verify run compares the drawable's bytes against an independent full draw, including Direct fallback frames. Drawables are readable only in explicit verify mode or the existing debug test configuration. Same in verify mode takes Copy so the oracle has a drawable. |
| Lab, benchmark runner and reports | Kept the G5 native Cocoa workloads and versions. Preserved actual-display PX-2 diagnostics in the G0 parser and restored buffered planner and encoding counters. The native timing runner gained an explicit damage argument with the existing off default. |

## Measurement

The release is built with pinned Rust 1.98.0. Each workload runs five
alternating OFF/ON pairs, 60 seconds per capture after warmup. All runs use
`RUSTUI_G5_PROFILE=1`, display sync on, buffered logging, and the existing
back-off opt-in left off. The off and on timing arms use `RUSTUI_DAMAGE=0`
and `1`. Native Cocoa events drive typing, hover, drag, and scroll. The
idle caret uses the existing L8 focused boundary snapshots and never injects
input during the measured window. With the G5 profile active, the legacy synthetic warm-up did not satisfy
the owner-binding assertion. The L8 route requires
a key and active window at both boundaries, plus repeated alternation in the
caret scene's instance totals. Handler→SCK is inapplicable to idle. The visible
version barcode is identical instrumentation in both arms and can change
the scene's damage bounds.

The CPU verdict uses the P rule for process CPU per attempted present. The
report also gives process CPU per wall second and the original Stage G
draw+present CPU gate. The Metal pixel oracle runs separately from timing,
using `RUSTUI_DAMAGE=verify` on the same build. GPU time per frame excludes
Skip. GPU time per second includes its effect on the workload.

The final release build, candidate source verification, both stack checks
and Playground smoke passed. The workspace check passed 1,875 tests, with
7 ignored and zero failures across seven groups. All nine focused native
Metal proofs passed, including the deliberately wrong Partial box.

## Results

**The Stage G CPU gate remains closed.** Typing and scroll failed
the draw+present gate. Scroll also regressed in all five pairs on both
process CPU per attempt and process CPU per second. The median scroll
increases were 35.08% in draw+present and 6.66% in process CPU per second.

Apple M1 Max, macOS 27.3. All 25 pairs completed on one native release. The
Lab executable's SHA-256 is `df3e2fe5b156f57ce7508b2ca8be58b73b51df9423ec794aba14eaddd450ebd6`. Capture-boundary background CPU
ranged from 3.33% to 24.83%. Load samples above 5% were
rechecked after up to 30 seconds before a pair. The tables retain actual
load, attempts and sample counts. ScreenCaptureKit and the renderer both
showed a cadence shift in the first hover pair. This changes CPU per wall
second independently of CPU per attempted present. GPU time and latency
columns use medians of capture summaries.

| Scene | Valid pairs | Process CPU ms/attempt OFF → ON | P verdict | Draw+present G2 gate | GPU ms/frame OFF → ON | GPU ms/s OFF → ON | Handler→SCK OFF → ON p50/p95 ms |
| --- | ---: | --- | --- | --- | --- | --- | --- |
| idle | 5/5 | 4.254 → 4.412 | INCONCLUSIVE | INCONCLUSIVE | 0.367 → 0.237 | 0.70 → 0.45 | — → — |
| typing | 5/5 | 8.429 → 8.623 | INCONCLUSIVE | FAIL | 0.728 → 0.334 | 7.16 → 3.25 | 15.70 / 28.06 → 15.56 / 26.95 |
| hover | 5/5 | 3.739 → 3.758 | INCONCLUSIVE | PASS | 0.398 → 0.398 | 38.34 → 38.63 | 15.83 / 22.57 → 15.70 / 22.29 |
| drag | 5/5 | 2.778 → 2.885 | INCONCLUSIVE | INCONCLUSIVE | 0.663 → 0.403 | 67.49 → 41.06 | 14.86 / 19.95 → 14.74 / 19.23 |
| scroll | 5/5 | 2.959 → 3.169 | REGRESSION | FAIL | 0.587 → 0.589 | 60.37 → 59.91 | 15.51 / 21.67 → 16.11 / 21.98 |

## Metal pixel oracle

| Scene | Verified outputs by actual frame mode | Byte mismatches | Verdict |
| --- | --- | ---: | --- |
| idle | direct 1, partial 44, retained 1 | 0 | PASS on observed modes |
| typing | direct 1, partial 200, retained 1 | 0 | PASS on observed modes |
| hover | direct 633, retained 1 | 0 | PASS on observed modes |
| drag | copy 1, direct 1, partial 635, retained 1 | 0 | PASS on observed modes |
| scroll | direct 639, retained 2 | 0 | PASS on observed modes |

## Paired detail

### Idle

- Process CPU per attempt paired changes +0.51%  +2.77%  +1.67%  +2.39%  +8.22% (median +2.39%, min +0.51%, max +8.22%).
- Process CPU per measured second 0.0081 → 0.0084 s/s, INCONCLUSIVE. Paired changes +0.46%  +2.83%  +1.76%  +2.34%  +8.10% (median +2.34%, min +0.46%, max +8.10%).
- Draw + present per attempt 2.160 → 2.285 ms. Paired changes +0.02%  +5.28%  +1.32%  +6.65%  +13.19% (median +5.28%, min +0.02%, max +13.19%).
- Damage-on modes across pairs partial 575. Non-Partial reasons —.
- Native timestamp→Metal p50/p95 ms — → —.

| Pair | Present attempts OFF / ON | SCK joins OFF / ON | Off background at start/end | On background at start/end | Label |
| ---: | ---: | ---: | --- | --- | --- |
| 1 | 115 / 115 | — | 6.16% / 6.18% | 4.97% / 4.78% | measured on a loaded machine (6.18% peak background) |
| 2 | 115 / 115 | — | 4.99% / 10.26% | 3.33% / 5.97% | measured on a loaded machine (10.26% peak background) |
| 3 | 115 / 115 | — | 5.42% / 11.52% | 5.64% / 4.61% | measured on a loaded machine (11.52% peak background) |
| 4 | 115 / 115 | — | 5.22% / 7.11% | 5.23% / 6.88% | measured on a loaded machine (7.11% peak background) |
| 5 | 115 / 115 | — | 5.42% / 6.52% | 5.51% / 6.70% | measured on a loaded machine (6.70% peak background) |

### Typing

- Process CPU per attempt paired changes +0.39%  +4.54%  +0.79%  +1.50%  +4.56% (median +1.50%, min +0.39%, max +4.56%).
- Process CPU per measured second 0.0833 → 0.0837 s/s, NEUTRAL. Paired changes +1.40%  +1.63%  +0.36%  +0.52%  +2.98% (median +1.40%, min +0.36%, max +2.98%).
- Draw + present per attempt 1.685 → 1.876 ms. Paired changes +11.79%  +12.78%  +13.08%  +8.97%  +11.20% (median +11.79%, min +8.97%, max +13.08%).
- Damage-on modes across pairs partial 2916. Non-Partial reasons —.
- Native timestamp→Metal p50/p95 ms 36.12 / 48.14 → 35.97 / 47.16.

| Pair | Present attempts OFF / ON | SCK joins OFF / ON | Off background at start/end | On background at start/end | Label |
| ---: | ---: | ---: | --- | --- | --- |
| 1 | 577 / 583 | 578 / 584 | 3.90% / 7.45% | 8.52% / 7.14% | measured on a loaded machine (8.52% peak background) |
| 2 | 591 / 574 | 592 / 575 | 4.60% / 8.66% | 3.78% / 9.33% | measured on a loaded machine (9.33% peak background) |
| 3 | 595 / 592 | 596 / 593 | 5.90% / 7.99% | 5.08% / 7.35% | measured on a loaded machine (7.99% peak background) |
| 4 | 590 / 585 | 591 / 586 | 3.63% / 7.10% | 7.73% / 10.14% | measured on a loaded machine (10.14% peak background) |
| 5 | 591 / 582 | 592 / 583 | 9.48% / 6.87% | 4.18% / 8.56% | measured on a loaded machine (9.48% peak background) |

### Hover

- Process CPU per attempt paired changes -32.72%  -0.92%  -0.74%  +0.65%  +0.19% (median -0.74%, min -32.72%, max +0.65%).
- Process CPU per measured second 0.3596 → 0.3628 s/s, INCONCLUSIVE. Paired changes +8.21%  +1.77%  -0.60%  +0.77%  +0.44% (median +0.77%, min -0.60%, max +8.21%).
- Draw + present per attempt 1.896 → 1.916 ms. Paired changes -23.93%  +0.25%  +0.82%  +2.79%  +1.07% (median +0.82%, min -23.93%, max +2.79%).
- Damage-on modes across pairs direct 29314. Non-Partial reasons large 29315.
- Native timestamp→Metal p50/p95 ms 25.91 / 32.95 → 25.83 / 32.59.

| Pair | Present attempts OFF / ON | SCK joins OFF / ON | Off background at start/end | On background at start/end | Label |
| ---: | ---: | ---: | --- | --- | --- |
| 1 | 3694 / 5941 | 3680 / 5926 | 3.75% / 23.87% | 22.92% / 24.83% | measured on a loaded machine (24.83% peak background) |
| 2 | 5839 / 5997 | 5826 / 5981 | 4.87% / 8.26% | 3.51% / 11.02% | measured on a loaded machine (11.02% peak background) |
| 3 | 5786 / 5793 | 5777 / 5781 | 3.87% / 12.26% | 3.65% / 4.58% | measured on a loaded machine (12.26% peak background) |
| 4 | 5776 / 5783 | 5769 / 5773 | 4.83% / 8.01% | 5.34% / 8.81% | measured on a loaded machine (8.81% peak background) |
| 5 | 5787 / 5801 | 5779 / 5785 | 14.55% / 11.94% | 7.50% / 14.46% | measured on a loaded machine (14.55% peak background) |

### Drag

- Process CPU per attempt paired changes +8.41%  -18.99%  +4.87%  +0.47%  +3.89% (median +3.89%, min -18.99%, max +8.41%).
- Process CPU per measured second 0.2829 → 0.2937 s/s, INCONCLUSIVE. Paired changes +8.22%  -18.98%  +4.97%  +0.58%  +3.94% (median +3.94%, min -18.98%, max +8.22%).
- Draw + present per attempt 0.746 → 0.827 ms. Paired changes +16.48%  -16.38%  +12.86%  +6.43%  +11.79% (median +11.79%, min -16.38%, max +16.48%).
- Damage-on modes across pairs partial 30549. Non-Partial reasons —.
- Native timestamp→Metal p50/p95 ms 24.68 / 29.72 → 24.62 / 29.05.

| Pair | Present attempts OFF / ON | SCK joins OFF / ON | Off background at start/end | On background at start/end | Label |
| ---: | ---: | ---: | --- | --- | --- |
| 1 | 6115 / 6105 | 6067 / 6072 | 6.93% / 15.21% | 6.26% / 15.18% | measured on a loaded machine (15.21% peak background) |
| 2 | 6109 / 6110 | 6089 / 6057 | 8.83% / 8.79% | 7.72% / 9.34% | measured on a loaded machine (9.34% peak background) |
| 3 | 6102 / 6108 | 6062 / 6078 | 4.40% / 8.63% | 7.00% / 8.10% | measured on a loaded machine (10.38% peak background) |
| 4 | 6108 / 6114 | 6078 / 6086 | 4.16% / 8.01% | 8.49% / 10.00% | measured on a loaded machine (10.00% peak background) |
| 5 | 6109 / 6112 | 6080 / 6090 | 6.09% / 9.32% | 3.89% / 8.19% | measured on a loaded machine (9.32% peak background) |

### Scroll

- Process CPU per attempt paired changes +7.89%  +6.76%  +7.83%  +8.12%  +5.18% (median +7.83%, min +5.18%, max +8.12%).
- Process CPU per measured second 0.3040 → 0.3225 s/s, REGRESSION. Paired changes +6.71%  +5.69%  +6.66%  +6.76%  +4.26% (median +6.66%, min +4.26%, max +6.76%).
- Draw + present per attempt 1.064 → 1.436 ms. Paired changes +35.74%  +34.41%  +37.88%  +35.08%  +31.72% (median +35.08%, min +31.72%, max +37.88%).
- Damage-on modes across pairs direct 30520. Non-Partial reasons large 30521.
- Native timestamp→Metal p50/p95 ms 26.42 / 33.03 → 27.00 / 33.46.

| Pair | Present attempts OFF / ON | SCK joins OFF / ON | Off background at start/end | On background at start/end | Label |
| ---: | ---: | ---: | --- | --- | --- |
| 1 | 6159 / 6092 | 6129 / 6063 | 5.55% / 9.17% | 3.74% / 9.49% | measured on a loaded machine (9.49% peak background) |
| 2 | 6165 / 6104 | 6130 / 6075 | 3.44% / 13.78% | 13.65% / 10.03% | measured on a loaded machine (13.78% peak background) |
| 3 | 6173 / 6106 | 6145 / 6074 | 7.63% / 8.54% | 4.93% / 22.50% | measured on a loaded machine (22.50% peak background) |
| 4 | 6185 / 6107 | 6149 / 6086 | 6.06% / 16.54% | 8.27% / 8.90% | measured on a loaded machine (17.79% peak background) |
| 5 | 6165 / 6112 | 6147 / 6085 | 9.20% / 9.88% | 3.85% / 8.88% | measured on a loaded machine (9.88% peak background) |



The pixel counts include startup and drain from the separate 20-second
oracle runs. All 2,161 completed readbacks matched the full reference.
Idle, typing and drag exercised Partial. Hover and scroll verified mostly
Direct outputs, so their tests establish pixel equality for the modes they
actually used.

The Stage G CPU gate was not met across the five scenes. Typing, idle and
drag had lower GPU time per frame with damage on. Hover and scroll stayed
Direct in the timed windows, with `large` as their reason. That label does
not distinguish planner rejection from an oversized damage box. The
diagnosis is first in the revised Stage G plan.

The paired harness is `scripts/run_g_r0.py`. Run `scripts/build_g5_refs.sh`
first. Its `verify` and `matrix` commands save the build fingerprints,
workload states, diagnostics and raw telemetry under
`target/headstart/g-r0/runs/`.
