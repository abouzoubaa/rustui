use super::*;
use crate::runtime::component_host::ComponentHost;
use gpui::{EntityInputHandler, Focusable as _, ParentElement as _, Styled as _};

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
    struct Shell(gpui::Entity<ComponentHost<Owner>>, bool);
    impl gpui::Render for Shell {
        fn render(
            &mut self,
            _: &mut gpui::Window,
            cx: &mut gpui::Context<Self>,
        ) -> impl gpui::IntoElement {
            let label = if self.1 {
                self.0.read(cx).component().value.clone()
            } else {
                String::new()
            };
            gpui::div().child(self.0.clone()).child(label)
        }
    }
    let owner = cx.update(|cx| {
        cx.new(|cx| {
            ComponentHost::new(
                Owner {
                    value: "fn main() {}".into(),
                },
                cx,
            )
        })
    });
    let nested = owner.clone();
    let (_, window) = cx.add_window_view(|_, _| Shell(nested, false));
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
    let suppressed_before = crate::runtime::owner_projection::suppressed();
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
    assert!(
        crate::runtime::owner_projection::suppressed() > suppressed_before,
        "already-painted native updates must suppress an owner frame"
    );
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
    window.update(|w, cx| {
        w.root::<Shell>()
            .flatten()
            .unwrap()
            .update(cx, |shell, cx| {
                shell.1 = true;
                cx.notify();
            });
        w.draw(cx).clear(cx);
    });
    let before = crate::runtime::owner_projection::suppressed();
    window.update(|w, cx| {
        let end = editor.read(cx).value().len();
        editor.update(cx, |s, cx| {
            s.replace_text_in_range(Some(end..end), "q", w, cx)
        });
        w.draw(cx).clear(cx);
    });
    window.cx.run_until_parked();
    assert_eq!(
        crate::runtime::owner_projection::suppressed(),
        before,
        "another view reading the owner must keep its notification"
    );
}

#[gpui::test]
fn native_owner_projection_keeps_geometry_and_observer_notifications(
    cx: &mut gpui::TestAppContext,
) {
    use std::{cell::Cell, rc::Rc};
    struct Owner {
        value: String,
        geometry: bool,
    }
    impl View for Owner {
        fn body(&self) -> impl View {
            crate::layout::Container(
                CodeEditor(
                    "projection-fallback",
                    Binding::new(
                        self.value.clone(),
                        |o: &Owner| o.value.clone(),
                        |o, v| o.value = v,
                    ),
                )
                .language("rust"),
            )
            .width(if self.geometry {
                300. + self.value.len() as f32
            } else {
                400.
            })
        }
    }
    cx.update(gpui_component::init);
    let (owner, window) = cx.add_window_view(|_, cx| {
        ComponentHost::new(
            Owner {
                value: "fn a() {}".into(),
                geometry: true,
            },
            cx,
        )
    });
    let editor = window.update(|window, cx| {
        window.draw(cx).clear(cx);
        owner
            .read(cx)
            .retained_entity_for_test::<RetainedCodeEditor>(
                vec![],
                Identity::from("projection-fallback"),
            )
            .unwrap()
            .read(cx)
            .editor
            .clone()
    });
    let before = crate::runtime::owner_projection::suppressed();
    window.update(|window, cx| {
        editor.update(cx, |s, cx| {
            s.replace_text_in_range(Some(3..3), "x", window, cx)
        });
        window.draw(cx).clear(cx);
    });
    window.cx.run_until_parked();
    assert_eq!(
        crate::runtime::owner_projection::suppressed(),
        before,
        "changed container geometry must notify"
    );
    let notifications = Rc::new(Cell::new(0));
    let observed = notifications.clone();
    let observer =
        window.update(|_, cx| cx.observe(&owner, move |_, _| observed.set(observed.get() + 1)));
    window.update(|window, cx| {
        owner.update(cx, |s, _| s.component_mut().geometry = false);
        editor.update(cx, |s, cx| {
            s.replace_text_in_range(Some(3..3), "y", window, cx)
        });
        window.draw(cx).clear(cx);
    });
    window.cx.run_until_parked();
    assert!(
        notifications.get() > 0,
        "external owner observers must still be notified"
    );
    assert_eq!(crate::runtime::owner_projection::suppressed(), before);
    drop(observer);
}

#[gpui::test]
fn native_owner_projection_rebinds_latest_target_and_normalizes(cx: &mut gpui::TestAppContext) {
    struct Owner {
        a: String,
        b: String,
        second: bool,
        normalize: bool,
    }
    impl View for Owner {
        fn body(&self) -> impl View {
            let binding = if self.second {
                Binding::new(
                    self.b.clone(),
                    |o: &Owner| o.b.clone(),
                    |o, v| {
                        o.b = if o.normalize { v.to_uppercase() } else { v };
                    },
                )
            } else {
                Binding::new(
                    self.a.clone(),
                    |o: &Owner| o.a.clone(),
                    |o, v| {
                        let v = if o.normalize { v.to_uppercase() } else { v };
                        o.a = v.clone();
                        o.b = v;
                        o.second = true;
                    },
                )
            };
            CodeEditor("projection-target", binding).language("rust")
        }
    }
    cx.update(gpui_component::init);
    let (owner, window) = cx.add_window_view(|_, cx| {
        ComponentHost::new(
            Owner {
                a: "a".into(),
                b: "a".into(),
                second: false,
                normalize: false,
            },
            cx,
        )
    });
    let editor = window.update(|w, cx| {
        w.draw(cx).clear(cx);
        owner
            .read(cx)
            .retained_entity_for_test::<RetainedCodeEditor>(
                vec![],
                Identity::from("projection-target"),
            )
            .unwrap()
            .read(cx)
            .editor
            .clone()
    });
    for (text, expected) in [("x", "ax"), ("y", "axy")] {
        window.update(|w, cx| {
            let end = editor.read(cx).value().len();
            editor.update(cx, |s, cx| {
                s.replace_text_in_range(Some(end..end), text, w, cx)
            });
            w.draw(cx).clear(cx);
        });
        window.cx.run_until_parked();
        window.update(|_, cx| assert_eq!(owner.read(cx).component().b, expected));
    }
    window.update(|_, cx| {
        assert_eq!(
            owner.read(cx).component().a,
            "ax",
            "the second edit must use the rebound target"
        );
        owner.update(cx, |o, _| o.component_mut().normalize = true);
    });
    let before = crate::runtime::owner_projection::suppressed();
    window.update(|w, cx| {
        editor.update(cx, |s, cx| s.replace_text_in_range(Some(3..3), "z", w, cx));
        w.draw(cx).clear(cx);
    });
    window.cx.run_until_parked();
    window.update(|w, cx| {
        assert_eq!(owner.read(cx).component().b, "AXYZ");
        w.draw(cx).clear(cx);
        assert_eq!(
            editor.read(cx).value().as_ref(),
            "AXYZ",
            "owner normalization must reach native text"
        );
    });
    assert_eq!(
        crate::runtime::owner_projection::suppressed(),
        before,
        "a normalized value must notify"
    );
}

#[gpui::test]
fn custom_lowering_keeps_owner_notification(cx: &mut gpui::TestAppContext) {
    struct Owner {
        value: String,
    }
    impl View for Owner {
        fn body(&self) -> impl View {
            CodeEditor(
                "custom-projection",
                Binding::new(
                    self.value.clone(),
                    |o: &Owner| o.value.clone(),
                    |o, v| o.value = v,
                ),
            )
            .language("rust")
        }
        fn into_element(&self, context: &mut RenderContext<'_>) -> gpui::AnyElement {
            gpui::IntoElement::into_any_element(
                gpui::div()
                    .child(self.body().into_element(context))
                    .child(self.value.clone()),
            )
        }
    }
    cx.update(gpui_component::init);
    let (owner, window) = cx.add_window_view(|_, cx| {
        ComponentHost::new(
            Owner {
                value: "fn a() {}".into(),
            },
            cx,
        )
    });
    let editor = window.update(|w, cx| {
        w.draw(cx).clear(cx);
        owner
            .read(cx)
            .retained_entity_for_test::<RetainedCodeEditor>(
                vec![],
                Identity::from("custom-projection"),
            )
            .unwrap()
            .read(cx)
            .editor
            .clone()
    });
    let before = crate::runtime::owner_projection::suppressed();
    window.update(|w, cx| {
        editor.update(cx, |s, cx| s.replace_text_in_range(Some(3..3), "x", w, cx));
        w.draw(cx).clear(cx);
    });
    window.cx.run_until_parked();
    window.update(|_, cx| assert_eq!(owner.read(cx).component().value, "fn xa() {}"));
    assert_eq!(
        crate::runtime::owner_projection::suppressed(),
        before,
        "body() cannot prove what a custom lowering paints"
    );
}

#[gpui::test]
fn visible_prepaint_matches_full_after_edits_and_viewport_changes(cx: &mut gpui::TestAppContext) {
    struct Owner {
        value: String,
    }

    impl View for Owner {
        fn body(&self) -> impl View {
            CodeEditor(
                "visible-prepaint-oracle",
                Binding::new(
                    self.value.clone(),
                    |o: &Owner| o.value.clone(),
                    |o, value| o.value = value,
                ),
            )
            .language("rust")
            .height(350.0)
        }
    }

    let source: String = (0..120)
        .map(|row| {
            format!(
                "    let row_{row:03} = \"a long decorated line for the editor wrap oracle\";\n"
            )
        })
        .collect();
    let theme = crate::theme::parse_theme(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../themes/macos27.json"
    )))
    .unwrap();
    let owner = cx.update(|app| {
        gpui_component::init(app);
        crate::theme::activate_theme(&theme, Some(crate::theme::AppearanceMode::Dark), app)
            .unwrap();
        app.new(|cx| {
            crate::runtime::component_host::ComponentHost::new(
                Owner {
                    value: source.clone(),
                },
                cx,
            )
        })
    });
    let root = owner.clone();
    let (_, window) = cx.add_window_view(|_, _| WindowShell(root));
    struct WindowShell(gpui::Entity<crate::runtime::component_host::ComponentHost<Owner>>);
    impl gpui::Render for WindowShell {
        fn render(
            &mut self,
            _: &mut gpui::Window,
            _: &mut gpui::Context<Self>,
        ) -> impl gpui::IntoElement {
            gpui::div().size_full().child(self.0.clone())
        }
    }

    let editor = window.update(|window, app| {
        window.draw(app).clear(app);
        owner
            .read(app)
            .retained_entity_for_test::<RetainedCodeEditor>(
                vec![],
                Identity::from("visible-prepaint-oracle"),
            )
            .unwrap()
            .read(app)
            .editor
            .clone()
    });
    for (range, text) in [(12..12, "a"), (12..13, "")] {
        window.update(|window, app| {
            editor.update(app, |state, cx| {
                state.replace_text_in_range(Some(range), text, window, cx)
            });
            window.draw(app).clear(app);
        });
        window.cx.run_until_parked();
    }
    window.update(|window, app| {
        let focus = editor.read(app).focus_handle(app);
        window.focus(&focus, app);
        editor.update(app, |state, cx| state.set_selected_range(0..0, cx));
        app.write_to_clipboard(gpui::ClipboardItem::new_string(
            "// pasted one\n// pasted two\n".into(),
        ));
        window.draw(app).clear(app);
        window.dispatch_action(Box::new(gpui_base::input::Paste), app);
    });
    window.cx.run_until_parked();
    window.update(|window, app| {
        assert!(editor.read(app).value().starts_with("// pasted one"));
        window.draw(app).clear(app);
        window.dispatch_action(Box::new(gpui_base::input::Undo), app);
    });
    window.cx.run_until_parked();
    window.update(|window, app| {
        window.draw(app).clear(app);
        assert_eq!(editor.read(app).value().as_ref(), source);
        editor.update(app, |state, cx| {
            state.set_scroll_offset(gpui::point(gpui::px(0.), gpui::px(-420.)), cx)
        });
        window.draw(app).clear(app);
        window.draw(app).clear(app);
    });
    window.simulate_resize(gpui::size(gpui::px(340.), gpui::px(510.)));
    window.update(|window, app| {
        window.draw(app).clear(app);
        crate::theme::activate_theme(&theme, Some(crate::theme::AppearanceMode::Light), app)
            .unwrap();
        window.draw(app).clear(app);
        assert_eq!(editor.read(app).value().as_ref(), source);
    });
}
