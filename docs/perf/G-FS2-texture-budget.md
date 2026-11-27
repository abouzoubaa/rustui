# G-FS2 — Retained-texture budgets on large displays

Damage remains opt-in. The renderer now budgets retained textures per window, including its composition surfaces. On this M1 Max, ordinary damage used Partial at native fullscreen, 5K and 6K on the first changed frame after prime.

## What exhausted the previous budget

A native fullscreen Lab capture on the built-in display traced each successful reservation and release by identity and purpose. Immediately before verification requested its next reference, these were the only live reservations. The old allocator charged width × height × four BGRA8 bytes.

| 3456×2168 texture | Purpose | Bytes | State at refusal |
| --- | --- | ---: | --- |
| Product retained texture | Keeps the window image across frames for Partial redraw and Copy | 29,970,432 | Reserved |
| Previous verification reference | Holds the independent full redraw until the preceding GPU comparison completes | 29,970,432 | Reserved |
| Current verification reference | Supplies the independent full redraw for the new comparison | 29,970,432 | Refused |
| **Total already reserved** | | **59,940,864** | **7,168,000 bytes available under the old 67,108,864-byte cap** |

The third reservation would have taken the total to **89,911,296 bytes**. Neither the drawable nor the shared CPU readback buffers counted against that cap. The earlier 3456×2234 borderless capture had 30,882,816 bytes per texture. Native fullscreen on the same physical panel supplied a 3456×2168 drawable. [G-FS1 diagnosis](G-FS1-fullscreen.md)

## Large-drawable simulation

Each case used the real headless Metal renderer with verification off, a BGRA8Unorm drawable, a prime frame and a second frame with one changed list row. The tests waited for GPU completion and inspected the submitted mode and presence of a valid kept texture.

| Drawable | Raw BGRA8 bytes | Old allocation | Old prime → changed frame | New Metal allocation | New single-surface limit | New prime → changed frame |
| --- | ---: | --- | --- | ---: | ---: | --- |
| 3456×2168 | 29,970,432 | Succeeded | Retained → Partial (`changed`) | 30,605,312 | 33,665,844 | Retained → Partial |
| 5120×2880 | 58,982,400 | Succeeded | Retained → Partial (`changed`) | 60,030,976 | 66,034,074 | Retained → Partial |
| 6016×3384 | 81,432,576 | Refused | Direct → Direct (`large`) | 82,673,664 | 90,941,031 | Retained → Partial |

At 6K the old allocator refused the prime frame before it could create the kept texture. The following frame inherited the old planner’s `large` classification. Changing one row was enough to exercise Partial when the allocation succeeded.

## Budget and failure behavior

The window tracks the Metal storage needed for its live product textures. It uses the descriptor footprint rounded up to the host VM page size, adds 10% headroom to the working set, and caps the result at 536,870,912 bytes. Each newly created texture’s actual `allocatedSize` is checked before the renderer publishes it. Base, overlay and extra composition surfaces share the account. An in-flight texture retains its charge until its command completes. Other windows have independent accounts. A verify-only reference and its readback buffers are outside the product budget.

An over-budget request records `budget_exceeded`. A failed Metal allocation records `alloc_failed`. The window emits a warning with requested and limit bytes on its first failure and can still draw Direct. The failed surface suppresses further allocation attempts at the same size and retries when its drawable size changes. Tests and capture readers reject allocation failures, including a run in which a later resize recovers. The verify-only warning identifies its uncapped reference resource.

Three attempts at a simulated 16,384×8,193 drawable selected Direct with `budget_exceeded` and emitted one warning for 536,936,448 requested bytes. Additional tests exercised the 10% boundary, window isolation, capacity held through completion, ordinary allocation failure, and failed reference allocation followed by successful resize and verification.


## Native fullscreen oracle

The final release build entered native fullscreen, waited at least five seconds after AppKit’s completion notification and dragged through native Cocoa input for 60 seconds. The byte oracle confirmed **1,120 measured Partial frames** at 3456×2168 against independent full Metal draws. Measured submissions included 1,120 Partial frames out of 1,120 frames. There were **0 mismatches, 0 verify-disabled frames and 0 allocation failures**. All diagnostic frames reconciled with GPU completions with no dropped records.

## Fullscreen drag performance

**Draw+present gate FAIL. Process CPU NEUTRAL.** 5/5 matched pairs qualified. Both arms used the same frozen release binary, G5 profile, display sync on, native fullscreen at 3456×2168, in-process CPU clocks and buffered logging. Each arm ran for 60 seconds after warmup, alternating which arm ran first. Verification and fullscreen frame tracing were off for timing.

| Pair (attempt) | Partial % off/on | GPU ms/frame off/on | Draw+present ms/attempt off/on | Draw change | Process CPU s/s off/on | CPU change |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 (1) | 0.00 / 100.00 | 0.9747 / 0.4237 | 0.4721 / 0.5193 | +10.01% | 0.2409 / 0.2435 | +1.07% |
| 2 (3) | 0.00 / 100.00 | 0.9724 / 0.4248 | 0.4739 / 0.5163 | +8.96% | 0.2445 / 0.2408 | -1.52% |
| 3 (1) | 0.00 / 100.00 | 0.9702 / 0.4208 | 0.4636 / 0.5178 | +11.69% | 0.2388 / 0.2421 | +1.39% |
| 4 (1) | 0.00 / 99.98 | 0.9727 / 0.4250 | 0.4702 / 0.5018 | +6.72% | 0.2402 / 0.2339 | -2.62% |
| 5 (1) | 0.00 / 100.00 | 0.9798 / 0.4289 | 0.4704 / 0.5286 | +12.38% | 0.2402 / 0.2447 | +1.87% |
| **Median arm means** | 0 / 99.997 (pooled) | **0.9727 / 0.4248** | **0.4704 / 0.5178** | **+10.01%** | **0.2402 / 0.2421** | **+1.07%** |

Damage-on submitted 30,726 Partial frames out of 30,727. The process-CPU verdict uses paired changes in process CPU seconds per second. The separate historical drag gate uses paired changes in draw+present CPU per attempt against 3%. The change columns on the summary row are medians of paired changes.

| Pair / attempt | Qualified | Other CPU off/on | WindowServer off/on | kernel_task off/on | Thermal |
| --- | --- | ---: | ---: | ---: | --- |
| 1 / 1 | Yes | 31.41 / 35.49 | 48.33 / 48.32 | 11.67 / 13.33 | no limit reported |
| 2 / 1 | No | 24.76 / 103.08 | 50.00 / 48.33 | 13.33 / 18.33 | no limit reported |
| 2 / 2 | No | 59.46 / 34.90 | 48.33 / 48.33 | 15.00 / 11.67 | no limit reported |
| 2 / 3 | Yes | 38.25 / 44.24 | 48.33 / 50.00 | 13.33 / 11.67 | no limit reported |
| 3 / 1 | Yes | 31.09 / 37.40 | 49.99 / 48.33 | 13.33 / 13.33 | no limit reported |
| 4 / 1 | Yes | 34.20 / 34.70 | 48.33 / 49.99 | 13.33 / 11.67 | no limit reported |
| 5 / 1 | Yes | 31.90 / 41.39 | 48.33 / 48.33 | 13.33 / 13.33 | no limit reported |

Load percentages are fractions of one core. The rule excludes the Lab, WindowServer and kernel_task, requires both other-load means ≤100% and an absolute difference ≤15 percentage points, and permits three retries. A CPU speed limit below 100% delays a pair. At least four qualified pairs are required for a verdict.

## Reproduction and checks

The repository’s pinned Rust 1.98.0 check passed with 1,875 passed, 7 ignored and 0 failed. Both Fast stacks passed exact reverse/replay, candidate source verification passed, and all nine native Metal proofs passed. The large-drawable tests also ran on the real Metal device.

From the repository root, use fresh output directories for these commands.

```sh
bash scripts/build_g5_refs.sh
python3 scripts/g_fs1_fullscreen.py target/headstart/fs2-oracle --capture A --verify --seconds 60 --oracle-frames 300
RUSTUI_G_FS1_SEQUENCE=A RUSTUI_G_R2_VIEW_SIGNATURES=1 RUSTUI_DAMAGE_DECLARED=1 python3 scripts/run_g_r2.py matrix --scene drag --output target/headstart/fs2-pairs
```
