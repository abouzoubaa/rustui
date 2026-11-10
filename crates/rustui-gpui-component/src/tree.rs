use std::rc::Rc;

use gpui::{
    App, Context, ElementId, Entity, InteractiveElement as _, IntoElement, ParentElement as _,
    RenderOnce, SharedString, StyleRefinement, Styled, Window, div, prelude::FluentBuilder as _,
};

use crate::{
    Selectable as _, StyledExt,
    list::ListItem,
    menu::{ContextMenuExt as _, PopupMenu},
    scroll::ScrollableElement,
};

pub use gpui_base::{TreeEntry, TreeEvent, TreeItem, TreeState};

/// Create a [`Tree`].
pub fn tree<R>(state: &Entity<TreeState>, render_item: R) -> Tree
where
    R: Fn(usize, &TreeEntry, bool, &mut Window, &mut App) -> ListItem + 'static,
{
    Tree::new(state, render_item)
}

type RenderItem = dyn Fn(usize, &TreeEntry, bool, &mut Window, &mut App) -> ListItem;
type ContextMenuBuilder =
    dyn Fn(usize, &TreeEntry, PopupMenu, &mut Window, &mut Context<TreeState>) -> PopupMenu;

/// A styled tree view that preserves the legacy `gpui-component` API while
/// delegating tree behavior and interaction state to `gpui-base`.
#[derive(IntoElement)]
pub struct Tree {
    id: ElementId,
    state: Entity<TreeState>,
    style: StyleRefinement,
    accessibility_label: Option<SharedString>,
    render_item: Rc<RenderItem>,
    context_menu_builder: Option<Rc<ContextMenuBuilder>>,
    scroll_bounce: Option<bool>,
}

impl Tree {
    pub fn new<R>(state: &Entity<TreeState>, render_item: R) -> Self
    where
        R: Fn(usize, &TreeEntry, bool, &mut Window, &mut App) -> ListItem + 'static,
    {
        Self {
            id: ElementId::Name(format!("tree-{}", state.entity_id()).into()),
            state: state.clone(),
            style: StyleRefinement::default(),
            accessibility_label: None,
            render_item: Rc::new(render_item),
            context_menu_builder: None,
            scroll_bounce: None,
        }
    }

    /// Sets the accessible name of the tree.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    /// Add a context menu to the tree.
    pub fn context_menu<F>(mut self, f: F) -> Self
    where
        F: Fn(usize, &TreeEntry, PopupMenu, &mut Window, &mut Context<TreeState>) -> PopupMenu
            + 'static,
    {
        self.context_menu_builder = Some(Rc::new(f));
        self
    }

    /// Bounce at the vertical boundaries. On by default on iOS, Android and
    /// macOS, off elsewhere; reduced motion disables displacement.
    pub fn scroll_bounce(mut self, enabled: bool) -> Self {
        self.scroll_bounce = Some(enabled);
        self
    }
}

impl Styled for Tree {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for Tree {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = self.state.clone();
        let render_item = self.render_item;
        let context_menu_builder = self.context_menu_builder;
        let scroll_handle = self.state.read(cx).scroll_handle().clone();

        let mut tree = gpui_base::Tree::new(&self.state);
        if let Some(label) = self.accessibility_label {
            tree = tree.accessibility_label(label);
        }

        div()
            .id(self.id)
            .size_full()
            .child(
                tree
                    .item(move |ix, entry, entry_state, window, cx| {
                        // Capture only the id: cloning the entry would deep-clone
                        // its whole subtree for every visible row on every frame.
                        let id = entry.item().id.clone();
                        let context_menu_builder = context_menu_builder.clone();
                        let context_menu_state = state.clone();
                        let item = render_item(ix, entry, entry_state.is_selected(), window, cx)
                            .disabled(entry.is_disabled())
                            .selected(entry_state.is_selected())
                            .secondary_selected(entry_state.is_right_clicked());

                        div()
                            .child(item)
                            .context_menu(move |menu, window, cx| {
                                let Some(build) = context_menu_builder.as_ref() else {
                                    return menu;
                                };
                                context_menu_state.update(cx, |state, cx| {
                                    let Some(entry) = state
                                        .entry(ix)
                                        .filter(|entry| entry.item().id == id)
                                        .filter(|entry| !entry.is_disabled())
                                        .cloned()
                                    else {
                                        return menu;
                                    };
                                    build(ix, &entry, menu, window, cx)
                                })
                            })
                            .into_any_element()
                    })
                    .list_style(StyleRefinement::default().flex_grow_1().size_full())
                    .relative()
                    .size_full()
                    .map(|tree| {
                        gpui_base::ScrollBounce::new("tree-scroll-bounce", &scroll_handle, tree)
                            .when_some(self.scroll_bounce, |bounce, on| bounce.enabled(on))
                    }),
            )
            .refine_style(&self.style)
            .vertical_scrollbar(&scroll_handle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{AppContext as _, TestAppContext};

    #[test]
    fn legacy_tree_types_are_base_types() {
        fn accepts_base(_: gpui_base::TreeItem) {}
        accepts_base(TreeItem::new("id", "Label"));
    }

    struct NamedTreeRoot {
        state: Entity<TreeState>,
    }

    impl gpui::Render for NamedTreeRoot {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().size_full().child(
                Tree::new(&self.state, |_, entry, _, _, _| {
                    ListItem::new(entry.item().id.clone()).child(entry.item().label.clone())
                })
                .accessibility_label("Workspace files"),
            )
        }
    }

    #[gpui::test]
    fn tree_accessibility_label_and_levels_reach_the_tree_nodes(cx: &mut TestAppContext) {
        cx.update(|cx| crate::init(cx));
        let items = vec![
            TreeItem::new("root", "Root")
                .expanded(true)
                .child(
                    TreeItem::new("branch", "Branch")
                        .expanded(true)
                        .child(TreeItem::new("leaf", "Leaf")),
                )
                .child(TreeItem::new("disabled", "Disabled").disabled(true)),
        ];
        let state = cx.new(|cx| TreeState::new(cx).items(items));
        let (_, cx) = cx.add_window_view({
            let state = state.clone();
            move |_, _| NamedTreeRoot { state }
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let tree = cx.update(|window, _| window.debug_a11y_tree_json()).unwrap();
        let tree: serde_json::Value = serde_json::from_str(&tree).unwrap();
        let mut rows = tree["nodes"]
            .as_object()
            .expect("debug accessibility nodes are keyed by ephemeral ID")
            .values()
            .filter(|node| node["aria"]["role"] == "TreeItem")
            .map(|node| {
                (
                    node["aria"]["label"].as_str().unwrap().to_owned(),
                    node["aria"]["level"].as_u64().unwrap(),
                    node["aria"]["disabled"].as_bool().unwrap_or(false),
                )
            })
            .collect::<Vec<_>>();
        rows.sort_by_key(|(_, level, _)| *level);
        let tree_node = tree["nodes"]
            .as_object()
            .unwrap()
            .values()
            .find(|node| node["aria"]["role"] == "Tree")
            .unwrap();
        assert_eq!(tree_node["aria"]["label"], "Workspace files");
        assert_eq!(
            rows,
            vec![
                ("Root".to_owned(), 1, false),
                ("Branch".to_owned(), 2, false),
                ("Disabled".to_owned(), 2, true),
                ("Leaf".to_owned(), 3, false),
            ]
        );
    }
}
