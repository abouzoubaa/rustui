use super::{RetainedTextField, TextField, TextFieldView};
use crate::{
    components::input::{search_field::SearchField, secure_field::SecureField},
    runtime::component_host::ComponentHost,
    view::View,
};
use gpui::{AppContext as _, EntityInputHandler as _, Focusable as _};
use rustui_core::{Binding, Identity};

#[derive(Clone, Copy)]
enum Kind {
    Text,
    Secure,
    Search,
}

struct ValidationOwner {
    kind: Kind,
    value: String,
    valid: bool,
    valid_writes: usize,
    validate: bool,
}

impl View for ValidationOwner {
    fn body(&self) -> impl View {
        let value = Binding::new(
            self.value.clone(),
            |owner: &Self| owner.value.clone(),
            |owner, value| owner.value = value,
        );
        let valid = Binding::new(
            self.valid,
            |owner: &Self| owner.valid,
            |owner, valid| {
                owner.valid = valid;
                owner.valid_writes += 1;
            },
        );
        let field: TextFieldView = match self.kind {
            Kind::Text => TextField("Text", value),
            Kind::Secure => SecureField("Password", value),
            Kind::Search => SearchField("Search", value),
        };
        let field = field.id("validation-test").is_valid(valid);
        if self.validate {
            field.validate(|value| {
                if value == "ok" {
                    Ok(())
                } else {
                    Err("invalid value".into())
                }
            })
        } else {
            field
        }
    }
}

fn exercise(cx: &mut gpui::TestAppContext, kind: Kind, enabled: bool) {
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
    let (owner, window) = cx.add_window_view(move |_, cx| {
        ComponentHost::new(
            ValidationOwner {
                kind,
                value: "bad".into(),
                valid: true,
                valid_writes: 0,
                validate: enabled,
            },
            cx,
        )
    });
    let state = window.update(|window, app| {
        let _ = window.draw(app);
        let retained = owner
            .read(app)
            .retained_entity_for_test::<RetainedTextField>(
                vec![],
                Identity::from("validation-test"),
            )
            .unwrap();
        retained.read(app).state.clone()
    });
    window.cx.run_until_parked();
    window.update(|window, app| {
        let _ = window.draw(app);
        assert_eq!(owner.read(app).component().valid, !enabled);
        assert_eq!(owner.read(app).component().valid_writes > 0, enabled);
    });
    assert_eq!(
        window
            .debug_bounds("validation-error:invalid value")
            .is_some(),
        enabled
    );

    window.update(|window, app| {
        window.activate_window();
        state.update(app, |state, cx| state.focus(window, cx));
    });
    window.cx.run_until_parked();

    for (old_len, next, expected_valid) in [(3, "ok", true), (2, "bad", false), (3, "ok", true)] {
        window.update(|window, app| {
            state.update(app, |state, cx| {
                state.replace_text_in_range(Some(0..old_len), next, window, cx);
            });
        });
        window.cx.run_until_parked();
        window.update(|window, app| {
            let _ = window.draw(app);
        });
        window.cx.run_until_parked();
        window.update(|window, app| {
            let _ = window.draw(app);
            let owner = owner.read(app).component();
            assert_eq!(
                owner.value, next,
                "validation must not prevent the value write"
            );
            assert!(state.read(app).focus_handle(app).is_focused(window));
            assert_eq!(owner.valid, if enabled { expected_valid } else { true });
            if !enabled {
                assert_eq!(owner.valid_writes, 0);
            }
        });
        assert_eq!(
            window
                .debug_bounds("validation-error:invalid value")
                .is_some(),
            enabled && !expected_valid,
        );
    }
}

#[gpui::test]
fn text_field_validation_lifecycle(cx: &mut gpui::TestAppContext) {
    exercise(cx, Kind::Text, true);
}

#[gpui::test]
fn secure_field_validation_lifecycle(cx: &mut gpui::TestAppContext) {
    exercise(cx, Kind::Secure, true);
}

#[gpui::test]
fn search_field_validation_lifecycle(cx: &mut gpui::TestAppContext) {
    exercise(cx, Kind::Search, true);
}

#[gpui::test]
fn field_without_validator_preserves_binding_behavior(cx: &mut gpui::TestAppContext) {
    for kind in [Kind::Text, Kind::Secure, Kind::Search] {
        exercise(cx, kind, false);
    }
}
