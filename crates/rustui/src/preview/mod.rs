//! Versioned descriptors for explicitly registered, compiled RustUI components.
//!
//! The preview codec serializes declared inputs and action references only. It
//! does not serialize Rust views, component state, callbacks, or GPUI entities.

mod codec;
mod host;
mod model;
mod registry;
mod schema;

pub use codec::{decode_catalog, decode_preview, encode_catalog, encode_preview};
pub use model::{
    ActionRef, ComponentCatalog, ComponentId, ComponentSchema, EventId, EventSchema,
    PreviewDocument, PreviewError, PreviewId, PropertyId, PropertyKind, PropertySchema,
    PropertyValue, ValidatedPreview,
};
pub use schema::{validate_catalog, validate_preview};

pub use host::{PreviewHost, PreviewSignal, PreviewSnapshot, PreviewStamp};
pub use registry::PreviewRegistry;
