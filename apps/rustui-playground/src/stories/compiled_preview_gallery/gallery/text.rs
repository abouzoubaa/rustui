use super::*;

pub(in super::super) fn register_text_specimens(registry: &mut PreviewRegistry<FactoryContext>) {
    {
        let text_field_specimen_schema = ComponentSchema {
            id: ComponentId::new("text-field-specimen").unwrap(),
            version: 1,
            display_name: "TextField".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("placeholder").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Preview text input".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: None,
                        max_length: Some(200),
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("value").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Preview text input".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: None,
                        max_length: Some(200),
                        allowed_values: vec![],
                    },
                ),
            ]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                text_field_specimen_schema,
                |snapshot, context: &FactoryContext| {
                    let placeholder = property_string(snapshot, "placeholder")
                        .expect("validated text field placeholder");
                    AnyView::new(
                        TextField(placeholder.clone(), context.text.clone())
                            .accessibility_label("Preview text input")
                            .id(format!(
                                "compiled-preview-text-field-specimen-{}-{placeholder}",
                                snapshot.stamp.generation,
                            )),
                    )
                },
            )
            .unwrap();
        let token_field_specimen_schema = ComponentSchema {
            id: ComponentId::new("token-field-specimen").unwrap(),
            version: 1,
            display_name: "TokenField".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("placeholder").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Add recipients".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: None,
                        max_length: Some(80),
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("tokens_json").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String(
                            serde_json::to_string(&PREVIEW_TOKEN_FIELD_DEFAULTS).unwrap(),
                        )),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: None,
                        max_length: Some(PREVIEW_TOKEN_VALUES_MAX_LENGTH),
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("draft").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Engineering".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: None,
                        max_length: Some(80),
                        allowed_values: vec![],
                    },
                ),
            ]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                token_field_specimen_schema,
                |snapshot, context: &FactoryContext| {
                    let placeholder = property_string(snapshot, "placeholder")
                        .expect("validated TokenField placeholder");
                    AnyView::new(
                        TokenField(
                            placeholder,
                            context.token_values.clone(),
                            context.text.clone(),
                        )
                        .id(format!(
                            "compiled-preview-token-field-{}",
                            snapshot.stamp.generation
                        )),
                    )
                },
            )
            .unwrap();
        let search_field_specimen_schema = ComponentSchema {
            id: ComponentId::new("search-field-specimen").unwrap(),
            version: 1,
            display_name: "SearchField".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("placeholder").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Search RustUI".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: None,
                        max_length: Some(200),
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("query").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Search RustUI".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: None,
                        max_length: Some(200),
                        allowed_values: vec![],
                    },
                ),
            ]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                search_field_specimen_schema,
                |snapshot, context: &FactoryContext| {
                    let placeholder = property_string(snapshot, "placeholder")
                        .expect("validated search field placeholder");
                    AnyView::new(
                        SearchField(placeholder.clone(), context.text.clone())
                            .accessibility_label("Search RustUI")
                            .id(format!(
                                "compiled-preview-search-field-specimen-{}-{placeholder}",
                                snapshot.stamp.generation,
                            )),
                    )
                },
            )
            .unwrap();
        let text_editor_specimen_schema = ComponentSchema {
            id: ComponentId::new("text-editor-specimen").unwrap(),
            version: 1,
            display_name: "TextEditor".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("placeholder").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Write a note".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: None,
                        max_length: Some(200),
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("value").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("First line\nSecond line".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: None,
                        max_length: Some(2_000),
                        allowed_values: vec![],
                    },
                ),
            ]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                text_editor_specimen_schema,
                |snapshot, context: &FactoryContext| {
                    let placeholder = property_string(snapshot, "placeholder")
                        .expect("validated text editor placeholder");
                    AnyView::new(
                        TextEditor(placeholder, context.text.clone())
                            .accessibility_label("Preview notes")
                            .id(format!(
                                "compiled-preview-text-editor-specimen-{}",
                                snapshot.stamp.generation
                            )),
                    )
                },
            )
            .unwrap();
        let code_editor_specimen_schema = ComponentSchema {
            id: ComponentId::new("code-editor-specimen").unwrap(),
            version: 1,
            display_name: "CodeEditor".into(),
            properties: BTreeMap::from([(
                PropertyId::new("value").unwrap(),
                PropertySchema {
                    kind: PropertyKind::String,
                    default: Some(PropertyValue::String("Plain text preview".into())),
                    required: false,
                    editable: true,
                    minimum: None,
                    maximum: None,
                    min_length: None,
                    max_length: Some(2_000),
                    allowed_values: vec![],
                },
            )]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                code_editor_specimen_schema,
                |_snapshot, context: &FactoryContext| {
                    AnyView::new(
                        CodeEditor("compiled-preview-code-editor", context.text.clone())
                            .language("text")
                            .readonly(true)
                            .accessibility_label("Preview plain text"),
                    )
                },
            )
            .unwrap();

        let secure_field_specimen_schema = ComponentSchema {
            id: ComponentId::new("secure-field-specimen").unwrap(),
            version: 1,
            display_name: "SecureField".into(),
            properties: BTreeMap::from([(
                PropertyId::new("placeholder").unwrap(),
                PropertySchema {
                    kind: PropertyKind::String,
                    default: Some(PropertyValue::String("Enter a password".into())),
                    required: false,
                    editable: true,
                    minimum: None,
                    maximum: None,
                    min_length: Some(1),
                    max_length: Some(120),
                    allowed_values: vec![],
                },
            )]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                secure_field_specimen_schema,
                |snapshot, context: &FactoryContext| {
                    let placeholder = property_string(snapshot, "placeholder")
                        .expect("validated secure field placeholder");
                    AnyView::new(
                        SecureField(placeholder, context.secure_value.clone())
                            .accessibility_label("Preview secure input")
                            .localized_password_visibility_accessibility_label(
                                "secure_field.toggle_visibility",
                            )
                            .id(format!(
                                "compiled-preview-secure-field-specimen-{}",
                                snapshot.stamp.generation
                            )),
                    )
                },
            )
            .unwrap();
        let menu_specimen_schema = ComponentSchema {
            id: ComponentId::new("menu-specimen").unwrap(),
            version: 1,
            display_name: "Menu".into(),
            properties: BTreeMap::from([(
                PropertyId::new("trigger_label").unwrap(),
                PropertySchema {
                    kind: PropertyKind::String,
                    default: Some(PropertyValue::String("Preview menu".into())),
                    required: false,
                    editable: true,
                    minimum: None,
                    maximum: None,
                    min_length: Some(1),
                    max_length: Some(80),
                    allowed_values: vec![],
                },
            )]),
            events: BTreeMap::from([(
                EventId::new("select").unwrap(),
                EventSchema {
                    display_name: "Select menu action".into(),
                    payload: BTreeMap::new(),
                },
            )]),
        };
        registry
            .register(
                menu_specimen_schema,
                |snapshot, context: &FactoryContext| {
                    let trigger_label = property_string(snapshot, "trigger_label")
                        .expect("validated menu trigger label");
                    let action = context.owner.action_with(
                        snapshot.signal(EventId::new("select").unwrap()),
                        CompiledPreviewGallery::activate,
                    );
                    AnyView::new(
                        Menu(
                            trigger_label,
                            [MenuItem::action("select", "Run preview action", action)],
                        )
                        .id(format!(
                            "compiled-preview-menu-specimen-{}",
                            snapshot.stamp.generation
                        )),
                    )
                },
            )
            .unwrap();
        let link_specimen_schema = ComponentSchema {
            id: ComponentId::new("link-specimen").unwrap(),
            version: 1,
            display_name: "Link".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("label").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("RustUI documentation".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(80),
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("url").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("https://example.com/docs".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(2048),
                        allowed_values: vec![],
                    },
                ),
            ]),
            events: BTreeMap::from([(
                EventId::new("activate").unwrap(),
                EventSchema {
                    display_name: "Activate".into(),
                    payload: BTreeMap::new(),
                },
            )]),
        };
        registry
            .register(
                link_specimen_schema,
                |snapshot, context: &FactoryContext| {
                    let label = property_string(snapshot, "label").expect("validated link label");
                    let url = property_string(snapshot, "url").expect("validated link URL");
                    let action = context.owner.action_with(
                        snapshot.signal(EventId::new("activate").unwrap()),
                        CompiledPreviewGallery::activate,
                    );
                    AnyView::new(
                        Link(label, url)
                            .open_url_on_activate(false)
                            .on_activate(action)
                            .id(format!(
                                "compiled-preview-link-specimen-{}",
                                snapshot.stamp.generation
                            )),
                    )
                },
            )
            .unwrap();
        let dialog_specimen_schema = ComponentSchema {
            id: ComponentId::new("dialog-specimen").unwrap(),
            version: 1,
            display_name: "Dialog".into(),
            properties: BTreeMap::from([(
                PropertyId::new("trigger_label").unwrap(),
                PropertySchema {
                    kind: PropertyKind::String,
                    default: Some(PropertyValue::String("Open preview dialog".into())),
                    required: false,
                    editable: true,
                    minimum: None,
                    maximum: None,
                    min_length: Some(1),
                    max_length: Some(80),
                    allowed_values: vec![],
                },
            )]),
            events: BTreeMap::from([
                (
                    EventId::new("confirm").unwrap(),
                    EventSchema {
                        display_name: "Confirm dialog".into(),
                        payload: BTreeMap::new(),
                    },
                ),
                (
                    EventId::new("cancel").unwrap(),
                    EventSchema {
                        display_name: "Cancel dialog".into(),
                        payload: BTreeMap::new(),
                    },
                ),
            ]),
        };
        registry
            .register(
                dialog_specimen_schema,
                |snapshot, context: &FactoryContext| {
                    let trigger_label = property_string(snapshot, "trigger_label")
                        .expect("validated dialog trigger label");
                    let confirm = context.owner.action_with(
                        snapshot.signal(EventId::new("confirm").unwrap()),
                        CompiledPreviewGallery::activate,
                    );
                    let cancel = context.owner.action_with(
                        snapshot.signal(EventId::new("cancel").unwrap()),
                        CompiledPreviewGallery::activate,
                    );
                    AnyView::new(
                        Dialog(
                            format!("compiled-preview-dialog-{}", snapshot.stamp.generation),
                            trigger_label,
                        )
                        .title("Preview dialog")
                        .content(Text("This dialog is rendered by a compiled local factory."))
                        .confirm_label("Confirm preview")
                        .cancel_label("Cancel preview")
                        .on_confirm(confirm)
                        .on_cancel(cancel),
                    )
                },
            )
            .unwrap();
        let avatar_specimen_schema = ComponentSchema {
            id: ComponentId::new("avatar-specimen").unwrap(),
            version: 1,
            display_name: "Avatar".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("name").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Ada Lovelace".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(80),
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("diameter").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::Number,
                        default: Some(PropertyValue::Number(40.0)),
                        required: false,
                        editable: true,
                        minimum: Some(24.0),
                        maximum: Some(64.0),
                        min_length: None,
                        max_length: None,
                        allowed_values: vec![],
                    },
                ),
            ]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                avatar_specimen_schema,
                |snapshot, _context: &FactoryContext| {
                    let name = property_string(snapshot, "name").expect("validated avatar name");
                    let diameter = property_number(snapshot, "diameter")
                        .expect("validated avatar diameter")
                        as f32;
                    AnyView::new(Avatar(name).size(diameter).id(format!(
                        "compiled-preview-avatar-{}",
                        snapshot.stamp.generation
                    )))
                },
            )
            .unwrap();
        let badge_specimen_schema = ComponentSchema {
            id: ComponentId::new("badge-specimen").unwrap(),
            version: 1,
            display_name: "Badge".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("label").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Inbox".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(80),
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("count").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::Integer,
                        default: Some(PropertyValue::Integer(7)),
                        required: false,
                        editable: true,
                        minimum: Some(0.0),
                        maximum: Some(999.0),
                        min_length: None,
                        max_length: None,
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("dot").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::Boolean,
                        default: Some(PropertyValue::Boolean(false)),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: None,
                        max_length: None,
                        allowed_values: vec![],
                    },
                ),
            ]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                badge_specimen_schema,
                |snapshot, _context: &FactoryContext| {
                    let label = property_string(snapshot, "label").expect("validated badge label");
                    let count = property_integer(snapshot, "count").expect("validated badge count")
                        as usize;
                    let badge = if property_boolean(snapshot, "dot").expect("validated badge mode")
                    {
                        Badge(Text(label)).dot()
                    } else {
                        Badge(Text(label)).count(count)
                    };
                    AnyView::new(badge.id(format!(
                        "compiled-preview-badge-{}",
                        snapshot.stamp.generation
                    )))
                },
            )
            .unwrap();

        let label_specimen_schema = ComponentSchema {
            id: ComponentId::new("label-specimen").unwrap(),
            version: 1,
            display_name: "Label".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("text").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Search".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(80),
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("secondary").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("⌘K".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(0),
                        max_length: Some(80),
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("vibrant").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::Boolean,
                        default: Some(PropertyValue::Boolean(false)),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: None,
                        max_length: None,
                        allowed_values: vec![],
                    },
                ),
            ]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                label_specimen_schema,
                |snapshot, _context: &FactoryContext| {
                    let text = property_string(snapshot, "text").expect("validated label text");
                    let secondary = property_string(snapshot, "secondary")
                        .expect("validated secondary label text");
                    let mut label = if text.trim().is_empty() {
                        Label("Label")
                    } else {
                        Label(text)
                    };
                    if !secondary.trim().is_empty() {
                        label = label.secondary(secondary);
                    }
                    if property_boolean(snapshot, "vibrant").expect("validated label style") {
                        label = label.vibrant();
                    }
                    AnyView::new(label)
                },
            )
            .unwrap();

        let tooltip_specimen_schema = ComponentSchema {
            id: ComponentId::new("tooltip-specimen").unwrap(),
            version: 1,
            display_name: "Tooltip".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("text").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("More information".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(120),
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("label").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Help".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(80),
                        allowed_values: vec![],
                    },
                ),
            ]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                tooltip_specimen_schema,
                |snapshot, _context: &FactoryContext| {
                    let text = property_string(snapshot, "text").expect("validated tooltip text");
                    let label =
                        property_string(snapshot, "label").expect("validated tooltip child label");
                    AnyView::new(Tooltip(text, Text(label)).id(format!(
                        "compiled-preview-tooltip-{}",
                        snapshot.stamp.generation
                    )))
                },
            )
            .unwrap();

        let spinner_specimen_schema = ComponentSchema {
            id: ComponentId::new("spinner-specimen").unwrap(),
            version: 1,
            display_name: "Spinner".into(),
            properties: BTreeMap::from([(
                PropertyId::new("accessibility_label").unwrap(),
                PropertySchema {
                    kind: PropertyKind::String,
                    default: Some(PropertyValue::String("Loading".into())),
                    required: false,
                    editable: true,
                    minimum: None,
                    maximum: None,
                    min_length: Some(1),
                    max_length: Some(80),
                    allowed_values: vec![],
                },
            )]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                spinner_specimen_schema,
                |snapshot, _context: &FactoryContext| {
                    let accessibility_label = property_string(snapshot, "accessibility_label")
                        .expect("validated spinner accessible name");
                    let accessibility_label = if accessibility_label.trim().is_empty() {
                        "Loading".to_owned()
                    } else {
                        accessibility_label
                    };
                    AnyView::new(
                        Spinner()
                            .accessibility_label(accessibility_label)
                            .id(format!(
                                "compiled-preview-spinner-{}",
                                snapshot.stamp.generation
                            )),
                    )
                },
            )
            .unwrap();

        let divider_specimen_schema = ComponentSchema {
            id: ComponentId::new("divider-specimen").unwrap(),
            version: 1,
            display_name: "Divider".into(),
            properties: BTreeMap::from([(
                PropertyId::new("vertical").unwrap(),
                PropertySchema {
                    kind: PropertyKind::Boolean,
                    default: Some(PropertyValue::Boolean(false)),
                    required: false,
                    editable: true,
                    minimum: None,
                    maximum: None,
                    min_length: None,
                    max_length: None,
                    allowed_values: vec![],
                },
            )]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                divider_specimen_schema,
                |snapshot, _context: &FactoryContext| {
                    if property_boolean(snapshot, "vertical")
                        .expect("validated divider orientation")
                    {
                        AnyView::new(
                            HStack::new()
                                .spacing(8.0)
                                .child(Text("Left"))
                                .child(Divider().vertical())
                                .child(Text("Right")),
                        )
                    } else {
                        AnyView::new(
                            VStack::new()
                                .spacing(4.0)
                                .child(Text("Above"))
                                .child(Divider())
                                .child(Text("Below")),
                        )
                    }
                },
            )
            .unwrap();

        let skeleton_specimen_schema = ComponentSchema {
            id: ComponentId::new("skeleton-specimen").unwrap(),
            version: 1,
            display_name: "Skeleton".into(),
            properties: BTreeMap::from([(
                PropertyId::new("secondary").unwrap(),
                PropertySchema {
                    kind: PropertyKind::Boolean,
                    default: Some(PropertyValue::Boolean(false)),
                    required: false,
                    editable: true,
                    minimum: None,
                    maximum: None,
                    min_length: None,
                    max_length: None,
                    allowed_values: vec![],
                },
            )]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                skeleton_specimen_schema,
                |snapshot, _context: &FactoryContext| {
                    let skeleton = Skeleton()
                        .size(160.0, 20.0)
                        .corner_radius(6.0)
                        .accessibility_label("Loading content")
                        .id(format!(
                            "compiled-preview-skeleton-{}",
                            snapshot.stamp.generation
                        ));
                    let skeleton = if property_boolean(snapshot, "secondary")
                        .expect("validated skeleton color mode")
                    {
                        skeleton.secondary()
                    } else {
                        skeleton
                    };
                    AnyView::new(skeleton)
                },
            )
            .unwrap();

        let icon_specimen_schema = ComponentSchema {
            id: ComponentId::new("icon-specimen").unwrap(),
            version: 1,
            display_name: "Icon".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("accessibility_label").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Search".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(80),
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("size").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::Number,
                        default: Some(PropertyValue::Number(24.0)),
                        required: false,
                        editable: true,
                        minimum: Some(8.0),
                        maximum: Some(64.0),
                        min_length: None,
                        max_length: None,
                        allowed_values: vec![],
                    },
                ),
            ]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                icon_specimen_schema,
                |snapshot, _context: &FactoryContext| {
                    let accessibility_label = property_string(snapshot, "accessibility_label")
                        .expect("validated icon accessible name");
                    let size =
                        property_number(snapshot, "size").expect("validated icon size") as f32;
                    let accessibility_label = if accessibility_label.trim().is_empty() {
                        "Search".to_owned()
                    } else {
                        accessibility_label
                    };
                    AnyView::new(
                        Icon(IconName::Search)
                            .size(size)
                            .accessibility_label(accessibility_label)
                            .id(format!(
                                "compiled-preview-icon-{}",
                                snapshot.stamp.generation
                            )),
                    )
                },
            )
            .unwrap();
    }
}
