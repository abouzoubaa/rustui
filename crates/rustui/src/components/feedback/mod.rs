//! Feedback adapters backed by GPUI Components and GPUI primitives.
//!
//! Progress, Spinner, Skeleton, EmptyState, and ErrorState are implemented.
//! Toast is implemented separately in [`crate::components::overlay::toast`].

pub mod empty_state;
pub mod error_state;
pub mod progress;
pub mod skeleton;
pub mod spinner;
