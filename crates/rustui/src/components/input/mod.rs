//! Public input and selection controls with values connected through RustUI bindings.

pub mod checkbox;
pub mod color_picker;
pub mod combo_box;
pub mod date_picker;
pub mod picker;
pub mod radio;
pub mod search_field;
pub mod secure_field;
pub mod slider;
pub mod stepper;
pub mod text_editor;
pub mod text_field;
pub mod toggle;
pub mod token_field;

fn logical_input_row<T: gpui::Styled>(control: T, direction: rustui_i18n::LayoutDirection) -> T {
    let control = control.flex().flex_row();
    match direction {
        rustui_i18n::LayoutDirection::LeftToRight => control,
        rustui_i18n::LayoutDirection::RightToLeft => control.flex_row_reverse(),
    }
}

fn select_trailing_affordance<T: gpui::Styled>(
    control: T,
    direction: rustui_i18n::LayoutDirection,
) -> T {
    match direction {
        rustui_i18n::LayoutDirection::LeftToRight => control.right(gpui::px(9.0)),
        rustui_i18n::LayoutDirection::RightToLeft => control.left(gpui::px(9.0)),
    }
}

/// A select option keeps its stable binding value separate from its display title.
#[derive(Clone, Debug, PartialEq, Eq)]
struct SelectOption {
    value: String,
    title: gpui::SharedString,
}

impl gpui_component::searchable_list::SearchableListItem for SelectOption {
    type Value = String;

    fn title(&self) -> gpui::SharedString {
        self.title.clone()
    }

    fn value(&self) -> &String {
        &self.value
    }
}

fn select_options(
    values: &[String],
    keys: &std::collections::HashMap<String, gpui::SharedString>,
    mut resolve: impl FnMut(&str) -> String,
) -> Vec<SelectOption> {
    values
        .iter()
        .map(|value| {
            let title = keys
                .get(value)
                .map(|key| {
                    let text = resolve(key);
                    if text.trim().is_empty() || text.trim() == key.trim() {
                        value.clone()
                    } else {
                        text
                    }
                })
                .unwrap_or_else(|| value.clone());
            SelectOption {
                value: value.clone(),
                title: title.into(),
            }
        })
        .collect()
}

pub(super) fn mark_accessibility_node_disabled(node: &mut gpui::accesskit::Node) {
    node.set_disabled();
}

// Select keeps a displayed value snapshot and a separate popup cursor. Replacing
// its items can leave the cursor at the old index even when the value matches.
fn select_needs_reconciliation(
    options_changed: bool,
    previous_owner_value: &str,
    current_owner_value: &str,
) -> bool {
    options_changed || previous_owner_value != current_owner_value
}

#[cfg(test)]
mod tests {
    #[test]
    fn select_affordance_uses_the_logical_trailing_edge() {
        use gpui::{Styled, div, px};
        use rustui_i18n::LayoutDirection;
        let mut ltr = super::select_trailing_affordance(div(), LayoutDirection::LeftToRight);
        let mut rtl = super::select_trailing_affordance(div(), LayoutDirection::RightToLeft);
        assert_eq!(ltr.style().inset.right, Some(px(9.0).into()));
        assert_eq!(ltr.style().inset.left, None);
        assert_eq!(rtl.style().inset.left, Some(px(9.0).into()));
        assert_eq!(rtl.style().inset.right, None);
    }

    #[test]
    fn checkable_rows_follow_active_layout_direction() {
        use gpui::{FlexDirection, Styled};
        use rustui_i18n::LayoutDirection;
        for (direction, expected) in [
            (LayoutDirection::LeftToRight, FlexDirection::Row),
            (LayoutDirection::RightToLeft, FlexDirection::RowReverse),
        ] {
            let mut checkbox =
                super::logical_input_row(gpui_base::Checkbox::new("check"), direction);
            let mut radio = super::logical_input_row(gpui_base::Radio::new("radio"), direction);
            let mut switch = super::logical_input_row(gpui_base::Switch::new("switch"), direction);
            assert_eq!(checkbox.style().flex_direction, Some(expected));
            assert_eq!(radio.style().flex_direction, Some(expected));
            assert_eq!(switch.style().flex_direction, Some(expected));
        }
    }
    #[test]
    fn localized_select_titles_keep_stable_distinct_values_and_literal_fallbacks() {
        use super::select_options;
        use gpui_component::searchable_list::SearchableListItem;
        let values = vec!["first".to_owned(), "second".to_owned(), "third".to_owned()];
        let keys = [
            ("first".to_owned(), "first.key".into()),
            ("second".to_owned(), "second.key".into()),
        ]
        .into_iter()
        .collect();
        let translated = select_options(&values, &keys, |_| "Même titre".into());
        assert_eq!(translated[0].title(), "Même titre");
        assert_eq!(translated[1].title(), "Même titre");
        assert_eq!(translated[0].value(), "first");
        assert_eq!(translated[1].value(), "second");
        assert_eq!(translated[2].title(), "third");
        let missing = select_options(&values, &keys, |key| key.to_owned());
        assert_eq!(missing[0].title(), "first");
        let blank = select_options(&values, &keys, |_| " ".into());
        assert_eq!(blank[1].title(), "second");
    }

    #[test]
    fn localized_options_reject_unknown_values_and_blank_keys() {
        let binding = || {
            Binding::new(
                String::new(),
                |value: &String| value.clone(),
                |value: &mut String, next| *value = next,
            )
        };
        assert!(
            catch_unwind(AssertUnwindSafe(
                || Picker("Mode", binding(), ["one"]).localized_option("missing", "option.key")
            ))
            .is_err()
        );
        assert!(
            catch_unwind(AssertUnwindSafe(
                || ComboBox("Mode", binding(), ["one"]).localized_option("one", " ")
            ))
            .is_err()
        );
    }
    use super::select_needs_reconciliation;
    use crate::components::input::{
        checkbox::Checkbox, combo_box::ComboBox, picker::Picker, radio::Radio, slider::Slider,
        stepper::Stepper, toggle::Toggle,
    };
    use rustui_core::Binding;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    fn binding<T: Clone + 'static>(value: T) -> Binding<T> {
        Binding::new(
            value,
            |value: &T| value.clone(),
            |value, next| *value = next,
        )
    }

    fn assert_blank_identity_rejected(build: impl FnOnce()) {
        assert!(catch_unwind(AssertUnwindSafe(build)).is_err());
    }

    #[test]
    fn picker_rejects_blank_explicit_identity() {
        assert_blank_identity_rejected(|| {
            Picker("Mode", binding(String::new()), ["one"]).id("");
        });
    }

    #[test]
    fn combo_box_rejects_blank_explicit_identity() {
        assert_blank_identity_rejected(|| {
            ComboBox("Mode", binding(String::new()), ["one"]).id(" \t ");
        });
    }

    #[test]
    fn toggle_rejects_blank_explicit_identity() {
        assert_blank_identity_rejected(|| {
            Toggle("Enabled", binding(false)).id("");
        });
    }

    #[test]
    fn checkbox_rejects_blank_explicit_identity() {
        assert_blank_identity_rejected(|| {
            Checkbox("Enabled", binding(false)).id(" \t ");
        });
    }

    #[test]
    fn radio_rejects_blank_explicit_identity() {
        assert_blank_identity_rejected(|| {
            Radio("Enabled", binding(false)).id("");
        });
    }

    #[test]
    fn slider_rejects_blank_explicit_identity() {
        assert_blank_identity_rejected(|| {
            Slider(binding(0.5_f32), 0.0..=1.0).id(" \t ");
        });
    }

    #[test]
    fn stepper_rejects_blank_explicit_identity() {
        assert_blank_identity_rejected(|| {
            Stepper("Count", binding(1_i32)).id("");
        });
    }

    #[test]
    fn changed_options_resync_cursor_even_when_selected_value_is_unchanged() {
        assert!(select_needs_reconciliation(true, "Skia", "Skia"));
        assert!(select_needs_reconciliation(false, "GPUI", "Skia"));
        assert!(!select_needs_reconciliation(false, "Skia", "Skia"));
    }

    #[test]
    fn disabled_input_accessibility_node_is_marked_disabled() {
        let mut node = gpui::accesskit::Node::new(gpui::accesskit::Role::CheckBox);
        super::mark_accessibility_node_disabled(&mut node);
        assert!(node.is_disabled());
    }
}
#[cfg(test)]
mod focus_tests;

#[cfg(test)]
mod locale_transition_tests {
    use crate::{
        localization::{Catalog, Catalogs, Locale, activate_localization, set_locale},
        prelude::{Checkbox, Radio, Toggle, VStack},
        runtime::ComponentHost,
        theme::{AppearanceMode, activate_theme, parse_theme},
        view::View,
    };
    use rustui_core::Binding;

    struct Owner {
        checked: bool,
        selected: bool,
        toggled: bool,
    }
    impl View for Owner {
        fn body(&self) -> impl View {
            VStack::new()
                .child(
                    Checkbox(
                        "Checkbox",
                        Binding::new(self.checked, |o: &Owner| o.checked, |o, v| o.checked = v),
                    )
                    .id("locale-check"),
                )
                .child(
                    Radio(
                        "Radio",
                        Binding::new(self.selected, |o: &Owner| o.selected, |o, v| o.selected = v),
                    )
                    .id("locale-radio"),
                )
                .child(
                    Toggle(
                        "Toggle",
                        Binding::new(self.toggled, |o: &Owner| o.toggled, |o, v| o.toggled = v),
                    )
                    .id("locale-toggle"),
                )
        }
    }

    #[gpui::test]
    fn rendered_checkable_controls_keep_owner_state_after_arabic_switch(
        cx: &mut gpui::TestAppContext,
    ) {
        let en = Locale::parse("en").unwrap();
        let ar = Locale::parse("ar").unwrap();
        let mut catalogs = Catalogs::new(en.clone());
        catalogs
            .insert(Catalog::parse(en.clone(), "sample = Sample").unwrap())
            .unwrap();
        catalogs
            .insert(Catalog::parse(ar.clone(), "sample = مثال").unwrap())
            .unwrap();
        let theme = parse_theme(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../themes/macos27.json"
        )))
        .unwrap();
        cx.update(|app| {
            gpui_component::init(app);
            activate_theme(&theme, Some(AppearanceMode::Dark), app).unwrap();
            activate_localization(catalogs, en, app);
        });
        let (owner, window) = cx.add_window_view(|_, cx| {
            ComponentHost::new(
                Owner {
                    checked: false,
                    selected: false,
                    toggled: false,
                },
                cx,
            )
        });
        macro_rules! focus_next {
            () => {
                window.update(|window, app| {
                    let _ = window.draw(app);
                    window.focus_next(app);
                    let _ = window.draw(app);
                });
            };
        }
        macro_rules! press {
            () => {{
                let key = gpui::Keystroke::parse("enter").unwrap();
                window.simulate_event(gpui::KeyDownEvent {
                    keystroke: key.clone(),
                    is_held: false,
                    prefer_character_input: false,
                });
                window.simulate_event(gpui::KeyUpEvent { keystroke: key });
                window.update(|window, app| {
                    let _ = window.draw(app);
                });
            }};
        }
        focus_next!();
        press!();
        window.update(|_, app| assert!(owner.read(app).component().checked));
        window.update(|_, app| set_locale(ar, app));
        window.update(|window, app| {
            let _ = window.draw(app);
            assert!(owner.read(app).component().checked);
        });
        focus_next!();
        press!();
        window.update(|_, app| assert!(owner.read(app).component().selected));
        focus_next!();
        press!();
        window.update(|_, app| set_locale(Locale::parse("en").unwrap(), app));
        window.update(|window, app| {
            let _ = window.draw(app);
            let owner = owner.read(app).component();
            assert!(owner.checked && owner.selected && owner.toggled);
        });
    }
}
