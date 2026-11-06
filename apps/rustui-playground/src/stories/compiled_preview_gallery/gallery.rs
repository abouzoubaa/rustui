use super::*;

include!("gallery/basic.rs");
include!("gallery/text.rs");
include!("gallery/feedback.rs");
include!("gallery/data.rs");
include!("gallery/graph.rs");
include!("gallery/canvas.rs");
include!("gallery/overlay.rs");
include!("gallery/layout.rs");

include!("gallery/transport.rs");
include!("gallery/mount.rs");
include!("gallery/bindings.rs");
include!("gallery/view.rs");

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
