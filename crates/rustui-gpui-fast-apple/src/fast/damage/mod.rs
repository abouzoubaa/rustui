// Added by RustUI for Stage G; damage rendering remains off by default.
//! GPU damage instrumentation and, in later patches, retained frame planning.

pub(crate) mod frame;
pub(crate) mod inputs;
pub(crate) mod metrics;
pub(crate) mod plan;
pub(crate) mod retained;
pub(crate) mod shaders;

pub(crate) mod partial;

pub(crate) mod verify;

pub(crate) mod cpu;

pub(crate) mod log;

pub(crate) mod encode;

mod diagnosis;

pub(crate) mod timing;

mod order_probe;
