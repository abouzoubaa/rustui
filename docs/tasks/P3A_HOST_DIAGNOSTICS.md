# P3A: Opt-in host lowering diagnostics

`runtime::render_diagnostics` records aggregate RustUI lowering time per
concrete component type in a GPUI application global. Recording is disabled by
default. `enable_render_diagnostics(cx)` starts with empty counters;
`disable_render_diagnostics(cx)` stops and discards them.
`render_diagnostics_snapshot(cx)` returns a copied, type-name-sorted list;
`clear_render_diagnostics(cx)` resets an active session. A disabled snapshot is
empty. The map has at most one entry for each concrete component type rendered
during the active session, not one entry per render.

`ComponentHost::render` starts its monotonic timer only when the global exists.
It measures only `component.into_element(&mut context)`, then increments count,
total duration, and maximum duration with saturating arithmetic. The timing
does not include GPUI's subsequent layout, prepaint, paint, GPU work, platform
submission, or display presentation. Nested child hosts can be rendered inside
parent lowering, so per-type totals may overlap and must not be summed as a
process-wide duration. This is diagnostic evidence, not a frame-time or FPS
metric.

The focused arithmetic test covers multiple types, aggregate count/total/max,
snapshot ordering, and reset. A native Playground exercise and paired W1
overhead measurement are separate acceptance steps. Preserve the existing W1
CPU/RSS protocol and contemporary baseline; do not infer a speedup from these
timings alone. No GPUI cache, upstream patch, or additional rendering runtime is
part of P3A.

## Executed verification

Workspace check and runtime test suite exited 0 (61 tests across 12 test binaries at this integration point). Native M19 Review verified default-off state, enabled lowerings after three local state mutations, and empty records after disable/clear followed by another local mutation. Raw verification record with binary hash: `target/headstart/milestone20-host-diagnostics-gui.json`. These measurements are not a benchmark or evidence about frame/GPU/presentation latency.
