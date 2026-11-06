//! Theme-change invalidation for RustUI's GPUI-hosted component tree.
//!
//! RustUI renderers read [`crate::theme::ActiveTheme`] while lowering. Observe
//! the global through GPUI so an activation or replacement rerenders hosts that
//! already exist.

use gpui::{Context, Subscription};

use crate::theme::ActiveTheme;

pub(crate) fn observe<V: 'static>(cx: &mut Context<V>) -> Subscription {
    cx.observe_global::<ActiveTheme>(|_, cx| cx.notify())
}
