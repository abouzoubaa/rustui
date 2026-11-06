//! Sidebar selection and detail composition over the native split layout.

use std::{collections::HashSet, rc::Rc};

use crate::components::basic::button::{Button, ButtonStyle};
use crate::layout::{GeometryReader, GeometrySize, SplitPanel, SplitView};
use crate::localization::MessageArguments;
use crate::runtime::AppAction;
use gpui::{AnyElement, IntoElement as _, ParentElement as _, SharedString, Styled as _, div};
use rustui_core::{Binding, Identity};

use crate::{
    accessibility::semantics::{Semantics, SemanticsRole},
    components::navigation::sidebar::{Sidebar, SidebarItem},
    view::{AnyView, EmptyView, RenderContext, RetentionScope, View},
};

#[derive(Clone)]
enum SplitLabel {
    Literal(SharedString),
    Message {
        id: SharedString,
        fallback: SharedString,
    },
}

fn resolved_message_label(
    localized: String,
    message_id: &str,
    fallback: &SharedString,
) -> SharedString {
    if localized.trim().is_empty() || localized.trim() == message_id.trim() {
        fallback.clone()
    } else {
        SharedString::from(localized)
    }
}

fn resolved_navigation_tree_label(
    localized: String,
    message_id: &str,
    fallback: &SharedString,
) -> SharedString {
    if localized.trim().is_empty() || localized.trim() == message_id.trim() {
        fallback.clone()
    } else {
        SharedString::from(localized)
    }
}

/// A stable sidebar destination and its detail view.
#[derive(Clone)]
pub struct NavigationSplitItem {
    id: String,
    label: SplitLabel,
    content: Rc<AnyView>,
    disabled: bool,
}

impl NavigationSplitItem {
    /// Creates a destination. IDs must be nonempty and unique in the split view.
    pub fn new<V: View>(id: impl Into<String>, label: impl Into<SharedString>, content: V) -> Self {
        let id = id.into();
        let label = label.into();
        assert!(
            !id.trim().is_empty(),
            "NavigationSplit item IDs must be nonempty"
        );
        assert!(
            !label.trim().is_empty(),
            "NavigationSplit item labels must be nonempty"
        );
        Self {
            id,
            label: SplitLabel::Literal(label),
            content: Rc::new(AnyView::new(content)),
            disabled: false,
        }
    }

    /// Keeps this destination visible but prevents selecting it from the sidebar.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Resolves the sidebar label from the active RustUI message catalog.
    pub fn localized(mut self, message_id: impl Into<SharedString>) -> Self {
        let message_id = message_id.into();
        assert!(
            !message_id.trim().is_empty(),
            "NavigationSplit item localization keys must be nonempty"
        );
        let fallback = match &self.label {
            SplitLabel::Literal(label)
            | SplitLabel::Message {
                fallback: label, ..
            } => label.clone(),
        };
        self.label = SplitLabel::Message {
            id: message_id,
            fallback,
        };
        self
    }
}

fn validate_items(items: Vec<NavigationSplitItem>) -> Vec<NavigationSplitItem> {
    let mut ids = HashSet::with_capacity(items.len());
    for item in &items {
        assert!(
            !item.id.trim().is_empty(),
            "NavigationSplit item IDs must be nonempty"
        );
        assert!(
            ids.insert(item.id.as_str()),
            "duplicate NavigationSplit item ID: {}",
            item.id
        );
    }
    items
}

fn selected_index(items: &[NavigationSplitItem], selection: Option<&str>) -> Option<usize> {
    let selection = selection?;
    items.iter().position(|item| item.id == selection)
}

fn detail_accessibility_label(label: &str) -> SharedString {
    SharedString::from(label)
}

fn detail_semantics_identity(view_id: &Identity, route: &str) -> Identity {
    let view_id = view_id.as_str();
    Identity::from(format!(
        "navigation-split-detail:{}:{view_id}:{}:{route}",
        view_id.len(),
        route.len()
    ))
}

struct NavigationSplitDetail {
    view_id: Identity,
    route: Option<(String, SharedString, Rc<AnyView>)>,
}

struct SharedAnyView(Rc<AnyView>);

impl View for SharedAnyView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        self.0.into_element(context)
    }
}

impl View for NavigationSplitDetail {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        match &self.route {
            Some((route, label, content)) => {
                let detail = Semantics(
                    detail_semantics_identity(&self.view_id, route),
                    SharedAnyView(Rc::clone(content)),
                )
                .role(SemanticsRole::Region)
                // Reuse the destination's resolved label so localized rows
                // do not acquire an English-only "details" suffix.
                .label(detail_accessibility_label(label));
                context.with_element_scope(RetentionScope::Page(vec![route.clone()]), |context| {
                    detail.into_element(context)
                })
            }
            None => EmptyView.into_element(context),
        }
    }
}

/// A controlled sidebar and selected detail pane.
///
/// The caller owns selection and bound application values. Only the selected
/// destination is lowered; switching away releases its retained native controls.
/// At widths below 560 logical pixels, this view shows either the destination
/// list or the selected detail in one column. Like [`SplitView`], this view
/// requires a parent with bounded width and height. Use `Frame` or `Container`
/// to set an explicit viewport inside an intrinsic stack.
#[derive(Clone)]
pub struct NavigationSplitView {
    id: Identity,
    items: Vec<NavigationSplitItem>,
    selection: Binding<Option<String>>,
    accessibility_label: Option<SplitLabel>,
}

struct NavigationSplitNarrowState {
    last_selection: Option<String>,
    show_detail: bool,
}

// Back must focus the retained tree container after exposing the list again.
fn return_to_narrow_list(state: &mut NavigationSplitNarrowState) -> bool {
    state.show_detail = false;
    true
}

fn open_narrow_detail(state: &mut NavigationSplitNarrowState, selection: Option<String>) {
    state.last_selection = selection;
    state.show_detail = true;
}

impl NavigationSplitNarrowState {
    fn new(selection: Option<String>, has_selected_route: bool) -> Self {
        Self {
            last_selection: selection,
            show_detail: has_selected_route,
        }
    }
}

fn update_narrow_state(
    state: &mut NavigationSplitNarrowState,
    selection: Option<String>,
    has_selected_route: bool,
) -> bool {
    if state.last_selection != selection {
        state.show_detail = has_selected_route;
    }
    if !has_selected_route {
        state.show_detail = false;
    }
    state.last_selection = selection;
    state.show_detail
}

const NARROW_BREAKPOINT: f32 = 560.0;

fn is_narrow_width(width: f32) -> bool {
    width < NARROW_BREAKPOINT
}

struct NavigationSplitComposition {
    view: NavigationSplitView,
    size: GeometrySize,
    show_detail: bool,
    narrow_state: gpui::Entity<NavigationSplitNarrowState>,
}

impl View for NavigationSplitComposition {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let selected = self.view.selection.value();
        let sidebar_items = self.view.items.iter().map(|item| {
            let (label, message_id) = match &item.label {
                SplitLabel::Literal(label) => (label.clone(), None),
                SplitLabel::Message { id, fallback } => (fallback.clone(), Some(id.clone())),
            };
            let sidebar_item = SidebarItem::new(item.id.clone(), label).disabled(item.disabled);
            match message_id {
                None => sidebar_item,
                Some(message_id) => sidebar_item.localized(message_id),
            }
        });
        let sidebar = Sidebar(
            format!("{}:sidebar", self.view.id.as_str()),
            sidebar_items,
            self.view.selection.clone(),
        );
        let sidebar = match &self.view.accessibility_label {
            Some(SplitLabel::Literal(label)) => sidebar.accessibility_label(label.clone()),
            Some(SplitLabel::Message { id, fallback }) => {
                sidebar.accessibility_label(resolved_navigation_tree_label(
                    context.localized_message(id.as_ref()),
                    id.as_ref(),
                    fallback,
                ))
            }
            None => sidebar,
        };
        let route = selected
            .as_deref()
            .and_then(|id| selected_index(&self.view.items, Some(id)))
            .map(|index| {
                let item = &self.view.items[index];
                let label = match &item.label {
                    SplitLabel::Literal(label) => label.clone(),
                    SplitLabel::Message { id, fallback } => resolved_message_label(
                        context.localized_message(id.as_ref()),
                        id.as_ref(),
                        fallback,
                    ),
                };
                (item.id.clone(), label, Rc::clone(&item.content))
            });

        if is_narrow_width(self.size.width) {
            let split_scope = RetentionScope::SplitView(Identity::from(format!(
                "{}:layout",
                self.view.id.as_str()
            )));
            let sidebar_scope = RetentionScope::Key(Identity::from("sidebar"));

            if self.show_detail {
                if let Some((route_id, label, content)) = route {
                    let state = self.narrow_state.clone();
                    let selection_refresh =
                        context.binding_updater(self.view.selection.clone(), |_| true);
                    return context.with_retention_scope(split_scope, |context| {
                        context.preserve_retention_scope(sidebar_scope.clone());
                        let sidebar_focus = context
                            .with_retention_scope(sidebar_scope.clone(), |context| {
                                sidebar.focus_handle(context)
                            });
                        let back = Button(
                            context.localized_message("rustui-navigation-back"),
                            AppAction::with_window_action(move |_, window, cx| {
                                let restore_focus = state.update(cx, |state, cx| {
                                    let restore_focus = return_to_narrow_list(state);
                                    cx.notify();
                                    restore_focus
                                });
                                if restore_focus {
                                    sidebar_focus.focus(window, cx);
                                }
                                selection_refresh(cx);
                            }),
                        )
                        .id(format!("{}:back", self.view.id.as_str()))
                        .button_style(ButtonStyle::Plain);
                        let detail = context.with_element_scope(
                            RetentionScope::Key(Identity::from("detail")),
                            |context| {
                                NavigationSplitDetail {
                                    view_id: self.view.id.clone(),
                                    route: Some((route_id, label, content)),
                                }
                                .into_element(context)
                            },
                        );
                        div()
                            .flex()
                            .flex_col()
                            .size_full()
                            .child(back.into_element(context))
                            .child(detail)
                            .into_any_element()
                    });
                }
            }

            return context.with_retention_scope(split_scope, |context| {
                let mut list = div().flex().flex_col().size_full();
                if let Some((route_id, label, _content)) = route {
                    let state = self.narrow_state.clone();
                    let selection_refresh =
                        context.binding_updater(self.view.selection.clone(), |_| true);
                    let action_fallback = SharedString::from("Open selected destination");
                    let open_label = context
                        .locale()
                        .map(|_| {
                            context.localized_message("rustui-navigation-open-selected-destination")
                        })
                        .filter(|message| message != "rustui-navigation-open-selected-destination")
                        .map(|message| {
                            resolved_message_label(
                                message,
                                "rustui-navigation-open-selected-destination",
                                &action_fallback,
                            )
                        })
                        .unwrap_or_else(|| action_fallback.clone());
                    let open_accessibility_label = context
                        .locale()
                        .and_then(|_| {
                            context
                                .localized_message_with_arguments(
                                    "rustui-navigation-open-destination",
                                    &MessageArguments::new().with("destination", label.as_ref()),
                                )
                                .ok()
                        })
                        .filter(|message| message != "rustui-navigation-open-destination")
                        .map(|message| {
                            resolved_message_label(
                                message,
                                "rustui-navigation-open-destination",
                                &open_label,
                            )
                        })
                        .unwrap_or_else(|| open_label.clone());
                    let open = Button(
                        open_label,
                        AppAction::new(move |cx| {
                            state.update(cx, |state, cx| {
                                open_narrow_detail(state, Some(route_id.clone()));
                                cx.notify();
                            });
                            selection_refresh(cx);
                        }),
                    )
                    .id(format!("{}:open-selected", self.view.id.as_str()))
                    .accessibility_label(open_accessibility_label);
                    list = list.child(open.into_element(context));
                }
                let sidebar = context
                    .with_element_scope(sidebar_scope, |context| sidebar.into_element(context));
                list.child(sidebar).into_any_element()
            });
        }

        let detail = NavigationSplitDetail {
            view_id: self.view.id.clone(),
            route,
        };
        let layout = SplitView::horizontal(format!("{}:layout", self.view.id.as_str()))
            .panel(
                SplitPanel::new("sidebar", sidebar)
                    .initial_size(240.0)
                    .size_range(140.0..480.0),
            )
            .panel(SplitPanel::new("detail", detail));
        layout.into_element(context)
    }
}

impl NavigationSplitView {
    pub fn new(
        id: impl Into<Identity>,
        items: impl IntoIterator<Item = NavigationSplitItem>,
        selection: Binding<Option<String>>,
    ) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "NavigationSplit ID must be nonempty"
        );
        Self {
            id,
            items: validate_items(items.into_iter().collect()),
            selection,
            accessibility_label: Some(SplitLabel::Message {
                id: "rustui-navigation-tree".into(),
                fallback: "Navigation".into(),
            }),
        }
    }

    /// Names the navigation tree for assistive technology.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "NavigationSplit accessibility labels must be nonempty"
        );
        self.accessibility_label = Some(SplitLabel::Literal(label));
        self
    }

    /// Resolves the navigation tree name from the active RustUI message catalog.
    pub fn localized_accessibility_label(mut self, message_id: impl Into<SharedString>) -> Self {
        let message_id = message_id.into();
        assert!(
            !message_id.trim().is_empty(),
            "NavigationSplit accessibility localization keys must be nonempty"
        );
        let fallback = match &self.accessibility_label {
            Some(SplitLabel::Literal(label))
            | Some(SplitLabel::Message {
                fallback: label, ..
            }) => label.clone(),
            None => SharedString::from("Navigation"),
        };
        self.accessibility_label = Some(SplitLabel::Message {
            id: message_id,
            fallback,
        });
        self
    }
}

#[allow(non_snake_case)]
pub fn NavigationSplit(
    id: impl Into<Identity>,
    items: impl IntoIterator<Item = NavigationSplitItem>,
    selection: Binding<Option<String>>,
) -> NavigationSplitView {
    NavigationSplitView::new(id, items, selection)
}

impl View for NavigationSplitView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let selected = self.selection.value();
        let has_selected_route = selected
            .as_deref()
            .and_then(|id| selected_index(&self.items, Some(id)))
            .is_some();
        let view = self.clone();
        context.with_element_scope(
            RetentionScope::NavigationSplit(self.id.clone()),
            |context| {
                let (narrow_state, _) = context
                    .retained_entity(Identity::from("narrow-presentation"), |_, _| {
                        NavigationSplitNarrowState::new(selected.clone(), has_selected_route)
                    });
                let show_detail = context.update_entity(&narrow_state, |state, _, _| {
                    update_narrow_state(state, selected, has_selected_route)
                });
                GeometryReader::new(format!("{}:responsive", self.id.as_str()), move |size| {
                    NavigationSplitComposition {
                        view: view.clone(),
                        size,
                        show_detail,
                        narrow_state: narrow_state.clone(),
                    }
                })
                .into_element(context)
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{
        NARROW_BREAKPOINT, NavigationSplitItem, NavigationSplitNarrowState, NavigationSplitView,
        detail_accessibility_label, detail_semantics_identity, is_narrow_width,
        open_narrow_detail, resolved_message_label, resolved_navigation_tree_label,
        return_to_narrow_list, selected_index, update_narrow_state, validate_items,
    };
    use crate::view::EmptyView;
    use rustui_core::{Binding, Identity};

    struct Owner(Option<String>);

    fn read_selection(owner: &Owner) -> Option<String> {
        owner.0.clone()
    }

    fn write_selection(owner: &mut Owner, selection: Option<String>) {
        owner.0 = selection;
    }

    fn item(id: &str) -> NavigationSplitItem {
        NavigationSplitItem::new(id, id, EmptyView)
    }

    #[test]
    fn defaults_to_localized_navigation_tree_name_and_can_be_overridden() {
        let selection = Binding::new(None, read_selection, write_selection);
        let view = NavigationSplitView::new("main", [item("home")], selection);
        assert!(matches!(
            view.accessibility_label.as_ref(),
            Some(super::SplitLabel::Message { id, fallback })
                if id == "rustui-navigation-tree" && fallback == "Navigation"
        ));

        let view = view.accessibility_label("Main navigation");
        assert!(matches!(
            view.accessibility_label,
            Some(super::SplitLabel::Literal(label)) if label.as_ref() == "Main navigation"
        ));
    }

    #[test]
    fn unavailable_navigation_tree_message_uses_literal_fallback() {
        assert_eq!(
            resolved_navigation_tree_label(
                "rustui-navigation-tree".into(),
                "rustui-navigation-tree",
                &"Navigation".into(),
            ),
            "Navigation"
        );
    }

    #[test]
    fn localized_accessibility_label_replaces_literal_label() {
        let selection = Binding::new(None, read_selection, write_selection);
        let view = NavigationSplitView::new("main", [item("home")], selection)
            .accessibility_label("Main navigation")
            .localized_accessibility_label("navigation.main");
        assert!(matches!(
            view.accessibility_label,
            Some(super::SplitLabel::Message { id, fallback })
                if id.as_ref() == "navigation.main" && fallback == "Main navigation"
        ));
    }

    #[test]
    fn localized_destination_label_is_used_for_its_detail_region() {
        assert_eq!(detail_accessibility_label("Fichiers").as_ref(), "Fichiers");
    }

    #[test]
    fn detail_semantics_identity_is_unique_per_split_instance() {
        let first = detail_semantics_identity(&Identity::from("left"), "files");
        let second = detail_semantics_identity(&Identity::from("right"), "files");
        let delimiter_left = detail_semantics_identity(&Identity::from("a:b"), "c");
        let delimiter_right = detail_semantics_identity(&Identity::from("a"), "b:c");

        assert_ne!(first, second);
        assert_ne!(delimiter_left, delimiter_right);
        assert_eq!(first.as_str(), "navigation-split-detail:4:left:5:files");
    }

    #[test]
    fn unresolved_destination_label_uses_its_visible_fallback() {
        assert_eq!(
            resolved_message_label(
                "navigation.files".into(),
                "navigation.files",
                &"Files".into(),
            ),
            "Files"
        );
    }

    #[test]
    fn selection_tracks_stable_ids_and_missing_selection_is_empty() {
        let before = vec![item("files"), item("settings")];
        let after = vec![item("settings"), item("files")];

        assert_eq!(selected_index(&before, Some("settings")), Some(1));
        assert_eq!(selected_index(&after, Some("settings")), Some(0));
        assert_eq!(selected_index(&after, None), None);
        assert_eq!(selected_index(&after, Some("missing")), None);
    }

    #[test]
    fn narrow_presentation_defaults_and_transitions_preserve_controlled_selection() {
        let mut state = NavigationSplitNarrowState::new(Some("files".into()), true);
        assert!(
            state.show_detail,
            "an initially selected route opens in narrow mode"
        );

        assert!(return_to_narrow_list(&mut state));
        assert!(!state.show_detail);
        assert_eq!(state.last_selection.as_deref(), Some("files"));

        open_narrow_detail(&mut state, Some("files".into()));
        assert!(
            state.show_detail,
            "the selected route can be reopened from the list"
        );

        assert!(update_narrow_state(
            &mut state,
            Some("settings".into()),
            true
        ));
        assert!(update_narrow_state(&mut state, Some("missing".into()), false) == false);
        assert_eq!(state.last_selection.as_deref(), Some("missing"));
        assert!(!update_narrow_state(&mut state, None, false));
        assert_eq!(state.last_selection, None);
    }

    #[test]
    fn back_requests_sidebar_focus_restoration_and_preserves_controlled_selection() {
        let mut state = NavigationSplitNarrowState::new(Some("files".into()), true);

        assert!(return_to_narrow_list(&mut state));
        assert!(!state.show_detail);
        assert_eq!(state.last_selection.as_deref(), Some("files"));
    }

    #[test]
    fn narrow_breakpoint_is_strict_and_uses_logical_pixels() {
        assert!(is_narrow_width(NARROW_BREAKPOINT - 0.01));
        assert!(!is_narrow_width(NARROW_BREAKPOINT));
        assert!(!is_narrow_width(NARROW_BREAKPOINT + 0.01));
    }

    #[test]
    fn rejects_empty_and_duplicate_item_ids() {
        assert!(std::panic::catch_unwind(|| item(" ")).is_err());
        assert!(
            std::panic::catch_unwind(|| validate_items(vec![item("files"), item("files")]))
                .is_err()
        );
    }

    #[test]
    fn rejects_blank_visible_and_accessibility_labels_and_localization_keys() {
        assert!(
            std::panic::catch_unwind(|| { NavigationSplitItem::new("home", " \t ", EmptyView) })
                .is_err()
        );
        assert!(std::panic::catch_unwind(|| item("home").localized(" \t ")).is_err());

        assert!(
            std::panic::catch_unwind(|| {
                NavigationSplitView::new(
                    "main",
                    [item("home")],
                    Binding::new(None, read_selection, write_selection),
                )
                .accessibility_label(" \t ")
            })
            .is_err()
        );
        assert!(
            std::panic::catch_unwind(|| {
                NavigationSplitView::new(
                    "main",
                    [item("home")],
                    Binding::new(None, read_selection, write_selection),
                )
                .localized_accessibility_label(" \t ")
            })
            .is_err()
        );
    }

    #[test]
    fn blank_localized_labels_keep_visible_fallbacks() {
        let destination =
            NavigationSplitItem::new("home", "Home", EmptyView).localized("navigation.home");
        assert!(matches!(
            destination.label,
            super::SplitLabel::Message { ref fallback, .. } if fallback == "Home"
        ));
        assert_eq!(
            resolved_message_label(" \t\n".into(), "navigation.home", &"Home".into()),
            "Home"
        );
        assert_eq!(
            resolved_message_label("Accueil".into(), "navigation.home", &"Home".into()),
            "Accueil"
        );

        let tree_name = NavigationSplitView::new(
            "main",
            [item("home")],
            Binding::new(None, read_selection, write_selection),
        )
        .localized_accessibility_label("navigation.main");
        assert!(matches!(
            tree_name.accessibility_label,
            Some(super::SplitLabel::Message { ref fallback, .. }) if fallback == "Navigation"
        ));
    }
}
