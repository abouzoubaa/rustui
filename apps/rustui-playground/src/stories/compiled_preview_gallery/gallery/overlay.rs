pub(super) fn register_overlay_specimens(registry: &mut PreviewRegistry<FactoryContext>) {
    {
        let popover_specimen_schema = ComponentSchema {
            id: ComponentId::new("popover-specimen").unwrap(),
            version: 1,
            display_name: "Popover".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("trigger_label").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Preview details".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(PREVIEW_POPOVER_TRIGGER_MAX_LENGTH),
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
            events: BTreeMap::new(),
        };
        registry
            .register(
                popover_specimen_schema,
                |snapshot, context: &FactoryContext| {
                    let trigger_label = property_string(snapshot, "trigger_label")
                        .expect("validated Popover trigger label");
                    AnyView::new(Popover(
                        format!("compiled-preview-popover-{}", snapshot.stamp.generation),
                        trigger_label,
                        context.checked.clone(),
                        Text("Popover preview content"),
                    ))
                },
            )
            .unwrap();

        let group_specimen_schema = ComponentSchema {
            id: ComponentId::new("group-specimen").unwrap(),
            version: 1,
            display_name: "Group".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("first_label").unwrap(),
                    tree_label_schema("First grouped child"),
                ),
                (
                    PropertyId::new("second_label").unwrap(),
                    tree_label_schema("Second grouped child"),
                ),
            ]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                group_specimen_schema,
                |snapshot, _context: &FactoryContext| {
                    let first = property_string(snapshot, "first_label")
                        .expect("validated Group first label");
                    let second = property_string(snapshot, "second_label")
                        .expect("validated Group second label");
                    AnyView::new(
                        VStack::new().group(
                            rustui::layout::Group::new()
                                .child(Text(first))
                                .child(Text(second)),
                        ),
                    )
                },
            )
            .unwrap();

        let animated_opacity_specimen_schema = ComponentSchema {
            id: ComponentId::new("animated-opacity-specimen").unwrap(),
            version: 1,
            display_name: "AnimatedOpacity".into(),
            properties: [("from", 0.0), ("to", 1.0)]
                .into_iter()
                .map(|(name, default)| {
                    (
                        PropertyId::new(name).unwrap(),
                        PropertySchema {
                            kind: PropertyKind::Number,
                            default: Some(PropertyValue::Number(default)),
                            required: false,
                            editable: true,
                            minimum: Some(0.0),
                            maximum: Some(1.0),
                            min_length: None,
                            max_length: None,
                            allowed_values: vec![],
                        },
                    )
                })
                .collect(),
            events: BTreeMap::new(),
        };
        registry
            .register(
                animated_opacity_specimen_schema,
                |snapshot, _context: &FactoryContext| {
                    let from =
                        property_number(snapshot, "from").expect("validated opacity start") as f32;
                    let to = property_number(snapshot, "to").expect("validated opacity end") as f32;
                    AnyView::new(
                        rustui::modifiers::opacity::AnimatedOpacity::new(
                            format!(
                                "compiled-preview-animated-opacity-{}",
                                snapshot.stamp.generation
                            ),
                            Text("Animated opacity preview"),
                            from,
                            to,
                            Duration::from_millis(500),
                        )
                        .expect("validated endpoints and nonzero duration"),
                    )
                },
            )
            .unwrap();

        let spring_opacity_specimen_schema = ComponentSchema {
            id: ComponentId::new("spring-opacity-specimen").unwrap(),
            version: 1,
            display_name: "SpringOpacity".into(),
            properties: BTreeMap::from([(
                PropertyId::new("target").unwrap(),
                PropertySchema {
                    kind: PropertyKind::Number,
                    default: Some(PropertyValue::Number(0.15)),
                    required: false,
                    editable: true,
                    minimum: Some(0.0),
                    maximum: Some(1.0),
                    min_length: None,
                    max_length: None,
                    allowed_values: vec![],
                },
            )]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                spring_opacity_specimen_schema,
                |snapshot, _context: &FactoryContext| {
                    let target = property_number(snapshot, "target")
                        .expect("validated spring opacity target")
                        as f32;
                    AnyView::new(
                        rustui::animation::spring::SpringOpacity::new(
                            format!(
                                "compiled-preview-spring-opacity-{}",
                                snapshot.stamp.generation
                            ),
                            Text("Spring opacity preview"),
                            target,
                        )
                        .expect("validated spring opacity target")
                        .config(
                            rustui::animation::spring::SpringConfig::new(170.0, 26.0, 1.0)
                                .expect("documented default spring parameters"),
                        ),
                    )
                },
            )
            .unwrap();

        let shadow_specimen_schema = ComponentSchema {
            id: ComponentId::new("shadow-specimen").unwrap(),
            version: 1,
            display_name: "Shadow".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("label").unwrap(),
                    tree_label_schema("Preview shadow"),
                ),
                (
                    PropertyId::new("elevation").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("medium".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(6),
                        allowed_values: ["small", "medium", "large"]
                            .into_iter()
                            .map(str::to_owned)
                            .collect(),
                    },
                ),
            ]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                shadow_specimen_schema,
                |snapshot, _context: &FactoryContext| {
                    use rustui::modifiers::shadow::{Shadow, ShadowElevation};
                    let label = property_string(snapshot, "label").expect("validated Shadow label");
                    let elevation = match property_string(snapshot, "elevation")
                        .expect("validated Shadow elevation")
                        .as_str()
                    {
                        "small" => ShadowElevation::Small,
                        "medium" => ShadowElevation::Medium,
                        "large" => ShadowElevation::Large,
                        _ => unreachable!("Shadow elevation is schema-validated"),
                    };
                    AnyView::new(
                        Shadow(
                            format!("compiled-preview-shadow-{}", snapshot.stamp.generation),
                            Container(Text(label))
                                .padding(12.0)
                                .background(Surface::Panel),
                        )
                        .elevation(elevation),
                    )
                },
            )
            .unwrap();

        let background_specimen_schema = ComponentSchema {
            id: ComponentId::new("background-specimen").unwrap(),
            version: 1,
            display_name: "Background".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("label").unwrap(),
                    tree_label_schema("Preview background"),
                ),
                (
                    PropertyId::new("surface").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("panel".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(PREVIEW_LAYOUT_LABEL_MAX_LENGTH),
                        allowed_values: PREVIEW_CONTAINER_SURFACES
                            .iter()
                            .map(|surface| (*surface).into())
                            .collect(),
                    },
                ),
            ]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                background_specimen_schema,
                |snapshot, _context: &FactoryContext| {
                    let label =
                        property_string(snapshot, "label").expect("validated Background label");
                    let surface = match property_string(snapshot, "surface")
                        .expect("validated Background surface")
                        .as_str()
                    {
                        "background" => Surface::Background,
                        "sidebar" => Surface::Sidebar,
                        "panel" => Surface::Panel,
                        _ => unreachable!("Background surface is schema-validated"),
                    };
                    AnyView::new(rustui::modifiers::background::Background(
                        format!("compiled-preview-background-{}", snapshot.stamp.generation),
                        Container(Text(label)).padding(12.0),
                        surface,
                    ))
                },
            )
            .unwrap();

        let border_specimen_schema = ComponentSchema {
            id: ComponentId::new("border-specimen").unwrap(),
            version: 1,
            display_name: "Border".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("width").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::Number,
                        default: Some(PropertyValue::Number(1.0)),
                        required: false,
                        editable: true,
                        minimum: Some(0.0),
                        maximum: Some(8.0),
                        min_length: None,
                        max_length: None,
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("color").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("border".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(10),
                        allowed_values: ["border", "accent", "focus-ring"]
                            .into_iter()
                            .map(str::to_owned)
                            .collect(),
                    },
                ),
            ]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                border_specimen_schema,
                |snapshot, _context: &FactoryContext| {
                    use rustui::modifiers::border::{Border, BorderColor};
                    let width =
                        property_number(snapshot, "width").expect("validated Border width") as f32;
                    let color = match property_string(snapshot, "color")
                        .expect("validated Border color")
                        .as_str()
                    {
                        "border" => BorderColor::Border,
                        "accent" => BorderColor::Accent,
                        "focus-ring" => BorderColor::FocusRing,
                        _ => unreachable!("Border color is schema-validated"),
                    };
                    AnyView::new(
                        Border(
                            format!("compiled-preview-border-{}", snapshot.stamp.generation),
                            Container(Text("Border preview")).padding(12.0),
                        )
                        .width(width)
                        .color(color),
                    )
                },
            )
            .unwrap();

        let semantics_specimen_schema = ComponentSchema {
            id: ComponentId::new("semantics-specimen").unwrap(),
            version: 1,
            display_name: "Semantics".into(),
            properties: BTreeMap::from([(
                PropertyId::new("role").unwrap(),
                PropertySchema {
                    kind: PropertyKind::String,
                    default: Some(PropertyValue::String("group".into())),
                    required: false,
                    editable: true,
                    minimum: None,
                    maximum: None,
                    min_length: Some(1),
                    max_length: Some(10),
                    allowed_values: ["group", "region", "section", "navigation"]
                        .into_iter()
                        .map(str::to_owned)
                        .collect(),
                },
            )]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                semantics_specimen_schema,
                |snapshot, _context: &FactoryContext| {
                    use rustui::accessibility::semantics::{Semantics, SemanticsRole};
                    let role = match property_string(snapshot, "role")
                        .expect("validated Semantics role")
                        .as_str()
                    {
                        "group" => SemanticsRole::Group,
                        "region" => SemanticsRole::Region,
                        "section" => SemanticsRole::Section,
                        "navigation" => SemanticsRole::Navigation,
                        _ => unreachable!("Semantics role is schema-validated"),
                    };
                    AnyView::new(
                        Semantics(
                            format!("compiled-preview-semantics-{}", snapshot.stamp.generation),
                            Text("Semantic content preview"),
                        )
                        .role(role)
                        .label("Preview semantic container"),
                    )
                },
            )
            .unwrap();

        let card_specimen_schema = ComponentSchema {
            id: ComponentId::new("card-specimen").unwrap(),
            version: 1,
            display_name: "Card".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("label").unwrap(),
                    tree_label_schema("Preview card"),
                ),
                (
                    PropertyId::new("border").unwrap(),
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
                card_specimen_schema,
                |snapshot, _context: &FactoryContext| {
                    let label = property_string(snapshot, "label").expect("validated Card label");
                    let border =
                        property_boolean(snapshot, "border").expect("validated Card border state");
                    let card = Card(
                        format!("compiled-preview-card-{}", snapshot.stamp.generation),
                        Text(label),
                    );
                    AnyView::new(card.border(border))
                },
            )
            .unwrap();

        let container_specimen_schema = ComponentSchema {
            id: ComponentId::new("container-specimen").unwrap(),
            version: 1,
            display_name: "Container".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("label").unwrap(),
                    tree_label_schema("Preview container"),
                ),
                (
                    PropertyId::new("surface").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("panel".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(PREVIEW_LAYOUT_LABEL_MAX_LENGTH),
                        allowed_values: PREVIEW_CONTAINER_SURFACES
                            .iter()
                            .map(|surface| (*surface).into())
                            .collect(),
                    },
                ),
                (
                    PropertyId::new("padding").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::Number,
                        default: Some(PropertyValue::Number(PREVIEW_CONTAINER_PADDING_DEFAULT)),
                        required: false,
                        editable: true,
                        minimum: Some(PREVIEW_CONTAINER_PADDING_MIN),
                        maximum: Some(PREVIEW_CONTAINER_PADDING_MAX),
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
                container_specimen_schema,
                |snapshot, _context: &FactoryContext| {
                    let label =
                        property_string(snapshot, "label").expect("validated Container label");
                    let padding = property_number(snapshot, "padding")
                        .expect("validated Container padding")
                        as f32;
                    let surface = match property_string(snapshot, "surface")
                        .expect("validated Container surface")
                        .as_str()
                    {
                        "background" => Surface::Background,
                        "sidebar" => Surface::Sidebar,
                        "panel" => Surface::Panel,
                        _ => unreachable!("Container surface is schema-validated"),
                    };
                    AnyView::new(
                        Container(Text(label))
                            .padding(padding)
                            .background(surface)
                            .border(),
                    )
                },
            )
            .unwrap();

        let split_view_specimen_schema = ComponentSchema {
            id: ComponentId::new("split-view-specimen").unwrap(),
            version: 1,
            display_name: "SplitView".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("orientation").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("horizontal".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(PREVIEW_LAYOUT_LABEL_MAX_LENGTH),
                        allowed_values: PREVIEW_SPLIT_ORIENTATIONS
                            .iter()
                            .map(|orientation| (*orientation).into())
                            .collect(),
                    },
                ),
                (
                    PropertyId::new("first_size").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::Number,
                        default: Some(PropertyValue::Number(180.0)),
                        required: false,
                        editable: true,
                        minimum: Some(120.0),
                        maximum: Some(360.0),
                        min_length: None,
                        max_length: None,
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("first_label").unwrap(),
                    tree_label_schema("Files"),
                ),
                (
                    PropertyId::new("second_label").unwrap(),
                    tree_label_schema("Details"),
                ),
            ]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                split_view_specimen_schema,
                |snapshot, _context: &FactoryContext| {
                    let orientation = property_string(snapshot, "orientation")
                        .expect("validated SplitView orientation");
                    let first_label = property_string(snapshot, "first_label")
                        .expect("validated first SplitView label");
                    let second_label = property_string(snapshot, "second_label")
                        .expect("validated second SplitView label");
                    let first_size = property_number(snapshot, "first_size")
                        .expect("validated first SplitView panel size")
                        as f32;
                    let split = if orientation == "vertical" {
                        SplitView::vertical(format!(
                            "compiled-preview-split-view-{}",
                            snapshot.stamp.generation
                        ))
                    } else {
                        SplitView::horizontal(format!(
                            "compiled-preview-split-view-{}",
                            snapshot.stamp.generation
                        ))
                    };
                    let first = SplitPanel::new(
                        "first",
                        Container(Text(first_label))
                            .padding(8.0)
                            .background(Surface::Panel),
                    )
                    .size_range(120.0..360.0)
                    .initial_size(first_size);
                    let second = SplitPanel::new(
                        "second",
                        Container(Text(second_label))
                            .padding(8.0)
                            .background(Surface::Sidebar),
                    )
                    .size_range(120.0..360.0)
                    .initial_size(180.0);
                    AnyView::new(split.panel(first).panel(second))
                },
            )
            .unwrap();

    }
}
