use super::*;

#[test]
fn animated_opacity_specimen_round_trips_bounded_endpoints_without_events() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_animated_opacity_specimen();
    assert!(
        gallery
            .status
            .starts_with("Mounted animated-opacity-specimen"),
        "{}",
        gallery.status
    );
    let component_id = ComponentId::new("animated-opacity-specimen").unwrap();
    let from_id = PropertyId::new("from").unwrap();
    let to_id = PropertyId::new("to").unwrap();
    let schema = &gallery.host.catalog().components[&component_id];
    assert_eq!(schema.properties.len(), 2);
    assert!(schema.events.is_empty());
    for id in [&from_id, &to_id] {
        assert_eq!(schema.properties[id].kind, PropertyKind::Number);
        assert_eq!(schema.properties[id].minimum, Some(0.0));
        assert_eq!(schema.properties[id].maximum, Some(1.0));
    }
    let mounted = gallery.host.snapshot().unwrap().clone();
    assert!(mounted.preview.document.actions.is_empty());
    assert_eq!(
        mounted.preview.properties[&from_id],
        PropertyValue::Number(0.0)
    );
    assert_eq!(
        mounted.preview.properties[&to_id],
        PropertyValue::Number(1.0)
    );
    let mut customized = mounted.preview.document.clone();
    customized
        .properties
        .insert(from_id.clone(), PropertyValue::Number(0.25));
    customized
        .properties
        .insert(to_id.clone(), PropertyValue::Number(0.75));
    gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();
    let imported = gallery.host.snapshot().unwrap();
    assert_eq!(
        imported.preview.properties[&from_id],
        PropertyValue::Number(0.25)
    );
    assert_eq!(
        imported.preview.properties[&to_id],
        PropertyValue::Number(0.75)
    );
    let mut invalid = imported.preview.document.clone();
    invalid
        .properties
        .insert(from_id, PropertyValue::Number(-0.01));
    assert!(encode_preview(&invalid, gallery.host.catalog()).is_err());
    invalid.properties.insert(
        PropertyId::new("from").unwrap(),
        PropertyValue::Number(0.25),
    );
    invalid
        .properties
        .insert(to_id, PropertyValue::Number(1.01));
    assert!(encode_preview(&invalid, gallery.host.catalog()).is_err());
}

#[test]
fn spring_opacity_specimen_round_trips_bounded_target_without_events() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_spring_opacity_specimen();
    assert!(
        gallery
            .status
            .starts_with("Mounted spring-opacity-specimen"),
        "{}",
        gallery.status
    );

    let component_id = ComponentId::new("spring-opacity-specimen").unwrap();
    let target_id = PropertyId::new("target").unwrap();
    let schema = &gallery.host.catalog().components[&component_id];
    assert_eq!(schema.properties.len(), 1);
    assert!(schema.events.is_empty());
    assert_eq!(schema.properties[&target_id].kind, PropertyKind::Number);
    assert_eq!(schema.properties[&target_id].minimum, Some(0.0));
    assert_eq!(schema.properties[&target_id].maximum, Some(1.0));

    let mounted = gallery.host.snapshot().unwrap().clone();
    assert!(mounted.preview.document.actions.is_empty());
    assert_eq!(
        mounted.preview.properties[&target_id],
        PropertyValue::Number(0.15)
    );
    let mut customized = mounted.preview.document.clone();
    customized
        .properties
        .insert(target_id.clone(), PropertyValue::Number(0.75));
    gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();
    let imported = gallery.host.snapshot().unwrap();
    assert_eq!(
        imported.preview.properties[&target_id],
        PropertyValue::Number(0.75)
    );

    for target in [-0.01, 1.01] {
        let mut invalid = imported.preview.document.clone();
        invalid
            .properties
            .insert(target_id.clone(), PropertyValue::Number(target));
        assert!(encode_preview(&invalid, gallery.host.catalog()).is_err());
    }
}

#[test]
fn shadow_specimen_round_trips_bounded_label_and_elevation_without_events() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_shadow_specimen();
    assert!(
        gallery.status.starts_with("Mounted shadow-specimen"),
        "{}",
        gallery.status
    );
    let component_id = ComponentId::new("shadow-specimen").unwrap();
    let label_id = PropertyId::new("label").unwrap();
    let elevation_id = PropertyId::new("elevation").unwrap();
    let schema = &gallery.host.catalog().components[&component_id];
    assert_eq!(schema.properties.len(), 2);
    assert!(schema.events.is_empty());
    assert_eq!(schema.properties[&label_id].kind, PropertyKind::String);
    assert_eq!(
        schema.properties[&label_id].max_length,
        Some(PREVIEW_TREE_LABEL_MAX_LENGTH)
    );
    assert_eq!(
        schema.properties[&elevation_id].allowed_values,
        ["small", "medium", "large"].map(str::to_owned)
    );
    let mounted = gallery.host.snapshot().unwrap().clone();
    assert!(mounted.preview.document.actions.is_empty());
    assert_eq!(
        mounted.preview.properties[&label_id],
        PropertyValue::String("Preview shadow".into())
    );
    assert_eq!(
        mounted.preview.properties[&elevation_id],
        PropertyValue::String("medium".into())
    );
    let mut customized = mounted.preview.document.clone();
    customized.properties.insert(
        label_id.clone(),
        PropertyValue::String("Raised panel".into()),
    );
    customized
        .properties
        .insert(elevation_id.clone(), PropertyValue::String("large".into()));
    gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();
    let imported = gallery.host.snapshot().unwrap();
    assert_eq!(
        imported.preview.properties[&label_id],
        PropertyValue::String("Raised panel".into())
    );
    assert_eq!(
        imported.preview.properties[&elevation_id],
        PropertyValue::String("large".into())
    );
    let mut invalid = imported.preview.document.clone();
    invalid
        .properties
        .insert(elevation_id, PropertyValue::String("custom".into()));
    assert!(encode_preview(&invalid, gallery.host.catalog()).is_err());
}

#[test]
fn background_specimen_round_trips_bounded_label_and_surface_without_events() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_background_specimen();
    assert!(
        gallery.status.starts_with("Mounted background-specimen"),
        "{}",
        gallery.status
    );
    let component_id = ComponentId::new("background-specimen").unwrap();
    let label_id = PropertyId::new("label").unwrap();
    let surface_id = PropertyId::new("surface").unwrap();
    let schema = &gallery.host.catalog().components[&component_id];
    assert_eq!(schema.properties.len(), 2);
    assert!(schema.events.is_empty());
    assert_eq!(schema.properties[&label_id].kind, PropertyKind::String);
    assert_eq!(
        schema.properties[&label_id].max_length,
        Some(PREVIEW_TREE_LABEL_MAX_LENGTH)
    );
    assert_eq!(
        schema.properties[&surface_id].allowed_values,
        PREVIEW_CONTAINER_SURFACES.map(str::to_owned)
    );
    let mounted = gallery.host.snapshot().unwrap().clone();
    assert!(mounted.preview.document.actions.is_empty());
    assert_eq!(
        mounted.preview.properties[&label_id],
        PropertyValue::String("Preview background".into())
    );
    assert_eq!(
        mounted.preview.properties[&surface_id],
        PropertyValue::String("panel".into())
    );
    let mut customized = mounted.preview.document.clone();
    customized.properties.insert(
        label_id.clone(),
        PropertyValue::String("Sidebar surface".into()),
    );
    customized
        .properties
        .insert(surface_id.clone(), PropertyValue::String("sidebar".into()));
    gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();
    let imported = gallery.host.snapshot().unwrap();
    assert_eq!(
        imported.preview.properties[&label_id],
        PropertyValue::String("Sidebar surface".into())
    );
    assert_eq!(
        imported.preview.properties[&surface_id],
        PropertyValue::String("sidebar".into())
    );
    let mut invalid = imported.preview.document.clone();
    invalid
        .properties
        .insert(surface_id, PropertyValue::String("custom".into()));
    assert!(encode_preview(&invalid, gallery.host.catalog()).is_err());
}

#[test]
fn border_specimen_round_trips_bounded_width_and_color_without_events() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_border_specimen();
    assert!(
        gallery.status.starts_with("Mounted border-specimen"),
        "{}",
        gallery.status
    );
    let component_id = ComponentId::new("border-specimen").unwrap();
    let width_id = PropertyId::new("width").unwrap();
    let color_id = PropertyId::new("color").unwrap();
    let schema = &gallery.host.catalog().components[&component_id];
    assert_eq!(schema.properties.len(), 2);
    assert!(schema.events.is_empty());
    assert_eq!(schema.properties[&width_id].kind, PropertyKind::Number);
    assert_eq!(schema.properties[&width_id].minimum, Some(0.0));
    assert_eq!(schema.properties[&width_id].maximum, Some(8.0));
    assert_eq!(
        schema.properties[&color_id].allowed_values,
        ["border", "accent", "focus-ring"].map(str::to_owned)
    );
    let mounted = gallery.host.snapshot().unwrap().clone();
    assert!(mounted.preview.document.actions.is_empty());
    assert_eq!(
        mounted.preview.properties[&width_id],
        PropertyValue::Number(1.0)
    );
    assert_eq!(
        mounted.preview.properties[&color_id],
        PropertyValue::String("border".into())
    );
    let mut customized = mounted.preview.document.clone();
    customized
        .properties
        .insert(width_id.clone(), PropertyValue::Number(2.5));
    customized
        .properties
        .insert(color_id.clone(), PropertyValue::String("accent".into()));
    gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();
    let imported = gallery.host.snapshot().unwrap();
    assert_eq!(
        imported.preview.properties[&width_id],
        PropertyValue::Number(2.5)
    );
    assert_eq!(
        imported.preview.properties[&color_id],
        PropertyValue::String("accent".into())
    );
    let mut invalid = imported.preview.document.clone();
    invalid
        .properties
        .insert(width_id.clone(), PropertyValue::Number(-0.5));
    assert!(encode_preview(&invalid, gallery.host.catalog()).is_err());
    invalid
        .properties
        .insert(width_id, PropertyValue::Number(2.5));
    invalid
        .properties
        .insert(color_id, PropertyValue::String("custom".into()));
    assert!(encode_preview(&invalid, gallery.host.catalog()).is_err());
}

#[test]
fn semantics_specimen_round_trips_allowlisted_static_role_without_events() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_semantics_specimen();
    assert!(
        gallery.status.starts_with("Mounted semantics-specimen"),
        "{}",
        gallery.status
    );
    let component_id = ComponentId::new("semantics-specimen").unwrap();
    let role_id = PropertyId::new("role").unwrap();
    let schema = &gallery.host.catalog().components[&component_id];
    assert_eq!(schema.properties.len(), 1);
    assert!(schema.events.is_empty());
    assert_eq!(schema.properties[&role_id].kind, PropertyKind::String);
    assert_eq!(
        schema.properties[&role_id].allowed_values,
        ["group", "region", "section", "navigation"].map(str::to_owned)
    );
    let mounted = gallery.host.snapshot().unwrap().clone();
    assert!(mounted.preview.document.actions.is_empty());
    assert_eq!(
        mounted.preview.properties[&role_id],
        PropertyValue::String("group".into())
    );
    let mut customized = mounted.preview.document.clone();
    customized
        .properties
        .insert(role_id.clone(), PropertyValue::String("navigation".into()));
    gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();
    let imported = gallery.host.snapshot().unwrap();
    assert_eq!(
        imported.preview.properties[&role_id],
        PropertyValue::String("navigation".into())
    );
    let mut invalid = imported.preview.document.clone();
    invalid
        .properties
        .insert(role_id, PropertyValue::String("button".into()));
    assert!(encode_preview(&invalid, gallery.host.catalog()).is_err());
}

#[test]
fn card_specimen_round_trips_bounded_label_and_border_without_events() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_card_specimen();
    assert!(
        gallery.status.starts_with("Mounted card-specimen"),
        "{}",
        gallery.status
    );
    let component_id = ComponentId::new("card-specimen").unwrap();
    let label_id = PropertyId::new("label").unwrap();
    let border_id = PropertyId::new("border").unwrap();
    let schema = &gallery.host.catalog().components[&component_id];
    assert_eq!(schema.properties.len(), 2);
    assert!(schema.events.is_empty());
    assert_eq!(schema.properties[&label_id].kind, PropertyKind::String);
    assert_eq!(schema.properties[&border_id].kind, PropertyKind::Boolean);

    let mounted = gallery.host.snapshot().unwrap().clone();
    assert!(mounted.preview.document.actions.is_empty());
    let mut customized = mounted.preview.document.clone();
    customized.properties.insert(
        label_id.clone(),
        PropertyValue::String("Settings card".into()),
    );
    customized
        .properties
        .insert(border_id.clone(), PropertyValue::Boolean(true));
    gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();
    let imported = gallery.host.snapshot().unwrap();
    assert_eq!(
        imported.preview.properties[&label_id],
        PropertyValue::String("Settings card".into())
    );
    assert_eq!(
        imported.preview.properties[&border_id],
        PropertyValue::Boolean(true)
    );
}

#[test]
fn container_specimen_round_trips_label_and_allowlisted_surface() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_container_specimen();
    assert!(
        gallery.status.starts_with("Mounted container-specimen"),
        "{}",
        gallery.status
    );
    let component_id = ComponentId::new("container-specimen").unwrap();
    let label_id = PropertyId::new("label").unwrap();
    let surface_id = PropertyId::new("surface").unwrap();
    let padding_id = PropertyId::new("padding").unwrap();
    let schema = &gallery.host.catalog().components[&component_id];
    assert!(schema.events.is_empty());
    assert_eq!(schema.properties.len(), 3);
    assert_eq!(schema.properties[&padding_id].kind, PropertyKind::Number);
    assert_eq!(
        schema.properties[&padding_id].minimum,
        Some(PREVIEW_CONTAINER_PADDING_MIN)
    );
    assert_eq!(
        schema.properties[&padding_id].maximum,
        Some(PREVIEW_CONTAINER_PADDING_MAX)
    );
    assert_eq!(
        schema.properties[&padding_id].default,
        Some(PropertyValue::Number(PREVIEW_CONTAINER_PADDING_DEFAULT))
    );
    assert_eq!(
        schema.properties[&surface_id].allowed_values,
        PREVIEW_CONTAINER_SURFACES
            .iter()
            .map(|surface| (*surface).to_string())
            .collect::<Vec<String>>()
    );

    let mounted = gallery.host.snapshot().unwrap().clone();
    let mut customized = mounted.preview.document.clone();
    customized.properties.insert(
        label_id.clone(),
        PropertyValue::String("Navigation surface".into()),
    );
    customized
        .properties
        .insert(surface_id.clone(), PropertyValue::String("sidebar".into()));
    customized
        .properties
        .insert(padding_id.clone(), PropertyValue::Number(24.0));
    gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();
    let imported = gallery.host.snapshot().unwrap();
    assert_eq!(
        imported.preview.properties[&label_id],
        PropertyValue::String("Navigation surface".into())
    );
    assert_eq!(
        imported.preview.properties[&surface_id],
        PropertyValue::String("sidebar".into())
    );
    assert_eq!(
        imported.preview.properties[&padding_id],
        PropertyValue::Number(24.0)
    );
}

#[test]
fn split_view_specimen_round_trips_orientation_labels_and_first_size() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_split_view_specimen();
    assert!(
        gallery.status.starts_with("Mounted split-view-specimen"),
        "{}",
        gallery.status
    );
    let component_id = ComponentId::new("split-view-specimen").unwrap();
    let orientation_id = PropertyId::new("orientation").unwrap();
    let first_size_id = PropertyId::new("first_size").unwrap();
    let first_label_id = PropertyId::new("first_label").unwrap();
    let second_label_id = PropertyId::new("second_label").unwrap();
    let schema = &gallery.host.catalog().components[&component_id];
    assert_eq!(schema.properties.len(), 4);
    assert!(schema.events.is_empty());
    assert_eq!(
        schema.properties[&orientation_id].allowed_values,
        PREVIEW_SPLIT_ORIENTATIONS
            .iter()
            .map(|orientation| (*orientation).to_string())
            .collect::<Vec<String>>()
    );
    assert_eq!(schema.properties[&first_size_id].kind, PropertyKind::Number);
    assert_eq!(
        schema.properties[&first_size_id].default,
        Some(PropertyValue::Number(180.0))
    );
    assert_eq!(schema.properties[&first_size_id].minimum, Some(120.0));
    assert_eq!(schema.properties[&first_size_id].maximum, Some(360.0));

    let mounted = gallery.host.snapshot().unwrap().clone();
    assert!(mounted.preview.document.actions.is_empty());
    let mut customized = mounted.preview.document.clone();
    customized.properties.insert(
        orientation_id.clone(),
        PropertyValue::String("vertical".into()),
    );
    customized
        .properties
        .insert(first_size_id.clone(), PropertyValue::Number(250.0));
    customized.properties.insert(
        first_label_id.clone(),
        PropertyValue::String("Projects".into()),
    );
    customized.properties.insert(
        second_label_id.clone(),
        PropertyValue::String("Overview".into()),
    );
    gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();
    let imported = gallery.host.snapshot().unwrap();
    assert_eq!(
        imported.preview.properties[&orientation_id],
        PropertyValue::String("vertical".into())
    );
    assert_eq!(
        imported.preview.properties[&first_label_id],
        PropertyValue::String("Projects".into())
    );
    assert_eq!(
        imported.preview.properties[&second_label_id],
        PropertyValue::String("Overview".into())
    );
    assert_eq!(
        imported.preview.properties[&first_size_id],
        PropertyValue::Number(250.0)
    );
    gallery.export_descriptor();
    let exported = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
    assert_eq!(
        exported.properties[&first_size_id],
        PropertyValue::Number(250.0)
    );
}

#[test]
fn label_specimen_round_trips_text_secondary_and_vibrant_descriptor() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_label_specimen();
    assert!(
        gallery.status.starts_with("Mounted label-specimen"),
        "{}",
        gallery.status
    );

    let component_id = ComponentId::new("label-specimen").unwrap();
    let text_id = PropertyId::new("text").unwrap();
    let secondary_id = PropertyId::new("secondary").unwrap();
    let vibrant_id = PropertyId::new("vibrant").unwrap();
    let schema = &gallery.host.catalog().components[&component_id];
    assert_eq!(schema.properties.len(), 3);
    assert_eq!(schema.properties[&text_id].kind, PropertyKind::String);
    assert_eq!(schema.properties[&secondary_id].kind, PropertyKind::String);
    assert_eq!(schema.properties[&vibrant_id].kind, PropertyKind::Boolean);
    assert!(schema.properties.values().all(|property| property.editable));
    assert!(schema.events.is_empty());

    let mounted = gallery.host.snapshot().unwrap().clone();
    assert!(mounted.preview.document.actions.is_empty());
    let mut customized = mounted.preview.document.clone();
    customized
        .properties
        .insert(text_id.clone(), PropertyValue::String("Profile".into()));
    customized
        .properties
        .insert(secondary_id.clone(), PropertyValue::String("⌥P".into()));
    customized
        .properties
        .insert(vibrant_id.clone(), PropertyValue::Boolean(true));
    gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();
    assert!(gallery.status.contains("Imported descriptor"));
    let imported = gallery.host.snapshot().unwrap();
    assert_eq!(
        imported.preview.properties[&text_id],
        PropertyValue::String("Profile".into())
    );
    assert_eq!(
        imported.preview.properties[&secondary_id],
        PropertyValue::String("⌥P".into())
    );
    assert_eq!(
        imported.preview.properties[&vibrant_id],
        PropertyValue::Boolean(true)
    );

    gallery.export_descriptor();
    let exported = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
    assert_eq!(
        exported.properties[&text_id],
        PropertyValue::String("Profile".into())
    );
    assert_eq!(
        exported.properties[&secondary_id],
        PropertyValue::String("⌥P".into())
    );
    assert_eq!(
        exported.properties[&vibrant_id],
        PropertyValue::Boolean(true)
    );
}

#[test]
fn divider_specimen_round_trips_vertical_orientation_descriptor() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_divider_specimen();
    assert!(
        gallery.status.starts_with("Mounted divider-specimen"),
        "{}",
        gallery.status
    );

    let component_id = ComponentId::new("divider-specimen").unwrap();
    let vertical_id = PropertyId::new("vertical").unwrap();
    let schema = &gallery.host.catalog().components[&component_id];
    assert_eq!(schema.properties.len(), 1);
    assert_eq!(schema.properties[&vertical_id].kind, PropertyKind::Boolean);
    assert_eq!(
        schema.properties[&vertical_id].default,
        Some(PropertyValue::Boolean(false))
    );
    assert!(schema.properties[&vertical_id].editable);
    assert!(schema.events.is_empty());

    let mounted = gallery.host.snapshot().unwrap().clone();
    assert!(mounted.preview.document.actions.is_empty());
    assert_eq!(
        mounted.preview.properties[&vertical_id],
        PropertyValue::Boolean(false)
    );
    let mut customized = mounted.preview.document.clone();
    customized
        .properties
        .insert(vertical_id.clone(), PropertyValue::Boolean(true));
    gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();
    assert!(gallery.status.contains("Imported descriptor"));
    assert_eq!(
        gallery.host.snapshot().unwrap().preview.properties[&vertical_id],
        PropertyValue::Boolean(true)
    );

    gallery.export_descriptor();
    let exported = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
    assert_eq!(
        exported.properties[&vertical_id],
        PropertyValue::Boolean(true)
    );
}

#[test]
fn skeleton_specimen_round_trips_secondary_descriptor() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_skeleton_specimen();
    assert!(
        gallery.status.starts_with("Mounted skeleton-specimen"),
        "{}",
        gallery.status
    );

    let component_id = ComponentId::new("skeleton-specimen").unwrap();
    let secondary_id = PropertyId::new("secondary").unwrap();
    let schema = &gallery.host.catalog().components[&component_id];
    assert_eq!(schema.properties.len(), 1);
    assert_eq!(schema.properties[&secondary_id].kind, PropertyKind::Boolean);
    assert_eq!(
        schema.properties[&secondary_id].default,
        Some(PropertyValue::Boolean(false))
    );
    assert!(schema.properties[&secondary_id].editable);
    assert!(schema.events.is_empty());

    let mounted = gallery.host.snapshot().unwrap().clone();
    assert!(mounted.preview.document.actions.is_empty());
    assert_eq!(
        mounted.preview.properties[&secondary_id],
        PropertyValue::Boolean(false)
    );
    let mut customized = mounted.preview.document.clone();
    customized
        .properties
        .insert(secondary_id.clone(), PropertyValue::Boolean(true));
    gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();
    assert!(gallery.status.contains("Imported descriptor"));
    assert_eq!(
        gallery.host.snapshot().unwrap().preview.properties[&secondary_id],
        PropertyValue::Boolean(true)
    );

    gallery.export_descriptor();
    let exported = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
    assert_eq!(
        exported.properties[&secondary_id],
        PropertyValue::Boolean(true)
    );
}

#[test]
fn scroll_view_specimen_round_trips_bounded_axis_label_and_scrollbars() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_scroll_view_specimen();
    assert!(
        gallery.status.starts_with("Mounted scroll-view-specimen"),
        "{}",
        gallery.status
    );

    let component_id = ComponentId::new("scroll-view-specimen").unwrap();
    let label_id = PropertyId::new("label").unwrap();
    let axis_id = PropertyId::new("axis").unwrap();
    let scrollbars_id = PropertyId::new("scrollbars").unwrap();
    let schema = &gallery.host.catalog().components[&component_id];
    assert_eq!(schema.properties.len(), 3);
    assert!(schema.events.is_empty());
    assert_eq!(
        schema.properties[&axis_id].allowed_values,
        PREVIEW_SCROLL_AXES
    );
    assert_eq!(
        schema.properties[&scrollbars_id].kind,
        PropertyKind::Boolean
    );

    let mounted = gallery.host.snapshot().unwrap().clone();
    assert!(mounted.preview.document.actions.is_empty());
    let mut customized = mounted.preview.document.clone();
    customized.properties.insert(
        label_id.clone(),
        PropertyValue::String("Scrollable files".into()),
    );
    customized
        .properties
        .insert(axis_id.clone(), PropertyValue::String("both".into()));
    customized
        .properties
        .insert(scrollbars_id.clone(), PropertyValue::Boolean(false));
    gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();
    let imported = gallery.host.snapshot().unwrap();
    assert_eq!(
        imported.preview.properties[&label_id],
        PropertyValue::String("Scrollable files".into())
    );
    assert_eq!(
        imported.preview.properties[&axis_id],
        PropertyValue::String("both".into())
    );
    assert_eq!(
        imported.preview.properties[&scrollbars_id],
        PropertyValue::Boolean(false)
    );
}

#[test]
fn property_inspector_specimen_round_trips_bounded_labels_and_disabled_state() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_property_inspector_specimen();
    assert!(
        gallery
            .status
            .starts_with("Mounted property-inspector-specimen"),
        "{}",
        gallery.status
    );

    let component_id = ComponentId::new("property-inspector-specimen").unwrap();
    let schema = &gallery.host.catalog().components[&component_id];
    let text_label_id = PropertyId::new("text_label").unwrap();
    let toggle_label_id = PropertyId::new("toggle_label").unwrap();
    let integer_label_id = PropertyId::new("integer_label").unwrap();
    let disabled_id = PropertyId::new("disabled").unwrap();
    assert_eq!(schema.properties.len(), 4);
    assert!(schema.events.is_empty());
    assert!(
        gallery
            .host
            .snapshot()
            .unwrap()
            .preview
            .document
            .actions
            .is_empty()
    );
    assert_eq!(
        schema.properties[&text_label_id].max_length,
        Some(PREVIEW_TREE_LABEL_MAX_LENGTH)
    );

    let mounted = gallery.host.snapshot().unwrap().clone();
    let mut customized = mounted.preview.document.clone();
    customized.properties.insert(
        text_label_id.clone(),
        PropertyValue::String("Display name".into()),
    );
    customized.properties.insert(
        toggle_label_id.clone(),
        PropertyValue::String("Visible".into()),
    );
    customized.properties.insert(
        integer_label_id.clone(),
        PropertyValue::String("Items".into()),
    );
    customized
        .properties
        .insert(disabled_id.clone(), PropertyValue::Boolean(true));
    gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();
    let imported = gallery.host.snapshot().unwrap();
    assert_eq!(
        imported.preview.properties[&text_label_id],
        PropertyValue::String("Display name".into())
    );
    assert_eq!(
        imported.preview.properties[&toggle_label_id],
        PropertyValue::String("Visible".into())
    );
    assert_eq!(
        imported.preview.properties[&integer_label_id],
        PropertyValue::String("Items".into())
    );
    assert_eq!(
        imported.preview.properties[&disabled_id],
        PropertyValue::Boolean(true)
    );
}

#[test]
fn number_inspector_drafts_follow_property_values_and_imports() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_avatar_specimen();
    let diameter_id = PropertyId::new("diameter").unwrap();
    let avatar_snapshot = gallery.host.snapshot().unwrap().clone();
    let diameter_context = PropertyBindingContext {
        property: diameter_id.clone(),
        stamp: gallery.stamp_lease.clone(),
    };
    let PropertyValue::Number(default_diameter) = avatar_snapshot.preview.properties[&diameter_id]
    else {
        panic!("Avatar diameter should be a number");
    };
    assert_eq!(
        gallery.read_number_draft(&diameter_context),
        default_diameter.to_string()
    );
    assert!(gallery.write_number_draft(&diameter_context, "56".into()));
    assert_eq!(gallery.amount, 0.375);
    assert_eq!(
        gallery.host.snapshot().unwrap().preview.properties[&diameter_id],
        PropertyValue::Number(56.0)
    );

    gallery.mount_hstack_specimen();
    assert_eq!(gallery.read_number_draft(&diameter_context), "");
    assert!(!gallery.write_number_draft(&diameter_context, "60".into()));

    let spacing_id = PropertyId::new("spacing").unwrap();
    let spacing_context = PropertyBindingContext {
        property: spacing_id.clone(),
        stamp: gallery.stamp_lease.clone(),
    };
    let hstack_snapshot = gallery.host.snapshot().unwrap().clone();
    let PropertyValue::Number(default_spacing) = hstack_snapshot.preview.properties[&spacing_id]
    else {
        panic!("HStack spacing should be a number");
    };
    assert_eq!(
        gallery.read_number_draft(&spacing_context),
        default_spacing.to_string()
    );
    assert!(gallery.write_number_draft(&spacing_context, "-".into()));
    assert_eq!(gallery.read_number_draft(&spacing_context), "-");
    assert_eq!(
        gallery.host.snapshot().unwrap().preview.properties[&spacing_id],
        PropertyValue::Number(default_spacing)
    );
    assert!(gallery.write_number_draft(&spacing_context, "24.5".into()));
    assert_eq!(gallery.amount, 0.375);
    assert_eq!(
        gallery.host.snapshot().unwrap().preview.properties[&spacing_id],
        PropertyValue::Number(24.5)
    );

    let mut imported = gallery.host.snapshot().unwrap().preview.document.clone();
    imported
        .properties
        .insert(spacing_id.clone(), PropertyValue::Number(30.25));
    gallery.descriptor_json = encode_preview(&imported, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();
    let imported_context = PropertyBindingContext {
        property: spacing_id.clone(),
        stamp: gallery.stamp_lease.clone(),
    };
    assert_eq!(gallery.read_number_draft(&imported_context), "30.25");
    assert_eq!(
        gallery.host.snapshot().unwrap().preview.properties[&spacing_id],
        PropertyValue::Number(30.25)
    );
}

#[test]
fn frame_specimen_round_trips_bounded_width_without_events() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_frame_specimen();
    assert!(
        gallery.status.starts_with("Mounted frame-specimen"),
        "{}",
        gallery.status
    );
    let component_id = ComponentId::new("frame-specimen").unwrap();
    let width_id = PropertyId::new("width").unwrap();
    let schema = &gallery.host.catalog().components[&component_id];
    assert_eq!(schema.properties.len(), 1);
    assert!(schema.events.is_empty());
    let width = &schema.properties[&width_id];
    assert_eq!(width.kind, PropertyKind::Number);
    assert_eq!(width.minimum, Some(PREVIEW_FRAME_WIDTH_MIN));
    assert_eq!(width.maximum, Some(PREVIEW_FRAME_WIDTH_MAX));
    let mounted = gallery.host.snapshot().unwrap().clone();
    assert!(mounted.preview.document.actions.is_empty());
    assert_eq!(
        mounted.preview.properties[&width_id],
        PropertyValue::Number(PREVIEW_FRAME_WIDTH_DEFAULT)
    );

    let mut customized = mounted.preview.document.clone();
    customized
        .properties
        .insert(width_id.clone(), PropertyValue::Number(192.5));
    gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();
    gallery.export_descriptor();
    let exported = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
    assert_eq!(exported.properties[&width_id], PropertyValue::Number(192.5));
    let accepted = gallery.host.snapshot().unwrap().clone();
    let valid_descriptor = gallery.descriptor_json.clone();
    for width in [PREVIEW_FRAME_WIDTH_MIN - 1.0, PREVIEW_FRAME_WIDTH_MAX + 1.0] {
        let mut invalid = accepted.preview.document.clone();
        invalid
            .properties
            .insert(width_id.clone(), PropertyValue::Number(width));
        assert!(encode_preview(&invalid, gallery.host.catalog()).is_err());
        gallery.descriptor_json = valid_descriptor.replacen("192.5", &width.to_string(), 1);
        gallery.import_descriptor();
        let current = gallery.host.snapshot().unwrap();
        assert_eq!(current.stamp, accepted.stamp);
        assert_eq!(current.preview.document, accepted.preview.document);
    }
}

#[test]
fn hstack_specimen_round_trips_bounded_spacing_without_events() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_hstack_specimen();
    assert!(
        gallery.status.starts_with("Mounted hstack-specimen"),
        "{}",
        gallery.status
    );

    let component_id = ComponentId::new("hstack-specimen").unwrap();
    let spacing_id = PropertyId::new("spacing").unwrap();
    let schema = &gallery.host.catalog().components[&component_id];
    assert_eq!(schema.properties.len(), 1);
    assert!(schema.events.is_empty());
    assert_eq!(schema.properties[&spacing_id].kind, PropertyKind::Number);
    assert_eq!(
        schema.properties[&spacing_id].minimum,
        Some(PREVIEW_HSTACK_SPACING_MIN)
    );
    assert_eq!(
        schema.properties[&spacing_id].maximum,
        Some(PREVIEW_HSTACK_SPACING_MAX)
    );

    let mounted = gallery.host.snapshot().unwrap().clone();
    assert!(mounted.preview.document.actions.is_empty());
    assert_eq!(
        mounted.preview.properties[&spacing_id],
        PropertyValue::Number(PREVIEW_HSTACK_SPACING_DEFAULT)
    );
    let mut customized = mounted.preview.document.clone();
    customized
        .properties
        .insert(spacing_id.clone(), PropertyValue::Number(24.5));
    gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();
    let imported = gallery.host.snapshot().unwrap();
    assert_eq!(
        imported.preview.properties[&spacing_id],
        PropertyValue::Number(24.5)
    );
}

#[test]
fn vstack_specimen_round_trips_bounded_spacing_without_events() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_vstack_specimen();
    assert!(
        gallery.status.starts_with("Mounted vstack-specimen"),
        "{}",
        gallery.status
    );

    let component_id = ComponentId::new("vstack-specimen").unwrap();
    let spacing_id = PropertyId::new("spacing").unwrap();
    let schema = &gallery.host.catalog().components[&component_id];
    assert_eq!(schema.properties.len(), 1);
    assert!(schema.events.is_empty());
    assert_eq!(schema.properties[&spacing_id].kind, PropertyKind::Number);
    assert_eq!(
        schema.properties[&spacing_id].minimum,
        Some(PREVIEW_VSTACK_SPACING_MIN)
    );
    assert_eq!(
        schema.properties[&spacing_id].maximum,
        Some(PREVIEW_VSTACK_SPACING_MAX)
    );

    let mounted = gallery.host.snapshot().unwrap().clone();
    assert!(mounted.preview.document.actions.is_empty());
    assert_eq!(
        mounted.preview.properties[&spacing_id],
        PropertyValue::Number(PREVIEW_VSTACK_SPACING_DEFAULT)
    );

    let mut customized = mounted.preview.document.clone();
    customized
        .properties
        .insert(spacing_id.clone(), PropertyValue::Number(24.5));
    gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();
    let imported = gallery.host.snapshot().unwrap();
    assert_eq!(
        imported.preview.properties[&spacing_id],
        PropertyValue::Number(24.5)
    );
}
