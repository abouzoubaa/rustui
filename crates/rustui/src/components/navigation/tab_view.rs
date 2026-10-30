//! Controlled tab selection and selected-content composition.

use std::collections::HashSet;

use gpui::{
    AnyElement, InteractiveElement as _, IntoElement, ParentElement as _, Role,
    StatefulInteractiveElement as _, Styled as _, div, px,
};
use gpui_component::{
    Sizable as _,
    tab::{Tab as GpuiTab, TabBar},
};
use rustui_core::{Binding, Identity};
use rustui_i18n::LayoutDirection;

use crate::{
    theme::registry::active_theme,
    view::{AnyView, EmptyView, RenderContext, RetentionScope, View},
};

/// A stable tab and its RustUI content.
pub struct TabItem {
    id: String,
    label: String,
    localized_label_key: Option<String>,
    accessibility_label: Option<TabAccessibilityLabel>,
    content: AnyView,
    disabled: bool,
}

enum TabAccessibilityLabel {
    Literal(String),
    Message { id: String, fallback: String },
}

impl TabItem {
    pub fn new<V: View>(id: impl Into<String>, label: impl Into<String>, content: V) -> Self {
        let id = id.into();
        assert!(
            !id.trim().is_empty(),
            "RustUI TabItem requires a non-empty stable ID"
        );
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "RustUI TabItem requires a non-empty visible label"
        );
        Self {
            id,
            label,
            localized_label_key: None,
            accessibility_label: None,
            content: AnyView::new(content),
            disabled: false,
        }
    }

    /// Resolves the visible tab label from the active catalog at render time.
    /// The constructor label remains the fallback and stable identity seed.
    pub fn localized_label(mut self, message_key: impl Into<String>) -> Self {
        let key = message_key.into();
        assert!(
            !key.trim().is_empty(),
            "TabItem label localization key must not be blank"
        );
        self.localized_label_key = Some(key);
        self
    }

    /// Overrides the accessible name without changing the visible tab label.
    pub fn accessibility_label(mut self, label: impl Into<String>) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "RustUI TabItem accessibility label must be non-empty"
        );
        self.accessibility_label = Some(TabAccessibilityLabel::Literal(label));
        self
    }

    /// Resolves the accessible name from the active locale catalog.
    /// An explicit literal name, or the visible tab label, is used as fallback.
    pub fn localized_accessibility_label(mut self, message_id: impl Into<String>) -> Self {
        let id = message_id.into();
        assert!(
            !id.trim().is_empty(),
            "RustUI TabItem accessibility localization key must be non-empty"
        );
        let fallback = match &self.accessibility_label {
            Some(TabAccessibilityLabel::Literal(label)) => label.clone(),
            Some(TabAccessibilityLabel::Message { fallback, .. }) => fallback.clone(),
            None => self.label.clone(),
        };
        self.accessibility_label = Some(TabAccessibilityLabel::Message { id, fallback });
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// Visual style supplied by GPUI Components TabBar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TabViewStyle {
    #[default]
    Automatic,
    Pill,
    Outline,
    Segmented,
    Underline,
}

/// A controlled tab bar that renders the selected RustUI child.
pub struct TabViewControl {
    tabs: Vec<TabItem>,
    selection: Binding<String>,
    id: Option<Identity>,
    style: TabViewStyle,
    accessibility_label: Option<String>,
    accessibility_label_message_id: Option<String>,
}

impl TabViewControl {
    pub fn new(tabs: impl IntoIterator<Item = TabItem>, selection: Binding<String>) -> Self {
        let tabs = tabs.into_iter().collect::<Vec<_>>();
        let mut ids = HashSet::new();
        assert!(
            tabs.iter().all(|tab| ids.insert(tab.id.as_str())),
            "every RustUI TabItem must have a unique ID"
        );

        Self {
            tabs,
            selection,
            id: None,
            style: TabViewStyle::Automatic,
            accessibility_label: None,
            accessibility_label_message_id: None,
        }
    }

    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "TabView identity must not be blank"
        );
        self.id = Some(id);
        self
    }

    pub fn style(mut self, style: TabViewStyle) -> Self {
        self.style = style;
        self
    }

    /// Names the keyboard-focusable tab group for assistive technology.
    pub fn accessibility_label(mut self, label: impl Into<String>) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "RustUI TabView accessibility label must be non-empty"
        );
        self.accessibility_label = Some(label);
        self
    }

    /// Uses a localized message as the accessible name of the tab group.
    /// The literal [`Self::accessibility_label`] override takes precedence.
    pub fn localized_accessibility_label(mut self, message_id: impl Into<String>) -> Self {
        let message_id = message_id.into();
        assert!(
            !message_id.trim().is_empty(),
            "RustUI TabView accessibility label message ID must be non-empty"
        );
        self.accessibility_label_message_id = Some(message_id);
        self
    }

    fn identity_with(&self, allocate: impl FnOnce(Identity) -> Identity) -> Identity {
        self.id.clone().unwrap_or_else(|| {
            allocate(Identity::from(format!(
                "tab-view:{}:{}",
                self.tabs.len(),
                self.tabs.first().map_or("", |tab| tab.id.as_str())
            )))
        })
    }
}

#[allow(non_snake_case)]
pub fn TabView(
    tabs: impl IntoIterator<Item = TabItem>,
    selection: Binding<String>,
) -> TabViewControl {
    TabViewControl::new(tabs, selection)
}

impl View for TabViewControl {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let selected = self.selection.value();
        let selected_index = self.tabs.iter().position(|tab| tab.id == selected);
        let tab_visible_labels = self
            .tabs
            .iter()
            .map(|tab| tab_visible_label(tab, |id| context.localized_message(id)))
            .collect::<Vec<_>>();
        let tab_accessible_names = self
            .tabs
            .iter()
            .map(|tab| tab_accessibility_label(tab, |id| context.localized_message(id)))
            .collect::<Vec<_>>();
        let message_id = self
            .accessibility_label_message_id
            .as_deref()
            .unwrap_or("rustui-navigation-tabs");
        let localized_label = context.localized_message(message_id);
        let localized_label = resolved_localized_group_label(message_id, &localized_label);
        let group_label =
            group_accessibility_label(self.accessibility_label.as_deref(), localized_label);
        let identity = self.identity_with(|base| context.next_implicit_retained_identity(base));
        let group_element_id = format!("rustui-tab-view-group:{}", identity.as_str());
        let direction = context.layout_direction();
        let items = self
            .tabs
            .iter()
            .map(|tab| (tab.id.clone(), tab.disabled))
            .collect::<Vec<_>>();
        let default_roving_id =
            roving_tab_index(&items, &selected).map(|index| items[index].0.clone());
        let roving_identity =
            Identity::from(format!("tab-view-roving-focus:{}", identity.as_str()));
        let (roving_state, _) = context.retained_entity(roving_identity, |_, _| RovingTabState {
            selection: selected.clone(),
            focused_tab: default_roving_id.clone(),
        });
        let prior_roving_id = roving_state.read(context.app()).focused_tab.clone();
        let prior_roving_index = focused_tab_index(&items, prior_roving_id.as_deref(), &selected);
        let tab_focus_handles = self
            .tabs
            .iter()
            .enumerate()
            .map(|(index, tab)| {
                let focus_identity = tab_focus_identity(&identity, &tab.id);
                let (focus_state, _) =
                    context.retained_entity(focus_identity, |_, cx| cx.focus_handle());
                focus_state
                    .read(context.app())
                    .clone()
                    .tab_stop(prior_roving_index == Some(index))
            })
            .collect::<Vec<_>>();
        let selected_for_state = selected.clone();
        let default_for_state = default_roving_id.clone();
        let state_items = items.clone();
        let state_focus_handles = tab_focus_handles.clone();
        context.update_entity(&roving_state, move |state, window, _| {
            let focused_enabled_tab = state
                .focused_tab
                .as_deref()
                .and_then(|id| {
                    state_items
                        .iter()
                        .position(|(item_id, disabled)| !disabled && item_id == id)
                })
                .and_then(|index| state_focus_handles.get(index))
                .is_some_and(|handle| handle.is_focused(window));
            if state.selection != selected_for_state {
                state.selection = selected_for_state;
                if !focused_enabled_tab {
                    state.focused_tab = default_for_state;
                }
            } else if focused_tab_index(
                &state_items,
                state.focused_tab.as_deref(),
                &state.selection,
            )
            .is_none()
            {
                state.focused_tab = default_for_state;
            }
        });
        let roving_tab_id = roving_state.read(context.app()).focused_tab.clone();
        let roving_tab_index = focused_tab_index(&items, roving_tab_id.as_deref(), &selected);
        for (index, focus_handle) in tab_focus_handles.iter().enumerate() {
            focus_handle
                .clone()
                .tab_stop(roving_tab_index == Some(index));
        }
        let click_focus_handles = tab_focus_handles.clone();
        let click_roving_state = roving_state.clone();
        let ids = self
            .tabs
            .iter()
            .map(|tab| tab.id.clone())
            .collect::<Vec<_>>();
        let write = context.binding_writer(self.selection.clone());
        let keyboard_write = context.binding_writer(self.selection.clone());
        let keyboard_roving_state = roving_state.clone();
        let keyboard_focus_handles = tab_focus_handles.clone();

        let tabs = self.tabs.iter().enumerate().map(|(index, tab)| {
            with_tab_set_position(
                GpuiTab::new()
                    .label(tab_visible_labels[index].clone())
                    .aria_label(tab_accessible_names[index].clone())
                    .disabled(tab.disabled)
                    .track_focus(&tab_focus_handles[index])
                    .tab_stop(roving_tab_index == Some(index)),
                index,
                self.tabs.len(),
            )
        });
        let mut tab_bar =
            TabBar::new(identity.to_string())
                .children(tabs)
                .on_click(move |index, window, cx| {
                    if let Some(focus_handle) = click_focus_handles.get(*index) {
                        for (tab_index, focus_handle) in click_focus_handles.iter().enumerate() {
                            focus_handle.clone().tab_stop(tab_index == *index);
                        }
                        focus_handle.focus(window, cx);
                    }
                    if let Some(id) = ids.get(*index) {
                        click_roving_state.update(cx, |state, _| {
                            state.selection = id.clone();
                            state.focused_tab = Some(id.clone());
                        });
                        write(id.clone(), cx);
                    }
                });
        if let Some(index) = selected_index {
            tab_bar = tab_bar.selected_index(index);
        }
        tab_bar = match self.style {
            TabViewStyle::Automatic => tab_bar,
            TabViewStyle::Pill => tab_bar.pill(),
            TabViewStyle::Outline => tab_bar.outline(),
            TabViewStyle::Segmented => tab_bar.segmented(),
            TabViewStyle::Underline => tab_bar.underline(),
        };
        if let Some(theme) = active_theme(context.app()) {
            tab_bar = tab_bar
                .small()
                .h(px(theme.profile.dimensions.control_height as f32));
        }

        context.with_element_scope(RetentionScope::Tabs(identity.clone()), |context| {
            for (index, tab) in self.tabs.iter().enumerate() {
                if Some(index) != selected_index {
                    context.preserve_retention_scope(RetentionScope::Tab(tab.id.clone()));
                }
            }
            let content = selected_index.and_then(|index| {
                self.tabs.get(index).map(|tab| {
                    context.with_element_scope(RetentionScope::Tab(tab.id.clone()), |context| {
                        let content = tab.content.into_element(context);
                        selected_tab_panel_element(
                            &identity,
                            tab,
                            &tab_accessible_names[index],
                            content,
                        )
                        .into_any_element()
                    })
                })
            });

            let mut root = div()
                .id(group_element_id)
                .role(Role::Group)
                .on_key_down(move |event, window, cx| {
                    if !tab_view_keyboard_event_eligible(
                        event.prefer_character_input,
                        event.keystroke.is_ime_in_progress(),
                        event.keystroke.modifiers.modified(),
                    ) {
                        return;
                    }

                    let Some(focused_index) = keyboard_focus_handles
                        .iter()
                        .position(|focus_handle| focus_handle.is_focused(&*window))
                    else {
                        return;
                    };

                    match navigation_key(event.keystroke.key.as_ref(), direction) {
                        Some(
                            command @ (TabViewKey::Previous
                            | TabViewKey::Next
                            | TabViewKey::Home
                            | TabViewKey::End),
                        ) => {
                            if let Some(next_index) =
                                focus_after_key(&items, focused_index, command)
                            {
                                for (index, focus_handle) in
                                    keyboard_focus_handles.iter().enumerate()
                                {
                                    focus_handle.clone().tab_stop(index == next_index);
                                }
                                let next_id = items[next_index].0.clone();
                                keyboard_roving_state.update(cx, |state, _| {
                                    state.focused_tab = Some(next_id);
                                });
                                if let Some(focus_handle) = keyboard_focus_handles.get(next_index) {
                                    focus_handle.focus(&mut *window, cx);
                                }
                                cx.stop_propagation();
                            }
                        }
                        Some(TabViewKey::Activate) => {
                            if let Some(id) = activation_selection(&items, focused_index) {
                                keyboard_write(id, cx);
                                cx.stop_propagation();
                            }
                        }
                        None => {}
                    }
                })
                .child(tab_bar);
            root = root.aria_label(group_label).w_full();
            if let Some(content) = content {
                root = root.child(div().pt_3().child(content));
            }
            root.into_any_element()
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TabViewKey {
    Previous,
    Next,
    Home,
    End,
    Activate,
}

struct RovingTabState {
    selection: String,
    focused_tab: Option<String>,
}

fn group_accessibility_label(custom: Option<&str>, localized: Option<&str>) -> String {
    custom.or(localized).unwrap_or("Tabs").to_owned()
}

fn resolved_localized_group_label<'a>(message_id: &str, localized: &'a str) -> Option<&'a str> {
    (!localized.trim().is_empty() && localized.trim() != message_id.trim()).then_some(localized)
}

fn tab_visible_label(tab: &TabItem, localized_message: impl FnOnce(&str) -> String) -> String {
    tab.localized_label_key.as_ref().map_or_else(
        || tab.label.clone(),
        |key| {
            let localized = localized_message(key);
            if localized.trim().is_empty() || localized.trim() == key.trim() {
                tab.label.clone()
            } else {
                localized
            }
        },
    )
}

fn tab_accessibility_label(
    tab: &TabItem,
    localized_message: impl FnOnce(&str) -> String,
) -> String {
    match &tab.accessibility_label {
        Some(TabAccessibilityLabel::Literal(label)) => label.clone(),
        Some(TabAccessibilityLabel::Message { id, fallback }) => {
            let localized = localized_message(id);
            if localized.trim().is_empty() || localized.trim() == id.trim() {
                fallback.clone()
            } else {
                localized
            }
        }
        None => tab_visible_label(tab, localized_message),
    }
}

fn tab_panel_id(tab_view_identity: &Identity, selected_tab_id: &str) -> String {
    format!(
        "rustui-tab-view-panel:{}:{}:{}:{}",
        tab_view_identity.as_str().len(),
        tab_view_identity.as_str(),
        selected_tab_id.len(),
        selected_tab_id
    )
}

fn tab_focus_identity(tab_view_identity: &Identity, tab_id: &str) -> Identity {
    Identity::from(format!(
        "tab-view-tab-focus:{}:{}:{}:{}",
        tab_view_identity.as_str().len(),
        tab_view_identity.as_str(),
        tab_id.len(),
        tab_id
    ))
}

fn with_tab_set_position<E: gpui::StatefulInteractiveElement>(
    element: E,
    index: usize,
    size: usize,
) -> E {
    element
        .aria_position_in_set(index + 1)
        .aria_size_of_set(size)
}

fn selected_tab_panel_element(
    tab_view_identity: &Identity,
    tab: &TabItem,
    accessible_name: &str,
    content: impl IntoElement,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id(tab_panel_id(tab_view_identity, &tab.id))
        .role(Role::TabPanel)
        .aria_label(accessible_name.to_owned())
        .child(content)
}

fn navigation_key(key: &str, direction: LayoutDirection) -> Option<TabViewKey> {
    match key {
        "left" if direction == LayoutDirection::LeftToRight => Some(TabViewKey::Previous),
        "left" => Some(TabViewKey::Next),
        "right" if direction == LayoutDirection::LeftToRight => Some(TabViewKey::Next),
        "right" => Some(TabViewKey::Previous),
        "home" => Some(TabViewKey::Home),
        "end" => Some(TabViewKey::End),
        "enter" | "space" => Some(TabViewKey::Activate),
        _ => None,
    }
}

fn roving_tab_index(items: &[(String, bool)], selected: &str) -> Option<usize> {
    items
        .iter()
        .position(|(id, disabled)| !disabled && id == selected)
        .or_else(|| items.iter().position(|(_, disabled)| !disabled))
}

fn focused_tab_index(
    items: &[(String, bool)],
    focused_tab: Option<&str>,
    selected: &str,
) -> Option<usize> {
    focused_tab
        .and_then(|id| {
            items
                .iter()
                .position(|(item_id, disabled)| !disabled && item_id == id)
        })
        .or_else(|| roving_tab_index(items, selected))
}

fn tab_view_keyboard_event_eligible(
    prefer_character_input: bool,
    ime_in_progress: bool,
    modified: bool,
) -> bool {
    !prefer_character_input && !ime_in_progress && !modified
}

fn focus_after_key(
    items: &[(String, bool)],
    current_index: usize,
    key: TabViewKey,
) -> Option<usize> {
    let len = items.len();
    if len == 0 || current_index >= len {
        return None;
    }
    let start = match key {
        TabViewKey::Previous => (current_index + len - 1) % len,
        TabViewKey::Next => (current_index + 1) % len,
        TabViewKey::Home => 0,
        TabViewKey::End => len - 1,
        TabViewKey::Activate => return None,
    };

    for offset in 0..len {
        let index = match key {
            TabViewKey::Previous => (start + len - offset) % len,
            TabViewKey::Next => (start + offset) % len,
            TabViewKey::Home => offset,
            TabViewKey::End => len - 1 - offset,
            TabViewKey::Activate => unreachable!(),
        };
        if !items[index].1 {
            return Some(index);
        }
    }

    None
}

fn activation_selection(items: &[(String, bool)], focused_index: usize) -> Option<String> {
    items
        .get(focused_index)
        .filter(|(_, disabled)| !disabled)
        .map(|(id, _)| id.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[gpui::test]
    fn native_keyboard_skips_disabled_tabs_and_commits_only_on_activation(
        cx: &mut gpui::TestAppContext,
    ) {
        use crate::runtime::ComponentHost;

        struct Owner {
            selection: String,
        }
        impl View for Owner {
            fn body(&self) -> impl View {
                TabView(
                    [
                        TabItem::new("first", "First", EmptyView),
                        TabItem::new("disabled", "Disabled", EmptyView).disabled(true),
                        TabItem::new("third", "Third", EmptyView),
                    ],
                    Binding::new(
                        self.selection.clone(),
                        |owner: &Owner| owner.selection.clone(),
                        |owner, value| owner.selection = value,
                    ),
                )
                .id("keyboard-tabs")
            }
        }
        let theme = crate::theme::parse_theme(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../themes/macos27.json"
        )))
        .unwrap();
        cx.update(|app| {
            gpui_component::init(app);
            crate::theme::activate_theme(&theme, Some(crate::theme::AppearanceMode::Dark), app)
                .unwrap();
        });
        let (owner, window) = cx.add_window_view(|_, cx| {
            ComponentHost::new(
                Owner {
                    selection: "first".into(),
                },
                cx,
            )
        });
        window.update(|window, app| {
            let _ = window.draw(app);
            window.focus_next(app);
            let _ = window.draw(app);
        });
        window.simulate_keystrokes("right");
        window.update(|window, app| {
            let _ = window.draw(app);
            assert_eq!(owner.read(app).component().selection, "first");
        });
        window.simulate_keystrokes("enter");
        window.update(|window, app| {
            let _ = window.draw(app);
            assert_eq!(owner.read(app).component().selection, "third");
        });
        window.simulate_keystrokes("right");
        window.simulate_keystrokes("space");
        window.update(|window, app| {
            let _ = window.draw(app);
            assert_eq!(owner.read(app).component().selection, "first");
        });
    }

    #[test]
    fn tab_accessibility_metadata_reports_one_based_position_and_full_set_size() {
        use gpui::Element as _;

        let tab = with_tab_set_position(
            div()
                .id("settings-tab")
                .role(Role::Tab)
                .aria_label("Settings"),
            1,
            3,
        );
        assert_eq!(tab.a11y_role(), Some(Role::Tab));

        let mut node = gpui::accesskit::Node::new(Role::Tab);
        tab.write_a11y_info(&mut node);
        assert_eq!(node.position_in_set(), Some(2));
        assert_eq!(node.size_of_set(), Some(3));
    }

    #[test]
    fn selected_tab_panel_serializes_its_role_name_and_identity() {
        use gpui::Element as _;

        let identity = Identity::from("settings-view");
        let tab = TabItem::new("activity", "Activity", EmptyView)
            .accessibility_label("Recent activity")
            .localized_accessibility_label("tab.activity");
        let accessible_name = tab_accessibility_label(&tab, |_| "Activité récente".to_owned());
        let panel = selected_tab_panel_element(&identity, &tab, &accessible_name, div());

        assert_eq!(panel.a11y_role(), Some(Role::TabPanel));
        assert!(gpui::Element::id(&panel).is_some());

        let mut node = gpui::accesskit::Node::new(Role::TabPanel);
        panel.write_a11y_info(&mut node);
        assert_eq!(node.label(), Some("Activité récente"));
    }

    #[test]
    fn implicit_tab_view_identity_distinguishes_siblings_and_is_stable_by_slot() {
        let selection = Binding::new(
            String::from("overview"),
            |value: &String| value.clone(),
            |value: &mut String, next| *value = next,
        );
        let tabs = [TabItem::new("overview", "Overview", EmptyView)];
        let tab_view = TabViewControl::new(tabs, selection);
        let allocate_at_slot = |slot| {
            move |base: Identity| Identity::from(format!("owner:implicit-retained:{slot}:{base}"))
        };

        let first = tab_view.identity_with(allocate_at_slot(0));
        let second = tab_view.identity_with(allocate_at_slot(1));
        let first_after_rerender = tab_view.identity_with(allocate_at_slot(0));

        assert_ne!(first, second);
        assert_eq!(first, first_after_rerender);
    }

    fn selection_binding() -> Binding<String> {
        Binding::new(
            String::new(),
            |value: &String| value.clone(),
            |value, next| *value = next,
        )
    }

    #[test]
    #[should_panic(expected = "TabView identity must not be blank")]
    fn rejects_blank_explicit_identity() {
        TabViewControl::new(Vec::<TabItem>::new(), selection_binding()).id(" \t ");
    }

    #[test]
    fn preserves_nonblank_explicit_identity() {
        let tabs = TabViewControl::new(Vec::<TabItem>::new(), selection_binding()).id(" tabs ");

        assert_eq!(
            tabs.identity_with(|_| panic!("explicit identity must bypass fallback allocation"))
                .as_str(),
            " tabs "
        );
    }

    fn keyboard_items() -> Vec<(String, bool)> {
        vec![
            ("overview".into(), false),
            ("activity".into(), true),
            ("settings".into(), false),
        ]
    }

    #[test]
    fn tab_group_name_is_stable_or_uses_the_callers_label() {
        assert_eq!(group_accessibility_label(None, None), "Tabs");
        assert_eq!(group_accessibility_label(None, Some("Onglets")), "Onglets");
        assert_eq!(
            group_accessibility_label(
                None,
                resolved_localized_group_label("tabs.accessibility", " \ttabs.accessibility\n"),
            ),
            "Tabs"
        );
        assert_eq!(
            resolved_localized_group_label("tabs.accessibility", " Onglets "),
            Some(" Onglets ")
        );
        assert_eq!(
            group_accessibility_label(Some("Workspace tabs"), Some("Onglets")),
            "Workspace tabs"
        );
    }

    #[test]
    fn tab_accessible_name_uses_the_visible_label_or_explicit_override() {
        let visible = TabItem::new("settings", "Settings", EmptyView);
        assert_eq!(
            tab_accessibility_label(&visible, |_| unreachable!()),
            "Settings"
        );

        let overridden = TabItem::new("settings", "Settings", EmptyView)
            .accessibility_label("Workspace preferences");
        assert_eq!(
            tab_accessibility_label(&overridden, |_| unreachable!()),
            "Workspace preferences"
        );
    }

    #[test]
    fn localized_tab_accessible_name_uses_catalog_and_fallbacks() {
        let visible = TabItem::new("settings", "Settings", EmptyView)
            .localized_accessibility_label("tab.settings");
        assert_eq!(
            tab_accessibility_label(&visible, |id| format!("{id} traduit")),
            "tab.settings traduit"
        );
        assert_eq!(
            tab_accessibility_label(&visible, |id| id.to_owned()),
            "Settings"
        );
        assert_eq!(
            tab_accessibility_label(&visible, |_| " \ttab.settings\n".to_owned()),
            "Settings"
        );
        assert_eq!(
            tab_accessibility_label(&visible, |_| " \t ".to_owned()),
            "Settings"
        );

        let explicit = TabItem::new("settings", "Settings", EmptyView)
            .accessibility_label("Workspace preferences")
            .localized_accessibility_label("tab.settings");
        assert_eq!(
            tab_accessibility_label(&explicit, |id| id.to_owned()),
            "Workspace preferences"
        );
        assert_eq!(
            tab_accessibility_label(&explicit, |_| "Préférences".to_owned()),
            "Préférences"
        );

        let literal_after_message = explicit.accessibility_label("Literal name");
        assert_eq!(
            tab_accessibility_label(&literal_after_message, |_| unreachable!()),
            "Literal name"
        );
    }

    #[test]
    #[should_panic(expected = "TabItem accessibility localization key must be non-empty")]
    fn rejects_blank_localized_tab_accessible_name_key() {
        TabItem::new("settings", "Settings", EmptyView).localized_accessibility_label(" \t ");
    }

    #[test]
    fn selected_tab_panel_identity_is_stable_and_scoped_to_both_ids() {
        let view = Identity::from("settings-view");
        let expected = "rustui-tab-view-panel:13:settings-view:8:activity";

        assert_eq!(tab_panel_id(&view, "activity"), expected);
        assert_eq!(tab_panel_id(&view, "activity"), expected);
        assert_ne!(
            tab_panel_id(&view, "activity"),
            tab_panel_id(&view, "overview")
        );
        assert_ne!(
            tab_panel_id(&view, "activity"),
            tab_panel_id(&Identity::from("other-view"), "activity")
        );
    }

    #[test]
    fn tab_focus_identity_disambiguates_delimited_components() {
        let first = tab_focus_identity(&Identity::from("settings:activity"), "other");
        let second = tab_focus_identity(&Identity::from("settings"), "activity:other");

        assert_ne!(first, second);
        assert_eq!(
            first,
            tab_focus_identity(&Identity::from("settings:activity"), "other")
        );
    }

    #[test]
    fn selected_tab_panel_name_uses_the_selected_tabs_accessible_name() {
        let default = TabItem::new("settings", "Settings", EmptyView);
        assert_eq!(
            tab_accessibility_label(&default, |_| unreachable!()),
            "Settings"
        );

        let overridden = TabItem::new("settings", "Settings", EmptyView)
            .accessibility_label("Workspace preferences");
        assert_eq!(
            tab_accessibility_label(&overridden, |_| unreachable!()),
            String::from("Workspace preferences")
        );
        let localized = overridden.localized_accessibility_label("tab.settings");
        assert_eq!(
            tab_accessibility_label(&localized, |id| format!("{id} traduit")),
            "tab.settings traduit"
        );
    }

    #[test]
    fn localized_visible_tab_label_updates_default_accessible_name() {
        let tab = TabItem::new("colors", "Colors", EmptyView).localized_label("tab.colors");
        assert_eq!(
            tab_visible_label(&tab, |_| "Couleurs".to_owned()),
            "Couleurs"
        );
        assert_eq!(
            tab_accessibility_label(&tab, |_| "ألوان".to_owned()),
            "ألوان"
        );
        assert_eq!(
            tab_visible_label(&tab, |_| "tab.colors".to_owned()),
            "Colors"
        );
    }

    #[test]
    fn arrow_navigation_moves_focus_wraps_and_skips_disabled_tabs() {
        let items = keyboard_items();
        assert_eq!(focus_after_key(&items, 0, TabViewKey::Next), Some(2));
        assert_eq!(focus_after_key(&items, 2, TabViewKey::Next), Some(0));
        assert_eq!(focus_after_key(&items, 0, TabViewKey::Previous), Some(2));
    }

    #[test]
    fn keyboard_event_eligibility_rejects_text_ime_and_modified_input() {
        assert!(tab_view_keyboard_event_eligible(false, false, false));
        assert!(!tab_view_keyboard_event_eligible(true, false, false));
        assert!(!tab_view_keyboard_event_eligible(false, true, false));
        assert!(!tab_view_keyboard_event_eligible(false, false, true));
    }

    #[test]
    fn home_end_and_invalid_selection_choose_enabled_edges() {
        let items = keyboard_items();
        assert_eq!(roving_tab_index(&items, "unknown"), Some(0));
        assert_eq!(roving_tab_index(&items, "activity"), Some(0));
        assert_eq!(focus_after_key(&items, 2, TabViewKey::Home), Some(0));
        assert_eq!(focus_after_key(&items, 0, TabViewKey::End), Some(2));
    }

    #[test]
    fn roving_focus_stays_independent_from_selection_and_recovers_invalid_focus() {
        let items = keyboard_items();
        assert_eq!(
            focused_tab_index(&items, Some("settings"), "overview"),
            Some(2)
        );
        assert_eq!(
            focused_tab_index(&items, Some("activity"), "settings"),
            Some(2)
        );
        assert_eq!(
            focused_tab_index(&items, Some("removed"), "settings"),
            Some(2)
        );
    }

    #[test]
    fn manual_activation_selects_focused_enabled_tab_only() {
        let items = keyboard_items();
        assert_eq!(activation_selection(&items, 2), Some("settings".into()));
        assert_eq!(activation_selection(&items, 1), None);
        assert_eq!(activation_selection(&items, 3), None);
        assert_eq!(
            navigation_key("enter", LayoutDirection::LeftToRight),
            Some(TabViewKey::Activate)
        );
        assert_eq!(
            navigation_key("space", LayoutDirection::LeftToRight),
            Some(TabViewKey::Activate)
        );
    }

    #[test]
    fn rtl_reverses_horizontal_navigation_and_empty_or_disabled_tabs_are_safe() {
        assert_eq!(
            navigation_key("left", LayoutDirection::RightToLeft),
            Some(TabViewKey::Next)
        );
        assert_eq!(
            navigation_key("right", LayoutDirection::RightToLeft),
            Some(TabViewKey::Previous)
        );
        let all_disabled = vec![("activity".into(), true)];
        assert_eq!(roving_tab_index(&all_disabled, "activity"), None);
        assert_eq!(focus_after_key(&[], 0, TabViewKey::End), None);
        assert_eq!(
            focus_after_key(&keyboard_items(), 9, TabViewKey::Next),
            None
        );
        assert_eq!(navigation_key("tab", LayoutDirection::LeftToRight), None);
    }

    #[test]
    fn navigation_has_no_target_when_every_tab_is_disabled() {
        let all_disabled = vec![("overview".into(), true), ("settings".into(), true)];

        assert_eq!(focus_after_key(&all_disabled, 0, TabViewKey::Next), None);
        assert_eq!(focus_after_key(&all_disabled, 1, TabViewKey::Home), None);
    }

    #[test]
    #[should_panic(expected = "RustUI TabItem requires a non-empty visible label")]
    fn blank_tab_label_is_rejected() {
        TabItem::new("settings", " \t ", EmptyView);
    }

    #[test]
    #[should_panic(expected = "RustUI TabItem requires a non-empty visible label")]
    fn empty_tab_label_is_rejected() {
        TabItem::new("settings", "", EmptyView);
    }

    #[test]
    #[should_panic(expected = "RustUI TabItem accessibility label must be non-empty")]
    fn blank_tab_accessibility_label_is_rejected() {
        TabItem::new("settings", "Settings", EmptyView).accessibility_label(" \t ");
    }

    #[test]
    #[should_panic(expected = "RustUI TabItem requires a non-empty stable ID")]
    fn blank_tab_id_is_rejected() {
        TabItem::new(" \t ", "Settings", EmptyView);
    }

    #[test]
    #[should_panic(expected = "RustUI TabItem requires a non-empty stable ID")]
    fn empty_tab_id_is_rejected() {
        TabItem::new("", "Settings", EmptyView);
    }
}
