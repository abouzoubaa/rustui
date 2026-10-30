//! Controlled integer stepper composed from GPUI Components buttons.

use std::ops::RangeInclusive;

use gpui::{AnyElement, Div, Hsla, IntoElement, ParentElement, SharedString, Styled, div};
use gpui_component::{
    Disableable as _, Sizable as _, Size,
    button::{Button as GpuiButton, ButtonVariant, ButtonVariants as _},
    theme::try_parse_color,
};
use rustui_core::{Binding, Identity};
use rustui_i18n::LayoutDirection;

use crate::{
    components::theme::{required_style, selector, styled_element},
    localization::MessageArguments,
    theme::{AppearanceMode, ThemeProfile, registry::active_theme},
    view::{EmptyView, RenderContext, View},
};

fn stepper_label_color(profile: &ThemeProfile, mode: AppearanceMode, disabled: bool) -> Hsla {
    let colors = profile.colors(Some(mode));
    let token = if disabled {
        &colors.disabled_text
    } else {
        &colors.primary_text
    };
    try_parse_color(token).expect("validated RustUI stepper label color")
}

pub struct StepperView {
    label: SharedString,
    label_message_id: Option<SharedString>,
    accessibility_label: Option<StepperAccessibilityLabel>,
    value: Binding<i32>,
    range: RangeInclusive<i32>,
    step: i32,
    id: Option<Identity>,
    disabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum StepperAccessibilityLabel {
    Literal(SharedString),
    Message {
        id: SharedString,
        fallback: SharedString,
    },
}

impl StepperAccessibilityLabel {
    fn fallback(&self) -> &SharedString {
        match self {
            Self::Literal(label)
            | Self::Message {
                fallback: label, ..
            } => label,
        }
    }

    fn resolve(&self, context: &RenderContext<'_>) -> SharedString {
        match self {
            Self::Literal(label) => label.clone(),
            Self::Message { id, fallback } => {
                resolved_message_label(context.localized_message(id.as_ref()), id, fallback)
            }
        }
    }
}

fn resolved_message_label(
    localized: String,
    id: &SharedString,
    fallback: &SharedString,
) -> SharedString {
    if localized.trim().is_empty() || localized.trim() == id.trim() {
        fallback.clone()
    } else {
        SharedString::from(localized)
    }
}

fn native_stepper_row(direction: LayoutDirection) -> Div {
    let row = div().flex().flex_row().items_center().gap_2();
    match direction {
        LayoutDirection::LeftToRight => row,
        LayoutDirection::RightToLeft => row.flex_row_reverse(),
    }
}

fn displayed_value(value: i32, locale: Option<&crate::localization::Locale>) -> String {
    locale.map_or_else(
        || value.to_string(),
        |locale| crate::localization::format_integer(i64::from(value), locale),
    )
}

fn stepped_value(value: i32, minimum: i32, maximum: i32, step: i32, increment: bool) -> i32 {
    let value = value.clamp(minimum, maximum);
    let stepped = if increment {
        value.saturating_add(step)
    } else {
        value.saturating_sub(step)
    };
    stepped.clamp(minimum, maximum)
}

fn action_accessibility_labels<E>(
    visible_label: &str,
    accessibility_label: Option<&str>,
    mut localized_message: impl FnMut(&str, &MessageArguments) -> Result<String, E>,
) -> (String, String) {
    let label = accessibility_label.unwrap_or(visible_label);
    assert!(
        !label.trim().is_empty(),
        "Stepper accessible name must not be blank"
    );
    let arguments = MessageArguments::new().with("label", label);
    let mut resolve = |id, fallback| {
        let fallback = SharedString::from(format!("{fallback} {label}"));
        resolved_message_label(
            localized_message(id, &arguments).unwrap_or_default(),
            &SharedString::from(id),
            &fallback,
        )
        .to_string()
    };
    (
        resolve("rustui-stepper-increment", "Increase"),
        resolve("rustui-stepper-decrement", "Decrease"),
    )
}

impl StepperView {
    pub fn new(label: impl Into<SharedString>, value: Binding<i32>) -> Self {
        let label = label.into();
        Self {
            label,
            label_message_id: None,
            accessibility_label: None,
            value,
            range: i32::MIN..=i32::MAX,
            step: 1,
            id: None,
            disabled: false,
        }
    }

    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "Stepper IDs must be nonempty"
        );
        self.id = Some(id);
        self
    }

    pub fn range(mut self, range: RangeInclusive<i32>) -> Self {
        assert!(
            range.start() <= range.end(),
            "Stepper range must be ascending"
        );
        self.range = range;
        self
    }

    pub fn step(mut self, step: i32) -> Self {
        assert!(step > 0, "Stepper step must be positive");
        self.step = step;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Resolves the visible label from RustUI's active message catalog.
    pub fn localized_label(mut self, message_id: impl Into<SharedString>) -> Self {
        let id = message_id.into();
        assert!(
            !id.trim().is_empty(),
            "Stepper localization key must not be blank"
        );
        self.label_message_id = Some(id);
        self
    }

    /// Overrides the label used to name the increment and decrement buttons.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "Stepper accessibility label must not be blank"
        );
        self.accessibility_label = Some(StepperAccessibilityLabel::Literal(label));
        self
    }

    /// Resolves the increment and decrement button names from RustUI's active message catalog.
    pub fn localized_accessibility_label(mut self, message_id: impl Into<SharedString>) -> Self {
        let id = message_id.into();
        assert!(
            !id.trim().is_empty(),
            "Stepper accessibility localization key must not be blank"
        );
        let fallback = self
            .accessibility_label
            .as_ref()
            .map_or_else(|| self.label.clone(), |label| label.fallback().clone());
        self.accessibility_label = Some(StepperAccessibilityLabel::Message { id, fallback });
        self
    }

    fn identity(&self, context: &mut RenderContext<'_>) -> Identity {
        self.identity_with(|base| context.next_implicit_retained_identity(base))
    }

    fn identity_with(&self, allocate_implicit: impl FnOnce(Identity) -> Identity) -> Identity {
        self.id
            .clone()
            .unwrap_or_else(|| allocate_implicit(Identity::from("stepper")))
    }
}

#[allow(non_snake_case)]
pub fn Stepper(label: impl Into<SharedString>, value: Binding<i32>) -> StepperView {
    StepperView::new(label, value)
}

impl View for StepperView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let identity = self.identity(context);
        let current = self
            .value
            .value()
            .clamp(*self.range.start(), *self.range.end());
        let decrement = context.binding_updater(self.value.clone(), {
            let minimum = *self.range.start();
            let maximum = *self.range.end();
            let step = self.step;
            move |value| {
                let next = stepped_value(*value, minimum, maximum, step, false);
                if next == *value {
                    return false;
                }
                *value = next;
                true
            }
        });
        let increment = context.binding_updater(self.value.clone(), {
            let minimum = *self.range.start();
            let maximum = *self.range.end();
            let step = self.step;
            move |value| {
                let next = stepped_value(*value, minimum, maximum, step, true);
                if next == *value {
                    return false;
                }
                *value = next;
                true
            }
        });

        let selector = selector(context);
        let increment_style = required_style(context, "stepper", "increment_button", &selector);
        let decrement_style = required_style(context, "stepper", "decrement_button", &selector);
        let separator_style = required_style(context, "stepper", "separator", &selector);
        let increment_glyph = required_style(context, "stepper", "increment_glyph", &selector);
        let decrement_glyph = required_style(context, "stepper", "decrement_glyph", &selector);
        let active = active_theme(context.app()).expect("stepper requires an active RustUI theme");
        let label_color = stepper_label_color(&active.profile, active.mode, self.disabled);
        let label = self.label_message_id.as_ref().map_or_else(
            || self.label.clone(),
            |id| resolved_message_label(context.localized_message(id.as_ref()), id, &self.label),
        );
        let accessible_name = self
            .accessibility_label
            .as_ref()
            .map(|label| label.resolve(context));
        let (increment_accessibility_label, decrement_accessibility_label) =
            action_accessibility_labels(
                label.as_ref(),
                accessible_name.as_deref(),
                |id, arguments| context.localized_message_with_arguments(id, arguments),
            );
        let increment_button = GpuiButton::new(format!("{identity}-increment"))
            .accessibility_label(increment_accessibility_label)
            .compact()
            .with_size(Size::XSmall)
            .with_variant(ButtonVariant::Secondary)
            .disabled(self.disabled || current >= *self.range.end())
            .on_click(move |_, _, cx| increment(cx))
            .child(styled_element(div().child("+"), &increment_glyph));
        let decrement_button = GpuiButton::new(format!("{identity}-decrement"))
            .accessibility_label(decrement_accessibility_label)
            .compact()
            .with_size(Size::XSmall)
            .with_variant(ButtonVariant::Secondary)
            .disabled(self.disabled || current <= *self.range.start())
            .on_click(move |_, _, cx| decrement(cx))
            .child(styled_element(div().child("−"), &decrement_glyph));

        native_stepper_row(context.layout_direction())
            .child(div().text_color(label_color).child(label))
            .child(displayed_value(current, context.locale()))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .child(styled_element(increment_button, &increment_style))
                    .child(styled_element(div(), &separator_style))
                    .child(styled_element(decrement_button, &decrement_style)),
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Stepper, StepperAccessibilityLabel, action_accessibility_labels, resolved_message_label,
        stepped_value, stepper_label_color,
    };
    use crate::theme::{AppearanceMode, parse_theme};
    use gpui_component::theme::try_parse_color;

    fn binding() -> rustui_core::Binding<i32> {
        rustui_core::Binding::new(0, |value: &i32| *value, |value, next| *value = next)
    }

    #[test]
    fn implicit_identity_survives_localized_label_and_distinguishes_siblings() {
        let first = Stepper("Quantity", binding());
        let second = Stepper("Quantity", binding());
        let allocate_at_slot = |slot| {
            move |base: rustui_core::Identity| {
                rustui_core::Identity::from(format!("owner:implicit-retained:{slot}:{base}"))
            }
        };

        let first_sibling = first.identity_with(allocate_at_slot(0));
        let second_sibling = second.identity_with(allocate_at_slot(1));
        let first_after_rerender = first.identity_with(allocate_at_slot(0));
        let first_after_locale_change =
            Stepper("Quantité", binding()).identity_with(allocate_at_slot(0));

        assert_ne!(first_sibling, second_sibling);
        assert_eq!(first_sibling, first_after_rerender);
        assert_eq!(first_sibling, first_after_locale_change);
    }

    #[test]
    fn explicit_accessible_name_allows_blank_visible_label() {
        let stepper = Stepper(" \t\n", binding()).accessibility_label("Number of guests");

        assert_eq!(
            action_accessibility_labels(
                stepper.label.as_ref(),
                stepper
                    .accessibility_label
                    .as_ref()
                    .map(StepperAccessibilityLabel::fallback)
                    .map(|label| label.as_ref()),
                |_, _| Err(()),
            ),
            (
                "Increase Number of guests".into(),
                "Decrease Number of guests".into()
            )
        );
    }

    #[test]
    #[should_panic(expected = "Stepper accessible name must not be blank")]
    fn rejects_blank_resolved_accessible_name() {
        action_accessibility_labels(" \t\n", None, |_, _| Err(()));
    }

    #[test]
    fn preserves_nonblank_visible_label_used_in_derived_names() {
        let stepper = Stepper(" Quantity ", binding());

        assert_eq!(stepper.label.as_ref(), " Quantity ");
        assert_eq!(stepper.accessibility_label, None);
        assert_eq!(
            action_accessibility_labels(stepper.label.as_ref(), None, |_, _| Err(())),
            ("Increase  Quantity ".into(), "Decrease  Quantity ".into())
        );
    }

    #[test]
    fn accessible_name_override_is_used_for_increment_and_decrement() {
        let stepper = Stepper("Quantity", binding()).accessibility_label("Number of guests");

        assert_eq!(stepper.label.as_ref(), "Quantity");
        assert_eq!(
            stepper.accessibility_label,
            Some(StepperAccessibilityLabel::Literal(
                "Number of guests".into()
            ))
        );
        assert_eq!(
            action_accessibility_labels(
                stepper.label.as_ref(),
                stepper
                    .accessibility_label
                    .as_ref()
                    .map(StepperAccessibilityLabel::fallback)
                    .map(|label| label.as_ref()),
                |_, _| Err(()),
            ),
            (
                "Increase Number of guests".into(),
                "Decrease Number of guests".into()
            )
        );
    }

    #[test]
    #[should_panic(expected = "Stepper accessibility label must not be blank")]
    fn rejects_blank_accessible_name_override() {
        Stepper("Quantity", binding()).accessibility_label(" \t\n");
    }

    #[test]
    #[should_panic(expected = "Stepper accessibility localization key must not be blank")]
    fn rejects_blank_localized_accessible_name_key() {
        Stepper("Quantity", binding()).localized_accessibility_label(" \t\n");
    }

    #[test]
    fn localized_accessible_name_uses_literal_or_visible_label_fallback() {
        let visible =
            Stepper("Quantity", binding()).localized_accessibility_label("stepper.quantity");
        let explicit = Stepper("Quantity", binding())
            .accessibility_label("Number of guests")
            .localized_accessibility_label("stepper.quantity");

        assert_eq!(
            visible.accessibility_label,
            Some(StepperAccessibilityLabel::Message {
                id: "stepper.quantity".into(),
                fallback: "Quantity".into(),
            })
        );
        assert_eq!(
            explicit.accessibility_label,
            Some(StepperAccessibilityLabel::Message {
                id: "stepper.quantity".into(),
                fallback: "Number of guests".into(),
            })
        );
    }

    #[test]
    fn localized_accessible_name_resolves_message_and_falls_back_when_blank() {
        let fallback: gpui::SharedString = "Quantity".into();
        let id: gpui::SharedString = "stepper.quantity".into();
        let localized = resolved_message_label("Quantité".into(), &id, &fallback);

        assert_eq!(localized, "Quantité");
        assert_eq!(
            action_accessibility_labels("Quantity", Some(localized.as_ref()), |_, _| Err(())),
            ("Increase Quantité".into(), "Decrease Quantité".into())
        );
        assert_eq!(
            resolved_message_label(" \t".into(), &id, &fallback),
            fallback
        );
        assert_eq!(
            resolved_message_label(id.to_string(), &id, &fallback),
            fallback
        );
        assert_eq!(
            resolved_message_label(format!(" \t{}\n", id.as_ref()), &id, &fallback),
            fallback
        );

        let explicit_fallback: gpui::SharedString = "Number of guests".into();
        assert_eq!(
            resolved_message_label(format!(" \t{}\n", id.as_ref()), &id, &explicit_fallback),
            explicit_fallback
        );
    }

    #[test]
    fn derived_accessible_names_localize_actions_and_interpolate_the_name() {
        let labels = action_accessibility_labels("Quantity", Some("Quantité"), |id, arguments| {
            let pattern = match id {
                "rustui-stepper-increment" => "Augmenter {label}",
                "rustui-stepper-decrement" => "Diminuer {label}",
                other => panic!("unexpected stepper accessibility message: {other}"),
            };
            crate::localization::interpolate(pattern, arguments)
        });

        assert_eq!(
            labels,
            ("Augmenter Quantité".into(), "Diminuer Quantité".into())
        );
    }

    #[test]
    fn unresolved_action_names_use_english_fallbacks() {
        let missing =
            action_accessibility_labels("Quantity", None, |id, _| Ok::<_, ()>(id.to_owned()));
        let blank =
            action_accessibility_labels("Quantity", None, |_, _| Ok::<_, ()>(" \t ".to_owned()));

        assert_eq!(
            missing,
            ("Increase Quantity".into(), "Decrease Quantity".into())
        );
        assert_eq!(
            blank,
            ("Increase Quantity".into(), "Decrease Quantity".into())
        );
    }

    #[test]
    fn stepper_label_uses_active_appearance_text_tokens() {
        let profile = parse_theme(include_str!("../../../../../themes/macos27.json"))
            .unwrap()
            .platform_snapshot;
        for mode in [AppearanceMode::Light, AppearanceMode::Dark] {
            let colors = profile.colors(Some(mode));
            assert_eq!(
                stepper_label_color(&profile, mode, true),
                try_parse_color(&colors.disabled_text).unwrap()
            );
            assert_eq!(
                stepper_label_color(&profile, mode, false),
                try_parse_color(&colors.primary_text).unwrap()
            );
        }
    }

    #[test]
    fn native_stepper_row_follows_layout_direction() {
        use gpui::Styled;
        let mut ltr = super::native_stepper_row(rustui_i18n::LayoutDirection::LeftToRight);
        let mut rtl = super::native_stepper_row(rustui_i18n::LayoutDirection::RightToLeft);
        assert_eq!(ltr.style().flex_direction, Some(gpui::FlexDirection::Row));
        assert_eq!(
            rtl.style().flex_direction,
            Some(gpui::FlexDirection::RowReverse)
        );
    }

    #[test]
    fn displayed_integer_follows_locale_and_preserves_unlocalized_fallback() {
        use crate::localization::Locale;
        assert_eq!(super::displayed_value(1234, None), "1234");
        assert_eq!(
            super::displayed_value(1234, Some(&Locale::parse("en").unwrap())),
            "1,234"
        );
        assert_eq!(
            super::displayed_value(1234, Some(&Locale::parse("fr").unwrap())),
            "1\u{202f}234"
        );
        assert_eq!(
            super::displayed_value(-1234, Some(&Locale::parse("ar").unwrap())),
            "\u{061c}-١\u{066c}٢٣٤"
        );
    }

    #[test]
    fn stepper_clamps_owner_value_before_applying_each_step() {
        assert_eq!(stepped_value(4, 5, 15, 2, false), 5);
        assert_eq!(stepped_value(16, 5, 15, 2, true), 15);
        assert_eq!(stepped_value(10, 5, 15, 2, true), 12);
        assert_eq!(stepped_value(10, 5, 15, 2, false), 8);
    }

    #[test]
    fn stepper_saturates_at_integer_limits_and_configured_bounds() {
        assert_eq!(
            stepped_value(i32::MAX, i32::MIN, i32::MAX, 1, true),
            i32::MAX
        );
        assert_eq!(
            stepped_value(i32::MIN, i32::MIN, i32::MAX, 1, false),
            i32::MIN
        );
        assert_eq!(stepped_value(14, 5, 15, 4, true), 15);
        assert_eq!(stepped_value(6, 5, 15, 4, false), 5);
    }
}
