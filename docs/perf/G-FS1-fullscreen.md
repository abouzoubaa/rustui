# G-FS1 — Fullscreen damage fallback

**Cause 4 (the verifier exhausts the per-device retained-texture budget) — documented.** In the G-R3 reproduction, the new fullscreen texture was valid on frame 4. On frame 5, allocating the verifier’s full-frame reference would have raised usage to 92,648,448 bytes against a 67,108,864-byte limit. The verifier disabled damage for that window, and all 351 subsequent submissions were Direct. Ordinary damage-on drag reached Partial in both native fullscreen sequences. Damage remains opt-in.

## Native fullscreen and the windowed control

The Apple M1 Max drove its built-in 3456×2234 Retina display. AppKit gave native fullscreen a **3456×2168** drawable. G-R3 used GPUI’s borderless fullscreen, which produced **3456×2234** on this display. All submitted drawables and allocated retained textures used Metal format 80 (BGRA8Unorm), at 2× scale. The drawable size and format stayed constant after AppKit delivered `NSWindowDidEnterFullScreenNotification` in both native sequences.

A entered native fullscreen from a painted window. B requested native fullscreen at window creation. Each waited five seconds after the native notification before starting drag input. C held the 2400×1600 window still for five seconds. All three used the same release build, G5 profile, display sync on, native Cocoa input and ten seconds of drag. The numerator is submitted Partial frames. The denominator includes every submitted frame from startup, including prime, Copy and Direct. The settled interval starts with the first drawable of the final uninterrupted size. The ten-second column uses the Lab’s recorded interval.

| Damage on | Partial overall | Partial after last size change | First Partial frame | Partial during drag |
| --- | ---: | ---: | ---: | ---: |
| A · enter fullscreen | 1,025/1,030 (99.51%) | 1,025/1,028 (99.71%) | 6 | 1,024/1,024 |
| B · launch fullscreen | 1,023/1,028 (99.51%) | 1,023/1,027 (99.61%) | 6 | 1,022/1,022 |
| C · windowed | 1,011/1,014 (99.70%) | 1,011/1,014 (99.70%) | 4 | 1,010/1,010 |

| Submitted decision / reason | A | B | C |
| --- | ---: | ---: | ---: |
| Direct / `live_resize` | 2 | 2 | 1 |
| Retained / `prime` | 2 | 2 | 1 |
| Copy / `required` | 1 | 1 | 1 |
| Partial / `changed` | 1,025 | 1,023 | 1,011 |

**Retained-texture lifetime.** A allocated at 2400×1600 on frame 2. Frame 3 was Direct at the new size with no kept texture. Frame 4 allocated and filled a valid 3456×2168 target. B allocated at fullscreen size on frame 2, invalidated during its synchronous transition on frame 3 and refilled on frame 4. C allocated once on frame 2. None allocated another kept texture during measured drag. A size update can invalidate the cache when the stored geometry differs. Allocation happens on a subsequent retained draw. A synchronous presentation can also invalidate at the *same* size, as B demonstrates.

## Verification and the historical fallback

| Verify capture | Partial overall | Partial after last size change | First Partial | Direct / live_resize | Retained / prime | Copy / required | Direct / large | Direct / verify_reference_budget | Direct / verify_off_path |
| --- | ---: | ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| A · enter fullscreen | 0/1,020 | 0/1,018 | NEVER | 2 | 2 | 1 | 0 | 1 | 1,014 |
| B · launch fullscreen | 1/1,029 | 1/1,028 | 6 | 1 | 2 | 2 | 0 | 1 | 1,022 |
| C · windowed | 365/368 | 365/368 | 4 | 1 | 1 | 1 | 0 | 0 | 0 |
| G-R3 · borderless fullscreen typing | 0/356 | 0/353 | NEVER | 1 | 2 | 0 | 1 | 1 | 351 |

The layer reported framebuffer-only on in all ordinary runs and off in all verify runs, including the successful windowed control. In native fullscreen the verifier refused a **29,970,432-byte** reference with **59,940,864 bytes** already reserved, on frame 6 in A and frame 7 in B. In the 35-second G-R3 reproduction the corresponding values were **30,882,816** and **61,765,632 bytes**. The failing frame produced Direct, invalidated the kept texture and set the verifier’s per-window disabled flag. Both native verify runs also ended with the kept texture invalid. The windowed verify control kept a valid texture and produced 364 Partial frames in the ten-second interval. All seven captures had zero lost trace records and one GPU completion per submitted frame. The G-R3 replay completed 350 typing ticks. After verification disabled itself, subsequent Direct frames supplied no pixel-comparison evidence.

The reservation counter measures all live reservations on the device. The verifier allocates a full-size reference from the same budget as the kept frame and holds it through command completion. Peak demand therefore depends on overlapping GPU work. The trace identifies the refused reference allocation and its exact size. It does not assign every outstanding reservation to an individual command. See [the budget](../../crates/rustui-gpui-fast-apple/src/fast/damage/retained.rs), [the reference lifetime](../../crates/rustui-gpui-fast-apple/src/fast/damage/verify.rs) and [the per-window fallback](../../crates/rustui-gpui-fast-apple/src/fast/damage/frame.rs).

## Verification

The repository-pinned Rust 1.98.0 check completed with 1,875 passed, 7 ignored and 0 failed. All nine native Metal damage proofs passed, both candidate Fast stacks replayed and reversed exactly, and candidate source verification passed. The focused summary test checks repeated drawable sizes, startup and measured intervals, deferred attempts, and refusal attribution. The seven real traces reconciled submitted frames with GPU completion records.

## Reproduce

Build using `bash scripts/build_g5_refs.sh`. From the repository root, run the following with fresh capture directory names. Each invocation saves method, binary hashes, raw buffered trace and native completion records in its directory. Omit `--capture` to summarize an existing directory.

```sh
for sequence in A B C; do
    python3 scripts/g_fs1_fullscreen.py "target/headstart/g-fs1/on-$sequence" --capture "$sequence"
done
for sequence in historical A B C; do
    python3 scripts/g_fs1_fullscreen.py "target/headstart/g-fs1/verify-$sequence" --capture "$sequence" --verify
done
```

`RUSTUI_G_FS1_TRACE=1` with buffered frame logging records per draw into the preallocated damage log and flushes at process exit. Each record reports the frame index and time, surface, proposed layer size, scale, framebuffer-only flag, actual drawable dimensions and format, kept dimensions/format/validity before and after, allocation, planned and submitted mode, reason, and any refused reservation. Zero texture dimensions mean the texture was absent. The capture runner also records the native animation-completion notification. The renderer retains its existing default-off setting. [Stage G closeout](G-close.md)

