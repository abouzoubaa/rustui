//! Accessible link activation through the pinned GPUI Components Button.

use gpui::{AnyElement, FontWeight, IntoElement, SharedString, Styled as _, px};
use gpui_component::{
    Disableable as _,
    button::{Button as GpuiButton, ButtonVariant, ButtonVariants as _},
};
use rustui_core::Identity;

use crate::{
    components::theme::{required_style, selector},
    theme::RgbaValue,
    view::{ControlAction, EmptyView, RenderContext, View},
};

/// A platform-opened link with an accessible visible label.
#[derive(Clone)]
pub struct LinkView {
    label: SharedString,
    href: SharedString,
    id: Option<Identity>,
    accessibility_label: Option<LinkAccessibilityLabel>,
    disabled: bool,
    open_url_on_activate: bool,
    activation: Option<ControlAction>,
}

#[derive(Clone)]
enum LinkAccessibilityLabel {
    Literal(SharedString),
    Message {
        id: SharedString,
        fallback: SharedString,
    },
}

impl LinkView {
    pub fn new(label: impl Into<SharedString>, href: impl Into<SharedString>) -> Self {
        let href = href.into();
        assert!(
            !href.trim().is_empty(),
            "Link destination must not be empty"
        );
        Self {
            label: label.into(),
            href,
            id: None,
            accessibility_label: None,
            disabled: false,
            open_url_on_activate: true,
            activation: None,
        }
    }

    /// Sets a stable element identity. Use distinct IDs for repeated links with the same label and URL.
    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "Link identity must not be blank"
        );
        self.id = Some(id);
        self
    }

    /// Overrides the accessible name; the visible label remains unchanged.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "Link accessibility label must not be empty"
        );
        self.accessibility_label = Some(LinkAccessibilityLabel::Literal(label));
        self
    }

    /// Resolves the accessible name from the active locale, falling back to
    /// the explicit name or visible link label.
    pub fn localized_accessibility_label(mut self, message_id: impl Into<SharedString>) -> Self {
        let id = message_id.into();
        assert!(
            !id.trim().is_empty(),
            "Link accessibility localization key must not be blank"
        );
        let fallback = match &self.accessibility_label {
            Some(LinkAccessibilityLabel::Literal(label))
            | Some(LinkAccessibilityLabel::Message {
                fallback: label, ..
            }) => label.clone(),
            None => self.label.clone(),
        };
        assert!(
            !fallback.trim().is_empty(),
            "Link must provide a non-empty accessible fallback"
        );
        self.accessibility_label = Some(LinkAccessibilityLabel::Message { id, fallback });
        self
    }

    /// Observes activation before the URL is handed to the platform.
    /// This reports a user activation, not completion of browser navigation.
    pub fn on_activate(mut self, action: impl Into<ControlAction>) -> Self {
        self.activation = Some(action.into());
        self
    }

    /// Controls whether activation hands the URL to the platform opener.
    /// Defaults to `true`; set to `false` for links whose activation is handled locally.
    pub fn open_url_on_activate(mut self, open: bool) -> Self {
        self.open_url_on_activate = open;
        self
    }

    /// Disables pointer and keyboard activation and removes the link from tab traversal.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    fn identity(&self) -> Identity {
        self.id.clone().unwrap_or_else(|| {
            Identity::from(format!(
                "link:{}:{}:{}:{}",
                self.label.len(),
                self.label,
                self.href.len(),
                self.href
            ))
        })
    }

    fn resolved_accessibility_label(
        &self,
        localized_message: impl FnOnce(&str) -> String,
    ) -> SharedString {
        let label = match &self.accessibility_label {
            None => self.label.clone(),
            Some(LinkAccessibilityLabel::Literal(label)) => label.clone(),
            Some(LinkAccessibilityLabel::Message { id, fallback }) => {
                let localized = localized_message(id.as_ref());
                if localized.trim().is_empty() || localized.trim() == id.trim() {
                    fallback.clone()
                } else {
                    localized.into()
                }
            }
        };
        assert!(
            !label.trim().is_empty(),
            "Link must provide a non-empty accessible name"
        );
        label
    }
}

/// Creates a platform-opened link.
#[allow(non_snake_case)]
pub fn Link(label: impl Into<SharedString>, href: impl Into<SharedString>) -> LinkView {
    LinkView::new(label, href)
}

fn link_foreground(
    foreground: RgbaValue,
    disabled_foreground: RgbaValue,
    disabled: bool,
) -> RgbaValue {
    if disabled {
        disabled_foreground
    } else {
        foreground
    }
}

impl View for LinkView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let style = required_style(context, "link", "body", &selector(context));
        let href = self.href.clone();
        let open_url_on_activate = self.open_url_on_activate;
        let activation = self
            .activation
            .as_ref()
            .map(|action| context.bind_control_action(action.clone()));
        let accessibility_label =
            self.resolved_accessibility_label(|id| context.localized_message(id));

        let typography = style
            .typography
            .as_ref()
            .expect("link theme needs typography");
        let disabled_foreground = context
            .button_theme()
            .expect("link requires an active RustUI theme")
            .disabled_foreground;
        GpuiButton::new(self.identity().to_string())
            .label(self.label.clone())
            .accessibility_label(accessibility_label)
            .with_variant(ButtonVariant::Link)
            .text_color(crate::theme::registry::hsla(link_foreground(
                *style
                    .colors
                    .get("foreground")
                    .expect("link theme needs foreground"),
                disabled_foreground,
                self.disabled,
            )))
            .font_family(typography.family.clone())
            .text_size(px(typography.size))
            .font_weight(FontWeight(typography.weight))
            .disabled(self.disabled)
            .tab_stop(!self.disabled)
            .on_click(move |event, window, cx| {
                if let Some(activation) = &activation {
                    activation(event, window, cx);
                }
                if open_url_on_activate {
                    cx.open_url(&href);
                }
            })
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_defaults_enabled_and_uses_visible_label_as_accessible_name() {
        let link = Link("RustUI", "https://example.com");
        assert!(!link.disabled);
        assert_eq!(link.label.as_ref(), "RustUI");
        assert!(link.accessibility_label.is_none());
        assert_eq!(
            link.resolved_accessibility_label(|_| unreachable!()),
            "RustUI"
        );
    }

    #[test]
    #[should_panic(expected = "Link destination must not be empty")]
    fn rejects_blank_destination() {
        Link("RustUI", " \t\n ");
    }

    #[test]
    fn empty_visible_label_is_allowed_with_nonblank_accessible_name_override() {
        let link = Link("", "https://example.com").accessibility_label("RustUI overview");

        assert_eq!(link.label.as_ref(), "");
        assert_eq!(
            link.resolved_accessibility_label(|_| unreachable!()),
            "RustUI overview"
        );
    }

    #[test]
    #[should_panic(expected = "Link must provide a non-empty accessible name")]
    fn empty_visible_label_requires_accessible_name_override() {
        Link("", "https://example.com").resolved_accessibility_label(|_| unreachable!());
    }

    #[test]
    fn explicit_accessible_name_and_identity_are_preserved() {
        let link = Link("Read more", "https://example.com")
            .accessibility_label("Read the RustUI overview")
            .id("overview-link");
        assert_eq!(
            link.resolved_accessibility_label(|_| unreachable!()),
            "Read the RustUI overview"
        );
        assert_eq!(link.identity().as_str(), "overview-link");
    }

    #[test]
    #[should_panic(expected = "Link identity must not be blank")]
    fn rejects_blank_explicit_identity() {
        Link("Read more", "https://example.com").id(" \t ");
    }

    #[test]
    fn explicit_accessible_name_preserves_nonblank_text_and_visible_link_data() {
        let link = Link("Read more", "https://example.com")
            .accessibility_label("  Read the RustUI overview  ");

        assert_eq!(
            link.resolved_accessibility_label(|_| unreachable!()),
            "  Read the RustUI overview  "
        );
        assert_eq!(link.label.as_ref(), "Read more");
        assert_eq!(link.href.as_ref(), "https://example.com");
    }

    #[test]
    #[should_panic(expected = "Link accessibility label must not be empty")]
    fn rejects_blank_explicit_accessible_name() {
        Link("Read more", "https://example.com").accessibility_label(" \t\n ");
    }

    #[test]
    fn localized_accessible_name_uses_catalog_and_retains_fallbacks() {
        let localized = Link("Open link fixture", "https://example.com")
            .localized_accessibility_label("rustui-link-open-fixture");
        assert_eq!(
            localized.resolved_accessibility_label(|id| {
                assert_eq!(id, "rustui-link-open-fixture");
                "Ouvrir le lien de démonstration".to_owned()
            }),
            "Ouvrir le lien de démonstration"
        );
        assert_eq!(
            localized.resolved_accessibility_label(|id| id.to_owned()),
            "Open link fixture"
        );
        assert_eq!(
            localized.resolved_accessibility_label(|id| format!("  {id} \t")),
            "Open link fixture"
        );
        assert_eq!(
            localized.resolved_accessibility_label(|_| " \t ".to_owned()),
            "Open link fixture"
        );

        let explicit = Link("Open link fixture", "https://example.com")
            .accessibility_label("Open RustUI guide")
            .localized_accessibility_label("rustui-link-open-fixture");
        assert_eq!(
            explicit.resolved_accessibility_label(|_| String::new()),
            "Open RustUI guide"
        );
    }

    #[test]
    #[should_panic(expected = "Link accessibility localization key must not be blank")]
    fn localized_accessible_name_rejects_blank_message_id() {
        Link("Open link fixture", "https://example.com").localized_accessibility_label(" \t ");
    }

    #[test]
    #[should_panic(expected = "Link must provide a non-empty accessible fallback")]
    fn localized_accessible_name_requires_a_nonblank_fallback() {
        Link("", "https://example.com").localized_accessibility_label("rustui-link-open-fixture");
    }

    #[test]
    fn fallback_identity_is_stable_and_uses_both_label_and_url() {
        let first = Link("RustUI", "https://example.com").identity();
        let same = Link("RustUI", "https://example.com").identity();
        let different_url = Link("RustUI", "https://example.org").identity();
        let different_label = Link("Rust", "https://example.com").identity();

        assert_eq!(first, same);
        assert_ne!(first, different_url);
        assert_ne!(first, different_label);
    }

    #[test]
    fn disabled_state_is_configurable() {
        let link = Link("RustUI", "https://example.com").disabled(true);
        assert!(link.disabled);
    }

    #[test]
    fn disabled_link_uses_platform_disabled_text_color() {
        let document =
            crate::theme::parse_theme(include_str!("../../../../../themes/macos27.json")).unwrap();
        let mode = crate::theme::AppearanceMode::Light;
        let selector = crate::components::theme::selector_for_mode(mode);
        let style =
            crate::theme::resolve_component_style_with(&document, "link", "body", &selector, &[])
                .unwrap();
        let button = crate::theme::resolve_button(&document, Some(mode)).unwrap();
        let foreground = style.colors["foreground"];

        assert_eq!(
            link_foreground(foreground, button.disabled_foreground, true),
            button.disabled_foreground
        );
        assert_eq!(
            link_foreground(foreground, button.disabled_foreground, false),
            foreground
        );
    }

    #[test]
    fn url_opening_defaults_on_and_can_be_suppressed_for_local_activation() {
        assert!(Link("RustUI", "https://example.com").open_url_on_activate);
        assert!(
            !Link("Preview", "https://example.com")
                .open_url_on_activate(false)
                .open_url_on_activate
        );
        assert!(
            Link("RustUI", "https://example.com")
                .open_url_on_activate(false)
                .open_url_on_activate(true)
                .open_url_on_activate
        );
    }
}
