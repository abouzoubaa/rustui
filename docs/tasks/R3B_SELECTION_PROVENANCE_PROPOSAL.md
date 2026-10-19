# R3b: Table selection provenance proposal

Status: Contract A is selected and implemented by the existing R3a table guard. Contract B remains a future option. No native GUI acceptance of A→B→A ordering is claimed.

## Problem and current contract

R3a reconciles the owner's `Option<usize>` into retained `TableState` and records the raw owner value last seen at render. Its event handler accepts a deferred `SelectRow` or `ClearSelection` only when the current owner value still equals that marker and the event payload equals the native selected row (`crates/rustui/src/components/data/table.rs:246-286`). This prevents many stale and programmatic events from writing the owner. Under selected Contract A, an external owner value observed during reconciliation takes precedence; changes that return to the same value before reconciliation are not observable (`docs/tasks/R3A_COLLECTION_SYNC.md`).

The marker is a value, not provenance. Consider owner selection A at reconciliation. A native C selection queues `SelectRow(C)`. Before that event is delivered, external owner code sets A→B→A. At delivery, both the current owner and marker equal A and the native row equals C, so the guard accepts C. This is a source-derived ordering case, not a reproduced runtime failure. A same-value final owner state cannot reveal the intervening external updates.

Pinned Components emits `SelectRow` from `TableState::set_selected_row` and `ClearSelection` from `clear_selection`, including RustUI's programmatic reconciliation (`vendor/gpui-component-0.7.3/src/table/state.rs:502-524, 624-630`). GPUI enqueues emits and flushes effects when the outer update finishes (`vendor/gpui-pre-0.3.9/src/app/context.rs:778-786`, `src/app.rs:1313-1325, 1997-2015`). A guard scoped only to the setter's stack frame does not establish the origin of the delivered event.

## Selected precedence guarantee

| Contract | Meaning | Cost and limit |
| --- | --- | --- |
| **A. Render-observed precedence (selected)** | Keep R3a's value and native-row checks. External values observed at reconciliation take precedence; an unseen A→B→A transition can lose to an already queued native event. | No public API or upstream change. The narrow ordering case is accepted as a documented limit, not a reproduced defect. |
| **B. Mutation-aware precedence (future option)** | Any external change to the bound selection after a native event is queued invalidates that event, even if the value returns to A before delivery. Later genuine native selections remain eligible. | Requires a reliable selection-specific mutation revision plus event provenance or an equivalent sequence token. The current binding and table event APIs do not supply both. This changes the public mutation contract and may require an approved upstream patch. |

Contract A is selected because the A→B→A case is a source-derived ordering possibility, not a reproduced defect, and the current APIs lack the mutation and event provenance needed for Contract B. Contract A preserves the existing public API and explicitly accepts that narrow limit.

Contract A is the selected policy; B is not required for this milestone. Do not implement a per-owner action counter as a substitute if B is reconsidered: unrelated owner changes could invalidate a legitimate native click. A revision advanced only by `Binding::write_to`/`update_on` is also insufficient. The component macro generates direct `set_*` and `update_*` methods (`crates/rustui-macros/src/component.rs:55-90`), `ComponentHost::component_mut` is public (`crates/rustui/src/runtime/component_host.rs:295-297`), and owner tasks mutate `host.component` directly (`crates/rustui/src/runtime/tasks.rs:100-123`). The normal `OwnerAction` and binding writer/updater paths pass through the host updater, but that is not an exhaustive mutation boundary (`crates/rustui/src/view.rs:62-92, 490-520`; `crates/rustui/src/runtime/component_host.rs:315-324`).

## Implementation boundary if B is revisited

Design the source-of-truth revision before changing the table guard. It must advance on every effective **external selection** mutation, including direct setters, updater closures, task completions, and binding writes, while leaving unrelated owner fields alone. An action counter cannot infer which field an arbitrary closure changed without comparing a before/after selection snapshot at a common owner mutation boundary. The current public `component_mut` escape hatch must be covered by that boundary or given a documented mutation/notification contract. The revision should identify the retained table/binding instance so sibling tables cannot invalidate one another. Define whether A→B→A entirely inside one mutation counts as a change; a before/after snapshot cannot observe it.

The native event also needs an origin and sequence that can be associated with the revision at **emission**, not merely read at delivery. `TableEvent` currently carries no origin or revision (`vendor/gpui-component-0.7.3/src/table/state.rs:81-88`). Reading the latest revision or native row at delivery cannot distinguish a stale user event from a current one, and a single “last event” flag risks losing consecutive native events. A table-local queue of expected programmatic payloads is not enough when user and programmatic emissions carry the same payload and interleave. If upstream event metadata is needed, use a separately approved, reproducible versioned patch against the pinned Components package; do not edit `vendor/` directly or change its revision as part of this proposal.

Likely RustUI touch points are `crates/rustui-core/src/binding.rs` for binding identity or revision exposure, `crates/rustui/src/runtime/component_host.rs` and `crates/rustui/src/view.rs` for mutation tracking, `crates/rustui-macros/src/component.rs` for generated mutation paths, `crates/rustui/src/runtime/tasks.rs` for task coverage, and `crates/rustui/src/components/data/table.rs` for event validation. That is an impact map, not authorization to edit those files. Avoid extending List or Tree solely for Table's upstream event behavior; assess their contracts separately if a shared public API changes.

## Focused verification plan if B is revisited

1. Exercise the A→B→A sequence with a queued native C event and assert the selected contract's owner result. Also cover A→B where B differs at delivery, owner clear, an out-of-range owner index, and data replacement before delivery.
2. Apply owner selection programmatically and verify its emitted `SelectRow`/`ClearSelection` never writes an older value back. Verify both payload equality and native-state equality cases.
3. Queue consecutive genuine native selections, including a select followed by clear, and verify the final owner value and number of notifications. An unrelated owner-field mutation between those events must not discard either under contract B.
4. Under contract B, exercise generated setters and `update_*`, ordinary `OwnerAction`, binding writer/updater, public `component_mut`, and task completion. Show each effective external selection mutation advances provenance and unrelated mutations do not. Include a mutation that returns to the original value where the chosen transaction semantics require it.
5. Run a native Playground interaction check for click, keyboard selection, clear, external selection update, and rapid alternating external/native updates. Record actual event order and screenshots or observations. Unit or headless checks alone do not establish GUI acceptance.

No Cargo command or GUI check was run for this proposal. Preserve the pinned dependency and the R3a behavior until the precedence contract and any upstream patch scope are approved.
