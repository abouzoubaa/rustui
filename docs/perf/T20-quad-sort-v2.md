# T20 — Quad-sort V2 measurement

Measured on Apple M1 Max, macOS 27.0 and Rust 1.97.1 against commit `92acc752f572703a45cb6b8df467234a575a3815`. The candidate applies the proposed patch only in its disposable worktree. Runs alternated baseline and candidate, three per group; each JSON records three workload runs.

Each median below is the median of the three JSON case medians. Spread is `(max - min) / median × 100` over those three medians, shown separately for baseline and candidate. These are GPUI test-window measurements, not native frame or GPU timings.

## 10 rows, update only

| Baseline median (ms) | Candidate median (ms) | Candidate / baseline | Baseline spread | Candidate spread |
|---:|---:|---:|---:|---:|
| 0.0635 | 0.0622 | 0.9795 | 2.49% | 2.99% |

## 10 rows, update + forced draw

| Baseline median (ms) | Candidate median (ms) | Candidate / baseline | Baseline spread | Candidate spread |
|---:|---:|---:|---:|---:|
| 0.1236 | 0.1227 | 0.9930 | 2.52% | 1.45% |

## 100 rows, update only

| Baseline median (ms) | Candidate median (ms) | Candidate / baseline | Baseline spread | Candidate spread |
|---:|---:|---:|---:|---:|
| 0.6046 | 0.6102 | 1.0092 | 1.46% | 1.00% |

## 100 rows, update + forced draw

| Baseline median (ms) | Candidate median (ms) | Candidate / baseline | Baseline spread | Candidate spread |
|---:|---:|---:|---:|---:|
| 1.1670 | 1.1725 | 1.0047 | 0.89% | 0.57% |

## 1,000 rows, update only

| Baseline median (ms) | Candidate median (ms) | Candidate / baseline | Baseline spread | Candidate spread |
|---:|---:|---:|---:|---:|
| 6.2735 | 6.2895 | 1.0026 | 2.47% | 1.78% |

## 1,000 rows, update + forced draw

| Baseline median (ms) | Candidate median (ms) | Candidate / baseline | Baseline spread | Candidate spread |
|---:|---:|---:|---:|---:|
| 12.0571 | 12.1015 | 1.0037 | 2.34% | 2.63% |

Verdict: no meaningful difference (|ratio-1| < 5%) for the 1,000-row forced-draw case.

The archived ordered 131,071-element regression is not covered by this workload.
