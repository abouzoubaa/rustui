use super::*;

pub(in super::super) fn register_data_specimens(registry: &mut PreviewRegistry<FactoryContext>) {
    {
        let keyed_list_specimen_schema = ComponentSchema {
            id: ComponentId::new("keyed-list-specimen").unwrap(),
            version: 1,
            display_name: "KeyedList".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("first_label").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("First item".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(PREVIEW_KEYED_LIST_LABEL_MAX_LENGTH),
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("second_label").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Second item".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(PREVIEW_KEYED_LIST_LABEL_MAX_LENGTH),
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("selected_id").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("first".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: None,
                        max_length: None,
                        allowed_values: vec!["first".into(), "second".into()],
                    },
                ),
            ]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                keyed_list_specimen_schema,
                |snapshot, context: &FactoryContext| {
                    let first_label = property_string(snapshot, "first_label")
                        .expect("validated KeyedList first label");
                    let second_label = property_string(snapshot, "second_label")
                        .expect("validated KeyedList second label");
                    let selected_id = property_string(snapshot, "selected_id")
                        .expect("validated KeyedList selection");
                    let selection = CompiledPreviewGallery::string_selection_binding(
                        Some(selected_id),
                        PropertyBindingContext {
                            property: PropertyId::new("selected_id").unwrap(),
                            stamp: context.stamp_lease.clone(),
                        },
                    );
                    AnyView::new(KeyedList(
                        format!("compiled-preview-keyed-list-{}", snapshot.stamp.generation),
                        [
                            KeyedListItem::new("first", first_label),
                            KeyedListItem::new("second", second_label),
                        ],
                        selection,
                    ))
                },
            )
            .unwrap();

        let data_grid_specimen_schema = ComponentSchema {
            id: ComponentId::new("data-grid-specimen").unwrap(),
            version: 1,
            display_name: "DataGrid".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("label").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Quarterly revenue".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(PREVIEW_DATAGRID_LABEL_MAX_LENGTH),
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("selected_id").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("none".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: None,
                        max_length: None,
                        allowed_values: vec![
                            "none".into(),
                            "row-0".into(),
                            "row-1".into(),
                            "row-2".into(),
                        ],
                    },
                ),
            ]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                data_grid_specimen_schema,
                |snapshot, context: &FactoryContext| {
                    let label =
                        property_string(snapshot, "label").expect("validated DataGrid label");
                    let selected_id = property_string(snapshot, "selected_id")
                        .expect("validated DataGrid selection");
                    let selected_row = selected_id
                        .strip_prefix("row-")
                        .and_then(|row| row.parse::<usize>().ok());
                    let selection = CompiledPreviewGallery::data_grid_selection_binding(
                        selected_row,
                        PropertyBindingContext {
                            property: PropertyId::new("selected_id").unwrap(),
                            stamp: context.stamp_lease.clone(),
                        },
                    );
                    let table = rustui::components::data::table::Table(
                        PREVIEW_DATAGRID_COLUMNS,
                        PREVIEW_DATAGRID_ROWS,
                        selection,
                    )
                    .id(format!(
                        "compiled-preview-data-grid-table-{}",
                        snapshot.stamp.generation
                    ));
                    AnyView::new(
                        rustui::components::data::data_grid::DataGrid(table, label).id(format!(
                            "compiled-preview-data-grid-{}",
                            snapshot.stamp.generation
                        )),
                    )
                },
            )
            .unwrap();
        let list_specimen_schema = ComponentSchema {
            id: ComponentId::new("list-specimen").unwrap(),
            version: 1,
            display_name: "List".into(),
            properties: BTreeMap::from([(
                PropertyId::new("selected_id").unwrap(),
                PropertySchema {
                    kind: PropertyKind::String,
                    default: Some(PropertyValue::String("none".into())),
                    required: false,
                    editable: true,
                    minimum: None,
                    maximum: None,
                    min_length: None,
                    max_length: None,
                    allowed_values: PREVIEW_LIST_SELECTIONS
                        .iter()
                        .map(|value| (*value).into())
                        .collect(),
                },
            )]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                list_specimen_schema,
                |snapshot, context: &FactoryContext| {
                    let selected_id =
                        property_string(snapshot, "selected_id").expect("validated List selection");
                    let selected_index = selected_id
                        .strip_prefix("row-")
                        .and_then(|row| row.parse::<usize>().ok());
                    let selection = CompiledPreviewGallery::list_selection_binding(
                        selected_index,
                        PropertyBindingContext {
                            property: PropertyId::new("selected_id").unwrap(),
                            stamp: context.stamp_lease.clone(),
                        },
                    );
                    AnyView::new(List(PREVIEW_LIST_ROWS, selection).id(format!(
                        "compiled-preview-list-{}",
                        snapshot.stamp.generation
                    )))
                },
            )
            .unwrap();
        let table_specimen_schema = ComponentSchema {
            id: ComponentId::new("table-specimen").unwrap(),
            version: 1,
            display_name: "Table".into(),
            properties: BTreeMap::from([(
                PropertyId::new("selected_id").unwrap(),
                PropertySchema {
                    kind: PropertyKind::String,
                    default: Some(PropertyValue::String("none".into())),
                    required: false,
                    editable: true,
                    minimum: None,
                    maximum: None,
                    min_length: None,
                    max_length: None,
                    allowed_values: PREVIEW_LIST_SELECTIONS
                        .iter()
                        .map(|value| (*value).into())
                        .collect(),
                },
            )]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                table_specimen_schema,
                |snapshot, context: &FactoryContext| {
                    let selected_id = property_string(snapshot, "selected_id")
                        .expect("validated Table selection");
                    let selected_row = selected_id
                        .strip_prefix("row-")
                        .and_then(|row| row.parse::<usize>().ok());
                    let selection = CompiledPreviewGallery::data_grid_selection_binding(
                        selected_row,
                        PropertyBindingContext {
                            property: PropertyId::new("selected_id").unwrap(),
                            stamp: context.stamp_lease.clone(),
                        },
                    );
                    AnyView::new(
                        rustui::components::data::table::Table(
                            PREVIEW_TABLE_COLUMNS,
                            PREVIEW_TABLE_ROWS,
                            selection,
                        )
                        .id(format!(
                            "compiled-preview-table-specimen-{}",
                            snapshot.stamp.generation
                        )),
                    )
                },
            )
            .unwrap();
        for (component, display_name, outline) in [
            ("tree-specimen", "Tree", false),
            ("outline-specimen", "Outline", true),
        ] {
            let mut tree_specimen_schema = ComponentSchema {
                id: ComponentId::new(component).unwrap(),
                version: 1,
                display_name: display_name.into(),
                properties: BTreeMap::from([
                    (
                        PropertyId::new("root_label").unwrap(),
                        tree_label_schema("Project"),
                    ),
                    (
                        PropertyId::new("src_label").unwrap(),
                        tree_label_schema("src"),
                    ),
                    (
                        PropertyId::new("readme_label").unwrap(),
                        tree_label_schema("README.md"),
                    ),
                    (
                        PropertyId::new("tests_label").unwrap(),
                        tree_label_schema("tests"),
                    ),
                    (
                        PropertyId::new("selected_id").unwrap(),
                        PropertySchema {
                            kind: PropertyKind::String,
                            default: Some(PropertyValue::String("none".into())),
                            required: false,
                            editable: true,
                            minimum: None,
                            maximum: None,
                            min_length: None,
                            max_length: None,
                            allowed_values: std::iter::once("none".into())
                                .chain(PREVIEW_TREE_NODE_IDS.iter().map(|id| (*id).into()))
                                .collect(),
                        },
                    ),
                ]),
                events: BTreeMap::new(),
            };
            if outline {
                tree_specimen_schema.properties.insert(
                    PropertyId::new("expanded").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("all".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: None,
                        max_length: None,
                        allowed_values: ["collapsed", "root", "nested", "all"]
                            .into_iter()
                            .map(str::to_owned)
                            .collect(),
                    },
                );
            }
            registry
                .register(
                    tree_specimen_schema,
                    move |snapshot, context: &FactoryContext| {
                        let root_label = property_string(snapshot, "root_label")
                            .expect("validated Tree root label");
                        let src_label = property_string(snapshot, "src_label")
                            .expect("validated Tree src label");
                        let readme_label = property_string(snapshot, "readme_label")
                            .expect("validated Tree README label");
                        let tests_label = property_string(snapshot, "tests_label")
                            .expect("validated Tree tests label");
                        let selected_id = property_string(snapshot, "selected_id")
                            .expect("validated Tree selection");
                        let selection = CompiledPreviewGallery::tree_selection_binding(
                            (selected_id != "none").then_some(selected_id),
                            PropertyBindingContext {
                                property: PropertyId::new("selected_id").unwrap(),
                                stamp: context.stamp_lease.clone(),
                            },
                        );
                        let nodes = [rustui::components::data::tree::TreeNode::new(
                            PREVIEW_TREE_NODE_IDS[0],
                            root_label,
                        )
                        .expanded(true)
                        .child(
                            rustui::components::data::tree::TreeNode::new(
                                PREVIEW_TREE_NODE_IDS[1],
                                src_label,
                            )
                            .expanded(true)
                            .children([
                                rustui::components::data::tree::TreeNode::new(
                                    PREVIEW_TREE_NODE_IDS[2],
                                    readme_label,
                                ),
                                rustui::components::data::tree::TreeNode::new(
                                    PREVIEW_TREE_NODE_IDS[3],
                                    tests_label,
                                ),
                            ]),
                        )];
                        if outline {
                            let expanded = property_string(snapshot, "expanded")
                                .expect("validated Outline expansion");
                            let expanded = CompiledPreviewGallery::outline_expansion_binding(
                                expanded,
                                PropertyBindingContext {
                                    property: PropertyId::new("expanded").unwrap(),
                                    stamp: context.stamp_lease.clone(),
                                },
                            );
                            AnyView::new(
                                Outline(
                                    format!(
                                        "compiled-preview-outline-{}",
                                        snapshot.stamp.generation
                                    ),
                                    nodes,
                                    selection,
                                    expanded,
                                )
                                .accessibility_label("Project outline"),
                            )
                        } else {
                            AnyView::new(rustui::components::data::tree::Tree(nodes, selection).id(
                                format!("compiled-preview-tree-{}", snapshot.stamp.generation),
                            ))
                        }
                    },
                )
                .unwrap();
        }
        let navigation_split_specimen_schema = ComponentSchema {
            id: ComponentId::new("navigation-split-specimen").unwrap(),
            version: 1,
            display_name: "NavigationSplitView".into(),
            properties: BTreeMap::from([(
                PropertyId::new("selected_id").unwrap(),
                PropertySchema {
                    kind: PropertyKind::String,
                    default: Some(PropertyValue::String("none".into())),
                    required: false,
                    editable: true,
                    minimum: None,
                    maximum: None,
                    min_length: None,
                    max_length: None,
                    allowed_values: PREVIEW_NAVIGATION_SPLIT_SELECTIONS
                        .iter()
                        .map(|value| (*value).into())
                        .collect(),
                },
            )]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                navigation_split_specimen_schema,
                |snapshot, context: &FactoryContext| {
                    let selected_id = property_string(snapshot, "selected_id")
                        .expect("validated NavigationSplit selection");
                    let selection = CompiledPreviewGallery::string_selection_binding(
                        (selected_id != "none").then_some(selected_id),
                        PropertyBindingContext {
                            property: PropertyId::new("selected_id").unwrap(),
                            stamp: context.stamp_lease.clone(),
                        },
                    );
                    AnyView::new(
                        Container(
                            NavigationSplitView::new(
                                format!(
                                    "compiled-preview-navigation-split-{}",
                                    snapshot.stamp.generation
                                ),
                                [
                                    NavigationSplitItem::new(
                                        "files",
                                        "Files",
                                        Text("Files detail"),
                                    ),
                                    NavigationSplitItem::new(
                                        "settings",
                                        "Settings",
                                        Text("Settings detail"),
                                    ),
                                ],
                                selection,
                            )
                            .accessibility_label("Preview navigation"),
                        )
                        .width(520.0)
                        .height(240.0),
                    )
                },
            )
            .unwrap();
        let navigation_stack_specimen_schema = ComponentSchema {
            id: ComponentId::new("navigation-stack-specimen").unwrap(),
            version: 1,
            display_name: "NavigationStack".into(),
            properties: BTreeMap::from([(
                PropertyId::new("selected_id").unwrap(),
                PropertySchema {
                    kind: PropertyKind::String,
                    default: Some(PropertyValue::String("none".into())),
                    required: false,
                    editable: true,
                    minimum: None,
                    maximum: None,
                    min_length: None,
                    max_length: None,
                    allowed_values: PREVIEW_NAVIGATION_STACK_SELECTIONS
                        .iter()
                        .map(|value| (*value).into())
                        .collect(),
                },
            )]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                navigation_stack_specimen_schema,
                |snapshot, context: &FactoryContext| {
                    let selected_id = property_string(snapshot, "selected_id")
                        .expect("validated NavigationStack selection");
                    let path = if selected_id == "none" {
                        Vec::new()
                    } else {
                        vec!["details".into()]
                    };
                    let path = CompiledPreviewGallery::navigation_stack_path_binding(
                        path,
                        PropertyBindingContext {
                            property: PropertyId::new("selected_id").unwrap(),
                            stamp: context.stamp_lease.clone(),
                        },
                    );
                    AnyView::new(
                        Container(NavigationStack(
                            VStack::new()
                                .spacing(8.0)
                                .child(Text("Overview"))
                                .child(NavigationLink("Open details", "details", path.clone())),
                            [NavigationDestination::new(
                                "details",
                                Text("Compiled details destination"),
                            )],
                            path,
                        ))
                        .width(520.0)
                        .height(240.0),
                    )
                },
            )
            .unwrap();
        let collection_specimen_schema = ComponentSchema {
            id: ComponentId::new("collection-specimen").unwrap(),
            version: 1,
            display_name: "Collection".into(),
            properties: BTreeMap::from([(
                PropertyId::new("selected_id").unwrap(),
                PropertySchema {
                    kind: PropertyKind::String,
                    default: Some(PropertyValue::String("none".into())),
                    required: false,
                    editable: true,
                    minimum: None,
                    maximum: None,
                    min_length: None,
                    max_length: None,
                    allowed_values: PREVIEW_COLLECTION_SELECTIONS
                        .iter()
                        .map(|value| (*value).into())
                        .collect(),
                },
            )]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                collection_specimen_schema,
                |snapshot, context: &FactoryContext| {
                    let selected_id = property_string(snapshot, "selected_id")
                        .expect("validated Collection selection");
                    let items: Rc<[CollectionItem<&'static str>]> = PREVIEW_COLLECTION_ITEMS
                        .iter()
                        .map(|(id, label)| CollectionItem::new(*id, *label).label(*label))
                        .collect::<Vec<_>>()
                        .into();
                    let selection = CompiledPreviewGallery::collection_selection_binding(
                        (selected_id != "none").then(|| Identity::from(selected_id)),
                        PropertyBindingContext {
                            property: PropertyId::new("selected_id").unwrap(),
                            stamp: context.stamp_lease.clone(),
                        },
                    );
                    AnyView::new(
                        Collection::new(
                            format!("compiled-preview-collection-{}", snapshot.stamp.generation),
                            items,
                            180.0,
                            44.0,
                            |_, label| Text(*label),
                        )
                        .selection(selection)
                        .label("Preview collection"),
                    )
                },
            )
            .unwrap();
    }
}
