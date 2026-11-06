//! A keyed, size-constrained panel hosted by [`crate::layout::SplitView`].
//!
//! This is the same panel contract as `SplitPanel`: GPUI owns resizing and the
//! enclosing SplitView owns retained sizes and child identity. It is a panel
//! descriptor, not a standalone view or an additional resize runtime.

pub use crate::layout::split_view::SplitPanel as ResizablePanel;
