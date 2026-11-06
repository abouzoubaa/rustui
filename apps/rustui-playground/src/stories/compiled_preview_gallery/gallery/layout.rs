pub(super) fn register_layout_specimens(registry: &mut PreviewRegistry<FactoryContext>) {
    {
        let scroll_view_specimen_schema = ComponentSchema {
            id: ComponentId::new("scroll-view-specimen").unwrap(),
            version: 1,
            display_name: "ScrollView".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("label").unwrap(),
                    tree_label_schema("Scrollable preview row"),
                ),
                (
                    PropertyId::new("axis").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("vertical".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(PREVIEW_LAYOUT_LABEL_MAX_LENGTH),
                        allowed_values: PREVIEW_SCROLL_AXES
                            .iter()
                            .map(|axis| (*axis).into())
                            .collect(),
                    },
                ),
                (
                    PropertyId::new("scrollbars").unwrap(),
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
            events: BTreeMap::new(),
        };
        registry
            .register(
                scroll_view_specimen_schema,
                |snapshot, _context: &FactoryContext| {
                    let label =
                        property_string(snapshot, "label").expect("validated ScrollView label");
                    let axis = match property_string(snapshot, "axis")
                        .expect("validated ScrollView axis")
                        .as_str()
                    {
                        "vertical" => ScrollAxis::Vertical,
                        "horizontal" => ScrollAxis::Horizontal,
                        "both" => ScrollAxis::Both,
                        _ => unreachable!("ScrollView axis is schema-validated"),
                    };
                    let scrollbars = property_boolean(snapshot, "scrollbars")
                        .expect("validated ScrollView scrollbar setting");
                    let content = match axis {
                        ScrollAxis::Vertical => AnyView::new(
                            VStack::new()
                                .spacing(8.0)
                                .children((1..=8).map(|row| Text(format!("{label} {row}")))),
                        ),
                        ScrollAxis::Horizontal => AnyView::new(
                            HStack::new()
                                .spacing(8.0)
                                .children((1..=8).map(|column| Text(format!("{label} {column}")))),
                        ),
                        ScrollAxis::Both => {
                            AnyView::new(VStack::new().spacing(8.0).children((1..=8).map(|row| {
                                HStack::new().spacing(8.0).children(
                                    (1..=8).map(|column| Text(format!("{label} {row}:{column}"))),
                                )
                            })))
                        }
                    };
                    AnyView::new(
                        ScrollView(content)
                            .id(format!(
                                "compiled-preview-scroll-view-{}",
                                snapshot.stamp.generation
                            ))
                            .width(280.0)
                            .height(180.0)
                            .axis(axis)
                            .scrollbars(scrollbars),
                    )
                },
            )
            .unwrap();

        let frame_specimen_schema = ComponentSchema {
            id: ComponentId::new("frame-specimen").unwrap(),
            version: 1,
            display_name: "Frame".into(),
            properties: BTreeMap::from([(
                PropertyId::new("width").unwrap(),
                PropertySchema {
                    kind: PropertyKind::Number,
                    default: Some(PropertyValue::Number(PREVIEW_FRAME_WIDTH_DEFAULT)),
                    required: false,
                    editable: true,
                    minimum: Some(PREVIEW_FRAME_WIDTH_MIN),
                    maximum: Some(PREVIEW_FRAME_WIDTH_MAX),
                    min_length: None,
                    max_length: None,
                    allowed_values: vec![],
                },
            )]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                frame_specimen_schema,
                |snapshot, _context: &FactoryContext| {
                    let width =
                        property_number(snapshot, "width").expect("validated Frame width") as f32;
                    AnyView::new(Frame(Text("Framed content")).width(width).height(64.0))
                },
            )
            .unwrap();
        let hstack_specimen_schema = ComponentSchema {
            id: ComponentId::new("hstack-specimen").unwrap(),
            version: 1,
            display_name: "HStack".into(),
            properties: BTreeMap::from([(
                PropertyId::new("spacing").unwrap(),
                PropertySchema {
                    kind: PropertyKind::Number,
                    default: Some(PropertyValue::Number(PREVIEW_HSTACK_SPACING_DEFAULT)),
                    required: false,
                    editable: true,
                    minimum: Some(PREVIEW_HSTACK_SPACING_MIN),
                    maximum: Some(PREVIEW_HSTACK_SPACING_MAX),
                    min_length: None,
                    max_length: None,
                    allowed_values: vec![],
                },
            )]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                hstack_specimen_schema,
                |snapshot, _context: &FactoryContext| {
                    let spacing = property_number(snapshot, "spacing")
                        .expect("validated HStack spacing")
                        as f32;
                    AnyView::new(
                        HStack::new()
                            .spacing(spacing)
                            .alignment(VerticalAlignment::Center)
                            .child(Text("First"))
                            .child(Text("Second"))
                            .child(Text("Third")),
                    )
                },
            )
            .unwrap();
        let vstack_specimen_schema = ComponentSchema {
            id: ComponentId::new("vstack-specimen").unwrap(),
            version: 1,
            display_name: "VStack".into(),
            properties: BTreeMap::from([(
                PropertyId::new("spacing").unwrap(),
                PropertySchema {
                    kind: PropertyKind::Number,
                    default: Some(PropertyValue::Number(PREVIEW_VSTACK_SPACING_DEFAULT)),
                    required: false,
                    editable: true,
                    minimum: Some(PREVIEW_VSTACK_SPACING_MIN),
                    maximum: Some(PREVIEW_VSTACK_SPACING_MAX),
                    min_length: None,
                    max_length: None,
                    allowed_values: vec![],
                },
            )]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                vstack_specimen_schema,
                |snapshot, _context: &FactoryContext| {
                    let spacing = property_number(snapshot, "spacing")
                        .expect("validated VStack spacing")
                        as f32;
                    AnyView::new(
                        VStack::new()
                            .spacing(spacing)
                            .alignment(HorizontalAlignment::Center)
                            .child(Text("First"))
                            .child(Text("Second"))
                            .child(Text("Third")),
                    )
                },
            )
            .unwrap();

        let grid_specimen_schema = ComponentSchema {
            id: ComponentId::new("grid-specimen").unwrap(),
            version: 1,
            display_name: "Grid".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("columns").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::Integer,
                        default: Some(PropertyValue::Integer(PREVIEW_GRID_COLUMNS_DEFAULT)),
                        required: false,
                        editable: true,
                        minimum: Some(PREVIEW_GRID_COLUMNS_MIN),
                        maximum: Some(PREVIEW_GRID_COLUMNS_MAX),
                        min_length: None,
                        max_length: None,
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("spacing").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::Number,
                        default: Some(PropertyValue::Number(PREVIEW_GRID_SPACING_DEFAULT)),
                        required: false,
                        editable: true,
                        minimum: Some(PREVIEW_GRID_SPACING_MIN),
                        maximum: Some(PREVIEW_GRID_SPACING_MAX),
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
                grid_specimen_schema,
                |snapshot, _context: &FactoryContext| {
                    let columns =
                        property_integer(snapshot, "columns").expect("validated Grid column count");
                    let columns =
                        u16::try_from(columns).expect("validated Grid column count fits u16");
                    let columns = std::num::NonZeroU16::new(columns)
                        .expect("validated Grid column count is nonzero");
                    let spacing = property_number(snapshot, "spacing")
                        .expect("validated Grid spacing") as f32;
                    AnyView::new(
                        Grid::new(columns)
                            .spacing(spacing)
                            .child(Text("First"))
                            .child(Text("Second"))
                            .child(Text("Third"))
                            .child(Text("Fourth")),
                    )
                },
            )
            .unwrap();

        let lazy_grid_specimen_schema = ComponentSchema {
            id: ComponentId::new("lazy-grid-specimen").unwrap(),
            version: 1,
            display_name: "LazyGrid".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("columns").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::Integer,
                        default: Some(PropertyValue::Integer(PREVIEW_LAZY_GRID_COLUMNS_DEFAULT)),
                        required: false,
                        editable: true,
                        minimum: Some(PREVIEW_LAZY_GRID_COLUMNS_MIN),
                        maximum: Some(PREVIEW_LAZY_GRID_COLUMNS_MAX),
                        min_length: None,
                        max_length: None,
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("spacing").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::Number,
                        default: Some(PropertyValue::Number(PREVIEW_LAZY_GRID_SPACING_DEFAULT)),
                        required: false,
                        editable: true,
                        minimum: Some(PREVIEW_LAZY_GRID_SPACING_MIN),
                        maximum: Some(PREVIEW_LAZY_GRID_SPACING_MAX),
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
                lazy_grid_specimen_schema,
                |snapshot, _context: &FactoryContext| {
                    let columns =
                        property_integer(snapshot, "columns").expect("validated LazyGrid columns");
                    let columns = std::num::NonZeroU16::new(
                        u16::try_from(columns).expect("validated LazyGrid columns fit u16"),
                    )
                    .expect("validated LazyGrid columns are nonzero");
                    let spacing = property_number(snapshot, "spacing")
                        .expect("validated LazyGrid spacing")
                        as f32;
                    let items: Rc<[LazyGridItem<&'static str>]> = [
                        ("row-1", "First"),
                        ("row-2", "Second"),
                        ("row-3", "Third"),
                        ("row-4", "Fourth"),
                        ("row-5", "Fifth"),
                        ("row-6", "Sixth"),
                    ]
                    .into_iter()
                    .map(|(id, label)| LazyGridItem::new(id, label))
                    .collect::<Vec<_>>()
                    .into();
                    AnyView::new(
                        LazyGrid::new(
                            "compiled-preview-lazy-grid",
                            items,
                            columns,
                            180.0,
                            32.0,
                            |_, label| Text(*label),
                        )
                        .column_gap(spacing)
                        .row_gap(spacing),
                    )
                },
            )
            .unwrap();

        let spacer_specimen_schema = ComponentSchema {
            id: ComponentId::new("spacer-specimen").unwrap(),
            version: 1,
            display_name: "Spacer".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("axis").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("horizontal".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(16),
                        allowed_values: PREVIEW_SPACER_AXES
                            .iter()
                            .map(|axis| (*axis).into())
                            .collect(),
                    },
                ),
                (
                    PropertyId::new("min_length").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::Number,
                        default: Some(PropertyValue::Number(PREVIEW_SPACER_MIN_LENGTH_DEFAULT)),
                        required: false,
                        editable: true,
                        minimum: Some(PREVIEW_SPACER_MIN_LENGTH_MIN),
                        maximum: Some(PREVIEW_SPACER_MIN_LENGTH_MAX),
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
                spacer_specimen_schema,
                |snapshot, _context: &FactoryContext| {
                    let axis = property_string(snapshot, "axis").expect("validated Spacer axis");
                    let axis = match axis.as_str() {
                        "horizontal" => SpacerAxis::Horizontal,
                        "vertical" => SpacerAxis::Vertical,
                        _ => unreachable!("validated Spacer axis"),
                    };
                    let min_length = property_number(snapshot, "min_length")
                        .expect("validated Spacer minimum length")
                        as f32;
                    let spacer = Spacer().min_length(axis, min_length);
                    if axis == SpacerAxis::Horizontal {
                        AnyView::new(
                            HStack::new()
                                .spacing(8.0)
                                .child(Text("Leading"))
                                .child(spacer)
                                .child(Text("Trailing")),
                        )
                    } else {
                        AnyView::new(
                            VStack::new()
                                .spacing(8.0)
                                .child(Text("Top"))
                                .child(spacer)
                                .child(Text("Bottom")),
                        )
                    }
                },
            )
            .unwrap();

        let zstack_specimen_schema = ComponentSchema {
            id: ComponentId::new("zstack-specimen").unwrap(),
            version: 1,
            display_name: "ZStack".into(),
            properties: BTreeMap::from([(
                PropertyId::new("alignment").unwrap(),
                PropertySchema {
                    kind: PropertyKind::String,
                    default: Some(PropertyValue::String("center".into())),
                    required: false,
                    editable: true,
                    minimum: None,
                    maximum: None,
                    min_length: Some(1),
                    max_length: Some(32),
                    allowed_values: PREVIEW_ZSTACK_ALIGNMENTS
                        .iter()
                        .map(|alignment| (*alignment).into())
                        .collect(),
                },
            )]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                zstack_specimen_schema,
                |snapshot, _context: &FactoryContext| {
                    let (horizontal, vertical) = match property_string(snapshot, "alignment")
                        .expect("validated ZStack alignment")
                        .as_str()
                    {
                        "top-leading" => (HorizontalAlignment::Start, VerticalAlignment::Start),
                        "bottom-trailing" => (HorizontalAlignment::End, VerticalAlignment::End),
                        _ => (HorizontalAlignment::Center, VerticalAlignment::Center),
                    };
                    AnyView::new(
                        Container(
                            ZStack::new()
                                .fill()
                                .alignment(ZStackAlignment::new(horizontal, vertical))
                                .child(
                                    Container(Text("Base layer"))
                                        .padding(8.0)
                                        .background(Surface::Background),
                                )
                                .child(
                                    Container(Text("Overlay layer"))
                                        .padding(8.0)
                                        .background(Surface::Sidebar),
                                ),
                        )
                        .width(240.0)
                        .height(120.0)
                        .padding(8.0)
                        .background(Surface::Panel),
                    )
                },
            )
            .unwrap();

        let geometry_reader_specimen_schema = ComponentSchema {
            id: ComponentId::new("geometry-reader-specimen").unwrap(),
            version: 1,
            display_name: "GeometryReader".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("width").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::Number,
                        default: Some(PropertyValue::Number(280.0)),
                        required: false,
                        editable: true,
                        minimum: Some(80.0),
                        maximum: Some(320.0),
                        min_length: None,
                        max_length: None,
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("height").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::Number,
                        default: Some(PropertyValue::Number(120.0)),
                        required: false,
                        editable: true,
                        minimum: Some(64.0),
                        maximum: Some(240.0),
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
                geometry_reader_specimen_schema,
                |snapshot, _context: &FactoryContext| {
                    let width = property_number(snapshot, "width")
                        .expect("validated GeometryReader width")
                        as f32;
                    let height = property_number(snapshot, "height")
                        .expect("validated GeometryReader height")
                        as f32;
                    AnyView::new(
                        GeometryReader::new(
                            format!(
                                "compiled-preview-geometry-reader-{}",
                                snapshot.stamp.generation
                            ),
                            |size| Text(format!("Observed {:.0} × {:.0}", size.width, size.height)),
                        )
                        .width(width)
                        .height(height),
                    )
                },
            )
            .unwrap();

        let property_inspector_specimen_schema = ComponentSchema {
            id: ComponentId::new("property-inspector-specimen").unwrap(),
            version: 1,
            display_name: "PropertyInspector".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("text_label").unwrap(),
                    tree_label_schema("Name"),
                ),
                (
                    PropertyId::new("toggle_label").unwrap(),
                    tree_label_schema("Enabled"),
                ),
                (
                    PropertyId::new("integer_label").unwrap(),
                    tree_label_schema("Count"),
                ),
                (
                    PropertyId::new("disabled").unwrap(),
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
                property_inspector_specimen_schema,
                |snapshot, context: &FactoryContext| {
                    let text_label = property_string(snapshot, "text_label")
                        .expect("validated PropertyInspector text label");
                    let toggle_label = property_string(snapshot, "toggle_label")
                        .expect("validated PropertyInspector toggle label");
                    let integer_label = property_string(snapshot, "integer_label")
                        .expect("validated PropertyInspector integer label");
                    let disabled = property_boolean(snapshot, "disabled")
                        .expect("validated PropertyInspector disabled state");
                    let inspector = PropertyInspector(
                        format!(
                            "compiled-preview-property-inspector-{}",
                            snapshot.stamp.generation
                        ),
                        [
                            PropertyInspectorRow::text("text", text_label, context.text.clone()),
                            PropertyInspectorRow::toggle(
                                "toggle",
                                toggle_label,
                                context.checked.clone(),
                            ),
                            PropertyInspectorRow::integer(
                                "integer",
                                integer_label,
                                context.stepper_value.clone(),
                            ),
                        ],
                    )
                    .expect("compiled PropertyInspector rows have stable unique IDs")
                    .disabled(disabled);
                    AnyView::new(inspector)
                },
            )
            .unwrap();
        let date_picker_specimen_schema = ComponentSchema {
            id: ComponentId::new("date-picker-specimen").unwrap(),
            version: 1,
            display_name: "DatePicker".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("label").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Choose a date".into())),
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
                    PropertyId::new("accessibility_label").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Start date".into())),
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
                    PropertyId::new("date").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String(String::new())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: None,
                        max_length: Some(10),
                        allowed_values: vec![],
                    },
                ),
            ]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                date_picker_specimen_schema,
                |snapshot, context: &FactoryContext| {
                    let label =
                        property_string(snapshot, "label").expect("validated DatePicker label");
                    let accessibility_label = property_string(snapshot, "accessibility_label")
                        .expect("validated DatePicker accessible name");
                    let picker = DatePicker(label, context.date_value.clone()).id(format!(
                        "compiled-preview-date-picker-{}",
                        snapshot.stamp.generation
                    ));
                    let picker = if accessibility_label.trim().is_empty() {
                        picker
                    } else {
                        picker.accessibility_label(accessibility_label)
                    };
                    AnyView::new(picker)
                },
            )
            .unwrap();
        registry
            .register_action(ActionRef::new("increment").unwrap())
            .unwrap();
        registry
            .register_action(ActionRef::new("dismiss").unwrap())
            .unwrap();
        registry
            .register_action(ActionRef::new("toggle").unwrap())
            .unwrap();
        registry
            .register_action(ActionRef::new("toolbar_primary").unwrap())
            .unwrap();
        registry
            .register_action(ActionRef::new("toolbar_secondary").unwrap())
            .unwrap();
        registry
            .register_action(ActionRef::new("breadcrumb_parent").unwrap())
            .unwrap();
        registry
            .register_action(ActionRef::new("context_menu_open_preview").unwrap())
            .unwrap();
        registry
            .register_action(ActionRef::new("toast_show").unwrap())
            .unwrap();
    }
}
