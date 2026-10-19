# P3B: native drag and reorder correctness probe

The opt-in `cache_drag_benchmark` example is an interactive correctness probe.
Cache acceptance requires AX to remain inactive. Run it from the repository root with:

```sh
./scripts/cargo-headstart run -p rustui --example cache_drag_benchmark
```

The `RustUI Cache Drag Probe` window activates on its first frame and uses
white text on explicit dark backgrounds. When it opens, keep accessibility
inactive, click item A once to increment its child-local counter, then drag A
onto item B and let the example
finish. A JSON result is written under `target/headstart/`.
The example times out with `timeout_no_native_drop` if no native drop occurs.
An active accessibility session remains interactive, but its cache acceptance
is excluded because `.cached_fixed` takes its normal-lowering fallback in that
state. The 120-second timeout still applies.

The example uses stable IDs for three fixed-size child components and GPUI's
native `.on_drag`, `.on_drag_move`, and `.on_drop` APIs. Drag moves update the
parent so the cache eligibility path is exercised during an active native drag;
the drop target reorders the keyed children. The output records observed order,
per-child local values and mount counts, active-drag samples with child host
lowering counts, AX state and the observed cache bypass reason, and the child
lowering delta from the last move callback to the post-drop idle update. Frame
samples additionally record root and child lowering deltas between consecutive
polls while native drag remains active. After drop, one parent update warms the
cached representation; a separate parent update measures idle reuse.

The result only reports `complete` if A's incremented value remains associated
with A after reorder, all three child mount counts remain one, an active frame
shows parent lowering plus all three child lowerings, and the measured idle
parent render produces zero child lowerings. Missing active-frame evidence is
`active_fallback_unobserved`; missing idle reuse is
`idle_cache_reuse_unobserved`. Move callbacks alone cannot establish fallback.
Hold the drag and move it for several frames before releasing over B. Any AX
activation during polling or drag samples excludes cache acceptance. The probe
continues through drop to retain order, local state, mount counts, and
`runtime_identity_status` even under AX; its overall status then remains
`excluded_ax_active` and `cache_acceptance_eligible` is false. Active frames
with AX enabled cannot establish the no-AX fallback measurement. Timeout
records also retain the observed AX flag and cache eligibility. These are
correctness observations only, not a speed or GPU measurement.

## Status

Native reorder and component identity diagnostics completed under AX. No-AX
cache fallback and post-drag reuse acceptance remain pending; milestone 44
remains open. If the host cannot deliver the required native gesture with AX
inactive and intermediate movement, preserve the result and report that limit.

Initial automated launches exited immediately on AX activation and retained
`excluded_ax_active` results without interaction details. They do not establish
native drag, reorder, or cache acceptance.

### Retained native interaction evidence

The revised probe was launched as `M44 Cache Drag Interactive.app`; the native CUA
session clicked A, dragged A onto B, and the process exited after its drop
report. The launched executable at
`target/headstart/verification/M44 Cache Drag Interactive.app/Contents/MacOS/cache_drag_benchmark`
was independently hashed with `shasum -a 256`, producing SHA-256
`284c1f5a155125bd5eba77b62325dac4047a6e2cc1bc1deab55f97b13859d4a8`.
The retained result was reviewed directly at
`target/headstart/cache-drag-benchmark-61040-1792323040505224000.json`.
The evidence summary is saved separately in
`target/headstart/milestone44-cache-drag-gui.json`; the original result is retained.

| Observation | Result |
| --- | --- |
| Native dragged ID / drop target | A / B |
| Final order | B, A, C |
| A local value | 166, incremented from 165 |
| A mutation retained after reorder | true |
| Mount counts | A: 1, B: 1, C: 1 |
| Mount identity preserved | true |
| Runtime identity status | `complete` |
| AX seen / final AX | true / true |
| Overall status | `excluded_ax_active` |
| Cache acceptance eligible | false |
| Instrumented drag move updates | 0 |
| Active move / frame samples | both empty |
| Post-drop measured root / child lowering deltas | 1 / 3 |
| Observed idle cache reuse | false |

The native drop establishes reorder and retained local state under AX. This
gesture did not produce instrumented intermediate drag callbacks or active
frame samples, so it supplies no active-drag fallback measurement. The three
child lowerings during the idle parent render are consistent with AX forcing
normal lowering; they do not establish idle cache reuse. A separate native
run with AX inactive, sustained intermediate movement, and both cache gates
satisfied is still required.

Some UI automation sessions activate accessibility when connecting to an app.
Such a session must be recorded as excluded; it cannot establish no-AX cache
fallback evidence. Use a separate native session with AX inactive for that
acceptance run.
