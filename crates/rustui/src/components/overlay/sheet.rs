//! Trigger-driven side sheet backed by GPUI Components.

use std::rc::Rc;

use gpui::{
    AnyElement, InteractiveElement as _, IntoElement as _, ParentElement as _, Role, SharedString,
    StatefulInteractiveElement as _, Window, div,
};
use gpui_component::WindowExt as _;
use rustui_core::Identity;

use crate::{
    components::basic::button::Button,
    runtime::AppAction,
    view::{AnyView, EmptyView, RenderContext, RetentionScope, View},
};

#[derive(Clone)]
enum SheetText {
    Literal(SharedString),
    Message(SharedString),
}

impl SheetText {
    fn resolve(&self, context: &RenderContext<'_>) -> SharedString {
        match self {
            Self::Literal(text) => text.clone(),
            Self::Message(key) => SharedString::from(context.localized_message(key.as_ref())),
        }
    }
}

fn first_nonblank_accessibility_label(
    labels: impl IntoIterator<Item = SharedString>,
    identity: SharedString,
) -> SharedString {
    labels
        .into_iter()
        .find(|label| !label.trim().is_empty())
        .unwrap_or(identity)
}

fn first_nonblank_trigger_label(
    key: Option<&SharedString>,
    localized: Option<SharedString>,
    fallback: SharedString,
) -> SharedString {
    localized
        .filter(|label| {
            !label.trim().is_empty() && key.is_none_or(|key| label.trim() != key.trim())
        })
        .unwrap_or(fallback)
}

fn resolved_localized_accessibility_candidate(
    key: &SharedString,
    label: SharedString,
) -> Option<SharedString> {
    (label.trim() != key.trim() && !label.trim().is_empty()).then_some(label)
}

/// Keeps arbitrary RustUI sheet content owned across native builder rerenders.
struct SheetContent {
    content: Rc<AnyView>,
    accessibility_id: SharedString,
    accessibility_label: SharedString,
}

fn update_sheet_accessibility_label(content: &mut SheetContent, label: &SharedString) {
    content.accessibility_label = label.clone();
}

impl View for SheetContent {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        // GPUI Components does not expose its Sheet surface element. Keep the
        // semantic dialog node around the RustUI body; this cannot assign a
        // role to the upstream Sheet host itself.
        div()
            .id(self.accessibility_id.clone())
            .role(Role::Dialog)
            .aria_label(self.accessibility_label.clone())
            .child(self.content.into_element(context))
            .into_any_element()
    }
}

/// A trigger that opens one GPUI Components side sheet in its window.
///
/// The sheet uses the upstream default right placement and sizing. This first
/// slice has no controlled visibility or owner-driven close operation.
pub struct SheetView {
    id: Identity,
    trigger: SheetText,
    localized_trigger: Option<SharedString>,
    title: Option<SheetText>,
    accessibility_label: Option<SheetText>,
    content: Option<Rc<AnyView>>,
}

impl SheetView {
    pub fn new(id: impl Into<Identity>, trigger: impl Into<SharedString>) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "Sheet identity must be nonempty"
        );
        let trigger = trigger.into();
        assert!(
            !trigger.trim().is_empty(),
            "Sheet trigger requires a non-empty visible label"
        );
        Self {
            id,
            trigger: SheetText::Literal(trigger),
            localized_trigger: None,
            title: None,
            accessibility_label: None,
            content: None,
        }
    }

    /// Resolves the trigger label from the active RustUI message catalog.
    pub fn localized_trigger(mut self, message_key: impl Into<SharedString>) -> Self {
        let message_key = message_key.into();
        assert!(
            !message_key.trim().is_empty(),
            "Sheet trigger requires a non-empty localization key"
        );
        self.localized_trigger = Some(message_key);
        self
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        let title = title.into();
        assert!(!title.trim().is_empty(), "Sheet title must not be blank");
        self.title = Some(SheetText::Literal(title));
        self
    }

    /// Resolves the visible title from the active RustUI message catalog when opened.
    pub fn localized_title(mut self, message_key: impl Into<SharedString>) -> Self {
        let message_key = message_key.into();
        assert!(
            !message_key.trim().is_empty(),
            "Sheet title requires a non-empty localization key"
        );
        self.title = Some(SheetText::Message(message_key));
        self
    }

    /// Names the sheet content for assistive technology.
    ///
    /// By default, the visible title is used when present; otherwise the
    /// trigger label is used.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "Sheet accessibility label must not be blank"
        );
        self.accessibility_label = Some(SheetText::Literal(label));
        self
    }

    /// Resolves the sheet's accessible name from the active RustUI message catalog.
    pub fn localized_accessibility_label(mut self, message_key: impl Into<SharedString>) -> Self {
        let message_key = message_key.into();
        assert!(
            !message_key.trim().is_empty(),
            "Sheet accessibility label localization key must not be blank"
        );
        self.accessibility_label = Some(SheetText::Message(message_key));
        self
    }

    #[cfg(test)]
    fn accessibility_label_text(&self) -> SheetText {
        self.accessibility_label
            .clone()
            .or_else(|| self.title.clone())
            .or_else(|| self.localized_trigger.clone().map(SheetText::Message))
            .unwrap_or_else(|| self.trigger.clone())
    }

    fn resolve_accessibility_label(&self, context: &RenderContext<'_>) -> SharedString {
        first_nonblank_accessibility_label(
            self.accessibility_label
                .iter()
                .chain(self.title.iter())
                .filter_map(|label| match label {
                    SheetText::Literal(_) => Some(label.resolve(context)),
                    SheetText::Message(key) => {
                        resolved_localized_accessibility_candidate(key, label.resolve(context))
                    }
                })
                .chain(std::iter::once(self.resolve_trigger_label(context))),
            self.id.as_str().into(),
        )
    }

    fn resolve_trigger_label(&self, context: &RenderContext<'_>) -> SharedString {
        first_nonblank_trigger_label(
            self.localized_trigger.as_ref(),
            self.localized_trigger
                .as_ref()
                .map(|key| SharedString::from(context.localized_message(key.as_ref()))),
            self.trigger.resolve(context),
        )
    }

    /// Adds arbitrary RustUI content to the sheet body.
    pub fn content<V: View>(mut self, content: V) -> Self {
        self.content = Some(Rc::new(AnyView::new(content)));
        self
    }
}

/// Creates a trigger-driven side sheet using upstream default placement.
#[allow(non_snake_case)]
pub fn Sheet(id: impl Into<Identity>, trigger: impl Into<SharedString>) -> SheetView {
    SheetView::new(id, trigger)
}

impl View for SheetView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        context.with_element_scope(RetentionScope::Key(self.id.clone()), |context| {
            let accessibility_label = self.resolve_accessibility_label(context);
            let trigger = self.resolve_trigger_label(context);
            let content = self.content.as_ref().map(|content| {
                context
                    .retained_component_with_props(
                        Identity::from(format!("{}:content", self.id)),
                        || SheetContent {
                            content: Rc::clone(content),
                            accessibility_id: format!("{}:dialog", self.id).into(),
                            accessibility_label: accessibility_label.clone(),
                        },
                        &accessibility_label,
                        update_sheet_accessibility_label,
                    )
                    .0
            });
            let title = self.title.as_ref().map(|title| title.resolve(context));
            let title_id = format!("{}:title", self.id);
            let id = self.id.clone();
            let open = AppAction::with_window_action(move |_, window: &mut Window, cx| {
                if super::drawer::is_active(window, cx) {
                    return;
                }
                let content = content.clone();
                let title = title.clone();
                let title_id = title_id.clone();
                window.open_sheet(cx, move |mut sheet, _, _| {
                    if let Some(title) = title.clone() {
                        let accessible_title = div()
                            .id(title_id.clone())
                            .role(Role::Label)
                            .aria_label(title.clone())
                            .child(title)
                            .into_any_element();
                        sheet = sheet.title(accessible_title);
                    }
                    if let Some(body) = content.clone() {
                        sheet = sheet.child(body.clone());
                    }
                    sheet
                });
            });

            Button(trigger, open)
                .id(format!("{}:trigger", id))
                .into_element(context)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sheet_requires_a_nonempty_stable_identity() {
        assert_eq!(Sheet("details", "Open").id.as_str(), "details");
        assert!(std::panic::catch_unwind(|| Sheet(" ", "Open")).is_err());
    }

    #[test]
    #[should_panic(expected = "Sheet trigger requires a non-empty visible label")]
    fn blank_visible_trigger_is_rejected() {
        Sheet("details", " \t");
    }

    #[test]
    #[should_panic(expected = "Sheet trigger requires a non-empty localization key")]
    fn blank_localized_trigger_key_is_rejected() {
        Sheet("details", "Open").localized_trigger(" \t");
    }

    #[test]
    #[should_panic(expected = "Sheet title requires a non-empty localization key")]
    fn blank_localized_title_key_is_rejected() {
        Sheet("details", "Open").localized_title(" \t");
    }

    #[test]
    #[should_panic(expected = "Sheet title must not be blank")]
    fn blank_literal_title_is_rejected() {
        Sheet("details", "Open").title(" \t");
    }

    #[test]
    fn blank_localized_trigger_falls_back_to_its_visible_label() {
        assert_eq!(
            first_nonblank_trigger_label(None, Some(" \t".into()), "Open details".into()),
            "Open details"
        );
    }

    #[test]
    fn title_and_content_are_optional() {
        let sheet = Sheet("details", "Open").title("Details").content(EmptyView);

        assert!(sheet.title.is_some());
        assert!(sheet.content.is_some());
        assert!(matches!(
            sheet.accessibility_label_text(),
            SheetText::Literal(label) if label.as_ref() == "Details"
        ));
    }

    #[test]
    fn trigger_and_title_can_be_localized() {
        let sheet = Sheet("details", "Open")
            .localized_trigger("sheet.open")
            .localized_title("sheet.title");

        assert!(matches!(
            sheet.localized_trigger,
            Some(key) if key.as_ref() == "sheet.open"
        ));
        assert!(matches!(
            sheet.title,
            Some(SheetText::Message(key)) if key.as_ref() == "sheet.title"
        ));
    }

    #[test]
    fn retained_sheet_accessible_name_tracks_catalog_changes() {
        let mut content = SheetContent {
            content: Rc::new(AnyView::new(EmptyView)),
            accessibility_id: "details:dialog".into(),
            accessibility_label: "Details".into(),
        };

        update_sheet_accessibility_label(&mut content, &"Détails".into());

        assert_eq!(content.accessibility_label.as_ref(), "Détails");
    }

    #[test]
    fn trigger_names_sheet_when_title_is_omitted() {
        let sheet = Sheet("details", "Open details");

        assert!(matches!(
            sheet.accessibility_label_text(),
            SheetText::Literal(label) if label.as_ref() == "Open details"
        ));
    }

    #[test]
    fn localized_title_or_trigger_is_the_accessible_name_fallback() {
        let titled = Sheet("details", "Open")
            .localized_trigger("sheet.open")
            .localized_title("sheet.title");
        assert!(matches!(
            titled.accessibility_label_text(),
            SheetText::Message(key) if key.as_ref() == "sheet.title"
        ));

        let untitled = Sheet("details", "Open").localized_trigger("sheet.open");
        assert!(matches!(
            untitled.accessibility_label_text(),
            SheetText::Message(key) if key.as_ref() == "sheet.open"
        ));
        assert_eq!(untitled.localized_trigger.as_deref(), Some("sheet.open"));
    }

    #[test]
    #[should_panic(expected = "Sheet accessibility label must not be blank")]
    fn rejects_blank_explicit_accessible_name() {
        Sheet("details", "Open").accessibility_label(" \t ");
    }

    #[test]
    #[should_panic(expected = "Sheet accessibility label localization key must not be blank")]
    fn rejects_blank_localized_accessible_name_key() {
        Sheet("details", "Open").localized_accessibility_label(" \t ");
    }

    #[test]
    fn preserves_nonblank_explicit_accessible_name_verbatim() {
        let sheet = Sheet("details", "Open").accessibility_label("  Details panel  ");

        assert!(matches!(
            sheet.accessibility_label_text(),
            SheetText::Literal(label) if label.as_ref() == "  Details panel  "
        ));
    }

    #[test]
    fn blank_resolved_accessible_name_falls_back_to_the_next_candidate() {
        assert_eq!(
            first_nonblank_accessibility_label([" \t ".into(), "Details".into()], "details".into()),
            "Details"
        );
    }

    #[test]
    fn unresolved_localized_accessible_name_falls_back_to_visible_title() {
        let key: SharedString = "sheet.accessibility.details".into();
        let unresolved = resolved_localized_accessibility_candidate(&key, key.clone());

        assert_eq!(
            first_nonblank_accessibility_label(
                [unresolved.unwrap_or_default(), "Visible title".into()],
                "details".into(),
            ),
            "Visible title"
        );
    }

    #[test]
    fn padded_untranslated_accessibility_key_falls_back_to_visible_title() {
        let key: SharedString = "sheet.accessibility.details".into();
        let padded_echo: SharedString = format!(" \t{key}\n").into();
        let unresolved = resolved_localized_accessibility_candidate(&key, padded_echo);

        assert_eq!(
            first_nonblank_accessibility_label(
                [unresolved.unwrap_or_default(), "Visible title".into()],
                "details".into(),
            ),
            "Visible title"
        );
    }

    #[test]
    fn all_blank_resolved_accessible_name_candidates_fall_back_to_identity() {
        assert_eq!(
            first_nonblank_accessibility_label([" ".into(), "\n".into()], "details".into()),
            "details"
        );
    }

    #[test]
    fn unresolved_localized_trigger_key_falls_back_to_visible_trigger() {
        let key: SharedString = "sheet.open_details".into();

        assert_eq!(
            first_nonblank_trigger_label(Some(&key), Some(key.clone()), "Open".into()),
            "Open"
        );
        assert_eq!(
            first_nonblank_trigger_label(
                Some(&key),
                Some(format!(" \t{key}\n").into()),
                "Open".into(),
            ),
            "Open"
        );
        assert_eq!(
            first_nonblank_trigger_label(Some(&key), Some("Ouvrir".into()), "Open".into()),
            "Ouvrir"
        );
    }

    #[test]
    fn accessible_name_prefers_explicit_label_then_title_then_trigger() {
        let explicit = Sheet("details", "Open")
            .title("Visible title")
            .accessibility_label("Explicit name");
        assert!(matches!(
            explicit.accessibility_label_text(),
            SheetText::Literal(label) if label.as_ref() == "Explicit name"
        ));

        let localized = explicit.localized_accessibility_label("sheet.details.name");
        assert!(matches!(
            localized.accessibility_label_text(),
            SheetText::Message(key) if key.as_ref() == "sheet.details.name"
        ));
    }
}
