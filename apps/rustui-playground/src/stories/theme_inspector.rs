//! Read-only preview of the public theme-document export boundary.

#[cfg(not(target_family = "wasm"))]
#[cfg(not(target_family = "wasm"))]
use rustui::theme::load_theme;
#[cfg(target_family = "wasm")]
use rustui::theme::parse_theme;
use rustui::{
    prelude::*,
    theme::{AppearanceMode, assess_theme_coverage, resolve_button, serialize_theme},
};

pub fn demo() -> impl View {
    Component("theme-document-inspector", ThemeInspector::new)
}

#[component]
struct ThemeInspector {
    summary: String,
    preview: String,
}

impl ThemeInspector {
    fn new() -> Self {
        let mut inspector = Self::default();
        inspector.reload();
        inspector
    }

    fn reload(&mut self) {
        match inspect_theme() {
            Ok((summary, preview)) => {
                self.summary = summary;
                self.preview = preview;
            }
            Err(error) => {
                self.summary = format!("Theme inspection failed: {error}");
                self.preview.clear();
            }
        }
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
    fn body(&self) -> impl View {
        VStack::new()
            .spacing(8.0)
            .child(Text("Theme document inspector").font_weight(TextWeight::Semibold))
            .child(Text("Read-only normalized export preview. Reload reads the file; it does not apply a theme."))
            .child(
                Button("Reload theme inspection", self.action(Self::reload))
                    .id("theme-inspector-reload")
                    .accessibility_label(format!("Reload theme inspection. {}", self.summary)),
            )
            .child(Text(self.summary.clone()).font_size(12.0))
            .child(Text("First 18 lines of normalized JSON"))
            .child(Text(self.preview.clone()).font_size(11.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
