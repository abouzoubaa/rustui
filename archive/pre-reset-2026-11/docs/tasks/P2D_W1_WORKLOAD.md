# P2d — W1 retained-child scenario runner

Implemented 2026-10-18 in the canonical RustUI Playground Layout story. This is
a manually paced scenario runner, not an automated or calibrated benchmark.

## Use

Open Layout and click **Reset W1**. Reset creates a new child-key epoch, clearing
child counters and notes, restoring Alpha/Beta presence and normal order, and
removing the title prefix. Existing manual controls remain available; avoid them
during W1 because child-local mutations cannot be read back by the parent.

Click **Next W1 step** once per action, inspecting the result before continuing:

1. Request Alpha increment: Alpha 1, Beta 0.
2. Request Beta increment: Alpha 1, Beta 1.
3. Enable the title prefix; counts remain 1/1.
4. Remove the prefix; counts remain 1/1.
5. Reverse the keyed order: Beta 1 precedes Alpha 1.
6. Restore order: Alpha 1 precedes Beta 1.

The child keys stay stable throughout these six steps. Increment commands are
applied through the existing public `Component.props` path using one-shot counters.
Repeated props reconciliation must not increment again. This exercises a
props-driven update, not a native click delivery or input-latency measurement.

The parent checks its own expected flags and command counters. A mismatch blocks
further dispatch until reset. It cannot inspect mounted child state or confirm
presentation. The UI shows expected counts; **Mark visual pass/fail** records an
operator assertion after the sequence. No automatic child correctness pass is
generated. Reset starts a new run and clears the previous operator result.

## Evidence semantics

The existing event panel records progress. Standard output emits JSON lines with
`kind: rustui_w1`, run, step, and event fields, plus dispatch or operator-result
fields where applicable. Capture application stdout when launching from a
terminal to retain these records. This code does not write a result file itself.
Records have no presented-frame, main-thread, or cycle timing. An operator pass
is manually supplied evidence, not an instrumented assertion.

The process collector remains separate. Sampling CPU/RSS during manually paced
steps does not establish comparable cycle latency or performance improvement.
Automated action scheduling, mounted-child readback, a trustworthy completion
boundary, repeated baseline runs, and calibrated thresholds remain pending.

## Files and validation

- `apps/rustui-playground/src/stories/mod.rs`: runner, expected parent state,
  controls, and JSON/event records.
- `apps/rustui-playground/src/stories/canvas.rs`: reset epoch and command props
  passed to the retained children.
- `apps/rustui-playground/src/stories/child_component.rs`: idempotent command
  application alongside existing manual state.

The first Cargo check caught an unfinished stack-expression integration error;
it was corrected. Final compile and review evidence is recorded in the roadmap.
No GUI acceptance or new tests have been run for this slice. No framework runtime,
vendor source, dependency pin, or GPUI Fast configuration changed.

## Milestone 14 completion

The pending W1 GUI and CPU/RSS baseline checks above are now completed. The
compact W1 layout keeps both children visible, and accessible action labels allow
mounted title/count/order readback. See `P2E_W1_ACCEPTANCE_BASELINE.md` for native
visual observations, 90 measured cycles, raw evidence, calibration and limits.
Native input automation is now available through the CUA driver. Application
cycle/presentation timing remains unavailable; automation time is not app latency.
