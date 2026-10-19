//! Navigation components built on the pinned GPUI Components controls.

pub mod breadcrumb;
pub mod disclosure;
pub mod navigation_split;
pub mod navigation_stack;
pub mod sidebar;
pub mod tab_view;
pub mod toolbar;
pub use breadcrumb::{Breadcrumb, BreadcrumbItem, BreadcrumbView};
pub use disclosure::{Disclosure, DisclosureView};
pub use navigation_split::{NavigationSplit, NavigationSplitItem, NavigationSplitView};
pub use navigation_stack::{
    NavigationDestination, NavigationLink, NavigationLinkView, NavigationStack, NavigationStackView,
};
pub use sidebar::{Sidebar, SidebarItem, SidebarView};
pub use tab_view::{TabItem, TabView, TabViewControl, TabViewStyle};
pub use toolbar::{Toolbar, ToolbarItem, ToolbarView};
