# G5-3 — RustUI latency measurement protocol

All behavior stays off by default. Base: g5-integration after the G5-2 safety merge. The accepted G5-2 outliers remain known; this ticket does not relabel them as a passing timing gate.

`GPUI_FAST_DRAWABLE_RESERVE_TWO=1`, with `GPUI_FAST_IMMEDIATE_PRESENT=1`, requires `in_flight <= pool_size - 2` immediately before nextDrawable and in readiness checks. With a three-drawable pool, at most one drawable may already be outstanding when acquiring. The newly acquired drawable can make the count two. The pool-size formula is tested for zero, one, two and three slots.

One frozen pinned-toolchain release build; damage off, buffered logging, identical diagnostic flags. All six workloads have five matched off/immediate pairs in the required alternating order. Hover and drag, where the known outliers occurred, use matched triplets off→immediate→reserve and the reverse order. Both adjacent comparisons have five alternating pairs and share the immediate capture. The reserve timing comparison is scoped to hover/drag; the other scenes receive reserve functional checks. Each capture measures 60 seconds after normal warmup, takes getrusage snapshots, and logs corrected background CPU/top processes. Results remain labeled loaded. No build/profiling overlaps timing.

Workloads: G0 hover, drag, typing with PX-3 on/off, log; the existing F0 acceptance page's Collection scrolling panel. Default behavior and the other F0 panels remain separate from this measured workload.

Latency uses MTLDrawable.presentedTime. Missed-refresh figures reuse the disclosed two-refresh fixed floor from the same-host G5-1 standalone Metal control; this is an inherited control, not a newly measured floor. CPU verdicts use process CPU per attempted present under the P rule.

For hover/drag, every observed display-link refresh without a new displayed version is assigned no new input, frame not ready, no drawable free, presented but replaced, or WindowServer late. Capacity logs identify denied admission (including reserved slots). GPU-ready uses the control-floor eligibility deadline. Replacement is inferred from an unmatched submission superseded by a later shown version; it is not physical-pixel proof. WindowServer late means a GPU-ready frame was handed off but not observed in its eligible slot; the trace cannot separate compositor scheduling from other post-GPU waiting. Observer gaps are disclosed; missing observer ticks are not invented.

## Results and reproduction

The complete before/after tables, five paired CPU changes per comparison, every acquisition over 1 ms, per-capture diagnostics and observed-refresh diagnosis are published in [PR #173](https://github.com/abouzoubaa/rustui/pull/173). Raw capture files stay local. Incomplete sets receive no P verdict.

Use `scripts/build_g5_refs.sh` with the pinned toolchain before recording. `scripts/run_g5_rustui.py --output <folder> --deadline <UTC ISO timestamp>` records the matched cohort without rebuilding; `--functional` runs short correctness checks, never a performance verdict. `scripts/report_g5_rustui.py <folder>/captures.json --document <local Markdown report>` generates the full tables from a copy of this protocol.

Interpret CPU-per-attempt together with total process CPU and attempt counts. More declined/retried presents can change the denominator without reducing total CPU. This diagnostic does not alter the owner's P verdict rule. The five slot labels classify observable trace states: "frame not ready" can include waiting for a provider callback, while "no drawable free" means an explicit application admission refusal, including reserved capacity. Hardware availability and compositor causality are not established by these labels.
