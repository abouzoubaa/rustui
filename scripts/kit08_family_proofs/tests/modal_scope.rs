use gpui::{AppContext as _, Context, FocusHandle, IntoElement, InteractiveElement as _, ParentElement as _, Render, TestAppContext, Window, div};

struct ModalView { before: FocusHandle, scope: FocusHandle, child: FocusHandle, after: FocusHandle }
impl Render for ModalView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().child(div().track_focus(&self.before).tab_stop(true)).child(
            div().track_focus(&self.scope).tab_stop(true).child(div().track_focus(&self.child).tab_stop(true)),
        ).child(div().track_focus(&self.after).tab_stop(true))
    }
}

#[gpui::test]
fn modal_scope_restores_focus_and_restricts_traversal(cx: &mut TestAppContext) {
    let (before, scope, child, after) = cx.update(|cx| (
        cx.focus_handle().tab_stop(true), cx.focus_handle().tab_stop(true),
        cx.focus_handle().tab_stop(true), cx.focus_handle().tab_stop(true),
    ));
    let window = cx.add_window({
        let (before, scope, child, after) = (before.clone(), scope.clone(), child.clone(), after.clone());
        move |_, _| ModalView { before, scope, child, after }
    }).into();
    cx.update_window(window, |_, window, cx| window.draw(cx).clear(cx)).unwrap();
    cx.update_window(window, |_, window, cx| {
        let owner = window.root::<ModalView>().and_then(|root| root).unwrap().entity_id();
        window.set_modal_focus_scope(Some(&scope), owner);
        window.focus(&before, cx);
        assert_eq!(window.focused(cx), None, "outside focus request must be rejected");
        window.focus(&child, cx);
        assert_eq!(window.focused(cx), Some(child.clone()));
        window.focus_next(cx);
        assert!(matches!(window.focused(cx), Some(ref focused) if focused == &scope || focused == &child));
        assert!(window.restore_focus_within(&scope, Some(&child), cx));
        assert_eq!(window.focused(cx), Some(child.clone()));
        window.set_modal_focus_scope(None, owner);
        window.focus(&after, cx);
        assert_eq!(window.focused(cx), Some(after.clone()));
    }).unwrap();
}
