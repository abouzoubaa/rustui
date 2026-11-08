use super::*;

#[test]
fn toolbar_specimen_maps_signal_only_actions_and_round_trips() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_toolbar_specimen();
    let component_id = ComponentId::new("toolbar-specimen").unwrap();
    let primary_id = EventId::new("primary").unwrap();
    let secondary_id = EventId::new("secondary").unwrap();
    let schema = &gallery.host.catalog().components[&component_id];
    for name in ["primary_label", "secondary_label"] {
        let property = &schema.properties[&PropertyId::new(name).unwrap()];
        assert_eq!(property.kind, PropertyKind::String);
        assert_eq!(property.min_length, Some(1));
        assert_eq!(property.max_length, Some(PREVIEW_TOOLBAR_LABEL_MAX_LENGTH));
    }
    let disabled_id = PropertyId::new("secondary_disabled").unwrap();
    assert_eq!(schema.properties[&disabled_id].kind, PropertyKind::Boolean);
    assert_eq!(schema.events.len(), 2);
    assert!(schema.events[&primary_id].payload.is_empty());
    assert!(schema.events[&secondary_id].payload.is_empty());

    let mounted = gallery.host.snapshot().unwrap().clone();
    assert_eq!(
        mounted.preview.document.actions[&primary_id],
        ActionRef::new("toolbar_primary").unwrap()
    );
    assert_eq!(
        mounted.preview.document.actions[&secondary_id],
        ActionRef::new("toolbar_secondary").unwrap()
    );
    assert_eq!(
        gallery
            .host
            .dispatch(&mounted.signal(primary_id.clone()))
            .unwrap(),
        ActionRef::new("toolbar_primary").unwrap()
    );
    assert_eq!(
        gallery
            .host
            .dispatch(&mounted.signal(secondary_id.clone()))
            .unwrap(),
        ActionRef::new("toolbar_secondary").unwrap()
    );

    let mut customized = mounted.preview.document.clone();
    customized.properties.insert(
        PropertyId::new("primary_label").unwrap(),
        PropertyValue::String("Apply".into()),
    );
    customized.properties.insert(
        PropertyId::new("secondary_label").unwrap(),
        PropertyValue::String("Later".into()),
    );
    customized
        .properties
        .insert(disabled_id.clone(), PropertyValue::Boolean(true));
    gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();
    let imported = gallery.host.snapshot().unwrap();
    assert_eq!(
        imported.preview.properties[&PropertyId::new("primary_label").unwrap()],
        PropertyValue::String("Apply".into())
    );
    assert_eq!(
        imported.preview.properties[&PropertyId::new("secondary_label").unwrap()],
        PropertyValue::String("Later".into())
    );
    assert_eq!(
        imported.preview.properties[&disabled_id],
        PropertyValue::Boolean(true)
    );

    let before = imported.clone();
    let mut invalid = before.preview.document.clone();
    invalid.properties.insert(
        PropertyId::new("primary_label").unwrap(),
        PropertyValue::String("  ".into()),
    );
    gallery.descriptor_json = encode_preview(&invalid, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();
    assert!(
        gallery
            .status
            .contains("Toolbar primary_label must not be blank")
    );
    assert_eq!(gallery.host.snapshot().unwrap().stamp, before.stamp);
}

#[test]
fn image_specimen_uses_bounded_label_and_constrained_fit_round_trip() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_image_specimen();

    let component_id = ComponentId::new("image-specimen").unwrap();
    let label_id = PropertyId::new("accessibility_label").unwrap();
    let fit_id = PropertyId::new("fit").unwrap();
    let schema = &gallery.host.catalog().components[&component_id];
    assert_eq!(schema.properties.len(), 2);
    assert_eq!(schema.properties[&label_id].kind, PropertyKind::String);
    assert_eq!(schema.properties[&label_id].min_length, Some(1));
    assert_eq!(
        schema.properties[&label_id].max_length,
        Some(PREVIEW_IMAGE_LABEL_MAX_LENGTH)
    );
    assert_eq!(
        schema.properties[&label_id].default,
        Some(PropertyValue::String("Purple and cyan sample image".into()))
    );
    assert_eq!(schema.properties[&fit_id].kind, PropertyKind::String);
    assert_eq!(
        schema.properties[&fit_id].default,
        Some(PropertyValue::String("contain".into()))
    );
    assert_eq!(
        schema.properties[&fit_id].allowed_values,
        PREVIEW_IMAGE_FITS
            .iter()
            .map(|fit| (*fit).into())
            .collect::<Vec<String>>()
    );
    assert!(schema.events.is_empty());

    let mounted = gallery.host.snapshot().unwrap();
    assert_eq!(mounted.preview.document.properties.len(), 2);
    assert_eq!(mounted.preview.document.actions.len(), 0);
    assert_eq!(
        mounted.preview.properties[&label_id],
        PropertyValue::String("Purple and cyan sample image".into())
    );
    assert_eq!(
        mounted.preview.properties[&fit_id],
        PropertyValue::String("contain".into())
    );

    let mut customized = mounted.preview.document.clone();
    customized.properties.insert(
        label_id.clone(),
        PropertyValue::String("Sample artwork".into()),
    );
    customized
        .properties
        .insert(fit_id.clone(), PropertyValue::String("cover".into()));
    gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();
    let imported = gallery.host.snapshot().unwrap();
    assert_eq!(
        imported.preview.properties[&label_id],
        PropertyValue::String("Sample artwork".into())
    );
    assert_eq!(
        imported.preview.properties[&fit_id],
        PropertyValue::String("cover".into())
    );

    let before = imported.clone();
    let mut invalid = before.preview.document.clone();
    invalid
        .properties
        .insert(label_id, PropertyValue::String(" \t".into()));
    gallery.descriptor_json = encode_preview(&invalid, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();
    assert!(
        gallery
            .status
            .contains("Image accessibility label must not be blank")
    );
    assert_eq!(gallery.host.snapshot().unwrap().stamp, before.stamp);
    assert_eq!(
        gallery.host.snapshot().unwrap().preview.document,
        before.preview.document
    );
}

#[test]
fn breadcrumb_specimen_dispatches_parent_and_round_trips_fixed_labels() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_breadcrumb_specimen();

    let component_id = ComponentId::new("breadcrumb-specimen").unwrap();
    let parent_id = PropertyId::new("parent_label").unwrap();
    let current_id = PropertyId::new("current_label").unwrap();
    let event_id = EventId::new("parent").unwrap();
    let schema = &gallery.host.catalog().components[&component_id];
    assert_eq!(schema.properties.len(), 2);
    for (id, default) in [(&parent_id, "Components"), (&current_id, "Image")] {
        let property = &schema.properties[id];
        assert_eq!(property.kind, PropertyKind::String);
        assert_eq!(property.min_length, Some(1));
        assert_eq!(
            property.max_length,
            Some(PREVIEW_BREADCRUMB_LABEL_MAX_LENGTH)
        );
        assert_eq!(
            property.default,
            Some(PropertyValue::String(default.into()))
        );
    }
    assert_eq!(schema.events.len(), 1);
    assert!(schema.events[&event_id].payload.is_empty());

    let mounted = gallery.host.snapshot().unwrap().clone();
    assert_eq!(mounted.preview.document.properties.len(), 2);
    assert_eq!(
        mounted.preview.document.actions.get(&event_id),
        Some(&ActionRef::new("breadcrumb_parent").unwrap())
    );
    gallery.navigate_breadcrumb_parent(&mounted.signal(event_id.clone()));
    assert_eq!(gallery.activations, 1);
    assert!(gallery.status.contains("Allowlisted Breadcrumb parent"));

    let mut customized = mounted.preview.document.clone();
    customized.properties.insert(
        parent_id.clone(),
        PropertyValue::String("Design system".into()),
    );
    customized.properties.insert(
        current_id.clone(),
        PropertyValue::String("Components".into()),
    );
    gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();
    let imported = gallery.host.snapshot().unwrap();
    assert_eq!(
        imported.preview.properties[&parent_id],
        PropertyValue::String("Design system".into())
    );
    assert_eq!(
        imported.preview.properties[&current_id],
        PropertyValue::String("Components".into())
    );

    let before = imported.clone();
    for name in ["parent_label", "current_label"] {
        let mut invalid = before.preview.document.clone();
        invalid.properties.insert(
            PropertyId::new(name).unwrap(),
            PropertyValue::String(" \t".into()),
        );
        gallery.descriptor_json = encode_preview(&invalid, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        assert!(
            gallery
                .status
                .contains(&format!("Breadcrumb {name} must not be blank"))
        );
        assert_eq!(gallery.host.snapshot().unwrap().stamp, before.stamp);
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.document,
            before.preview.document
        );
    }
}

#[test]
fn text_specimen_round_trips_bounded_value_without_events_or_actions() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_text_specimen();

    let component_id = ComponentId::new("text-specimen").unwrap();
    let value_id = PropertyId::new("value").unwrap();
    let schema = &gallery.host.catalog().components[&component_id];
    assert_eq!(schema.properties.len(), 1);
    assert_eq!(schema.properties[&value_id].kind, PropertyKind::String);
    assert_eq!(schema.properties[&value_id].min_length, Some(1));
    assert_eq!(
        schema.properties[&value_id].max_length,
        Some(PREVIEW_TEXT_VALUE_MAX_LENGTH)
    );
    assert_eq!(
        schema.properties[&value_id].default,
        Some(PropertyValue::String("Preview text".into()))
    );
    assert!(schema.events.is_empty());

    let mounted = gallery.host.snapshot().unwrap().clone();
    assert_eq!(mounted.preview.document.properties.len(), 1);
    assert!(mounted.preview.document.actions.is_empty());
    assert_eq!(
        mounted.preview.properties[&value_id],
        PropertyValue::String("Preview text".into())
    );

    let mut customized = mounted.preview.document.clone();
    customized.properties.insert(
        value_id.clone(),
        PropertyValue::String("Compiled text preview".into()),
    );
    gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();
    assert_eq!(
        gallery.host.snapshot().unwrap().preview.properties[&value_id],
        PropertyValue::String("Compiled text preview".into())
    );

    let before = gallery.host.snapshot().unwrap().clone();
    let mut invalid = before.preview.document.clone();
    invalid
        .properties
        .insert(value_id.clone(), PropertyValue::String(" \t".into()));
    gallery.descriptor_json = encode_preview(&invalid, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();
    assert!(gallery.status.contains("Text value must not be blank"));
    assert_eq!(gallery.host.snapshot().unwrap().stamp, before.stamp);
    assert_eq!(
        gallery.host.snapshot().unwrap().preview.document,
        before.preview.document
    );
}

#[test]
fn sheet_specimen_round_trips_bounded_strings_and_rejects_blank_imports() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_sheet_specimen();

    let component_id = ComponentId::new("sheet-specimen").unwrap();
    let schema = &gallery.host.catalog().components[&component_id];
    assert_eq!(schema.properties.len(), 3);
    assert!(schema.events.is_empty());
    for (name, default, max_length) in [
        (
            "trigger_label",
            "Open details",
            PREVIEW_SHEET_TRIGGER_MAX_LENGTH,
        ),
        ("title", "Details", PREVIEW_SHEET_TITLE_MAX_LENGTH),
        (
            "body_text",
            "Preview sheet content",
            PREVIEW_SHEET_BODY_MAX_LENGTH,
        ),
    ] {
        let property = &schema.properties[&PropertyId::new(name).unwrap()];
        assert_eq!(property.kind, PropertyKind::String);
        assert_eq!(property.min_length, Some(1));
        assert_eq!(property.max_length, Some(max_length));
        assert_eq!(
            property.default,
            Some(PropertyValue::String(default.into()))
        );
    }

    let mounted = gallery.host.snapshot().unwrap().clone();
    assert_eq!(mounted.preview.document.properties.len(), 3);
    assert!(mounted.preview.document.actions.is_empty());
    let mut customized = mounted.preview.document.clone();
    customized.properties.insert(
        PropertyId::new("trigger_label").unwrap(),
        PropertyValue::String("Open project details".into()),
    );
    customized.properties.insert(
        PropertyId::new("title").unwrap(),
        PropertyValue::String("Project details".into()),
    );
    customized.properties.insert(
        PropertyId::new("body_text").unwrap(),
        PropertyValue::String("Build and test status".into()),
    );
    gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();
    gallery.export_descriptor();
    let exported = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
    for (name, value) in [
        ("trigger_label", "Open project details"),
        ("title", "Project details"),
        ("body_text", "Build and test status"),
    ] {
        assert_eq!(
            exported.properties[&PropertyId::new(name).unwrap()],
            PropertyValue::String(value.into())
        );
    }

    let before = gallery.host.snapshot().unwrap().clone();
    for name in ["trigger_label", "title", "body_text"] {
        let mut invalid = before.preview.document.clone();
        invalid.properties.insert(
            PropertyId::new(name).unwrap(),
            PropertyValue::String(" \t".into()),
        );
        gallery.descriptor_json = encode_preview(&invalid, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        assert!(
            gallery
                .status
                .contains(&format!("Sheet {name} must not be blank"))
        );
        let current = gallery.host.snapshot().unwrap();
        assert_eq!(current.stamp, before.stamp);
        assert_eq!(current.preview.document, before.preview.document);
    }
}

#[test]
fn drawer_specimen_round_trips_bounded_strings_and_rejects_blank_imports() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_drawer_specimen();

    let component_id = ComponentId::new("drawer-specimen").unwrap();
    let schema = &gallery.host.catalog().components[&component_id];
    assert_eq!(schema.properties.len(), 3);
    assert!(schema.events.is_empty());
    for (name, default, max_length) in [
        (
            "trigger_label",
            "Open details",
            PREVIEW_SHEET_TRIGGER_MAX_LENGTH,
        ),
        ("title", "Details", PREVIEW_SHEET_TITLE_MAX_LENGTH),
        (
            "body_text",
            "Preview sheet content",
            PREVIEW_SHEET_BODY_MAX_LENGTH,
        ),
    ] {
        let property = &schema.properties[&PropertyId::new(name).unwrap()];
        assert_eq!(property.kind, PropertyKind::String);
        assert_eq!(property.min_length, Some(1));
        assert_eq!(property.max_length, Some(max_length));
        assert_eq!(
            property.default,
            Some(PropertyValue::String(default.into()))
        );
    }

    let mounted = gallery.host.snapshot().unwrap().clone();
    assert_eq!(mounted.preview.document.properties.len(), 3);
    assert!(mounted.preview.document.actions.is_empty());
    let mut customized = mounted.preview.document.clone();
    customized.properties.insert(
        PropertyId::new("trigger_label").unwrap(),
        PropertyValue::String("Open project details".into()),
    );
    customized.properties.insert(
        PropertyId::new("title").unwrap(),
        PropertyValue::String("Project details".into()),
    );
    customized.properties.insert(
        PropertyId::new("body_text").unwrap(),
        PropertyValue::String("Build and test status".into()),
    );
    gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();
    gallery.export_descriptor();
    let exported = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
    for (name, value) in [
        ("trigger_label", "Open project details"),
        ("title", "Project details"),
        ("body_text", "Build and test status"),
    ] {
        assert_eq!(
            exported.properties[&PropertyId::new(name).unwrap()],
            PropertyValue::String(value.into())
        );
    }

    let before = gallery.host.snapshot().unwrap().clone();
    for name in ["trigger_label", "title", "body_text"] {
        let mut invalid = before.preview.document.clone();
        invalid.properties.insert(
            PropertyId::new(name).unwrap(),
            PropertyValue::String(" \t".into()),
        );
        gallery.descriptor_json = encode_preview(&invalid, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        assert!(
            gallery
                .status
                .contains(&format!("Drawer {name} must not be blank"))
        );
        let current = gallery.host.snapshot().unwrap();
        assert_eq!(current.stamp, before.stamp);
        assert_eq!(current.preview.document, before.preview.document);
    }
}

#[test]
fn sidebar_specimen_round_trips_labels_and_selection_and_rejects_invalid_import() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_sidebar_specimen();

    let component_id = ComponentId::new("sidebar-specimen").unwrap();
    let home_label_id = PropertyId::new("home_label").unwrap();
    let settings_label_id = PropertyId::new("settings_label").unwrap();
    let selected_id = PropertyId::new("selected_id").unwrap();
    let schema = &gallery.host.catalog().components[&component_id];
    assert_eq!(schema.properties.len(), 3);
    assert!(schema.events.is_empty());
    for (name, default) in [("home_label", "Home"), ("settings_label", "Settings")] {
        let property = &schema.properties[&PropertyId::new(name).unwrap()];
        assert_eq!(property.kind, PropertyKind::String);
        assert_eq!(property.min_length, Some(1));
        assert_eq!(property.max_length, Some(PREVIEW_SIDEBAR_LABEL_MAX_LENGTH));
        assert_eq!(
            property.default,
            Some(PropertyValue::String(default.into()))
        );
    }
    assert_eq!(schema.properties[&selected_id].kind, PropertyKind::String);
    assert_eq!(
        schema.properties[&selected_id].default,
        Some(PropertyValue::String("home".into()))
    );
    assert_eq!(
        schema.properties[&selected_id].allowed_values,
        vec!["home".to_owned(), "settings".to_owned()]
    );

    let mounted = gallery.host.snapshot().unwrap().clone();
    assert_eq!(mounted.preview.document.properties.len(), 3);
    assert!(mounted.preview.document.actions.is_empty());
    assert_eq!(
        mounted.preview.document.properties[&home_label_id],
        PropertyValue::String("Home".into())
    );
    assert_eq!(
        mounted.preview.document.properties[&settings_label_id],
        PropertyValue::String("Settings".into())
    );
    assert_eq!(
        mounted.preview.document.properties[&selected_id],
        PropertyValue::String("home".into())
    );
    let stamp_lease = gallery.stamp_lease.clone();
    let home_context = PropertyBindingContext {
        property: home_label_id.clone(),
        stamp: stamp_lease.clone(),
    };
    let settings_context = PropertyBindingContext {
        property: settings_label_id.clone(),
        stamp: stamp_lease.clone(),
    };
    let before_blank_edit = gallery.host.snapshot().unwrap().clone();
    assert!(!gallery.write_text(&home_context, " \t ".into()));
    assert!(!gallery.write_text(&settings_context, " \t ".into()));
    let after_blank_edit = gallery.host.snapshot().unwrap();
    assert_eq!(after_blank_edit.stamp, before_blank_edit.stamp);
    assert_eq!(
        after_blank_edit.preview.document,
        before_blank_edit.preview.document
    );
    assert!(gallery.write_text(&home_context, "Start".into()));
    assert!(gallery.write_text(&settings_context, "Preferences".into()));

    let selection_context = PropertyBindingContext {
        property: selected_id.clone(),
        stamp: stamp_lease.clone(),
    };
    let selection =
        CompiledPreviewGallery::string_selection_binding(Some("home".into()), selection_context);
    assert!(selection.write_to(&mut gallery, Some("settings".into())));
    assert_eq!(gallery.text, "settings");
    assert_eq!(
        gallery.host.snapshot().unwrap().preview.document.properties[&selected_id],
        PropertyValue::String("settings".into())
    );

    gallery.export_descriptor();
    let exported = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
    assert_eq!(
        exported.properties[&home_label_id],
        PropertyValue::String("Start".into())
    );
    assert_eq!(
        exported.properties[&settings_label_id],
        PropertyValue::String("Preferences".into())
    );
    assert_eq!(
        exported.properties[&selected_id],
        PropertyValue::String("settings".into())
    );
    gallery.import_descriptor();
    let imported = gallery.host.snapshot().unwrap().clone();
    assert_eq!(imported.stamp.preview_id.as_str(), "compiled-gallery");
    assert_eq!(imported.preview.document.properties.len(), 3);
    assert!(imported.preview.document.actions.is_empty());

    let before_invalid_import = imported.clone();
    let invalid_descriptor = gallery
        .descriptor_json
        .replacen("\"settings\"", "\"unknown\"", 1);
    assert_ne!(invalid_descriptor, gallery.descriptor_json);
    gallery.descriptor_json = invalid_descriptor;
    gallery.import_descriptor();
    assert!(gallery.status.starts_with("Import rejected:"));
    let after_invalid_import = gallery.host.snapshot().unwrap();
    assert_eq!(after_invalid_import.stamp, before_invalid_import.stamp);
    assert_eq!(
        after_invalid_import.preview.document,
        before_invalid_import.preview.document
    );

    gallery.export_descriptor();
    for name in ["home_label", "settings_label"] {
        let invalid_descriptor = gallery.descriptor_json.replacen(
            if name == "home_label" {
                "\"Start\""
            } else {
                "\"Preferences\""
            },
            "\" \"",
            1,
        );
        assert_ne!(invalid_descriptor, gallery.descriptor_json);
        gallery.descriptor_json = invalid_descriptor;
        gallery.import_descriptor();
        assert!(
            gallery
                .status
                .contains(&format!("Sidebar {name} must not be blank"))
        );
        let after_blank_label = gallery.host.snapshot().unwrap();
        assert_eq!(after_blank_label.stamp, before_invalid_import.stamp);
        assert_eq!(
            after_blank_label.preview.document,
            before_invalid_import.preview.document
        );
        gallery.export_descriptor();
    }
}

#[test]
fn keyed_list_specimen_round_trips_labels_and_selection_and_rejects_invalid_import() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_keyed_list_specimen();

    let component_id = ComponentId::new("keyed-list-specimen").unwrap();
    let first_label_id = PropertyId::new("first_label").unwrap();
    let second_label_id = PropertyId::new("second_label").unwrap();
    let selected_id = PropertyId::new("selected_id").unwrap();
    let schema = &gallery.host.catalog().components[&component_id];
    assert_eq!(schema.properties.len(), 3);
    assert!(schema.events.is_empty());
    for (name, default) in [
        ("first_label", "First item"),
        ("second_label", "Second item"),
    ] {
        let property = &schema.properties[&PropertyId::new(name).unwrap()];
        assert_eq!(property.kind, PropertyKind::String);
        assert_eq!(property.min_length, Some(1));
        assert_eq!(
            property.max_length,
            Some(PREVIEW_KEYED_LIST_LABEL_MAX_LENGTH)
        );
        assert_eq!(
            property.default,
            Some(PropertyValue::String(default.into()))
        );
    }
    assert_eq!(schema.properties[&selected_id].kind, PropertyKind::String);
    assert_eq!(
        schema.properties[&selected_id].default,
        Some(PropertyValue::String("first".into()))
    );
    assert_eq!(
        schema.properties[&selected_id].allowed_values,
        vec!["first".to_owned(), "second".to_owned()]
    );

    let mounted = gallery.host.snapshot().unwrap().clone();
    assert_eq!(mounted.preview.document.properties.len(), 3);
    assert!(mounted.preview.document.actions.is_empty());
    assert_eq!(
        mounted.preview.document.properties[&first_label_id],
        PropertyValue::String("First item".into())
    );
    assert_eq!(
        mounted.preview.document.properties[&second_label_id],
        PropertyValue::String("Second item".into())
    );
    assert_eq!(
        mounted.preview.document.properties[&selected_id],
        PropertyValue::String("first".into())
    );

    let stamp_lease = gallery.stamp_lease.clone();
    let first_context = PropertyBindingContext {
        property: first_label_id.clone(),
        stamp: stamp_lease.clone(),
    };
    let second_context = PropertyBindingContext {
        property: second_label_id.clone(),
        stamp: stamp_lease.clone(),
    };
    let before_blank_edit = gallery.host.snapshot().unwrap().clone();
    assert!(!gallery.write_text(&first_context, " \t ".into()));
    assert!(!gallery.write_text(&second_context, " \t ".into()));
    let after_blank_edit = gallery.host.snapshot().unwrap();
    assert_eq!(after_blank_edit.stamp, before_blank_edit.stamp);
    assert_eq!(
        after_blank_edit.preview.document,
        before_blank_edit.preview.document
    );
    assert!(gallery.write_text(&first_context, "Primary".into()));
    assert!(gallery.write_text(&second_context, "Secondary".into()));

    let selection = CompiledPreviewGallery::string_selection_binding(
        Some("first".into()),
        PropertyBindingContext {
            property: selected_id.clone(),
            stamp: stamp_lease,
        },
    );
    assert!(selection.write_to(&mut gallery, Some("second".into())));
    assert_eq!(gallery.text, "second");
    assert_eq!(
        gallery.host.snapshot().unwrap().preview.document.properties[&selected_id],
        PropertyValue::String("second".into())
    );

    gallery.export_descriptor();
    let exported = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
    assert_eq!(
        exported.properties[&first_label_id],
        PropertyValue::String("Primary".into())
    );
    assert_eq!(
        exported.properties[&second_label_id],
        PropertyValue::String("Secondary".into())
    );
    assert_eq!(
        exported.properties[&selected_id],
        PropertyValue::String("second".into())
    );
    gallery.import_descriptor();
    let imported = gallery.host.snapshot().unwrap().clone();
    assert_eq!(imported.stamp.preview_id.as_str(), "compiled-gallery");
    assert_eq!(imported.preview.document.properties.len(), 3);
    assert!(imported.preview.document.actions.is_empty());

    let before_invalid_import = imported.clone();
    let invalid_descriptor = gallery
        .descriptor_json
        .replacen("\"second\"", "\"unknown\"", 1);
    assert_ne!(invalid_descriptor, gallery.descriptor_json);
    gallery.descriptor_json = invalid_descriptor;
    gallery.import_descriptor();
    assert!(gallery.status.starts_with("Import rejected:"));
    let after_invalid_import = gallery.host.snapshot().unwrap();
    assert_eq!(after_invalid_import.stamp, before_invalid_import.stamp);
    assert_eq!(
        after_invalid_import.preview.document,
        before_invalid_import.preview.document
    );
}

#[test]
fn toast_specimen_round_trips_bounded_message_and_optional_title() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_toast_specimen();

    let component_id = ComponentId::new("toast-specimen").unwrap();
    let message_id = PropertyId::new("message").unwrap();
    let title_id = PropertyId::new("title").unwrap();
    let show_id = EventId::new("show").unwrap();
    let schema = &gallery.host.catalog().components[&component_id];
    assert_eq!(schema.properties.len(), 2);
    let message = &schema.properties[&message_id];
    assert_eq!(message.kind, PropertyKind::String);
    assert_eq!(message.min_length, Some(1));
    assert_eq!(message.max_length, Some(PREVIEW_TOAST_MESSAGE_MAX_LENGTH));
    assert_eq!(
        message.default,
        Some(PropertyValue::String("Changes saved".into()))
    );
    let title = &schema.properties[&title_id];
    assert_eq!(title.kind, PropertyKind::String);
    assert_eq!(title.min_length, Some(1));
    assert_eq!(title.max_length, Some(PREVIEW_TOAST_TITLE_MAX_LENGTH));
    assert_eq!(title.default, None);
    assert!(!title.required);
    assert_eq!(schema.events.len(), 1);
    assert!(schema.events[&show_id].payload.is_empty());

    let mounted = gallery.host.snapshot().unwrap().clone();
    assert_eq!(mounted.preview.document.properties.len(), 1);
    assert_eq!(mounted.preview.document.actions.len(), 1);
    assert_eq!(
        mounted.preview.document.actions.get(&show_id),
        Some(&ActionRef::new("toast_show").unwrap())
    );
    gallery.show_toast(&mounted.signal(show_id.clone()));
    assert_eq!(gallery.activations, 1);
    assert_eq!(gallery.status, "Allowlisted Toast show action");

    let mut customized = mounted.preview.document.clone();
    customized.properties.insert(
        message_id.clone(),
        PropertyValue::String("All changes are saved".into()),
    );
    customized.properties.insert(
        title_id.clone(),
        PropertyValue::String("Project updated".into()),
    );
    gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();
    gallery.export_descriptor();
    let exported = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
    assert_eq!(
        exported.properties[&message_id],
        PropertyValue::String("All changes are saved".into())
    );
    assert_eq!(
        exported.properties[&title_id],
        PropertyValue::String("Project updated".into())
    );

    let before = gallery.host.snapshot().unwrap().clone();
    for (name, value) in [("message", " \t"), ("title", " \t")] {
        let mut invalid = before.preview.document.clone();
        invalid.properties.insert(
            PropertyId::new(name).unwrap(),
            PropertyValue::String(value.into()),
        );
        gallery.descriptor_json = encode_preview(&invalid, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        assert!(
            gallery
                .status
                .contains(&format!("Toast {name} must not be blank"))
        );
        let current = gallery.host.snapshot().unwrap();
        assert_eq!(current.stamp, before.stamp);
        assert_eq!(current.preview.document, before.preview.document);
    }
}

#[test]
fn optional_toast_title_inspector_initial_value_does_not_use_shared_text() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.text = "unrelated shared text".into();
    gallery.mount_toast_specimen();
    let snapshot = gallery.host.snapshot().unwrap();
    let title = PropertyId::new("title").unwrap();
    assert!(!snapshot.preview.properties.contains_key(&title));

    let initial = preview_inspector_string_initial_value(Some(snapshot), title.as_str());
    assert_eq!(initial, "");
    let context = PropertyBindingContext {
        property: title,
        stamp: gallery.stamp_lease.clone(),
    };
    let binding = Binding::new_with(
        initial,
        context,
        CompiledPreviewGallery::read_text,
        CompiledPreviewGallery::write_text,
    );
    assert_eq!(binding.value(), "");
}

#[test]
fn context_menu_specimen_dispatches_fixed_action_and_round_trips_labels() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_context_menu_specimen();

    let component_id = ComponentId::new("context-menu-specimen").unwrap();
    let target_id = PropertyId::new("target_label").unwrap();
    let action_label_id = PropertyId::new("action_label").unwrap();
    let event_id = EventId::new("activate").unwrap();
    let schema = &gallery.host.catalog().components[&component_id];
    assert_eq!(schema.properties.len(), 2);
    for (id, default) in [
        (&target_id, "Preview target"),
        (&action_label_id, "Open preview"),
    ] {
        let property = &schema.properties[id];
        assert_eq!(property.kind, PropertyKind::String);
        assert_eq!(property.min_length, Some(1));
        assert_eq!(
            property.max_length,
            Some(PREVIEW_CONTEXT_MENU_LABEL_MAX_LENGTH)
        );
        assert_eq!(
            property.default,
            Some(PropertyValue::String(default.into()))
        );
    }
    assert_eq!(schema.events.len(), 1);
    assert!(schema.events[&event_id].payload.is_empty());

    let mounted = gallery.host.snapshot().unwrap().clone();
    assert_eq!(mounted.preview.document.properties.len(), 2);
    assert_eq!(
        mounted.preview.document.actions.get(&event_id),
        Some(&ActionRef::new("context_menu_open_preview").unwrap())
    );
    gallery.activate_context_menu_item(&mounted.signal(event_id.clone()));
    assert_eq!(gallery.activations, 1);
    assert!(gallery.status.contains("Allowlisted ContextMenu"));

    let mut customized = mounted.preview.document.clone();
    customized.properties.insert(
        target_id.clone(),
        PropertyValue::String("Sample card".into()),
    );
    customized.properties.insert(
        action_label_id.clone(),
        PropertyValue::String("Open card".into()),
    );
    gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();
    let imported = gallery.host.snapshot().unwrap();
    assert_eq!(
        imported.preview.properties[&target_id],
        PropertyValue::String("Sample card".into())
    );
    assert_eq!(
        imported.preview.properties[&action_label_id],
        PropertyValue::String("Open card".into())
    );

    let before = imported.clone();
    for name in ["target_label", "action_label"] {
        let mut invalid = before.preview.document.clone();
        invalid.properties.insert(
            PropertyId::new(name).unwrap(),
            PropertyValue::String(" \t".into()),
        );
        gallery.descriptor_json = encode_preview(&invalid, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        assert!(
            gallery
                .status
                .contains(&format!("ContextMenu {name} must not be blank"))
        );
        assert_eq!(gallery.host.snapshot().unwrap().stamp, before.stamp);
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.document,
            before.preview.document
        );
    }
}

#[test]
fn popover_specimen_accepts_controlled_open_feedback_and_preserves_invalid_import() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_popover_specimen();

    let component_id = ComponentId::new("popover-specimen").unwrap();
    let trigger_id = PropertyId::new("trigger_label").unwrap();
    let open_id = PropertyId::new("open").unwrap();
    let schema = &gallery.host.catalog().components[&component_id];
    assert_eq!(schema.properties[&trigger_id].min_length, Some(1));
    assert_eq!(
        schema.properties[&trigger_id].max_length,
        Some(PREVIEW_POPOVER_TRIGGER_MAX_LENGTH)
    );
    assert_eq!(schema.properties[&open_id].kind, PropertyKind::Boolean);
    assert_eq!(
        schema.properties[&open_id].default,
        Some(PropertyValue::Boolean(false))
    );
    assert!(schema.events.is_empty());

    let open_context = PropertyBindingContext {
        property: open_id.clone(),
        stamp: gallery.stamp_lease.clone(),
    };
    assert!(!gallery.enabled);
    assert!(gallery.write_enabled(&open_context, true));
    assert_eq!(
        gallery.host.snapshot().unwrap().preview.properties[&open_id],
        PropertyValue::Boolean(true)
    );

    gallery.export_descriptor();
    gallery.import_descriptor();
    assert!(gallery.enabled);
    assert_eq!(
        gallery.host.snapshot().unwrap().preview.properties[&open_id],
        PropertyValue::Boolean(true)
    );
    let round_trip = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
    assert_eq!(
        round_trip.properties[&trigger_id],
        PropertyValue::String("Preview details".into())
    );
    assert_eq!(
        round_trip.properties[&open_id],
        PropertyValue::Boolean(true)
    );

    let close_context = PropertyBindingContext {
        property: open_id.clone(),
        stamp: gallery.stamp_lease.clone(),
    };
    assert!(gallery.write_enabled(&close_context, false));
    assert!(!gallery.enabled);
    assert_eq!(
        gallery.host.snapshot().unwrap().preview.properties[&open_id],
        PropertyValue::Boolean(false)
    );

    let before = gallery.host.snapshot().unwrap().clone();
    let mut invalid = before.preview.document.clone();
    invalid
        .properties
        .insert(trigger_id, PropertyValue::String(" \t".into()));
    gallery.descriptor_json = encode_preview(&invalid, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();
    assert!(
        gallery
            .status
            .contains("Popover trigger label must not be blank")
    );
    assert_eq!(gallery.host.snapshot().unwrap().stamp, before.stamp);
    assert_eq!(
        gallery.host.snapshot().unwrap().preview.document,
        before.preview.document
    );
    assert!(!gallery.enabled);
}

#[test]
fn missing_current_text_property_does_not_reuse_previous_component_text() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.text = "Previous component text".into();
    gallery.mount_icon_specimen();

    let binding = gallery.text_binding();

    assert_eq!(binding.read_from(&gallery), Some(String::new()));
}
