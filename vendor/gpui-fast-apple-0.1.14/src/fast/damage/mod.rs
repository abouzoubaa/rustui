// Added by RustUI for Stage G; damage rendering remains off by default.
//! GPU damage instrumentation and, in later patches, retained frame planning.

pub(crate) mod metrics;
pub(crate) mod shaders;
pub(crate) mod plan;
pub(crate) mod inputs;
pub(crate) mod frame;
pub(crate) mod retained;

pub(crate) mod partial;

pub(crate) mod verify;

pub(crate) mod cpu;

pub(crate) mod log;
