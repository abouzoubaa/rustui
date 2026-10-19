//! A RustUI child view with a pinned GPUI Components context menu.

use gpui::{AnyElement, InteractiveElement as _, IntoElement, ParentElement as _, div};
use gpui_component::menu::ContextMenuExt as _;
use rustui_core::Identity;

use crate::{
    components::overlay::menu::{MenuItem, append_menu_items, bind_items, validated_items},
    view::{EmptyView, RenderContext, RetentionScope, View},
};

/// A view whose right-click or long press opens RustUI menu actions.
pub struct ContextMenuView<V> {
    id: Identity,
    content: V,
    items: Vec<MenuItem>,
}

impl<V: View> ContextMenuView<V> {
    pub fn new(
        id: impl Into<Identity>,
        content: V,
        items: impl IntoIterator<Item = MenuItem>,
    ) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "ContextMenu identity must be nonempty"
        );
        Self {
            id,
            content,
            items: validated_items(items),
        }
    }
}

#[allow(non_snake_case)]
pub fn ContextMenu<V: View>(
    id: impl Into<Identity>,
    content: V,
    items: impl IntoIterator<Item = MenuItem>,
) -> ContextMenuView<V> {
    ContextMenuView::new(id, content, items)
}

impl<V: View> View for ContextMenuView<V> {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        context.with_element_scope(RetentionScope::ContextMenu(self.id.clone()), |context| {
            let items = bind_items(&self.items, context);
            // Upstream derives its open-state ID from the interactive wrapper's
            // explicit ID. Lowering the RustUI child inside this wrapper leaves
            // its own actions, bindings, and retention owner intact.
            div()
                .id(self.id.to_string())
                .child(self.content.into_element(context))
                .context_menu(move |menu, _window, _cx| append_menu_items(menu, &items))
                .into_any_element()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{components::basic::text::Text, runtime::AppAction};

    fn item(id: &str) -> MenuItem {
        MenuItem::action(id, "Action", AppAction::new(|_| {}))
    }

    #[test]
    fn validates_wrapper_and_shared_menu_action_ids() {
        assert!(
            std::panic::catch_unwind(|| ContextMenu(" ", Text("Target"), [item("one")])).is_err()
        );
        assert!(
            std::panic::catch_unwind(|| ContextMenu(
                "target",
                Text("Target"),
                [item("one"), item("one")]
            ))
            .is_err()
        );
    }

    #[test]
    fn accepts_separators_and_disabled_actions() {
        let menu = ContextMenu(
            "target",
            Text("Target"),
            [
                item("one"),
                MenuItem::separator(),
                item("two").disabled(true),
            ],
        );
        assert_eq!(menu.id.as_str(), "target");
        assert_eq!(menu.items.len(), 3);
    }
}
