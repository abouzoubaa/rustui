pub(super) fn register_canvas_specimens(registry: &mut PreviewRegistry<FactoryContext>) {
    {
        let canvas_specimen_schema = ComponentSchema {
            id: ComponentId::new("canvas-specimen").unwrap(),
            version: 1,
            display_name: "Canvas".into(),
            properties: BTreeMap::from([(
                PropertyId::new("composition").unwrap(),
                PropertySchema {
                    kind: PropertyKind::String,
                    default: Some(PropertyValue::String("basic".into())),
                    required: false,
                    editable: true,
                    minimum: None,
                    maximum: None,
                    min_length: None,
                    max_length: None,
                    allowed_values: PREVIEW_CANVAS_COMPOSITIONS
                        .iter()
                        .map(|value| (*value).into())
                        .collect(),
                },
            )]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                canvas_specimen_schema,
                |snapshot, _context: &FactoryContext| {
                    let composition = property_string(snapshot, "composition")
                        .expect("validated Canvas composition");
                    let background = Hsla {
                        h: 0.62,
                        s: 0.2,
                        l: 0.15,
                        a: 1.0,
                    };
                    let blue = Hsla {
                        h: 0.58,
                        s: 0.82,
                        l: 0.58,
                        a: 0.72,
                    };
                    let amber = Hsla {
                        h: 0.11,
                        s: 0.9,
                        l: 0.58,
                        a: 0.72,
                    };
                    let mut scene = CanvasScene::builder();
                    scene
                        .fill_rect(CanvasRect::new(0.0, 0.0, 240.0, 140.0).unwrap(), background)
                        .unwrap();
                    if composition == "overlap" {
                        scene
                            .fill_rect(CanvasRect::new(24.0, 28.0, 112.0, 76.0).unwrap(), amber)
                            .unwrap()
                            .fill_rect(CanvasRect::new(104.0, 38.0, 112.0, 76.0).unwrap(), blue)
                            .unwrap();
                    } else {
                        scene
                            .fill_rect(CanvasRect::new(24.0, 34.0, 92.0, 72.0).unwrap(), blue)
                            .unwrap()
                            .stroke_rect(
                                CanvasRect::new(148.0, 34.0, 68.0, 72.0).unwrap(),
                                3.0,
                                amber,
                            )
                            .unwrap();
                    }
                    AnyView::new(
                        Semantics(
                            "compiled-preview-canvas",
                            Canvas(CanvasSize::new(240.0, 140.0).unwrap(), scene.build()),
                        )
                        .role(SemanticsRole::Group)
                        .label("Compiled Canvas specimen with fixed vector geometry"),
                    )
                },
            )
            .unwrap();
    }
}
