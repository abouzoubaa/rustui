# Descriptor-first composition implementation

The user's approval of the pending proposals on 2026-10-25 selects the prepared
Strategy A: a versioned descriptor composition over compiled RustUI factories.
RustUI stays independent of any editor. This adds no rendering/reactive runtime,
external transport, arbitrary code execution, or Rust source serialization.

## Source contract

1. `CompositionDocument` uses its own `rustui.composition` format/version 1,
   stable node IDs, one root, and named ordered child slots. Each node contains
   the existing validated `PreviewDocument`. Existing preview/catalog formats
   and ordinary leaf registrations remain compatible.
2. `register_composable` adds explicit child-component allowlists and slot
   cardinality. Factories receive owned child view descriptions, so they can
   place them in ordinary RustUI containers. Leaf-only preview rendering rejects
   components with required child slots rather than rendering incomplete trees.
3. `CompositionHost` validates topology/schema before recursively constructing
   compiled views and wraps nodes in GPUI-backed RustUI keyed identity scopes.
   Root remount generations separate ownership; reordering preserves stable IDs.
4. Insert, remove-subtree, move-to-end and typed property edits apply to a clone,
   validate as a batch, and replace the live document only on success. Exact
   generation/revision stamps reject stale requests. No-op edits retain stamps.
5. Encode/decode reuse the existing node codec and numeric/schema checks,
   rejecting duplicate maps, unsupported versions and malformed documents.
   `mount_source` decodes before changing the host; `encode` returns its current
   document. No version migration is invented; unsupported versions fail.
6. Local signals include node identity and exact composition stamp. Dispatch
   resolves only the declared, allowlisted local action for that node; owners
   invoke compiled handlers. Edits, removal, unmount and remount reject stale
   signals. This is local Rust bookkeeping, not an external bridge.

Inputs/outputs are capped at 1 MiB; node count at 4,096; graph/render depth at
128; slots at 128 per component/node; edits at 8,192 per batch. Topology and
remove traversal reject cycles before recursive rendering. Rendering uses the
existing ownership/layout/rendering path. The registry remains fixed after host
construction. Registered factories are trusted compiled Rust code.

## Canonical consumer and evidence

The existing Playground Layout page now includes a public-API composition
specimen: add children, remove the last child, reorder the first to the end,
and edit a child's text property. It uses the same component application and
shows validation errors. No second application or editor-specific dependency
was introduced.

Focused preview source checks passed 43/43, including 11 composition tests;
public-API Playground composition checks passed 1/1 after property editing,
source reload and codec additions. Source
contracts 1–6 above are implemented; native UI/AX/keyboard, browser/IME/AX and
size/performance acceptance are 0/3 closed at this checkpoint. These results do
not establish that the rendered controls were operated in an OS/browser.

D1d remains separate: the user has been asked for the external consumer and
transport. No transport, remote action policy, or external-host acceptance is
inferred from this composition implementation.
