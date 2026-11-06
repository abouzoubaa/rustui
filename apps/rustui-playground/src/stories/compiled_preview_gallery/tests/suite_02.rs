    #[test]
    fn progress_preview_preserves_f64_descriptor_and_bounds_feedback() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_progress();
        let component_id = ComponentId::new("progress").unwrap();
        let label_id = PropertyId::new("label").unwrap();
        let value_id = PropertyId::new("value").unwrap();
        let indeterminate_id = PropertyId::new("indeterminate").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 3);
        assert_eq!(schema.properties[&label_id].kind, PropertyKind::String);
        assert_eq!(schema.properties[&value_id].kind, PropertyKind::Number);
        assert_eq!(schema.properties[&value_id].minimum, Some(0.0));
        assert_eq!(schema.properties[&value_id].maximum, Some(100.0));
        assert_eq!(
            schema.properties[&value_id].default,
            Some(PropertyValue::Number(PREVIEW_PROGRESS_DEFAULT))
        );
        assert_eq!(
            schema.properties[&indeterminate_id].kind,
            PropertyKind::Boolean
        );
        assert_eq!(
            schema.properties[&indeterminate_id].default,
            Some(PropertyValue::Boolean(false))
        );
        assert!(schema.events.is_empty());

        let inexact = 37.123_456_789_f64;
        let mut document = gallery.host.snapshot().unwrap().preview.document.clone();
        document
            .properties
            .insert(label_id.clone(), PropertyValue::String("Deployment".into()));
        document
            .properties
            .insert(value_id.clone(), PropertyValue::Number(inexact));
        document
            .properties
            .insert(indeterminate_id.clone(), PropertyValue::Boolean(true));
        gallery.descriptor_json = encode_preview(&document, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();

        assert_eq!(gallery.text, "Deployment");
        assert_eq!(gallery.amount, inexact);
        assert_eq!(progress_control_value(gallery.amount), inexact as f32);
        assert_ne!(f64::from(progress_control_value(gallery.amount)), inexact);
        gallery.export_descriptor();
        let exported = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
        assert_eq!(
            exported.properties[&value_id],
            PropertyValue::Number(inexact)
        );
        assert_eq!(
            exported.properties[&indeterminate_id],
            PropertyValue::Boolean(true)
        );

        let label_context = PropertyBindingContext {
            property: label_id.clone(),
            stamp: gallery.stamp_lease.clone(),
        };
        assert!(gallery.write_text(&label_context, "Release upload".into()));
        let value_context = PropertyBindingContext {
            property: value_id.clone(),
            stamp: gallery.stamp_lease.clone(),
        };
        assert!(gallery.write_amount_draft(&value_context, "64.5".into()));
        assert_eq!(gallery.amount, 64.5);
        assert!(gallery.write_amount_draft(&value_context, "100.5".into()));
        assert_eq!(gallery.amount, 64.5);
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&value_id],
            PropertyValue::Number(64.5)
        );

        gallery.export_descriptor();
        let exported = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
        assert_eq!(
            exported.properties[&label_id],
            PropertyValue::String("Release upload".into())
        );
        assert_eq!(exported.properties[&value_id], PropertyValue::Number(64.5));
        gallery.import_descriptor();
        assert_eq!(gallery.text, "Release upload");
        assert_eq!(gallery.amount, 64.5);
    }

    #[test]
    fn toggle_preview_declares_label_and_checked_and_accepts_checked_feedback() {
        let mut gallery = CompiledPreviewGallery::default();
        let toggle_id = ComponentId::new("toggle").unwrap();
        let schema = &gallery.host.catalog().components[&toggle_id];
        assert_eq!(schema.properties.len(), 2);
        assert_eq!(
            schema.properties[&PropertyId::new("label").unwrap()].kind,
            PropertyKind::String
        );
        assert_eq!(
            schema.properties[&PropertyId::new("checked").unwrap()].kind,
            PropertyKind::Boolean
        );

        let stamp = gallery
            .host
            .mount(PreviewDocument::new(
                PreviewId::new("toggle-test").unwrap(),
                toggle_id,
                1,
            ))
            .unwrap();
        let next = gallery
            .host
            .binding_feedback(
                &stamp,
                PropertyId::new("checked").unwrap(),
                PropertyValue::Boolean(true),
            )
            .unwrap();

        assert_eq!(next.revision, stamp.revision + 1);
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties
                [&PropertyId::new("checked").unwrap()],
            PropertyValue::Boolean(true),
        );
    }

    #[test]
    fn checkbox_preview_declares_label_and_checked_and_accepts_checked_feedback() {
        let mut gallery = CompiledPreviewGallery::default();
        let checkbox_id = ComponentId::new("checkbox").unwrap();
        let schema = &gallery.host.catalog().components[&checkbox_id];
        assert_eq!(schema.properties.len(), 2);
        assert_eq!(
            schema.properties[&PropertyId::new("label").unwrap()].kind,
            PropertyKind::String
        );
        assert_eq!(
            schema.properties[&PropertyId::new("checked").unwrap()].kind,
            PropertyKind::Boolean
        );

        let stamp = gallery
            .host
            .mount(PreviewDocument::new(
                PreviewId::new("checkbox-test").unwrap(),
                checkbox_id,
                1,
            ))
            .unwrap();
        let next = gallery
            .host
            .binding_feedback(
                &stamp,
                PropertyId::new("checked").unwrap(),
                PropertyValue::Boolean(true),
            )
            .unwrap();

        assert_eq!(next.revision, stamp.revision + 1);
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties
                [&PropertyId::new("checked").unwrap()],
            PropertyValue::Boolean(true),
        );
    }

    #[test]
    fn text_field_specimen_uses_stamp_checked_value_and_round_trips() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_text_field_specimen();
        let component_id = ComponentId::new("text-field-specimen").unwrap();
        let placeholder_id = PropertyId::new("placeholder").unwrap();
        let value_id = PropertyId::new("value").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 2);
        assert_eq!(
            schema.properties[&placeholder_id].kind,
            PropertyKind::String
        );
        assert_eq!(
            schema.properties[&placeholder_id].default,
            Some(PropertyValue::String("Preview text input".into()))
        );
        assert_eq!(schema.properties[&value_id].kind, PropertyKind::String);
        assert_eq!(
            schema.properties[&value_id].default,
            Some(PropertyValue::String("Preview text input".into()))
        );
        assert!(schema.events.is_empty());

        let stamp = gallery.host.snapshot().unwrap().stamp.clone();
        let context = PropertyBindingContext {
            property: value_id.clone(),
            stamp: gallery.stamp_lease.clone(),
        };
        assert!(gallery.write_text(&context, "Typed value".into()));
        assert_eq!(gallery.text, "Typed value");
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&value_id],
            PropertyValue::String("Typed value".into())
        );
        assert_eq!(
            gallery.host.snapshot().unwrap().stamp.revision,
            stamp.revision + 1
        );

        let stale_context = PropertyBindingContext {
            property: value_id.clone(),
            stamp: Rc::new(RefCell::new(Some(stamp))),
        };
        assert!(!gallery.write_text(&stale_context, "Stale value".into()));
        assert_eq!(gallery.text, "Typed value");

        let placeholder_context = PropertyBindingContext {
            property: placeholder_id.clone(),
            stamp: gallery.stamp_lease.clone(),
        };
        assert_eq!(
            gallery.read_text(&placeholder_context),
            "Preview text input"
        );
        assert!(gallery.write_text(&placeholder_context, "Search by name".into()));
        assert_eq!(gallery.text_field_placeholder, "Search by name");
        assert_eq!(gallery.text, "Typed value");
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&placeholder_id],
            PropertyValue::String("Search by name".into())
        );

        gallery.export_descriptor();
        let exported = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
        assert_eq!(
            exported.properties[&placeholder_id],
            PropertyValue::String("Search by name".into())
        );
        assert_eq!(
            exported.properties[&value_id],
            PropertyValue::String("Typed value".into())
        );
        gallery.import_descriptor();
        assert_eq!(gallery.text_field_placeholder, "Search by name");
        assert_eq!(gallery.text, "Typed value");
        let imported_placeholder_context = PropertyBindingContext {
            property: placeholder_id.clone(),
            stamp: gallery.stamp_lease.clone(),
        };
        assert_eq!(
            gallery.read_text(&imported_placeholder_context),
            "Search by name"
        );
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&placeholder_id],
            PropertyValue::String("Search by name".into())
        );
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&value_id],
            PropertyValue::String("Typed value".into())
        );
    }

    #[test]
    fn search_field_specimen_uses_stamp_checked_query_and_round_trips() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_search_field_specimen();
        let component_id = ComponentId::new("search-field-specimen").unwrap();
        let placeholder_id = PropertyId::new("placeholder").unwrap();
        let query_id = PropertyId::new("query").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 2);
        assert_eq!(
            schema.properties[&placeholder_id].kind,
            PropertyKind::String
        );
        assert_eq!(
            schema.properties[&placeholder_id].default,
            Some(PropertyValue::String("Search RustUI".into()))
        );
        assert_eq!(schema.properties[&query_id].kind, PropertyKind::String);
        assert_eq!(
            schema.properties[&query_id].default,
            Some(PropertyValue::String("Search RustUI".into()))
        );
        assert!(schema.events.is_empty());

        let stamp = gallery.host.snapshot().unwrap().stamp.clone();
        let binding = gallery.text_binding();
        assert_eq!(binding.read_from(&gallery), Some("Search RustUI".into()));
        assert!(binding.write_to(&mut gallery, "Search controls".into()));
        assert_eq!(gallery.text, "Search controls");
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&query_id],
            PropertyValue::String("Search controls".into())
        );
        assert_eq!(
            gallery.host.snapshot().unwrap().stamp.revision,
            stamp.revision + 1
        );

        let stale_context = PropertyBindingContext {
            property: query_id.clone(),
            stamp: Rc::new(RefCell::new(Some(stamp))),
        };
        assert!(!gallery.write_text(&stale_context, "Stale query".into()));
        assert_eq!(gallery.text, "Search controls");

        let placeholder_context = PropertyBindingContext {
            property: placeholder_id.clone(),
            stamp: gallery.stamp_lease.clone(),
        };
        assert_eq!(gallery.read_text(&placeholder_context), "Search RustUI");
        assert!(gallery.write_text(&placeholder_context, "Search by name".into()));
        assert_eq!(gallery.search_field_placeholder, "Search by name");
        assert_eq!(gallery.text, "Search controls");

        gallery.export_descriptor();
        let exported = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
        assert_eq!(
            exported.properties[&query_id],
            PropertyValue::String("Search controls".into())
        );
        assert_eq!(
            exported.properties[&placeholder_id],
            PropertyValue::String("Search by name".into())
        );
        gallery.import_descriptor();
        assert_eq!(gallery.text, "Search controls");
        assert_eq!(gallery.search_field_placeholder, "Search by name");
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&placeholder_id],
            PropertyValue::String("Search by name".into())
        );
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&query_id],
            PropertyValue::String("Search controls".into())
        );
    }

    #[test]
    fn text_editor_specimen_uses_stamp_checked_multiline_value_and_round_trips() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_text_editor_specimen();
        let component_id = ComponentId::new("text-editor-specimen").unwrap();
        let placeholder_id = PropertyId::new("placeholder").unwrap();
        let value_id = PropertyId::new("value").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 2);
        assert_eq!(
            schema.properties[&placeholder_id].kind,
            PropertyKind::String
        );
        assert_eq!(
            schema.properties[&placeholder_id].default,
            Some(PropertyValue::String("Write a note".into()))
        );
        assert_eq!(schema.properties[&value_id].kind, PropertyKind::String);
        assert_eq!(
            schema.properties[&value_id].default,
            Some(PropertyValue::String("First line\nSecond line".into()))
        );
        assert!(schema.events.is_empty());

        let stamp = gallery.host.snapshot().unwrap().stamp.clone();
        let value_context = PropertyBindingContext {
            property: value_id.clone(),
            stamp: gallery.stamp_lease.clone(),
        };
        let multiline = "Draft title\nSecond paragraph";
        assert!(gallery.write_text(&value_context, multiline.into()));
        assert_eq!(gallery.text, multiline);
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&value_id],
            PropertyValue::String(multiline.into())
        );
        assert_eq!(
            gallery.host.snapshot().unwrap().stamp.revision,
            stamp.revision + 1
        );

        let stale_context = PropertyBindingContext {
            property: value_id.clone(),
            stamp: Rc::new(RefCell::new(Some(stamp))),
        };
        assert!(!gallery.write_text(&stale_context, "Stale note".into()));
        assert_eq!(gallery.text, multiline);

        let placeholder_context = PropertyBindingContext {
            property: placeholder_id.clone(),
            stamp: gallery.stamp_lease.clone(),
        };
        assert_eq!(gallery.read_text(&placeholder_context), "Write a note");
        assert!(gallery.write_text(&placeholder_context, "Meeting notes".into()));
        assert_eq!(gallery.text_editor_placeholder, "Meeting notes");
        assert_eq!(gallery.text, multiline);

        gallery.export_descriptor();
        let exported = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
        assert_eq!(
            exported.properties[&placeholder_id],
            PropertyValue::String("Meeting notes".into())
        );
        assert_eq!(
            exported.properties[&value_id],
            PropertyValue::String(multiline.into())
        );

        gallery.import_descriptor();
        assert_eq!(gallery.text_editor_placeholder, "Meeting notes");
        assert_eq!(gallery.text, multiline);
        let imported = gallery.host.snapshot().unwrap();
        assert_eq!(
            imported.preview.properties[&placeholder_id],
            PropertyValue::String("Meeting notes".into())
        );
        assert_eq!(
            imported.preview.properties[&value_id],
            PropertyValue::String(multiline.into())
        );
    }

    #[test]
    fn localized_text_specimen_keeps_mount_stamp_while_catalog_resolution_changes() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_localized_text_specimen();

        let mut catalogs = Catalogs::new(Locale::parse("en").unwrap());
        for (locale, source) in [
            ("en", include_str!("../../../../../../locales/en/rustui.ftl")),
            ("fr", include_str!("../../../../../../locales/fr/rustui.ftl")),
            ("ar", include_str!("../../../../../../locales/ar/rustui.ftl")),
        ] {
            catalogs
                .insert(Catalog::parse(Locale::parse(locale).unwrap(), source).unwrap())
                .unwrap();
        }
        let initial = gallery.host.snapshot().unwrap().clone();
        let mounted_stamp = initial.stamp.clone();
        let source_id = localized_text_message_id(&initial);
        for (locale, expected_close, expected_confirm) in [
            ("en", "Close", "Confirm"),
            ("fr", "Fermer", "Confirmer"),
            ("ar", "إغلاق", "تأكيد"),
        ] {
            let locale = Locale::parse(locale).unwrap();
            assert_eq!(catalogs.resolve(&locale, &source_id), expected_close);
            assert_eq!(
                catalogs.resolve(&locale, "rustui-action-confirm"),
                expected_confirm
            );
            let current = gallery.host.snapshot().unwrap();
            assert_eq!(current.stamp, mounted_stamp);
            assert_eq!(localized_text_message_id(current), source_id);
        }

        let component_id = ComponentId::new("localized-text-specimen").unwrap();
        let message_id = PropertyId::new("message_id").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.display_name, "LocalizedText");
        assert_eq!(schema.properties.len(), 1);
        assert_eq!(schema.properties[&message_id].kind, PropertyKind::String);
        assert_eq!(
            schema.properties[&message_id].default,
            Some(PropertyValue::String(PREVIEW_LOCALIZED_TEXT_DEFAULT.into()))
        );
        assert_eq!(
            schema.properties[&message_id].allowed_values,
            PREVIEW_LOCALIZED_TEXT_KEYS
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        );
        assert!(schema.events.is_empty());

        assert_eq!(
            initial.preview.properties.get(&message_id),
            Some(&PropertyValue::String("rustui-action-close".into()))
        );
        assert_eq!(localized_text_message_id(&initial), "rustui-action-close");
        gallery
            .host
            .binding_feedback(
                &initial.stamp,
                message_id.clone(),
                PropertyValue::String("rustui-action-confirm".into()),
            )
            .unwrap();
        let updated = gallery.host.snapshot().unwrap();
        assert_eq!(localized_text_message_id(updated), "rustui-action-confirm");

        gallery.export_descriptor();
        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap();
        assert_eq!(
            imported.preview.document.component_id,
            ComponentId::new("localized-text-specimen").unwrap()
        );
        assert_eq!(localized_text_message_id(imported), "rustui-action-confirm");
        assert!(imported.preview.document.actions.is_empty());
    }

    #[test]
    fn code_editor_specimen_round_trips_bounded_plain_text_without_actions_or_events() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_code_editor_specimen();
        let component_id = ComponentId::new("code-editor-specimen").unwrap();
        let value_id = PropertyId::new("value").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 1);
        assert_eq!(schema.properties[&value_id].kind, PropertyKind::String);
        assert_eq!(
            schema.properties[&value_id].default,
            Some(PropertyValue::String("Plain text preview".into()))
        );
        assert_eq!(schema.properties[&value_id].max_length, Some(2_000));
        assert!(schema.events.is_empty());

        let initial_snapshot = gallery.host.snapshot().unwrap();
        assert_eq!(initial_snapshot.preview.document.component_id, component_id);
        let code_editor_binding = gallery.text_binding();
        assert_eq!(
            code_editor_binding.read_from(&gallery),
            Some("Plain text preview".into())
        );
        assert!(code_editor_binding.write_to(&mut gallery, "Revised plain text".into()));
        assert_eq!(gallery.text, "Revised plain text");
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&value_id],
            PropertyValue::String("Revised plain text".into())
        );

        let snapshot = gallery.host.snapshot().unwrap();
        assert!(snapshot.preview.document.actions.is_empty());
        let boundary = "x".repeat(2_000);
        let mut document = snapshot.preview.document.clone();
        document
            .properties
            .insert(value_id.clone(), PropertyValue::String(boundary.clone()));
        let encoded = encode_preview(&document, gallery.host.catalog()).unwrap();
        let decoded = decode_preview(&encoded, gallery.host.catalog()).unwrap();
        assert_eq!(
            decoded.properties[&value_id],
            PropertyValue::String(boundary)
        );

        document
            .properties
            .insert(value_id, PropertyValue::String("x".repeat(2_001)));
        assert!(encode_preview(&document, gallery.host.catalog()).is_err());
    }

    #[test]
    fn button_specimen_uses_serializable_inputs_and_signal_only_action() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_button_specimen();
        let component_id = ComponentId::new("button-specimen").unwrap();
        let label_id = PropertyId::new("label").unwrap();
        let enabled_id = PropertyId::new("enabled").unwrap();
        let event_id = EventId::new("activate").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 2);
        assert_eq!(schema.properties[&label_id].kind, PropertyKind::String);
        assert_eq!(schema.properties[&enabled_id].kind, PropertyKind::Boolean);
        assert_eq!(schema.events.len(), 1);
        assert!(schema.events[&event_id].payload.is_empty());

        let stale_signal = gallery.host.snapshot().unwrap().signal(event_id.clone());
        let label_context = PropertyBindingContext {
            property: label_id.clone(),
            stamp: gallery.stamp_lease.clone(),
        };
        let enabled_context = PropertyBindingContext {
            property: enabled_id.clone(),
            stamp: gallery.stamp_lease.clone(),
        };
        assert!(gallery.write_text(&label_context, "Save changes".into()));
        assert!(gallery.write_enabled(&enabled_context, false));
        let snapshot = gallery.host.snapshot().unwrap();
        assert_eq!(snapshot.stamp.revision, stale_signal.stamp.revision + 2);
        assert_eq!(
            snapshot.preview.properties[&label_id],
            PropertyValue::String("Save changes".into())
        );
        assert_eq!(
            snapshot.preview.properties[&enabled_id],
            PropertyValue::Boolean(false)
        );
        assert!(gallery.host.dispatch(&stale_signal).is_err());

        gallery.export_descriptor();
        let exported = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
        assert_eq!(
            exported.properties[&label_id],
            PropertyValue::String("Save changes".into())
        );
        assert_eq!(
            exported.properties[&enabled_id],
            PropertyValue::Boolean(false)
        );
        gallery.import_descriptor();
        let snapshot = gallery.host.snapshot().unwrap();
        assert_eq!(
            snapshot.preview.properties[&label_id],
            PropertyValue::String("Save changes".into())
        );
        assert_eq!(
            snapshot.preview.properties[&enabled_id],
            PropertyValue::Boolean(false)
        );

        let signal = snapshot.signal(event_id);
        gallery.activate(&signal);
        assert_eq!(gallery.activations, 1);
    }

    #[test]
    fn stepper_preview_uses_bounded_label_integer_feedback_and_rejects_invalid_imports() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_stepper();
        let component_id = ComponentId::new("stepper").unwrap();
        let label_id = PropertyId::new("label").unwrap();
        let value_id = PropertyId::new("value").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];

        assert_eq!(schema.properties.len(), 2);
        assert_eq!(schema.properties[&label_id].kind, PropertyKind::String);
        assert_eq!(
            schema.properties[&label_id].default,
            Some(PropertyValue::String("Preview count".into()))
        );
        assert_eq!(schema.properties[&label_id].min_length, Some(1));
        assert_eq!(
            schema.properties[&label_id].max_length,
            Some(PREVIEW_STEPPER_LABEL_MAX_LENGTH)
        );
        assert_eq!(schema.properties[&value_id].kind, PropertyKind::Integer);
        assert_eq!(schema.properties[&value_id].minimum, Some(0.0));
        assert_eq!(schema.properties[&value_id].maximum, Some(10.0));
        assert!(schema.events.is_empty());

        let stamp = gallery.host.snapshot().unwrap().stamp.clone();
        let context = PropertyBindingContext {
            property: value_id.clone(),
            stamp: gallery.stamp_lease.clone(),
        };
        let binding = Binding::new_with(
            PREVIEW_STEPPER_DEFAULT,
            context,
            CompiledPreviewGallery::read_stepper_value,
            CompiledPreviewGallery::write_stepper_value,
        );
        assert!(binding.write_to(&mut gallery, 6));
        assert_eq!(gallery.stepper_count, 6);
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&value_id],
            PropertyValue::Integer(6)
        );
        assert_eq!(
            gallery.host.snapshot().unwrap().stamp.revision,
            stamp.revision + 1
        );

        let stale_context = PropertyBindingContext {
            property: value_id.clone(),
            stamp: Rc::new(RefCell::new(Some(stamp))),
        };
        assert!(!gallery.write_stepper_value(&stale_context, 8));
        assert_eq!(
            gallery.read_stepper_value(&stale_context),
            PREVIEW_STEPPER_MIN
        );
        assert_eq!(gallery.stepper_count, 6);

        gallery.export_descriptor();
        let exported = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
        assert_eq!(exported.properties[&value_id], PropertyValue::Integer(6));
        gallery.import_descriptor();
        assert_eq!(gallery.stepper_count, 6);
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&value_id],
            PropertyValue::Integer(6)
        );

        let mut relabeled = exported.document.clone();
        relabeled.properties.insert(
            label_id.clone(),
            PropertyValue::String("Selected items".into()),
        );
        gallery.descriptor_json = encode_preview(&relabeled, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&label_id],
            PropertyValue::String("Selected items".into())
        );

        let valid_descriptor = gallery.descriptor_json.clone();
        let before_rejected_import = gallery.host.snapshot().unwrap().stamp.clone();
        let overlong_label = format!(
            "\"label\": \"{}\"",
            "x".repeat(PREVIEW_STEPPER_LABEL_MAX_LENGTH + 1)
        );
        let invalid_descriptors = [
            valid_descriptor.replace("\"label\": \"Selected items\"", "\"label\": \"   \""),
            valid_descriptor.replace("\"label\": \"Selected items\"", &overlong_label),
            valid_descriptor.replace("\"value\": 6", "\"value\": 11"),
            valid_descriptor.replace("\"value\": 6", "\"value\": \"six\""),
        ];
        for invalid_descriptor in invalid_descriptors {
            assert_ne!(invalid_descriptor, valid_descriptor);
            gallery.descriptor_json = invalid_descriptor;
            gallery.import_descriptor();
            assert!(gallery.status.starts_with("Import rejected:"));
            let snapshot = gallery.host.snapshot().unwrap();
            assert_eq!(snapshot.stamp, before_rejected_import);
            assert_eq!(
                snapshot.preview.properties[&label_id],
                PropertyValue::String("Selected items".into())
            );
            assert_eq!(
                snapshot.preview.properties[&value_id],
                PropertyValue::Integer(6)
            );
            assert_eq!(gallery.stepper_count, 6);
        }
    }

    #[test]
    fn radio_preview_uses_stamp_checked_selection_and_round_trips() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_radio();
        let component_id = ComponentId::new("radio").unwrap();
        let label_id = PropertyId::new("label").unwrap();
        let selected_id = PropertyId::new("selected").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 2);
        assert_eq!(schema.properties[&label_id].kind, PropertyKind::String);
        assert_eq!(schema.properties[&selected_id].kind, PropertyKind::Boolean);
        assert!(schema.events.is_empty());

        let stamp = gallery.host.snapshot().unwrap().stamp.clone();
        let label_context = PropertyBindingContext {
            property: label_id.clone(),
            stamp: gallery.stamp_lease.clone(),
        };
        let selected_context = PropertyBindingContext {
            property: selected_id.clone(),
            stamp: gallery.stamp_lease.clone(),
        };
        assert!(gallery.write_text(&label_context, "Monthly".into()));
        assert!(gallery.write_enabled(&selected_context, true));
        assert_eq!(
            gallery.host.snapshot().unwrap().stamp.revision,
            stamp.revision + 1
        );
        assert_eq!(gallery.text, "Monthly");
        assert!(gallery.enabled);

        gallery.export_descriptor();
        let decoded = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
        assert_eq!(
            decoded.properties[&label_id],
            PropertyValue::String("Monthly".into())
        );
        assert_eq!(
            decoded.properties[&selected_id],
            PropertyValue::Boolean(true)
        );

        gallery.import_descriptor();
        let mounted = gallery.host.snapshot().unwrap();
        assert_eq!(mounted.stamp.preview_id.as_str(), "compiled-gallery");
        assert_eq!(mounted.stamp.generation, stamp.generation + 1);
        assert_eq!(gallery.text, "Monthly");
        assert!(gallery.enabled);
        assert_eq!(
            mounted.preview.properties[&label_id],
            PropertyValue::String("Monthly".into())
        );
        assert_eq!(
            mounted.preview.properties[&selected_id],
            PropertyValue::Boolean(true)
        );
    }

    #[test]
    fn picker_specimen_uses_constrained_selection_and_round_trips() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_picker();
        let selection_id = PropertyId::new("selection").unwrap();
        let schema = &gallery.host.catalog().components[&ComponentId::new("picker").unwrap()];
        assert_eq!(
            schema.properties[&selection_id].allowed_values,
            vec![
                "compact".to_owned(),
                "comfortable".to_owned(),
                "spacious".to_owned()
            ]
        );

        let context = PropertyBindingContext {
            property: selection_id.clone(),
            stamp: gallery.stamp_lease.clone(),
        };
        assert!(gallery.write_text(&context, "spacious".into()));
        assert_eq!(gallery.text, "spacious");
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&selection_id],
            PropertyValue::String("spacious".into())
        );
        assert!(!gallery.write_text(&context, "unbounded".into()));
        assert_eq!(gallery.text, "spacious");

        gallery.export_descriptor();
        let decoded = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
        assert_eq!(
            decoded.properties[&selection_id],
            PropertyValue::String("spacious".into())
        );
        gallery.import_descriptor();
        assert_eq!(gallery.text, "spacious");
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&selection_id],
            PropertyValue::String("spacious".into())
        );
    }

    #[test]
    fn combo_box_specimen_uses_constrained_selection_and_round_trips() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_combo_box_specimen();
        let component_id = ComponentId::new("combo-box-specimen").unwrap();
        let selection_id = PropertyId::new("selection").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 1);
        assert_eq!(schema.properties[&selection_id].kind, PropertyKind::String);
        assert_eq!(
            schema.properties[&selection_id].default,
            Some(PropertyValue::String("rust".into()))
        );
        assert_eq!(
            schema.properties[&selection_id].allowed_values,
            vec!["rust".to_owned(), "swift".to_owned(), "go".to_owned()]
        );
        assert!(schema.events.is_empty());

        let initial_stamp = gallery.host.snapshot().unwrap().stamp.clone();
        let context = PropertyBindingContext {
            property: selection_id.clone(),
            stamp: gallery.stamp_lease.clone(),
        };
        assert!(gallery.write_text(&context, "swift".into()));
        assert_eq!(gallery.text, "swift");
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&selection_id],
            PropertyValue::String("swift".into())
        );
        assert_eq!(
            gallery.host.snapshot().unwrap().stamp.revision,
            initial_stamp.revision + 1
        );
        assert!(!gallery.write_text(&context, "kotlin".into()));
        assert_eq!(gallery.text, "swift");

        gallery.export_descriptor();
        let exported = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
        assert_eq!(
            exported.properties[&selection_id],
            PropertyValue::String("swift".into())
        );

        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap();
        assert_eq!(imported.stamp.preview_id.as_str(), "compiled-gallery");
        assert_eq!(imported.stamp.generation, initial_stamp.generation + 1);
        assert_eq!(gallery.text, "swift");
        assert_eq!(
            imported.preview.properties[&selection_id],
            PropertyValue::String("swift".into())
        );
    }

    #[test]
    fn tab_view_specimen_uses_constrained_selection_and_round_trips() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_tab_view_specimen();

        let component_id = ComponentId::new("tab-view-specimen").unwrap();
        let selection_id = PropertyId::new("selection").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 1);
        assert_eq!(schema.properties[&selection_id].kind, PropertyKind::String);
        assert_eq!(
            schema.properties[&selection_id].default,
            Some(PropertyValue::String("overview".into()))
        );
        assert_eq!(
            schema.properties[&selection_id].allowed_values,
            vec![
                "overview".to_owned(),
                "activity".to_owned(),
                "settings".to_owned()
            ]
        );
        assert!(schema.events.is_empty());

        let initial_stamp = gallery.host.snapshot().unwrap().stamp.clone();
        let context = PropertyBindingContext {
            property: selection_id.clone(),
            stamp: gallery.stamp_lease.clone(),
        };
        assert!(gallery.write_text(&context, "activity".into()));
        assert_eq!(gallery.text, "activity");
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&selection_id],
            PropertyValue::String("activity".into())
        );
        assert_eq!(
            gallery.host.snapshot().unwrap().stamp.revision,
            initial_stamp.revision + 1
        );
        assert!(!gallery.write_text(&context, "unknown".into()));
        assert_eq!(gallery.text, "activity");

        gallery.export_descriptor();
        let exported = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
        assert_eq!(
            exported.properties[&selection_id],
            PropertyValue::String("activity".into())
        );

        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap();
        assert_eq!(imported.stamp.preview_id.as_str(), "compiled-gallery");
        assert_eq!(imported.stamp.generation, initial_stamp.generation + 1);
        assert_eq!(gallery.text, "activity");
        assert_eq!(
            imported.preview.properties[&selection_id],
            PropertyValue::String("activity".into())
        );
    }

    #[test]
    fn color_picker_inspector_exposes_each_editable_property() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_color_picker_specimen();
        let component_id = ComponentId::new("color-picker-specimen").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        let visible = schema
            .properties
            .iter()
            .filter(|(_, property)| preview_inspector_property_is_visible(property))
            .map(|(id, _)| id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            visible,
            [
                "alpha",
                "disabled",
                "has_color",
                "hue",
                "label",
                "lightness",
                "saturation"
            ]
        );

        let context = PropertyBindingContext {
            property: PropertyId::new("has_color").unwrap(),
            stamp: gallery.stamp_lease.clone(),
        };
        assert!(gallery.read_color_presence(&context));
        let original_enabled = gallery.enabled;
        assert!(gallery.write_color_presence(&context, false));
        assert!(!gallery.read_color_presence(&context));
        assert_eq!(gallery.enabled, original_enabled);
        assert!(gallery.write_color_presence(&context, true));
        assert!(gallery.read_color_presence(&context));
        assert_eq!(gallery.enabled, original_enabled);

        let disabled_context = PropertyBindingContext {
            property: PropertyId::new("disabled").unwrap(),
            stamp: gallery.stamp_lease.clone(),
        };
        let disabled = gallery.preview_inspector_boolean_binding(disabled_context);
        assert!(disabled.write_to(&mut gallery, true));
        assert_eq!(
            property_boolean(gallery.host.snapshot().unwrap(), "disabled"),
            Some(true)
        );
        gallery.export_descriptor();
        gallery.import_descriptor();
        assert_eq!(
            property_boolean(gallery.host.snapshot().unwrap(), "disabled"),
            Some(true)
        );
    }
