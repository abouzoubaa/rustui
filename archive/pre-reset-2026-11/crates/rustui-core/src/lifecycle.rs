//! Lifecycle callbacks belong to explicitly owned `rustui::View` components.
//!
//! `ComponentHost` invokes mount on GPUI entity creation and unmount on entity
//! release. Release can be deferred while another GPUI reference owns the
//! entity. Plain nested view descriptions do not receive independent callbacks.
//! GPUI owns subscriptions and entity destruction; no separate core lifecycle
//! type or runtime is needed for this contract. See `F1E_OWNER_CALLBACKS.md` and
//! `STAGE1_LIFECYCLE_API_PROPOSAL.md` in `docs/tasks/` for the boundary.
