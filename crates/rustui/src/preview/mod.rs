//! Versioned descriptors for explicitly registered, compiled RustUI components.
//!
//! The preview codec serializes declared inputs and action references only. It
//! does not serialize Rust views, component state, callbacks, or GPUI entities.

mod codec;
mod composition;
mod host;
mod model;
mod registry;
mod schema;

pub use codec::{decode_catalog, decode_preview, encode_catalog, encode_preview};
pub use composition::{
    COMPOSITION_FORMAT, COMPOSITION_FORMAT_VERSION, CompositionDocument, CompositionEdit,
    CompositionHost, CompositionNode, CompositionNodeSnapshot, CompositionSignal, CompositionSlots,
    CompositionStamp, MAX_COMPOSITION_DEPTH, MAX_COMPOSITION_EDITS, MAX_COMPOSITION_NODES,
    MAX_SLOTS_PER_COMPONENT, NodeId, SlotId, SlotSchema, decode_composition, encode_composition,
};
pub use model::{
    ActionRef, ComponentCatalog, ComponentId, ComponentSchema, EventId, EventSchema,
    PreviewDocument, PreviewError, PreviewId, PropertyId, PropertyKind, PropertySchema,
    PropertyValue, ValidatedPreview,
};
pub use schema::{validate_catalog, validate_preview};

pub use host::{PreviewHost, PreviewSignal, PreviewSnapshot, PreviewStamp};
pub use registry::PreviewRegistry;
