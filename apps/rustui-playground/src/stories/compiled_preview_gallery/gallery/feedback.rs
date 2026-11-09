use super::*;

pub(in super::super) fn register_feedback_specimens(
    registry: &mut PreviewRegistry<FactoryContext>,
) {
    {
        let empty_state_specimen_schema = ComponentSchema {
            id: ComponentId::new("empty-state-specimen").unwrap(),
            version: 1,
            display_name: "EmptyState".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("title").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("No results".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(PREVIEW_EMPTYSTATE_TITLE_MAX_LENGTH),
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("detail").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Try changing your filters.".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: None,
                        max_length: Some(PREVIEW_EMPTYSTATE_DETAIL_MAX_LENGTH),
                        allowed_values: vec![],
                    },
                ),
            ]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                empty_state_specimen_schema,
                |snapshot, _context: &FactoryContext| {
                    let title =
                        property_string(snapshot, "title").expect("validated EmptyState title");
                    let detail =
                        property_string(snapshot, "detail").expect("validated EmptyState detail");
                    let mut empty_state = EmptyState(title);
                    if let Some(detail) = preview_nonblank_detail(&detail) {
                        empty_state = empty_state.detail(detail.to_owned());
                    }
                    // EmptyState uses size_full(); constrain it to the preview pane so the
                    // inspector keeps its allocated space in the surrounding HStack.
                    AnyView::new(Frame(empty_state).width(320.0).height(180.0))
                },
            )
            .unwrap();

        let error_state_specimen_schema = ComponentSchema {
            id: ComponentId::new("error-state-specimen").unwrap(),
            version: 1,
            display_name: "ErrorState".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("title").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Could not load".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(PREVIEW_ERRORSTATE_TITLE_MAX_LENGTH),
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("detail").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String(
                            "Check your connection and try again.".into(),
                        )),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: None,
                        max_length: Some(PREVIEW_ERRORSTATE_DETAIL_MAX_LENGTH),
                        allowed_values: vec![],
                    },
                ),
            ]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                error_state_specimen_schema,
                |snapshot, _context: &FactoryContext| {
                    let title =
                        property_string(snapshot, "title").expect("validated ErrorState title");
                    let detail =
                        property_string(snapshot, "detail").expect("validated ErrorState detail");
                    let mut error_state = ErrorState(title);
                    if let Some(detail) = preview_nonblank_detail(&detail) {
                        error_state = error_state.detail(detail.to_owned());
                    }
                    AnyView::new(error_state)
                },
            )
            .unwrap();

        let disclosure_specimen_schema = ComponentSchema {
            id: ComponentId::new("disclosure-specimen").unwrap(),
            version: 1,
            display_name: "Disclosure".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("label").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("More information".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(PREVIEW_DISCLOSURE_LABEL_MAX_LENGTH),
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("open").unwrap(),
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
            events: BTreeMap::from([(
                EventId::new("toggle").unwrap(),
                EventSchema {
                    display_name: "Toggle disclosure".into(),
                    payload: BTreeMap::new(),
                },
            )]),
        };
        registry
            .register(
                disclosure_specimen_schema,
                |snapshot, context: &FactoryContext| {
                    let label =
                        property_string(snapshot, "label").expect("validated Disclosure label");
                    let open = property_boolean(snapshot, "open")
                        .expect("validated Disclosure open state");
                    let open = Binding::new_with(
                        open,
                        PropertyBindingContext {
                            property: PropertyId::new("open").unwrap(),
                            stamp: context.stamp_lease.clone(),
                        },
                        CompiledPreviewGallery::read_disclosure_open,
                        CompiledPreviewGallery::write_disclosure_open,
                    );
                    AnyView::new(Disclosure(
                        format!("compiled-preview-disclosure-{}", snapshot.stamp.generation),
                        label,
                        open,
                        Text("The disclosure content is compiled locally."),
                    ))
                },
            )
            .unwrap();

        let toolbar_specimen_schema = ComponentSchema {
            id: ComponentId::new("toolbar-specimen").unwrap(),
            version: 1,
            display_name: "Toolbar".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("primary_label").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Save".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(PREVIEW_TOOLBAR_LABEL_MAX_LENGTH),
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("secondary_label").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Cancel".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(PREVIEW_TOOLBAR_LABEL_MAX_LENGTH),
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("secondary_disabled").unwrap(),
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
            events: BTreeMap::from([
                (
                    EventId::new("primary").unwrap(),
                    EventSchema {
                        display_name: "Primary action".into(),
                        payload: BTreeMap::new(),
                    },
                ),
                (
                    EventId::new("secondary").unwrap(),
                    EventSchema {
                        display_name: "Secondary action".into(),
                        payload: BTreeMap::new(),
                    },
                ),
            ]),
        };
        registry
            .register(
                toolbar_specimen_schema,
                |snapshot, context: &FactoryContext| {
                    let primary_label = property_string(snapshot, "primary_label")
                        .expect("validated Toolbar primary label");
                    let secondary_label = property_string(snapshot, "secondary_label")
                        .expect("validated Toolbar secondary label");
                    let secondary_disabled = property_boolean(snapshot, "secondary_disabled")
                        .expect("validated Toolbar disabled state");
                    let items = vec![
                        ToolbarItem::action(
                            "preview-primary",
                            primary_label,
                            context.owner.action_with(
                                snapshot.signal(EventId::new("primary").unwrap()),
                                CompiledPreviewGallery::toolbar_primary,
                            ),
                        ),
                        ToolbarItem::action(
                            "preview-secondary",
                            secondary_label,
                            context.owner.action_with(
                                snapshot.signal(EventId::new("secondary").unwrap()),
                                CompiledPreviewGallery::toolbar_secondary,
                            ),
                        )
                        .disabled(secondary_disabled),
                    ];
                    AnyView::new(Toolbar(
                        format!("compiled-preview-toolbar-{}", snapshot.stamp.generation),
                        items,
                    ))
                },
            )
            .unwrap();

        let image_specimen_schema = ComponentSchema {
            id: ComponentId::new("image-specimen").unwrap(),
            version: 1,
            display_name: "Image".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("accessibility_label").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Purple and cyan sample image".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(PREVIEW_IMAGE_LABEL_MAX_LENGTH),
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("fit").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("contain".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: None,
                        max_length: None,
                        allowed_values: PREVIEW_IMAGE_FITS
                            .iter()
                            .map(|fit| (*fit).into())
                            .collect(),
                    },
                ),
            ]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                image_specimen_schema,
                |snapshot, _context: &FactoryContext| {
                    let accessibility_label = property_string(snapshot, "accessibility_label")
                        .expect("validated Image accessibility label");
                    let fit = match property_string(snapshot, "fit")
                        .expect("validated Image fit")
                        .as_str()
                    {
                        "contain" => ImageFit::Contain,
                        "cover" => ImageFit::Cover,
                        "fill" => ImageFit::Fill,
                        "scale-down" => ImageFit::ScaleDown,
                        "none" => ImageFit::None,
                        _ => unreachable!("Image fit is schema-validated"),
                    };
                    #[cfg(target_family = "wasm")]
                    let image = Image("assets/playground-sample.png");
                    #[cfg(not(target_family = "wasm"))]
                    let image = Image(crate::playground_resource_path(
                        "assets/playground-sample.png",
                    ));
                    AnyView::new(
                        image
                            .size(128.0, 96.0)
                            .fit(fit)
                            .id(format!(
                                "compiled-preview-image-{}",
                                snapshot.stamp.generation
                            ))
                            .accessibility_label(accessibility_label),
                    )
                },
            )
            .unwrap();

        let breadcrumb_specimen_schema = ComponentSchema {
            id: ComponentId::new("breadcrumb-specimen").unwrap(),
            version: 1,
            display_name: "Breadcrumb".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("parent_label").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Components".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(PREVIEW_BREADCRUMB_LABEL_MAX_LENGTH),
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("current_label").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Image".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(PREVIEW_BREADCRUMB_LABEL_MAX_LENGTH),
                        allowed_values: vec![],
                    },
                ),
            ]),
            events: BTreeMap::from([(
                EventId::new("parent").unwrap(),
                EventSchema {
                    display_name: "Navigate to parent".into(),
                    payload: BTreeMap::new(),
                },
            )]),
        };
        registry
            .register(
                breadcrumb_specimen_schema,
                |snapshot, context: &FactoryContext| {
                    let parent_label = property_string(snapshot, "parent_label")
                        .expect("validated Breadcrumb parent label");
                    let current_label = property_string(snapshot, "current_label")
                        .expect("validated Breadcrumb current label");
                    AnyView::new(
                        Breadcrumb([
                            BreadcrumbItem::new(parent_label).action(context.owner.action_with(
                                snapshot.signal(EventId::new("parent").unwrap()),
                                CompiledPreviewGallery::navigate_breadcrumb_parent,
                            )),
                            BreadcrumbItem::new(current_label),
                        ])
                        .id(format!(
                            "compiled-preview-breadcrumb-{}",
                            snapshot.stamp.generation
                        ))
                        .accessibility_label("Preview location"),
                    )
                },
            )
            .unwrap();

        let context_menu_specimen_schema = ComponentSchema {
            id: ComponentId::new("context-menu-specimen").unwrap(),
            version: 1,
            display_name: "ContextMenu".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("target_label").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Preview target".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(PREVIEW_CONTEXT_MENU_LABEL_MAX_LENGTH),
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("action_label").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Open preview".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(PREVIEW_CONTEXT_MENU_LABEL_MAX_LENGTH),
                        allowed_values: vec![],
                    },
                ),
            ]),
            events: BTreeMap::from([(
                EventId::new("activate").unwrap(),
                EventSchema {
                    display_name: "Activate context menu item".into(),
                    payload: BTreeMap::new(),
                },
            )]),
        };
        registry
            .register(
                context_menu_specimen_schema,
                |snapshot, context: &FactoryContext| {
                    let target_label = property_string(snapshot, "target_label")
                        .expect("validated ContextMenu target label");
                    let action_label = property_string(snapshot, "action_label")
                        .expect("validated ContextMenu action label");
                    AnyView::new(ContextMenu(
                        format!(
                            "compiled-preview-context-menu-{}",
                            snapshot.stamp.generation
                        ),
                        Text(target_label),
                        [MenuItem::action(
                            "open-preview",
                            action_label,
                            context.owner.action_with(
                                snapshot.signal(EventId::new("activate").unwrap()),
                                CompiledPreviewGallery::activate_context_menu_item,
                            ),
                        )],
                    ))
                },
            )
            .unwrap();

        let alert_specimen_schema = ComponentSchema {
            id: ComponentId::new("alert-specimen").unwrap(),
            version: 1,
            display_name: "Alert".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("message").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Connection lost".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(200),
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("visible").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::Boolean,
                        default: Some(PropertyValue::Boolean(true)),
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
            events: BTreeMap::from([(
                EventId::new("dismiss").unwrap(),
                EventSchema {
                    display_name: "Dismiss alert".into(),
                    payload: BTreeMap::new(),
                },
            )]),
        };
        registry
            .register(
                alert_specimen_schema,
                |snapshot, context: &FactoryContext| {
                    let message =
                        property_string(snapshot, "message").expect("validated alert message");
                    let visible =
                        property_boolean(snapshot, "visible").expect("validated alert visibility");
                    let dismiss = context.owner.action_with(
                        snapshot.signal(EventId::new("dismiss").unwrap()),
                        CompiledPreviewGallery::dismiss_alert,
                    );
                    let mut preview = VStack::new().spacing(8.0).child(
                        Alert(
                            format!("compiled-preview-alert-{}", snapshot.stamp.generation),
                            message,
                            context.checked.clone(),
                        )
                        .title("Connection issue")
                        .variant(AlertVariant::Warning),
                    );
                    if visible {
                        preview = preview.child(Button("Dismiss alert", dismiss));
                    }
                    AnyView::new(preview)
                },
            )
            .unwrap();

        let toast_specimen_schema = ComponentSchema {
            id: ComponentId::new("toast-specimen").unwrap(),
            version: 1,
            display_name: "Toast".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("message").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Changes saved".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(PREVIEW_TOAST_MESSAGE_MAX_LENGTH),
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("title").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: None,
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(PREVIEW_TOAST_TITLE_MAX_LENGTH),
                        allowed_values: vec![],
                    },
                ),
            ]),
            events: BTreeMap::from([(
                EventId::new("show").unwrap(),
                EventSchema {
                    display_name: "Show toast".into(),
                    payload: BTreeMap::new(),
                },
            )]),
        };
        registry
            .register(
                toast_specimen_schema,
                |snapshot, context: &FactoryContext| {
                    let message =
                        property_string(snapshot, "message").expect("validated Toast message");
                    let title = property_string(snapshot, "title");
                    let mut toast = Toast(message).kind(ToastKind::Info).autohide(true);
                    if let Some(title) = title {
                        toast = toast.title(title);
                    }
                    let show_toast = toast.show();
                    let signal_action = context.owner.action_with(
                        snapshot.signal(EventId::new("show").unwrap()),
                        CompiledPreviewGallery::show_toast,
                    );
                    AnyView::new(Button("Show preview toast", signal_action.then(show_toast)))
                },
            )
            .unwrap();

        let sheet_specimen_schema = ComponentSchema {
            id: ComponentId::new("sheet-specimen").unwrap(),
            version: 1,
            display_name: "Sheet".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("trigger_label").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Open details".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(PREVIEW_SHEET_TRIGGER_MAX_LENGTH),
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("title").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Details".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(PREVIEW_SHEET_TITLE_MAX_LENGTH),
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("body_text").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Preview sheet content".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(PREVIEW_SHEET_BODY_MAX_LENGTH),
                        allowed_values: vec![],
                    },
                ),
            ]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                sheet_specimen_schema,
                |snapshot, _context: &FactoryContext| {
                    let trigger_label = property_string(snapshot, "trigger_label")
                        .expect("validated Sheet trigger label");
                    let title = property_string(snapshot, "title").expect("validated Sheet title");
                    let body_text =
                        property_string(snapshot, "body_text").expect("validated Sheet body text");
                    AnyView::new(
                        Sheet(
                            format!("compiled-preview-sheet-{}", snapshot.stamp.generation),
                            trigger_label,
                        )
                        .title(title)
                        .content(Text(body_text)),
                    )
                },
            )
            .unwrap();

        let drawer_specimen_schema = ComponentSchema {
            id: ComponentId::new("drawer-specimen").unwrap(),
            version: 1,
            display_name: "Drawer".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("trigger_label").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Open details".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(PREVIEW_SHEET_TRIGGER_MAX_LENGTH),
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("title").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Details".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(PREVIEW_SHEET_TITLE_MAX_LENGTH),
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("body_text").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Preview sheet content".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(PREVIEW_SHEET_BODY_MAX_LENGTH),
                        allowed_values: vec![],
                    },
                ),
            ]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                drawer_specimen_schema,
                |snapshot, _context: &FactoryContext| {
                    let trigger_label = property_string(snapshot, "trigger_label")
                        .expect("validated Drawer trigger label");
                    let title = property_string(snapshot, "title").expect("validated Drawer title");
                    let body_text =
                        property_string(snapshot, "body_text").expect("validated Drawer body text");
                    AnyView::new(super::super::super::drawer_gallery::specimen(
                        format!("compiled-preview-drawer-{}", snapshot.stamp.generation),
                        trigger_label,
                        title,
                        body_text,
                    ))
                },
            )
            .unwrap();

        let sidebar_specimen_schema = ComponentSchema {
            id: ComponentId::new("sidebar-specimen").unwrap(),
            version: 1,
            display_name: "Sidebar".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("home_label").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Home".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(PREVIEW_SIDEBAR_LABEL_MAX_LENGTH),
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("settings_label").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Settings".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(PREVIEW_SIDEBAR_LABEL_MAX_LENGTH),
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("selected_id").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("home".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: None,
                        max_length: None,
                        allowed_values: vec!["home".into(), "settings".into()],
                    },
                ),
            ]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                sidebar_specimen_schema,
                |snapshot, context: &FactoryContext| {
                    let home_label = property_string(snapshot, "home_label")
                        .expect("validated Sidebar Home label");
                    let settings_label = property_string(snapshot, "settings_label")
                        .expect("validated Sidebar Settings label");
                    let selected_id = property_string(snapshot, "selected_id")
                        .expect("validated Sidebar selection");
                    let selection = CompiledPreviewGallery::string_selection_binding(
                        Some(selected_id),
                        PropertyBindingContext {
                            property: PropertyId::new("selected_id").unwrap(),
                            stamp: context.stamp_lease.clone(),
                        },
                    );
                    AnyView::new(
                        Frame(Sidebar(
                            format!("compiled-preview-sidebar-{}", snapshot.stamp.generation),
                            [
                                SidebarItem::new("home", home_label),
                                SidebarItem::new("settings", settings_label),
                            ],
                            selection,
                        ))
                        .width(240.0)
                        .height(240.0)
                        .alignment(HorizontalAlignment::Start, VerticalAlignment::Stretch),
                    )
                },
            )
            .unwrap();
    }
}
