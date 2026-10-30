//! Read-only preview of the public theme-document export boundary.

#[cfg(not(target_family = "wasm"))]
#[cfg(not(target_family = "wasm"))]
use rustui::theme::load_theme;
#[cfg(target_family = "wasm")]
use rustui::theme::parse_theme;
use rustui::{
    localization::MessageArguments,
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

#[derive(Clone)]
struct SummaryLine {
    key: &'static str,
    arguments: MessageArguments,
}

#[derive(Default)]
struct InspectorState {
    summary: String,
    localized_summary: Vec<SummaryLine>,
    preview: String,
    error: Option<String>,
    epoch: u64,
    #[cfg(target_family = "wasm")]
    task: Option<Pin<Box<dyn Future<Output = ()>>>>,
}

impl InspectorState {
    fn apply(&mut self, result: Result<(String, Vec<SummaryLine>, String), String>) {
        match result {
            Ok((summary, localized_summary, preview)) => {
                self.summary = summary;
                self.localized_summary = localized_summary;
                self.preview = preview;
                self.error = None;
            }
            Err(error) => {
                self.error = Some(error);
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

fn inspect_theme() -> Result<(String, Vec<SummaryLine>, String), String> {
    #[cfg(target_family = "wasm")]
    let document = parse_theme(include_str!("../../../../themes/macos27.json"))
        .map_err(|error| error.to_string())?;
    #[cfg(not(target_family = "wasm"))]
    let path = crate::playground_resource_path("themes/macos27.json");
    #[cfg(not(target_family = "wasm"))]
    let document = load_theme(path).map_err(|error| error.to_string())?;
    inspect_document(document)
}

fn inspect_document(document: ThemeDocument) -> Result<(String, Vec<SummaryLine>, String), String> {
    let json = serialize_theme(&document).map_err(|error| error.to_string())?;
    let mut summary = format!(
        "Profile {} · format {} · {} component records · {} exported bytes",
        document.profile,
        document.format_version,
        document.components.len(),
        json.len(),
    );
    let mut localized_summary = vec![SummaryLine {
        key: "lab-theme-inspector-profile",
        arguments: MessageArguments::new()
            .with("profile", document.profile.to_string())
            .with("format", document.format_version.to_string())
            .with("components", document.components.len().to_string())
            .with("bytes", json.len().to_string()),
    }];
    let coverage = assess_theme_coverage(&document);
    summary.push_str(&format!(
        "\nRenderer coverage: {}/{} style rules · {} uncovered · {} reference-only records · {} not-applicable records",
        coverage.covered_rules,
        coverage.total_rules,
        coverage.uncovered_rules.len(),
        coverage.reference_only_records.len(),
        coverage.not_applicable_records.len(),
    ));
    localized_summary.push(SummaryLine {
        key: "lab-theme-inspector-coverage",
        arguments: MessageArguments::new()
            .with("covered", coverage.covered_rules.to_string())
            .with("total", coverage.total_rules.to_string())
            .with("uncovered", coverage.uncovered_rules.len().to_string())
            .with(
                "reference",
                coverage.reference_only_records.len().to_string(),
            )
            .with(
                "inapplicable",
                coverage.not_applicable_records.len().to_string(),
            ),
    });
    for uncovered in coverage.uncovered_rules.iter().take(8) {
        summary.push_str(&format!(
            "\nUncovered: {}/{} ({:?}) — {}",
            uncovered.component, uncovered.part, uncovered.appearance, uncovered.reason,
        ));
        let arguments = MessageArguments::new()
            .with("component", uncovered.component.to_string())
            .with("part", uncovered.part.to_string())
            .with("appearance", format!("{:?}", uncovered.appearance));
        let (key, arguments) = if uncovered.reason == "no renderer capability for this rule" {
            ("lab-theme-inspector-uncovered-no-renderer", arguments)
        } else if let Some(properties) = uncovered.reason.strip_prefix("unapplied properties: ") {
            (
                "lab-theme-inspector-uncovered-properties",
                arguments.with("properties", properties.to_owned()),
            )
        } else {
            (
                "lab-theme-inspector-uncovered",
                arguments.with("reason", uncovered.reason.to_string()),
            )
        };
        localized_summary.push(SummaryLine { key, arguments });
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
        localized_summary.push(SummaryLine {
            key: match mode {
                AppearanceMode::Light => "lab-theme-inspector-button-light",
                AppearanceMode::Dark => "lab-theme-inspector-button-dark",
            },
            arguments: MessageArguments::new()
                .with("width", button.width.to_string())
                .with("height", button.height.to_string())
                .with("font", button.font_size.to_string())
                .with("padding", button.horizontal_padding.to_string())
                .with("radius", button.corner_radius.to_string()),
        });
    }
    let preview = json.lines().take(18).collect::<Vec<_>>().join("\n");
    Ok((summary, localized_summary, preview))
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
        let localized_summary = state.localized_summary.clone();
        let preview = state.preview.clone();
        let error = state.error.clone();
        drop(state);
        let error = match error {
            Some(error) => LocalizedText("lab-theme-inspector-error")
                .with_arguments(MessageArguments::new().with("error", error)),
            None => Text(""),
        };
        let mut content = VStack::new()
            .spacing(8.0)
            .child(LocalizedText("lab-theme-inspector-title").font_weight(TextWeight::Semibold))
            .child(LocalizedText("lab-theme-inspector-description"))
            .child(
                Button("Reload theme inspection", self.reload_action())
                    .localized_label("lab-theme-inspector-reload")
                    .id("theme-inspector-reload")
                    .accessibility_label(format!("Reload theme inspection. {summary}")),
            )
            .child(error.font_size(12.0));
        for line in localized_summary {
            content = content.child(
                LocalizedText(line.key)
                    .with_arguments(line.arguments)
                    .font_size(12.0),
            );
        }
        content
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
                "lab-theme-inspector-profile",
                "lab-theme-inspector-coverage",
                "lab-theme-inspector-uncovered",
                "lab-theme-inspector-uncovered-no-renderer",
                "lab-theme-inspector-uncovered-properties",
                "lab-theme-inspector-button-light",
                "lab-theme-inspector-button-dark",
                "lab-theme-inspector-error",
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
        state.apply(Ok((
            "valid summary".into(),
            Vec::new(),
            "valid preview".into(),
        )));
        state.apply(Err("invalid JSON".into()));
        assert_eq!(state.summary, "valid summary");
        assert_eq!(state.preview, "valid preview");
        assert!(state.error.as_deref().unwrap().contains("invalid JSON"));
        state.apply(Ok(("recovered".into(), Vec::new(), "new preview".into())));
        assert_eq!(state.summary, "recovered");
        assert_eq!(state.preview, "new preview");
        assert!(state.error.is_none());
    }

    #[test]
    fn inspection_loads_current_theme_and_resolves_both_appearances() {
        let (summary, localized_summary, preview) =
            inspect_theme().expect("current theme is inspectable");
        assert!(!localized_summary.is_empty());
        assert!(
            localized_summary
                .iter()
                .any(|line| line.key == "lab-theme-inspector-uncovered-no-renderer")
        );
        assert!(
            localized_summary
                .iter()
                .any(|line| line.key == "lab-theme-inspector-uncovered-properties")
        );
        let catalogs = super::super::locale_selector::catalogs();
        for language in ["en", "fr", "ar"] {
            let locale = rustui::localization::Locale::parse(language).unwrap();
            let catalog = catalogs.get(&locale).unwrap();
            for line in &localized_summary {
                let template = catalog.get(line.key).unwrap();
                rustui::localization::interpolate(template, &line.arguments).unwrap();
            }
        }
        assert!(summary.contains("Light Button:"));
        assert!(summary.contains("Dark Button:"));
        assert!(summary.contains("Renderer coverage:"));
        assert!(preview.contains("rustui-appearance"));
        assert!(preview.lines().count() <= 18);
    }
}
