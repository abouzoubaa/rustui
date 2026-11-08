use super::*;

#[test]
fn grid_specimen_round_trips_bounded_columns_and_spacing_without_events() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_grid_specimen();
    assert!(
        gallery.status.starts_with("Mounted grid-specimen"),
        "{}",
        gallery.status
    );

    let component_id = ComponentId::new("grid-specimen").unwrap();
    let columns_id = PropertyId::new("columns").unwrap();
    let spacing_id = PropertyId::new("spacing").unwrap();
    let schema = &gallery.host.catalog().components[&component_id];
    assert_eq!(schema.properties.len(), 2);
    assert!(schema.events.is_empty());
    assert_eq!(schema.properties[&columns_id].kind, PropertyKind::Integer);
    assert_eq!(
        schema.properties[&columns_id].minimum,
        Some(PREVIEW_GRID_COLUMNS_MIN)
    );
    assert_eq!(
        schema.properties[&columns_id].maximum,
        Some(PREVIEW_GRID_COLUMNS_MAX)
    );
    assert_eq!(schema.properties[&spacing_id].kind, PropertyKind::Number);
    assert_eq!(
        schema.properties[&spacing_id].minimum,
        Some(PREVIEW_GRID_SPACING_MIN)
    );
    assert_eq!(
        schema.properties[&spacing_id].maximum,
        Some(PREVIEW_GRID_SPACING_MAX)
    );

    let mounted = gallery.host.snapshot().unwrap().clone();
    assert!(mounted.preview.document.actions.is_empty());
    assert_eq!(
        mounted.preview.properties[&columns_id],
        PropertyValue::Integer(PREVIEW_GRID_COLUMNS_DEFAULT)
    );
    assert_eq!(
        mounted.preview.properties[&spacing_id],
        PropertyValue::Number(PREVIEW_GRID_SPACING_DEFAULT)
    );
    let mut customized = mounted.preview.document.clone();
    customized
        .properties
        .insert(columns_id.clone(), PropertyValue::Integer(3));
    customized
        .properties
        .insert(spacing_id.clone(), PropertyValue::Number(24.5));
    gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();
    let imported = gallery.host.snapshot().unwrap();
    assert_eq!(
        imported.preview.properties[&columns_id],
        PropertyValue::Integer(3)
    );
    assert_eq!(
        imported.preview.properties[&spacing_id],
        PropertyValue::Number(24.5)
    );
}

#[test]
fn lazy_grid_specimen_round_trips_bounded_columns_and_gaps_without_events() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_lazy_grid_specimen();
    assert!(
        gallery.status.starts_with("Mounted lazy-grid-specimen"),
        "{}",
        gallery.status
    );

    let component_id = ComponentId::new("lazy-grid-specimen").unwrap();
    let columns_id = PropertyId::new("columns").unwrap();
    let spacing_id = PropertyId::new("spacing").unwrap();
    let schema = &gallery.host.catalog().components[&component_id];
    assert_eq!(schema.properties.len(), 2);
    assert!(schema.events.is_empty());
    assert_eq!(schema.properties[&columns_id].kind, PropertyKind::Integer);
    assert_eq!(
        schema.properties[&columns_id].minimum,
        Some(PREVIEW_LAZY_GRID_COLUMNS_MIN)
    );
    assert_eq!(
        schema.properties[&columns_id].maximum,
        Some(PREVIEW_LAZY_GRID_COLUMNS_MAX)
    );
    assert_eq!(schema.properties[&spacing_id].kind, PropertyKind::Number);
    assert_eq!(
        schema.properties[&spacing_id].minimum,
        Some(PREVIEW_LAZY_GRID_SPACING_MIN)
    );
    assert_eq!(
        schema.properties[&spacing_id].maximum,
        Some(PREVIEW_LAZY_GRID_SPACING_MAX)
    );

    let mounted = gallery.host.snapshot().unwrap().clone();
    assert!(mounted.preview.document.actions.is_empty());
    assert_eq!(
        mounted.preview.properties[&columns_id],
        PropertyValue::Integer(PREVIEW_LAZY_GRID_COLUMNS_DEFAULT)
    );
    assert_eq!(
        mounted.preview.properties[&spacing_id],
        PropertyValue::Number(PREVIEW_LAZY_GRID_SPACING_DEFAULT)
    );
    let mut customized = mounted.preview.document.clone();
    customized
        .properties
        .insert(columns_id.clone(), PropertyValue::Integer(3));
    customized
        .properties
        .insert(spacing_id.clone(), PropertyValue::Number(24.5));
    gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();
    let imported = gallery.host.snapshot().unwrap();
    assert_eq!(
        imported.preview.properties[&columns_id],
        PropertyValue::Integer(3)
    );
    assert_eq!(
        imported.preview.properties[&spacing_id],
        PropertyValue::Number(24.5)
    );
}

#[test]
fn spacer_specimen_round_trips_bounded_axis_and_minimum_without_events() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_spacer_specimen();
    assert!(
        gallery.status.starts_with("Mounted spacer-specimen"),
        "{}",
        gallery.status
    );

    let component_id = ComponentId::new("spacer-specimen").unwrap();
    let axis_id = PropertyId::new("axis").unwrap();
    let min_length_id = PropertyId::new("min_length").unwrap();
    let schema = &gallery.host.catalog().components[&component_id];
    assert_eq!(schema.properties.len(), 2);
    assert!(schema.events.is_empty());
    assert_eq!(schema.properties[&axis_id].kind, PropertyKind::String);
    assert_eq!(
        schema.properties[&axis_id].allowed_values,
        PREVIEW_SPACER_AXES
            .iter()
            .map(|axis| (*axis).into())
            .collect::<Vec<String>>()
    );
    assert_eq!(schema.properties[&min_length_id].kind, PropertyKind::Number);
    assert_eq!(
        schema.properties[&min_length_id].minimum,
        Some(PREVIEW_SPACER_MIN_LENGTH_MIN)
    );
    assert_eq!(
        schema.properties[&min_length_id].maximum,
        Some(PREVIEW_SPACER_MIN_LENGTH_MAX)
    );

    let mounted = gallery.host.snapshot().unwrap().clone();
    assert!(mounted.preview.document.actions.is_empty());
    assert_eq!(
        mounted.preview.properties[&axis_id],
        PropertyValue::String("horizontal".into())
    );
    assert_eq!(
        mounted.preview.properties[&min_length_id],
        PropertyValue::Number(PREVIEW_SPACER_MIN_LENGTH_DEFAULT)
    );
    let mut customized = mounted.preview.document.clone();
    customized
        .properties
        .insert(axis_id.clone(), PropertyValue::String("vertical".into()));
    customized
        .properties
        .insert(min_length_id.clone(), PropertyValue::Number(32.5));
    gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();
    let imported = gallery.host.snapshot().unwrap();
    assert_eq!(
        imported.preview.properties[&axis_id],
        PropertyValue::String("vertical".into())
    );
    assert_eq!(
        imported.preview.properties[&min_length_id],
        PropertyValue::Number(32.5)
    );
}

#[test]
fn zstack_specimen_round_trips_constrained_alignment_without_events() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount("zstack-specimen");
    assert!(
        gallery.status.starts_with("Mounted zstack-specimen"),
        "{}",
        gallery.status
    );

    let component_id = ComponentId::new("zstack-specimen").unwrap();
    let alignment_id = PropertyId::new("alignment").unwrap();
    let schema = &gallery.host.catalog().components[&component_id];
    assert_eq!(schema.properties.len(), 1);
    assert!(schema.events.is_empty());
    assert_eq!(schema.properties[&alignment_id].kind, PropertyKind::String);
    assert_eq!(
        schema.properties[&alignment_id].allowed_values,
        PREVIEW_ZSTACK_ALIGNMENTS
            .iter()
            .map(|alignment| String::from(*alignment))
            .collect::<Vec<_>>()
    );

    let mounted = gallery.host.snapshot().unwrap().clone();
    assert!(mounted.preview.document.actions.is_empty());
    assert_eq!(
        mounted.preview.properties[&alignment_id],
        PropertyValue::String("center".into())
    );
    let mut customized = mounted.preview.document.clone();
    customized.properties.insert(
        alignment_id.clone(),
        PropertyValue::String("top-leading".into()),
    );
    gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();
    let imported = gallery.host.snapshot().unwrap();
    assert_eq!(
        imported.preview.properties[&alignment_id],
        PropertyValue::String("top-leading".into())
    );

    customized
        .properties
        .insert(alignment_id, PropertyValue::String("arbitrary".into()));
    assert!(encode_preview(&customized, gallery.host.catalog()).is_err());
}

#[test]
fn geometry_reader_specimen_round_trips_bounded_size() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_geometry_reader_specimen();
    assert!(
        gallery
            .status
            .starts_with("Mounted geometry-reader-specimen"),
        "{}",
        gallery.status
    );

    let component_id = ComponentId::new("geometry-reader-specimen").unwrap();
    let schema = &gallery.host.catalog().components[&component_id];
    let width_id = PropertyId::new("width").unwrap();
    let height_id = PropertyId::new("height").unwrap();
    assert_eq!(schema.properties.len(), 2);
    assert!(schema.events.is_empty());
    assert_eq!(schema.properties[&width_id].minimum, Some(80.0));
    assert_eq!(schema.properties[&width_id].maximum, Some(320.0));
    assert_eq!(schema.properties[&height_id].minimum, Some(64.0));
    assert_eq!(schema.properties[&height_id].maximum, Some(240.0));

    let mounted = gallery.host.snapshot().unwrap().clone();
    assert_eq!(
        mounted.preview.properties[&width_id],
        PropertyValue::Number(280.0)
    );
    assert_eq!(
        mounted.preview.properties[&height_id],
        PropertyValue::Number(120.0)
    );
    let mut customized = mounted.preview.document.clone();
    customized
        .properties
        .insert(width_id.clone(), PropertyValue::Number(312.5));
    customized
        .properties
        .insert(height_id.clone(), PropertyValue::Number(192.0));
    gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();

    let imported = gallery.host.snapshot().unwrap();
    assert_eq!(
        imported.preview.properties[&width_id],
        PropertyValue::Number(312.5)
    );
    assert_eq!(
        imported.preview.properties[&height_id],
        PropertyValue::Number(192.0)
    );
}
