#[cfg(test)]
mod tests {
    use gpui::{
        App, AppContext as _, Context, Element as _, Entity, InteractiveElement as _, MouseButton,
        ParentElement as _, Render, StatefulInteractiveElement as _, Styled as _, Window,
        accesskit::{Action, Node, Role},
        div, px,
    };
    use gpui_component::{
        IndexPath,
        list::{List, ListDelegate, ListItem, ListState},
    };

    #[test]
    fn primary_mousedown_advertises_the_existing_click_fallback() {
        let element = div().on_mouse_down(MouseButton::Left, |_, _, _| {});
        let mut node = Node::new(Role::GenericContainer);
        element.write_a11y_info(&mut node);
        assert!(node.supports_action(Action::Click));
    }

    #[test]
    fn secondary_and_outside_listeners_do_not_advertise_primary_click() {
        let right = div().on_mouse_down(MouseButton::Right, |_, _, _| {});
        let outside = div().on_mouse_down_out(|_, _, _| {});
        for element in [right, outside] {
            let mut node = Node::new(Role::GenericContainer);
            element.write_a11y_info(&mut node);
            assert!(!node.supports_action(Action::Click));
        }
    }

    #[test]
    fn explicit_click_advertisement_is_preserved() {
        let element = div().id("click-probe").on_click(|_, _, _| {});
        let mut node = Node::new(Role::GenericContainer);
        element.write_a11y_info(&mut node);
        assert!(node.supports_action(Action::Click));
    }

    struct Delegate;
    impl ListDelegate for Delegate {
        type Item = ListItem;
        fn set_selected_index(
            &mut self,
            _: Option<IndexPath>,
            _: &mut Window,
            _: &mut Context<ListState<Self>>,
        ) {
        }
        fn items_count(&self, section: usize, _: &App) -> usize {
            if section == 0 { 3 } else { 0 }
        }
        fn render_item(
            &mut self,
            index: IndexPath,
            _: &mut Window,
            _: &mut Context<ListState<Self>>,
        ) -> Option<ListItem> {
            Some(ListItem::new(index.row).child(format!("Row {}", index.row)))
        }
    }

    struct Probe {
        state: Entity<ListState<Delegate>>,
    }
    impl Render for Probe {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl gpui::IntoElement {
            List::new(&self.state).w(px(320.0)).h(px(180.0))
        }
    }

    #[gpui::test]
    fn rendered_list_roles_match_selection_mode(cx: &mut gpui::TestAppContext) {
        cx.update(gpui_component::init);
        for selectable in [true, false] {
            let (_, window) = cx.add_window_view(|window, app| Probe {
                state: app.new(|cx| ListState::new(Delegate, window, cx).selectable(selectable)),
            });
            let snapshot = window.update(|window, app| {
                let _ = window.draw(app);
                window
                    .debug_a11y_tree_json()
                    .expect("test accessibility active")
            });
            let tree: serde_json::Value = serde_json::from_str(&snapshot).unwrap();
            let nodes = tree["nodes"]
                .as_object()
                .unwrap()
                .values()
                .collect::<Vec<_>>();
            let (root_role, row_role) = if selectable {
                ("ListBox", "ListBoxOption")
            } else {
                ("List", "ListItem")
            };
            assert!(
                nodes.iter().any(|n| n["aria"]["role"] == root_role),
                "{tree}"
            );
            let rows = nodes
                .iter()
                .filter(|n| n["aria"]["role"] == row_role)
                .collect::<Vec<_>>();
            assert_eq!(rows.len(), 3, "{tree}");
            for row in rows {
                assert_eq!(!row["aria"]["selected"].is_null(), selectable, "{row}");
            }
        }
    }
}
