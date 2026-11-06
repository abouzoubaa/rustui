use super::gallery;
use super::*;

impl Default for CompiledPreviewGallery {
    fn default() -> Self {
        let mut registry = PreviewRegistry::new();
        gallery::register_basic_specimens(&mut registry);

        gallery::register_text_specimens(&mut registry);

        gallery::register_feedback_specimens(&mut registry);

        gallery::register_data_specimens(&mut registry);

        gallery::register_graph_specimens(&mut registry);
        gallery::register_canvas_specimens(&mut registry);

        gallery::register_overlay_specimens(&mut registry);

        gallery::register_layout_specimens(&mut registry);

        let mut result = Self {
            reference_session: None,
            #[cfg(target_family = "wasm")]
            reference_bridge: None,
            host: PreviewHost::new(registry),
            stamp_lease: Rc::new(RefCell::new(None)),
            text: "Preview text".into(),
            text_field_placeholder: "Preview text input".into(),
            search_field_placeholder: "Search RustUI".into(),
            text_editor_placeholder: "Write a note".into(),
            secure_field_placeholder: "Enter a password".into(),
            secure_value: String::new(),
            token_values: Vec::new(),
            token_field_placeholder: "Add recipients".into(),
            enabled: true,
            amount: 0.375,
            amount_draft: "0.375".into(),
            number_drafts: BTreeMap::new(),
            stepper_count: PREVIEW_STEPPER_DEFAULT,
            stale_amount_callback: None,
            activations: 0,
            renders: 0,
            status: "Mounted TextField".into(),
            status_presentation: None,
            descriptor_json: String::new(),
        };
        result.mount("text-field");
        result.export_descriptor();
        result
    }
}
