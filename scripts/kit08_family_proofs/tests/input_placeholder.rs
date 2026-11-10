use gpui::hsla;
use gpui_base::{SemanticThemeTokens, input::InputEditorStyle};

#[test]
fn placeholder_override_survives_palette_resolution() {
    let color = hsla(0.2, 0.3, 0.4, 1.0);
    let style = InputEditorStyle {
        placeholder_foreground: Some(color),
        ..Default::default()
    };

    assert_eq!(
        style
            .resolved(&SemanticThemeTokens::default())
            .placeholder_foreground,
        Some(color)
    );
    assert_eq!(
        InputEditorStyle::default()
            .resolved(&SemanticThemeTokens::default())
            .placeholder_foreground,
        None
    );
}
