//! Controlled flat sidebar selection over GPUI Components.

use std::collections::HashSet;

use gpui::{
    AnyElement, App, ElementId, FocusHandle, InteractiveElement as _, IntoElement,
    ParentElement as _, Role, SharedString, StatefulInteractiveElement as _, Styled as _, Window,
    div, px,
};
use gpui_component::{
    Collapsible,
    sidebar::{
        Sidebar as GpuiSidebar, SidebarItem as GpuiSidebarItem,
        SidebarMenuItem as GpuiSidebarMenuItem,
    },
};
use rustui_core::{Binding, Identity};

use crate::{
    theme::registry::active_theme,
    view::{EmptyView, RenderContext, View},
};

#[derive(Clone)]
enum SidebarLabel {
    Literal(SharedString),
    Message {
        id: SharedString,
        fallback: SharedString,
    },
}

#[derive(Clone)]
enum SidebarAccessibilityLabel {
    Literal(SharedString),
    Message {
        id: SharedString,
        fallback: SharedString,
    },
}

#[derive(Clone)]
struct StableSidebarMenu {
    sidebar_id: String,
    items: Vec<(String, GpuiSidebarMenuItem)>,
}

impl Collapsible for StableSidebarMenu {
    fn collapsed(self, _collapsed: bool) -> Self {
        self
    }

    fn is_collapsed(&self) -> bool {
        false
    }
}

impl GpuiSidebarItem for StableSidebarMenu {
    fn render(
        self,
        id: impl Into<ElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> impl IntoElement {
        let mut menu = div().id(id.into()).flex().flex_col().gap_2();
        for (item_id, item) in self.items {
            menu = menu.child(item.render(
                sidebar_item_element_id(&self.sidebar_id, &item_id),
                window,
                cx,
            ));
        }
        menu
    }
}

fn require_nonempty_name(name: SharedString, description: &str) -> SharedString {
    assert!(!name.trim().is_empty(), "{description} must be nonempty");
    name
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

fn resolved_accessibility_label(
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

/// One flat sidebar destination with caller-owned stable identity.
pub struct SidebarItem {
    id: String,
    label: SidebarLabel,
    disabled: bool,
}

impl SidebarItem {
    /// Creates a destination. IDs must be nonempty and unique in the sidebar.
    pub fn new(id: impl Into<String>, label: impl Into<SharedString>) -> Self {
        let id = id.into();
        let label = require_nonempty_name(label.into(), "Sidebar item label");
        assert!(!id.trim().is_empty(), "Sidebar item IDs must be nonempty");
        Self {
            id,
            label: SidebarLabel::Literal(label),
            disabled: false,
        }
    }

    /// Prevents this destination from changing the bound selection.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Resolves the visible label from the active RustUI message catalog.
    pub fn localized(mut self, message_id: impl Into<SharedString>) -> Self {
        let message_id = require_nonempty_name(message_id.into(), "Sidebar item message ID");
        let fallback = match &self.label {
            SidebarLabel::Literal(label)
            | SidebarLabel::Message {
                fallback: label, ..
            } => label.clone(),
        };
        self.label = SidebarLabel::Message {
            id: message_id,
            fallback,
        };
        self
    }
}

fn validate_items(items: Vec<SidebarItem>) -> Vec<SidebarItem> {
    let mut ids = HashSet::with_capacity(items.len());
    for item in &items {
        assert!(
            !item.id.trim().is_empty(),
            "Sidebar item IDs must be nonempty"
        );
        assert!(
            ids.insert(item.id.as_str()),
            "duplicate Sidebar item ID: {}",
            item.id
        );
    }
    items
}

fn selected_index(items: &[SidebarItem], selected_id: Option<&str>) -> Option<usize> {
    let selected_id = selected_id?;
    items.iter().position(|item| item.id == selected_id)
}

fn sidebar_item_element_id(sidebar_id: &str, item_id: &str) -> String {
    format!(
        "rustui-sidebar-item:{}:{}:{}:{}",
        sidebar_id.len(),
        sidebar_id,
        item_id.len(),
        item_id
    )
}

/// A flat, controlled sidebar. Selection is a caller-owned stable item ID.
pub struct SidebarView {
    id: Identity,
    items: Vec<SidebarItem>,
    selection: Binding<Option<String>>,
    accessibility_label: Option<SidebarAccessibilityLabel>,
}

impl SidebarView {
    pub fn new(
        id: impl Into<Identity>,
        items: impl IntoIterator<Item = SidebarItem>,
        selection: Binding<Option<String>>,
    ) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "Sidebar ID must be nonempty"
        );
        Self {
            id,
            items: validate_items(items.into_iter().collect()),
            selection,
            accessibility_label: None,
        }
    }

    /// Names the navigation tree for assistive technology.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(SidebarAccessibilityLabel::Literal(require_nonempty_name(
            label.into(),
            "Sidebar accessibility label",
        )));
        self
    }

    /// Resolves the navigation tree name from the active RustUI message catalog.
    pub fn localized_accessibility_label(mut self, message_id: impl Into<SharedString>) -> Self {
        let id = require_nonempty_name(message_id.into(), "Sidebar accessibility localization key");
        let fallback = match &self.accessibility_label {
            Some(SidebarAccessibilityLabel::Literal(label))
            | Some(SidebarAccessibilityLabel::Message {
                fallback: label, ..
            }) => label.clone(),
            None => SharedString::from("Navigation"),
        };
        self.accessibility_label = Some(SidebarAccessibilityLabel::Message { id, fallback });
        self
    }

    pub(crate) fn focus_handle(&self, context: &mut RenderContext<'_>) -> FocusHandle {
        let focus_identity = Identity::from(format!("sidebar-focus:{}", self.id.as_str()));
        let (focus_state, _) = context.retained_entity(focus_identity, |_, cx| cx.focus_handle());
        focus_state.read(context.app()).clone().tab_stop(true)
    }
}

#[allow(non_snake_case)]
pub fn Sidebar(
    id: impl Into<Identity>,
    items: impl IntoIterator<Item = SidebarItem>,
    selection: Binding<Option<String>>,
) -> SidebarView {
    SidebarView::new(id, items, selection)
}

impl View for SidebarView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let selected = selected_index(&self.items, self.selection.value().as_deref());
        let write_selection = context.binding_writer(self.selection.clone());
        let focus_handle = self.focus_handle(context);
        let keyboard_focus = focus_handle.clone();
        let rows: Vec<_> = self
            .items
            .iter()
            .map(|item| (item.id.clone(), item.disabled))
            .collect();
        let navigation_actions: Vec<_> = [
            ("up", SidebarKey::Up),
            ("down", SidebarKey::Down),
            ("home", SidebarKey::Home),
            ("end", SidebarKey::End),
            ("enter", SidebarKey::Enter),
        ]
        .into_iter()
        .map(|(key, command)| {
            let rows = rows.clone();
            let update = context.binding_updater(self.selection.clone(), move |current| {
                let next = selection_after_key(&rows, current.as_deref(), command);
                if !should_update_selection(&rows, current, &next, command) {
                    return false;
                }
                *current = next;
                true
            });
            (key, command, update)
        })
        .collect();
        let items: Vec<_> = self
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let label = match &item.label {
                    SidebarLabel::Literal(label) => label.clone(),
                    SidebarLabel::Message { id, fallback } => resolved_message_label(
                        context.localized_message(id.as_ref()),
                        id.as_ref(),
                        fallback,
                    ),
                };
                let stable_id = item.id.clone();
                let id = stable_id.clone();
                let disabled = item.disabled;
                let write_selection = write_selection.clone();
                let focus_handle = focus_handle.clone();

                let row = GpuiSidebarMenuItem::new(label)
                    .active(selected == Some(index))
                    .disable(disabled)
                    .on_click(move |_, window, cx| {
                        focus_handle.focus(window, cx);
                        // Keep the owner guard even though upstream also disables the row.
                        if !disabled {
                            write_selection(Some(id.clone()), cx);
                        }
                    });
                (stable_id, row)
            })
            .collect();
        let menu = StableSidebarMenu {
            sidebar_id: self.id.to_string(),
            items,
        };

        let mut focus_scope = div()
            .id(format!("rustui-sidebar-focus:{}", self.id.as_str()))
            .size_full()
            .track_focus(&focus_handle)
            .role(Role::Tree)
            .on_key_down(move |event, window, cx| {
                if !sidebar_keyboard_event_eligible(
                    keyboard_focus.is_focused(window),
                    event.prefer_character_input,
                    event.keystroke.is_ime_in_progress(),
                    event.keystroke.modifiers.modified(),
                ) {
                    return;
                }
                let Some((_, command, action)) = navigation_actions
                    .iter()
                    .find(|(key, _, _)| event.keystroke.key == *key)
                else {
                    return;
                };
                action(cx);
                if should_consume_sidebar_key(&rows, *command) {
                    cx.stop_propagation();
                }
            });
        let label = match &self.accessibility_label {
            Some(SidebarAccessibilityLabel::Literal(label)) => label.clone(),
            Some(SidebarAccessibilityLabel::Message { id, fallback }) => {
                resolved_accessibility_label(
                    context.localized_message(id.as_ref()),
                    id.as_ref(),
                    fallback,
                )
            }
            None => resolved_accessibility_label(
                context.localized_message("rustui-navigation-tree"),
                "rustui-navigation-tree",
                &"Navigation".into(),
            ),
        };
        focus_scope = focus_scope.aria_label(label);

        let mut sidebar = GpuiSidebar::new(self.id.to_string())
            .collapsible(false)
            .child(menu);
        if let Some(theme) = active_theme(context.app()) {
            sidebar = sidebar.w(px(theme.profile.dimensions.sidebar_width as f32));
        } else {
            sidebar = sidebar.w_full();
        }
        focus_scope.child(sidebar).into_any_element()
    }
}

#[derive(Clone, Copy)]
enum SidebarKey {
    Up,
    Down,
    Home,
    End,
    Enter,
}

fn sidebar_keyboard_event_eligible(
    focused: bool,
    prefer_character_input: bool,
    ime_in_progress: bool,
    modified: bool,
) -> bool {
    focused && !prefer_character_input && !ime_in_progress && !modified
}

fn should_update_selection(
    rows: &[(String, bool)],
    current: &Option<String>,
    next: &Option<String>,
    key: SidebarKey,
) -> bool {
    current != next
        || (matches!(key, SidebarKey::Enter)
            && current.as_deref().is_some_and(|selected| {
                rows.iter()
                    .any(|(id, disabled)| id == selected && !disabled)
            }))
}

fn should_consume_sidebar_key(rows: &[(String, bool)], key: SidebarKey) -> bool {
    matches!(key, SidebarKey::Enter) || rows.iter().any(|(_, disabled)| !disabled)
}

fn selection_after_key(
    rows: &[(String, bool)],
    current: Option<&str>,
    key: SidebarKey,
) -> Option<String> {
    let first_enabled = || rows.iter().position(|(_, disabled)| !disabled);
    let last_enabled = || rows.iter().rposition(|(_, disabled)| !disabled);
    let current_index = rows.iter().position(|(id, _)| Some(id.as_str()) == current);

    let next = match key {
        SidebarKey::Up => match current_index {
            Some(index) => rows[..index]
                .iter()
                .rposition(|(_, disabled)| !disabled)
                .or_else(|| {
                    rows.get(index)
                        .is_some_and(|(_, disabled)| *disabled)
                        .then(first_enabled)
                        .flatten()
                })
                .unwrap_or(index),
            None => match last_enabled() {
                Some(index) => index,
                None => return current.map(str::to_owned),
            },
        },
        SidebarKey::Down => match current_index {
            Some(index) => rows[index + 1..]
                .iter()
                .position(|(_, disabled)| !disabled)
                .map(|offset| index + 1 + offset)
                .or_else(|| {
                    rows.get(index)
                        .is_some_and(|(_, disabled)| *disabled)
                        .then(last_enabled)
                        .flatten()
                })
                .unwrap_or(index),
            None => match first_enabled() {
                Some(index) => index,
                None => return current.map(str::to_owned),
            },
        },
        SidebarKey::Home => match first_enabled() {
            Some(index) => index,
            None => return current.map(str::to_owned),
        },
        SidebarKey::End => match last_enabled() {
            Some(index) => index,
            None => return current.map(str::to_owned),
        },
        SidebarKey::Enter => return current.map(str::to_owned),
    };

    Some(rows[next].0.clone())
}

#[cfg(test)]
mod tests {
    use super::{
        SidebarAccessibilityLabel, SidebarItem, SidebarKey, SidebarLabel, SidebarView,
        resolved_accessibility_label, resolved_message_label, selected_index, selection_after_key,
        should_consume_sidebar_key, should_update_selection, sidebar_item_element_id,
        sidebar_keyboard_event_eligible, validate_items,
    };
    use rustui_core::Binding;

    struct Owner;

    fn read_selection(_: &Owner) -> Option<String> {
        None
    }

    fn write_selection(_: &mut Owner, _: Option<String>) {}

    fn item(id: &str, label: &str) -> SidebarItem {
        SidebarItem::new(id, label)
    }

    #[test]
    fn sidebar_keyboard_navigation_ignores_ime_candidate_keys() {
        assert!(sidebar_keyboard_event_eligible(true, false, false, false));
        assert!(!sidebar_keyboard_event_eligible(true, false, true, false));
        assert!(!sidebar_keyboard_event_eligible(true, true, false, false));
        assert!(!sidebar_keyboard_event_eligible(true, false, false, true));
        assert!(!sidebar_keyboard_event_eligible(false, false, false, false));
    }

    #[test]
    fn rejects_empty_and_duplicate_ids() {
        assert!(std::panic::catch_unwind(|| item(" ", "Blank")).is_err());
        assert!(
            std::panic::catch_unwind(|| validate_items(vec![
                item("home", "Home"),
                item("home", "Start")
            ]))
            .is_err()
        );
    }

    #[test]
    fn retained_item_identity_tracks_view_and_item_ids_across_reordering() {
        let first = sidebar_item_element_id("sidebar:one", "item:a");
        let second = sidebar_item_element_id("sidebar:one", "item:b");
        let same_item_in_other_sidebar = sidebar_item_element_id("sidebar", "one:item:a");

        assert_eq!(first, sidebar_item_element_id("sidebar:one", "item:a"));
        assert_ne!(first, second);
        assert_ne!(first, same_item_in_other_sidebar);
    }

    #[test]
    fn rejects_blank_visible_and_localized_item_names() {
        assert!(std::panic::catch_unwind(|| item("home", " \t")).is_err());
        assert!(std::panic::catch_unwind(|| item("home", "Home").localized(" \n")).is_err());
        assert!(matches!(
            item("home", " Home ").label,
            SidebarLabel::Literal(label) if label.as_ref() == " Home "
        ));
    }

    #[test]
    fn blank_localized_item_name_falls_back_to_its_visible_label() {
        let item = item("home", "Home").localized("navigation.home");
        let SidebarLabel::Message { id, fallback } = item.label else {
            panic!("localized item should retain its message ID and fallback");
        };

        assert_eq!(id.as_ref(), "navigation.home");
        assert_eq!(
            resolved_message_label(" \t\n".into(), "navigation.home", &fallback),
            "Home"
        );
        assert_eq!(
            resolved_message_label("navigation.home".into(), "navigation.home", &fallback),
            "Home"
        );
        assert_eq!(
            resolved_message_label("Accueil".into(), "navigation.home", &fallback),
            "Accueil"
        );
    }

    #[test]
    #[should_panic(expected = "Sidebar accessibility label must be nonempty")]
    fn rejects_blank_navigation_tree_accessibility_name() {
        let selection = Binding::new(None, read_selection, write_selection);
        let view = SidebarView::new("main", [item("home", "Home")], selection);
        let _ = view.accessibility_label("  ");
    }

    #[test]
    fn preserves_nonblank_navigation_tree_accessibility_name() {
        let selection = Binding::new(None, read_selection, write_selection);
        let view = SidebarView::new("main", [item("home", "Home")], selection);
        assert!(matches!(
            view.accessibility_label(" Main navigation ").accessibility_label,
            Some(SidebarAccessibilityLabel::Literal(label)) if label.as_ref() == " Main navigation "
        ));
    }

    #[test]
    fn localized_navigation_tree_accessibility_name_keeps_key_and_fallback() {
        let selection = Binding::new(None, read_selection, write_selection);
        let view = SidebarView::new("main", [item("home", "Home")], selection)
            .localized_accessibility_label("navigation.custom-tree");
        assert!(matches!(
            view.accessibility_label,
            Some(SidebarAccessibilityLabel::Message { id, fallback })
                if id.as_ref() == "navigation.custom-tree" && fallback == "Navigation"
        ));
    }

    #[test]
    fn unavailable_navigation_tree_message_uses_literal_fallback() {
        assert_eq!(
            resolved_accessibility_label(
                "rustui-navigation-tree".into(),
                "rustui-navigation-tree",
                &"Navigation".into(),
            ),
            "Navigation"
        );
    }

    #[test]
    fn selection_tracks_semantic_id_across_order_and_labels() {
        let before = vec![item("home", "Home"), item("settings", "Settings")];
        let after = vec![item("settings", "Preferences"), item("home", "Start")];

        assert_eq!(selected_index(&before, Some("settings")), Some(1));
        assert_eq!(selected_index(&after, Some("settings")), Some(0));
        assert_eq!(selected_index(&after, Some("missing")), None);
        assert_eq!(selected_index(&after, None), None);
    }

    #[test]
    fn arrows_skip_disabled_rows_and_stop_at_the_ends() {
        let rows = vec![
            ("first".into(), false),
            ("disabled".into(), true),
            ("last".into(), false),
        ];

        assert_eq!(
            selection_after_key(&rows, Some("first"), SidebarKey::Down).as_deref(),
            Some("last")
        );
        assert_eq!(
            selection_after_key(&rows, Some("last"), SidebarKey::Up).as_deref(),
            Some("first")
        );
        assert_eq!(
            selection_after_key(&rows, Some("first"), SidebarKey::Up).as_deref(),
            Some("first")
        );
        assert_eq!(
            selection_after_key(&rows, Some("last"), SidebarKey::Down).as_deref(),
            Some("last")
        );
    }

    #[test]
    fn arrows_from_a_disabled_selection_move_to_the_nearest_enabled_row() {
        let rows = vec![
            ("first".into(), false),
            ("selected-disabled".into(), true),
            ("last".into(), false),
        ];

        assert_eq!(
            selection_after_key(&rows, Some("selected-disabled"), SidebarKey::Up).as_deref(),
            Some("first")
        );
        assert_eq!(
            selection_after_key(&rows, Some("selected-disabled"), SidebarKey::Down).as_deref(),
            Some("last")
        );
    }

    #[test]
    fn arrows_from_boundary_disabled_selection_fall_back_to_enabled_rows() {
        let rows = vec![
            ("disabled-first".into(), true),
            ("middle".into(), false),
            ("disabled-last".into(), true),
        ];

        assert_eq!(
            selection_after_key(&rows, Some("disabled-first"), SidebarKey::Up).as_deref(),
            Some("middle")
        );
        assert_eq!(
            selection_after_key(&rows, Some("disabled-last"), SidebarKey::Down).as_deref(),
            Some("middle")
        );
    }

    #[test]
    fn home_end_and_empty_selection_use_enabled_stable_ids() {
        let rows = vec![
            ("disabled-first".into(), true),
            ("middle".into(), false),
            ("disabled-last".into(), true),
            ("end".into(), false),
        ];

        assert_eq!(
            selection_after_key(&rows, Some("end"), SidebarKey::Home).as_deref(),
            Some("middle")
        );
        assert_eq!(
            selection_after_key(&rows, Some("middle"), SidebarKey::End).as_deref(),
            Some("end")
        );
        assert_eq!(
            selection_after_key(&rows, None, SidebarKey::Down).as_deref(),
            Some("middle")
        );
        assert_eq!(
            selection_after_key(&rows, None, SidebarKey::Up).as_deref(),
            Some("end")
        );
    }

    #[test]
    fn normal_fallback_label_row_remains_selectable_by_stable_id() {
        let items = vec![
            item("inbox", "Inbox"),
            item("settings", "Settings").disabled(true),
            item("projects", "Projects"),
            item("localized-components", "Components")
                .localized("lab-navigation-components"),
            item("missing-label-fallback", "Fallback sample").localized("lab-sidebar-missing"),
        ];
        let rows: Vec<_> = items
            .iter()
            .map(|item| (item.id.clone(), item.disabled))
            .collect();

        let fallback = selection_after_key(
            &rows,
            Some("localized-components"),
            SidebarKey::End,
        );
        assert_eq!(fallback.as_deref(), Some("missing-label-fallback"));
        assert_eq!(selected_index(&items, fallback.as_deref()), Some(4));
        assert_eq!(
            selection_after_key(
                &rows,
                fallback.as_deref(),
                SidebarKey::Down
            )
            .as_deref(),
            Some("missing-label-fallback")
        );
        assert!(should_update_selection(
            &rows,
            &fallback,
            &fallback,
            SidebarKey::Enter
        ));
    }

    #[test]
    fn disabled_current_and_all_disabled_rows_do_not_activate_or_clear_owner() {
        let rows = vec![("selected-disabled".into(), true)];
        for key in [
            SidebarKey::Up,
            SidebarKey::Down,
            SidebarKey::Home,
            SidebarKey::End,
            SidebarKey::Enter,
        ] {
            assert_eq!(
                selection_after_key(&rows, Some("selected-disabled"), key).as_deref(),
                Some("selected-disabled")
            );
        }
        assert_eq!(selection_after_key(&rows, None, SidebarKey::Down), None);
        let current = Some("selected-disabled".to_owned());
        assert!(!should_update_selection(
            &rows,
            &current,
            &current,
            SidebarKey::Enter
        ));
    }

    #[test]
    fn navigation_keys_propagate_when_sidebar_has_no_enabled_rows() {
        for rows in [Vec::new(), vec![("disabled".to_owned(), true)]] {
            for key in [
                SidebarKey::Up,
                SidebarKey::Down,
                SidebarKey::Home,
                SidebarKey::End,
            ] {
                assert!(!should_consume_sidebar_key(&rows, key));
            }
        }

        assert!(should_consume_sidebar_key(
            &[("enabled".to_owned(), false)],
            SidebarKey::Down
        ));
        assert!(should_consume_sidebar_key(&[], SidebarKey::Enter));
    }

    #[test]
    fn enter_updates_the_owner_with_the_current_selection() {
        let rows = vec![("settings".to_owned(), false)];
        let current = Some("settings".to_owned());
        assert!(should_update_selection(
            &rows,
            &current,
            &current,
            SidebarKey::Enter
        ));
        assert!(!should_update_selection(
            &rows,
            &current,
            &current,
            SidebarKey::Down
        ));
        assert!(!should_update_selection(
            &rows,
            &None,
            &None,
            SidebarKey::Enter
        ));
    }
}
