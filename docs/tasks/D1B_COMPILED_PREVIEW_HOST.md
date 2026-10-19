# D1b local compiled preview host

## Contract

`rustui::preview::PreviewRegistry<C>` pairs D1a component schemas with explicitly registered, ordinary compiled Rust factories. `C` is typed local owner context, such as RustUI bindings and owner actions. The registry rejects event schemas with payload properties. Its catalog records explicit action targets; descriptors can reference only those targets. Factories, closures, component state, bindings and GPUI entities are never serialized.

`PreviewHost<C>` is a value stored in an ordinary RustUI component. GPUI remains the owner, scheduler and renderer. The host validates descriptors with the existing D1a validator and invokes factories returning existing `AnyView` values. It has no transport, task loop, reactive runtime or dependency additions.

A `PreviewStamp` contains preview ID, generation and revision. Mount validates before replacing a current specimen and advances generation. Edits and binding feedback require the exact current stamp, apply the D1a typed property batch atomically, and advance revision. Failed validation leaves both values and stamp unchanged. Teardown also requires the exact stamp. A subsequent mount advances generation, including when the same preview/component IDs are reused.

A `PreviewSignal` carries only `EventId` plus the stamp. `dispatch` checks the exact current stamp and resolves the event's explicitly allowlisted `ActionRef`. The owner then invokes its compiled handler. Signals carry no arbitrary payload values; typed binding feedback is a direct local method call through D1a property validation.

`render_factory()` captures an immutable render description and shared immutable registry. It cannot mount, edit or clone the live host. Captured callbacks remain bound to that description's stamp and are checked against the owner's live host before mutation. The captured registry contains ordinary compiled factories, never serialized code.

Rendering uses existing `Keyed` retention scopes, with preview ID plus generation as identity. A property revision or parent rerender keeps that scope and stable control IDs. Teardown/remount changes the scope. Native edit-state preservation still requires GUI evidence; the scope design alone does not prove selection, caret or IME acceptance.

## Narrow owner APIs

`OwnerAction::action_with(value, handler)` captures an ordinary typed value and routes its mutation through the existing GPUI owner updater. The Playground captures the immutable signal using this method.

`Binding::new_with(current, context, read, write)` captures typed context. The PreviewHost API still requires an exact stamp for every edit. The Playground’s retained TextField uses a mount-scoped shared revision lease: accepted feedback advances the lease so the retained InputState can keep editing, and each remount receives a new lease. A callback retained from an older mount therefore stays stale; callbacks sharing the current mount lease intentionally use its latest accepted revision. Ordinary existing actions and bindings retain their existing APIs and behavior.

Callers must dispatch signals and feedback through the live owner host. Resolving an action on a separately constructed host or applying a binding directly outside this validation path cannot establish stale-callback rejection. There is no mutable host clone API.

## Playground specimen

The Layout story includes a dedicated `compiled_preview_gallery` component. It registers representative Text, Button and TextField factories through the public `rustui::preview` API. Controls select a specimen, apply a typed inspector property edit, rerender the parent, teardown, and remount through the specimen selector. Visible status includes owner text, mount stamp, activation count and parent rerender count. Rejection probes exercise stale callback stamps and a wrong property type.

The Button's `activate` event maps only to the explicitly allowlisted `increment` target. TextField edits feed the owner value back to the current descriptor through the captured-stamp binding. Its native control has one stable explicit ID within the mounted preview generation.

## Focused coverage and acceptance status

Source tests cover current action resolution, revision advancement, stale action/edit/binding rejection, atomic wrong-type rejection, teardown, remount generation changes and rejection of unregistered action targets. The core captured binding test accepts one matching-context write and rejects the next stale write without changing the owner value. Workspace `--lib` tests after M127 passed 335 tests across six result groups (`target/headstart/roadmap-workspace-library-tests-20261019.log`);

Formatting was run on the implementation files. The selected packages' locked/offline all-targets check passed, and the Playground build and packaging succeeded. Workspace library tests passed 335 tests across six result groups (`target/headstart/roadmap-workspace-library-tests-20261019.log`); doctests were not part of that run. Native Playground checks confirmed the three specimens mount; a typed inspector edit and two successive typed TextField edits update the owner value in the same mount (generation 1, revisions 0→14→29); a 201-character entry is rejected at the schema’s 200-character limit with both owner and field showing the accepted 200-character value and validation feedback; the recorded parent-rerender sequence reports a generation advance, but source shows that action only increments a render counter; do not attribute that transition or value preservation to a parent-only rerender; the allowlisted Button action fires once; teardown/remount advances generations; and stale callbacks and wrong property types are rejected. NavigationSplit's catalog route exposed the `Workspace navigation` outline and named `Files details`/`Settings details` containers after selection changes. Caret or text-selection retention across rerender remains unverified. The complete native observations and required disposable-app cleanup are recorded in `target/headstart/evidence/d1b-preview-host-acceptance-20261019.json`.

The remaining native check is to establish a TextField caret or selection before parent rerender and observe whether it is retained. The accessibility tree did not expose caret/selection state, so no preservation result is claimed.
