# ResizablePanel API proposal

**Status: proposed for review; not approved or implemented.**

This proposal defines `ResizablePanel` as a two-pane convenience component
implemented on top of the existing `SplitView` contract. `SplitView` already
supports arbitrary keyed panes and native resizing. A second resizable-state
engine would duplicate behavior and create competing identity, constraint, and
accessibility rules. Reviewers may instead decide that the proposed convenience
API is unnecessary and keep this scaffold planned.

## Proposed use and public API

The component represents one resizable boundary between a primary and a
secondary pane. It does not replace `SplitView` for three or more panes.

```rust
ResizablePanel::horizontal("workspace")
    .primary(Text("Files"))
    .secondary(Text("Document"))
    .initial_size(220.0)
    .size_range(140.0..420.0)
```

The proposed API is:

```rust
pub struct ResizablePanel<P, S> { /* private fields */ }

impl ResizablePanel<(), ()> {
    pub fn horizontal(id: impl Into<Identity>) -> Self;
    pub fn vertical(id: impl Into<Identity>) -> Self;
}

impl<P, S> ResizablePanel<P, S> {
    pub fn primary<V: View>(self, content: V) -> ResizablePanel<V, S>;
    pub fn secondary<V: View>(self, content: V) -> ResizablePanel<P, V>;
    pub fn initial_size(self, points: f32) -> Self;
    pub fn size_range(self, range: Range<f32>) -> Self;
}
```

The final renderable type must implement the crate's `View` contract only when
both panes are present. Missing panes must not silently create placeholder
content. If the generic builder type is awkward with the existing view macros,
prefer storing `Rc<dyn View>` or the smallest established type-erasure pattern
already used by `SplitView`; do not add another public wrapper hierarchy.

The axis names refer to pane arrangement: `horizontal` places panes left to
right and exposes a vertical divider; `vertical` places them top to bottom and
exposes a horizontal divider. This mirrors `SplitView`.

## Composition and size constraints

- Require a nonempty split ID. Derive stable internal IDs for `primary`,
  `secondary`, and their divider from that ID, so sibling instances do not
  collide.
- Arrange both caller-provided child views in the official GPUI Components
  resizable panel group through `SplitView` or a factored private helper shared
  with it. Keep native layout, pointer capture, fitting, and notifications in
  GPUI and GPUI Components.
- Default each pane's native minimum to 100 points, matching `SplitPanel`.
  `.size_range(min..max)` sets the primary pane's finite nonnegative minimum
  and finite maximum, with `min < max`. The secondary pane retains the native
  default minimum. Reject non-finite initial sizes and invalid ranges at
  construction, matching `SplitPanel` validation.
- `.initial_size(points)` is a first-insertion preference for the primary pane,
  clamped to its effective range. The secondary pane fills the remaining
  available extent subject to its minimum and the container bounds. If both
  panes cannot satisfy their minima, preserve GPUI's documented overflow or
  fitting behavior; do not promise exact sizes outside feasible constraints.
- Callers must provide a bounded extent along the split axis. An unbounded or
  smaller-than-minimum parent may prevent the requested layout from fitting.
- Use the existing native resizable group and constraints. Do not add a new
  dependency, copied upstream code, global size map, or parallel reactive
  runtime.

## Pointer and keyboard behavior

- Pointer dragging is provided by the native resizable divider. The interactive
  hit target is 9 points wide for a vertical divider or 9 points high for a
  horizontal divider, following the current `SplitView` adapter; the visual
  gutter remains owned by GPUI Components.
- The divider is keyboard focusable and a tab stop. Left/Right resize a
  horizontal arrangement; Up/Down resize a vertical arrangement. Each key
  changes the primary pane by 10 points, in the direction indicated by the key,
  and clamps to the currently feasible range. Character input, IME, and
  modified-key events do not resize the panes.
- Keyboard resizing calls the existing public native `ResizableState` resize
  operation. No RustUI-owned drag loop or alternate resize notification path is
  introduced.

## Retained sizing and child identity

- Retain one native split entity under the component's stable split identity.
  Unchanged axis and constraints preserve the native user's resized sizes across
  ordinary parent renders.
- A changed size range reconciles measured primary-pane size with the new range
  and creates fresh native state only when required by the pinned upstream API.
  Before a measurable container exists, use the configured initial preference;
  do not treat upstream placeholder sizes as measurements.
- Axis changes reset size preferences because width and height measurements
  are not interchangeable. The primary and secondary child retention scopes
  remain stable across parent renders and axis changes.
- Removing the component releases its retained native state and children.
  Recreating it begins with the configured initial size and fresh child state.
- Do not promise exact pixels or proportions after container resizing, axis
  changes, constraint changes, or upstream fitting. Preserve only behavior the
  pinned GPUI APIs support.

## Accessibility semantics

- Expose exactly one focusable divider with `Role::Splitter`, an orientation
  matching the divider (vertical divider for horizontal pane arrangement), and
  the localized `rustui-splitter-resize-divider` accessible name with the
  existing “Resize divider” fallback.
- Publish current, minimum, maximum, and 10-point step values from the same
  measured feasible interval used by keyboard resizing. Before measurement,
  use the configured range as the fallback; when constraints conflict, expose
  the current clamped value as both effective bounds rather than advertising an
  impossible range.
- Pane labels are not inferred from arbitrary child content. Do not add
  accessible labels to pane containers unless an explicit, separately reviewed
  API is proposed; the divider name and numeric values are the minimum contract.
- Source-level AccessKit values do not prove native accessibility tree output or
  VoiceOver speech. Those require runtime acceptance.

## Scope, dependency, and limitations

This design is a RustUI-owned convenience adapter over existing `SplitView`,
GPUI, and pinned GPUI Components 0.7.3 APIs. It does not require a dependency
revision, upstream patch, new rendering or state runtime, or copied vendor code.
The current `SplitView` contract and native implementation are documented in
`SPLIT_VIEW_IMPLEMENTATION.md`; its source-level splitter semantics are
summarized in `SPLIT_VIEW_SPLITTER_SEMANTICS_AUDIT.md`.

The convenience API is intentionally limited to exactly two panes and one
divider. Multi-pane composition, collapsible panes, persistence across app
restarts, proportional sizing, custom divider appearance, and pane-level
accessibility labels are out of scope. Use `SplitView` when those needs apply.

The main design question for approval is whether this two-pane naming and
builder convenience justifies a new public API when `SplitView` already
expresses the same layout. If approved, implementation should share its native
state and reconciliation behavior rather than fork it.

## Focused acceptance criteria

Implementation acceptance should remain separate from this proposal review.

1. Builder tests cover horizontal and vertical arrangements, mandatory primary
   and secondary content, initial size, default values, valid constraints, and
   rejection of empty IDs, non-finite sizes, and invalid ranges.
2. Focused state tests show that a native resize survives an ordinary parent
   render, range changes clamp the primary size, axis changes reset only size
   preferences, and component removal releases its retained state.
3. Focused interaction tests verify native pointer drag, axis-specific arrow
   resizing in 10-point steps, no resizing for unrelated/modified/IME input,
   and clamping at feasible sibling/container bounds.
4. Accessibility source checks verify divider role, orientation, focusability,
   localized name and fallback, and current/minimum/maximum/step values derived
   from the same effective interval as keyboard behavior.
5. A canonical Playground specimen uses only the public RustUI API and shows
   both orientations, a bounded parent, and caller-provided pane content.
6. Native review confirms pointer resizing, keyboard resizing, retained size
   after parent observation, focus traversal, accessibility-tree values and
   name, VoiceOver output, and release/remount behavior. Record exact platform
   and unverified cases; unit checks alone do not satisfy this criterion.
7. Compiled-preview coverage is added only after the public API and its bounded
   serializable configuration are reviewed. Preview descriptors must not encode
   closures, GPUI entities, or arbitrary runtime content.

No implementation, runtime behavior, test result, or acceptance claim is
established by this proposal.
