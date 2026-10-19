//! runtime integration.
//!
//! Application hosting, control ownership, and cancellable owner tasks use GPUI.
//! Other runtime facilities are documented in their respective modules.

pub mod application;
pub mod component_host;
pub mod events;
pub mod invalidation;
pub mod render_diagnostics;
pub(crate) mod scoped_element;
pub mod tasks;
pub mod window;

pub use application::RustUI;
pub use component_host::ComponentHost;
pub use component_host::ComponentHost as ViewHost;
pub use events::AppAction;
pub use rustui_core::{
    Action, AsyncState, Binding, EnvironmentValues, Identity, Invalidation, State,
};
pub use tasks::OwnerTask;
