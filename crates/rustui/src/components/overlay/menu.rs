//! A named button that opens an upstream GPUI Components popup menu.

use std::{collections::HashSet, rc::Rc};

use gpui::{
    AnyElement, InteractiveElement as _, IntoElement, ParentElement as _, SharedString,
    StatefulInteractiveElement as _, Styled as _, div,
};
use gpui_component::menu::{DropdownMenu as _, PopupMenu, PopupMenuItem};
use rustui_core::Identity;

use crate::{
    components::{
        basic::button::{ButtonStyle, themed_button},
        theme::{required_style, required_style_with_artwork, selector, styled_container},
    },
    view::{ActionHandler, ControlAction, EmptyView, RenderContext, View},
};

#[derive(Clone)]
enum MenuLabel {
    Literal(SharedString),
    Message {
        key: SharedString,
        fallback: SharedString,
    },
}

fn localized_action_label(
    resolved: SharedString,
    message_key: &SharedString,
    fallback: &SharedString,
) -> SharedString {
    if resolved.trim().is_empty() || resolved.trim() == message_key.trim() {
        fallback.clone()
    } else {
        resolved
    }
}

enum MenuItemKind {
    Action {
        id: Identity,
        label: MenuLabel,
        action: ControlAction,
        disabled: bool,
    },
    Separator,
}

/// One action or separator in a RustUI menu.
pub struct MenuItem {
    kind: MenuItemKind,
}

impl MenuItem {
    /// Creates an action item with a stable semantic ID.
    pub fn action(
        id: impl Into<Identity>,
        label: impl Into<SharedString>,
        action: impl Into<ControlAction>,
    ) -> Self {
        let id = id.into();
        let label = label.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "Menu action IDs must not be empty"
        );
        assert!(
            !label.trim().is_empty(),
            "Menu actions require a non-empty visible label"
        );
        Self {
            kind: MenuItemKind::Action {
                id,
                label: MenuLabel::Literal(label),
                action: action.into(),
                disabled: false,
            },
        }
    }

    /// Creates a separator between menu groups.
    pub fn separator() -> Self {
        Self {
            kind: MenuItemKind::Separator,
        }
    }

    /// Uses an active RustUI message key in place of the literal label.
    pub fn localized(mut self, message_key: impl Into<SharedString>) -> Self {
        let message_key = message_key.into();
        assert!(
            !message_key.trim().is_empty(),
            "Menu actions require a non-empty localization key"
        );
        match &mut self.kind {
            MenuItemKind::Action { label, .. } => match label {
                MenuLabel::Literal(fallback) => {
                    *label = MenuLabel::Message {
                        key: message_key,
                        fallback: fallback.clone(),
                    };
                }
                MenuLabel::Message { key, .. } => *key = message_key,
            },
            MenuItemKind::Separator => {
                panic!("MenuItem::localized can only be used on an action item");
            }
        }
        self
    }

    /// Prevents pointer and keyboard activation while retaining the item.
    pub fn disabled(mut self, disabled: bool) -> Self {
        if let MenuItemKind::Action {
            disabled: item_disabled,
            ..
        } = &mut self.kind
        {
            *item_disabled = disabled;
        }
        self
    }
}

/// A named button that opens a controlled RustUI action menu.
pub struct MenuView {
    label: SharedString,
    label_key: Option<SharedString>,
    items: Vec<MenuItem>,
    id: Option<Identity>,
}

impl MenuView {
    pub fn new<I>(label: impl Into<SharedString>, items: I) -> Self
    where
        I: IntoIterator<Item = MenuItem>,
    {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "Menu trigger requires a non-empty visible label"
        );
        Self {
            label,
            label_key: None,
            items: validated_items(items),
            id: None,
        }
    }

    /// Resolves the trigger label from the active catalog, retaining its literal fallback.
    pub fn localized_label(mut self, key: impl Into<SharedString>) -> Self {
        let key = key.into();
        assert!(
            !key.trim().is_empty(),
            "Menu label localization key must not be blank"
        );
        self.label_key = Some(key);
        self
    }

    fn resolved_label(&self, localized: impl FnOnce(&str) -> String) -> SharedString {
        match &self.label_key {
            Some(key) => localized_action_label(localized(key).into(), key, &self.label),
            None => self.label.clone(),
        }
    }

    /// Sets the required stable identity of this menu trigger.
    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "Menu trigger IDs must not be empty"
        );
        self.id = Some(id);
        self
    }
}

/// Creates a named button that opens a menu.
#[allow(non_snake_case)]
pub fn Menu<I>(label: impl Into<SharedString>, items: I) -> MenuView
where
    I: IntoIterator<Item = MenuItem>,
{
    MenuView::new(label, items)
}

pub(crate) enum BoundMenuItem {
    Action {
        _id: Identity,
        label: SharedString,
        disabled: bool,
        handler: Rc<ActionHandler>,
        style: crate::theme::ResolvedComponentStyle,
        direction: rustui_i18n::LayoutDirection,
    },
    Separator,
}

pub(crate) fn validated_items(items: impl IntoIterator<Item = MenuItem>) -> Vec<MenuItem> {
    let items: Vec<_> = items.into_iter().collect();
    let mut ids = HashSet::new();
    for item in &items {
        if let MenuItemKind::Action { id, .. } = &item.kind {
            assert!(
                ids.insert(id.clone()),
                "Menu action IDs must be unique within a menu"
            );
        }
    }
    items
}

pub(crate) fn bind_items(
    items: &[MenuItem],
    context: &mut RenderContext<'_>,
) -> Vec<BoundMenuItem> {
    let style = required_style(context, "menu_item", "body", &selector(context));
    let direction = context.layout_direction();

    // Bind before the native popup opens so callbacks keep only the existing
    // weak owner update path and retain its locale snapshot for this popup.
    items
        .iter()
        .map(|item| match &item.kind {
            MenuItemKind::Action {
                id,
                label,
                action,
                disabled,
            } => {
                let label = match label {
                    MenuLabel::Literal(label) => label.clone(),
                    MenuLabel::Message { key, fallback } => localized_action_label(
                        SharedString::from(context.localized_message(key.as_ref())),
                        key,
                        fallback,
                    ),
                };
                BoundMenuItem::Action {
                    _id: id.clone(),
                    label,
                    disabled: *disabled,
                    handler: Rc::new(context.bind_control_action(action.clone())),
                    style: style.clone(),
                    direction,
                }
            }
            MenuItemKind::Separator => BoundMenuItem::Separator,
        })
        .collect()
}

pub(crate) fn append_menu_items(mut menu: PopupMenu, items: &[BoundMenuItem]) -> PopupMenu {
    for item in items {
        match item {
            BoundMenuItem::Action {
                _id,
                label,
                disabled,
                handler,
                style,
                direction,
                ..
            } => {
                menu = menu.item(popup_action_item(
                    SharedString::from(format!("menu-item-label-{}", _id.as_str())),
                    label.clone(),
                    *disabled,
                    handler.clone(),
                    style.clone(),
                    *direction,
                ));
            }
            BoundMenuItem::Separator => menu = menu.separator(),
        }
    }
    menu
}

fn popup_action_item(
    label_id: SharedString,
    label: SharedString,
    disabled: bool,
    handler: Rc<ActionHandler>,
    mut style: crate::theme::ResolvedComponentStyle,
    direction: rustui_i18n::LayoutDirection,
) -> PopupMenuItem {
    assert!(
        !label.trim().is_empty(),
        "Menu actions require a non-empty visible label"
    );
    // Let GPUI Components' disabled treatment inherit from its menu-item
    // wrapper instead of pinning the normal foreground.
    if disabled {
        style.colors.remove("foreground");
    }
    PopupMenuItem::element(move |_window, _cx| {
        let text = div().flex_1().child(label.clone());
        let text = if direction == rustui_i18n::LayoutDirection::RightToLeft {
            text.text_right()
        } else {
            text.text_left()
        };
        styled_container(text, &style)
            .id(label_id.clone())
            .aria_label(label.clone())
    })
    .disabled(disabled)
    .on_click(move |event, window, cx| {
        // The pinned upstream Confirm path also calls disabled items.
        if !disabled {
            handler(event, window, cx);
        }
    })
}

impl View for MenuView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        // PopupMenu paints from the GPUI popover role, set from this record at activation.
        let style = required_style_with_artwork(context, "menu", "body", &selector(context));
        let expected = crate::theme::registry::hsla(
            *style
                .colors
                .get("background")
                .expect("menu theme needs background"),
        );
        debug_assert_eq!(
            gpui_component::Theme::global(context.app()).colors.popover,
            expected
        );
        let id = self.id.as_ref().unwrap_or_else(|| {
            panic!("Menu requires an explicit stable identity; add `.id(\"menu-name\")`")
        });
        let id = SharedString::from(id.to_string());

        let items = bind_items(&self.items, context);

        let label = self.resolved_label(|key| context.localized_message(key));
        let trigger = themed_button(
            context,
            id,
            label.clone(),
            label,
            ButtonStyle::Automatic,
            false,
            0,
            true,
        );
        let dropdown =
            trigger.dropdown_menu(move |menu, _window, _cx| append_menu_items(menu, &items));

        div().flex().child(dropdown).into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::AppAction;

    #[test]
    fn localized_trigger_uses_literal_for_missing_or_blank_messages() {
        let menu = Menu("Options", []).localized_label("menu.label");
        assert_eq!(menu.resolved_label(|_| "menu.label".into()), "Options");
        assert_eq!(menu.resolved_label(|_| " ".into()), "Options");
        assert_eq!(
            menu.resolved_label(|_| "Options françaises".into()),
            "Options françaises"
        );
    }

    fn empty_handler() -> Rc<ActionHandler> {
        Rc::new(Box::new(|_, _, _| {}))
    }

    #[test]
    fn action_items_use_upstream_element_rows_with_theme_style() {
        // GPUI's ElementItem wrapper keeps the menu-item role, keyboard focus,
        // and selection behavior while allowing RustUI to style its text child.
        let document =
            crate::theme::parse_theme(include_str!("../../../../../themes/macos27.json"))
                .expect("macOS 27 theme parses");
        let style = crate::theme::resolve_component_style(
            &document,
            "menu_item",
            "body",
            &crate::components::theme::selector_for_mode(crate::theme::AppearanceMode::Light),
        )
        .expect("menu item style resolves");
        assert!(style.colors.contains_key("foreground"));
        assert_eq!(style.typography.as_ref().unwrap().family, "SF Pro");
        assert_eq!(style.typography.as_ref().unwrap().size, 13.0);
        assert_eq!(style.typography.as_ref().unwrap().weight, 510.0);

        for disabled in [false, true] {
            let item = popup_action_item(
                "menu-item-label-open".into(),
                "Open".into(),
                disabled,
                empty_handler(),
                style.clone(),
                rustui_i18n::LayoutDirection::LeftToRight,
            );
            let PopupMenuItem::ElementItem {
                disabled: actual, ..
            } = item
            else {
                panic!("Menu actions must retain the upstream menu-item wrapper");
            };
            assert_eq!(actual, disabled);
        }
    }

    #[test]
    #[should_panic(expected = "Menu trigger requires a non-empty visible label")]
    fn blank_trigger_labels_are_rejected() {
        Menu("  ", [action("open")]);
    }

    #[test]
    #[should_panic(expected = "Menu actions require a non-empty visible label")]
    fn blank_action_labels_are_rejected_when_built() {
        MenuItem::action("open", "  ", AppAction::new(|_| {}));
    }

    #[test]
    #[should_panic(expected = "Menu actions require a non-empty localization key")]
    fn blank_localization_keys_are_rejected_when_built() {
        action("open").localized("  ");
    }

    #[test]
    #[should_panic(expected = "Menu trigger IDs must not be empty")]
    fn blank_trigger_ids_are_rejected() {
        Menu("Actions", [action("open")]).id(" \t ");
    }

    #[test]
    fn blank_resolved_menu_translation_uses_original_label() {
        assert_eq!(
            localized_action_label(" \t".into(), &"menu.open".into(), &"Open settings".into()),
            "Open settings"
        );
    }

    #[test]
    fn unresolved_menu_translation_echo_uses_original_label() {
        assert_eq!(
            localized_action_label(
                " \tmenu.open\n".into(),
                &"menu.open".into(),
                &"Open settings".into()
            ),
            "Open settings"
        );
    }

    #[test]
    fn nonblank_resolved_menu_translation_takes_precedence() {
        assert_eq!(
            localized_action_label(
                "Ouvrir les réglages".into(),
                &"menu.open".into(),
                &"Open settings".into()
            ),
            "Ouvrir les réglages"
        );
    }

    #[test]
    fn repeated_localization_keeps_the_original_literal_fallback() {
        let item = action("open")
            .localized("menu.first")
            .localized("menu.second");
        let MenuItemKind::Action {
            label: MenuLabel::Message { key, fallback },
            ..
        } = item.kind
        else {
            panic!("Localized menu actions must retain their key and fallback");
        };
        assert_eq!(key, "menu.second");
        assert_eq!(fallback, "Action");
    }

    fn action(id: &str) -> MenuItem {
        MenuItem::action(id.to_owned(), "Action", AppAction::new(|_| {}))
    }

    #[test]
    #[should_panic(expected = "Menu action IDs must not be empty")]
    fn empty_action_identity_is_rejected() {
        action(" ");
    }

    #[test]
    #[should_panic(expected = "Menu action IDs must be unique within a menu")]
    fn duplicate_action_identity_is_rejected() {
        Menu("Actions", [action("same"), action("same")]);
    }

    #[test]
    fn separators_do_not_require_action_identity() {
        let menu = Menu(
            "Actions",
            [action("first"), MenuItem::separator(), action("last")],
        )
        .id("actions");
        assert_eq!(menu.items.len(), 3);
        assert_eq!(menu.id.as_ref().map(Identity::as_str), Some("actions"));
    }
}
