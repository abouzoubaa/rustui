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

#[gpui::test]
fn validation_description_reaches_the_editor_accessibility_node(cx: &mut gpui::TestAppContext) {
    use gpui::{
        AppContext as _, Context, Entity, InteractiveElement as _, IntoElement, ParentElement as _,
        Render, Window, div,
    };
    use gpui_component::input::{Input, InputState};

    struct InputProbe {
        input: Entity<InputState>,
    }

    impl Render for InputProbe {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().id("input-wrapper").child(
                Input::new(&self.input).aria_description("Validation error: required"),
            )
        }
    }

    cx.update(gpui_component::init);
    let (_, window) = cx.add_window_view(|window, cx| InputProbe {
        input: cx.new(|cx| InputState::new(window, cx)),
    });
    window.update(|window, cx| window.draw(cx).clear(cx));
    let tree = window.update(|window, _| window.debug_a11y_tree_json().unwrap());
    let tree: serde_json::Value = serde_json::from_str(&tree).unwrap();
    let nodes = tree["nodes"].as_object().unwrap();
    assert!(nodes.values().any(|node| {
        node["aria"]["role"] == "TextInput"
            && node["aria"]["description"] == "Validation error: required"
    }), "description must be on the editor-owned TextInput node: {tree}");
}
