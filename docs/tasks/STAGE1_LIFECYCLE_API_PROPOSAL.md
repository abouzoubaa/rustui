# Stage 1 lifecycle API proposal

**Status:** Proposal only. No lifecycle implementation or public API change is
authorized by this document.

## Existing boundary

`crates/rustui-core/src/lifecycle.rs` is an explicit scaffold: it reserves a
location for lifecycle integration through GPUI ownership and subscriptions,
and requires a separate proposal plus explicit approval. The current public
lifecycle callbacks already live on `rustui::View`:

```rust
pub trait View: 'static {
    fn body(&self) -> impl View;
    fn on_mount(&mut self, _cx: &mut App) {}
    fn on_unmount(&mut self, _cx: &mut App) {}
}
```

`ComponentHost` calls `on_mount` for an owned component when its GPUI entity is
created and registers `on_unmount` with GPUI's entity-release callback. Plain
nested view descriptions do not receive these hooks. GPUI `Subscription`s are
retained by the owning host internally. The F1 ownership design already
specifies this boundary; a second public lifecycle runtime would duplicate it.

## Recommendation: keep the current public callback API

Treat `View::on_mount` and `View::on_unmount` as the Stage 1 lifecycle API, with
the existing signatures and GPUI entity ownership as the sole lifecycle
authority. Do not add a lifecycle event enum, lifecycle manager, token, second
subscription registry, or frame-based mount/unmount callbacks.

The precise contract is:

- Each owned component entity receives one `on_mount` call when that entity is
  initialized, and one `on_unmount` call when GPUI releases that entity.
- A view description that is merely nested in another view receives neither
  callback unless it is made an explicit owned `Component`.
- Removing a child from a RustUI retention map releases RustUI's reference; it
  does not promise immediate `on_unmount` if GPUI still retains the entity.
- GPUI remains responsible for entity destruction and subscription lifetime.
  Subscriptions retained in a host are released with that host. Callbacks must
  not assume that visual disappearance alone means entity release.
- The callbacks keep their current `&mut App` context. No new owner handle,
  scheduling API, or cross-owner mutation capability is introduced.

## Minimum file boundary

If approved, the minimum implementation is **no runtime source change**: the
recommended contract is already implemented by `View` and `ComponentHost`.
Align the scaffold's module documentation and `docs/FILE_REGISTRY.md` status to
state that no shared lifecycle type is currently needed. If later source work
requires a new shared lifecycle type or changes callback behavior, it needs a
new scoped proposal before implementation.

Do not change `View` signatures, `ComponentHost` ownership, GPUI dependencies,
vendored code, or subscription retention in this slice.

## Compatibility and risks

- Preserving the existing callback signatures is source-compatible.
- Clarifying that release can be delayed avoids promising synchronous cleanup
  when other GPUI references still own the entity.
- Keeping subscription ownership inside `ComponentHost` avoids leaking GPUI
  implementation details into a second public abstraction.
- The limitation is intentional: plain nested descriptions cannot observe
  mount/unmount independently. If a concrete use case requires that behavior,
  define its owner identity and release semantics in a separate proposal.
- This contract does not establish native interaction, screen-reader, or
  cross-platform lifecycle acceptance; those remain separate evidence gates.

## Approval requested

Please approve or revise each decision before any scaffold or registry edit:

1. **API:** retain the existing `View::on_mount(&mut App)` and
   `View::on_unmount(&mut App)` signatures as the complete public Stage 1
   lifecycle callback surface; add no public lifecycle types now.
2. **Semantics:** accept GPUI entity creation/release as the callback boundary,
   including potentially deferred release, and limit callbacks to explicitly
   owned components.
3. **Boundary:** if approved, make only the documentation/status alignment in
   `crates/rustui-core/src/lifecycle.rs` and `docs/FILE_REGISTRY.md`; authorize
   no runtime, dependency, vendor, or public API edits in this slice.
