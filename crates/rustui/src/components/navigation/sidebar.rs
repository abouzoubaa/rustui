//! Controlled flat sidebar selection over GPUI Components.

use std::collections::HashSet;

use gpui::{
    AnyElement, InteractiveElement as _, IntoElement, ParentElement as _, Role, SharedString,
    StatefulInteractiveElement as _, Styled as _, div, px,
};
use gpui_component::sidebar::{
    Sidebar as GpuiSidebar, SidebarMenu, SidebarMenuItem as GpuiSidebarMenuItem,
};
use rustui_core::{Binding, Identity};

use crate::{
    theme::registry::active_theme,
    view::{EmptyView, RenderContext, View},
};

#[derive(Clone)]
enum SidebarLabel {
    Literal(SharedString),
    Message(SharedString),
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
        assert!(!id.trim().is_empty(), "Sidebar item IDs must be nonempty");
        Self {
            id,
            label: SidebarLabel::Literal(label.into()),
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
        self.label = SidebarLabel::Message(message_id.into());
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

/// A flat, controlled sidebar. Selection is a caller-owned stable item ID.
pub struct SidebarView {
    id: Identity,
    items: Vec<SidebarItem>,
    selection: Binding<Option<String>>,
    accessibility_label: Option<SharedString>,
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
        self.accessibility_label = Some(label.into());
        self
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
        let focus_identity = Identity::from(format!("sidebar-focus:{}", self.id.as_str()));
        let (focus_state, _) = context.retained_entity(focus_identity, |_, cx| cx.focus_handle());
        let focus_handle = focus_state.read(context.app()).clone().tab_stop(true);
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
                if *current == next {
                    return false;
                }
                *current = next;
                true
            });
            (key, update)
        })
        .collect();
        let items = self.items.iter().enumerate().map(|(index, item)| {
            let label = match &item.label {
                SidebarLabel::Literal(label) => label.clone(),
                SidebarLabel::Message(message_id) => {
                    SharedString::from(context.localized_message(message_id.as_ref()))
                }
            };
            let id = item.id.clone();
            let disabled = item.disabled;
            let write_selection = write_selection.clone();
            let focus_handle = focus_handle.clone();

            GpuiSidebarMenuItem::new(label)
                .active(selected == Some(index))
                .disable(disabled)
                .on_click(move |_, window, cx| {
                    focus_handle.focus(window, cx);
                    // Keep the owner guard even though upstream also disables the row.
                    if !disabled {
                        write_selection(Some(id.clone()), cx);
                    }
                })
        });

        let mut focus_scope = div()
            .id(format!("rustui-sidebar-focus:{}", self.id.as_str()))
            .track_focus(&focus_handle)
            .role(Role::Tree)
            .on_key_down(move |event, window, cx| {
                if !keyboard_focus.is_focused(window)
                    || event.prefer_character_input
                    || event.keystroke.modifiers.modified()
                {
                    return;
                }
                let Some((_, action)) = navigation_actions
                    .iter()
                    .find(|(key, _)| event.keystroke.key == *key)
                else {
                    return;
                };
                action(cx);
                cx.stop_propagation();
            });
        if let Some(label) = &self.accessibility_label {
            focus_scope = focus_scope.aria_label(label.clone());
        }

        let mut sidebar = GpuiSidebar::new(self.id.to_string())
            .collapsible(false)
            .child(SidebarMenu::new().children(items));
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

fn selection_after_key(
    rows: &[(String, bool)],
    current: Option<&str>,
    key: SidebarKey,
) -> Option<String> {
    let first_enabled = || rows.iter().position(|(_, disabled)| !disabled);
    let last_enabled = || rows.iter().rposition(|(_, disabled)| !disabled);
    let selected_enabled = rows
        .iter()
        .position(|(id, disabled)| !disabled && Some(id.as_str()) == current);

    let next = match key {
        SidebarKey::Up => match selected_enabled {
            Some(index) => rows[..index]
                .iter()
                .rposition(|(_, disabled)| !disabled)
                .unwrap_or(index),
            None => match last_enabled() {
                Some(index) => index,
                None => return current.map(str::to_owned),
            },
        },
        SidebarKey::Down => match selected_enabled {
            Some(index) => rows[index + 1..]
                .iter()
                .position(|(_, disabled)| !disabled)
                .map(|offset| index + 1 + offset)
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
    use super::{SidebarItem, SidebarKey, selected_index, selection_after_key, validate_items};

    fn item(id: &str, label: &str) -> SidebarItem {
        SidebarItem::new(id, label)
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
    }
}
