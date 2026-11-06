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
//!
//! A component can own state and expose an action that updates it. Applications
//! using themed controls must activate a theme during startup; provide a valid
//! RustUI theme file at the path used below.
//!
//! ```no_run
//! use rustui::prelude::*;
//! use rustui::theme::{activate_theme, load_theme};
//!
//! #[component]
//! struct Counter {
//!     #[state(0)]
//!     count: i32,
//! }
//!
//! impl Counter {
//!     fn increment(&mut self) {
//!         self.count += 1;
//!     }
//! }
//!
//! impl View for Counter {
//!     fn body(&self) -> impl View {
//!         VStack::new()
//!             .child(Text(format!("Count: {}", self.count())))
//!             .child(Button("Increment", self.action(Self::increment)))
//!     }
//! }
//!
//! let theme = load_theme("themes/macos27.json").expect("load application theme");
//! RustUI::new()
//!     .title("Counter")
//!     .on_launch(move |cx| {
//!         activate_theme(&theme, None, cx).expect("activate application theme");
//!     })
//!     .run(Counter::default());
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
