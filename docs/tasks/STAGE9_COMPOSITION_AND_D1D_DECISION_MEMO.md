# Stage 9 composition editing and D1d decision memo

**Status: decision requested; no architecture is selected by this memo.** D1a–D1c
already provide the versioned descriptor/codec, local compiled preview host, and
bounded Playground inspector. They do not implement structural composition
editing or an external-host adapter. This memo distills the options already in
`SERIALIZATION_BRIDGE_PROPOSAL.md`; it is not an ADR and authorizes no code,
dependency, or upstream changes.

## Composition editing choices

| Choice | Boundary and likely ownership | Tradeoff |
|---|---|---|
| **A. Descriptor-first composition document** | Extend the current `rustui::preview` contract with a separately versioned composition document: registered component IDs and declared properties, plus explicit slots, stable tree identities, structural operations, and migrations. Keep schemas and validation in `crates/rustui/src/preview/`; keep the editor in the consumer/Playground. | Reuses D1a–D1c and preserves compiled RustUI factories, GPUI ownership, and typed runtime state. Requires defining and maintaining the composition language and catalog/slot coverage. It still cannot serialize arbitrary Rust views, closures, or private state. The current `PreviewDocument` is one compiled root and has no `children` field; structural edits are new scope. |
| **B. Rust source/AST authoring** | Associate a preview with authored Rust source and implement source-aware edits, formatting, conflict handling, build diagnostics, and recompilation in a source-authoring tool. The proposal places this in a later project, outside the runtime descriptor boundary. | Preserves authored program structure and can represent arbitrary compositions, but needs parser/toolchain and source mapping policy, conflict handling, and rebuild integration. It does not make runtime state portable; D1a–D1c remain necessary for live preview properties and actions. No parser or source-editing API is established in the current framework. |
| **C. Keep D1a–D1c as the scope; defer structural editing** | Continue using compiled Rust compositions with registered editable properties. No new framework files or dependencies. | Keeps the implemented, bounded model and avoids prematurely defining a second document language. It does not deliver broader visual composition editing, so Stage 9 remains incomplete. |

The proposal rejects blanket serialization of arbitrary `View`, `State<T>`,
bindings, or closures as an unportable runtime snapshot. It also rejects putting
a UI Studio/Chorus schema inside RustUI: the host may depend on RustUI's public
contract, while RustUI must remain independent of Chorus and any editor app.

## D1d external-host boundary

The only recorded D1d shape is a **consumer-owned adapter**, added after an
actual host and transport are named. It may use the existing `rustui::preview`
descriptor, catalog, validation, and compiled host. Its contract must specify
capability/version negotiation, request/reply/error handling, stale revisions,
disconnect/close, explicit action policy, and owner-safe dispatch. The adapter
does not belong in RustUI core and must not add a Chorus or UI Studio dependency.

The design records do not select a remote host or transport. The concrete
alternative is to keep D1b in-process and defer D1d until that consumer is
identified. Choosing IPC, sockets, or a browser bridge without a named consumer
would invent a design not supported by the current records.

## Decision needed to unblock implementation

1. For structural composition editing, select **A**, **B**, or **C** above.
   Selecting A authorizes a follow-up design slice for the versioned tree/slot
   schema and edit operations before implementation. Selecting B authorizes a
   separate source-authoring architecture proposal; it does not imply a parser
   or dependency has been approved. Selecting C explicitly defers composition
   editing and leaves that Stage 9 goal open.
2. For D1d, name the intended consuming application and transport, or explicitly
   defer D1d. Then decide whether the host may invoke allowlisted preview actions
   remotely; current D1b only dispatches locally bound signal-only actions.
3. Confirm whether the existing Playground inspector remains the canonical
   local consumer. It currently is; this does not by itself choose the external
   host or composition model.

## Source records

- `docs/tasks/SERIALIZATION_BRIDGE_PROPOSAL.md` — implemented D1 boundaries,
  alternatives, future composition and source-authoring scope, and D1d shape.
- `docs/tasks/D1B_COMPILED_PREVIEW_HOST.md` — actual in-process preview host,
  ownership, edit, signal, and lifecycle limits.
- `docs/tasks/D1C_PLAYGROUND_PREVIEW_INSPECTOR.md` — canonical Playground
  inspector and its outstanding native/accessibility acceptance.
- `docs/tasks/ROADMAP_REMAINING_AUDIT.md` — current statement that composition
  editing and D1d remain unresolved.

## Preview-registry readiness check — 2026-10-24

The un-gated descriptor/schema source slice is ready for focused verification.
`PreviewRegistry::register` rejects payload-bearing events, delegates schema
validation and duplicate-ID rejection to `ComponentCatalog`, and inserts the
compiled factory only after catalog registration succeeds. A failed duplicate
registration therefore cannot replace the existing factory. The focused host
regression `registry_rejects_payload_events_and_unmapped_signals` is present.

The filtered regression was attempted but did not reach test execution: Cargo
reported E0463 for missing `gpui`, `gpui_component`, `rustui_core`,
`rustui_i18n`, `chrono`, `gpui_base`, and `serde` crates. Treat the source
coverage as present and runtime verification as blocked; do not report the test
as passing. The next un-gated action is to rerun only that filtered regression
after the required crate artifacts are available.

No un-gated Stage 9 descriptor/schema implementation task remains identified
in this audit. Structural composition remains subject to the owner decision
above, and D1d remains deferred until a consuming application and transport
are named. This check does not authorize either gated slice.

### Separate acceptance item: Stage 9 registry focused regression

Read-only inventory of `target/headstart` found no Cargo `deps` directory in
the standard profile locations (`debug`, `dev`, `headstart`, or `release`) and
no current `rustui` lib-test executable in the inspected dependency output
locations. Some candidate-specific build outputs contain artifacts for preview
dependencies, but those isolated `build/.../out` files do not provide a
matching test executable or the active profile's dependency outputs. This is
the registry test
`registry_rejects_payload_events_and_unmapped_signals`; it is separate from
M1192's completed SearchField query-binding regression. The registry test
cannot run from existing artifacts as-is. Resume this acceptance item only
when either (a) the configured Headstart target tree contains a current
`rustui` lib-test executable and
loadable artifacts for those seven crates, or (b) rebuilding is allowed and
the local Cargo sources/cache can produce them with the approved Headstart
toolchain. Then run only the filtered regression recorded above. This item
does not include D1b, D1d, composition, or GUI acceptance.
