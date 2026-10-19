//! A named button that opens an upstream GPUI Components popup menu.

use std::{collections::HashSet, rc::Rc};

use gpui::{AnyElement, IntoElement, ParentElement as _, SharedString, Styled as _, div};
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
    Message(SharedString),
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
        assert!(
            !id.as_str().trim().is_empty(),
            "Menu action IDs must not be empty"
        );
        Self {
            kind: MenuItemKind::Action {
                id,
                label: MenuLabel::Literal(label.into()),
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
        match &mut self.kind {
            MenuItemKind::Action { label, .. } => {
                *label = MenuLabel::Message(message_key.into());
            }
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
    items: Vec<MenuItem>,
    id: Option<Identity>,
}

impl MenuView {
    pub fn new<I>(label: impl Into<SharedString>, items: I) -> Self
    where
        I: IntoIterator<Item = MenuItem>,
    {
        Self {
            label: label.into(),
            items: validated_items(items),
            id: None,
        }
    }

    /// Sets the required stable identity of this menu trigger.
    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        self.id = Some(id.into());
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
                    MenuLabel::Message(key) => {
                        SharedString::from(context.localized_message(key.as_ref()))
                    }
                };
                BoundMenuItem::Action {
                    _id: id.clone(),
                    label,
                    disabled: *disabled,
                    handler: Rc::new(context.bind_control_action(action.clone())),
                    style: style.clone(),
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
                label,
                disabled,
                handler,
                style,
                ..
            } => {
                let handler = handler.clone();
                let disabled = *disabled;
                let label_for_element = label.clone();
                let style = style.clone();
                menu = menu.item(
                    // Keep GPUI's menu-item wrapper for focus, selection,
                    // keyboard handling, and role semantics. The custom child
                    // applies RustUI typography and foreground to the visible
                    // label itself, which the standard item API does not expose.
                    PopupMenuItem::element(move |_window, _cx| {
                        styled_container(div().child(label_for_element.clone()), &style)
                    })
                    .disabled(disabled)
                    .on_click(move |event, window, cx| {
                        // The pinned upstream Confirm path also calls disabled items.
                        if !disabled {
                            handler(event, window, cx);
                        }
                    }),
                );
            }
            BoundMenuItem::Separator => menu = menu.separator(),
        }
    }
    menu
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

        let trigger = themed_button(
            context,
            id,
            self.label.clone(),
            self.label.clone(),
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
