//! GPUI-aligned state, identity, binding, and environment contracts.

pub mod actions;
pub use actions::Action;
pub mod async_state;
pub use async_state::AsyncState;
pub mod binding;
pub mod component;
pub mod component_host;
pub mod environment;
pub mod identity;
pub mod observation;
#[doc(hidden)]
pub use component as state;
#[doc(hidden)]
pub use component_host as lifecycle;

pub use binding::Binding;
pub use component::State;
pub use environment::EnvironmentValues;
pub use identity::Identity;
pub use observation::Invalidation;
