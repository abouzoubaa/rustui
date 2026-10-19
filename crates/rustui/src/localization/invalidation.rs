//! Locale-change invalidation integration.
//!
//! A locale change can alter translated content, text metrics, and layout.
//! This module coordinates notifications through GPUI's existing update model.

use gpui::{Context, Subscription};

use crate::localization::ActiveLocalization;

pub(crate) fn observe<V: 'static>(cx: &mut Context<V>) -> Subscription {
    cx.observe_global::<ActiveLocalization>(|_, cx| cx.notify())
}
