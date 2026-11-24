# G5-L3b incremental editor wrapping and folds

The candidate branches from L3 commit `b42c70999`. `RUSTUI_G5_INCREMENTAL_EDITOR=1`
enables it. GPUI Kit already retains wrap rows outside the edit with a sum-tree
splice. This patch also reuses byte-identical boundary rows when their prior wrap
geometry is reusable. An inline element can change the geometry without changing
the text, so an unproven row is rebuilt. Tabs are measured only when needed.

With draw-first active, the fold candidates are published by the revision-checked
background parse. The input handler leaves that work to the parse when it is
pending. An unchanged syntax tree and unchanged line geometry can reuse cached
folds. Structural edits refresh the affected subtrees. The editor skips reinstall
only when the cached candidates match the list currently installed. A visible
colour change still requests a redraw.

The wrap oracle compares incremental rows with a full rebuild after insert,
delete, multiline paste/reversal, a 2,048-byte insertion, and a width change.
A separate test covers removing inline geometry while keeping identical text.
Both passed. Two fold oracles against a freshly scanned tree covered insert,
delete, paste, undo and structural edits. They also assert zero fold traversal
on a structure-preserving rename. The focused harness compiled the production
splice and fold-cache source against the release dependencies.

## Paired native typing

RustUI Lab’s 10,000-line editor, 10 posted native Cocoa key events/s, direct launch.
Five matched 60-second pairs after warmup, with alternating arm order and one
frozen release binary. Both arms used L3 editor trimming and late drawable
acquisition, PX-3, buffered logging, damage off and passive display link off.
The #177 display-sync patch was combined with L3 for the measurement build only.
The layer reported `displaySyncEnabled=0` in all ten measured windows.
ScreenCaptureKit matched visible version markers to input versions. P is native
handler entry to GPU end. Times are milliseconds. All captures were measured on a
loaded machine. The table gives the corrected background CPU at each boundary.

| Pair | P off → on (median / p95) | Handler → SCK off → on (median / p95) | CPU s/s off → on | Background CPU off / on (start → end) |
| --- | --- | --- | --- | --- |
| 1 | 4.27 / 5.84 → 4.28 / 6.12 | 7.91 / 17.00 → 8.43 / 17.59 | 0.06407 → 0.06791 | 7.05% → 18.13% loaded / 12.53% → 15.98% loaded |
| 2 | 4.41 / 6.46 → 4.33 / 6.06 | 8.73 / 18.15 → 8.73 / 18.60 | 0.06924 → 0.06930 | 14.76% → 19.12% loaded / 13.54% → 19.99% loaded |
| 3 | 4.42 / 6.82 → 4.35 / 6.11 | 9.15 / 18.32 → 8.36 / 17.72 | 0.06990 → 0.06940 | 12.53% → 17.58% loaded / 14.12% → 18.74% loaded |
| 4 | 4.45 / 8.92 → 4.35 / 6.28 | 8.77 / 22.59 → 7.86 / 17.01 | 0.06951 → 0.06932 | 10.73% → 16.13% loaded / 11.58% → 16.37% loaded |
| 5 | 4.39 / 6.85 → 4.39 / 6.92 | 8.56 / 19.15 → 8.46 / 18.87 | 0.06978 → 0.07066 | 8.68% → 17.06% loaded / 14.19% → 38.19% loaded |

| Arm | P median / p95 | Handler → SCK median / p95 / max | Handler → Metal median / p95 / max (sync off) | Native / SCK joined | Lost edits / order violations |
| --- | --- | --- | --- | --- | --- |
| Off | 4.40 / 6.75 | 8.59 / 18.85 / 32.47 | 8.22 / 30.60 / 47.15 | 3000 / 2958 | 0 / 0 |
| On | 4.34 / 6.33 | 8.34 / 17.92 / 32.35 | 8.01 / 30.19 / 45.59 | 3000 / 2936 | 0 / 0 |

| CPU normalization | Median off → on | Five paired changes | Median / min / max change | P verdict |
| --- | --- | --- | --- | --- |
| Process CPU ms/attempt | 7.05221 → 7.12264 | +5.96%; +2.12%; +1.13%; -0.77%; +1.30% | +1.30% / -0.77% / +5.96% | INCONCLUSIVE |
| Process CPU s/s | 0.06951 → 0.06932 | +5.99%; +0.10%; -0.72%; -0.27%; +1.26% | +0.10% / -0.72% / +5.99% | INCONCLUSIVE |

P p95 was **6.33 ms** (target ≤ 4.5 ms). Handler → SCK p95 was
**17.92 ms** (target ≤ 15 ms). Both latency targets were missed.
Physical display refresh and missed slots were not measured in this passive-link-off typing run.
The Metal column is same-mode telemetry. SCK is the clock shared across display-sync modes.

### Drawable acquisitions over 1 ms

All eight outliers were in warmup. There were zero measured calls over 1 ms.

| Pair | Switch | Phase | Duration (ms) | Path | In flight | Since link detach (ms) |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Off | warmup | 1.258 | non-immediate | 0 (pre-call) | No prior detach |
| 1 | On | warmup | 1.001 | non-immediate | 0 (pre-call) | No prior detach |
| 2 | Off | warmup | 1.094 | non-immediate | 0 (pre-call) | No prior detach |
| 3 | Off | warmup | 1.068 | non-immediate | 0 (pre-call) | No prior detach |
| 3 | On | warmup | 1.090 | non-immediate | 0 (pre-call) | No prior detach |
| 4 | Off | warmup | 1.089 | non-immediate | 0 (pre-call) | No prior detach |
| 5 | Off | warmup | 1.069 | non-immediate | 0 (pre-call) | No prior detach |
| 5 | On | warmup | 1.040 | non-immediate | 0 (pre-call) | No prior detach |
