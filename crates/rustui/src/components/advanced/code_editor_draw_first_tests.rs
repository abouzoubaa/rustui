use super::*;
use crate::runtime::component_host::ComponentHost;
use gpui::{EntityInputHandler, Focusable as _};

#[gpui::test]
fn local_edit_survives_draw_before_binding_effects(cx: &mut gpui::TestAppContext) {
    struct Owner {
        value: String,
    }
    impl View for Owner {
        fn body(&self) -> impl View {
            CodeEditor(
                "draw-first-oracle",
                Binding::new(
                    self.value.clone(),
                    |o: &Owner| o.value.clone(),
                    |o, value| o.value = value,
                ),
            )
            .language("rust")
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
                value: "fn main() {}".into(),
            },
            cx,
        )
    });
    let editor = window.update(|window, app| {
        window.draw(app).clear(app);
        owner
            .read(app)
            .retained_entity_for_test::<RetainedCodeEditor>(
                vec![],
                Identity::from("draw-first-oracle"),
            )
            .unwrap()
            .read(app)
            .editor
            .clone()
    });
    for (range, text, expected) in [
        (Some(11..11), "x", "fn main() {x}"),
        (Some(11..12), "", "fn main() {}"),
        (
            Some(11..11),
            "\n let café = 1;\n",
            "fn main() {\n let café = 1;\n}",
        ),
    ] {
        window.update(|window, app| {
            let old = owner.read(app).component().value.clone();
            editor.update(app, |state, cx| {
                state.replace_text_in_range(range, text, window, cx)
            });
            assert_eq!(
                owner.read(app).component().value,
                old,
                "binding effects must still be queued"
            );
            assert_eq!(editor.read(app).value().as_ref(), expected);
            window.draw(app).clear(app);
            assert_eq!(
                editor.read(app).value().as_ref(),
                expected,
                "drawing must retain the local text while the owner is stale"
            );
        });
        window.cx.run_until_parked();
        window.update(|_, app| assert_eq!(owner.read(app).component().value, expected));
    }
    if !gpui::fast_draw_first_enabled() {
        return;
    }
    let before_ime = window.update(|_, app| owner.read(app).component().value.clone());
    window.update(|window, app| {
        let old = owner.read(app).component().value.clone();
        editor.update(app, |state, cx| {
            state.replace_and_mark_text_in_range(Some(11..11), "か", Some(1..1), window, cx)
        });
        window.draw(app).clear(app);
        assert!(editor.read(app).value().contains('か'));
        assert_eq!(
            owner.read(app).component().value,
            old,
            "composition must not publish an uncommitted owner value"
        );
        editor.update(app, |state, cx| {
            state.replace_text_in_range(None, "漢字", window, cx)
        });
        window.draw(app).clear(app);
        assert!(editor.read(app).value().contains("漢字"));
    });
    window.cx.run_until_parked();
    window.update(|_, app| assert!(owner.read(app).component().value.contains("漢字")));
    window.update(|window, app| {
        let focus = editor.read(app).focus_handle(app);
        window.focus(&focus, app);
        window.draw(app).clear(app);
        window.dispatch_action(Box::new(gpui_base::input::Undo), app);
    });
    window.cx.run_until_parked();
    window.update(|window, app| {
        assert_eq!(
            owner.read(app).component().value,
            before_ime,
            "undo must restore the complete IME transaction"
        );
        window.draw(app).clear(app);
        assert_eq!(editor.read(app).value().as_ref(), before_ime);
    });
}
