//! Navigation components built on the pinned GPUI Components controls.

fn resolved_message_label(
    localized: String,
    message_id: &str,
    fallback: &gpui::SharedString,
) -> gpui::SharedString {
    if localized.trim().is_empty() || localized.trim() == message_id.trim() {
        fallback.clone()
    } else {
        gpui::SharedString::from(localized)
    }
}

#[cfg(test)]
mod label_tests {
    use super::resolved_message_label;

    #[test]
    fn unavailable_destination_keeps_a_readable_fallback() {
        let key = "rustui-navigation-unavailable-destination";
        let fallback = "Destination unavailable".into();
        assert_eq!(resolved_message_label(key.into(), key, &fallback), fallback);
        assert_eq!(
            resolved_message_label("  ".into(), key, &fallback),
            fallback
        );
        assert_eq!(
            resolved_message_label("Destination indisponible".into(), key, &fallback),
            "Destination indisponible"
        );
    }
}

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
