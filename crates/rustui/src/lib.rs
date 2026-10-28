//! Standalone declarative Rust UI framework built on GPUI.
//!
//! Compose views through the public prelude:
//!
//! ```
//! use rustui::prelude::{Text, VStack, View};
//!
//! fn greeting() -> impl View {
//!     VStack::new()
//!         .spacing(8.0)
//!         .child(Text("Hello"))
//!         .child(Text("Welcome to RustUI"))
//! }
//!
//! let _view = greeting();
//! ```

extern crate self as rustui;

pub mod accessibility;
pub mod adapters;
pub mod animation;
pub mod components;
pub mod constructors;
pub mod interaction;
pub mod layout;
pub mod localization;
pub mod modifiers;
pub mod prelude;
pub mod preview;
pub mod runtime;
pub mod text;
pub mod theme;
pub mod view;

pub use animation::offset::{SpringOffset, SpringOffsetError};
pub use gpui::{App, AssetSource, Result as AssetResult, SharedString};
pub use modifiers::interaction::{PointerCursor, PointerCursorView};
pub use modifiers::opacity::{AnimatedOpacity, OpacityAnimationError};
pub use modifiers::transform::{Offset, OffsetView};
pub use runtime::{RustUI, component_assets, with_embedded_asset};
pub use rustui_macros::{HStack, VStack, component};

pub use components::data::collection::{Collection, CollectionItem};
