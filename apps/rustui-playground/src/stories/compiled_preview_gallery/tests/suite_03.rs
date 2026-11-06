    #[test]
    fn color_picker_preview_uses_bounded_channels_and_round_trips() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_color_picker_specimen();

        let component_id = ComponentId::new("color-picker-specimen").unwrap();
        let has_color_id = PropertyId::new("has_color").unwrap();
        let hue_id = PropertyId::new("hue").unwrap();
        let saturation_id = PropertyId::new("saturation").unwrap();
        let lightness_id = PropertyId::new("lightness").unwrap();
        let alpha_id = PropertyId::new("alpha").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 7);
        let disabled = &schema.properties[&PropertyId::new("disabled").unwrap()];
        assert_eq!(disabled.kind, PropertyKind::Boolean);
        assert_eq!(disabled.default, Some(PropertyValue::Boolean(false)));
        assert!(disabled.editable);
        assert!(schema.events.is_empty());
        for id in [&hue_id, &saturation_id, &lightness_id, &alpha_id] {
            assert_eq!(schema.properties[id].kind, PropertyKind::Number);
            assert_eq!(schema.properties[id].minimum, Some(0.0));
            assert_eq!(schema.properties[id].maximum, Some(1.0));
            assert!(schema.properties[id].editable);
        }

        let initial_stamp = gallery.host.snapshot().unwrap().stamp.clone();
        let context = PropertyBindingContext {
            property: hue_id.clone(),
            stamp: gallery.stamp_lease.clone(),
        };
        let binding = Binding::new_with(
            gallery.read_color_value(&context),
            context.clone(),
            CompiledPreviewGallery::read_color_value,
            CompiledPreviewGallery::write_color_value,
        );
        let color = Hsla {
            h: 0.25,
            s: 0.4,
            l: 0.75,
            a: 0.8,
        };
        assert!(binding.write_to(&mut gallery, Some(color)));
        let snapshot = gallery.host.snapshot().unwrap();
        assert_eq!(snapshot.stamp.revision, initial_stamp.revision + 1);
        assert_eq!(
            snapshot.preview.properties[&has_color_id],
            PropertyValue::Boolean(true)
        );
        for (id, value) in [
            (&hue_id, color.h),
            (&saturation_id, color.s),
            (&lightness_id, color.l),
            (&alpha_id, color.a),
        ] {
            assert_eq!(
                snapshot.preview.properties[id],
                PropertyValue::Number(f64::from(value))
            );
        }

        let stale_context = PropertyBindingContext {
            property: hue_id.clone(),
            stamp: Rc::new(RefCell::new(Some(initial_stamp))),
        };
        assert_eq!(gallery.read_color_value(&stale_context), None);
        assert!(!gallery.write_color_value(&stale_context, Some(Hsla { h: 0.5, ..color })));
        assert!(!gallery.write_color_value(&context, Some(Hsla { h: 1.1, ..color })));
        assert_eq!(gallery.host.snapshot().unwrap().stamp.revision, 1);

        gallery.export_descriptor();
        let exported = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
        for id in [
            &has_color_id,
            &hue_id,
            &saturation_id,
            &lightness_id,
            &alpha_id,
        ] {
            assert_eq!(
                exported.properties[id],
                gallery.host.snapshot().unwrap().preview.properties[id]
            );
        }
        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap();
        assert_eq!(property_color(imported), Some(Some(color)));
        assert_eq!(
            imported.preview.properties[&hue_id],
            PropertyValue::Number(0.25)
        );

        let clear_context = PropertyBindingContext {
            property: hue_id.clone(),
            stamp: gallery.stamp_lease.clone(),
        };
        assert!(gallery.write_color_value(&clear_context, None));
        let cleared = gallery.host.snapshot().unwrap();
        assert_eq!(property_color(cleared), Some(None));
        assert_eq!(
            cleared.preview.properties[&hue_id],
            PropertyValue::Number(0.25)
        );
        gallery.export_descriptor();
        gallery.import_descriptor();
        assert_eq!(property_color(gallery.host.snapshot().unwrap()), Some(None));

        let presence_context = PropertyBindingContext {
            property: has_color_id.clone(),
            stamp: gallery.stamp_lease.clone(),
        };
        assert!(gallery.write_color_presence(&presence_context, true));
        assert_eq!(
            property_color(gallery.host.snapshot().unwrap()),
            Some(Some(color))
        );

        let mut invalid_document = gallery.host.snapshot().unwrap().preview.document.clone();
        invalid_document
            .properties
            .insert(hue_id, PropertyValue::Number(1.1));
        assert!(gallery.host.mount(invalid_document).is_err());
    }

    #[test]
    fn browser_reference_exports_full_catalog_and_keeps_protected_owner_value_private() {
        use crate::preview_transport::{CONSUMER, Delivery, PROTOCOL, Session, decode};
        fn send(
            owner: &mut CompiledPreviewGallery,
            token: Option<&str>,
            sequence: u64,
            command: serde_json::Value,
        ) -> crate::preview_transport::Reply {
            let request = decode(
                &serde_json::json!({
                    "protocol": PROTOCOL, "version": 1, "client": CONSUMER,
                    "request": format!("privacy-{sequence}"), "session": token,
                    "sequence": sequence.to_string(), "command": command,
                })
                .to_string(),
            )
            .unwrap();
            owner
                .reference_request(Delivery {
                    request,
                    fresh: "a".repeat(32),
                })
                .0
        }
        let mut owner = CompiledPreviewGallery::default();
        owner.teardown();
        owner.reference_session = Some(Session::default());
        let reply = send(
            &mut owner,
            None,
            0,
            serde_json::json!({"kind":"connect", "nonce":"b".repeat(32)}),
        );
        let token = reply.session.unwrap();
        let reply = send(
            &mut owner,
            Some(&token),
            1,
            serde_json::json!({"kind":"catalog"}),
        );
        let source = reply.result.unwrap()["source"].as_str().unwrap().to_owned();
        let catalog = decode_catalog(&source).unwrap();
        assert_eq!(
            catalog.components.len(),
            owner.host.catalog().components.len()
        );
        assert!(catalog.components.len() > 20);
        let secure = &catalog.components[&ComponentId::new("secure-field-specimen").unwrap()];
        assert_eq!(secure.properties.len(), 1);
        assert!(
            secure
                .properties
                .contains_key(&PropertyId::new("placeholder").unwrap())
        );
        owner.mount_secure_field_specimen();
        owner.secure_value = "owner-private-value-9223372036854775807".into();
        let stamp = owner.host.snapshot().unwrap().stamp.clone();
        let stamp = serde_json::json!({"preview_id":stamp.preview_id,"generation":stamp.generation.to_string(),"revision":stamp.revision.to_string()});
        let rejected = send(
            &mut owner,
            Some(&token),
            2,
            serde_json::json!({"kind":"edit_properties", "stamp":stamp, "updates_source":"[[\"value\",\"remote secret\"]]"}),
        );
        assert_eq!(rejected.error, Some("invalid_edit"));
        let reply = send(
            &mut owner,
            Some(&token),
            3,
            serde_json::json!({"kind":"export", "stamp":stamp}),
        );
        assert_eq!(reply.error, None);
        let wire = reply.encode();
        assert!(!wire.contains(&owner.secure_value));
        assert!(!wire.contains("remote secret"));
        assert_eq!(
            owner.secure_value,
            "owner-private-value-9223372036854775807"
        );
    }

    #[test]
    fn secure_field_preview_keeps_value_ephemeral_and_round_trips_configuration() {
        const TEST_SECRET: &str = "correct horse battery staple";

        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_secure_field_specimen();
        let component_id = ComponentId::new("secure-field-specimen").unwrap();
        let placeholder_id = PropertyId::new("placeholder").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 1);
        assert_eq!(
            schema.properties[&placeholder_id].kind,
            PropertyKind::String
        );
        assert_eq!(
            schema.properties[&placeholder_id].default,
            Some(PropertyValue::String("Enter a password".into()))
        );
        assert!(schema.events.is_empty());

        let placeholder_context = PropertyBindingContext {
            property: placeholder_id.clone(),
            stamp: gallery.stamp_lease.clone(),
        };
        assert!(gallery.write_text(&placeholder_context, "Account password".into()));

        let secure_binding = Binding::new_with(
            String::new(),
            PropertyBindingContext {
                property: placeholder_id.clone(),
                stamp: gallery.stamp_lease.clone(),
            },
            CompiledPreviewGallery::read_secure_value,
            CompiledPreviewGallery::write_secure_value,
        );
        assert!(secure_binding.write_to(&mut gallery, TEST_SECRET.into()));
        assert!(gallery.secure_value == TEST_SECRET);
        assert!(!gallery.status.contains(TEST_SECRET));

        gallery.export_descriptor();
        assert!(!gallery.descriptor_json.contains(TEST_SECRET));
        let exported = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
        assert_eq!(exported.properties.len(), 1);
        assert_eq!(
            exported.properties[&placeholder_id],
            PropertyValue::String("Account password".into())
        );
        assert!(exported.document.actions.is_empty());

        gallery.import_descriptor();
        assert_eq!(gallery.secure_field_placeholder, "Account password");
        assert!(gallery.secure_value.is_empty());
        assert!(!gallery.descriptor_json.contains(TEST_SECRET));
        let current_secure_binding = Binding::new_with(
            String::new(),
            PropertyBindingContext {
                property: placeholder_id.clone(),
                stamp: gallery.stamp_lease.clone(),
            },
            CompiledPreviewGallery::read_secure_value,
            CompiledPreviewGallery::write_secure_value,
        );
        const NEW_GENERATION_SECRET: &str = "new generation secret";
        assert!(current_secure_binding.write_to(&mut gallery, NEW_GENERATION_SECRET.into()));
        assert_eq!(secure_binding.read_from(&gallery), Some(String::new()));
        assert!(!gallery.descriptor_json.contains(NEW_GENERATION_SECRET));
        assert!(!secure_binding.write_to(&mut gallery, TEST_SECRET.into()));
        assert_eq!(gallery.secure_value, NEW_GENERATION_SECRET);
        assert!(!gallery.status.contains(TEST_SECRET));

        let mut text_preview = gallery.host.snapshot().unwrap().preview.document.clone();
        text_preview.component_id = ComponentId::new("text-field-specimen").unwrap();
        text_preview.properties = BTreeMap::from([
            (
                PropertyId::new("placeholder").unwrap(),
                PropertyValue::String("Preview text input".into()),
            ),
            (
                PropertyId::new("value").unwrap(),
                PropertyValue::String("Preview text input".into()),
            ),
        ]);
        gallery.descriptor_json = encode_preview(&text_preview, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        assert!(gallery.secure_value.is_empty());
        assert_eq!(
            gallery
                .host
                .snapshot()
                .unwrap()
                .preview
                .document
                .component_id,
            ComponentId::new("text-field-specimen").unwrap()
        );
    }

    #[test]
    fn menu_specimen_routes_payload_free_signal_and_round_trips_label() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_menu_specimen();

        let component_id = ComponentId::new("menu-specimen").unwrap();
        let label_id = PropertyId::new("trigger_label").unwrap();
        let event_id = EventId::new("select").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 1);
        assert_eq!(schema.properties[&label_id].kind, PropertyKind::String);
        assert_eq!(
            schema.properties[&label_id].default,
            Some(PropertyValue::String("Preview menu".into()))
        );
        assert_eq!(schema.properties[&label_id].min_length, Some(1));
        assert_eq!(schema.properties[&label_id].max_length, Some(80));
        assert_eq!(schema.events.len(), 1);
        assert!(schema.events[&event_id].payload.is_empty());

        let snapshot = gallery.host.snapshot().unwrap();
        assert_eq!(
            snapshot.preview.document.actions[&event_id],
            ActionRef::new("increment").unwrap()
        );
        let signal = snapshot.signal(event_id);
        gallery.activate(&signal);
        assert_eq!(gallery.activations, 1);

        let label_context = PropertyBindingContext {
            property: label_id.clone(),
            stamp: gallery.stamp_lease.clone(),
        };
        assert!(gallery.write_text(&label_context, "Actions".into()));
        gallery.activate(&signal);
        assert_eq!(gallery.activations, 1);
        assert!(gallery.status.contains("Rejected signal"));

        gallery.export_descriptor();
        let exported = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
        assert_eq!(
            exported.properties[&label_id],
            PropertyValue::String("Actions".into())
        );
        gallery.import_descriptor();
        assert_eq!(gallery.text, "Actions");
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&label_id],
            PropertyValue::String("Actions".into())
        );
    }

    #[test]
    fn link_specimen_keeps_bounded_label_and_url_and_dispatches_empty_signal() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_link_specimen();
        let component_id = ComponentId::new("link-specimen").unwrap();
        let label_id = PropertyId::new("label").unwrap();
        let url_id = PropertyId::new("url").unwrap();
        let event_id = EventId::new("activate").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 2);
        assert_eq!(schema.properties[&label_id].min_length, Some(1));
        assert_eq!(schema.properties[&label_id].max_length, Some(80));
        assert_eq!(schema.properties[&url_id].kind, PropertyKind::String);
        assert_eq!(schema.properties[&url_id].min_length, Some(1));
        assert_eq!(schema.properties[&url_id].max_length, Some(2048));
        assert_eq!(schema.events.len(), 1);
        assert!(schema.events[&event_id].payload.is_empty());

        let initial = gallery.host.snapshot().unwrap();
        assert_eq!(
            initial.preview.document.actions[&event_id],
            ActionRef::new("increment").unwrap()
        );
        let label_context = PropertyBindingContext {
            property: label_id.clone(),
            stamp: gallery.stamp_lease.clone(),
        };
        let url_context = PropertyBindingContext {
            property: url_id.clone(),
            stamp: gallery.stamp_lease.clone(),
        };
        assert!(gallery.write_text(&label_context, "Read the guide".into()));
        assert!(gallery.write_text(&url_context, "https://example.org/guide".into()));
        let edited = gallery.host.snapshot().unwrap();
        assert_eq!(
            edited.preview.properties[&label_id],
            PropertyValue::String("Read the guide".into())
        );
        assert_eq!(
            edited.preview.properties[&url_id],
            PropertyValue::String("https://example.org/guide".into())
        );

        let signal = edited.signal(event_id);
        gallery.activate(&signal);
        assert_eq!(gallery.activations, 1);

        gallery.export_descriptor();
        let exported = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
        assert_eq!(
            exported.properties[&label_id],
            PropertyValue::String("Read the guide".into())
        );
        assert_eq!(
            exported.properties[&url_id],
            PropertyValue::String("https://example.org/guide".into())
        );
        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap();
        assert_eq!(
            imported.preview.properties[&label_id],
            PropertyValue::String("Read the guide".into())
        );
        assert_eq!(
            imported.preview.properties[&url_id],
            PropertyValue::String("https://example.org/guide".into())
        );
    }

    #[test]
    fn tooltip_specimen_rejects_blank_text_import_without_replacing_snapshot() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_tooltip_specimen();

        assert!(
            gallery.status.starts_with("Mounted tooltip-specimen"),
            "{}",
            gallery.status
        );
        let text_id = PropertyId::new("text").unwrap();
        let before = gallery.host.snapshot().unwrap().clone();
        assert_eq!(
            before.preview.document.component_id.as_str(),
            "tooltip-specimen"
        );
        let mut invalid = before.preview.document.clone();
        invalid
            .properties
            .insert(text_id, PropertyValue::String(" \t".into()));
        assert!(matches!(
            invalid.properties.get(&PropertyId::new("text").unwrap()),
            Some(PropertyValue::String(text)) if text.trim().is_empty()
        ));
        gallery.descriptor_json = encode_preview(&invalid, gallery.host.catalog()).unwrap();

        gallery.import_descriptor();

        assert!(
            gallery.status.contains("Tooltip text must not be blank"),
            "unexpected import status: {}",
            gallery.status
        );
        let current = gallery.host.snapshot().unwrap();
        assert_eq!(current.stamp, before.stamp);
        assert_eq!(current.preview.document, before.preview.document);
    }

    #[test]
    fn spinner_specimen_round_trips_accessibility_label_descriptors() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_spinner_specimen();
        assert!(
            gallery.status.starts_with("Mounted spinner-specimen"),
            "{}",
            gallery.status
        );

        let component_id = ComponentId::new("spinner-specimen").unwrap();
        let label_id = PropertyId::new("accessibility_label").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 1);
        assert_eq!(schema.properties[&label_id].min_length, Some(1));
        assert_eq!(schema.properties[&label_id].max_length, Some(80));
        assert!(schema.events.is_empty());

        let mounted = gallery.host.snapshot().unwrap().clone();
        assert_eq!(
            mounted.preview.properties[&label_id],
            PropertyValue::String("Loading".into())
        );
        let mut customized = mounted.preview.document.clone();
        customized.properties.insert(
            label_id.clone(),
            PropertyValue::String("Uploading files".into()),
        );
        gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        assert!(gallery.status.contains("Imported descriptor"));
        let imported = gallery.host.snapshot().unwrap().clone();
        assert_eq!(
            imported.preview.properties[&label_id],
            PropertyValue::String("Uploading files".into())
        );

        let mut blank = imported.preview.document.clone();
        blank
            .properties
            .insert(label_id.clone(), PropertyValue::String(" \t".into()));
        gallery.descriptor_json = encode_preview(&blank, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        assert!(gallery.status.contains("Imported descriptor"));
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&label_id],
            PropertyValue::String(" \t".into())
        );
        gallery.export_descriptor();
        let exported = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
        assert_eq!(
            exported.properties[&label_id],
            PropertyValue::String(" \t".into())
        );
    }

    #[test]
    fn avatar_specimen_exposes_bounded_name_and_size_without_events() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_avatar_specimen();

        let component_id = ComponentId::new("avatar-specimen").unwrap();
        let name_id = PropertyId::new("name").unwrap();
        let diameter_id = PropertyId::new("diameter").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 2);
        assert_eq!(schema.properties[&name_id].kind, PropertyKind::String);
        assert_eq!(schema.properties[&name_id].min_length, Some(1));
        assert_eq!(schema.properties[&name_id].max_length, Some(80));
        assert_eq!(schema.properties[&diameter_id].kind, PropertyKind::Number);
        assert_eq!(schema.properties[&diameter_id].minimum, Some(24.0));
        assert_eq!(schema.properties[&diameter_id].maximum, Some(64.0));
        assert!(schema.events.is_empty());

        let snapshot = gallery.host.snapshot().unwrap();
        assert_eq!(
            snapshot.preview.properties[&name_id],
            PropertyValue::String("Ada Lovelace".into())
        );
        assert_eq!(
            snapshot.preview.properties[&diameter_id],
            PropertyValue::Number(40.0)
        );
        let mut edited = snapshot.preview.document.clone();
        edited.properties.insert(
            name_id.clone(),
            PropertyValue::String("Grace Hopper".into()),
        );
        edited
            .properties
            .insert(diameter_id.clone(), PropertyValue::Number(56.0));
        gallery.host.mount(edited).unwrap();
        gallery.export_descriptor();
        let exported = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
        assert_eq!(
            exported.properties[&name_id],
            PropertyValue::String("Grace Hopper".into())
        );
        assert_eq!(
            exported.properties[&diameter_id],
            PropertyValue::Number(56.0)
        );
        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap();
        assert_eq!(
            imported.preview.properties[&name_id],
            PropertyValue::String("Grace Hopper".into())
        );
        assert_eq!(
            imported.preview.properties[&diameter_id],
            PropertyValue::Number(56.0)
        );
    }

    #[test]
    fn badge_specimen_round_trips_bounded_count_and_dot_mode() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_badge_specimen();

        let component_id = ComponentId::new("badge-specimen").unwrap();
        let label_id = PropertyId::new("label").unwrap();
        let count_id = PropertyId::new("count").unwrap();
        let dot_id = PropertyId::new("dot").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 3);
        assert_eq!(schema.properties[&label_id].min_length, Some(1));
        assert_eq!(schema.properties[&label_id].max_length, Some(80));
        assert_eq!(schema.properties[&count_id].kind, PropertyKind::Integer);
        assert_eq!(schema.properties[&count_id].minimum, Some(0.0));
        assert_eq!(schema.properties[&count_id].maximum, Some(999.0));
        assert_eq!(schema.properties[&dot_id].kind, PropertyKind::Boolean);
        assert!(schema.events.is_empty());

        let initial = gallery.host.snapshot().unwrap();
        assert_eq!(
            initial.preview.properties[&label_id],
            PropertyValue::String("Inbox".into())
        );
        assert_eq!(
            initial.preview.properties[&count_id],
            PropertyValue::Integer(7)
        );
        assert_eq!(
            initial.preview.properties[&dot_id],
            PropertyValue::Boolean(false)
        );
        let mut edited = initial.preview.document.clone();
        edited
            .properties
            .insert(label_id.clone(), PropertyValue::String("Updates".into()));
        edited
            .properties
            .insert(count_id.clone(), PropertyValue::Integer(42));
        edited
            .properties
            .insert(dot_id.clone(), PropertyValue::Boolean(true));
        gallery.host.mount(edited).unwrap();

        gallery.export_descriptor();
        let exported = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
        assert_eq!(
            exported.properties[&label_id],
            PropertyValue::String("Updates".into())
        );
        assert_eq!(exported.properties[&count_id], PropertyValue::Integer(42));
        assert_eq!(exported.properties[&dot_id], PropertyValue::Boolean(true));
        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap();
        assert_eq!(
            imported.preview.properties[&label_id],
            PropertyValue::String("Updates".into())
        );
        assert_eq!(
            imported.preview.properties[&count_id],
            PropertyValue::Integer(42)
        );
        assert_eq!(
            imported.preview.properties[&dot_id],
            PropertyValue::Boolean(true)
        );
    }

    #[test]
    fn integer_inspector_uses_the_mounted_property_and_schema_bounds() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_badge_specimen();
        let count_id = PropertyId::new("count").unwrap();
        let count_context = PropertyBindingContext {
            property: count_id.clone(),
            stamp: gallery.stamp_lease.clone(),
        };

        assert_eq!(gallery.read_integer_property(&count_context), 7);
        assert!(gallery.write_integer_property(&count_context, 42));
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&count_id],
            PropertyValue::Integer(42)
        );
        assert!(gallery.write_integer_property(&count_context, 999));
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&count_id],
            PropertyValue::Integer(999)
        );
        assert!(!gallery.write_integer_property(&count_context, 1_000));
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&count_id],
            PropertyValue::Integer(999)
        );

        gallery.mount("stepper");
        let value_id = PropertyId::new("value").unwrap();
        let value_context = PropertyBindingContext {
            property: value_id.clone(),
            stamp: gallery.stamp_lease.clone(),
        };
        assert_eq!(
            gallery.read_integer_property(&value_context),
            i64::from(PREVIEW_STEPPER_DEFAULT)
        );
        assert!(gallery.write_integer_property(&value_context, 4));
        assert_eq!(gallery.stepper_count, 4);
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&value_id],
            PropertyValue::Integer(4)
        );
    }

    #[test]
    fn boolean_inspector_reads_the_mounted_property() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_badge_specimen();
        let dot_id = PropertyId::new("dot").unwrap();
        let dot_context = PropertyBindingContext {
            property: dot_id.clone(),
            stamp: gallery.stamp_lease.clone(),
        };

        // Badge's `dot` defaults to false while the gallery's shared Boolean
        // state defaults to true. The inspector must show the mounted value.
        assert!(gallery.enabled);
        let binding = gallery.preview_inspector_boolean_binding(dot_context.clone());
        assert!(!binding.value());
        assert!(!gallery.read_enabled(&dot_context));
        assert!(gallery.write_enabled(&dot_context, true));
        assert!(gallery.read_enabled(&dot_context));
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&dot_id],
            PropertyValue::Boolean(true)
        );
    }

    #[test]
    fn empty_state_specimen_bounds_imports_and_omits_blank_detail() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_empty_state_specimen();
        let mounted = gallery.host.snapshot().unwrap();
        assert_eq!(
            mounted.preview.document.component_id.as_str(),
            "empty-state-specimen"
        );
        assert!(gallery.status.contains("Mounted empty-state-specimen"));
        let component_id = ComponentId::new("empty-state-specimen").unwrap();
        let title_id = PropertyId::new("title").unwrap();
        let detail_id = PropertyId::new("detail").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties[&title_id].min_length, Some(1));
        assert_eq!(
            schema.properties[&title_id].max_length,
            Some(PREVIEW_EMPTYSTATE_TITLE_MAX_LENGTH)
        );
        assert_eq!(
            schema.properties[&detail_id].max_length,
            Some(PREVIEW_EMPTYSTATE_DETAIL_MAX_LENGTH)
        );
        assert!(schema.events.is_empty());
        assert_eq!(preview_nonblank_detail(" \t"), None);
        assert_eq!(preview_nonblank_detail(" Detail "), Some(" Detail "));

        let before = gallery.host.snapshot().unwrap().clone();
        let mut invalid = before.preview.document.clone();
        invalid
            .properties
            .insert(title_id.clone(), PropertyValue::String("  ".into()));
        gallery.descriptor_json = encode_preview(&invalid, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        assert!(
            gallery
                .status
                .contains("EmptyState title must not be blank")
        );
        assert_eq!(gallery.host.snapshot().unwrap().stamp, before.stamp);
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.document,
            before.preview.document
        );

        let mut valid = before.preview.document.clone();
        valid.properties.insert(
            title_id.clone(),
            PropertyValue::String("Nothing here".into()),
        );
        valid
            .properties
            .insert(detail_id.clone(), PropertyValue::String("  ".into()));
        gallery.descriptor_json = encode_preview(&valid, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap();
        assert_eq!(
            imported.preview.properties[&title_id],
            PropertyValue::String("Nothing here".into())
        );
        assert_eq!(
            imported.preview.properties[&detail_id],
            PropertyValue::String("  ".into())
        );
        gallery.export_descriptor();
        let round_trip = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
        assert_eq!(
            round_trip.properties[&detail_id],
            PropertyValue::String("  ".into())
        );
    }

    #[test]
    fn error_state_specimen_bounds_imports_and_omits_blank_detail() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_error_state_specimen();
        let mounted = gallery.host.snapshot().unwrap();
        assert_eq!(
            mounted.preview.document.component_id.as_str(),
            "error-state-specimen"
        );
        assert!(gallery.status.contains("Mounted error-state-specimen"));
        let component_id = ComponentId::new("error-state-specimen").unwrap();
        let title_id = PropertyId::new("title").unwrap();
        let detail_id = PropertyId::new("detail").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties[&title_id].kind, PropertyKind::String);
        assert_eq!(schema.properties[&title_id].min_length, Some(1));
        assert_eq!(
            schema.properties[&title_id].max_length,
            Some(PREVIEW_ERRORSTATE_TITLE_MAX_LENGTH)
        );
        assert_eq!(
            schema.properties[&detail_id].max_length,
            Some(PREVIEW_ERRORSTATE_DETAIL_MAX_LENGTH)
        );
        assert!(schema.events.is_empty());
        assert_eq!(preview_nonblank_detail(" \t"), None);
        assert_eq!(preview_nonblank_detail(" Detail "), Some(" Detail "));

        let before = gallery.host.snapshot().unwrap().clone();
        assert_eq!(
            before.preview.properties[&title_id],
            PropertyValue::String("Could not load".into())
        );
        assert_eq!(
            before.preview.properties[&detail_id],
            PropertyValue::String("Check your connection and try again.".into())
        );
        let mut invalid = before.preview.document.clone();
        invalid
            .properties
            .insert(title_id.clone(), PropertyValue::String("  ".into()));
        gallery.descriptor_json = encode_preview(&invalid, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        assert!(
            gallery
                .status
                .contains("ErrorState title must not be blank")
        );
        assert_eq!(gallery.host.snapshot().unwrap().stamp, before.stamp);
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.document,
            before.preview.document
        );

        let mut valid = before.preview.document.clone();
        valid.properties.insert(
            title_id.clone(),
            PropertyValue::String("Request failed".into()),
        );
        valid
            .properties
            .insert(detail_id.clone(), PropertyValue::String("  ".into()));
        gallery.descriptor_json = encode_preview(&valid, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap();
        assert_eq!(
            imported.preview.properties[&title_id],
            PropertyValue::String("Request failed".into())
        );
        assert_eq!(
            imported.preview.properties[&detail_id],
            PropertyValue::String("  ".into())
        );
        gallery.export_descriptor();
        let round_trip = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
        assert_eq!(
            round_trip.properties[&title_id],
            PropertyValue::String("Request failed".into())
        );
        assert_eq!(
            round_trip.properties[&detail_id],
            PropertyValue::String("  ".into())
        );
    }

    #[test]
    fn alert_specimen_dispatches_signal_only_dismiss_and_round_trips() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_alert_specimen();
        let component_id = ComponentId::new("alert-specimen").unwrap();
        let message_id = PropertyId::new("message").unwrap();
        let visible_id = PropertyId::new("visible").unwrap();
        let dismiss_id = EventId::new("dismiss").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties[&message_id].min_length, Some(1));
        assert_eq!(schema.properties[&message_id].max_length, Some(200));
        assert_eq!(schema.properties[&visible_id].kind, PropertyKind::Boolean);
        assert_eq!(schema.events.len(), 1);
        assert!(schema.events[&dismiss_id].payload.is_empty());

        let mounted = gallery.host.snapshot().unwrap().clone();
        assert_eq!(
            mounted.preview.document.actions[&dismiss_id],
            ActionRef::new("dismiss").unwrap()
        );
        gallery.dismiss_alert(&mounted.signal(dismiss_id.clone()));
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&visible_id],
            PropertyValue::Boolean(false)
        );

        gallery.export_descriptor();
        gallery.import_descriptor();
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&message_id],
            PropertyValue::String("Connection lost".into())
        );
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&visible_id],
            PropertyValue::Boolean(false)
        );

        let before = gallery.host.snapshot().unwrap().clone();
        let mut invalid = before.preview.document.clone();
        invalid
            .properties
            .insert(message_id, PropertyValue::String("  ".into()));
        gallery.descriptor_json = encode_preview(&invalid, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        assert!(gallery.status.contains("Alert message must not be blank"));
        assert_eq!(gallery.host.snapshot().unwrap().stamp, before.stamp);
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.document,
            before.preview.document
        );
    }

    #[test]
    fn stale_disclosure_binding_cannot_read_new_generation() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_disclosure_specimen();
        let open_id = PropertyId::new("open").unwrap();
        let stale_stamp = gallery.host.snapshot().unwrap().stamp.clone();
        let stale_context = PropertyBindingContext {
            property: open_id.clone(),
            stamp: Rc::new(RefCell::new(Some(stale_stamp))),
        };

        let mut document = gallery.host.snapshot().unwrap().preview.document.clone();
        document
            .properties
            .insert(open_id.clone(), PropertyValue::Boolean(true));
        gallery.descriptor_json = encode_preview(&document, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();

        let current_context = PropertyBindingContext {
            property: open_id,
            stamp: gallery.stamp_lease.clone(),
        };
        assert!(gallery.read_disclosure_open(&current_context));
        assert!(!gallery.read_disclosure_open(&stale_context));
    }

    #[test]
    fn disclosure_specimen_dispatches_signal_only_toggle_and_round_trips() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_disclosure_specimen();

        let component_id = ComponentId::new("disclosure-specimen").unwrap();
        let label_id = PropertyId::new("label").unwrap();
        let open_id = PropertyId::new("open").unwrap();
        let toggle_id = EventId::new("toggle").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties[&label_id].min_length, Some(1));
        assert_eq!(
            schema.properties[&label_id].max_length,
            Some(PREVIEW_DISCLOSURE_LABEL_MAX_LENGTH)
        );
        assert_eq!(schema.properties[&open_id].kind, PropertyKind::Boolean);
        assert_eq!(
            schema.properties[&open_id].default,
            Some(PropertyValue::Boolean(false))
        );
        assert_eq!(schema.events.len(), 1);
        assert!(schema.events[&toggle_id].payload.is_empty());

        let mounted = gallery.host.snapshot().unwrap().clone();
        assert_eq!(
            mounted.preview.document.actions.get(&toggle_id),
            Some(&ActionRef::new("toggle").unwrap()),
            "mounted actions: {:?}",
            mounted.preview.document.actions
        );
        let context = PropertyBindingContext {
            property: open_id.clone(),
            stamp: gallery.stamp_lease.clone(),
        };
        assert!(gallery.write_disclosure_open(&context, true));
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&open_id],
            PropertyValue::Boolean(true)
        );
        assert!(gallery.status.contains("Signal-only"));

        let stale_context = PropertyBindingContext {
            property: open_id.clone(),
            stamp: Rc::new(RefCell::new(Some(mounted.stamp))),
        };
        assert!(!gallery.write_disclosure_open(&stale_context, false));
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&open_id],
            PropertyValue::Boolean(true)
        );

        gallery.export_descriptor();
        gallery.import_descriptor();
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&label_id],
            PropertyValue::String("More information".into())
        );
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&open_id],
            PropertyValue::Boolean(true)
        );

        let before = gallery.host.snapshot().unwrap().clone();
        let mut invalid = before.preview.document.clone();
        invalid
            .properties
            .insert(label_id, PropertyValue::String("  ".into()));
        gallery.descriptor_json = encode_preview(&invalid, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        assert!(
            gallery
                .status
                .contains("Disclosure label must not be blank")
        );
        assert_eq!(gallery.host.snapshot().unwrap().stamp, before.stamp);
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.document,
            before.preview.document
        );
    }
