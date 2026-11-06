//! Integration boundaries for GPUI and RustUI services.
//!
//! Theme and localization adapters have runtime implementations. The `gpui`
//! and `gpui_components` modules reserve paths for future adapter APIs.

pub mod gpui;
pub mod gpui_components;
pub mod localization;
pub mod themes;
