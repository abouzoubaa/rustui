//! Read-only preview of the public theme-document export boundary.

#[cfg(not(target_family = "wasm"))]
#[cfg(not(target_family = "wasm"))]
use rustui::theme::load_theme;
#[cfg(target_family = "wasm")]
use rustui::theme::parse_theme;
use rustui::{
    prelude::*,
    theme::{
        AppearanceMode, ThemeDocument, assess_theme_coverage, resolve_button, serialize_theme,
    },
};
use std::{cell::RefCell, rc::Rc};
#[cfg(target_family = "wasm")]
use std::{future::Future, pin::Pin};

pub fn demo() -> impl View {
    Component("theme-document-inspector", ThemeInspector::new)
}

struct ThemeInspector {
    state: Rc<RefCell<InspectorState>>,
}

#[derive(Default)]
struct InspectorState {
    summary: String,
    preview: String,
    error: Option<String>,
    epoch: u64,
    #[cfg(target_family = "wasm")]
    task: Option<Pin<Box<dyn Future<Output = ()>>>>,
}

impl InspectorState {
    fn apply(&mut self, result: Result<(String, String), String>) {
        match result {
            Ok((summary, preview)) => {
                self.summary = summary;
                self.preview = preview;
                self.error = None;
            }
            Err(error) => {
                self.error = Some(format!("Theme inspection failed: {error}"));
            }
        }
    }
}

impl ThemeInspector {
    fn new() -> Self {
        let mut state = InspectorState::default();
        state.apply(inspect_theme());
        Self {
            state: Rc::new(RefCell::new(state)),
        }
    }

    fn reload_action(&self) -> AppAction {
        let weak = Rc::downgrade(&self.state);
        let epoch = self.state.borrow().epoch;
        AppAction::new(move |cx| {
            let Some(state) = weak.upgrade() else {
                return;
            };
            if state.borrow().epoch != epoch {
                return;
            }
            #[cfg(not(target_family = "wasm"))]
            {
                state.borrow_mut().apply(inspect_theme());
                cx.refresh_windows();
            }
            #[cfg(target_family = "wasm")]
            {
                let generation = {
                    let mut state = state.borrow_mut();
                    state.epoch = state
                        .epoch
                        .checked_add(1)
                        .expect("theme inspector epoch exhausted");
                    state.epoch
                };
                let weak = Rc::downgrade(&state);
                let task = cx.spawn(async move |cx| {
                    let result = super::foundation_additions::load_web_theme()
                        .await
                        .and_then(|source| parse_theme(&source).map_err(|error| error.to_string()))
                        .and_then(inspect_document);
                    let Some(state) = weak.upgrade() else {
                        return;
                    };
                    cx.update(|cx| {
                        let mut state = state.borrow_mut();
                        if state.epoch != generation {
                            return;
                        }
                        state.apply(result);
                        cx.refresh_windows();
                    });
                });
                state.borrow_mut().task = Some(Box::pin(task));
                cx.refresh_windows();
            }
        })
    }
}

fn inspect_theme() -> Result<(String, String), String> {
    #[cfg(target_family = "wasm")]
    let document = parse_theme(include_str!("../../../../themes/macos27.json"))
        .map_err(|error| error.to_string())?;
    #[cfg(not(target_family = "wasm"))]
    let path = crate::playground_resource_path("themes/macos27.json");
    #[cfg(not(target_family = "wasm"))]
    let document = load_theme(path).map_err(|error| error.to_string())?;
    inspect_document(document)
}

fn inspect_document(document: ThemeDocument) -> Result<(String, String), String> {
    let json = serialize_theme(&document).map_err(|error| error.to_string())?;
    let mut summary = format!(
        "Profile {} · format {} · {} component records · {} exported bytes",
        document.profile,
        document.format_version,
        document.components.len(),
        json.len(),
    );
    let coverage = assess_theme_coverage(&document);
    summary.push_str(&format!(
        "\nRenderer coverage: {}/{} style rules · {} uncovered · {} reference-only records · {} not-applicable records",
        coverage.covered_rules,
        coverage.total_rules,
        coverage.uncovered_rules.len(),
        coverage.reference_only_records.len(),
        coverage.not_applicable_records.len(),
    ));
    for uncovered in coverage.uncovered_rules.iter().take(8) {
        summary.push_str(&format!(
            "\nUncovered: {}/{} ({:?}) — {}",
            uncovered.component, uncovered.part, uncovered.appearance, uncovered.reason,
        ));
    }
    for mode in [AppearanceMode::Light, AppearanceMode::Dark] {
        let button = resolve_button(&document, Some(mode)).map_err(|error| error.to_string())?;
        summary.push_str(&format!(
            "\n{mode:?} Button: {} × {} px · font {} px · padding {} px · radius {} px",
            button.width,
            button.height,
            button.font_size,
            button.horizontal_padding,
            button.corner_radius,
        ));
    }
    let preview = json.lines().take(18).collect::<Vec<_>>().join("\n");
    Ok((summary, preview))
}

impl View for ThemeInspector {
    fn on_unmount(&mut self, _: &mut rustui::App) {
        let mut state = self.state.borrow_mut();
        state.epoch = state
            .epoch
            .checked_add(1)
            .expect("theme inspector epoch exhausted");
        #[cfg(target_family = "wasm")]
        state.task.take();
    }

    fn body(&self) -> impl View {
        let state = self.state.borrow();
        let summary = state.summary.clone();
        let preview = state.preview.clone();
        let error = state.error.clone().unwrap_or_default();
        drop(state);
        VStack::new()
            .spacing(8.0)
            .child(LocalizedText("lab-theme-inspector-title").font_weight(TextWeight::Semibold))
            .child(LocalizedText("lab-theme-inspector-description"))
            .child(
                Button("Reload theme inspection", self.reload_action())
                    .localized_label("lab-theme-inspector-reload")
                    .id("theme-inspector-reload")
                    .accessibility_label(format!("Reload theme inspection. {summary}")),
            )
            .child(Text(error).font_size(12.0))
            .child(Text(summary).font_size(12.0))
            .child(LocalizedText("lab-theme-inspector-preview"))
            .child(Text(preview).font_size(11.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inspector_controls_exist_in_all_supported_locales() {
        let catalogs = super::super::locale_selector::catalogs();
        for language in ["en", "fr", "ar"] {
            let locale = rustui::localization::Locale::parse(language).unwrap();
            let catalog = catalogs.get(&locale).unwrap();
            for key in [
                "lab-theme-inspector-title",
                "lab-theme-inspector-description",
                "lab-theme-inspector-reload",
                "lab-theme-inspector-preview",
            ] {
                assert!(
                    catalog
                        .get(key)
                        .is_some_and(|value| !value.trim().is_empty()),
                    "{language}: {key}"
                );
            }
        }
    }

    #[test]
    fn invalid_inspection_preserves_last_valid_preview_and_recovers() {
        let mut state = InspectorState::default();
        state.apply(Ok(("valid summary".into(), "valid preview".into())));
        state.apply(Err("invalid JSON".into()));
        assert_eq!(state.summary, "valid summary");
        assert_eq!(state.preview, "valid preview");
        assert!(state.error.as_deref().unwrap().contains("invalid JSON"));
        state.apply(Ok(("recovered".into(), "new preview".into())));
        assert_eq!(state.summary, "recovered");
        assert_eq!(state.preview, "new preview");
        assert!(state.error.is_none());
    }

    #[test]
    fn inspection_loads_current_theme_and_resolves_both_appearances() {
        let (summary, preview) = inspect_theme().expect("current theme is inspectable");
        assert!(summary.contains("Light Button:"));
        assert!(summary.contains("Dark Button:"));
        assert!(summary.contains("Renderer coverage:"));
        assert!(preview.contains("rustui-appearance"));
        assert!(preview.lines().count() <= 18);
    }
}
