    #[test]
    fn icon_specimen_has_search_accessible_name_and_round_trips_size() {
        let mut gallery = CompiledPreviewGallery::default();
        let component_id = ComponentId::new("icon-specimen").unwrap();
        let label_id = PropertyId::new("accessibility_label").unwrap();
        let size_id = PropertyId::new("size").unwrap();
        let stale_context = PropertyBindingContext {
            property: label_id.clone(),
            stamp: Rc::new(RefCell::new(Some(
                gallery.host.snapshot().unwrap().stamp.clone(),
            ))),
        };
        gallery.mount_icon_specimen();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 2);
        assert_eq!(schema.properties[&label_id].kind, PropertyKind::String);
        assert_eq!(schema.properties[&label_id].min_length, Some(1));
        assert_eq!(schema.properties[&label_id].max_length, Some(80));
        assert_eq!(schema.properties[&size_id].kind, PropertyKind::Number);
        assert_eq!(
            schema.properties[&size_id].default,
            Some(PropertyValue::Number(24.0))
        );
        assert_eq!(schema.properties[&size_id].minimum, Some(8.0));
        assert_eq!(schema.properties[&size_id].maximum, Some(64.0));
        assert!(schema.events.is_empty());

        let mounted = gallery.host.snapshot().unwrap();
        assert_eq!(
            mounted.preview.properties[&label_id],
            PropertyValue::String("Search".into())
        );
        assert_eq!(
            mounted.preview.properties[&size_id],
            PropertyValue::Number(24.0)
        );
        let label_context = PropertyBindingContext {
            property: label_id.clone(),
            stamp: gallery.stamp_lease.clone(),
        };
        assert_eq!(gallery.read_text(&label_context), "Search");
        assert_eq!(gallery.read_text(&stale_context), "");
        gallery.export_descriptor();
        gallery.import_descriptor();
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&label_id],
            PropertyValue::String("Search".into())
        );

        let mut edited = gallery.host.snapshot().unwrap().preview.document.clone();
        edited
            .properties
            .insert(label_id.clone(), PropertyValue::String("Find item".into()));
        edited
            .properties
            .insert(size_id.clone(), PropertyValue::Number(36.0));
        gallery.descriptor_json = encode_preview(&edited, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap();
        assert_eq!(
            imported.preview.properties[&label_id],
            PropertyValue::String("Find item".into())
        );
        assert_eq!(
            imported.preview.properties[&size_id],
            PropertyValue::Number(36.0)
        );
        let imported_label_context = PropertyBindingContext {
            property: label_id.clone(),
            stamp: gallery.stamp_lease.clone(),
        };
        assert_eq!(gallery.read_text(&imported_label_context), "Find item");
        gallery.export_descriptor();
        let exported = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
        assert_eq!(
            exported.properties[&label_id],
            PropertyValue::String("Find item".into())
        );
        assert_eq!(exported.properties[&size_id], PropertyValue::Number(36.0));
    }

    #[test]
    fn outline_specimen_round_trips_controlled_expansion_and_rejects_stale_writes() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_outline_specimen();
        let component_id = ComponentId::new("outline-specimen").unwrap();
        let expanded_id = PropertyId::new("expanded").unwrap();
        let selected_id = PropertyId::new("selected_id").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 6);
        assert_eq!(
            schema.properties[&expanded_id].allowed_values,
            ["collapsed", "root", "nested", "all"]
        );
        let initial_stamp = gallery.host.snapshot().unwrap().stamp.clone();
        let stale = CompiledPreviewGallery::outline_expansion_binding(
            "all".into(),
            PropertyBindingContext {
                property: expanded_id.clone(),
                stamp: Rc::new(RefCell::new(Some(initial_stamp))),
            },
        );
        let expansion = CompiledPreviewGallery::outline_expansion_binding(
            "all".into(),
            PropertyBindingContext {
                property: expanded_id.clone(),
                stamp: gallery.stamp_lease.clone(),
            },
        );
        let selection = CompiledPreviewGallery::tree_selection_binding(
            None,
            PropertyBindingContext {
                property: selected_id.clone(),
                stamp: gallery.stamp_lease.clone(),
            },
        );
        assert!(selection.write_to(&mut gallery, Some("readme".into())));
        assert!(!stale.write_to(&mut gallery, Default::default()));
        for (ids, encoded) in [
            (vec!["src"], "nested"),
            (vec![], "collapsed"),
            (vec!["root"], "root"),
            (vec!["root", "src"], "all"),
        ] {
            let ids = ids.into_iter().map(str::to_owned).collect();
            assert!(expansion.write_to(&mut gallery, ids));
            assert_eq!(
                property_string(gallery.host.snapshot().unwrap(), "expanded").as_deref(),
                Some(encoded)
            );
            assert_eq!(selection.read_from(&gallery), Some(Some("readme".into())));
        }
        let before_invalid = gallery.host.snapshot().unwrap().clone();
        assert!(!expansion.write_to(&mut gallery, ["readme".into()].into_iter().collect()));
        assert_eq!(gallery.host.snapshot().unwrap().stamp, before_invalid.stamp);
        let label = PropertyBindingContext {
            property: PropertyId::new("root_label").unwrap(),
            stamp: gallery.stamp_lease.clone(),
        };
        assert!(!gallery.write_text(&label, " \t".into()));
        assert_eq!(gallery.host.snapshot().unwrap().stamp, before_invalid.stamp);
        assert!(expansion.write_to(&mut gallery, ["src".into()].into_iter().collect()));
        gallery.export_descriptor();
        let saved = gallery.descriptor_json.clone();
        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap().clone();
        assert_eq!(
            property_string(&imported, "expanded").as_deref(),
            Some("nested")
        );
        assert_eq!(
            property_string(&imported, "selected_id").as_deref(),
            Some("readme")
        );
        assert_eq!(
            decode_preview(&saved, gallery.host.catalog())
                .unwrap()
                .document
                .component_id,
            component_id
        );
        gallery.descriptor_json = saved.replacen("\"nested\"", "\"unknown\"", 1);
        gallery.import_descriptor();
        assert!(gallery.status.starts_with("Import rejected:"));
        assert_eq!(gallery.host.snapshot().unwrap().stamp, imported.stamp);
        gallery.descriptor_json = saved.replacen("\"Project\"", "\"   \"", 1);
        gallery.import_descriptor();
        assert!(
            gallery
                .status
                .contains("Outline root_label must not be blank")
        );
        assert_eq!(gallery.host.snapshot().unwrap().stamp, imported.stamp);
    }

    #[test]
    fn tree_specimen_round_trips_fixed_nodes_and_rejects_invalid_imports() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_tree_specimen();

        let component_id = ComponentId::new("tree-specimen").unwrap();
        let selected_id = PropertyId::new("selected_id").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 5);
        assert!(schema.events.is_empty());
        assert_eq!(
            schema.properties[&selected_id].allowed_values,
            vec!["none", "root", "src", "readme", "tests"]
        );
        assert_eq!(PREVIEW_TREE_NODE_IDS, ["root", "src", "readme", "tests"]);
        assert_eq!(
            PREVIEW_TREE_NODE_IDS
                .iter()
                .copied()
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            4
        );

        for (name, default) in [
            ("root_label", "Project"),
            ("src_label", "src"),
            ("readme_label", "README.md"),
            ("tests_label", "tests"),
        ] {
            let property_id = PropertyId::new(name).unwrap();
            let property = &schema.properties[&property_id];
            assert_eq!(property.kind, PropertyKind::String);
            assert_eq!(property.min_length, Some(1));
            assert_eq!(property.max_length, Some(PREVIEW_TREE_LABEL_MAX_LENGTH));
            assert_eq!(
                property.default,
                Some(PropertyValue::String(default.into()))
            );
        }

        let mounted = gallery.host.snapshot().unwrap().clone();
        assert_eq!(mounted.preview.document.properties.len(), 5);
        assert!(mounted.preview.document.actions.is_empty());
        assert_eq!(
            mounted.preview.document.properties[&selected_id],
            PropertyValue::String("none".into())
        );

        let before_blank_edits = gallery.host.snapshot().unwrap().clone();
        for name in ["root_label", "src_label", "readme_label", "tests_label"] {
            let context = PropertyBindingContext {
                property: PropertyId::new(name).unwrap(),
                stamp: gallery.stamp_lease.clone(),
            };
            assert!(!gallery.write_text(&context, " \t ".into()));
        }
        let after_blank_edits = gallery.host.snapshot().unwrap();
        assert_eq!(after_blank_edits.stamp, before_blank_edits.stamp);
        assert_eq!(
            after_blank_edits.preview.document,
            before_blank_edits.preview.document
        );

        let root_label_context = PropertyBindingContext {
            property: PropertyId::new("root_label").unwrap(),
            stamp: gallery.stamp_lease.clone(),
        };
        assert!(gallery.write_text(
            &root_label_context,
            "x".repeat(PREVIEW_TREE_LABEL_MAX_LENGTH)
        ));
        assert!(!gallery.write_text(
            &root_label_context,
            "x".repeat(PREVIEW_TREE_LABEL_MAX_LENGTH + 1)
        ));
        assert!(gallery.write_text(&root_label_context, "Project".into()));

        let stale_stamp = gallery.host.snapshot().unwrap().stamp.clone();
        let stale_selection = CompiledPreviewGallery::tree_selection_binding(
            None,
            PropertyBindingContext {
                property: selected_id.clone(),
                stamp: Rc::new(RefCell::new(Some(stale_stamp))),
            },
        );
        let selection = CompiledPreviewGallery::tree_selection_binding(
            None,
            PropertyBindingContext {
                property: selected_id.clone(),
                stamp: gallery.stamp_lease.clone(),
            },
        );
        assert!(selection.write_to(&mut gallery, Some("tests".into())));
        assert!(!stale_selection.write_to(&mut gallery, Some("src".into())));
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.document.properties[&selected_id],
            PropertyValue::String("tests".into())
        );
        assert!(selection.write_to(&mut gallery, None));

        gallery.export_descriptor();
        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap().clone();
        assert_eq!(
            imported.preview.document.properties[&selected_id],
            PropertyValue::String("none".into())
        );
        let before_invalid_import = imported.clone();

        let invalid_selection = gallery
            .descriptor_json
            .replacen("\"none\"", "\"unknown\"", 1);
        assert_ne!(invalid_selection, gallery.descriptor_json);
        gallery.descriptor_json = invalid_selection;
        gallery.import_descriptor();
        assert!(gallery.status.starts_with("Import rejected:"));
        let after_invalid_selection = gallery.host.snapshot().unwrap();
        assert_eq!(after_invalid_selection.stamp, before_invalid_import.stamp);
        assert_eq!(
            after_invalid_selection.preview.document,
            before_invalid_import.preview.document
        );

        for (name, default) in [
            ("root_label", "Project"),
            ("src_label", "src"),
            ("readme_label", "README.md"),
            ("tests_label", "tests"),
        ] {
            gallery.export_descriptor();
            let invalid_label =
                gallery
                    .descriptor_json
                    .replacen(&format!("\"{default}\""), "\" \"", 1);
            assert_ne!(invalid_label, gallery.descriptor_json);
            gallery.descriptor_json = invalid_label;
            gallery.import_descriptor();
            assert!(
                gallery
                    .status
                    .contains(&format!("Tree {name} must not be blank"))
            );
            let after_blank_label = gallery.host.snapshot().unwrap();
            assert_eq!(after_blank_label.stamp, before_invalid_import.stamp);
            assert_eq!(
                after_blank_label.preview.document,
                before_invalid_import.preview.document
            );
        }
    }

    #[test]
    fn list_specimen_round_trips_selection_and_rejects_invalid_imports() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_list_specimen();

        let component_id = ComponentId::new("list-specimen").unwrap();
        let selected_id = PropertyId::new("selected_id").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 1);
        assert!(schema.events.is_empty());
        assert_eq!(
            schema.properties[&selected_id].default,
            Some(PropertyValue::String("none".into()))
        );
        assert_eq!(
            schema.properties[&selected_id].allowed_values,
            PREVIEW_LIST_SELECTIONS
        );
        assert_eq!(
            PREVIEW_LIST_ROWS,
            ["First item", "Second item", "Third item"]
        );

        let mounted = gallery.host.snapshot().unwrap().clone();
        assert_eq!(mounted.preview.document.properties.len(), 1);
        assert!(mounted.preview.document.actions.is_empty());
        assert_eq!(
            mounted.preview.document.properties[&selected_id],
            PropertyValue::String("none".into())
        );

        let stale_stamp = mounted.stamp.clone();
        let stale_selection = CompiledPreviewGallery::list_selection_binding(
            None,
            PropertyBindingContext {
                property: selected_id.clone(),
                stamp: Rc::new(RefCell::new(Some(stale_stamp))),
            },
        );
        let selection = CompiledPreviewGallery::list_selection_binding(
            None,
            PropertyBindingContext {
                property: selected_id.clone(),
                stamp: gallery.stamp_lease.clone(),
            },
        );
        assert!(selection.write_to(&mut gallery, Some(2)));
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.document.properties[&selected_id],
            PropertyValue::String("row-2".into())
        );
        assert!(!stale_selection.write_to(&mut gallery, Some(1)));
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.document.properties[&selected_id],
            PropertyValue::String("row-2".into())
        );
        assert!(!selection.write_to(&mut gallery, Some(PREVIEW_LIST_ROWS.len())));
        assert!(selection.write_to(&mut gallery, None));
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.document.properties[&selected_id],
            PropertyValue::String("none".into())
        );

        gallery.export_descriptor();
        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap().clone();
        assert_eq!(
            imported.preview.document.properties[&selected_id],
            PropertyValue::String("none".into())
        );
        let before_invalid_import = imported.clone();

        let invalid_selection = gallery.descriptor_json.replacen("\"none\"", "\"row-9\"", 1);
        assert_ne!(invalid_selection, gallery.descriptor_json);
        gallery.descriptor_json = invalid_selection;
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
    fn table_specimen_round_trips_bounded_selection_feedback() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_table_specimen();

        let component_id = ComponentId::new("table-specimen").unwrap();
        let selected_id = PropertyId::new("selected_id").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 1);
        assert!(schema.events.is_empty());
        assert_eq!(
            schema.properties[&selected_id].default,
            Some(PropertyValue::String("none".into()))
        );
        assert_eq!(
            schema.properties[&selected_id].allowed_values,
            PREVIEW_LIST_SELECTIONS
        );
        assert_eq!(PREVIEW_TABLE_COLUMNS, ["Task", "Status"]);
        assert_eq!(PREVIEW_TABLE_ROWS.len(), 3);
        assert!(
            PREVIEW_TABLE_ROWS
                .iter()
                .all(|row| row.len() == PREVIEW_TABLE_COLUMNS.len())
        );
        assert_eq!(PREVIEW_TABLE_ROWS[0], ["Design", "Complete"]);
        assert_eq!(PREVIEW_TABLE_ROWS[1], ["Implement", "In progress"]);
        assert_eq!(PREVIEW_TABLE_ROWS[2], ["Review", "Queued"]);

        let mounted = gallery.host.snapshot().unwrap().clone();
        assert_eq!(mounted.preview.document.properties.len(), 1);
        assert!(mounted.preview.document.actions.is_empty());
        assert_eq!(
            mounted.preview.document.properties[&selected_id],
            PropertyValue::String("none".into())
        );

        let stale_selection = CompiledPreviewGallery::data_grid_selection_binding(
            None,
            PropertyBindingContext {
                property: selected_id.clone(),
                stamp: Rc::new(RefCell::new(Some(mounted.stamp.clone()))),
            },
        );
        let selection = CompiledPreviewGallery::data_grid_selection_binding(
            None,
            PropertyBindingContext {
                property: selected_id.clone(),
                stamp: gallery.stamp_lease.clone(),
            },
        );
        assert!(selection.write_to(&mut gallery, Some(2)));
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.document.properties[&selected_id],
            PropertyValue::String("row-2".into())
        );
        assert!(!stale_selection.write_to(&mut gallery, Some(1)));
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.document.properties[&selected_id],
            PropertyValue::String("row-2".into())
        );
        assert!(!selection.write_to(&mut gallery, Some(PREVIEW_TABLE_ROWS.len())));
        assert!(selection.write_to(&mut gallery, None));
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.document.properties[&selected_id],
            PropertyValue::String("none".into())
        );

        gallery.export_descriptor();
        gallery.import_descriptor();
        let before_invalid_import = gallery.host.snapshot().unwrap().clone();
        assert_eq!(
            before_invalid_import.preview.document.properties[&selected_id],
            PropertyValue::String("none".into())
        );
        let invalid_selection = gallery.descriptor_json.replacen("\"none\"", "\"row-9\"", 1);
        assert_ne!(invalid_selection, gallery.descriptor_json);
        gallery.descriptor_json = invalid_selection;
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
    fn data_grid_specimen_round_trips_selection_and_rejects_invalid_imports() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_data_grid_specimen();

        let component_id = ComponentId::new("data-grid-specimen").unwrap();
        let label_id = PropertyId::new("label").unwrap();
        let selected_id = PropertyId::new("selected_id").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 2);
        assert!(schema.events.is_empty());
        assert_eq!(
            schema.properties[&label_id].default,
            Some(PropertyValue::String("Quarterly revenue".into()))
        );
        assert_eq!(schema.properties[&label_id].min_length, Some(1));
        assert_eq!(
            schema.properties[&label_id].max_length,
            Some(PREVIEW_DATAGRID_LABEL_MAX_LENGTH)
        );
        assert_eq!(
            schema.properties[&selected_id].default,
            Some(PropertyValue::String("none".into()))
        );
        assert_eq!(
            schema.properties[&selected_id].allowed_values,
            vec!["none", "row-0", "row-1", "row-2"]
        );
        assert_eq!(PREVIEW_DATAGRID_COLUMNS, ["Quarter", "Revenue"]);
        assert_eq!(PREVIEW_DATAGRID_ROWS.len(), 3);
        assert!(
            PREVIEW_DATAGRID_ROWS
                .iter()
                .all(|row| row.len() == PREVIEW_DATAGRID_COLUMNS.len())
        );
        assert_eq!(PREVIEW_DATAGRID_ROWS[0], ["Q1", "$18.2M"]);
        assert_eq!(PREVIEW_DATAGRID_ROWS[1], ["Q2", "$20.4M"]);
        assert_eq!(PREVIEW_DATAGRID_ROWS[2], ["Q3", "$22.1M"]);

        let mounted = gallery.host.snapshot().unwrap().clone();
        assert_eq!(mounted.preview.document.properties.len(), 2);
        assert!(mounted.preview.document.actions.is_empty());
        assert_eq!(
            mounted.preview.document.properties[&selected_id],
            PropertyValue::String("none".into())
        );

        let label_context = PropertyBindingContext {
            property: label_id.clone(),
            stamp: gallery.stamp_lease.clone(),
        };
        let before_blank_edit = gallery.host.snapshot().unwrap().clone();
        assert!(!gallery.write_text(&label_context, " \t ".into()));
        let after_blank_edit = gallery.host.snapshot().unwrap();
        assert_eq!(after_blank_edit.stamp, before_blank_edit.stamp);
        assert_eq!(
            after_blank_edit.preview.document,
            before_blank_edit.preview.document
        );
        assert!(gallery.write_text(
            &label_context,
            "x".repeat(PREVIEW_DATAGRID_LABEL_MAX_LENGTH)
        ));
        assert!(!gallery.write_text(
            &label_context,
            "x".repeat(PREVIEW_DATAGRID_LABEL_MAX_LENGTH + 1)
        ));
        assert!(gallery.write_text(&label_context, "Quarterly revenue".into()));

        let stale_stamp = gallery.host.snapshot().unwrap().stamp.clone();
        let stale_selection = CompiledPreviewGallery::data_grid_selection_binding(
            None,
            PropertyBindingContext {
                property: selected_id.clone(),
                stamp: Rc::new(RefCell::new(Some(stale_stamp))),
            },
        );

        let selection = CompiledPreviewGallery::data_grid_selection_binding(
            None,
            PropertyBindingContext {
                property: selected_id.clone(),
                stamp: gallery.stamp_lease.clone(),
            },
        );
        assert!(selection.write_to(&mut gallery, Some(2)));
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.document.properties[&selected_id],
            PropertyValue::String("row-2".into())
        );
        assert!(!stale_selection.write_to(&mut gallery, Some(1)));
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.document.properties[&selected_id],
            PropertyValue::String("row-2".into())
        );
        assert!(selection.write_to(&mut gallery, None));
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.document.properties[&selected_id],
            PropertyValue::String("none".into())
        );

        gallery.export_descriptor();
        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap().clone();
        assert_eq!(
            imported.preview.document.properties[&selected_id],
            PropertyValue::String("none".into())
        );
        let before_invalid_import = imported.clone();

        let invalid_selection = gallery.descriptor_json.replacen("\"none\"", "\"row-9\"", 1);
        assert_ne!(invalid_selection, gallery.descriptor_json);
        gallery.descriptor_json = invalid_selection;
        gallery.import_descriptor();
        assert!(gallery.status.starts_with("Import rejected:"));
        let after_invalid_selection = gallery.host.snapshot().unwrap();
        assert_eq!(after_invalid_selection.stamp, before_invalid_import.stamp);
        assert_eq!(
            after_invalid_selection.preview.document,
            before_invalid_import.preview.document
        );

        gallery.export_descriptor();
        let blank_label = gallery
            .descriptor_json
            .replacen("\"Quarterly revenue\"", "\" \"", 1);
        assert_ne!(blank_label, gallery.descriptor_json);
        gallery.descriptor_json = blank_label;
        gallery.import_descriptor();
        assert!(gallery.status.contains("DataGrid label must not be blank"));
        let after_blank_label = gallery.host.snapshot().unwrap();
        assert_eq!(after_blank_label.stamp, before_invalid_import.stamp);
        assert_eq!(
            after_blank_label.preview.document,
            before_invalid_import.preview.document
        );
    }

    #[test]
    fn navigation_split_specimen_round_trips_selection_and_rejects_invalid_imports() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_navigation_split_specimen();

        let component_id = ComponentId::new("navigation-split-specimen").unwrap();
        let selected_id = PropertyId::new("selected_id").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 1);
        assert!(schema.events.is_empty());
        assert_eq!(
            schema.properties[&selected_id].default,
            Some(PropertyValue::String("none".into()))
        );
        assert_eq!(
            schema.properties[&selected_id].allowed_values,
            PREVIEW_NAVIGATION_SPLIT_SELECTIONS
        );

        let mounted = gallery.host.snapshot().unwrap().clone();
        assert_eq!(mounted.preview.document.properties.len(), 1);
        assert!(mounted.preview.document.actions.is_empty());
        assert_eq!(
            mounted.preview.document.properties[&selected_id],
            PropertyValue::String("none".into())
        );
        let selection = CompiledPreviewGallery::string_selection_binding(
            Some("files".into()),
            PropertyBindingContext {
                property: selected_id.clone(),
                stamp: gallery.stamp_lease.clone(),
            },
        );
        assert!(selection.write_to(&mut gallery, Some("settings".into())));
        let selected = gallery.host.snapshot().unwrap().clone();
        assert_eq!(selected.stamp.generation, mounted.stamp.generation);
        assert_eq!(selected.stamp.revision, mounted.stamp.revision + 1);
        assert_eq!(
            selected.preview.document.properties[&selected_id],
            PropertyValue::String("settings".into())
        );

        gallery.export_descriptor();
        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap().clone();
        assert_eq!(
            imported.preview.document.properties[&selected_id],
            PropertyValue::String("settings".into())
        );
        let invalid_selection =
            gallery
                .descriptor_json
                .replacen(r#""settings""#, r#""unknown""#, 1);
        assert_ne!(invalid_selection, gallery.descriptor_json);
        gallery.descriptor_json = invalid_selection;
        gallery.import_descriptor();
        assert!(gallery.status.starts_with("Import rejected:"));
        let after_invalid_import = gallery.host.snapshot().unwrap();
        assert_eq!(after_invalid_import.stamp, imported.stamp);
        assert_eq!(
            after_invalid_import.preview.document,
            imported.preview.document
        );
    }

    #[test]
    fn navigation_stack_specimen_pushes_backs_round_trips_and_rejects_invalid_imports() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_navigation_stack_specimen();

        let component_id = ComponentId::new("navigation-stack-specimen").unwrap();
        let selected_id = PropertyId::new("selected_id").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 1);
        assert!(schema.events.is_empty());
        assert_eq!(
            schema.properties[&selected_id].default,
            Some(PropertyValue::String("none".into()))
        );
        assert_eq!(
            schema.properties[&selected_id].allowed_values,
            PREVIEW_NAVIGATION_STACK_SELECTIONS
        );

        let mounted = gallery.host.snapshot().unwrap().clone();
        assert_eq!(mounted.preview.document.properties.len(), 1);
        assert!(mounted.preview.document.actions.is_empty());
        assert_eq!(
            mounted.preview.document.properties[&selected_id],
            PropertyValue::String("none".into())
        );
        let path = CompiledPreviewGallery::navigation_stack_path_binding(
            Vec::new(),
            PropertyBindingContext {
                property: selected_id.clone(),
                stamp: gallery.stamp_lease.clone(),
            },
        );

        assert!(path.update_on(&mut gallery, |path| {
            path.push("details".into());
            true
        }));
        let pushed = gallery.host.snapshot().unwrap().clone();
        assert_eq!(pushed.stamp.generation, mounted.stamp.generation);
        assert_eq!(pushed.stamp.revision, mounted.stamp.revision + 1);
        assert_eq!(
            pushed.preview.document.properties[&selected_id],
            PropertyValue::String("details".into())
        );

        let before_invalid_path = pushed.clone();
        assert!(!path.write_to(&mut gallery, vec!["details".into(), "unknown".into()]));
        let after_invalid_path = gallery.host.snapshot().unwrap();
        assert_eq!(after_invalid_path.stamp, before_invalid_path.stamp);
        assert_eq!(
            after_invalid_path.preview.document,
            before_invalid_path.preview.document
        );

        gallery.export_descriptor();
        gallery.import_descriptor();
        let imported_details = gallery.host.snapshot().unwrap().clone();
        assert_eq!(
            imported_details.preview.document.properties[&selected_id],
            PropertyValue::String("details".into())
        );

        let details_path = CompiledPreviewGallery::navigation_stack_path_binding(
            vec!["details".into()],
            PropertyBindingContext {
                property: selected_id.clone(),
                stamp: gallery.stamp_lease.clone(),
            },
        );
        assert!(details_path.update_on(&mut gallery, |path| path.pop().is_some()));
        let backed = gallery.host.snapshot().unwrap().clone();
        assert_eq!(
            backed.preview.document.properties[&selected_id],
            PropertyValue::String("none".into())
        );
        gallery.export_descriptor();
        gallery.import_descriptor();
        let imported_none = gallery.host.snapshot().unwrap().clone();
        assert_eq!(
            imported_none.preview.document.properties[&selected_id],
            PropertyValue::String("none".into())
        );

        let invalid_route = gallery
            .descriptor_json
            .replacen(r#""none""#, r#""unknown""#, 1);
        assert_ne!(invalid_route, gallery.descriptor_json);
        gallery.descriptor_json = invalid_route;
        gallery.import_descriptor();
        assert!(gallery.status.starts_with("Import rejected:"));
        let after_invalid_import = gallery.host.snapshot().unwrap();
        assert_eq!(after_invalid_import.stamp, imported_none.stamp);
        assert_eq!(
            after_invalid_import.preview.document,
            imported_none.preview.document
        );
    }

    #[test]
    fn collection_specimen_round_trips_stable_selection_and_rejects_invalid_imports() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_collection_specimen();

        let component_id = ComponentId::new("collection-specimen").unwrap();
        let selected_id = PropertyId::new("selected_id").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 1);
        assert!(schema.events.is_empty());
        assert_eq!(
            schema.properties[&selected_id].default,
            Some(PropertyValue::String("none".into()))
        );
        assert_eq!(
            schema.properties[&selected_id].allowed_values,
            PREVIEW_COLLECTION_SELECTIONS
        );
        assert_eq!(PREVIEW_COLLECTION_ITEMS.len(), 3);

        let mounted = gallery.host.snapshot().unwrap().clone();
        assert_eq!(mounted.preview.document.properties.len(), 1);
        assert!(mounted.preview.document.actions.is_empty());
        assert_eq!(
            mounted.preview.document.properties[&selected_id],
            PropertyValue::String("none".into())
        );
        let selection = CompiledPreviewGallery::collection_selection_binding(
            None,
            PropertyBindingContext {
                property: selected_id.clone(),
                stamp: gallery.stamp_lease.clone(),
            },
        );

        assert!(selection.write_to(&mut gallery, Some(Identity::from("second"))));
        let selected = gallery.host.snapshot().unwrap().clone();
        assert_eq!(selected.stamp.generation, mounted.stamp.generation);
        assert_eq!(selected.stamp.revision, mounted.stamp.revision + 1);
        assert_eq!(
            selected.preview.document.properties[&selected_id],
            PropertyValue::String("second".into())
        );

        let before_invalid_selection = selected.clone();
        assert!(!selection.write_to(&mut gallery, Some(Identity::from("unknown"))));
        let after_invalid_selection = gallery.host.snapshot().unwrap();
        assert_eq!(
            after_invalid_selection.stamp,
            before_invalid_selection.stamp
        );
        assert_eq!(
            after_invalid_selection.preview.document,
            before_invalid_selection.preview.document
        );

        gallery.export_descriptor();
        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap().clone();
        assert_eq!(
            imported.preview.document.properties[&selected_id],
            PropertyValue::String("second".into())
        );

        let invalid_selection = gallery
            .descriptor_json
            .replacen(r#""second""#, r#""unknown""#, 1);
        assert_ne!(invalid_selection, gallery.descriptor_json);
        gallery.descriptor_json = invalid_selection;
        gallery.import_descriptor();
        assert!(gallery.status.starts_with("Import rejected:"));
        let after_invalid_import = gallery.host.snapshot().unwrap();
        assert_eq!(after_invalid_import.stamp, imported.stamp);
        assert_eq!(
            after_invalid_import.preview.document,
            imported.preview.document
        );
    }

    #[test]
    fn canvas_specimen_round_trips_only_fixed_composition_choices() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_canvas_specimen();

        let component_id = ComponentId::new("canvas-specimen").unwrap();
        let composition_id = PropertyId::new("composition").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 1);
        assert!(schema.events.is_empty());
        assert_eq!(
            schema.properties[&composition_id].default,
            Some(PropertyValue::String("basic".into()))
        );
        assert_eq!(
            schema.properties[&composition_id].allowed_values,
            PREVIEW_CANVAS_COMPOSITIONS
        );

        let mounted = gallery.host.snapshot().unwrap().clone();
        assert_eq!(mounted.preview.document.properties.len(), 1);
        assert!(mounted.preview.document.actions.is_empty());
        assert_eq!(
            mounted.preview.document.properties[&composition_id],
            PropertyValue::String("basic".into())
        );

        let updated = gallery
            .host
            .binding_feedback(
                &mounted.stamp,
                composition_id.clone(),
                PropertyValue::String("overlap".into()),
            )
            .unwrap();
        assert_eq!(updated.revision, mounted.stamp.revision + 1);
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.document.properties[&composition_id],
            PropertyValue::String("overlap".into())
        );

        gallery.export_descriptor();
        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap().clone();
        assert_eq!(
            imported.preview.document.properties[&composition_id],
            PropertyValue::String("overlap".into())
        );

        let invalid_composition =
            gallery
                .descriptor_json
                .replacen(r#""overlap""#, r#""custom geometry""#, 1);
        assert_ne!(invalid_composition, gallery.descriptor_json);
        gallery.descriptor_json = invalid_composition;
        gallery.import_descriptor();
        assert!(gallery.status.starts_with("Import rejected:"));
        let after_invalid_import = gallery.host.snapshot().unwrap();
        assert_eq!(after_invalid_import.stamp, imported.stamp);
        assert_eq!(
            after_invalid_import.preview.document,
            imported.preview.document
        );
    }

    #[test]
    fn delayed_rerender_changes_state_only_on_completion_or_error() {
        let mut gallery = CompiledPreviewGallery::default();
        let initial_renders = gallery.renders;
        let initial_status = gallery.status.clone();

        assert!(!gallery.finish_delayed_rerender(AsyncState::Loading));
        assert!(!gallery.finish_delayed_rerender(AsyncState::Idle));
        assert_eq!(gallery.renders, initial_renders);
        assert_eq!(gallery.status, initial_status);

        assert!(gallery.finish_delayed_rerender(AsyncState::Success(())));
        assert_eq!(gallery.renders, initial_renders + 1);
        assert_eq!(gallery.status, "Completed delayed parent rerender");

        assert!(gallery.finish_delayed_rerender(AsyncState::Error("timeout".into())));
        assert_eq!(gallery.renders, initial_renders + 1);
        assert_eq!(gallery.status, "Delayed parent rerender failed: timeout");
        assert_eq!(
            preview_inspector_rejection_live_region_value(&gallery.status),
            Some(gallery.status.as_str())
        );
    }

    #[test]
    fn group_specimen_round_trips_bounded_child_labels_without_events() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_group_specimen();
        assert!(
            gallery.status.starts_with("Mounted group-specimen"),
            "{}",
            gallery.status
        );
        let component_id = ComponentId::new("group-specimen").unwrap();
        let first_id = PropertyId::new("first_label").unwrap();
        let second_id = PropertyId::new("second_label").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 2);
        assert!(schema.events.is_empty());
        for id in [&first_id, &second_id] {
            assert_eq!(schema.properties[id].kind, PropertyKind::String);
            assert_eq!(
                schema.properties[id].max_length,
                Some(PREVIEW_TREE_LABEL_MAX_LENGTH)
            );
        }
        let mounted = gallery.host.snapshot().unwrap().clone();
        assert!(mounted.preview.document.actions.is_empty());
        assert_eq!(
            mounted.preview.properties[&first_id],
            PropertyValue::String("First grouped child".into())
        );
        assert_eq!(
            mounted.preview.properties[&second_id],
            PropertyValue::String("Second grouped child".into())
        );
        let mut customized = mounted.preview.document.clone();
        customized
            .properties
            .insert(first_id.clone(), PropertyValue::String("North".into()));
        customized
            .properties
            .insert(second_id.clone(), PropertyValue::String("South".into()));
        gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap();
        assert_eq!(
            imported.preview.properties[&first_id],
            PropertyValue::String("North".into())
        );
        assert_eq!(
            imported.preview.properties[&second_id],
            PropertyValue::String("South".into())
        );
    }
