use gpui::{AnyElement, FontWeight, IntoElement, ParentElement, SharedString, Styled, div, px};
use gpui_component::{
    Disableable as _,
    button::{Button as GpuiButton, ButtonVariant, ButtonVariants as _},
};
use rustui_core::Identity;

use crate::view::{ControlAction, EmptyView, RenderContext, View};

/// Native button appearances backed by GPUI Components.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonStyle {
    #[default]
    Automatic,
    Plain,
    Bordered,
    BorderedProminent,
    Borderless,
}

impl ButtonStyle {
    fn variant(self) -> ButtonVariant {
        match self {
            Self::Automatic | Self::Bordered => ButtonVariant::Secondary,
            Self::BorderedProminent => ButtonVariant::Primary,
            Self::Plain => ButtonVariant::Ghost,
            Self::Borderless => ButtonVariant::Text,
        }
    }
}

pub(crate) fn themed_button(
    context: &RenderContext<'_>,
    id: SharedString,
    label: SharedString,
    accessibility_label: SharedString,
    style: ButtonStyle,
    disabled: bool,
    tab_index: isize,
    focusable: bool,
) -> GpuiButton {
    let resolved_theme = context.button_theme().cloned();
    let control_font_size = resolved_theme
        .as_ref()
        .map(|theme| px(theme.font_size))
        .unwrap_or_else(|| context.control_font_size());
    let mut label_element = div().text_size(control_font_size).child(label);
    if let Some(theme) = &resolved_theme {
        label_element = label_element
            .font_family(theme.font_family.clone())
            .font_weight(FontWeight(theme.font_weight));
        if disabled {
            label_element =
                label_element.text_color(crate::theme::registry::hsla(theme.disabled_foreground));
        }
    }

    let mut button = GpuiButton::new(id)
        .accessibility_label(accessibility_label)
        .child(label_element)
        .with_variant(style.variant())
        .px(control_font_size)
        .disabled(disabled)
        .tab_index(tab_index)
        .tab_stop(focusable);
    if let Some(theme) = &resolved_theme {
        button = button
            .min_w(px(theme.width))
            .h(px(theme.height))
            .px(px(theme.horizontal_padding))
            .rounded(px(theme.corner_radius));
        if matches!(style, ButtonStyle::Automatic | ButtonStyle::Bordered) {
            let background = button_background(theme, disabled);
            button = button
                .bg(crate::theme::registry::hsla(background))
                .text_color(crate::theme::registry::hsla(if disabled {
                    theme.disabled_foreground
                } else {
                    theme.foreground
                }));
        }
    }
    button
}

fn button_background(
    theme: &crate::theme::ResolvedButtonTheme,
    disabled: bool,
) -> crate::theme::RgbaValue {
    if disabled {
        theme.disabled_background
    } else {
        theme.background
    }
}

#[derive(Clone)]
enum ButtonAccessibilityLabel {
    Literal(SharedString),
    Message {
        key: SharedString,
        fallback: SharedString,
    },
}

impl ButtonAccessibilityLabel {
    fn resolve(&self, context: &RenderContext<'_>) -> SharedString {
        match self {
            Self::Literal(label) => label.clone(),
            Self::Message { key, fallback } => {
                resolved_message_label(context.localized_message(key.as_ref()), key, fallback)
            }
        }
    }
}

fn resolved_message_label(
    localized: String,
    key: &SharedString,
    fallback: &SharedString,
) -> SharedString {
    if localized.trim().is_empty() || localized.trim() == key.as_ref().trim() {
        fallback.clone()
    } else {
        SharedString::from(localized)
    }
}

fn require_accessible_name(label: SharedString) -> SharedString {
    assert!(
        !label.trim().is_empty(),
        "Button must provide a non-empty accessible name"
    );
    label
}

/// A RustUI button that dispatches an action on its owning component.
pub struct ButtonView {
    label: SharedString,
    localized_label_key: Option<SharedString>,
    localized_label_arguments: Option<crate::localization::MessageArguments>,
    action: ControlAction,
    id: Option<Identity>,
    style: ButtonStyle,
    disabled: bool,
    toggled: Option<bool>,
    accessibility_label: Option<ButtonAccessibilityLabel>,
    help: Option<SharedString>,
    tab_index: isize,
    focusable: bool,
}

impl ButtonView {
    pub fn new(label: impl Into<SharedString>, action: impl Into<ControlAction>) -> Self {
        Self {
            label: label.into(),
            localized_label_key: None,
            localized_label_arguments: None,
            action: action.into(),
            id: None,
            style: ButtonStyle::Automatic,
            disabled: false,
            toggled: None,
            accessibility_label: None,
            help: None,
            tab_index: 0,
            focusable: true,
        }
    }

    /// Resolves the visible label from the active catalog while retaining the
    /// constructor label as a stable fallback and identity seed.
    pub fn localized_label(mut self, message_key: impl Into<SharedString>) -> Self {
        let key = message_key.into();
        assert!(
            !key.trim().is_empty(),
            "Button label localization key must not be blank"
        );
        self.localized_label_key = Some(key);
        self.localized_label_arguments = None;
        self
    }

    /// Resolves the visible label with locale-aware arguments, retaining the
    /// original literal label when the message is missing or invalid.
    pub fn localized_label_with_arguments(
        self,
        message_key: impl Into<SharedString>,
        arguments: crate::localization::MessageArguments,
    ) -> Self {
        let mut button = self.localized_label(message_key);
        button.localized_label_arguments = Some(arguments);
        button
    }

    /// Sets an explicit GPUI element ID when labels are repeated.
    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "Button identity must be nonempty"
        );
        self.id = Some(id);
        self
    }

    /// Selects a native GPUI Components button appearance.
    pub fn button_style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }

    /// Disables pointer and keyboard activation while retaining native styling.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Exposes a push button's pressed state to assistive technology.
    ///
    /// Use this only when the button represents a persistent on/off state.
    pub fn toggled(mut self, toggled: bool) -> Self {
        self.toggled = Some(toggled);
        self
    }

    /// Replaces the name announced by assistive technologies.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "Button accessibility label must not be empty"
        );
        self.accessibility_label = Some(ButtonAccessibilityLabel::Literal(label));
        self
    }

    /// Uses a catalog message as the name announced by assistive technologies.
    pub fn localized_accessibility_label(mut self, message_key: impl Into<SharedString>) -> Self {
        let key = message_key.into();
        assert!(
            !key.trim().is_empty(),
            "Button accessibility localization key must not be blank"
        );
        let fallback = match &self.accessibility_label {
            Some(ButtonAccessibilityLabel::Literal(label))
            | Some(ButtonAccessibilityLabel::Message {
                fallback: label, ..
            }) => label.clone(),
            None => self.label.clone(),
        };
        self.accessibility_label = Some(ButtonAccessibilityLabel::Message { key, fallback });
        self
    }

    /// Adds explanatory hover help without changing the visible label.
    pub fn help(mut self, help: impl Into<SharedString>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Sets keyboard traversal order. The default is zero.
    pub fn tab_index(mut self, tab_index: isize) -> Self {
        self.tab_index = tab_index;
        self
    }

    /// Includes or excludes this button from keyboard focus traversal.
    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    fn resolved_label(&self, context: &RenderContext<'_>) -> SharedString {
        self.localized_label_key.as_ref().map_or_else(
            || self.label.clone(),
            |key| {
                let resolved = match &self.localized_label_arguments {
                    Some(arguments) => context
                        .localized_message_with_arguments(key.as_ref(), arguments)
                        .unwrap_or_else(|_| key.to_string()),
                    None => context.localized_message(key.as_ref()),
                };
                resolved_message_label(resolved, key, &self.label)
            },
        )
    }

    fn resolved_accessibility_label(&self, context: &RenderContext<'_>) -> SharedString {
        require_accessible_name(self.accessibility_label.as_ref().map_or_else(
            || self.resolved_label(context),
            |label| label.resolve(context),
        ))
    }

    fn identity(&self, context: &mut RenderContext<'_>) -> Identity {
        self.identity_with(|base| context.next_implicit_retained_identity(base))
    }

    fn identity_with(&self, allocate_implicit: impl FnOnce(Identity) -> Identity) -> Identity {
        self.id
            .clone()
            .unwrap_or_else(|| allocate_implicit(Identity::from("button")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[gpui::test]
    fn changing_label_preserves_implicit_button_focus(cx: &mut gpui::TestAppContext) {
        struct Owner {
            changed: bool,
            activations: usize,
        }
        impl View for Owner {
            fn body(&self) -> impl View {
                Button(
                    if self.changed { "Updated" } else { "Initial" },
                    crate::runtime::Action::new(|owner: &mut Owner| owner.activations += 1),
                )
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
        let (owner, window) = cx.add_window_view(|_, app| {
            crate::runtime::ComponentHost::new(
                Owner {
                    changed: false,
                    activations: 0,
                },
                app,
            )
        });
        window.update(|window, app| {
            let _ = window.draw(app);
            window.focus_next(app);
            let _ = window.draw(app);
        });
        let press_space = |window: &mut gpui::VisualTestContext| {
            let keystroke = gpui::Keystroke::parse("space").unwrap();
            window.simulate_event(gpui::KeyDownEvent {
                keystroke: keystroke.clone(),
                is_held: false,
                prefer_character_input: false,
            });
            window.simulate_event(gpui::KeyUpEvent { keystroke });
        };
        press_space(window);
        window.update(|window, app| {
            assert_eq!(owner.read(app).component().activations, 1);
            owner.update(app, |host, cx| {
                host.component_mut().changed = true;
                cx.notify();
            });
            let _ = window.draw(app);
        });
        press_space(window);
        window.update(|_, app| assert_eq!(owner.read(app).component().activations, 2));
    }

    #[test]
    fn disabled_button_uses_resolved_disabled_background() {
        let mut theme = crate::theme::resolve_button(
            &crate::theme::parse_theme(include_str!("../../../../../themes/macos27.json")).unwrap(),
            None,
        )
        .unwrap();
        theme.background = crate::theme::RgbaValue {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        theme.disabled_background = crate::theme::RgbaValue {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        };

        assert_eq!(button_background(&theme, true), theme.disabled_background);
        assert_eq!(button_background(&theme, false), theme.background);
    }

    #[test]
    fn accessible_name_preserves_nonblank_override_for_icon_only_button() {
        let button = Button("", crate::runtime::Action::new::<()>(|_| {}))
            .accessibility_label("  Open settings  ");

        assert_eq!(button.label.as_ref(), "");
        assert_eq!(
            button
                .accessibility_label
                .as_ref()
                .map(|label| match label {
                    ButtonAccessibilityLabel::Literal(label) => label.as_ref(),
                    ButtonAccessibilityLabel::Message { .. } => panic!("expected literal label"),
                }),
            Some("  Open settings  ")
        );
    }

    #[test]
    fn toggled_state_is_opt_in_and_preserved() {
        let action = || crate::runtime::Action::new::<()>(|_| {});
        assert_eq!(Button("Run", action()).toggled, None);
        assert_eq!(Button("Mute", action()).toggled(true).toggled, Some(true));
        assert_eq!(Button("Mute", action()).toggled(false).toggled, Some(false));
    }

    #[test]
    #[should_panic(expected = "Button must provide a non-empty accessible name")]
    fn empty_visible_label_requires_an_accessible_name_override() {
        require_accessible_name(" \t ".into());
    }

    #[test]
    #[should_panic(expected = "Button accessibility label must not be empty")]
    fn accessible_name_rejects_blank_override() {
        Button("Save", crate::runtime::Action::new::<()>(|_| {})).accessibility_label(" \t\n ");
    }

    #[test]
    #[should_panic(expected = "Button label localization key must not be blank")]
    fn localized_visible_label_rejects_blank_key() {
        Button("Save", crate::runtime::Action::new::<()>(|_| {})).localized_label(" \t\n ");
    }

    #[test]
    fn localized_visible_label_keeps_stable_identity_seed() {
        let button = Button("Save", crate::runtime::Action::new::<()>(|_| {}))
            .localized_label("button.save");
        assert_eq!(button.identity_with(|id| id).as_str(), "button");
        assert_eq!(
            button.localized_label_key.as_ref().unwrap().as_ref(),
            "button.save"
        );
    }

    #[test]
    fn localized_visible_label_arguments_can_be_replaced_by_a_static_label() {
        let args = crate::localization::MessageArguments::new().with("count", 7_i64);
        let button = Button("Count: 7", crate::runtime::Action::new::<()>(|_| {}))
            .localized_label_with_arguments("button.count", args.clone());
        assert_eq!(button.localized_label_arguments.as_ref(), Some(&args));
        assert_eq!(button.identity_with(|id| id).as_str(), "button");
        let button = button.localized_label("button.static");
        assert_eq!(button.localized_label_arguments, None);
    }

    #[test]
    #[should_panic(expected = "Button accessibility localization key must not be blank")]
    fn localized_accessible_name_rejects_blank_key() {
        Button("Save", crate::runtime::Action::new::<()>(|_| {}))
            .localized_accessibility_label(" \t\n ");
    }

    #[test]
    fn localized_accessible_name_uses_visible_or_explicit_fallback() {
        let visible = Button("Save", crate::runtime::Action::new::<()>(|_| {}))
            .localized_accessibility_label("button.save");
        let explicit = Button("Save", crate::runtime::Action::new::<()>(|_| {}))
            .accessibility_label("  Store changes  ")
            .localized_accessibility_label("button.save");

        assert!(matches!(
            visible.accessibility_label,
            Some(ButtonAccessibilityLabel::Message { ref key, ref fallback })
                if key.as_ref() == "button.save" && fallback.as_ref() == "Save"
        ));
        assert!(matches!(
            explicit.accessibility_label,
            Some(ButtonAccessibilityLabel::Message { ref key, ref fallback })
                if key.as_ref() == "button.save" && fallback.as_ref() == "  Store changes  "
        ));
    }

    #[test]
    fn localized_accessible_name_uses_translation_or_fallback() {
        let fallback: SharedString = "Save".into();
        let key: SharedString = "button.save".into();

        assert_eq!(
            resolved_message_label("Enregistrer".to_owned(), &key, &fallback).as_ref(),
            "Enregistrer"
        );
        assert_eq!(
            resolved_message_label(" \t".to_owned(), &key, &fallback).as_ref(),
            "Save"
        );
        assert_eq!(
            resolved_message_label("button.save".to_owned(), &key, &fallback).as_ref(),
            "Save"
        );
    }

    #[test]
    #[should_panic(expected = "Button identity must be nonempty")]
    fn rejects_whitespace_only_identity() {
        Button("Save", crate::runtime::Action::new::<()>(|_| {})).id(" \t");
    }

    #[test]
    fn explicit_identity_preserves_nonblank_value() {
        let button = Button("Save", crate::runtime::Action::new::<()>(|_| {})).id(" save ");

        assert_eq!(
            button
                .identity_with(|_| panic!("explicit identity must not allocate a fallback"))
                .as_str(),
            " save "
        );
    }

    #[test]
    fn implicit_identity_distinguishes_same_label_siblings_and_is_stable_by_slot() {
        let action = || crate::runtime::Action::new::<()>(|_| {});
        let first = Button("Save", action());
        let second = Button("Save", action());
        let allocate_at_slot = |slot| {
            move |base: Identity| Identity::from(format!("owner:implicit-retained:{slot}:{base}"))
        };

        let first_sibling = first.identity_with(allocate_at_slot(0));
        let second_sibling = second.identity_with(allocate_at_slot(1));
        let first_after_rerender = first.identity_with(allocate_at_slot(0));

        assert_ne!(first_sibling, second_sibling);
        assert_eq!(first_sibling, first_after_rerender);
    }
}

/// Creates a button whose action comes from `self.action(Self::handler)`.
#[allow(non_snake_case)]
pub fn Button(label: impl Into<SharedString>, action: impl Into<ControlAction>) -> ButtonView {
    ButtonView::new(label, action)
}

impl View for ButtonView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let id = SharedString::from(self.identity(context).to_string());
        let label = self.resolved_label(context);
        let accessibility_label = self.resolved_accessibility_label(context);
        let mut button = themed_button(
            context,
            id,
            label,
            accessibility_label,
            self.style,
            self.disabled,
            self.tab_index,
            self.focusable,
        )
        .on_click(context.bind_control_action(self.action.clone()));
        if let Some(toggled) = self.toggled {
            button = button.toggled(toggled);
        }
        if let Some(help) = &self.help {
            button = button.tooltip(help.clone());
        }
        div()
            .self_start()
            .items_start()
            .child(button)
            .into_any_element()
    }
}

#[cfg(test)]
mod whitespace_accessible_name_contract_regression {
    use super::require_accessible_name;
    use gpui::SharedString;

    #[test]
    fn rejects_whitespace_only_resolved_name() {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            require_accessible_name(SharedString::from(" \t\n"));
        }));

        assert!(
            result.is_err(),
            "whitespace-only accessible names must be rejected"
        );
    }
}
