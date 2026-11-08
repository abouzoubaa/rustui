use super::*;

#[test]
fn preview_mount_and_import_status_presentation_never_masks_later_errors() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_text();
    let mounted = gallery.current_status_presentation().unwrap();
    assert_eq!(mounted.key, "lab-preview-status-mounted");
    assert_eq!(mounted.subject, "text");
    gallery.export_descriptor();
    assert!(gallery.current_status_presentation().is_none());
    gallery.import_descriptor();
    let imported = gallery.current_status_presentation().unwrap();
    assert_eq!(imported.key, "lab-preview-status-imported");
    assert!(!imported.subject.is_empty());
    gallery.status = "Import rejected: invalid descriptor".into();
    assert!(gallery.current_status_presentation().is_none());
    let catalogs = crate::stories::locale_selector::catalogs();
    for language in ["en", "fr", "ar"] {
        let locale = Locale::parse(language).unwrap();
        let args = MessageArguments::new()
            .with("subject", "text")
            .with("generation", "7")
            .with("activations", "2")
            .with("renders", "3");
        for key in ["lab-preview-status-mounted", "lab-preview-status-imported"] {
            let text = catalogs.format(&locale, key, &args).unwrap();
            assert!(
                text.contains("text") && text.contains('7'),
                "{language}: {text}"
            );
        }
    }
}

#[test]
fn preview_status_summary_localizes_known_results_and_preserves_errors() {
    let catalogs = crate::stories::locale_selector::catalogs();
    for (status, translations) in [
        (
            "Exported normalized preview descriptor",
            ["Exported", "Descripteur", "تصدير"],
        ),
        (
            "Completed delayed parent rerender",
            ["Completed", "Actualisation", "تحديث"],
        ),
    ] {
        let key = preview_status_summary_key(status);
        for (language, expected) in ["en", "fr", "ar"].into_iter().zip(translations) {
            let locale = Locale::parse(language).unwrap();
            let args = MessageArguments::new()
                .with("activations", "2")
                .with("renders", "3");
            let text = catalogs.format(&locale, key, &args).unwrap();
            assert!(
                text.contains(expected) && text.contains('2') && text.contains('3'),
                "{language}: {text}"
            );
        }
    }
    assert_eq!(
        preview_status_summary_key("Export rejected: disk error"),
        "lab-compiled-preview-status-summary"
    );
}

#[test]
fn preview_owner_summary_formats_flags_and_amount_for_each_locale() {
    let catalogs = crate::stories::locale_selector::catalogs();
    for (language, yes, no, amount) in [
        ("en", "true", "false", "0.375"),
        ("fr", "oui", "non", "0,375"),
        ("ar", "نعم", "لا", "٠٫٣٧٥"),
    ] {
        let locale = Locale::parse(language).unwrap();
        let arguments = MessageArguments::new()
            .with("secure_present", "true")
            .with("enabled", "false")
            .with("amount", 0.375_f64)
            .with("stamp", "stamp");
        let message = catalogs
            .format(&locale, "lab-compiled-preview-owner-secure", &arguments)
            .unwrap();
        for expected in [yes, no, amount, "stamp"] {
            assert!(message.contains(expected), "{language}: {message}");
        }
    }
}

#[test]
fn every_implemented_public_component_has_a_preview_specimen() {
    let gallery = CompiledPreviewGallery::default();
    let catalog = gallery.host.catalog();
    let expected = [
        "alert-specimen",
        "animated-opacity-specimen",
        "spring-opacity-specimen",
        "avatar-specimen",
        "background-specimen",
        "badge-specimen",
        "border-specimen",
        "breadcrumb-specimen",
        "button-specimen",
        "card-specimen",
        "canvas-specimen",
        "graph-view-specimen",
        "node-editor-specimen",
        "checkbox",
        "code-editor-specimen",
        "collection-specimen",
        "container-specimen",
        "color-picker-specimen",
        "combo-box-specimen",
        "context-menu-specimen",
        "data-grid-specimen",
        "date-picker-specimen",
        "dialog-specimen",
        "disclosure-specimen",
        "divider-specimen",
        "empty-state-specimen",
        "error-state-specimen",
        "frame-specimen",
        "geometry-reader-specimen",
        "grid-specimen",
        "group-specimen",
        "hstack-specimen",
        "hover-region-specimen",
        "icon-specimen",
        "image-specimen",
        "keyed-list-specimen",
        "lazy-grid-specimen",
        "spacer-specimen",
        "label-specimen",
        "link-specimen",
        "list-specimen",
        "localized-text-specimen",
        "menu-specimen",
        "navigation-split-specimen",
        "navigation-stack-specimen",
        "picker",
        "popover-specimen",
        "progress",
        "property-inspector-specimen",
        "radio",
        "search-field-specimen",
        "scroll-view-specimen",
        "secure-field-specimen",
        "semantics-specimen",
        "shadow-specimen",
        "sheet-specimen",
        "drawer-specimen",
        "sidebar-specimen",
        "skeleton-specimen",
        "split-view-specimen",
        "slider",
        "spinner-specimen",
        "stepper",
        "tab-view-specimen",
        "table-specimen",
        "text-editor-specimen",
        "text-field-specimen",
        "token-field-specimen",
        "text-specimen",
        "tap-region-specimen",
        "toast-specimen",
        "toggle",
        "toolbar-specimen",
        "tooltip-specimen",
        "tree-specimen",
        "outline-specimen",
        "vstack-specimen",
        "zstack-specimen",
    ];

    for component_id in expected {
        assert!(
            catalog
                .components
                .contains_key(&ComponentId::new(component_id).unwrap()),
            "missing compiled preview specimen for {component_id}"
        );
    }
    assert_eq!(expected.len(), 78);
}

#[test]
fn token_field_specimen_round_trips_controlled_tokens_and_rejects_invalid_lists() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_token_field_specimen();

    let component_id = ComponentId::new("token-field-specimen").unwrap();
    let schema = &gallery.host.catalog().components[&component_id];
    assert_eq!(schema.properties.len(), 3);
    assert!(schema.properties.values().all(|property| property.editable));
    let tokens_id = PropertyId::new("tokens_json").unwrap();
    let placeholder_id = PropertyId::new("placeholder").unwrap();
    let draft_id = PropertyId::new("draft").unwrap();
    assert_eq!(
        property_string(gallery.host.snapshot().as_ref().unwrap(), "tokens_json")
            .and_then(|value| parse_preview_token_values(&value).ok()),
        Some(vec!["Atlas".to_owned(), "Studio".to_owned()])
    );

    let tokens = Binding::new_with(
        gallery.token_values.clone(),
        PropertyBindingContext {
            property: tokens_id.clone(),
            stamp: gallery.stamp_lease.clone(),
        },
        CompiledPreviewGallery::read_token_values,
        CompiledPreviewGallery::write_token_values,
    );
    assert!(tokens.write_to(
        &mut gallery,
        vec!["Atlas".into(), "Atlas".into(), "Research".into()]
    ));
    assert!(
        gallery
            .text_binding()
            .write_to(&mut gallery, "Design".into())
    );
    let placeholder = PropertyBindingContext {
        property: placeholder_id.clone(),
        stamp: gallery.stamp_lease.clone(),
    };
    assert!(gallery.write_text(&placeholder, "Add collaborators".into()));

    gallery.export_descriptor();
    let exported = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
    assert_eq!(
        parse_preview_token_values(match &exported.properties[&tokens_id] {
            PropertyValue::String(value) => value,
            _ => panic!("TokenField tokens_json must be a string"),
        })
        .unwrap(),
        ["Atlas", "Atlas", "Research"]
    );
    assert_eq!(
        exported.properties[&draft_id],
        PropertyValue::String("Design".into())
    );
    assert_eq!(
        exported.properties[&placeholder_id],
        PropertyValue::String("Add collaborators".into())
    );

    let generation = gallery.host.snapshot().unwrap().stamp.generation;
    gallery.import_descriptor();
    let imported = gallery.host.snapshot().unwrap().clone();
    assert_eq!(imported.stamp.generation, generation + 1);
    assert_eq!(gallery.token_values, ["Atlas", "Atlas", "Research"]);
    assert_eq!(gallery.text, "Design");
    assert_eq!(gallery.token_field_placeholder, "Add collaborators");

    let accepted = imported.preview.properties[&tokens_id].clone();
    let invalid_context = PropertyBindingContext {
        property: tokens_id.clone(),
        stamp: gallery.stamp_lease.clone(),
    };
    assert!(!gallery.write_text(&invalid_context, "not a JSON array".into()));
    assert_eq!(
        gallery.host.snapshot().unwrap().preview.properties[&tokens_id],
        accepted
    );

    let mut invalid_document = imported.preview.document.clone();
    invalid_document
        .properties
        .insert(tokens_id, PropertyValue::String("not a JSON array".into()));
    gallery.descriptor_json = encode_preview(&invalid_document, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();
    assert_eq!(
        gallery.host.snapshot().unwrap().stamp.generation,
        generation + 1
    );
    assert_eq!(gallery.token_values, ["Atlas", "Atlas", "Research"]);
    assert!(
        gallery
            .status
            .starts_with("Import rejected: TokenField tokens_json")
    );
}

#[test]
fn tap_region_specimen_round_trips_bounded_properties_and_signal_action() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount("tap-region-specimen");

    let component_id = ComponentId::new("tap-region-specimen").unwrap();
    let label_id = PropertyId::new("label").unwrap();
    let enabled_id = PropertyId::new("enabled").unwrap();
    let tap_id = EventId::new("tap").unwrap();
    let schema = &gallery.host.catalog().components[&component_id];
    assert_eq!(schema.properties.len(), 2);
    assert_eq!(schema.properties[&label_id].kind, PropertyKind::String);
    assert_eq!(schema.properties[&label_id].min_length, Some(1));
    assert_eq!(schema.properties[&label_id].max_length, Some(80));
    assert_eq!(schema.properties[&enabled_id].kind, PropertyKind::Boolean);
    assert_eq!(schema.events.len(), 1);
    assert!(schema.events[&tap_id].payload.is_empty());

    let mounted = gallery.host.snapshot().unwrap();
    assert_eq!(
        mounted.preview.document.actions[&tap_id].as_str(),
        "increment"
    );
    assert_eq!(
        mounted.preview.properties[&label_id],
        PropertyValue::String("Tap this text".into())
    );
    assert_eq!(
        mounted.preview.properties[&enabled_id],
        PropertyValue::Boolean(true)
    );

    let mut customized = mounted.preview.document.clone();
    customized.properties.insert(
        label_id.clone(),
        PropertyValue::String("Open details".into()),
    );
    customized
        .properties
        .insert(enabled_id.clone(), PropertyValue::Boolean(false));
    gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();
    let imported = gallery.host.snapshot().unwrap();
    assert_eq!(
        imported.preview.properties[&label_id],
        PropertyValue::String("Open details".into())
    );
    assert_eq!(
        imported.preview.properties[&enabled_id],
        PropertyValue::Boolean(false)
    );
    assert_eq!(
        imported.preview.document.actions[&tap_id].as_str(),
        "increment"
    );

    let mut invalid = imported.preview.document.clone();
    invalid
        .properties
        .insert(enabled_id, PropertyValue::String("true".into()));
    assert!(encode_preview(&invalid, gallery.host.catalog()).is_err());

    gallery.activate(&imported.signal(tap_id));
    assert_eq!(gallery.activations, 1);
}

#[test]
fn hover_region_specimen_round_trips_enter_and_exit_signals_without_closures() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount("hover-region-specimen");

    let component_id = ComponentId::new("hover-region-specimen").unwrap();
    let label_id = PropertyId::new("label").unwrap();
    let enter_id = EventId::new("enter").unwrap();
    let exit_id = EventId::new("exit").unwrap();
    let schema = &gallery.host.catalog().components[&component_id];
    assert_eq!(schema.properties.len(), 1);
    assert_eq!(schema.properties[&label_id].kind, PropertyKind::String);
    assert_eq!(schema.properties[&label_id].min_length, Some(1));
    assert_eq!(schema.properties[&label_id].max_length, Some(80));
    assert_eq!(schema.events.len(), 2);
    assert!(schema.events[&enter_id].payload.is_empty());
    assert!(schema.events[&exit_id].payload.is_empty());

    let mounted = gallery.host.snapshot().unwrap();
    assert_eq!(
        mounted.preview.document.actions[&enter_id].as_str(),
        "increment"
    );
    assert_eq!(
        mounted.preview.document.actions[&exit_id].as_str(),
        "increment"
    );
    assert_eq!(mounted.preview.document.actions.len(), 2);
    assert!(
        mounted
            .preview
            .document
            .actions
            .values()
            .all(|action| { action.as_str() == "increment" })
    );
    assert_eq!(
        mounted.preview.properties[&label_id],
        PropertyValue::String("Hover over this text".into())
    );

    let mut customized = mounted.preview.document.clone();
    customized.properties.insert(
        label_id.clone(),
        PropertyValue::String("Inspect this area".into()),
    );
    gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();
    let imported = gallery.host.snapshot().unwrap();
    assert_eq!(
        imported.preview.properties[&label_id],
        PropertyValue::String("Inspect this area".into())
    );
    assert_eq!(imported.preview.document.actions.len(), 2);
    let enter_signal = imported.signal(enter_id);
    let exit_signal = imported.signal(exit_id);
    gallery.record_hover_enter(&enter_signal);
    assert_eq!(gallery.activations, 1);
    assert_eq!(gallery.status, "Allowlisted HoverRegion enter action");
    gallery.record_hover_exit(&exit_signal);
    assert_eq!(gallery.activations, 2);
    assert_eq!(gallery.status, "Allowlisted HoverRegion exit action");
}

#[test]
fn date_picker_specimen_round_trips_optional_dates_and_rejects_stale_feedback() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_date_picker_specimen();
    let component_id = ComponentId::new("date-picker-specimen").unwrap();
    let date_id = PropertyId::new("date").unwrap();
    let accessible_name_id = PropertyId::new("accessibility_label").unwrap();
    let schema = &gallery.host.catalog().components[&component_id];
    assert!(schema.events.is_empty());
    assert_eq!(schema.properties[&date_id].max_length, Some(10));
    assert_eq!(
        schema.properties[&accessible_name_id].default,
        Some(PropertyValue::String("Start date".into()))
    );
    assert_eq!(
        property_string(&gallery.host.snapshot().unwrap(), "accessibility_label").as_deref(),
        Some("Start date")
    );
    assert_eq!(gallery.host.snapshot().and_then(property_date), None);
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
    let context = PropertyBindingContext {
        property: date_id.clone(),
        stamp: gallery.stamp_lease.clone(),
    };
    let selected = NaiveDate::from_ymd_opt(2026, 10, 20).unwrap();
    assert!(gallery.write_date_value(&context, Some(selected)));
    assert_eq!(gallery.read_date_value(&context), Some(selected));
    gallery.export_descriptor();
    let decoded = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
    assert_eq!(
        decoded.properties[&date_id],
        PropertyValue::String("2026-10-20".into())
    );
    assert_eq!(
        decoded.properties[&accessible_name_id],
        PropertyValue::String("Start date".into())
    );
    gallery.import_descriptor();
    assert_eq!(
        gallery.host.snapshot().and_then(property_date),
        Some(selected)
    );
    assert!(!gallery.write_date_value(&context, None));
    assert_eq!(
        gallery.host.snapshot().and_then(property_date),
        Some(selected)
    );

    let current = PropertyBindingContext {
        property: date_id.clone(),
        stamp: gallery.stamp_lease.clone(),
    };
    assert!(gallery.write_date_value(&current, None));
    gallery.export_descriptor();
    gallery.import_descriptor();
    assert_eq!(gallery.host.snapshot().and_then(property_date), None);
    let before = gallery.host.snapshot().unwrap().stamp.clone();
    for malformed in ["2026-02-30", "2026-2-03", "0000-01-01", "not-a-date"] {
        let mut document = gallery.host.snapshot().unwrap().preview.document.clone();
        document
            .properties
            .insert(date_id.clone(), PropertyValue::String(malformed.into()));
        gallery.descriptor_json = encode_preview(&document, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        assert!(gallery.status.starts_with("Import rejected:"));
        assert_eq!(gallery.host.snapshot().unwrap().stamp, before);
    }
    gallery.mount_text();
    assert!(!gallery.write_date_value(&current, Some(selected)));
}

#[test]
fn stale_date_binding_does_not_read_a_new_preview_generation() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_date_picker_specimen();
    let date_id = PropertyId::new("date").unwrap();
    let old_date = NaiveDate::from_ymd_opt(2026, 10, 20).unwrap();
    let current = PropertyBindingContext {
        property: date_id.clone(),
        stamp: gallery.stamp_lease.clone(),
    };
    assert!(gallery.write_date_value(&current, Some(old_date)));
    assert_eq!(gallery.read_date_value(&current), Some(old_date));

    let stale = PropertyBindingContext {
        property: date_id.clone(),
        stamp: Rc::new(RefCell::new(Some(
            gallery.host.snapshot().unwrap().stamp.clone(),
        ))),
    };
    let new_date = NaiveDate::from_ymd_opt(2026, 10, 21).unwrap();
    let mut document = gallery.host.snapshot().unwrap().preview.document.clone();
    document
        .properties
        .insert(date_id.clone(), PropertyValue::String(new_date.to_string()));
    gallery.descriptor_json = encode_preview(&document, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();

    let current = PropertyBindingContext {
        property: date_id,
        stamp: gallery.stamp_lease.clone(),
    };
    assert_eq!(gallery.read_date_value(&stale), None);
    assert_eq!(gallery.read_date_value(&current), Some(new_date));
}

#[test]
fn stale_enabled_binding_does_not_read_a_new_preview_generation() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_badge_specimen();
    let dot_id = PropertyId::new("dot").unwrap();
    let stale = PropertyBindingContext {
        property: dot_id.clone(),
        stamp: Rc::new(RefCell::new(Some(
            gallery.host.snapshot().unwrap().stamp.clone(),
        ))),
    };

    let mut document = gallery.host.snapshot().unwrap().preview.document.clone();
    document
        .properties
        .insert(dot_id.clone(), PropertyValue::Boolean(true));
    gallery.descriptor_json = encode_preview(&document, gallery.host.catalog()).unwrap();
    gallery.import_descriptor();

    let current = PropertyBindingContext {
        property: dot_id,
        stamp: gallery.stamp_lease.clone(),
    };
    assert!(!gallery.read_enabled(&stale));
    assert!(gallery.read_enabled(&current));
}

#[test]
fn stale_enabled_binding_does_not_read_a_new_preview_revision() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_badge_specimen();
    let current = PropertyBindingContext {
        property: PropertyId::new("dot").unwrap(),
        stamp: gallery.stamp_lease.clone(),
    };
    let stale = PropertyBindingContext {
        property: current.property.clone(),
        stamp: Rc::new(RefCell::new(gallery.stamp_lease.borrow().clone())),
    };

    assert!(gallery.write_enabled(&current, true));
    assert!(!gallery.read_enabled(&stale));
    assert!(gallery.read_enabled(&current));
}

#[test]
fn stale_amount_draft_binding_does_not_read_a_new_preview_generation() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_text();
    gallery.amount_draft = "generation one".into();
    let stale = PropertyBindingContext {
        property: PropertyId::new("amount").unwrap(),
        stamp: Rc::new(RefCell::new(Some(
            gallery.host.snapshot().unwrap().stamp.clone(),
        ))),
    };

    gallery.mount_text();
    gallery.amount_draft = "generation two".into();
    let current = PropertyBindingContext {
        property: PropertyId::new("amount").unwrap(),
        stamp: gallery.stamp_lease.clone(),
    };
    assert_eq!(gallery.read_amount_draft(&stale), "");
    assert_eq!(gallery.read_amount_draft(&current), "generation two");
}

#[test]
fn descriptor_import_rejection_is_announced_without_changing_mounted_preview() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_text();
    let before = gallery.host.snapshot().unwrap().clone();

    gallery.descriptor_json = "{".into();
    gallery.import_descriptor();

    assert!(gallery.status.starts_with("Import rejected:"));
    assert_eq!(
        preview_inspector_rejection_live_region_value(&gallery.status),
        Some(gallery.status.as_str())
    );
    assert_eq!(
        preview_inspector_rejection_live_region_value("Export rejected: invalid descriptor"),
        Some("Export rejected: invalid descriptor")
    );
    assert_eq!(
        preview_inspector_rejection_live_region_value("Wrong property type rejected: invalid"),
        Some("Wrong property type rejected: invalid")
    );
    assert_eq!(
        preview_inspector_rejection_live_region_value(
            "Color channels must be finite values from 0 to 1"
        ),
        Some("Color channels must be finite values from 0 to 1")
    );
    let after = gallery.host.snapshot().unwrap();
    assert_eq!(after.stamp, before.stamp);
    assert_eq!(after.preview.document, before.preview.document);
    assert_eq!(after.preview.properties, before.preview.properties);
}

#[test]
fn color_picker_inspector_rejection_is_announced_without_changing_preview() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_color_picker_specimen();
    let before = gallery.host.snapshot().unwrap().clone();
    let context = PropertyBindingContext {
        property: PropertyId::new("hue").unwrap(),
        stamp: gallery.stamp_lease.clone(),
    };

    assert!(!gallery.write_color_value(
        &context,
        Some(Hsla {
            h: f32::NAN,
            s: PREVIEW_COLOR_SATURATION as f32,
            l: PREVIEW_COLOR_LIGHTNESS as f32,
            a: PREVIEW_COLOR_ALPHA as f32,
        }),
    ));
    assert_eq!(
        gallery.status,
        "Color channels must be finite values from 0 to 1"
    );
    assert_eq!(
        preview_inspector_rejection_live_region_value(&gallery.status),
        Some(gallery.status.as_str())
    );
    let after = gallery.host.snapshot().unwrap();
    assert_eq!(after.stamp, before.stamp);
    assert_eq!(after.preview.properties, before.preview.properties);
}

#[test]
fn date_picker_inspector_rejects_noncanonical_date_writes() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_date_picker_specimen();
    let date_id = PropertyId::new("date").unwrap();
    let context = PropertyBindingContext {
        property: date_id.clone(),
        stamp: gallery.stamp_lease.clone(),
    };

    assert!(!gallery.write_text(&context, "2026-2-03".into()));
    assert_eq!(
        gallery.host.snapshot().unwrap().preview.properties[&date_id],
        PropertyValue::String(String::new())
    );
    assert_eq!(
        preview_inspector_rejection_live_region_value(&gallery.status),
        Some(DATE_PICKER_DATE_REJECTION)
    );

    assert!(gallery.write_text(&context, "2026-02-03".into()));
    assert_eq!(
        preview_inspector_rejection_live_region_value(&gallery.status),
        None
    );
    assert_eq!(
        gallery.host.snapshot().unwrap().preview.properties[&date_id],
        PropertyValue::String("2026-02-03".into())
    );

    assert!(!gallery.write_text(&context, "2026-02-30".into()));
    assert_eq!(
        gallery.host.snapshot().unwrap().preview.properties[&date_id],
        PropertyValue::String("2026-02-03".into())
    );
}

#[test]
fn stale_text_binding_does_not_change_preview_status_or_value() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_tree_specimen();
    let label_id = PropertyId::new("root_label").unwrap();
    let stale_context = PropertyBindingContext {
        property: label_id.clone(),
        stamp: Rc::new(RefCell::new(Some(
            gallery.host.snapshot().unwrap().stamp.clone(),
        ))),
    };

    gallery.mount_tree_specimen();
    let status = gallery.status.clone();
    let current_label = gallery.host.snapshot().unwrap().preview.properties[&label_id].clone();

    assert!(!gallery.write_text(&stale_context, "   ".into()));
    assert_eq!(gallery.status, status);
    assert_eq!(
        gallery.host.snapshot().unwrap().preview.properties[&label_id],
        current_label
    );
}

#[test]
fn stale_boolean_binding_does_not_change_preview_status_or_value() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_checkbox();
    let checked_id = PropertyId::new("checked").unwrap();
    let stale_context = PropertyBindingContext {
        property: checked_id.clone(),
        stamp: Rc::new(RefCell::new(Some(
            gallery.host.snapshot().unwrap().stamp.clone(),
        ))),
    };

    gallery.mount_checkbox();
    let status = gallery.status.clone();
    let current_value = gallery.host.snapshot().unwrap().preview.properties[&checked_id].clone();

    assert!(!gallery.write_enabled(&stale_context, true));
    assert_eq!(gallery.status, status);
    assert_eq!(
        gallery.host.snapshot().unwrap().preview.properties[&checked_id],
        current_value
    );
}

#[test]
fn preview_inspector_announces_rejected_text_and_number_writes() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_tree_specimen();
    let label_id = PropertyId::new("root_label").unwrap();
    let text_context = PropertyBindingContext {
        property: label_id.clone(),
        stamp: gallery.stamp_lease.clone(),
    };
    let previous_label = gallery.host.snapshot().unwrap().preview.properties[&label_id].clone();

    assert!(!gallery.write_text(&text_context, "   ".into()));
    assert_eq!(
        preview_inspector_rejection_live_region_value(&gallery.status),
        Some(gallery.status.as_str())
    );
    assert_eq!(
        gallery.host.snapshot().unwrap().preview.properties[&label_id],
        previous_label
    );

    gallery.mount_hstack_specimen();
    let amount_id = PropertyId::new("spacing").unwrap();
    let number_context = PropertyBindingContext {
        property: amount_id.clone(),
        stamp: gallery.stamp_lease.clone(),
    };
    let previous_amount = gallery.host.snapshot().unwrap().preview.properties[&amount_id].clone();

    assert!(gallery.write_number_draft(&number_context, "40".into()));
    assert_eq!(
        preview_inspector_rejection_live_region_value(&gallery.status),
        Some(gallery.status.as_str())
    );
    assert_eq!(
        gallery.host.snapshot().unwrap().preview.properties[&amount_id],
        previous_amount
    );

    assert!(gallery.write_number_draft(&number_context, "24.5".into()));
    assert_eq!(
        preview_inspector_rejection_live_region_value(&gallery.status),
        None
    );
    assert_eq!(
        gallery.host.snapshot().unwrap().preview.properties[&amount_id],
        PropertyValue::Number(24.5)
    );
    assert_eq!(
        preview_inspector_rejection_live_region_value("Exported normalized preview descriptor"),
        None
    );
}

#[test]
fn schema_amount_editor_updates_preview_owner() {
    let mut gallery = CompiledPreviewGallery::default();
    let amount = PropertyId::new("amount").unwrap();
    let context = PropertyBindingContext {
        property: amount.clone(),
        stamp: gallery.stamp_lease.clone(),
    };

    assert!(gallery.write_number_draft(&context, "0.625".into()));
    assert_eq!(gallery.amount, 0.625);
    assert_eq!(
        gallery.host.snapshot().unwrap().preview.properties[&amount],
        PropertyValue::Number(0.625)
    );
}

#[test]
fn stepper_inspector_rejects_out_of_range_i64_with_announcement() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_stepper();
    let before = gallery.host.snapshot().unwrap().clone();
    let context = PropertyBindingContext {
        property: PropertyId::new("value").unwrap(),
        stamp: gallery.stamp_lease.clone(),
    };

    assert!(!gallery.write_integer_property(&context, i64::from(i32::MAX) + 1));
    assert_eq!(gallery.status, "Stepper value must fit its control");
    assert_eq!(
        preview_inspector_rejection_live_region_value(&gallery.status),
        Some(gallery.status.as_str())
    );
    let after = gallery.host.snapshot().unwrap();
    assert_eq!(after.stamp, before.stamp);
    assert_eq!(after.preview.properties, before.preview.properties);
}

#[test]
fn dialog_specimen_uses_signal_only_actions_and_round_trips_trigger() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_dialog_specimen();

    let component_id = ComponentId::new("dialog-specimen").unwrap();
    let trigger_id = PropertyId::new("trigger_label").unwrap();
    let confirm_id = EventId::new("confirm").unwrap();
    let cancel_id = EventId::new("cancel").unwrap();
    let schema = &gallery.host.catalog().components[&component_id];
    assert_eq!(schema.properties.len(), 1);
    assert_eq!(schema.properties[&trigger_id].kind, PropertyKind::String);
    assert_eq!(schema.events.len(), 2);
    assert!(schema.events[&confirm_id].payload.is_empty());
    assert!(schema.events[&cancel_id].payload.is_empty());

    let mounted = gallery.host.snapshot().unwrap();
    assert_eq!(
        mounted.preview.properties[&trigger_id],
        PropertyValue::String("Open preview dialog".into())
    );
    for event in [confirm_id.clone(), cancel_id.clone()] {
        assert_eq!(
            mounted.preview.document.actions[&event],
            ActionRef::new("increment").unwrap()
        );
        assert!(gallery.host.dispatch(&mounted.signal(event)).is_ok());
    }

    let context = PropertyBindingContext {
        property: trigger_id.clone(),
        stamp: gallery.stamp_lease.clone(),
    };
    assert!(gallery.write_text(&context, "Review changes".into()));
    assert_eq!(gallery.text, "Review changes");
    gallery.export_descriptor();
    let exported = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
    assert_eq!(
        exported.properties[&trigger_id],
        PropertyValue::String("Review changes".into())
    );
    gallery.import_descriptor();
    assert_eq!(gallery.text, "Review changes");
    assert_eq!(
        gallery.host.snapshot().unwrap().preview.properties[&trigger_id],
        PropertyValue::String("Review changes".into())
    );
}

#[test]
fn slider_preview_uses_f32_feedback_and_round_trips() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_slider();
    let component_id = ComponentId::new("slider").unwrap();
    let value_id = PropertyId::new("value").unwrap();
    let schema = &gallery.host.catalog().components[&component_id];
    let value_schema = &schema.properties[&value_id];
    assert_eq!(schema.properties.len(), 1);
    assert_eq!(value_schema.kind, PropertyKind::Number);
    assert_eq!(value_schema.minimum, Some(0.0));
    assert_eq!(value_schema.maximum, Some(1.0));
    assert_eq!(
        value_schema.default,
        Some(PropertyValue::Number(PREVIEW_SLIDER_DEFAULT))
    );
    assert!(schema.events.is_empty());
    let stamp = gallery.host.snapshot().unwrap().stamp.clone();
    let context = PropertyBindingContext {
        property: value_id.clone(),
        stamp: gallery.stamp_lease.clone(),
    };
    let binding = Binding::new_with(
        PREVIEW_SLIDER_DEFAULT as f32,
        context,
        CompiledPreviewGallery::read_slider_value,
        CompiledPreviewGallery::write_slider_value,
    );
    assert!(binding.write_to(&mut gallery, 0.625));
    assert_eq!(gallery.amount, 0.625);
    assert_eq!(
        gallery.host.snapshot().unwrap().preview.properties[&value_id],
        PropertyValue::Number(0.625)
    );
    assert_eq!(
        gallery.host.snapshot().unwrap().stamp.revision,
        stamp.revision + 1
    );

    let stale_context = PropertyBindingContext {
        property: value_id.clone(),
        stamp: Rc::new(RefCell::new(Some(stamp))),
    };
    assert!(!gallery.write_slider_value(&stale_context, 0.75));
    assert_eq!(
        gallery.read_slider_value(&stale_context),
        PREVIEW_SLIDER_MIN
    );
    assert_eq!(gallery.amount, 0.625);

    gallery.export_descriptor();
    let descriptor = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
    assert_eq!(
        descriptor.properties[&value_id],
        PropertyValue::Number(0.625)
    );
    gallery.import_descriptor();
    assert_eq!(gallery.amount, 0.625);
    assert_eq!(
        gallery.host.snapshot().unwrap().preview.properties[&value_id],
        PropertyValue::Number(0.625)
    );
}

#[test]
fn slider_preview_imports_inexact_f32_value_without_mutating_descriptor() {
    let mut gallery = CompiledPreviewGallery::default();
    gallery.mount_slider();
    let value_id = PropertyId::new("value").unwrap();
    let mut document = gallery.host.snapshot().unwrap().preview.document.clone();
    document
        .properties
        .insert(value_id.clone(), PropertyValue::Number(0.1));
    gallery.descriptor_json = encode_preview(&document, gallery.host.catalog()).unwrap();

    gallery.import_descriptor();
    assert_eq!(gallery.amount, 0.1);
    assert_eq!(
        gallery.host.snapshot().unwrap().preview.properties[&value_id],
        PropertyValue::Number(0.1)
    );

    let context = PropertyBindingContext {
        property: value_id.clone(),
        stamp: gallery.stamp_lease.clone(),
    };
    assert_eq!(gallery.read_slider_value(&context), 0.1_f32);
    gallery.export_descriptor();
    let exported = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
    assert_eq!(exported.properties[&value_id], PropertyValue::Number(0.1));

    assert!(gallery.write_slider_value(&context, 0.125));
    assert_eq!(gallery.amount, f64::from(0.125_f32));
    assert_eq!(
        gallery.host.snapshot().unwrap().preview.properties[&value_id],
        PropertyValue::Number(f64::from(0.125_f32))
    );
}
