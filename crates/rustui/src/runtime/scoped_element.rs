//! GPUI identity boundary for a retained RustUI scope.

use std::panic::Location;

use gpui::{
    AnyElement, App, Bounds, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement,
    LayoutId, Pixels, Role, Window, accesskit,
};

use crate::view::RetentionScope;

struct ScopedElement {
    id: ElementId,
    child: AnyElement,
    semantics: Option<SemanticInfo>,
}

pub(crate) struct SemanticInfo {
    pub(crate) role: Role,
    pub(crate) label: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) live_value: Option<(String, accesskit::Live)>,
}

pub(crate) fn scoped_element(scope: &RetentionScope, child: AnyElement) -> AnyElement {
    ScopedElement {
        id: scope_id(scope),
        child,
        semantics: None,
    }
    .into_any_element()
}

pub(crate) fn semantic_scoped_element(
    scope: &RetentionScope,
    child: AnyElement,
    semantics: Option<SemanticInfo>,
) -> AnyElement {
    ScopedElement {
        id: scope_id(scope),
        child,
        semantics,
    }
    .into_any_element()
}

fn scope_id(scope: &RetentionScope) -> ElementId {
    let id = match scope {
        RetentionScope::Key(key) => segment_id("key", key.as_str()),
        RetentionScope::Popover(id) => segment_id("popover", id.as_str()),
        RetentionScope::ContextMenu(id) => segment_id("context-menu", id.as_str()),
        RetentionScope::SplitView(id) => segment_id("split-view", id.as_str()),
        RetentionScope::Semantics(id) => segment_id("semantics", id.as_str()),
        RetentionScope::Focus(id) => segment_id("focus", id.as_str()),
        RetentionScope::Keyboard(id) => segment_id("keyboard", id.as_str()),
        RetentionScope::Hover(id) => segment_id("hover", id.as_str()),
        RetentionScope::Tap(id) => segment_id("tap", id.as_str()),
        RetentionScope::PropertyInspector(id) => segment_id("property-inspector", id.as_str()),
        RetentionScope::LazyGrid(id) => segment_id("lazy-grid", id.as_str()),
        RetentionScope::Collection(id) => segment_id("collection", id.as_str()),
        RetentionScope::GeometryReader(id) => segment_id("geometry-reader", id.as_str()),
        RetentionScope::ScrollView(id) => segment_id("scroll-view", id.as_str()),
        RetentionScope::Background(id) => segment_id("background", id.as_str()),
        RetentionScope::Border(id) => segment_id("border", id.as_str()),
        RetentionScope::Shadow(id) => segment_id("shadow", id.as_str()),
        RetentionScope::Card(id) => segment_id("card", id.as_str()),
        RetentionScope::DragSource(id) => segment_id("drag-source", id.as_str()),
        RetentionScope::DropTarget(id) => segment_id("drop-target", id.as_str()),
        RetentionScope::ScrollViewContent => "rustui:scroll-view-content".to_owned(),
        RetentionScope::SpringOpacity(id) => segment_id("spring-opacity", id.as_str()),
        RetentionScope::SpringOffset(id) => segment_id("spring-offset", id.as_str()),
        RetentionScope::AnimatedOpacity(id) => segment_id("animated-opacity", id.as_str()),
        RetentionScope::Tabs(id) => segment_id("tabs", id.as_str()),
        RetentionScope::Tab(id) => segment_id("tab", id),
        RetentionScope::Navigation(id) => segment_id("navigation", id.as_str()),
        RetentionScope::NavigationSplit(id) => segment_id("navigation-split", id.as_str()),
        RetentionScope::Page(path) => {
            let mut id = format!("rustui:page:{}:", path.len());
            for segment in path {
                id.push_str(&format!("{}:{segment}", segment.len()));
            }
            id
        }
    };
    id.into()
}

fn segment_id(kind: &str, value: &str) -> String {
    format!("rustui:{kind}:{}:{value}", value.len())
}

#[cfg(test)]
mod scope_tests {
    use super::{ScopedElement, SemanticInfo, scope_id, semantic_scoped_element};
    use crate::view::RetentionScope;
    use gpui::{
        Element, IntoElement, Role,
        accesskit::{Live, Node},
    };
    use rustui_core::Identity;

    #[test]
    fn decoration_families_have_distinct_native_namespaces() {
        let id = Identity::from("same");
        let scopes = [
            RetentionScope::Background(id.clone()),
            RetentionScope::Border(id.clone()),
            RetentionScope::Shadow(id.clone()),
            RetentionScope::Card(id.clone()),
            RetentionScope::Tap(id.clone()),
            RetentionScope::PropertyInspector(id.clone()),
            RetentionScope::DragSource(id.clone()),
            RetentionScope::DropTarget(id.clone()),
            RetentionScope::Key(id),
        ];
        for (i, scope) in scopes.iter().enumerate() {
            for other in &scopes[i + 1..] {
                assert_ne!(scope_id(scope), scope_id(other));
            }
        }
    }

    #[test]
    fn scroll_view_has_a_distinct_native_namespace() {
        let id = Identity::from("same");
        let scroll = scope_id(&RetentionScope::ScrollView(id.clone()));
        for scope in [
            RetentionScope::Key(id.clone()),
            RetentionScope::GeometryReader(id.clone()),
            RetentionScope::Keyboard(id.clone()),
            RetentionScope::AnimatedOpacity(id),
        ] {
            assert_ne!(scroll, scope_id(&scope));
        }
    }

    #[test]
    fn geometry_has_a_distinct_native_namespace() {
        let id = Identity::from("same");
        let geometry = scope_id(&RetentionScope::GeometryReader(id.clone()));
        for scope in [
            RetentionScope::Key(id.clone()),
            RetentionScope::LazyGrid(id.clone()),
            RetentionScope::Keyboard(id.clone()),
            RetentionScope::Focus(id),
        ] {
            assert_ne!(geometry, scope_id(&scope));
        }
    }

    #[test]
    fn opacity_wrappers_have_distinct_native_namespaces() {
        let id = Identity::from("same");
        let spring = scope_id(&RetentionScope::SpringOpacity(id.clone()));
        let animated = scope_id(&RetentionScope::AnimatedOpacity(id.clone()));
        assert_ne!(spring, animated);
        assert_ne!(spring, scope_id(&RetentionScope::Key(id.clone())));
        assert_ne!(animated, scope_id(&RetentionScope::Key(id)));
    }

    #[test]
    fn context_menu_has_a_distinct_native_namespace() {
        let id = Identity::from("same");
        let context_menu = scope_id(&RetentionScope::ContextMenu(id.clone()));
        assert_ne!(context_menu, scope_id(&RetentionScope::Key(id.clone())));
        assert_ne!(context_menu, scope_id(&RetentionScope::Popover(id)));
    }

    #[test]
    fn split_view_has_a_distinct_native_namespace() {
        let id = Identity::from("same");
        let split = scope_id(&RetentionScope::SplitView(id.clone()));
        assert_ne!(split, scope_id(&RetentionScope::Key(id.clone())));
        assert_ne!(split, scope_id(&RetentionScope::Popover(id.clone())));
        assert_ne!(split, scope_id(&RetentionScope::ContextMenu(id.clone())));
        assert_ne!(split, scope_id(&RetentionScope::Tabs(id.clone())));
        assert_ne!(split, scope_id(&RetentionScope::Navigation(id)));
    }

    #[test]
    fn navigation_split_has_a_distinct_native_namespace() {
        let id = Identity::from("same");
        let navigation_split = scope_id(&RetentionScope::NavigationSplit(id.clone()));
        assert_ne!(navigation_split, scope_id(&RetentionScope::Key(id.clone())));
        assert_ne!(
            navigation_split,
            scope_id(&RetentionScope::SplitView(id.clone()))
        );
        assert_ne!(navigation_split, scope_id(&RetentionScope::Navigation(id)));
    }

    #[test]
    fn semantics_has_a_distinct_native_namespace() {
        let id = Identity::from("same");
        let semantics = scope_id(&RetentionScope::Semantics(id.clone()));
        assert_ne!(semantics, scope_id(&RetentionScope::Key(id.clone())));
        assert_ne!(semantics, scope_id(&RetentionScope::SplitView(id)));
    }

    #[test]
    fn focus_has_a_distinct_native_namespace() {
        let id = Identity::from("same");
        let focus = scope_id(&RetentionScope::Focus(id.clone()));
        assert_ne!(focus, scope_id(&RetentionScope::Key(id.clone())));
        assert_ne!(focus, scope_id(&RetentionScope::Semantics(id)));
    }

    #[test]
    fn semantic_node_writes_role_label_and_description() {
        let mut scoped = ScopedElement {
            id: scope_id(&RetentionScope::Semantics(Identity::from("group"))),
            child: gpui::div().into_any_element(),
            semantics: Some(SemanticInfo {
                role: Role::Region,
                label: Some("Settings".into()),
                description: Some("Change preferences".into()),
                live_value: None,
            }),
        };
        assert_eq!(scoped.a11y_role(), Some(Role::Region));
        let mut node = Node::new(scoped.a11y_role().unwrap());
        scoped.write_a11y_info(&mut node);
        assert_eq!(node.label(), Some("Settings"));
        assert_eq!(node.description(), Some("Change preferences"));

        scoped.semantics = Some(SemanticInfo {
            role: Role::Status,
            label: None,
            description: None,
            live_value: Some(("3 files saved".into(), Live::Polite)),
        });
        let mut node = Node::new(scoped.a11y_role().unwrap());
        scoped.write_a11y_info(&mut node);
        assert_eq!(node.value(), Some("3 files saved"));
        assert_eq!(node.live(), Some(Live::Polite));

        scoped.semantics = None;
        assert_eq!(scoped.a11y_role(), None);
    }

    #[test]
    fn semantic_scoped_element_serializes_navigation_accessible_name() {
        use gpui::Element as _;

        let mut element = semantic_scoped_element(
            &RetentionScope::Semantics(Identity::from("navigation")),
            gpui::div().into_any_element(),
            Some(SemanticInfo {
                role: Role::Navigation,
                label: Some("Primary navigation".into()),
                description: None,
                live_value: None,
            }),
        );
        let scoped = element
            .downcast_mut::<ScopedElement>()
            .expect("semantic wrapper remains the element stored by GPUI");

        let role = scoped
            .a11y_role()
            .expect("named Navigation semantics must expose an accessibility node");
        assert_eq!(role, Role::Navigation);
        let mut node = Node::new(role);
        scoped.write_a11y_info(&mut node);
        assert_eq!(node.label(), Some("Primary navigation"));
    }
}

impl IntoElement for ScopedElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for ScopedElement {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn a11y_role(&self) -> Option<Role> {
        self.semantics.as_ref().map(|info| info.role)
    }

    fn write_a11y_info(&self, node: &mut accesskit::Node) {
        if let Some(info) = &self.semantics {
            if let Some(label) = &info.label {
                node.set_label(label.clone());
            }
            if let Some(description) = &info.description {
                node.set_description(description.clone());
            }
            if let Some((value, politeness)) = &info.live_value {
                node.set_value(value.clone());
                node.set_live(*politeness);
            }
        }
    }

    fn source_location(&self) -> Option<&'static Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        (self.child.request_layout(window, cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.child.prepaint(window, cx);
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        _: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.child.paint(window, cx);
    }
}
