use super::*;

mod basic;
mod bindings;
mod canvas;
mod data;
mod feedback;
mod graph;
mod layout;
mod mount;
mod overlay;
mod text;
mod transport;
mod view;

pub(super) use basic::register_basic_specimens;
pub(super) use canvas::register_canvas_specimens;
pub(super) use data::register_data_specimens;
pub(super) use feedback::register_feedback_specimens;
pub(super) use graph::register_graph_specimens;
pub(super) use layout::register_layout_specimens;
pub(super) use overlay::register_overlay_specimens;
pub(super) use text::register_text_specimens;

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
