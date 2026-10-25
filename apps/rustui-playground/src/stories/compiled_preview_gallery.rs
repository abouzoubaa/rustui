//! D1c schema-driven local preview inspector; all controls use public RustUI APIs.
use rustui::{prelude::*, preview::*};
use std::{cell::RefCell, collections::BTreeMap, rc::Rc, time::Duration};

pub fn demo() -> impl View {
    Component("compiled-preview-gallery", CompiledPreviewGallery::default)
}

struct FactoryContext {
    owner: OwnerAction<CompiledPreviewGallery>,
    stamp_lease: Rc<RefCell<Option<PreviewStamp>>>,
    text: Binding<String>,
    secure_value: Binding<String>,
    checked: Binding<bool>,
    slider_value: Binding<f32>,
    stepper_value: Binding<i32>,
    color_value: Binding<Option<Hsla>>,
    date_value: Binding<Option<NaiveDate>>,
    token_values: Binding<Vec<String>>,
}

const PREVIEW_SLIDER_MIN: f32 = 0.0;
const PREVIEW_SLIDER_MAX: f32 = 1.0;
const PREVIEW_SLIDER_STEP: f32 = 0.125;
const PREVIEW_SLIDER_DEFAULT: f64 = 0.5;
const PREVIEW_PROGRESS_MIN: f64 = 0.0;
const PREVIEW_PROGRESS_MAX: f64 = 100.0;
const PREVIEW_PROGRESS_DEFAULT: f64 = 25.0;
const PREVIEW_STEPPER_MIN: i32 = 0;
const PREVIEW_STEPPER_MAX: i32 = 10;
const PREVIEW_STEPPER_LABEL_MAX_LENGTH: usize = 80;
const PREVIEW_POPOVER_TRIGGER_MAX_LENGTH: usize = 80;
const PREVIEW_EMPTYSTATE_TITLE_MAX_LENGTH: usize = 120;
const PREVIEW_EMPTYSTATE_DETAIL_MAX_LENGTH: usize = 240;
const PREVIEW_ERRORSTATE_TITLE_MAX_LENGTH: usize = 120;
const PREVIEW_ERRORSTATE_DETAIL_MAX_LENGTH: usize = 240;
const PREVIEW_DISCLOSURE_LABEL_MAX_LENGTH: usize = 80;
const PREVIEW_TOOLBAR_LABEL_MAX_LENGTH: usize = 80;
const PREVIEW_IMAGE_LABEL_MAX_LENGTH: usize = 120;
const PREVIEW_IMAGE_FITS: [&str; 5] = ["contain", "cover", "fill", "scale-down", "none"];
const PREVIEW_BREADCRUMB_LABEL_MAX_LENGTH: usize = 80;
const PREVIEW_CONTEXT_MENU_LABEL_MAX_LENGTH: usize = 80;
const PREVIEW_TEXT_VALUE_MAX_LENGTH: usize = 120;
const PREVIEW_TOKEN_VALUES_MAX_LENGTH: usize = 2048;
const PREVIEW_TOKEN_FIELD_DEFAULTS: [&str; 2] = ["Atlas", "Studio"];
const PREVIEW_TOAST_MESSAGE_MAX_LENGTH: usize = 120;
const PREVIEW_TOAST_TITLE_MAX_LENGTH: usize = 120;
const PREVIEW_SHEET_TRIGGER_MAX_LENGTH: usize = 80;
const PREVIEW_SHEET_TITLE_MAX_LENGTH: usize = 120;
const PREVIEW_SHEET_BODY_MAX_LENGTH: usize = 240;
const PREVIEW_SIDEBAR_LABEL_MAX_LENGTH: usize = 80;
const PREVIEW_KEYED_LIST_LABEL_MAX_LENGTH: usize = 80;
const PREVIEW_DATAGRID_LABEL_MAX_LENGTH: usize = 80;
const PREVIEW_TREE_LABEL_MAX_LENGTH: usize = 80;
const DATE_PICKER_DATE_REJECTION: &str = "DatePicker dates must use years 0001 through 9999";
const PREVIEW_TREE_NODE_IDS: [&str; 4] = ["root", "src", "readme", "tests"];
const PREVIEW_DATAGRID_COLUMNS: [&str; 2] = ["Quarter", "Revenue"];
const PREVIEW_DATAGRID_ROWS: [[&str; 2]; 3] =
    [["Q1", "$18.2M"], ["Q2", "$20.4M"], ["Q3", "$22.1M"]];
const PREVIEW_LIST_ROWS: [&str; 3] = ["First item", "Second item", "Third item"];
const PREVIEW_LIST_SELECTIONS: [&str; 4] = ["none", "row-0", "row-1", "row-2"];
const PREVIEW_TABLE_COLUMNS: [&str; 2] = ["Task", "Status"];
const PREVIEW_TABLE_ROWS: [[&str; 2]; 3] = [
    ["Design", "Complete"],
    ["Implement", "In progress"],
    ["Review", "Queued"],
];
const PREVIEW_NAVIGATION_SPLIT_SELECTIONS: [&str; 3] = ["none", "files", "settings"];
const PREVIEW_NAVIGATION_STACK_SELECTIONS: [&str; 2] = ["none", "details"];
const PREVIEW_COLLECTION_ITEMS: [(&str, &str); 3] = [
    ("first", "First item"),
    ("second", "Second item"),
    ("third", "Third item"),
];
const PREVIEW_COLLECTION_SELECTIONS: [&str; 4] = ["none", "first", "second", "third"];
const PREVIEW_CANVAS_COMPOSITIONS: [&str; 2] = ["basic", "overlap"];
const PREVIEW_LOCALIZED_TEXT_KEYS: [&str; 2] = ["rustui-action-close", "rustui-action-confirm"];
const PREVIEW_LOCALIZED_TEXT_DEFAULT: &str = PREVIEW_LOCALIZED_TEXT_KEYS[0];

fn preview_nonblank_detail(detail: &str) -> Option<&str> {
    (!detail.trim().is_empty()).then_some(detail)
}

fn preview_inspector_rejection_live_region_value(status: &str) -> Option<&str> {
    (status == DATE_PICKER_DATE_REJECTION
        || status.to_ascii_lowercase().contains("reject")
        || status.starts_with("Enter a finite number")
        || status.starts_with("Enter finite number")
        || status.starts_with("Color channels must be finite values")
        || status.starts_with("Stepper value must fit its control")
        || status.starts_with("Delayed parent rerender failed:")
        || status.starts_with("Import rejected:")
        || status.starts_with("Export rejected:"))
    .then_some(status)
}

fn preview_inspector_property_is_visible(property: &PropertySchema) -> bool {
    property.editable
}

fn tree_label_schema(default: &str) -> PropertySchema {
    PropertySchema {
        kind: PropertyKind::String,
        default: Some(PropertyValue::String(default.into())),
        required: false,
        editable: true,
        minimum: None,
        maximum: None,
        min_length: Some(1),
        max_length: Some(PREVIEW_TREE_LABEL_MAX_LENGTH),
        allowed_values: vec![],
    }
}

const PREVIEW_COLOR_HUE: f64 = 0.58;
const PREVIEW_COLOR_SATURATION: f64 = 0.65;
const PREVIEW_COLOR_LIGHTNESS: f64 = 0.55;
const PREVIEW_COLOR_ALPHA: f64 = 1.0;
const PREVIEW_STEPPER_STEP: i32 = 2;
const PREVIEW_STEPPER_DEFAULT: i32 = 4;
const PREVIEW_LAYOUT_LABEL_MAX_LENGTH: usize = 80;
const PREVIEW_SPLIT_ORIENTATIONS: [&str; 2] = ["horizontal", "vertical"];
const PREVIEW_SCROLL_AXES: [&str; 3] = ["vertical", "horizontal", "both"];
const PREVIEW_CONTAINER_SURFACES: [&str; 3] = ["background", "sidebar", "panel"];
const PREVIEW_CONTAINER_PADDING_MIN: f64 = 0.0;
const PREVIEW_CONTAINER_PADDING_MAX: f64 = 32.0;
const PREVIEW_CONTAINER_PADDING_DEFAULT: f64 = 12.0;
const PREVIEW_FRAME_WIDTH_MIN: f64 = 80.0;
const PREVIEW_FRAME_WIDTH_MAX: f64 = 320.0;
const PREVIEW_FRAME_WIDTH_DEFAULT: f64 = 240.0;
const PREVIEW_HSTACK_SPACING_MIN: f64 = 0.0;
const PREVIEW_HSTACK_SPACING_MAX: f64 = 32.0;
const PREVIEW_HSTACK_SPACING_DEFAULT: f64 = 12.0;
const PREVIEW_VSTACK_SPACING_MIN: f64 = 0.0;
const PREVIEW_VSTACK_SPACING_MAX: f64 = 32.0;
const PREVIEW_VSTACK_SPACING_DEFAULT: f64 = 12.0;
const PREVIEW_GRID_COLUMNS_MIN: f64 = 1.0;
const PREVIEW_GRID_COLUMNS_MAX: f64 = 4.0;
const PREVIEW_GRID_COLUMNS_DEFAULT: i64 = 2;
const PREVIEW_GRID_SPACING_MIN: f64 = 0.0;
const PREVIEW_GRID_SPACING_MAX: f64 = 32.0;
const PREVIEW_GRID_SPACING_DEFAULT: f64 = 12.0;
const PREVIEW_LAZY_GRID_COLUMNS_MIN: f64 = 1.0;
const PREVIEW_LAZY_GRID_COLUMNS_MAX: f64 = 4.0;
const PREVIEW_LAZY_GRID_COLUMNS_DEFAULT: i64 = 2;
const PREVIEW_LAZY_GRID_SPACING_MIN: f64 = 0.0;
const PREVIEW_LAZY_GRID_SPACING_MAX: f64 = 32.0;
const PREVIEW_LAZY_GRID_SPACING_DEFAULT: f64 = 12.0;
const PREVIEW_SPACER_AXES: [&str; 2] = ["horizontal", "vertical"];
const PREVIEW_SPACER_MIN_LENGTH_MIN: f64 = 0.0;
const PREVIEW_SPACER_MIN_LENGTH_MAX: f64 = 64.0;
const PREVIEW_SPACER_MIN_LENGTH_DEFAULT: f64 = 16.0;
const PREVIEW_ZSTACK_ALIGNMENTS: [&str; 3] = ["center", "top-leading", "bottom-trailing"];

// Empty means no selection; selected dates use canonical four-digit ISO years.
fn parse_preview_date(value: &str) -> Result<Option<NaiveDate>, ()> {
    if value.is_empty() {
        return Ok(None);
    }
    let date = NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| ())?;
    (value.len() == 10 && !value.starts_with("0000") && date.to_string() == value)
        .then_some(Some(date))
        .ok_or(())
}

fn property_date(snapshot: &PreviewSnapshot) -> Option<NaiveDate> {
    parse_preview_date(&property_string(snapshot, "date")?)
        .ok()
        .flatten()
}

#[derive(Clone)]
struct PropertyBindingContext {
    property: PropertyId,
    stamp: Rc<RefCell<Option<PreviewStamp>>>,
}

fn property_string(snapshot: &PreviewSnapshot, name: &str) -> Option<String> {
    match snapshot
        .preview
        .properties
        .get(&PropertyId::new(name).ok()?)?
    {
        PropertyValue::String(value) => Some(value.clone()),
        _ => None,
    }
}

fn preview_inspector_string_initial_value(
    snapshot: Option<&PreviewSnapshot>,
    property: &str,
) -> String {
    snapshot
        .and_then(|snapshot| property_string(snapshot, property))
        .unwrap_or_default()
}

fn parse_preview_token_values(value: &str) -> Result<Vec<String>, serde_json::Error> {
    serde_json::from_str(value)
}

fn localized_text_message_id(snapshot: &PreviewSnapshot) -> String {
    property_string(snapshot, "message_id").expect("validated LocalizedText message id")
}

fn property_boolean(snapshot: &PreviewSnapshot, name: &str) -> Option<bool> {
    match snapshot
        .preview
        .properties
        .get(&PropertyId::new(name).ok()?)?
    {
        PropertyValue::Boolean(value) => Some(*value),
        _ => None,
    }
}

fn property_number(snapshot: &PreviewSnapshot, name: &str) -> Option<f64> {
    match snapshot
        .preview
        .properties
        .get(&PropertyId::new(name).ok()?)?
    {
        PropertyValue::Number(value) => Some(*value),
        _ => None,
    }
}

fn property_integer(snapshot: &PreviewSnapshot, name: &str) -> Option<i64> {
    match snapshot
        .preview
        .properties
        .get(&PropertyId::new(name).ok()?)?
    {
        PropertyValue::Integer(value) => Some(*value),
        _ => None,
    }
}

fn property_color(snapshot: &PreviewSnapshot) -> Option<Option<Hsla>> {
    if !property_boolean(snapshot, "has_color")? {
        return Some(None);
    }
    Some(Some(Hsla {
        h: property_number(snapshot, "hue")? as f32,
        s: property_number(snapshot, "saturation")? as f32,
        l: property_number(snapshot, "lightness")? as f32,
        a: property_number(snapshot, "alpha")? as f32,
    }))
}

fn color_channel_schema(default: f64) -> PropertySchema {
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
    }
}

fn progress_control_value(value: f64) -> f32 {
    value as f32
}

struct CompiledPreviewGallery {
    host: PreviewHost<FactoryContext>,
    stamp_lease: Rc<RefCell<Option<PreviewStamp>>>,
    text: String,
    text_field_placeholder: String,
    search_field_placeholder: String,
    text_editor_placeholder: String,
    secure_field_placeholder: String,
    secure_value: String,
    token_values: Vec<String>,
    token_field_placeholder: String,
    enabled: bool,
    amount: f64,
    amount_draft: String,
    number_drafts: BTreeMap<PropertyId, String>,
    stepper_count: i32,
    stale_amount_callback: Option<Binding<String>>,
    activations: usize,
    renders: usize,
    status: String,
    descriptor_json: String,
}

impl Default for CompiledPreviewGallery {
    fn default() -> Self {
        let mut registry = PreviewRegistry::new();
        for name in ["text", "button", "text-field"] {
            let mut events = BTreeMap::new();
            if name == "button" {
                events.insert(
                    EventId::new("activate").unwrap(),
                    EventSchema {
                        display_name: "Activate".into(),
                        payload: BTreeMap::new(),
                    },
                );
            }
            let schema = ComponentSchema {
                id: ComponentId::new(name).unwrap(),
                version: 1,
                display_name: name.into(),
                properties: BTreeMap::from([
                    (
                        PropertyId::new("text").unwrap(),
                        PropertySchema {
                            kind: PropertyKind::String,
                            default: Some(PropertyValue::String("Preview text".into())),
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
                        PropertyId::new("enabled").unwrap(),
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
                    (
                        PropertyId::new("amount").unwrap(),
                        PropertySchema {
                            kind: PropertyKind::Number,
                            default: Some(PropertyValue::Number(0.375)),
                            required: false,
                            editable: true,
                            minimum: Some(0.0),
                            maximum: Some(1.0),
                            min_length: None,
                            max_length: None,
                            allowed_values: vec![],
                        },
                    ),
                ]),
                events,
            };
            registry
                .register(schema, move |snapshot, context: &FactoryContext| {
                    let PropertyValue::String(text) =
                        &snapshot.preview.properties[&PropertyId::new("text").unwrap()]
                    else {
                        unreachable!()
                    };
                    let PropertyValue::Boolean(enabled) =
                        &snapshot.preview.properties[&PropertyId::new("enabled").unwrap()]
                    else {
                        unreachable!()
                    };
                    let PropertyValue::Number(amount) =
                        &snapshot.preview.properties[&PropertyId::new("amount").unwrap()]
                    else {
                        unreachable!()
                    };
                    let value_label = format!("{text} · enabled={enabled} · amount={amount}");
                    match name {
                        "text" => AnyView::new(Text(value_label)),
                        "button" => AnyView::new(
                            Button(
                                value_label,
                                context.owner.action_with(
                                    snapshot.signal(EventId::new("activate").unwrap()),
                                    CompiledPreviewGallery::activate,
                                ),
                            )
                            .id("compiled-preview-button"),
                        ),
                        _ => AnyView::new(
                            VStack::new()
                                .spacing(4.0)
                                .child(
                                    TextField(text.clone(), context.text.clone())
                                        .id(format!(
                                            "compiled-preview-text-field-{}",
                                            snapshot.stamp.generation
                                        ))
                                        .accessibility_label("Compiled preview text field"),
                                )
                                .child(Text(format!("enabled={enabled} · amount={amount}"))),
                        ),
                    }
                })
                .unwrap();
        }
        let tap_region_specimen_schema = ComponentSchema {
            id: ComponentId::new("tap-region-specimen").unwrap(),
            version: 1,
            display_name: "TapRegion".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("label").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Tap this text".into())),
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
                    PropertyId::new("enabled").unwrap(),
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
                EventId::new("tap").unwrap(),
                EventSchema {
                    display_name: "Tap".into(),
                    payload: BTreeMap::new(),
                },
            )]),
        };
        registry
            .register(
                tap_region_specimen_schema,
                |snapshot, context: &FactoryContext| {
                    let label =
                        property_string(snapshot, "label").expect("validated TapRegion label");
                    let enabled =
                        property_boolean(snapshot, "enabled").expect("validated TapRegion enabled");
                    AnyView::new(
                        TapRegion(
                            format!("compiled-preview-tap-region-{}", snapshot.stamp.generation),
                            Text(label),
                            context.owner.action_with(
                                snapshot.signal(EventId::new("tap").unwrap()),
                                CompiledPreviewGallery::activate,
                            ),
                        )
                        .enabled(enabled),
                    )
                },
            )
            .unwrap();

        let hover_region_specimen_schema = ComponentSchema {
            id: ComponentId::new("hover-region-specimen").unwrap(),
            version: 1,
            display_name: "HoverRegion".into(),
            properties: BTreeMap::from([(
                PropertyId::new("label").unwrap(),
                PropertySchema {
                    kind: PropertyKind::String,
                    default: Some(PropertyValue::String("Hover over this text".into())),
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
                    EventId::new("enter").unwrap(),
                    EventSchema {
                        display_name: "Pointer entered".into(),
                        payload: BTreeMap::new(),
                    },
                ),
                (
                    EventId::new("exit").unwrap(),
                    EventSchema {
                        display_name: "Pointer exited".into(),
                        payload: BTreeMap::new(),
                    },
                ),
            ]),
        };
        registry
            .register(
                hover_region_specimen_schema,
                |snapshot, context: &FactoryContext| {
                    let label =
                        property_string(snapshot, "label").expect("validated HoverRegion label");
                    AnyView::new(
                        HoverRegion(
                            format!(
                                "compiled-preview-hover-region-{}",
                                snapshot.stamp.generation
                            ),
                            Text(label),
                        )
                        .on_enter(context.owner.action_with(
                            snapshot.signal(EventId::new("enter").unwrap()),
                            CompiledPreviewGallery::record_hover_enter,
                        ))
                        .on_exit(context.owner.action_with(
                            snapshot.signal(EventId::new("exit").unwrap()),
                            CompiledPreviewGallery::record_hover_exit,
                        )),
                    )
                },
            )
            .unwrap();

        let toggle_schema = ComponentSchema {
            id: ComponentId::new("toggle").unwrap(),
            version: 1,
            display_name: "Toggle".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("label").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Preview toggle".into())),
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
                    PropertyId::new("checked").unwrap(),
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
            .register(toggle_schema, |snapshot, context: &FactoryContext| {
                let label = property_string(snapshot, "label").expect("validated toggle label");
                AnyView::new(Toggle(label, context.checked.clone()).id(format!(
                    "compiled-preview-toggle-{}",
                    snapshot.stamp.generation
                )))
            })
            .unwrap();
        let checkbox_schema = ComponentSchema {
            id: ComponentId::new("checkbox").unwrap(),
            version: 1,
            display_name: "Checkbox".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("label").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Preview checkbox".into())),
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
                    PropertyId::new("checked").unwrap(),
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
            .register(checkbox_schema, |snapshot, context: &FactoryContext| {
                let label = property_string(snapshot, "label").expect("validated checkbox label");
                AnyView::new(Checkbox(label, context.checked.clone()).id(format!(
                    "compiled-preview-checkbox-{}",
                    snapshot.stamp.generation
                )))
            })
            .unwrap();
        let radio_schema = ComponentSchema {
            id: ComponentId::new("radio").unwrap(),
            version: 1,
            display_name: "Radio".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("label").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Preview radio".into())),
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
                    PropertyId::new("selected").unwrap(),
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
            .register(radio_schema, |snapshot, context: &FactoryContext| {
                let label = property_string(snapshot, "label").expect("validated radio label");
                AnyView::new(Radio(label, context.checked.clone()).id(format!(
                    "compiled-preview-radio-{}",
                    snapshot.stamp.generation
                )))
            })
            .unwrap();
        let color_picker_schema = ComponentSchema {
            id: ComponentId::new("color-picker-specimen").unwrap(),
            version: 1,
            display_name: "ColorPicker".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("label").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Accent color".into())),
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
                    PropertyId::new("has_color").unwrap(),
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
                (
                    PropertyId::new("hue").unwrap(),
                    color_channel_schema(PREVIEW_COLOR_HUE),
                ),
                (
                    PropertyId::new("saturation").unwrap(),
                    color_channel_schema(PREVIEW_COLOR_SATURATION),
                ),
                (
                    PropertyId::new("lightness").unwrap(),
                    color_channel_schema(PREVIEW_COLOR_LIGHTNESS),
                ),
                (
                    PropertyId::new("alpha").unwrap(),
                    color_channel_schema(PREVIEW_COLOR_ALPHA),
                ),
            ]),
            events: BTreeMap::new(),
        };
        registry
            .register(color_picker_schema, |snapshot, context: &FactoryContext| {
                let label =
                    property_string(snapshot, "label").expect("validated ColorPicker label");
                AnyView::new(ColorPicker(label, context.color_value.clone()).id(format!(
                    "compiled-preview-color-picker-{}",
                    snapshot.stamp.generation
                )))
            })
            .unwrap();

        let picker_values = vec!["compact", "comfortable", "spacious"];
        let picker_schema = ComponentSchema {
            id: ComponentId::new("picker").unwrap(),
            version: 1,
            display_name: "Picker".into(),
            properties: BTreeMap::from([(
                PropertyId::new("selection").unwrap(),
                PropertySchema {
                    kind: PropertyKind::String,
                    default: Some(PropertyValue::String("comfortable".into())),
                    required: false,
                    editable: true,
                    minimum: None,
                    maximum: None,
                    min_length: None,
                    max_length: Some(32),
                    allowed_values: picker_values.iter().map(|value| (*value).into()).collect(),
                },
            )]),
            events: BTreeMap::new(),
        };
        registry
            .register(picker_schema, move |_snapshot, context: &FactoryContext| {
                AnyView::new(
                    Picker(
                        "Choose spacing",
                        context.text.clone(),
                        picker_values.clone(),
                    )
                    .id("compiled-preview-picker"),
                )
            })
            .unwrap();
        let combo_box_values = vec!["rust", "swift", "go"];
        let combo_box_schema = ComponentSchema {
            id: ComponentId::new("combo-box-specimen").unwrap(),
            version: 1,
            display_name: "ComboBox".into(),
            properties: BTreeMap::from([(
                PropertyId::new("selection").unwrap(),
                PropertySchema {
                    kind: PropertyKind::String,
                    default: Some(PropertyValue::String("rust".into())),
                    required: false,
                    editable: true,
                    minimum: None,
                    maximum: None,
                    min_length: None,
                    max_length: Some(32),
                    allowed_values: combo_box_values
                        .iter()
                        .map(|value| (*value).into())
                        .collect(),
                },
            )]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                combo_box_schema,
                move |snapshot, context: &FactoryContext| {
                    AnyView::new(
                        ComboBox(
                            "Choose a language",
                            context.text.clone(),
                            combo_box_values.clone(),
                        )
                        .id(format!(
                            "compiled-preview-combo-box-specimen-{}",
                            snapshot.stamp.generation
                        )),
                    )
                },
            )
            .unwrap();
        let tab_view_values = vec!["overview", "activity", "settings"];
        let tab_view_schema = ComponentSchema {
            id: ComponentId::new("tab-view-specimen").unwrap(),
            version: 1,
            display_name: "TabView".into(),
            properties: BTreeMap::from([(
                PropertyId::new("selection").unwrap(),
                PropertySchema {
                    kind: PropertyKind::String,
                    default: Some(PropertyValue::String("overview".into())),
                    required: false,
                    editable: true,
                    minimum: None,
                    maximum: None,
                    min_length: Some(1),
                    max_length: Some(32),
                    allowed_values: tab_view_values
                        .iter()
                        .map(|value| (*value).into())
                        .collect(),
                },
            )]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                tab_view_schema,
                move |snapshot, context: &FactoryContext| {
                    let tabs = [
                        TabItem::new("overview", "Overview", Text("Preview overview content")),
                        TabItem::new("activity", "Activity", Text("Preview activity content")),
                        TabItem::new("settings", "Settings", Text("Preview settings content")),
                    ];
                    AnyView::new(TabView(tabs, context.text.clone()).id(format!(
                        "compiled-preview-tab-view-{}",
                        snapshot.stamp.generation
                    )))
                },
            )
            .unwrap();

        let slider_schema = ComponentSchema {
            id: ComponentId::new("slider").unwrap(),
            version: 1,
            display_name: "Slider".into(),
            properties: BTreeMap::from([(
                PropertyId::new("value").unwrap(),
                PropertySchema {
                    kind: PropertyKind::Number,
                    default: Some(PropertyValue::Number(PREVIEW_SLIDER_DEFAULT)),
                    required: false,
                    editable: true,
                    minimum: Some(f64::from(PREVIEW_SLIDER_MIN)),
                    maximum: Some(f64::from(PREVIEW_SLIDER_MAX)),
                    min_length: None,
                    max_length: None,
                    allowed_values: vec![],
                },
            )]),
            events: BTreeMap::new(),
        };
        registry
            .register(slider_schema, |snapshot, context: &FactoryContext| {
                AnyView::new(
                    Slider(
                        context.slider_value.clone(),
                        PREVIEW_SLIDER_MIN..=PREVIEW_SLIDER_MAX,
                    )
                    .step(PREVIEW_SLIDER_STEP)
                    .accessibility_label("Preview amount")
                    .id(format!(
                        "compiled-preview-slider-{}",
                        snapshot.stamp.generation
                    )),
                )
            })
            .unwrap();
        let progress_schema = ComponentSchema {
            id: ComponentId::new("progress").unwrap(),
            version: 1,
            display_name: "Progress".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("label").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Preview upload".into())),
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
                        kind: PropertyKind::Number,
                        default: Some(PropertyValue::Number(PREVIEW_PROGRESS_DEFAULT)),
                        required: false,
                        editable: true,
                        minimum: Some(PREVIEW_PROGRESS_MIN),
                        maximum: Some(PREVIEW_PROGRESS_MAX),
                        min_length: None,
                        max_length: None,
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("indeterminate").unwrap(),
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
            .register(progress_schema, |snapshot, _context: &FactoryContext| {
                let label = property_string(snapshot, "label").expect("validated progress label");
                let value = property_number(snapshot, "value").expect("validated progress value");
                let indeterminate =
                    property_boolean(snapshot, "indeterminate").expect("validated progress mode");
                AnyView::new(
                    Progress(
                        format!("compiled-preview-progress-{}", snapshot.stamp.generation),
                        progress_control_value(value),
                    )
                    .indeterminate(indeterminate)
                    .accessibility_label(label),
                )
            })
            .unwrap();
        let stepper_schema = ComponentSchema {
            id: ComponentId::new("stepper").unwrap(),
            version: 1,
            display_name: "Stepper".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("label").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Preview count".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(PREVIEW_STEPPER_LABEL_MAX_LENGTH),
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("value").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::Integer,
                        default: Some(PropertyValue::Integer(i64::from(PREVIEW_STEPPER_DEFAULT))),
                        required: false,
                        editable: true,
                        minimum: Some(f64::from(PREVIEW_STEPPER_MIN)),
                        maximum: Some(f64::from(PREVIEW_STEPPER_MAX)),
                        min_length: None,
                        max_length: None,
                        allowed_values: vec![],
                    },
                ),
            ]),
            events: BTreeMap::new(),
        };
        registry
            .register(stepper_schema, |snapshot, context: &FactoryContext| {
                let label = property_string(snapshot, "label").expect("validated Stepper label");
                AnyView::new(
                    Stepper(label, context.stepper_value.clone())
                        .range(PREVIEW_STEPPER_MIN..=PREVIEW_STEPPER_MAX)
                        .step(PREVIEW_STEPPER_STEP)
                        .id(format!(
                            "compiled-preview-stepper-{}",
                            snapshot.stamp.generation
                        )),
                )
            })
            .unwrap();
        let text_specimen_schema = ComponentSchema {
            id: ComponentId::new("text-specimen").unwrap(),
            version: 1,
            display_name: "Text".into(),
            properties: BTreeMap::from([(
                PropertyId::new("value").unwrap(),
                PropertySchema {
                    kind: PropertyKind::String,
                    default: Some(PropertyValue::String("Preview text".into())),
                    required: false,
                    editable: true,
                    minimum: None,
                    maximum: None,
                    min_length: Some(1),
                    max_length: Some(PREVIEW_TEXT_VALUE_MAX_LENGTH),
                    allowed_values: vec![],
                },
            )]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                text_specimen_schema,
                |snapshot, _context: &FactoryContext| {
                    let value = property_string(snapshot, "value").expect("validated Text value");
                    AnyView::new(Text(value))
                },
            )
            .unwrap();

        let localized_text_specimen_schema = ComponentSchema {
            id: ComponentId::new("localized-text-specimen").unwrap(),
            version: 1,
            display_name: "LocalizedText".into(),
            properties: BTreeMap::from([(
                PropertyId::new("message_id").unwrap(),
                PropertySchema {
                    kind: PropertyKind::String,
                    default: Some(PropertyValue::String(PREVIEW_LOCALIZED_TEXT_DEFAULT.into())),
                    required: false,
                    editable: true,
                    minimum: None,
                    maximum: None,
                    min_length: Some(1),
                    max_length: Some(40),
                    allowed_values: PREVIEW_LOCALIZED_TEXT_KEYS
                        .iter()
                        .map(|key| (*key).into())
                        .collect(),
                },
            )]),
            events: BTreeMap::new(),
        };
        registry
            .register(
                localized_text_specimen_schema,
                |snapshot, _context: &FactoryContext| {
                    AnyView::new(LocalizedText(localized_text_message_id(snapshot)))
                },
            )
            .unwrap();
        let button_specimen_schema = ComponentSchema {
            id: ComponentId::new("button-specimen").unwrap(),
            version: 1,
            display_name: "Button".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("label").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Preview action".into())),
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
                    PropertyId::new("enabled").unwrap(),
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
                EventId::new("activate").unwrap(),
                EventSchema {
                    display_name: "Activate".into(),
                    payload: BTreeMap::new(),
                },
            )]),
        };
        registry
            .register(
                button_specimen_schema,
                |snapshot, context: &FactoryContext| {
                    let label = property_string(snapshot, "label").expect("validated button label");
                    let enabled =
                        property_boolean(snapshot, "enabled").expect("validated button enabled");
                    AnyView::new(
                        Button(
                            label,
                            context.owner.action_with(
                                snapshot.signal(EventId::new("activate").unwrap()),
                                CompiledPreviewGallery::activate,
                            ),
                        )
                        .disabled(!enabled)
                        .id(format!(
                            "compiled-preview-button-specimen-{}",
                            snapshot.stamp.generation
                        )),
                    )
                },
            )
            .unwrap();
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
                    let source = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join("../../assets/playground-sample.png");
                    AnyView::new(
                        Image(source)
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
        let tree_specimen_schema = ComponentSchema {
            id: ComponentId::new("tree-specimen").unwrap(),
            version: 1,
            display_name: "Tree".into(),
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
        registry
            .register(
                tree_specimen_schema,
                |snapshot, context: &FactoryContext| {
                    let root_label =
                        property_string(snapshot, "root_label").expect("validated Tree root label");
                    let src_label =
                        property_string(snapshot, "src_label").expect("validated Tree src label");
                    let readme_label = property_string(snapshot, "readme_label")
                        .expect("validated Tree README label");
                    let tests_label = property_string(snapshot, "tests_label")
                        .expect("validated Tree tests label");
                    let selected_id =
                        property_string(snapshot, "selected_id").expect("validated Tree selection");
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
                    AnyView::new(rustui::components::data::tree::Tree(nodes, selection).id(
                        format!("compiled-preview-tree-{}", snapshot.stamp.generation),
                    ))
                },
            )
            .unwrap();
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
        let mut result = Self {
            host: PreviewHost::new(registry),
            stamp_lease: Rc::new(RefCell::new(None)),
            text: "Preview text".into(),
            text_field_placeholder: "Preview text input".into(),
            search_field_placeholder: "Search RustUI".into(),
            text_editor_placeholder: "Write a note".into(),
            secure_field_placeholder: "Enter a password".into(),
            secure_value: String::new(),
            token_values: Vec::new(),
            token_field_placeholder: "Add recipients".into(),
            enabled: true,
            amount: 0.375,
            amount_draft: "0.375".into(),
            number_drafts: BTreeMap::new(),
            stepper_count: PREVIEW_STEPPER_DEFAULT,
            stale_amount_callback: None,
            activations: 0,
            renders: 0,
            status: "Mounted TextField".into(),
            descriptor_json: String::new(),
        };
        result.mount("text-field");
        result.export_descriptor();
        result
    }
}

impl CompiledPreviewGallery {
    fn mount(&mut self, component: &str) {
        // Secure values are ephemeral and never enter the serializable preview
        // document. A remount clears the previous field value.
        self.secure_value.clear();
        let mut document = PreviewDocument::new(
            PreviewId::new("compiled-gallery").unwrap(),
            ComponentId::new(component).unwrap(),
            1,
        );
        if component == "color-picker-specimen" {
            document.properties.extend([
                (
                    PropertyId::new("label").unwrap(),
                    PropertyValue::String("Accent color".into()),
                ),
                (
                    PropertyId::new("has_color").unwrap(),
                    PropertyValue::Boolean(true),
                ),
                (
                    PropertyId::new("hue").unwrap(),
                    PropertyValue::Number(PREVIEW_COLOR_HUE),
                ),
                (
                    PropertyId::new("saturation").unwrap(),
                    PropertyValue::Number(PREVIEW_COLOR_SATURATION),
                ),
                (
                    PropertyId::new("lightness").unwrap(),
                    PropertyValue::Number(PREVIEW_COLOR_LIGHTNESS),
                ),
                (
                    PropertyId::new("alpha").unwrap(),
                    PropertyValue::Number(PREVIEW_COLOR_ALPHA),
                ),
            ]);
        } else if component == "picker" {
            self.text = "comfortable".into();
            document.properties.insert(
                PropertyId::new("selection").unwrap(),
                PropertyValue::String(self.text.clone()),
            );
        } else if component == "combo-box-specimen" {
            self.text = "rust".into();
            document.properties.insert(
                PropertyId::new("selection").unwrap(),
                PropertyValue::String(self.text.clone()),
            );
        } else if component == "token-field-specimen" {
            self.token_values = PREVIEW_TOKEN_FIELD_DEFAULTS
                .iter()
                .map(|token| (*token).to_owned())
                .collect();
            self.token_field_placeholder = "Add recipients".into();
            self.text = "Engineering".into();
            document.properties.extend([
                (
                    PropertyId::new("placeholder").unwrap(),
                    PropertyValue::String(self.token_field_placeholder.clone()),
                ),
                (
                    PropertyId::new("tokens_json").unwrap(),
                    PropertyValue::String(serde_json::to_string(&self.token_values).unwrap()),
                ),
                (
                    PropertyId::new("draft").unwrap(),
                    PropertyValue::String(self.text.clone()),
                ),
            ]);
        } else if component == "tab-view-specimen" {
            self.text = "overview".into();
            document.properties.insert(
                PropertyId::new("selection").unwrap(),
                PropertyValue::String(self.text.clone()),
            );
        } else if component == "stepper" {
            self.stepper_count = PREVIEW_STEPPER_DEFAULT;
            document.properties.insert(
                PropertyId::new("value").unwrap(),
                PropertyValue::Integer(i64::from(self.stepper_count)),
            );
        } else if component == "slider" {
            self.amount = PREVIEW_SLIDER_DEFAULT;
            self.amount_draft = self.amount.to_string();
            document.properties.insert(
                PropertyId::new("value").unwrap(),
                PropertyValue::Number(self.amount),
            );
        } else if component == "text-field-specimen" {
            self.text = "Preview text input".into();
            self.text_field_placeholder = "Preview text input".into();
            document.properties.insert(
                PropertyId::new("placeholder").unwrap(),
                PropertyValue::String(self.text_field_placeholder.clone()),
            );
            document.properties.insert(
                PropertyId::new("value").unwrap(),
                PropertyValue::String(self.text.clone()),
            );
        } else if component == "search-field-specimen" {
            self.text = "Search RustUI".into();
            self.search_field_placeholder = "Search RustUI".into();
            document.properties.insert(
                PropertyId::new("placeholder").unwrap(),
                PropertyValue::String(self.search_field_placeholder.clone()),
            );
            document.properties.insert(
                PropertyId::new("query").unwrap(),
                PropertyValue::String(self.text.clone()),
            );
        } else if component == "text-editor-specimen" {
            self.text = "First line\nSecond line".into();
            self.text_editor_placeholder = "Write a note".into();
            document.properties.insert(
                PropertyId::new("placeholder").unwrap(),
                PropertyValue::String(self.text_editor_placeholder.clone()),
            );
            document.properties.insert(
                PropertyId::new("value").unwrap(),
                PropertyValue::String(self.text.clone()),
            );
        } else if component == "code-editor-specimen" {
            self.text = "Plain text preview".into();
            document.properties.insert(
                PropertyId::new("value").unwrap(),
                PropertyValue::String(self.text.clone()),
            );
        } else if component == "secure-field-specimen" {
            self.secure_field_placeholder = "Enter a password".into();
            document.properties.insert(
                PropertyId::new("placeholder").unwrap(),
                PropertyValue::String(self.secure_field_placeholder.clone()),
            );
        } else if component == "progress" {
            self.text = "Preview upload".into();
            self.amount = PREVIEW_PROGRESS_DEFAULT;
            self.amount_draft = self.amount.to_string();
            document.properties.insert(
                PropertyId::new("label").unwrap(),
                PropertyValue::String(self.text.clone()),
            );
            document.properties.insert(
                PropertyId::new("value").unwrap(),
                PropertyValue::Number(self.amount),
            );
        } else if component == "text-specimen" {
            document.properties.insert(
                PropertyId::new("value").unwrap(),
                PropertyValue::String("Preview text".into()),
            );
        } else if component == "button-specimen" {
            self.text = "Preview action".into();
            self.enabled = true;
            document.properties.insert(
                PropertyId::new("label").unwrap(),
                PropertyValue::String(self.text.clone()),
            );
            document.properties.insert(
                PropertyId::new("enabled").unwrap(),
                PropertyValue::Boolean(self.enabled),
            );
        } else if component == "tap-region-specimen" {
            self.text = "Tap this text".into();
            self.enabled = true;
            document.properties.insert(
                PropertyId::new("label").unwrap(),
                PropertyValue::String(self.text.clone()),
            );
            document.properties.insert(
                PropertyId::new("enabled").unwrap(),
                PropertyValue::Boolean(self.enabled),
            );
        } else if component == "hover-region-specimen" {
            document.properties.insert(
                PropertyId::new("label").unwrap(),
                PropertyValue::String("Hover over this text".into()),
            );
        } else if component == "toolbar-specimen" {
            document.properties.extend([
                (
                    PropertyId::new("primary_label").unwrap(),
                    PropertyValue::String("Save".into()),
                ),
                (
                    PropertyId::new("secondary_label").unwrap(),
                    PropertyValue::String("Cancel".into()),
                ),
                (
                    PropertyId::new("secondary_disabled").unwrap(),
                    PropertyValue::Boolean(false),
                ),
            ]);
        } else if component == "image-specimen" {
            document.properties.extend([
                (
                    PropertyId::new("accessibility_label").unwrap(),
                    PropertyValue::String("Purple and cyan sample image".into()),
                ),
                (
                    PropertyId::new("fit").unwrap(),
                    PropertyValue::String("contain".into()),
                ),
            ]);
        } else if component == "breadcrumb-specimen" {
            document.properties.extend([
                (
                    PropertyId::new("parent_label").unwrap(),
                    PropertyValue::String("Components".into()),
                ),
                (
                    PropertyId::new("current_label").unwrap(),
                    PropertyValue::String("Image".into()),
                ),
            ]);
        } else if component == "context-menu-specimen" {
            document.properties.extend([
                (
                    PropertyId::new("target_label").unwrap(),
                    PropertyValue::String("Preview target".into()),
                ),
                (
                    PropertyId::new("action_label").unwrap(),
                    PropertyValue::String("Open preview".into()),
                ),
            ]);
        } else if component == "menu-specimen" {
            self.text = "Preview menu".into();
            document.properties.insert(
                PropertyId::new("trigger_label").unwrap(),
                PropertyValue::String(self.text.clone()),
            );
        } else if component == "link-specimen" {
            document.properties.extend([
                (
                    PropertyId::new("label").unwrap(),
                    PropertyValue::String("RustUI documentation".into()),
                ),
                (
                    PropertyId::new("url").unwrap(),
                    PropertyValue::String("https://example.com/docs".into()),
                ),
            ]);
        } else if component == "dialog-specimen" {
            self.text = "Open preview dialog".into();
            document.properties.insert(
                PropertyId::new("trigger_label").unwrap(),
                PropertyValue::String(self.text.clone()),
            );
        } else if component == "popover-specimen" {
            self.text = "Preview details".into();
            self.enabled = false;
            document.properties.extend([
                (
                    PropertyId::new("trigger_label").unwrap(),
                    PropertyValue::String(self.text.clone()),
                ),
                (
                    PropertyId::new("open").unwrap(),
                    PropertyValue::Boolean(self.enabled),
                ),
            ]);
        } else if component == "badge-specimen" {
            document.properties.extend([
                (
                    PropertyId::new("label").unwrap(),
                    PropertyValue::String("Inbox".into()),
                ),
                (PropertyId::new("count").unwrap(), PropertyValue::Integer(7)),
                (
                    PropertyId::new("dot").unwrap(),
                    PropertyValue::Boolean(false),
                ),
            ]);
        } else if component == "empty-state-specimen" {
            document.properties.extend([
                (
                    PropertyId::new("title").unwrap(),
                    PropertyValue::String("No results".into()),
                ),
                (
                    PropertyId::new("detail").unwrap(),
                    PropertyValue::String("Try changing your filters.".into()),
                ),
            ]);
        } else if component == "error-state-specimen" {
            document.properties.extend([
                (
                    PropertyId::new("title").unwrap(),
                    PropertyValue::String("Could not load".into()),
                ),
                (
                    PropertyId::new("detail").unwrap(),
                    PropertyValue::String("Check your connection and try again.".into()),
                ),
            ]);
        } else if component == "alert-specimen" {
            self.text = "Connection lost".into();
            self.enabled = true;
            document.properties.extend([
                (
                    PropertyId::new("message").unwrap(),
                    PropertyValue::String(self.text.clone()),
                ),
                (
                    PropertyId::new("visible").unwrap(),
                    PropertyValue::Boolean(self.enabled),
                ),
            ]);
        } else if component == "toast-specimen" {
            self.text = "Changes saved".into();
            document.properties.insert(
                PropertyId::new("message").unwrap(),
                PropertyValue::String(self.text.clone()),
            );
        } else if component == "sheet-specimen" {
            self.text = "Open details".into();
            document.properties.extend([
                (
                    PropertyId::new("trigger_label").unwrap(),
                    PropertyValue::String(self.text.clone()),
                ),
                (
                    PropertyId::new("title").unwrap(),
                    PropertyValue::String("Details".into()),
                ),
                (
                    PropertyId::new("body_text").unwrap(),
                    PropertyValue::String("Preview sheet content".into()),
                ),
            ]);
        } else if component == "sidebar-specimen" {
            document.properties.extend([
                (
                    PropertyId::new("home_label").unwrap(),
                    PropertyValue::String("Home".into()),
                ),
                (
                    PropertyId::new("settings_label").unwrap(),
                    PropertyValue::String("Settings".into()),
                ),
                (
                    PropertyId::new("selected_id").unwrap(),
                    PropertyValue::String("home".into()),
                ),
            ]);
        } else if component == "keyed-list-specimen" {
            document.properties.extend([
                (
                    PropertyId::new("first_label").unwrap(),
                    PropertyValue::String("First item".into()),
                ),
                (
                    PropertyId::new("second_label").unwrap(),
                    PropertyValue::String("Second item".into()),
                ),
                (
                    PropertyId::new("selected_id").unwrap(),
                    PropertyValue::String("first".into()),
                ),
            ]);
        } else if component == "data-grid-specimen" {
            document.properties.extend([
                (
                    PropertyId::new("label").unwrap(),
                    PropertyValue::String("Quarterly revenue".into()),
                ),
                (
                    PropertyId::new("selected_id").unwrap(),
                    PropertyValue::String("none".into()),
                ),
            ]);
        } else if component == "list-specimen" {
            document.properties.insert(
                PropertyId::new("selected_id").unwrap(),
                PropertyValue::String("none".into()),
            );
        } else if component == "table-specimen" {
            document.properties.insert(
                PropertyId::new("selected_id").unwrap(),
                PropertyValue::String("none".into()),
            );
        } else if component == "tree-specimen" {
            document.properties.extend([
                (
                    PropertyId::new("root_label").unwrap(),
                    PropertyValue::String("Project".into()),
                ),
                (
                    PropertyId::new("src_label").unwrap(),
                    PropertyValue::String("src".into()),
                ),
                (
                    PropertyId::new("readme_label").unwrap(),
                    PropertyValue::String("README.md".into()),
                ),
                (
                    PropertyId::new("tests_label").unwrap(),
                    PropertyValue::String("tests".into()),
                ),
                (
                    PropertyId::new("selected_id").unwrap(),
                    PropertyValue::String("none".into()),
                ),
            ]);
        } else if component == "navigation-split-specimen" {
            document.properties.insert(
                PropertyId::new("selected_id").unwrap(),
                PropertyValue::String("none".into()),
            );
        } else if component == "navigation-stack-specimen" {
            document.properties.insert(
                PropertyId::new("selected_id").unwrap(),
                PropertyValue::String("none".into()),
            );
        } else if component == "collection-specimen" {
            document.properties.insert(
                PropertyId::new("selected_id").unwrap(),
                PropertyValue::String("none".into()),
            );
        } else if component == "canvas-specimen" {
            document.properties.insert(
                PropertyId::new("composition").unwrap(),
                PropertyValue::String("basic".into()),
            );
        } else if component == "localized-text-specimen" {
            document.properties.insert(
                PropertyId::new("message_id").unwrap(),
                PropertyValue::String(PREVIEW_LOCALIZED_TEXT_DEFAULT.into()),
            );
        } else if component == "disclosure-specimen" {
            self.text = "More information".into();
            self.enabled = false;
            document.properties.extend([
                (
                    PropertyId::new("label").unwrap(),
                    PropertyValue::String(self.text.clone()),
                ),
                (
                    PropertyId::new("open").unwrap(),
                    PropertyValue::Boolean(self.enabled),
                ),
            ]);
        } else if component == "icon-specimen" {
            document.properties.insert(
                PropertyId::new("accessibility_label").unwrap(),
                PropertyValue::String("Search".into()),
            );
        } else if component == "date-picker-specimen" {
            // Label and empty optional date come from the catalog defaults.
        } else if component == "avatar-specimen" {
            document.properties.extend([
                (
                    PropertyId::new("name").unwrap(),
                    PropertyValue::String("Ada Lovelace".into()),
                ),
                (
                    PropertyId::new("diameter").unwrap(),
                    PropertyValue::Number(40.0),
                ),
            ]);
        } else if matches!(
            component,
            "animated-opacity-specimen"
                | "spring-opacity-specimen"
                | "group-specimen"
                | "shadow-specimen"
                | "background-specimen"
                | "border-specimen"
                | "semantics-specimen"
                | "card-specimen"
                | "container-specimen"
                | "split-view-specimen"
                | "frame-specimen"
                | "hstack-specimen"
                | "vstack-specimen"
                | "grid-specimen"
                | "lazy-grid-specimen"
                | "spacer-specimen"
                | "zstack-specimen"
                | "geometry-reader-specimen"
                | "property-inspector-specimen"
        ) {
            // These specimens take their initial values from the schema defaults.
        } else if component == "scroll-view-specimen" {
            // The specimen's initial properties come from its bounded schema defaults.
        }

        let component_schema = self.host.catalog().components.get(&document.component_id);
        let mut seed_generic_property = |name: &str, value: PropertyValue| {
            let property = PropertyId::new(name).unwrap();
            if component_schema.is_some_and(|schema| schema.properties.contains_key(&property)) {
                document.properties.entry(property).or_insert(value);
            }
        };
        if matches!(component, "toggle" | "checkbox" | "radio") {
            let value_property = if component == "radio" {
                "selected"
            } else {
                "checked"
            };
            seed_generic_property("label", PropertyValue::String(self.text.clone()));
            seed_generic_property(value_property, PropertyValue::Boolean(self.enabled));
        } else {
            seed_generic_property("text", PropertyValue::String(self.text.clone()));
            seed_generic_property("enabled", PropertyValue::Boolean(self.enabled));
            seed_generic_property("amount", PropertyValue::Number(self.amount));
        }
        if matches!(component, "button" | "button-specimen") {
            document.actions.insert(
                EventId::new("activate").unwrap(),
                ActionRef::new("increment").unwrap(),
            );
        }
        if component == "tap-region-specimen" {
            document.actions.insert(
                EventId::new("tap").unwrap(),
                ActionRef::new("increment").unwrap(),
            );
        }
        if component == "hover-region-specimen" {
            document.actions.extend([
                (
                    EventId::new("enter").unwrap(),
                    ActionRef::new("increment").unwrap(),
                ),
                (
                    EventId::new("exit").unwrap(),
                    ActionRef::new("increment").unwrap(),
                ),
            ]);
        }
        if component == "menu-specimen" {
            document.actions.insert(
                EventId::new("select").unwrap(),
                ActionRef::new("increment").unwrap(),
            );
        }
        if component == "link-specimen" {
            document.actions.insert(
                EventId::new("activate").unwrap(),
                ActionRef::new("increment").unwrap(),
            );
        }
        if component == "alert-specimen" {
            document.actions.insert(
                EventId::new("dismiss").unwrap(),
                ActionRef::new("dismiss").unwrap(),
            );
        }
        if component == "disclosure-specimen" {
            document.actions.insert(
                EventId::new("toggle").unwrap(),
                ActionRef::new("toggle").unwrap(),
            );
        }
        if component == "toolbar-specimen" {
            document.actions.extend([
                (
                    EventId::new("primary").unwrap(),
                    ActionRef::new("toolbar_primary").unwrap(),
                ),
                (
                    EventId::new("secondary").unwrap(),
                    ActionRef::new("toolbar_secondary").unwrap(),
                ),
            ]);
        }
        if component == "breadcrumb-specimen" {
            document.actions.insert(
                EventId::new("parent").unwrap(),
                ActionRef::new("breadcrumb_parent").unwrap(),
            );
        }
        if component == "context-menu-specimen" {
            document.actions.insert(
                EventId::new("activate").unwrap(),
                ActionRef::new("context_menu_open_preview").unwrap(),
            );
        }
        if component == "toast-specimen" {
            document.actions.insert(
                EventId::new("show").unwrap(),
                ActionRef::new("toast_show").unwrap(),
            );
        }
        if component == "dialog-specimen" {
            for event in ["confirm", "cancel"] {
                document.actions.insert(
                    EventId::new(event).unwrap(),
                    ActionRef::new("increment").unwrap(),
                );
            }
        }
        self.status = match self.host.mount(document) {
            Ok(stamp) => {
                // A new lease per generation keeps callbacks from an older mount stale.
                self.stamp_lease = Rc::new(RefCell::new(Some(stamp.clone())));
                self.refresh_number_drafts();
                format!("Mounted {component}, generation {}", stamp.generation)
            }
            Err(error) => error.to_string(),
        };
    }
    fn mount_text(&mut self) {
        self.mount("text");
    }
    fn mount_text_specimen(&mut self) {
        self.mount("text-specimen");
    }

    fn mount_localized_text_specimen(&mut self) {
        self.mount("localized-text-specimen");
    }
    fn mount_button(&mut self) {
        self.mount("button");
    }
    fn mount_field(&mut self) {
        self.mount("text-field");
    }
    fn mount_toggle(&mut self) {
        self.mount("toggle");
    }
    fn mount_checkbox(&mut self) {
        self.mount("checkbox");
    }
    fn mount_radio(&mut self) {
        self.mount("radio");
    }
    fn mount_picker(&mut self) {
        self.mount("picker");
    }
    fn mount_combo_box_specimen(&mut self) {
        self.mount("combo-box-specimen");
    }
    fn mount_tab_view_specimen(&mut self) {
        self.mount("tab-view-specimen");
    }
    fn mount_slider(&mut self) {
        self.mount("slider");
    }

    fn mount_progress(&mut self) {
        self.mount("progress");
    }

    fn mount_stepper(&mut self) {
        self.mount("stepper");
    }

    fn mount_text_field_specimen(&mut self) {
        self.mount("text-field-specimen");
    }
    fn mount_search_field_specimen(&mut self) {
        self.mount("search-field-specimen");
    }
    fn mount_token_field_specimen(&mut self) {
        self.mount("token-field-specimen");
    }
    fn mount_text_editor_specimen(&mut self) {
        self.mount("text-editor-specimen");
    }
    fn mount_code_editor_specimen(&mut self) {
        self.mount("code-editor-specimen");
    }

    fn mount_secure_field_specimen(&mut self) {
        self.mount("secure-field-specimen");
    }
    fn mount_button_specimen(&mut self) {
        self.mount("button-specimen");
    }
    fn mount_toolbar_specimen(&mut self) {
        self.mount("toolbar-specimen");
    }
    fn mount_image_specimen(&mut self) {
        self.mount("image-specimen");
    }
    fn mount_breadcrumb_specimen(&mut self) {
        self.mount("breadcrumb-specimen");
    }
    fn mount_context_menu_specimen(&mut self) {
        self.mount("context-menu-specimen");
    }
    fn mount_menu_specimen(&mut self) {
        self.mount("menu-specimen");
    }
    fn mount_link_specimen(&mut self) {
        self.mount("link-specimen");
    }
    fn mount_alert_specimen(&mut self) {
        self.mount("alert-specimen");
    }
    fn mount_toast_specimen(&mut self) {
        self.mount("toast-specimen");
    }
    fn mount_sheet_specimen(&mut self) {
        self.mount("sheet-specimen");
    }
    fn mount_sidebar_specimen(&mut self) {
        self.mount("sidebar-specimen");
    }
    fn mount_keyed_list_specimen(&mut self) {
        self.mount("keyed-list-specimen");
    }

    fn mount_data_grid_specimen(&mut self) {
        self.mount("data-grid-specimen");
    }

    fn mount_list_specimen(&mut self) {
        self.mount("list-specimen");
    }

    fn mount_table_specimen(&mut self) {
        self.mount("table-specimen");
    }

    fn mount_tree_specimen(&mut self) {
        self.mount("tree-specimen");
    }
    fn mount_navigation_split_specimen(&mut self) {
        self.mount("navigation-split-specimen");
    }

    fn mount_navigation_stack_specimen(&mut self) {
        self.mount("navigation-stack-specimen");
    }

    fn mount_collection_specimen(&mut self) {
        self.mount("collection-specimen");
    }
    fn mount_canvas_specimen(&mut self) {
        self.mount("canvas-specimen");
    }
    fn mount_disclosure_specimen(&mut self) {
        self.mount("disclosure-specimen");
    }
    fn mount_dialog_specimen(&mut self) {
        self.mount("dialog-specimen");
    }
    fn mount_popover_specimen(&mut self) {
        self.mount("popover-specimen");
    }
    fn mount_date_picker_specimen(&mut self) {
        self.mount("date-picker-specimen");
    }

    fn mount_property_inspector_specimen(&mut self) {
        self.mount("property-inspector-specimen");
    }

    fn mount_group_specimen(&mut self) {
        self.mount("group-specimen");
    }

    fn mount_animated_opacity_specimen(&mut self) {
        self.mount("animated-opacity-specimen");
    }

    fn mount_spring_opacity_specimen(&mut self) {
        self.mount("spring-opacity-specimen");
    }

    fn mount_shadow_specimen(&mut self) {
        self.mount("shadow-specimen");
    }

    fn mount_background_specimen(&mut self) {
        self.mount("background-specimen");
    }

    fn mount_border_specimen(&mut self) {
        self.mount("border-specimen");
    }

    fn mount_semantics_specimen(&mut self) {
        self.mount("semantics-specimen");
    }

    fn mount_card_specimen(&mut self) {
        self.mount("card-specimen");
    }

    fn mount_container_specimen(&mut self) {
        self.mount("container-specimen");
    }

    fn mount_split_view_specimen(&mut self) {
        self.mount("split-view-specimen");
    }

    fn mount_frame_specimen(&mut self) {
        self.mount("frame-specimen");
    }

    fn mount_hstack_specimen(&mut self) {
        self.mount("hstack-specimen");
    }

    fn mount_vstack_specimen(&mut self) {
        self.mount("vstack-specimen");
    }

    fn mount_grid_specimen(&mut self) {
        self.mount("grid-specimen");
    }

    fn mount_lazy_grid_specimen(&mut self) {
        self.mount("lazy-grid-specimen");
    }

    fn mount_spacer_specimen(&mut self) {
        self.mount("spacer-specimen");
    }

    fn mount_geometry_reader_specimen(&mut self) {
        self.mount("geometry-reader-specimen");
    }

    #[cfg(test)]
    fn mount_scroll_view_specimen(&mut self) {
        self.mount("scroll-view-specimen");
    }
    fn mount_avatar_specimen(&mut self) {
        self.mount("avatar-specimen");
    }
    fn mount_empty_state_specimen(&mut self) {
        self.mount("empty-state-specimen");
    }
    fn mount_error_state_specimen(&mut self) {
        self.mount("error-state-specimen");
    }

    fn mount_badge_specimen(&mut self) {
        self.mount("badge-specimen");
    }

    fn mount_label_specimen(&mut self) {
        self.mount("label-specimen");
    }

    fn mount_tooltip_specimen(&mut self) {
        self.mount("tooltip-specimen");
    }

    fn mount_spinner_specimen(&mut self) {
        self.mount("spinner-specimen");
    }

    fn mount_divider_specimen(&mut self) {
        self.mount("divider-specimen");
    }

    fn mount_skeleton_specimen(&mut self) {
        self.mount("skeleton-specimen");
    }
    fn mount_icon_specimen(&mut self) {
        self.mount("icon-specimen");
    }
    fn mount_color_picker_specimen(&mut self) {
        self.mount("color-picker-specimen");
    }
    fn rerender(&mut self) {
        self.renders += 1;
    }

    fn finish_delayed_rerender(&mut self, state: AsyncState<(), String>) -> bool {
        match state {
            AsyncState::Success(()) => {
                self.renders += 1;
                self.status = "Completed delayed parent rerender".into();
                true
            }
            AsyncState::Error(error) => {
                self.status = format!("Delayed parent rerender failed: {error}");
                true
            }
            AsyncState::Idle | AsyncState::Loading => false,
        }
    }
    fn export_descriptor(&mut self) {
        let Some(snapshot) = self.host.snapshot() else {
            self.status = "No mounted preview to export".into();
            return;
        };
        // The descriptor is the editable/importable representation; export
        // resolved values from the live host so accepted binding feedback is
        // included even if a compiled factory rendered an older snapshot.
        let mut document = snapshot.preview.document.clone();
        document.properties.clone_from(&snapshot.preview.properties);
        match encode_preview(&document, self.host.catalog()) {
            Ok(json) => {
                self.descriptor_json = json;
                self.status = "Exported normalized preview descriptor".into();
            }
            Err(error) => self.status = format!("Export rejected: {error}"),
        }
    }
    fn import_descriptor(&mut self) {
        let parsed = match decode_preview(&self.descriptor_json, self.host.catalog()) {
            Ok(preview) => preview.document,
            Err(error) => {
                self.status = format!("Import rejected: {error}");
                return;
            }
        };
        if parsed.component_id.as_str() == "date-picker-specimen"
            && let Some(PropertyValue::String(value)) =
                parsed.properties.get(&PropertyId::new("date").unwrap())
            && parse_preview_date(value).is_err()
        {
            self.status =
                "Import rejected: date must be empty or a canonical YYYY-MM-DD date".into();
            return;
        }
        if parsed.component_id.as_str() == "tooltip-specimen"
            && let Some(PropertyValue::String(text)) =
                parsed.properties.get(&PropertyId::new("text").unwrap())
            && text.trim().is_empty()
        {
            self.status = "Import rejected: Tooltip text must not be blank".into();
            return;
        }
        if parsed.component_id.as_str() == "stepper"
            && let Some(PropertyValue::String(label)) =
                parsed.properties.get(&PropertyId::new("label").unwrap())
            && (label.trim().is_empty() || label.len() > PREVIEW_STEPPER_LABEL_MAX_LENGTH)
        {
            self.status = format!(
                "Import rejected: Stepper label must be nonblank and at most {PREVIEW_STEPPER_LABEL_MAX_LENGTH} bytes"
            );
            return;
        }
        if parsed.component_id.as_str() == "sidebar-specimen" {
            for name in ["home_label", "settings_label"] {
                if let Some(PropertyValue::String(label)) =
                    parsed.properties.get(&PropertyId::new(name).unwrap())
                    && label.trim().is_empty()
                {
                    self.status = format!("Import rejected: Sidebar {name} must not be blank");
                    return;
                }
            }
        }
        if parsed.component_id.as_str() == "data-grid-specimen"
            && let Some(PropertyValue::String(label)) =
                parsed.properties.get(&PropertyId::new("label").unwrap())
            && label.trim().is_empty()
        {
            self.status = "Import rejected: DataGrid label must not be blank".into();
            return;
        }
        if parsed.component_id.as_str() == "tree-specimen" {
            for name in ["root_label", "src_label", "readme_label", "tests_label"] {
                if let Some(PropertyValue::String(label)) =
                    parsed.properties.get(&PropertyId::new(name).unwrap())
                    && label.trim().is_empty()
                {
                    self.status = format!("Import rejected: Tree {name} must not be blank");
                    return;
                }
            }
        }
        let uses_label_checked = matches!(parsed.component_id.as_str(), "toggle" | "checkbox");
        let uses_button_specimen = parsed.component_id.as_str() == "button-specimen";
        let uses_tap_region_specimen = parsed.component_id.as_str() == "tap-region-specimen";
        let uses_hover_region_specimen = parsed.component_id.as_str() == "hover-region-specimen";
        let uses_menu_specimen = parsed.component_id.as_str() == "menu-specimen";
        let uses_dialog_specimen = parsed.component_id.as_str() == "dialog-specimen";
        let uses_popover_specimen = parsed.component_id.as_str() == "popover-specimen";
        let uses_alert_specimen = parsed.component_id.as_str() == "alert-specimen";
        let uses_toast_specimen = parsed.component_id.as_str() == "toast-specimen";
        let uses_sheet_specimen = parsed.component_id.as_str() == "sheet-specimen";
        let uses_disclosure_specimen = parsed.component_id.as_str() == "disclosure-specimen";
        let uses_toolbar_specimen = parsed.component_id.as_str() == "toolbar-specimen";
        let uses_image_specimen = parsed.component_id.as_str() == "image-specimen";
        let uses_breadcrumb_specimen = parsed.component_id.as_str() == "breadcrumb-specimen";
        let uses_context_menu_specimen = parsed.component_id.as_str() == "context-menu-specimen";
        let uses_text_specimen = parsed.component_id.as_str() == "text-specimen";
        let uses_radio = parsed.component_id.as_str() == "radio";
        let uses_selection = matches!(
            parsed.component_id.as_str(),
            "picker" | "combo-box-specimen" | "tab-view-specimen"
        );
        let uses_slider = parsed.component_id.as_str() == "slider";
        let uses_progress = parsed.component_id.as_str() == "progress";
        let uses_stepper = parsed.component_id.as_str() == "stepper";
        let uses_text_field_specimen = parsed.component_id.as_str() == "text-field-specimen";
        let uses_search_field_specimen = parsed.component_id.as_str() == "search-field-specimen";
        let uses_text_editor_specimen = parsed.component_id.as_str() == "text-editor-specimen";
        let uses_code_editor_specimen = parsed.component_id.as_str() == "code-editor-specimen";
        let uses_secure_field_specimen = parsed.component_id.as_str() == "secure-field-specimen";
        let uses_token_field = parsed.component_id.as_str() == "token-field-specimen";
        let uses_color_picker_specimen = parsed.component_id.as_str() == "color-picker-specimen";
        if uses_token_field
            && let Some(PropertyValue::String(tokens_json)) = parsed
                .properties
                .get(&PropertyId::new("tokens_json").unwrap())
            && parse_preview_token_values(tokens_json).is_err()
        {
            self.status =
                "Import rejected: TokenField tokens_json must be a JSON string array".into();
            return;
        }
        if parsed.component_id.as_str() == "empty-state-specimen"
            && let Some(PropertyValue::String(title)) =
                parsed.properties.get(&PropertyId::new("title").unwrap())
            && title.trim().is_empty()
        {
            self.status = "Import rejected: EmptyState title must not be blank".into();
            return;
        }
        if parsed.component_id.as_str() == "error-state-specimen"
            && let Some(PropertyValue::String(title)) =
                parsed.properties.get(&PropertyId::new("title").unwrap())
            && title.trim().is_empty()
        {
            self.status = "Import rejected: ErrorState title must not be blank".into();
            return;
        }
        if parsed.component_id.as_str() == "alert-specimen"
            && let Some(PropertyValue::String(message)) =
                parsed.properties.get(&PropertyId::new("message").unwrap())
            && message.trim().is_empty()
        {
            self.status = "Import rejected: Alert message must not be blank".into();
            return;
        }
        if uses_toast_specimen {
            for name in ["message", "title"] {
                if let Some(PropertyValue::String(value)) =
                    parsed.properties.get(&PropertyId::new(name).unwrap())
                    && value.trim().is_empty()
                {
                    self.status = format!("Import rejected: Toast {name} must not be blank");
                    return;
                }
            }
        }
        if uses_sheet_specimen {
            for name in ["trigger_label", "title", "body_text"] {
                if let Some(PropertyValue::String(value)) =
                    parsed.properties.get(&PropertyId::new(name).unwrap())
                    && value.trim().is_empty()
                {
                    self.status = format!("Import rejected: Sheet {name} must not be blank");
                    return;
                }
            }
        }
        if uses_popover_specimen
            && let Some(PropertyValue::String(label)) = parsed
                .properties
                .get(&PropertyId::new("trigger_label").unwrap())
            && label.trim().is_empty()
        {
            self.status = "Import rejected: Popover trigger label must not be blank".into();
            return;
        }
        if uses_disclosure_specimen
            && let Some(PropertyValue::String(label)) =
                parsed.properties.get(&PropertyId::new("label").unwrap())
            && label.trim().is_empty()
        {
            self.status = "Import rejected: Disclosure label must not be blank".into();
            return;
        }
        if uses_toolbar_specimen {
            for name in ["primary_label", "secondary_label"] {
                if let Some(PropertyValue::String(label)) =
                    parsed.properties.get(&PropertyId::new(name).unwrap())
                    && label.trim().is_empty()
                {
                    self.status = format!("Import rejected: Toolbar {name} must not be blank");
                    return;
                }
            }
        }
        if uses_image_specimen
            && let Some(PropertyValue::String(label)) = parsed
                .properties
                .get(&PropertyId::new("accessibility_label").unwrap())
            && label.trim().is_empty()
        {
            self.status = "Import rejected: Image accessibility label must not be blank".into();
            return;
        }
        if uses_breadcrumb_specimen {
            for name in ["parent_label", "current_label"] {
                if let Some(PropertyValue::String(label)) =
                    parsed.properties.get(&PropertyId::new(name).unwrap())
                    && label.trim().is_empty()
                {
                    self.status = format!("Import rejected: Breadcrumb {name} must not be blank");
                    return;
                }
            }
        }
        if uses_context_menu_specimen {
            for name in ["target_label", "action_label"] {
                if let Some(PropertyValue::String(label)) =
                    parsed.properties.get(&PropertyId::new(name).unwrap())
                    && label.trim().is_empty()
                {
                    self.status = format!("Import rejected: ContextMenu {name} must not be blank");
                    return;
                }
            }
        }
        if uses_text_specimen
            && let Some(PropertyValue::String(value)) =
                parsed.properties.get(&PropertyId::new("value").unwrap())
            && value.trim().is_empty()
        {
            self.status = "Import rejected: Text value must not be blank".into();
            return;
        }
        match self.host.mount(parsed) {
            Ok(stamp) => {
                self.secure_value.clear();
                self.refresh_number_drafts();
                if let Some(snapshot) = self.host.snapshot() {
                    if !uses_secure_field_specimen {
                        self.text = property_string(
                            snapshot,
                            if uses_label_checked
                                || uses_radio
                                || uses_progress
                                || uses_button_specimen
                                || uses_tap_region_specimen
                                || uses_hover_region_specimen
                                || uses_color_picker_specimen
                            {
                                "label"
                            } else if uses_alert_specimen || uses_toast_specimen {
                                "message"
                            } else if uses_sheet_specimen {
                                "trigger_label"
                            } else if uses_disclosure_specimen {
                                "label"
                            } else if uses_toolbar_specimen {
                                "primary_label"
                            } else if uses_image_specimen {
                                "accessibility_label"
                            } else if uses_breadcrumb_specimen {
                                "current_label"
                            } else if uses_context_menu_specimen {
                                "target_label"
                            } else if uses_text_specimen {
                                "value"
                            } else if uses_menu_specimen
                                || uses_dialog_specimen
                                || uses_popover_specimen
                            {
                                "trigger_label"
                            } else if uses_selection {
                                "selection"
                            } else if uses_search_field_specimen {
                                "query"
                            } else if uses_token_field {
                                "draft"
                            } else if uses_text_field_specimen
                                || uses_text_editor_specimen
                                || uses_code_editor_specimen
                            {
                                "value"
                            } else {
                                "text"
                            },
                        )
                        .unwrap_or_default();
                    } else {
                        self.secure_field_placeholder =
                            property_string(snapshot, "placeholder").unwrap_or_default();
                    }
                    if uses_text_field_specimen {
                        self.text_field_placeholder =
                            property_string(snapshot, "placeholder").unwrap_or_default();
                    }
                    if uses_search_field_specimen {
                        self.search_field_placeholder =
                            property_string(snapshot, "placeholder").unwrap_or_default();
                    }
                    if uses_text_editor_specimen {
                        self.text_editor_placeholder =
                            property_string(snapshot, "placeholder").unwrap_or_default();
                    }
                    if uses_token_field {
                        self.token_field_placeholder =
                            property_string(snapshot, "placeholder").unwrap_or_default();
                        self.token_values = property_string(snapshot, "tokens_json")
                            .and_then(|value| parse_preview_token_values(&value).ok())
                            .unwrap_or_default();
                    }
                    self.enabled = property_boolean(
                        snapshot,
                        if uses_radio {
                            "selected"
                        } else if uses_label_checked {
                            "checked"
                        } else if uses_alert_specimen {
                            "visible"
                        } else if uses_disclosure_specimen {
                            "open"
                        } else if uses_toolbar_specimen {
                            "secondary_disabled"
                        } else if uses_popover_specimen {
                            "open"
                        } else {
                            "enabled"
                        },
                    )
                    .unwrap_or(true);
                    self.amount = property_number(
                        snapshot,
                        if uses_slider || uses_progress {
                            "value"
                        } else {
                            "amount"
                        },
                    )
                    .unwrap_or(0.375);
                    self.amount_draft = self.amount.to_string();
                    if uses_stepper {
                        if let Some(count) = property_integer(snapshot, "value")
                            .and_then(|value| i32::try_from(value).ok())
                        {
                            self.stepper_count = count;
                        }
                    }
                }
                self.stamp_lease = Rc::new(RefCell::new(Some(stamp.clone())));
                self.status = format!(
                    "Imported descriptor for {} · generation {}",
                    stamp.preview_id, stamp.generation
                );
                let status = self.status.clone();
                self.export_descriptor();
                self.status = status;
            }
            Err(error) => self.status = format!("Import rejected: {error}"),
        }
    }
    fn read_descriptor(owner: &Self) -> String {
        owner.descriptor_json.clone()
    }
    fn write_descriptor(owner: &mut Self, value: String) {
        owner.descriptor_json = value;
    }
    fn teardown(&mut self) {
        self.secure_value.clear();
        if let Some(stamp) = self.host.snapshot().map(|s| s.stamp.clone()) {
            self.status = format!("Teardown: {:?}", self.host.teardown(&stamp));
        }
    }
    fn string_selection_binding(
        selected_id: Option<String>,
        context: PropertyBindingContext,
    ) -> Binding<Option<String>> {
        Binding::new_with(
            selected_id,
            context,
            Self::read_string_selection,
            Self::write_string_selection,
        )
    }

    fn read_string_selection(&self, context: &PropertyBindingContext) -> Option<String> {
        let snapshot = self.host.snapshot()?;
        if context.stamp.borrow().as_ref() != Some(&snapshot.stamp) {
            return None;
        }
        property_string(&snapshot, context.property.as_str())
    }

    fn write_string_selection(
        &mut self,
        context: &PropertyBindingContext,
        selected_id: Option<String>,
    ) -> bool {
        selected_id.is_some_and(|selected_id| self.write_text(context, selected_id))
    }

    fn navigation_stack_path_binding(
        path: Vec<String>,
        context: PropertyBindingContext,
    ) -> Binding<Vec<String>> {
        Binding::new_with(
            path,
            context,
            Self::read_navigation_stack_path,
            Self::write_navigation_stack_path,
        )
    }

    fn read_navigation_stack_path(&self, context: &PropertyBindingContext) -> Vec<String> {
        match self.read_string_selection(context).as_deref() {
            Some("details") => vec!["details".into()],
            _ => Vec::new(),
        }
    }

    fn write_navigation_stack_path(
        &mut self,
        context: &PropertyBindingContext,
        path: Vec<String>,
    ) -> bool {
        let selected_id = match path.as_slice() {
            [] => "none",
            [route] if route == "details" => "details",
            _ => return false,
        };
        self.write_text(context, selected_id.into())
    }

    fn collection_selection_binding(
        selected_id: Option<Identity>,
        context: PropertyBindingContext,
    ) -> Binding<Option<Identity>> {
        Binding::new_with(
            selected_id,
            context,
            Self::read_collection_selection,
            Self::write_collection_selection,
        )
    }

    fn read_collection_selection(&self, context: &PropertyBindingContext) -> Option<Identity> {
        self.read_string_selection(context)
            .filter(|selected_id| selected_id != "none")
            .map(Identity::from)
    }

    fn write_collection_selection(
        &mut self,
        context: &PropertyBindingContext,
        selected_id: Option<Identity>,
    ) -> bool {
        let selected_id = selected_id
            .map(|id| id.as_str().to_owned())
            .unwrap_or_else(|| "none".into());
        if !PREVIEW_COLLECTION_SELECTIONS.contains(&selected_id.as_str()) {
            return false;
        }
        self.write_text(context, selected_id)
    }

    fn data_grid_selection_binding(
        selected_row: Option<usize>,
        context: PropertyBindingContext,
    ) -> Binding<Option<usize>> {
        Binding::new_with(
            selected_row,
            context,
            Self::read_data_grid_selection,
            Self::write_data_grid_selection,
        )
    }

    fn list_selection_binding(
        selected_index: Option<usize>,
        context: PropertyBindingContext,
    ) -> Binding<Option<usize>> {
        Binding::new_with(
            selected_index,
            context,
            Self::read_list_selection,
            Self::write_list_selection,
        )
    }

    fn read_list_selection(&self, context: &PropertyBindingContext) -> Option<usize> {
        self.read_string_selection(context).and_then(|selection| {
            selection
                .strip_prefix("row-")
                .and_then(|row| row.parse::<usize>().ok())
        })
    }

    fn write_list_selection(
        &mut self,
        context: &PropertyBindingContext,
        selected_index: Option<usize>,
    ) -> bool {
        if selected_index.is_some_and(|index| index >= PREVIEW_LIST_ROWS.len()) {
            return false;
        }
        let selected_id =
            selected_index.map_or_else(|| "none".into(), |index| format!("row-{index}"));
        self.write_text(context, selected_id)
    }

    fn read_data_grid_selection(&self, context: &PropertyBindingContext) -> Option<usize> {
        self.read_string_selection(context).and_then(|selection| {
            selection
                .strip_prefix("row-")
                .and_then(|row| row.parse::<usize>().ok())
        })
    }

    fn write_data_grid_selection(
        &mut self,
        context: &PropertyBindingContext,
        selected_row: Option<usize>,
    ) -> bool {
        if selected_row.is_some_and(|row| row >= 3) {
            return false;
        }
        let selected_id = selected_row.map_or_else(|| "none".into(), |row| format!("row-{row}"));
        self.write_text(context, selected_id)
    }

    fn tree_selection_binding(
        selected_id: Option<String>,
        context: PropertyBindingContext,
    ) -> Binding<Option<String>> {
        Binding::new_with(
            selected_id,
            context,
            Self::read_tree_selection,
            Self::write_tree_selection,
        )
    }

    fn read_tree_selection(&self, context: &PropertyBindingContext) -> Option<String> {
        self.read_string_selection(context)
            .filter(|selection| selection != "none")
    }

    fn write_tree_selection(
        &mut self,
        context: &PropertyBindingContext,
        selected_id: Option<String>,
    ) -> bool {
        if selected_id
            .as_deref()
            .is_some_and(|id| !PREVIEW_TREE_NODE_IDS.contains(&id))
        {
            return false;
        }
        self.write_text(context, selected_id.unwrap_or_else(|| "none".into()))
    }

    fn read_disclosure_open(&self, context: &PropertyBindingContext) -> bool {
        self.preview_snapshot_for_component(context, "disclosure-specimen")
            .and_then(|snapshot| property_boolean(&snapshot, "open"))
            .unwrap_or(false)
    }

    fn write_disclosure_open(&mut self, context: &PropertyBindingContext, open: bool) -> bool {
        let Some(stamp) = context.stamp.borrow().clone() else {
            return false;
        };
        let Some(snapshot) = self.host.snapshot() else {
            return false;
        };
        if snapshot.stamp != stamp
            || snapshot.preview.document.component_id.as_str() != "disclosure-specimen"
        {
            return false;
        }
        let signal = snapshot.signal(EventId::new("toggle").unwrap());
        if !matches!(self.host.dispatch(&signal), Ok(target) if target.as_str() == "toggle") {
            return false;
        }
        match self.host.binding_feedback(
            &stamp,
            context.property.clone(),
            PropertyValue::Boolean(open),
        ) {
            Ok(next) => {
                self.enabled = open;
                *context.stamp.borrow_mut() = Some(next);
                self.status = "Signal-only disclosure toggle accepted".into();
                true
            }
            Err(error) => {
                self.status = error.to_string();
                false
            }
        }
    }

    fn dismiss_alert(&mut self, signal: &PreviewSignal) {
        match self.host.dispatch(signal) {
            Ok(target) if target.as_str() == "dismiss" => {
                let context = PropertyBindingContext {
                    property: PropertyId::new("visible").unwrap(),
                    stamp: self.stamp_lease.clone(),
                };
                if self.write_enabled(&context, false) {
                    self.status = "Signal-only alert dismiss action".into();
                }
            }
            result => self.status = format!("Rejected alert signal: {result:?}"),
        }
    }

    fn show_toast(&mut self, signal: &PreviewSignal) {
        match self.host.dispatch(signal) {
            Ok(target) if target.as_str() == "toast_show" => {
                self.activations += 1;
                self.status = "Allowlisted Toast show action".into();
            }
            result => self.status = format!("Rejected Toast signal: {result:?}"),
        }
    }

    fn toolbar_primary(&mut self, signal: &PreviewSignal) {
        match self.host.dispatch(signal) {
            Ok(target) if target.as_str() == "toolbar_primary" => {
                self.activations += 1;
                self.status = "Allowlisted Toolbar primary action".into();
            }
            result => self.status = format!("Rejected Toolbar primary signal: {result:?}"),
        }
    }

    fn toolbar_secondary(&mut self, signal: &PreviewSignal) {
        match self.host.dispatch(signal) {
            Ok(target) if target.as_str() == "toolbar_secondary" => {
                self.activations += 1;
                self.status = "Allowlisted Toolbar secondary action".into();
            }
            result => self.status = format!("Rejected Toolbar secondary signal: {result:?}"),
        }
    }

    fn navigate_breadcrumb_parent(&mut self, signal: &PreviewSignal) {
        match self.host.dispatch(signal) {
            Ok(target) if target.as_str() == "breadcrumb_parent" => {
                self.activations += 1;
                self.status = "Allowlisted Breadcrumb parent action".into();
            }
            result => self.status = format!("Rejected Breadcrumb parent signal: {result:?}"),
        }
    }

    fn activate_context_menu_item(&mut self, signal: &PreviewSignal) {
        match self.host.dispatch(signal) {
            Ok(target) if target.as_str() == "context_menu_open_preview" => {
                self.activations += 1;
                self.status = "Allowlisted ContextMenu action".into();
            }
            result => self.status = format!("Rejected ContextMenu signal: {result:?}"),
        }
    }

    fn activate(&mut self, signal: &PreviewSignal) {
        match self.host.dispatch(signal) {
            Ok(target) if target.as_str() == "increment" => {
                self.activations += 1;
                self.status = "Allowlisted increment action".into();
            }
            result => self.status = format!("Rejected signal: {result:?}"),
        }
    }

    fn record_hover_enter(&mut self, signal: &PreviewSignal) {
        match self.host.dispatch(signal) {
            Ok(target) if target.as_str() == "increment" => {
                self.activations += 1;
                self.status = "Allowlisted HoverRegion enter action".into();
            }
            result => self.status = format!("Rejected HoverRegion enter signal: {result:?}"),
        }
    }

    fn record_hover_exit(&mut self, signal: &PreviewSignal) {
        match self.host.dispatch(signal) {
            Ok(target) if target.as_str() == "increment" => {
                self.activations += 1;
                self.status = "Allowlisted HoverRegion exit action".into();
            }
            result => self.status = format!("Rejected HoverRegion exit signal: {result:?}"),
        }
    }
    fn reject_stale(&mut self) {
        let Some(current) = self.host.snapshot().map(|snapshot| snapshot.stamp.clone()) else {
            return;
        };
        let stale = PreviewStamp {
            generation: 0,
            ..current
        };
        let signal = PreviewSignal {
            stamp: stale.clone(),
            event: EventId::new("activate").unwrap(),
        };
        let action_rejected = self.host.dispatch(&signal).is_err();
        let feedback_rejected = self
            .host
            .binding_feedback(
                &stale,
                PropertyId::new("text").unwrap(),
                PropertyValue::String("stale write".into()),
            )
            .is_err();
        self.status = format!(
            "Stale action rejected: {action_rejected}; stale binding rejected: {feedback_rejected}"
        );
    }
    fn capture_stale_amount_callback(&mut self) {
        let Some(stamp) = self.host.snapshot().map(|snapshot| snapshot.stamp.clone()) else {
            self.status = "No mounted preview to capture".into();
            return;
        };
        self.stale_amount_callback = Some(Binding::new_with(
            self.amount_draft.clone(),
            PropertyBindingContext {
                property: PropertyId::new("amount").unwrap(),
                stamp: Rc::new(RefCell::new(Some(stamp.clone()))),
            },
            Self::read_amount_draft,
            Self::write_amount_draft,
        ));
        self.status = format!(
            "Captured amount callback at generation {} revision {}",
            stamp.generation, stamp.revision
        );
    }
    fn replay_stale_amount_callback(&mut self) {
        let Some(callback) = self.stale_amount_callback.clone() else {
            self.status = "Capture an amount callback before replay".into();
            return;
        };
        let draft_before = self.amount_draft.clone();
        let amount_before = self
            .host
            .snapshot()
            .and_then(|snapshot| property_number(snapshot, "amount"));
        let accepted = callback.write_to(self, "0.987".into());
        let amount_after = self
            .host
            .snapshot()
            .and_then(|snapshot| property_number(snapshot, "amount"));
        let unchanged = self.amount_draft == draft_before && amount_after == amount_before;
        self.status =
            format!("Stale amount callback accepted={accepted}; state unchanged={unchanged}");
    }
    fn reject_wrong_type(&mut self) {
        let Some(stamp) = self.host.snapshot().map(|snapshot| snapshot.stamp.clone()) else {
            return;
        };
        let rejected = self
            .host
            .edit(
                &stamp,
                [(
                    PropertyId::new("text").unwrap(),
                    PropertyValue::Boolean(true),
                )],
            )
            .is_err();
        self.status = format!("Wrong property type rejected: {rejected}");
    }
    fn read_text(&self, context: &PropertyBindingContext) -> String {
        let snapshot = self.host.snapshot();
        if snapshot.as_ref().map(|snapshot| &snapshot.stamp) != context.stamp.borrow().as_ref() {
            return String::new();
        }
        let Some(snapshot) = snapshot else {
            return self.text.clone();
        };
        if context.property.as_str() == "title"
            && snapshot.preview.document.component_id.as_str() == "toast-specimen"
            && property_string(&snapshot, "title").is_none()
        {
            return String::new();
        }
        if context.property.as_str() == "placeholder" {
            match snapshot.preview.document.component_id.as_str() {
                "text-field-specimen" => self.text_field_placeholder.clone(),
                "search-field-specimen" => self.search_field_placeholder.clone(),
                "text-editor-specimen" => self.text_editor_placeholder.clone(),
                "secure-field-specimen" => self.secure_field_placeholder.clone(),
                "token-field-specimen" => self.token_field_placeholder.clone(),
                _ => String::new(),
            }
        } else {
            property_string(&snapshot, context.property.as_str()).unwrap_or_default()
        }
    }
    fn read_token_values(&self, context: &PropertyBindingContext) -> Vec<String> {
        let Some(snapshot) = self.host.snapshot() else {
            return Vec::new();
        };
        if context.stamp.borrow().as_ref() != Some(&snapshot.stamp)
            || snapshot.preview.document.component_id.as_str() != "token-field-specimen"
        {
            return Vec::new();
        }
        property_string(&snapshot, context.property.as_str())
            .and_then(|value| parse_preview_token_values(&value).ok())
            .unwrap_or_default()
    }
    fn write_token_values(
        &mut self,
        context: &PropertyBindingContext,
        values: Vec<String>,
    ) -> bool {
        let Some(stamp) = context.stamp.borrow().clone() else {
            return false;
        };
        let Some(snapshot) = self.host.snapshot() else {
            return false;
        };
        if snapshot.stamp != stamp
            || snapshot.preview.document.component_id.as_str() != "token-field-specimen"
        {
            return false;
        }
        let Ok(tokens_json) = serde_json::to_string(&values) else {
            self.status = "Rejected: TokenField token list could not be serialized".into();
            return false;
        };
        if tokens_json.len() > PREVIEW_TOKEN_VALUES_MAX_LENGTH {
            self.status = "Rejected: TokenField token list exceeds its preview limit".into();
            return false;
        }
        match self.host.binding_feedback(
            &stamp,
            context.property.clone(),
            PropertyValue::String(tokens_json),
        ) {
            Ok(next) => {
                self.token_values = values;
                *context.stamp.borrow_mut() = Some(next);
                self.status = "TokenField token list accepted".into();
                true
            }
            Err(error) => {
                self.status = format!("Rejected: {error}");
                false
            }
        }
    }
    fn write_text(&mut self, context: &PropertyBindingContext, text: String) -> bool {
        let Some(stamp) = context.stamp.borrow().clone() else {
            return false;
        };
        let Some(snapshot) = self.host.snapshot() else {
            return false;
        };
        if snapshot.stamp != stamp {
            return false;
        }

        let is_date_picker_date = snapshot.preview.document.component_id.as_str()
            == "date-picker-specimen"
            && context.property.as_str() == "date";
        if is_date_picker_date && parse_preview_date(&text).is_err() {
            self.status = DATE_PICKER_DATE_REJECTION.into();
            return false;
        }
        let is_token_values = snapshot.preview.document.component_id.as_str()
            == "token-field-specimen"
            && context.property.as_str() == "tokens_json";
        if is_token_values && parse_preview_token_values(&text).is_err() {
            self.status = "Rejected: TokenField tokens_json must be a JSON string array".into();
            return false;
        }
        let requires_nonblank = self
            .host
            .catalog()
            .components
            .get(&snapshot.preview.document.component_id)
            .and_then(|component| component.properties.get(&context.property))
            .is_some_and(|property| property.min_length.is_some_and(|minimum| minimum > 0));
        if requires_nonblank && text.trim().is_empty() {
            self.status = "Rejected blank value for a nonempty preview property".into();
            return false;
        }
        match self.host.binding_feedback(
            &stamp,
            context.property.clone(),
            PropertyValue::String(text.clone()),
        ) {
            Ok(next) => {
                if context.property.as_str() == "placeholder" {
                    match self
                        .host
                        .snapshot()
                        .map(|snapshot| snapshot.preview.document.component_id.as_str().to_owned())
                    {
                        Some(component_id) if component_id == "text-field-specimen" => {
                            self.text_field_placeholder = text;
                        }
                        Some(component_id) if component_id == "search-field-specimen" => {
                            self.search_field_placeholder = text;
                        }
                        Some(component_id) if component_id == "text-editor-specimen" => {
                            self.text_editor_placeholder = text;
                        }
                        Some(component_id) if component_id == "secure-field-specimen" => {
                            self.secure_field_placeholder = text;
                        }
                        Some(component_id) if component_id == "token-field-specimen" => {
                            self.token_field_placeholder = text;
                        }
                        _ => self.text = text,
                    }
                } else if is_token_values {
                    self.token_values = parse_preview_token_values(&text).unwrap_or_default();
                } else {
                    self.text = text;
                }
                self.status = "Binding feedback accepted".into();
                // The same retained InputState subscription will serve the next
                // edit, so advance its lease only after this exact revision accepts.
                *context.stamp.borrow_mut() = Some(next);
                true
            }
            Err(error) => {
                self.status = format!("Rejected: {error}");
                false
            }
        }
    }
    fn read_secure_value(&self, context: &PropertyBindingContext) -> String {
        if self
            .preview_snapshot_for_component(context, "secure-field-specimen")
            .is_none()
        {
            return String::new();
        }
        self.secure_value.clone()
    }
    fn write_secure_value(&mut self, context: &PropertyBindingContext, secret: String) -> bool {
        let Some(stamp) = context.stamp.borrow().clone() else {
            return false;
        };
        let Some(snapshot) = self.host.snapshot() else {
            return false;
        };
        if snapshot.stamp != stamp
            || snapshot.preview.document.component_id.as_str() != "secure-field-specimen"
        {
            return false;
        }

        self.secure_value = secret;
        true
    }
    fn preview_snapshot_for_binding(
        &self,
        context: &PropertyBindingContext,
    ) -> Option<PreviewSnapshot> {
        let stamp = context.stamp.borrow().clone()?;
        self.host
            .snapshot()
            .filter(|snapshot| snapshot.stamp == stamp)
            .cloned()
    }

    fn preview_snapshot_for_component(
        &self,
        context: &PropertyBindingContext,
        component_id: &str,
    ) -> Option<PreviewSnapshot> {
        self.preview_snapshot_for_binding(context)
            .filter(|snapshot| snapshot.preview.document.component_id.as_str() == component_id)
    }

    fn read_enabled(&self, context: &PropertyBindingContext) -> bool {
        let Some(snapshot) = self.preview_snapshot_for_binding(context) else {
            return false;
        };
        property_boolean(&snapshot, context.property.as_str()).unwrap_or(self.enabled)
    }

    fn preview_inspector_boolean_binding(&self, context: PropertyBindingContext) -> Binding<bool> {
        let initial = self.read_enabled(&context);
        Binding::new_with(initial, context, Self::read_enabled, Self::write_enabled)
    }

    fn read_date_value(&self, context: &PropertyBindingContext) -> Option<NaiveDate> {
        let snapshot = self.preview_snapshot_for_component(context, "date-picker-specimen")?;
        property_date(&snapshot)
    }

    fn write_date_value(
        &mut self,
        context: &PropertyBindingContext,
        value: Option<NaiveDate>,
    ) -> bool {
        let Some(stamp) = context.stamp.borrow().clone() else {
            return false;
        };
        if !self.host.snapshot().is_some_and(|snapshot| {
            snapshot.stamp == stamp
                && snapshot.preview.document.component_id.as_str() == "date-picker-specimen"
        }) {
            return false;
        }
        let encoded = value.map(|date| date.to_string()).unwrap_or_default();
        if parse_preview_date(&encoded).is_err() {
            self.status = DATE_PICKER_DATE_REJECTION.into();
            return false;
        }
        match self.host.binding_feedback(
            &stamp,
            context.property.clone(),
            PropertyValue::String(encoded),
        ) {
            Ok(next) => {
                *context.stamp.borrow_mut() = Some(next);
                self.status = "DatePicker feedback accepted".into();
                true
            }
            Err(error) => {
                self.status = error.to_string();
                false
            }
        }
    }

    fn read_color_value(&self, context: &PropertyBindingContext) -> Option<Hsla> {
        let snapshot = self.preview_snapshot_for_component(context, "color-picker-specimen")?;
        property_color(&snapshot).flatten()
    }

    fn read_color_presence(&self, context: &PropertyBindingContext) -> bool {
        self.preview_snapshot_for_component(context, "color-picker-specimen")
            .and_then(|snapshot| property_boolean(&snapshot, "has_color"))
            .unwrap_or(false)
    }

    fn write_color_presence(&mut self, context: &PropertyBindingContext, has_color: bool) -> bool {
        let color = self
            .preview_snapshot_for_component(context, "color-picker-specimen")
            .and_then(|snapshot| {
                Some(Hsla {
                    h: property_number(&snapshot, "hue")? as f32,
                    s: property_number(&snapshot, "saturation")? as f32,
                    l: property_number(&snapshot, "lightness")? as f32,
                    a: property_number(&snapshot, "alpha")? as f32,
                })
            })
            .unwrap_or_else(|| Hsla {
                h: PREVIEW_COLOR_HUE as f32,
                s: PREVIEW_COLOR_SATURATION as f32,
                l: PREVIEW_COLOR_LIGHTNESS as f32,
                a: PREVIEW_COLOR_ALPHA as f32,
            });
        self.write_color_value(context, has_color.then_some(color))
    }

    fn write_color_value(&mut self, context: &PropertyBindingContext, value: Option<Hsla>) -> bool {
        let Some(stamp) = context.stamp.borrow().clone() else {
            return false;
        };
        let Some((current, color)) = self.host.snapshot().and_then(|snapshot| {
            (snapshot.stamp == stamp
                && snapshot.preview.document.component_id.as_str() == "color-picker-specimen")
                .then(|| (snapshot.stamp.clone(), property_color(snapshot).flatten()))
        }) else {
            return false;
        };
        if current != stamp {
            return false;
        }

        let has_color = value.is_some();
        let color = value.or(color).unwrap_or_else(|| Hsla {
            h: PREVIEW_COLOR_HUE as f32,
            s: PREVIEW_COLOR_SATURATION as f32,
            l: PREVIEW_COLOR_LIGHTNESS as f32,
            a: PREVIEW_COLOR_ALPHA as f32,
        });
        let channels = [color.h, color.s, color.l, color.a];
        if channels
            .iter()
            .any(|channel| !channel.is_finite() || !(0.0..=1.0).contains(channel))
        {
            self.status = "Color channels must be finite values from 0 to 1".into();
            return false;
        }

        match self.host.edit(
            &stamp,
            [
                (
                    PropertyId::new("has_color").unwrap(),
                    PropertyValue::Boolean(has_color),
                ),
                (
                    PropertyId::new("hue").unwrap(),
                    PropertyValue::Number(f64::from(color.h)),
                ),
                (
                    PropertyId::new("saturation").unwrap(),
                    PropertyValue::Number(f64::from(color.s)),
                ),
                (
                    PropertyId::new("lightness").unwrap(),
                    PropertyValue::Number(f64::from(color.l)),
                ),
                (
                    PropertyId::new("alpha").unwrap(),
                    PropertyValue::Number(f64::from(color.a)),
                ),
            ],
        ) {
            Ok(next) => {
                *context.stamp.borrow_mut() = Some(next);
                self.status = "ColorPicker feedback accepted".into();
                true
            }
            Err(error) => {
                self.status = error.to_string();
                false
            }
        }
    }

    fn write_enabled(&mut self, context: &PropertyBindingContext, enabled: bool) -> bool {
        let Some(stamp) = context.stamp.borrow().clone() else {
            return false;
        };
        if self.host.snapshot().map(|snapshot| &snapshot.stamp) != Some(&stamp) {
            return false;
        }
        match self.host.binding_feedback(
            &stamp,
            context.property.clone(),
            PropertyValue::Boolean(enabled),
        ) {
            Ok(next) => {
                self.enabled = enabled;
                *context.stamp.borrow_mut() = Some(next);
                self.status = "Boolean property accepted".into();
                true
            }
            Err(error) => {
                self.status = error.to_string();
                false
            }
        }
    }
    fn refresh_number_drafts(&mut self) {
        let drafts = self
            .host
            .snapshot()
            .map(|snapshot| {
                snapshot
                    .preview
                    .properties
                    .iter()
                    .filter_map(|(property, value)| match value {
                        PropertyValue::Number(value) => Some((property.clone(), value.to_string())),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default();
        self.number_drafts = drafts;
    }

    fn read_number_draft(&self, context: &PropertyBindingContext) -> String {
        let Some(stamp) = context.stamp.borrow().clone() else {
            return String::new();
        };
        let Some(snapshot) = self.host.snapshot() else {
            return String::new();
        };
        if snapshot.stamp != stamp {
            return String::new();
        }
        self.number_drafts
            .get(&context.property)
            .cloned()
            .or_else(|| {
                property_number(&snapshot, context.property.as_str()).map(|value| value.to_string())
            })
            .unwrap_or_default()
    }

    fn write_number_draft(&mut self, context: &PropertyBindingContext, draft: String) -> bool {
        let Some(stamp) = context.stamp.borrow().clone() else {
            return false;
        };
        let Some(snapshot) = self.host.snapshot() else {
            return false;
        };
        if snapshot.stamp != stamp {
            return false;
        }

        self.number_drafts
            .insert(context.property.clone(), draft.clone());
        let Ok(value) = draft.trim().parse::<f64>() else {
            self.status = "Enter a finite number".into();
            return true;
        };
        if !value.is_finite() {
            self.status = "Enter a finite number".into();
            return true;
        }

        let component_id = snapshot.preview.document.component_id.as_str().to_owned();
        match self.host.binding_feedback(
            &stamp,
            context.property.clone(),
            PropertyValue::Number(value),
        ) {
            Ok(next) => {
                if (component_id == "property-inspector-specimen"
                    && context.property.as_str() == "amount")
                    || (matches!(component_id.as_str(), "slider" | "progress")
                        && context.property.as_str() == "value")
                {
                    self.amount = value;
                    self.amount_draft = value.to_string();
                }
                *context.stamp.borrow_mut() = Some(next);
                self.status = format!("Accepted f64 value {value}");
                true
            }
            Err(error) => {
                self.status = format!("Rejected: {error}");
                true
            }
        }
    }

    fn read_amount_draft(&self, context: &PropertyBindingContext) -> String {
        self.preview_snapshot_for_binding(context)
            .map(|_| self.amount_draft.clone())
            .unwrap_or_default()
    }
    fn write_amount_draft(&mut self, context: &PropertyBindingContext, draft: String) -> bool {
        let Some(stamp) = context.stamp.borrow().clone() else {
            return false;
        };
        if self.host.snapshot().map(|snapshot| &snapshot.stamp) != Some(&stamp) {
            return false;
        }
        self.amount_draft = draft.clone();
        let Ok(amount) = draft.trim().parse::<f64>() else {
            self.status = "Enter a finite number from 0 to 1".into();
            return true;
        };
        if !amount.is_finite() {
            self.status = "Enter a finite number from 0 to 1".into();
            return true;
        }
        match self.host.binding_feedback(
            &stamp,
            context.property.clone(),
            PropertyValue::Number(amount),
        ) {
            Ok(next) => {
                self.amount = amount;
                *context.stamp.borrow_mut() = Some(next);
                self.number_drafts
                    .insert(context.property.clone(), amount.to_string());
                self.status = format!("Accepted f64 value {}", amount);
                true
            }
            Err(error) => {
                self.status = format!("Rejected: {error}");
                true
            }
        }
    }
    fn read_stepper_value(&self, context: &PropertyBindingContext) -> i32 {
        self.preview_snapshot_for_component(context, "stepper")
            .map(|_| self.stepper_count)
            .unwrap_or(PREVIEW_STEPPER_MIN)
    }

    fn write_stepper_value(&mut self, context: &PropertyBindingContext, value: i32) -> bool {
        let Some(stamp) = context.stamp.borrow().clone() else {
            return false;
        };
        if self.host.snapshot().map(|snapshot| &snapshot.stamp) != Some(&stamp)
            || !(PREVIEW_STEPPER_MIN..=PREVIEW_STEPPER_MAX).contains(&value)
        {
            return false;
        }

        match self.host.binding_feedback(
            &stamp,
            context.property.clone(),
            PropertyValue::Integer(i64::from(value)),
        ) {
            Ok(next) => {
                self.stepper_count = value;
                *context.stamp.borrow_mut() = Some(next);
                self.status = format!("Accepted integer value {value}");
                true
            }
            Err(error) => {
                self.status = error.to_string();
                false
            }
        }
    }

    fn read_integer_property(&self, context: &PropertyBindingContext) -> i64 {
        let Some(stamp) = context.stamp.borrow().clone() else {
            return 0;
        };
        self.host
            .snapshot()
            .filter(|snapshot| snapshot.stamp == stamp)
            .and_then(|snapshot| property_integer(&snapshot, context.property.as_str()))
            .unwrap_or_default()
    }

    fn write_integer_property(&mut self, context: &PropertyBindingContext, value: i64) -> bool {
        let Some(stamp) = context.stamp.borrow().clone() else {
            return false;
        };
        let Some(snapshot) = self.host.snapshot() else {
            return false;
        };
        if snapshot.stamp != stamp {
            return false;
        }
        let stepper_value = if snapshot.preview.document.component_id.as_str() == "stepper" {
            let Ok(stepper_value) = i32::try_from(value) else {
                self.status = "Stepper value must fit its control".into();
                return false;
            };
            Some(stepper_value)
        } else {
            None
        };

        match self.host.binding_feedback(
            &stamp,
            context.property.clone(),
            PropertyValue::Integer(value),
        ) {
            Ok(next) => {
                if let Some(stepper_value) = stepper_value {
                    self.stepper_count = stepper_value;
                }
                *context.stamp.borrow_mut() = Some(next);
                self.status = format!("Accepted integer property {value}");
                true
            }
            Err(error) => {
                self.status = format!("Rejected: {error}");
                false
            }
        }
    }

    fn read_slider_value(&self, context: &PropertyBindingContext) -> f32 {
        self.preview_snapshot_for_component(context, "slider")
            .map(|_| self.amount as f32)
            .unwrap_or(PREVIEW_SLIDER_MIN)
    }
    fn write_slider_value(&mut self, context: &PropertyBindingContext, value: f32) -> bool {
        let Some(stamp) = context.stamp.borrow().clone() else {
            return false;
        };
        if self.host.snapshot().map(|snapshot| &snapshot.stamp) != Some(&stamp)
            || !value.is_finite()
            || !(PREVIEW_SLIDER_MIN..=PREVIEW_SLIDER_MAX).contains(&value)
        {
            return false;
        }
        let amount = f64::from(value);
        match self.host.binding_feedback(
            &stamp,
            context.property.clone(),
            PropertyValue::Number(amount),
        ) {
            Ok(next) => {
                self.amount = amount;
                self.amount_draft = amount.to_string();
                self.number_drafts
                    .insert(context.property.clone(), amount.to_string());
                *context.stamp.borrow_mut() = Some(next);
                self.status = format!("Accepted slider value {amount}");
                true
            }
            Err(error) => {
                self.status = error.to_string();
                false
            }
        }
    }
    fn text_binding(&self) -> Binding<String> {
        let context = PropertyBindingContext {
            property: self
                .host
                .snapshot()
                .map(
                    |snapshot| match snapshot.preview.document.component_id.as_str() {
                        "picker" | "combo-box-specimen" | "tab-view-specimen" => {
                            PropertyId::new("selection").unwrap()
                        }
                        "token-field-specimen" => PropertyId::new("draft").unwrap(),
                        "button-specimen" | "tap-region-specimen" | "hover-region-specimen" => {
                            PropertyId::new("label").unwrap()
                        }
                        "toast-specimen" => PropertyId::new("message").unwrap(),
                        "sheet-specimen" => PropertyId::new("trigger_label").unwrap(),
                        "search-field-specimen" => PropertyId::new("query").unwrap(),
                        "text-field-specimen" => PropertyId::new("value").unwrap(),
                        "text-editor-specimen" | "text-specimen" | "code-editor-specimen" => {
                            PropertyId::new("value").unwrap()
                        }
                        _ => PropertyId::new("text").unwrap(),
                    },
                )
                .unwrap_or_else(|| PropertyId::new("text").unwrap()),
            stamp: self.stamp_lease.clone(),
        };
        Binding::new_with(
            self.text.clone(),
            context,
            Self::read_text,
            Self::write_text,
        )
    }
}

impl View for CompiledPreviewGallery {
    fn body(&self) -> impl View {
        let render_preview = self.host.render_factory();
        let inspector_rows = self
            .host
            .snapshot()
            .and_then(|snapshot| {
                self.host
                    .catalog()
                    .components
                    .get(&snapshot.preview.document.component_id)
            })
            .map(|schema| {
                let generation = self
                    .host
                    .snapshot()
                    .map(|snapshot| snapshot.stamp.generation)
                    .unwrap_or_default();
                schema
                    .properties
                    .iter()
                    .filter(|(_, property)| preview_inspector_property_is_visible(property))
                    .filter_map(|(id, property)| {
                        // A remount is a new binding lease. Scope each editor
                        // to that generation so retained input state cannot
                        // keep the previous generation's captured lease.
                        let row_id = format!("{}-generation-{generation}", id.as_str());
                        let context = PropertyBindingContext {
                            property: id.clone(),
                            stamp: self.stamp_lease.clone(),
                        };
                        match property.kind {
                            PropertyKind::String => {
                                let snapshot = self.host.snapshot();
                                let value = Binding::new_with(
                                    preview_inspector_string_initial_value(snapshot, id.as_str()),
                                    context,
                                    Self::read_text,
                                    Self::write_text,
                                );
                                if property.allowed_values.is_empty() {
                                    Some(PropertyInspectorRow::text(
                                        row_id.clone(),
                                        id.as_str(),
                                        value,
                                    ))
                                } else {
                                    Some(PropertyInspectorRow::choice(
                                        row_id.clone(),
                                        id.as_str(),
                                        value,
                                        property.allowed_values.clone(),
                                    ))
                                }
                            }
                            PropertyKind::Boolean => {
                                let value = if schema.id.as_str() == "color-picker-specimen"
                                    && id.as_str() == "has_color"
                                {
                                    Binding::new_with(
                                        self.read_color_presence(&context),
                                        context,
                                        Self::read_color_presence,
                                        Self::write_color_presence,
                                    )
                                } else {
                                    self.preview_inspector_boolean_binding(context)
                                };
                                Some(PropertyInspectorRow::toggle(
                                    row_id.clone(),
                                    id.as_str(),
                                    value,
                                ))
                            }
                            PropertyKind::Integer => Some(PropertyInspectorRow::integer_value(
                                row_id,
                                format!("{} (integer)", id.as_str()),
                                Binding::new_with(
                                    self.host
                                        .snapshot()
                                        .and_then(|snapshot| {
                                            property_integer(&snapshot, id.as_str())
                                        })
                                        .unwrap_or_default(),
                                    context,
                                    Self::read_integer_property,
                                    Self::write_integer_property,
                                ),
                            )),
                            PropertyKind::Number => Some(PropertyInspectorRow::text(
                                row_id,
                                format!("{} (f64)", id.as_str()),
                                Binding::new_with(
                                    self.read_number_draft(&context),
                                    context,
                                    Self::read_number_draft,
                                    Self::write_number_draft,
                                ),
                            )),
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let descriptor_binding = Binding::new::<Self>(
            self.descriptor_json.clone(),
            Self::read_descriptor,
            Self::write_descriptor,
        );
        let binding = self.text_binding();
        let token_values_binding = Binding::new_with(
            self.token_values.clone(),
            PropertyBindingContext {
                property: PropertyId::new("tokens_json").unwrap(),
                stamp: self.stamp_lease.clone(),
            },
            Self::read_token_values,
            Self::write_token_values,
        );
        let secure_value_binding = Binding::new_with(
            self.secure_value.clone(),
            PropertyBindingContext {
                property: PropertyId::new("placeholder").unwrap(),
                stamp: self.stamp_lease.clone(),
            },
            CompiledPreviewGallery::read_secure_value,
            CompiledPreviewGallery::write_secure_value,
        );
        let checked_binding = Binding::new_with(
            self.enabled,
            PropertyBindingContext {
                property: if self.host.snapshot().is_some_and(|snapshot| {
                    snapshot.preview.document.component_id.as_str() == "radio"
                }) {
                    PropertyId::new("selected").unwrap()
                } else if self.host.snapshot().is_some_and(|snapshot| {
                    snapshot.preview.document.component_id.as_str() == "alert-specimen"
                }) {
                    PropertyId::new("visible").unwrap()
                } else if self.host.snapshot().is_some_and(|snapshot| {
                    snapshot.preview.document.component_id.as_str() == "popover-specimen"
                }) {
                    PropertyId::new("open").unwrap()
                } else {
                    PropertyId::new("checked").unwrap()
                },
                stamp: self.stamp_lease.clone(),
            },
            Self::read_enabled,
            Self::write_enabled,
        );
        let slider_binding = Binding::new_with(
            self.amount as f32,
            PropertyBindingContext {
                property: PropertyId::new("value").unwrap(),
                stamp: self.stamp_lease.clone(),
            },
            CompiledPreviewGallery::read_slider_value,
            CompiledPreviewGallery::write_slider_value,
        );
        let stepper_binding = Binding::new_with(
            self.stepper_count,
            PropertyBindingContext {
                property: PropertyId::new("value").unwrap(),
                stamp: self.stamp_lease.clone(),
            },
            CompiledPreviewGallery::read_stepper_value,
            CompiledPreviewGallery::write_stepper_value,
        );
        let date_value_binding = Binding::new_with(
            self.host.snapshot().and_then(property_date),
            PropertyBindingContext {
                property: PropertyId::new("date").unwrap(),
                stamp: self.stamp_lease.clone(),
            },
            Self::read_date_value,
            Self::write_date_value,
        );
        let color_value_binding = Binding::new_with(
            self.host.snapshot().and_then(property_color).flatten(),
            PropertyBindingContext {
                property: PropertyId::new("hue").unwrap(),
                stamp: self.stamp_lease.clone(),
            },
            CompiledPreviewGallery::read_color_value,
            CompiledPreviewGallery::write_color_value,
        );
        let stamp_lease = self.stamp_lease.clone();
        let owner_summary = if self.host.snapshot().is_some_and(|snapshot| {
            snapshot.preview.document.component_id.as_str() == "secure-field-specimen"
        }) {
            format!(
                "Owner: secure value present={} enabled={} amount={} · mounted stamp: {:?}",
                !self.secure_value.is_empty(),
                self.enabled,
                self.amount,
                self.host.snapshot().map(|snapshot| &snapshot.stamp)
            )
        } else {
            format!(
                "Owner: text={} enabled={} amount={} · mounted stamp: {:?}",
                self.text,
                self.enabled,
                self.amount,
                self.host.snapshot().map(|snapshot| &snapshot.stamp)
            )
        };
        let status_text = format!(
            "{} · activations {} · parent rerenders {}",
            self.status, self.activations, self.renders
        );
        let rejection_announcement =
            preview_inspector_rejection_live_region_value(&self.status).unwrap_or_default();

        VStack::new()
            .spacing(8.0)
            .child(Text("Compiled preview host"))
            .child(Text(owner_summary))
            .child(
                Semantics("compiled-preview-status", Text(status_text))
                    .live_region_value(rejection_announcement, LivePoliteness::Polite),
            )
            .child(
                VStack::new()
                    .spacing(6.0)
                    .child(Button(
                        "Preview Text specimen",
                        rustui::runtime::Action::new(Self::mount_text_specimen),
                    ))
                    .child(Button(
                        "Preview LocalizedText specimen",
                        rustui::runtime::Action::new(Self::mount_localized_text_specimen),
                    ))
                    .child(Button(
                        "Preview Text",
                        rustui::runtime::Action::new(Self::mount_text),
                    ))
                    .child(Button(
                        "Preview Button",
                        rustui::runtime::Action::new(Self::mount_button),
                    ))
                    .child(Button(
                        "Preview TextField",
                        rustui::runtime::Action::new(Self::mount_field),
                    ))
                    .child(Button(
                        "Preview Toggle",
                        rustui::runtime::Action::new(Self::mount_toggle),
                    ))
                    .child(Button(
                        "Preview Checkbox",
                        rustui::runtime::Action::new(Self::mount_checkbox),
                    ))
                    .child(Button(
                        "Preview Radio",
                        rustui::runtime::Action::new(Self::mount_radio),
                    ))
                    .child(Button(
                        "Preview Picker",
                        rustui::runtime::Action::new(Self::mount_picker),
                    ))
                    .child(Button(
                        "Preview Slider",
                        rustui::runtime::Action::new(Self::mount_slider),
                    ))
                    .child(Button(
                        "Preview Progress",
                        rustui::runtime::Action::new(Self::mount_progress),
                    ))
                    .child(Button(
                        "Preview Stepper",
                        rustui::runtime::Action::new(Self::mount_stepper),
                    )),
            )
            .child(Text("Schema inspector"))
            .child(
                PropertyInspector("compiled-preview-inspector", inspector_rows)
                    .expect("schema property IDs are unique and valid"),
            )
            .child(
                VStack::new()
                    .spacing(4.0)
                    .child(Button(
                        "Rerender preview parent",
                        rustui::runtime::Action::new(Self::rerender),
                    ))
                    .child(OwnerTaskReader(
                        |tasks: OwnerTask<CompiledPreviewGallery>| {
                            Button(
                                "Schedule no-blur parent rerender (5 seconds)",
                                tasks.start_with_executor(
                                    "preview-parent-rerender",
                                    |executor| {
                                        let timer = executor.timer(Duration::from_secs(5));
                                        async move {
                                            timer.await;
                                            Ok::<(), String>(())
                                        }
                                    },
                                    CompiledPreviewGallery::finish_delayed_rerender,
                                ),
                            )
                            .id("compiled-preview-no-blur-rerender")
                        },
                    ))
                    .child(Button(
                        "Preview TextField specimen",
                        rustui::runtime::Action::new(Self::mount_text_field_specimen),
                    ))
                    .child(Button(
                        "Preview TokenField specimen",
                        rustui::runtime::Action::new(Self::mount_token_field_specimen),
                    ))
                    .child(Button(
                        "Preview SearchField specimen",
                        rustui::runtime::Action::new(Self::mount_search_field_specimen),
                    ))
                    .child(Button(
                        "Preview TextEditor specimen",
                        rustui::runtime::Action::new(Self::mount_text_editor_specimen),
                    ))
                    .child(Button(
                        "Preview CodeEditor specimen",
                        rustui::runtime::Action::new(Self::mount_code_editor_specimen),
                    ))
                    .child(Button(
                        "Preview SecureField specimen",
                        rustui::runtime::Action::new(Self::mount_secure_field_specimen),
                    ))
                    .child(Button(
                        "Preview ComboBox specimen",
                        rustui::runtime::Action::new(Self::mount_combo_box_specimen),
                    ))
                    .child(Button(
                        "Preview TabView specimen",
                        rustui::runtime::Action::new(Self::mount_tab_view_specimen),
                    ))
                    .child(Button(
                        "Preview Button specimen",
                        rustui::runtime::Action::new(Self::mount_button_specimen),
                    ))
                    .child(Button(
                        "Preview Toolbar specimen",
                        rustui::runtime::Action::new(Self::mount_toolbar_specimen),
                    ))
                    .child(Button(
                        "Preview Image specimen",
                        rustui::runtime::Action::new(Self::mount_image_specimen),
                    ))
                    .child(Button(
                        "Preview Breadcrumb specimen",
                        rustui::runtime::Action::new(Self::mount_breadcrumb_specimen),
                    ))
                    .child(Button(
                        "Preview ContextMenu specimen",
                        rustui::runtime::Action::new(Self::mount_context_menu_specimen),
                    ))
                    .child(Button(
                        "Preview Menu specimen",
                        rustui::runtime::Action::new(Self::mount_menu_specimen),
                    ))
                    .child(Button(
                        "Preview Link specimen",
                        rustui::runtime::Action::new(Self::mount_link_specimen),
                    ))
                    .child(Button(
                        "Preview Dialog specimen",
                        rustui::runtime::Action::new(Self::mount_dialog_specimen),
                    ))
                    .child(Button(
                        "Preview Popover specimen",
                        rustui::runtime::Action::new(Self::mount_popover_specimen),
                    ))
                    .child(Button(
                        "Preview EmptyState specimen",
                        rustui::runtime::Action::new(Self::mount_empty_state_specimen),
                    ))
                    .child(Button(
                        "Preview Alert specimen",
                        rustui::runtime::Action::new(Self::mount_alert_specimen),
                    ))
                    .child(Button(
                        "Preview Toast specimen",
                        rustui::runtime::Action::new(Self::mount_toast_specimen),
                    ))
                    .child(Button(
                        "Preview Sheet specimen",
                        rustui::runtime::Action::new(Self::mount_sheet_specimen),
                    ))
                    .child(Button(
                        "Preview Sidebar specimen",
                        rustui::runtime::Action::new(Self::mount_sidebar_specimen),
                    ))
                    .child(Button(
                        "Preview KeyedList specimen",
                        rustui::runtime::Action::new(Self::mount_keyed_list_specimen),
                    ))
                    .child(Button(
                        "Preview DataGrid specimen",
                        rustui::runtime::Action::new(Self::mount_data_grid_specimen),
                    ))
                    .child(Button(
                        "Preview List specimen",
                        rustui::runtime::Action::new(Self::mount_list_specimen),
                    ))
                    .child(Button(
                        "Preview Table specimen",
                        rustui::runtime::Action::new(Self::mount_table_specimen),
                    ))
                    .child(Button(
                        "Preview Tree specimen",
                        rustui::runtime::Action::new(Self::mount_tree_specimen),
                    ))
                    .child(Button(
                        "Preview NavigationSplitView specimen",
                        rustui::runtime::Action::new(Self::mount_navigation_split_specimen),
                    ))
                    .child(Button(
                        "Preview NavigationStack specimen",
                        rustui::runtime::Action::new(Self::mount_navigation_stack_specimen),
                    ))
                    .child(Button(
                        "Preview Collection specimen",
                        rustui::runtime::Action::new(Self::mount_collection_specimen),
                    ))
                    .child(Button(
                        "Preview Canvas specimen",
                        rustui::runtime::Action::new(Self::mount_canvas_specimen),
                    ))
                    .child(Button(
                        "Preview Group specimen",
                        rustui::runtime::Action::new(Self::mount_group_specimen),
                    ))
                    .child(Button(
                        "Preview AnimatedOpacity specimen",
                        rustui::runtime::Action::new(Self::mount_animated_opacity_specimen),
                    ))
                    .child(Button(
                        "Preview SpringOpacity specimen",
                        rustui::runtime::Action::new(Self::mount_spring_opacity_specimen),
                    ))
                    .child(Button(
                        "Preview Shadow specimen",
                        rustui::runtime::Action::new(Self::mount_shadow_specimen),
                    ))
                    .child(Button(
                        "Preview Background specimen",
                        rustui::runtime::Action::new(Self::mount_background_specimen),
                    ))
                    .child(Button(
                        "Preview Border specimen",
                        rustui::runtime::Action::new(Self::mount_border_specimen),
                    ))
                    .child(Button(
                        "Preview Semantics specimen",
                        rustui::runtime::Action::new(Self::mount_semantics_specimen),
                    ))
                    .child(Button(
                        "Preview Card specimen",
                        rustui::runtime::Action::new(Self::mount_card_specimen),
                    ))
                    .child(Button(
                        "Preview Container specimen",
                        rustui::runtime::Action::new(Self::mount_container_specimen),
                    ))
                    .child(Button(
                        "Preview SplitView specimen",
                        rustui::runtime::Action::new(Self::mount_split_view_specimen),
                    ))
                    .child(Button(
                        "Preview Frame specimen",
                        rustui::runtime::Action::new(Self::mount_frame_specimen),
                    ))
                    .child(Button(
                        "Preview Spacer specimen",
                        rustui::runtime::Action::new(Self::mount_spacer_specimen),
                    ))
                    .child(Button(
                        "Preview HStack specimen",
                        rustui::runtime::Action::new(Self::mount_hstack_specimen),
                    ))
                    .child(Button(
                        "Preview VStack specimen",
                        rustui::runtime::Action::new(Self::mount_vstack_specimen),
                    ))
                    .child(Button(
                        "Preview Grid specimen",
                        rustui::runtime::Action::new(Self::mount_grid_specimen),
                    ))
                    .child(Button(
                        "Preview LazyGrid specimen",
                        rustui::runtime::Action::new(Self::mount_lazy_grid_specimen),
                    ))
                    .child(Button(
                        "Preview GeometryReader specimen",
                        rustui::runtime::Action::new(Self::mount_geometry_reader_specimen),
                    ))
                    .child(Button(
                        "Preview PropertyInspector specimen",
                        rustui::runtime::Action::new(Self::mount_property_inspector_specimen),
                    ))
                    .child(Button(
                        "Preview Disclosure specimen",
                        rustui::runtime::Action::new(Self::mount_disclosure_specimen),
                    ))
                    .child(Button(
                        "Preview ErrorState specimen",
                        rustui::runtime::Action::new(Self::mount_error_state_specimen),
                    ))
                    .child(Button(
                        "Preview DatePicker specimen",
                        rustui::runtime::Action::new(Self::mount_date_picker_specimen),
                    ))
                    .child(Button(
                        "Preview Badge specimen",
                        rustui::runtime::Action::new(Self::mount_badge_specimen),
                    ))
                    .child(Button(
                        "Preview Label specimen",
                        rustui::runtime::Action::new(Self::mount_label_specimen),
                    ))
                    .child(Button(
                        "Preview Tooltip specimen",
                        rustui::runtime::Action::new(Self::mount_tooltip_specimen),
                    ))
                    .child(Button(
                        "Preview Spinner specimen",
                        rustui::runtime::Action::new(Self::mount_spinner_specimen),
                    ))
                    .child(Button(
                        "Preview Divider specimen",
                        rustui::runtime::Action::new(Self::mount_divider_specimen),
                    ))
                    .child(Button(
                        "Preview Skeleton specimen",
                        rustui::runtime::Action::new(Self::mount_skeleton_specimen),
                    ))
                    .child(Button(
                        "Preview Icon specimen",
                        rustui::runtime::Action::new(Self::mount_icon_specimen),
                    ))
                    .child(Button(
                        "Preview Avatar specimen",
                        rustui::runtime::Action::new(Self::mount_avatar_specimen),
                    ))
                    .child(Button(
                        "Preview ColorPicker specimen",
                        rustui::runtime::Action::new(Self::mount_color_picker_specimen),
                    ))
                    .child(Button(
                        "Teardown preview",
                        rustui::runtime::Action::new(Self::teardown),
                    )),
            )
            .child(Text("Preview descriptor JSON"))
            .child(
                HStack::new()
                    .spacing(6.0)
                    .child(Button(
                        "Import descriptor",
                        rustui::runtime::Action::new(Self::import_descriptor),
                    ))
                    .child(Button(
                        "Export normalized descriptor",
                        rustui::runtime::Action::new(Self::export_descriptor),
                    )),
            )
            .child(OwnerActionReader(
                move |owner: OwnerAction<CompiledPreviewGallery>| {
                    KeyboardShortcuts(
                        "compiled-preview-import-shortcut",
                        TextEditor(
                            "Import or export normalized preview JSON",
                            descriptor_binding.clone(),
                        )
                        .id("compiled-preview-descriptor-json")
                        .height(150.0),
                    )
                    .shortcut(
                        "secondary-shift-enter",
                        owner.action(Self::import_descriptor),
                    )
                },
            ))
            .child(
                HStack::new()
                    .spacing(6.0)
                    .child(Button(
                        "Capture amount callback",
                        rustui::runtime::Action::new(Self::capture_stale_amount_callback),
                    ))
                    .child(Button(
                        "Replay captured amount callback",
                        rustui::runtime::Action::new(Self::replay_stale_amount_callback),
                    ))
                    .child(Button(
                        "Reject stale preview callbacks",
                        rustui::runtime::Action::new(Self::reject_stale),
                    ))
                    .child(Button(
                        "Reject wrong preview type",
                        rustui::runtime::Action::new(Self::reject_wrong_type),
                    )),
            )
            .child(OwnerActionReader(
                move |owner: OwnerAction<CompiledPreviewGallery>| {
                    render_preview(&FactoryContext {
                        owner,
                        stamp_lease: stamp_lease.clone(),
                        text: binding.clone(),
                        secure_value: secure_value_binding.clone(),
                        checked: checked_binding.clone(),
                        slider_value: slider_binding.clone(),
                        stepper_value: stepper_binding.clone(),
                        color_value: color_value_binding.clone(),
                        date_value: date_value_binding.clone(),
                        token_values: token_values_binding.clone(),
                    })
                    .expect("registered compiled factory")
                },
            ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustui::localization::{Catalog, Catalogs, Locale};

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
        assert_eq!(expected.len(), 74);
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
        gallery.descriptor_json =
            encode_preview(&invalid_document, gallery.host.catalog()).unwrap();
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
        let current_value =
            gallery.host.snapshot().unwrap().preview.properties[&checked_id].clone();

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
        let previous_amount =
            gallery.host.snapshot().unwrap().preview.properties[&amount_id].clone();

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
            ("en", include_str!("../../../../locales/en/rustui.ftl")),
            ("fr", include_str!("../../../../locales/fr/rustui.ftl")),
            ("ar", include_str!("../../../../locales/ar/rustui.ftl")),
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
    }

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
        assert_eq!(schema.properties.len(), 6);
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

    #[test]
    fn toolbar_specimen_maps_signal_only_actions_and_round_trips() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_toolbar_specimen();
        let component_id = ComponentId::new("toolbar-specimen").unwrap();
        let primary_id = EventId::new("primary").unwrap();
        let secondary_id = EventId::new("secondary").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        for name in ["primary_label", "secondary_label"] {
            let property = &schema.properties[&PropertyId::new(name).unwrap()];
            assert_eq!(property.kind, PropertyKind::String);
            assert_eq!(property.min_length, Some(1));
            assert_eq!(property.max_length, Some(PREVIEW_TOOLBAR_LABEL_MAX_LENGTH));
        }
        let disabled_id = PropertyId::new("secondary_disabled").unwrap();
        assert_eq!(schema.properties[&disabled_id].kind, PropertyKind::Boolean);
        assert_eq!(schema.events.len(), 2);
        assert!(schema.events[&primary_id].payload.is_empty());
        assert!(schema.events[&secondary_id].payload.is_empty());

        let mounted = gallery.host.snapshot().unwrap().clone();
        assert_eq!(
            mounted.preview.document.actions[&primary_id],
            ActionRef::new("toolbar_primary").unwrap()
        );
        assert_eq!(
            mounted.preview.document.actions[&secondary_id],
            ActionRef::new("toolbar_secondary").unwrap()
        );
        assert_eq!(
            gallery
                .host
                .dispatch(&mounted.signal(primary_id.clone()))
                .unwrap(),
            ActionRef::new("toolbar_primary").unwrap()
        );
        assert_eq!(
            gallery
                .host
                .dispatch(&mounted.signal(secondary_id.clone()))
                .unwrap(),
            ActionRef::new("toolbar_secondary").unwrap()
        );

        let mut customized = mounted.preview.document.clone();
        customized.properties.insert(
            PropertyId::new("primary_label").unwrap(),
            PropertyValue::String("Apply".into()),
        );
        customized.properties.insert(
            PropertyId::new("secondary_label").unwrap(),
            PropertyValue::String("Later".into()),
        );
        customized
            .properties
            .insert(disabled_id.clone(), PropertyValue::Boolean(true));
        gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap();
        assert_eq!(
            imported.preview.properties[&PropertyId::new("primary_label").unwrap()],
            PropertyValue::String("Apply".into())
        );
        assert_eq!(
            imported.preview.properties[&PropertyId::new("secondary_label").unwrap()],
            PropertyValue::String("Later".into())
        );
        assert_eq!(
            imported.preview.properties[&disabled_id],
            PropertyValue::Boolean(true)
        );

        let before = imported.clone();
        let mut invalid = before.preview.document.clone();
        invalid.properties.insert(
            PropertyId::new("primary_label").unwrap(),
            PropertyValue::String("  ".into()),
        );
        gallery.descriptor_json = encode_preview(&invalid, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        assert!(
            gallery
                .status
                .contains("Toolbar primary_label must not be blank")
        );
        assert_eq!(gallery.host.snapshot().unwrap().stamp, before.stamp);
    }

    #[test]
    fn image_specimen_uses_bounded_label_and_constrained_fit_round_trip() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_image_specimen();

        let component_id = ComponentId::new("image-specimen").unwrap();
        let label_id = PropertyId::new("accessibility_label").unwrap();
        let fit_id = PropertyId::new("fit").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 2);
        assert_eq!(schema.properties[&label_id].kind, PropertyKind::String);
        assert_eq!(schema.properties[&label_id].min_length, Some(1));
        assert_eq!(
            schema.properties[&label_id].max_length,
            Some(PREVIEW_IMAGE_LABEL_MAX_LENGTH)
        );
        assert_eq!(
            schema.properties[&label_id].default,
            Some(PropertyValue::String("Purple and cyan sample image".into()))
        );
        assert_eq!(schema.properties[&fit_id].kind, PropertyKind::String);
        assert_eq!(
            schema.properties[&fit_id].default,
            Some(PropertyValue::String("contain".into()))
        );
        assert_eq!(
            schema.properties[&fit_id].allowed_values,
            PREVIEW_IMAGE_FITS
                .iter()
                .map(|fit| (*fit).into())
                .collect::<Vec<String>>()
        );
        assert!(schema.events.is_empty());

        let mounted = gallery.host.snapshot().unwrap();
        assert_eq!(mounted.preview.document.properties.len(), 2);
        assert_eq!(mounted.preview.document.actions.len(), 0);
        assert_eq!(
            mounted.preview.properties[&label_id],
            PropertyValue::String("Purple and cyan sample image".into())
        );
        assert_eq!(
            mounted.preview.properties[&fit_id],
            PropertyValue::String("contain".into())
        );

        let mut customized = mounted.preview.document.clone();
        customized.properties.insert(
            label_id.clone(),
            PropertyValue::String("Sample artwork".into()),
        );
        customized
            .properties
            .insert(fit_id.clone(), PropertyValue::String("cover".into()));
        gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap();
        assert_eq!(
            imported.preview.properties[&label_id],
            PropertyValue::String("Sample artwork".into())
        );
        assert_eq!(
            imported.preview.properties[&fit_id],
            PropertyValue::String("cover".into())
        );

        let before = imported.clone();
        let mut invalid = before.preview.document.clone();
        invalid
            .properties
            .insert(label_id, PropertyValue::String(" \t".into()));
        gallery.descriptor_json = encode_preview(&invalid, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        assert!(
            gallery
                .status
                .contains("Image accessibility label must not be blank")
        );
        assert_eq!(gallery.host.snapshot().unwrap().stamp, before.stamp);
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.document,
            before.preview.document
        );
    }

    #[test]
    fn breadcrumb_specimen_dispatches_parent_and_round_trips_fixed_labels() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_breadcrumb_specimen();

        let component_id = ComponentId::new("breadcrumb-specimen").unwrap();
        let parent_id = PropertyId::new("parent_label").unwrap();
        let current_id = PropertyId::new("current_label").unwrap();
        let event_id = EventId::new("parent").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 2);
        for (id, default) in [(&parent_id, "Components"), (&current_id, "Image")] {
            let property = &schema.properties[id];
            assert_eq!(property.kind, PropertyKind::String);
            assert_eq!(property.min_length, Some(1));
            assert_eq!(
                property.max_length,
                Some(PREVIEW_BREADCRUMB_LABEL_MAX_LENGTH)
            );
            assert_eq!(
                property.default,
                Some(PropertyValue::String(default.into()))
            );
        }
        assert_eq!(schema.events.len(), 1);
        assert!(schema.events[&event_id].payload.is_empty());

        let mounted = gallery.host.snapshot().unwrap().clone();
        assert_eq!(mounted.preview.document.properties.len(), 2);
        assert_eq!(
            mounted.preview.document.actions.get(&event_id),
            Some(&ActionRef::new("breadcrumb_parent").unwrap())
        );
        gallery.navigate_breadcrumb_parent(&mounted.signal(event_id.clone()));
        assert_eq!(gallery.activations, 1);
        assert!(gallery.status.contains("Allowlisted Breadcrumb parent"));

        let mut customized = mounted.preview.document.clone();
        customized.properties.insert(
            parent_id.clone(),
            PropertyValue::String("Design system".into()),
        );
        customized.properties.insert(
            current_id.clone(),
            PropertyValue::String("Components".into()),
        );
        gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap();
        assert_eq!(
            imported.preview.properties[&parent_id],
            PropertyValue::String("Design system".into())
        );
        assert_eq!(
            imported.preview.properties[&current_id],
            PropertyValue::String("Components".into())
        );

        let before = imported.clone();
        for name in ["parent_label", "current_label"] {
            let mut invalid = before.preview.document.clone();
            invalid.properties.insert(
                PropertyId::new(name).unwrap(),
                PropertyValue::String(" \t".into()),
            );
            gallery.descriptor_json = encode_preview(&invalid, gallery.host.catalog()).unwrap();
            gallery.import_descriptor();
            assert!(
                gallery
                    .status
                    .contains(&format!("Breadcrumb {name} must not be blank"))
            );
            assert_eq!(gallery.host.snapshot().unwrap().stamp, before.stamp);
            assert_eq!(
                gallery.host.snapshot().unwrap().preview.document,
                before.preview.document
            );
        }
    }

    #[test]
    fn text_specimen_round_trips_bounded_value_without_events_or_actions() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_text_specimen();

        let component_id = ComponentId::new("text-specimen").unwrap();
        let value_id = PropertyId::new("value").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 1);
        assert_eq!(schema.properties[&value_id].kind, PropertyKind::String);
        assert_eq!(schema.properties[&value_id].min_length, Some(1));
        assert_eq!(
            schema.properties[&value_id].max_length,
            Some(PREVIEW_TEXT_VALUE_MAX_LENGTH)
        );
        assert_eq!(
            schema.properties[&value_id].default,
            Some(PropertyValue::String("Preview text".into()))
        );
        assert!(schema.events.is_empty());

        let mounted = gallery.host.snapshot().unwrap().clone();
        assert_eq!(mounted.preview.document.properties.len(), 1);
        assert!(mounted.preview.document.actions.is_empty());
        assert_eq!(
            mounted.preview.properties[&value_id],
            PropertyValue::String("Preview text".into())
        );

        let mut customized = mounted.preview.document.clone();
        customized.properties.insert(
            value_id.clone(),
            PropertyValue::String("Compiled text preview".into()),
        );
        gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&value_id],
            PropertyValue::String("Compiled text preview".into())
        );

        let before = gallery.host.snapshot().unwrap().clone();
        let mut invalid = before.preview.document.clone();
        invalid
            .properties
            .insert(value_id.clone(), PropertyValue::String(" \t".into()));
        gallery.descriptor_json = encode_preview(&invalid, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        assert!(gallery.status.contains("Text value must not be blank"));
        assert_eq!(gallery.host.snapshot().unwrap().stamp, before.stamp);
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.document,
            before.preview.document
        );
    }

    #[test]
    fn sheet_specimen_round_trips_bounded_strings_and_rejects_blank_imports() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_sheet_specimen();

        let component_id = ComponentId::new("sheet-specimen").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 3);
        assert!(schema.events.is_empty());
        for (name, default, max_length) in [
            (
                "trigger_label",
                "Open details",
                PREVIEW_SHEET_TRIGGER_MAX_LENGTH,
            ),
            ("title", "Details", PREVIEW_SHEET_TITLE_MAX_LENGTH),
            (
                "body_text",
                "Preview sheet content",
                PREVIEW_SHEET_BODY_MAX_LENGTH,
            ),
        ] {
            let property = &schema.properties[&PropertyId::new(name).unwrap()];
            assert_eq!(property.kind, PropertyKind::String);
            assert_eq!(property.min_length, Some(1));
            assert_eq!(property.max_length, Some(max_length));
            assert_eq!(
                property.default,
                Some(PropertyValue::String(default.into()))
            );
        }

        let mounted = gallery.host.snapshot().unwrap().clone();
        assert_eq!(mounted.preview.document.properties.len(), 3);
        assert!(mounted.preview.document.actions.is_empty());
        let mut customized = mounted.preview.document.clone();
        customized.properties.insert(
            PropertyId::new("trigger_label").unwrap(),
            PropertyValue::String("Open project details".into()),
        );
        customized.properties.insert(
            PropertyId::new("title").unwrap(),
            PropertyValue::String("Project details".into()),
        );
        customized.properties.insert(
            PropertyId::new("body_text").unwrap(),
            PropertyValue::String("Build and test status".into()),
        );
        gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        gallery.export_descriptor();
        let exported = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
        for (name, value) in [
            ("trigger_label", "Open project details"),
            ("title", "Project details"),
            ("body_text", "Build and test status"),
        ] {
            assert_eq!(
                exported.properties[&PropertyId::new(name).unwrap()],
                PropertyValue::String(value.into())
            );
        }

        let before = gallery.host.snapshot().unwrap().clone();
        for name in ["trigger_label", "title", "body_text"] {
            let mut invalid = before.preview.document.clone();
            invalid.properties.insert(
                PropertyId::new(name).unwrap(),
                PropertyValue::String(" \t".into()),
            );
            gallery.descriptor_json = encode_preview(&invalid, gallery.host.catalog()).unwrap();
            gallery.import_descriptor();
            assert!(
                gallery
                    .status
                    .contains(&format!("Sheet {name} must not be blank"))
            );
            let current = gallery.host.snapshot().unwrap();
            assert_eq!(current.stamp, before.stamp);
            assert_eq!(current.preview.document, before.preview.document);
        }
    }

    #[test]
    fn sidebar_specimen_round_trips_labels_and_selection_and_rejects_invalid_import() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_sidebar_specimen();

        let component_id = ComponentId::new("sidebar-specimen").unwrap();
        let home_label_id = PropertyId::new("home_label").unwrap();
        let settings_label_id = PropertyId::new("settings_label").unwrap();
        let selected_id = PropertyId::new("selected_id").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 3);
        assert!(schema.events.is_empty());
        for (name, default) in [("home_label", "Home"), ("settings_label", "Settings")] {
            let property = &schema.properties[&PropertyId::new(name).unwrap()];
            assert_eq!(property.kind, PropertyKind::String);
            assert_eq!(property.min_length, Some(1));
            assert_eq!(property.max_length, Some(PREVIEW_SIDEBAR_LABEL_MAX_LENGTH));
            assert_eq!(
                property.default,
                Some(PropertyValue::String(default.into()))
            );
        }
        assert_eq!(schema.properties[&selected_id].kind, PropertyKind::String);
        assert_eq!(
            schema.properties[&selected_id].default,
            Some(PropertyValue::String("home".into()))
        );
        assert_eq!(
            schema.properties[&selected_id].allowed_values,
            vec!["home".to_owned(), "settings".to_owned()]
        );

        let mounted = gallery.host.snapshot().unwrap().clone();
        assert_eq!(mounted.preview.document.properties.len(), 3);
        assert!(mounted.preview.document.actions.is_empty());
        assert_eq!(
            mounted.preview.document.properties[&home_label_id],
            PropertyValue::String("Home".into())
        );
        assert_eq!(
            mounted.preview.document.properties[&settings_label_id],
            PropertyValue::String("Settings".into())
        );
        assert_eq!(
            mounted.preview.document.properties[&selected_id],
            PropertyValue::String("home".into())
        );
        let stamp_lease = gallery.stamp_lease.clone();
        let home_context = PropertyBindingContext {
            property: home_label_id.clone(),
            stamp: stamp_lease.clone(),
        };
        let settings_context = PropertyBindingContext {
            property: settings_label_id.clone(),
            stamp: stamp_lease.clone(),
        };
        let before_blank_edit = gallery.host.snapshot().unwrap().clone();
        assert!(!gallery.write_text(&home_context, " \t ".into()));
        assert!(!gallery.write_text(&settings_context, " \t ".into()));
        let after_blank_edit = gallery.host.snapshot().unwrap();
        assert_eq!(after_blank_edit.stamp, before_blank_edit.stamp);
        assert_eq!(
            after_blank_edit.preview.document,
            before_blank_edit.preview.document
        );
        assert!(gallery.write_text(&home_context, "Start".into()));
        assert!(gallery.write_text(&settings_context, "Preferences".into()));

        let selection_context = PropertyBindingContext {
            property: selected_id.clone(),
            stamp: stamp_lease.clone(),
        };
        let selection = CompiledPreviewGallery::string_selection_binding(
            Some("home".into()),
            selection_context,
        );
        assert!(selection.write_to(&mut gallery, Some("settings".into())));
        assert_eq!(gallery.text, "settings");
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.document.properties[&selected_id],
            PropertyValue::String("settings".into())
        );

        gallery.export_descriptor();
        let exported = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
        assert_eq!(
            exported.properties[&home_label_id],
            PropertyValue::String("Start".into())
        );
        assert_eq!(
            exported.properties[&settings_label_id],
            PropertyValue::String("Preferences".into())
        );
        assert_eq!(
            exported.properties[&selected_id],
            PropertyValue::String("settings".into())
        );
        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap().clone();
        assert_eq!(imported.stamp.preview_id.as_str(), "compiled-gallery");
        assert_eq!(imported.preview.document.properties.len(), 3);
        assert!(imported.preview.document.actions.is_empty());

        let before_invalid_import = imported.clone();
        let invalid_descriptor = gallery
            .descriptor_json
            .replacen("\"settings\"", "\"unknown\"", 1);
        assert_ne!(invalid_descriptor, gallery.descriptor_json);
        gallery.descriptor_json = invalid_descriptor;
        gallery.import_descriptor();
        assert!(gallery.status.starts_with("Import rejected:"));
        let after_invalid_import = gallery.host.snapshot().unwrap();
        assert_eq!(after_invalid_import.stamp, before_invalid_import.stamp);
        assert_eq!(
            after_invalid_import.preview.document,
            before_invalid_import.preview.document
        );

        gallery.export_descriptor();
        for name in ["home_label", "settings_label"] {
            let invalid_descriptor = gallery.descriptor_json.replacen(
                if name == "home_label" {
                    "\"Start\""
                } else {
                    "\"Preferences\""
                },
                "\" \"",
                1,
            );
            assert_ne!(invalid_descriptor, gallery.descriptor_json);
            gallery.descriptor_json = invalid_descriptor;
            gallery.import_descriptor();
            assert!(
                gallery
                    .status
                    .contains(&format!("Sidebar {name} must not be blank"))
            );
            let after_blank_label = gallery.host.snapshot().unwrap();
            assert_eq!(after_blank_label.stamp, before_invalid_import.stamp);
            assert_eq!(
                after_blank_label.preview.document,
                before_invalid_import.preview.document
            );
            gallery.export_descriptor();
        }
    }

    #[test]
    fn keyed_list_specimen_round_trips_labels_and_selection_and_rejects_invalid_import() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_keyed_list_specimen();

        let component_id = ComponentId::new("keyed-list-specimen").unwrap();
        let first_label_id = PropertyId::new("first_label").unwrap();
        let second_label_id = PropertyId::new("second_label").unwrap();
        let selected_id = PropertyId::new("selected_id").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 3);
        assert!(schema.events.is_empty());
        for (name, default) in [
            ("first_label", "First item"),
            ("second_label", "Second item"),
        ] {
            let property = &schema.properties[&PropertyId::new(name).unwrap()];
            assert_eq!(property.kind, PropertyKind::String);
            assert_eq!(property.min_length, Some(1));
            assert_eq!(
                property.max_length,
                Some(PREVIEW_KEYED_LIST_LABEL_MAX_LENGTH)
            );
            assert_eq!(
                property.default,
                Some(PropertyValue::String(default.into()))
            );
        }
        assert_eq!(schema.properties[&selected_id].kind, PropertyKind::String);
        assert_eq!(
            schema.properties[&selected_id].default,
            Some(PropertyValue::String("first".into()))
        );
        assert_eq!(
            schema.properties[&selected_id].allowed_values,
            vec!["first".to_owned(), "second".to_owned()]
        );

        let mounted = gallery.host.snapshot().unwrap().clone();
        assert_eq!(mounted.preview.document.properties.len(), 3);
        assert!(mounted.preview.document.actions.is_empty());
        assert_eq!(
            mounted.preview.document.properties[&first_label_id],
            PropertyValue::String("First item".into())
        );
        assert_eq!(
            mounted.preview.document.properties[&second_label_id],
            PropertyValue::String("Second item".into())
        );
        assert_eq!(
            mounted.preview.document.properties[&selected_id],
            PropertyValue::String("first".into())
        );

        let stamp_lease = gallery.stamp_lease.clone();
        let first_context = PropertyBindingContext {
            property: first_label_id.clone(),
            stamp: stamp_lease.clone(),
        };
        let second_context = PropertyBindingContext {
            property: second_label_id.clone(),
            stamp: stamp_lease.clone(),
        };
        let before_blank_edit = gallery.host.snapshot().unwrap().clone();
        assert!(!gallery.write_text(&first_context, " \t ".into()));
        assert!(!gallery.write_text(&second_context, " \t ".into()));
        let after_blank_edit = gallery.host.snapshot().unwrap();
        assert_eq!(after_blank_edit.stamp, before_blank_edit.stamp);
        assert_eq!(
            after_blank_edit.preview.document,
            before_blank_edit.preview.document
        );
        assert!(gallery.write_text(&first_context, "Primary".into()));
        assert!(gallery.write_text(&second_context, "Secondary".into()));

        let selection = CompiledPreviewGallery::string_selection_binding(
            Some("first".into()),
            PropertyBindingContext {
                property: selected_id.clone(),
                stamp: stamp_lease,
            },
        );
        assert!(selection.write_to(&mut gallery, Some("second".into())));
        assert_eq!(gallery.text, "second");
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.document.properties[&selected_id],
            PropertyValue::String("second".into())
        );

        gallery.export_descriptor();
        let exported = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
        assert_eq!(
            exported.properties[&first_label_id],
            PropertyValue::String("Primary".into())
        );
        assert_eq!(
            exported.properties[&second_label_id],
            PropertyValue::String("Secondary".into())
        );
        assert_eq!(
            exported.properties[&selected_id],
            PropertyValue::String("second".into())
        );
        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap().clone();
        assert_eq!(imported.stamp.preview_id.as_str(), "compiled-gallery");
        assert_eq!(imported.preview.document.properties.len(), 3);
        assert!(imported.preview.document.actions.is_empty());

        let before_invalid_import = imported.clone();
        let invalid_descriptor = gallery
            .descriptor_json
            .replacen("\"second\"", "\"unknown\"", 1);
        assert_ne!(invalid_descriptor, gallery.descriptor_json);
        gallery.descriptor_json = invalid_descriptor;
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
    fn toast_specimen_round_trips_bounded_message_and_optional_title() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_toast_specimen();

        let component_id = ComponentId::new("toast-specimen").unwrap();
        let message_id = PropertyId::new("message").unwrap();
        let title_id = PropertyId::new("title").unwrap();
        let show_id = EventId::new("show").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 2);
        let message = &schema.properties[&message_id];
        assert_eq!(message.kind, PropertyKind::String);
        assert_eq!(message.min_length, Some(1));
        assert_eq!(message.max_length, Some(PREVIEW_TOAST_MESSAGE_MAX_LENGTH));
        assert_eq!(
            message.default,
            Some(PropertyValue::String("Changes saved".into()))
        );
        let title = &schema.properties[&title_id];
        assert_eq!(title.kind, PropertyKind::String);
        assert_eq!(title.min_length, Some(1));
        assert_eq!(title.max_length, Some(PREVIEW_TOAST_TITLE_MAX_LENGTH));
        assert_eq!(title.default, None);
        assert!(!title.required);
        assert_eq!(schema.events.len(), 1);
        assert!(schema.events[&show_id].payload.is_empty());

        let mounted = gallery.host.snapshot().unwrap().clone();
        assert_eq!(mounted.preview.document.properties.len(), 1);
        assert_eq!(mounted.preview.document.actions.len(), 1);
        assert_eq!(
            mounted.preview.document.actions.get(&show_id),
            Some(&ActionRef::new("toast_show").unwrap())
        );
        gallery.show_toast(&mounted.signal(show_id.clone()));
        assert_eq!(gallery.activations, 1);
        assert_eq!(gallery.status, "Allowlisted Toast show action");

        let mut customized = mounted.preview.document.clone();
        customized.properties.insert(
            message_id.clone(),
            PropertyValue::String("All changes are saved".into()),
        );
        customized.properties.insert(
            title_id.clone(),
            PropertyValue::String("Project updated".into()),
        );
        gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        gallery.export_descriptor();
        let exported = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
        assert_eq!(
            exported.properties[&message_id],
            PropertyValue::String("All changes are saved".into())
        );
        assert_eq!(
            exported.properties[&title_id],
            PropertyValue::String("Project updated".into())
        );

        let before = gallery.host.snapshot().unwrap().clone();
        for (name, value) in [("message", " \t"), ("title", " \t")] {
            let mut invalid = before.preview.document.clone();
            invalid.properties.insert(
                PropertyId::new(name).unwrap(),
                PropertyValue::String(value.into()),
            );
            gallery.descriptor_json = encode_preview(&invalid, gallery.host.catalog()).unwrap();
            gallery.import_descriptor();
            assert!(
                gallery
                    .status
                    .contains(&format!("Toast {name} must not be blank"))
            );
            let current = gallery.host.snapshot().unwrap();
            assert_eq!(current.stamp, before.stamp);
            assert_eq!(current.preview.document, before.preview.document);
        }
    }

    #[test]
    fn optional_toast_title_inspector_initial_value_does_not_use_shared_text() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.text = "unrelated shared text".into();
        gallery.mount_toast_specimen();
        let snapshot = gallery.host.snapshot().unwrap();
        let title = PropertyId::new("title").unwrap();
        assert!(!snapshot.preview.properties.contains_key(&title));

        let initial = preview_inspector_string_initial_value(Some(snapshot), title.as_str());
        assert_eq!(initial, "");
        let context = PropertyBindingContext {
            property: title,
            stamp: gallery.stamp_lease.clone(),
        };
        let binding = Binding::new_with(
            initial,
            context,
            CompiledPreviewGallery::read_text,
            CompiledPreviewGallery::write_text,
        );
        assert_eq!(binding.value(), "");
    }

    #[test]
    fn context_menu_specimen_dispatches_fixed_action_and_round_trips_labels() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_context_menu_specimen();

        let component_id = ComponentId::new("context-menu-specimen").unwrap();
        let target_id = PropertyId::new("target_label").unwrap();
        let action_label_id = PropertyId::new("action_label").unwrap();
        let event_id = EventId::new("activate").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 2);
        for (id, default) in [
            (&target_id, "Preview target"),
            (&action_label_id, "Open preview"),
        ] {
            let property = &schema.properties[id];
            assert_eq!(property.kind, PropertyKind::String);
            assert_eq!(property.min_length, Some(1));
            assert_eq!(
                property.max_length,
                Some(PREVIEW_CONTEXT_MENU_LABEL_MAX_LENGTH)
            );
            assert_eq!(
                property.default,
                Some(PropertyValue::String(default.into()))
            );
        }
        assert_eq!(schema.events.len(), 1);
        assert!(schema.events[&event_id].payload.is_empty());

        let mounted = gallery.host.snapshot().unwrap().clone();
        assert_eq!(mounted.preview.document.properties.len(), 2);
        assert_eq!(
            mounted.preview.document.actions.get(&event_id),
            Some(&ActionRef::new("context_menu_open_preview").unwrap())
        );
        gallery.activate_context_menu_item(&mounted.signal(event_id.clone()));
        assert_eq!(gallery.activations, 1);
        assert!(gallery.status.contains("Allowlisted ContextMenu"));

        let mut customized = mounted.preview.document.clone();
        customized.properties.insert(
            target_id.clone(),
            PropertyValue::String("Sample card".into()),
        );
        customized.properties.insert(
            action_label_id.clone(),
            PropertyValue::String("Open card".into()),
        );
        gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap();
        assert_eq!(
            imported.preview.properties[&target_id],
            PropertyValue::String("Sample card".into())
        );
        assert_eq!(
            imported.preview.properties[&action_label_id],
            PropertyValue::String("Open card".into())
        );

        let before = imported.clone();
        for name in ["target_label", "action_label"] {
            let mut invalid = before.preview.document.clone();
            invalid.properties.insert(
                PropertyId::new(name).unwrap(),
                PropertyValue::String(" \t".into()),
            );
            gallery.descriptor_json = encode_preview(&invalid, gallery.host.catalog()).unwrap();
            gallery.import_descriptor();
            assert!(
                gallery
                    .status
                    .contains(&format!("ContextMenu {name} must not be blank"))
            );
            assert_eq!(gallery.host.snapshot().unwrap().stamp, before.stamp);
            assert_eq!(
                gallery.host.snapshot().unwrap().preview.document,
                before.preview.document
            );
        }
    }

    #[test]
    fn popover_specimen_accepts_controlled_open_feedback_and_preserves_invalid_import() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_popover_specimen();

        let component_id = ComponentId::new("popover-specimen").unwrap();
        let trigger_id = PropertyId::new("trigger_label").unwrap();
        let open_id = PropertyId::new("open").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties[&trigger_id].min_length, Some(1));
        assert_eq!(
            schema.properties[&trigger_id].max_length,
            Some(PREVIEW_POPOVER_TRIGGER_MAX_LENGTH)
        );
        assert_eq!(schema.properties[&open_id].kind, PropertyKind::Boolean);
        assert_eq!(
            schema.properties[&open_id].default,
            Some(PropertyValue::Boolean(false))
        );
        assert!(schema.events.is_empty());

        let open_context = PropertyBindingContext {
            property: open_id.clone(),
            stamp: gallery.stamp_lease.clone(),
        };
        assert!(!gallery.enabled);
        assert!(gallery.write_enabled(&open_context, true));
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&open_id],
            PropertyValue::Boolean(true)
        );

        gallery.export_descriptor();
        gallery.import_descriptor();
        assert!(gallery.enabled);
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&open_id],
            PropertyValue::Boolean(true)
        );
        let round_trip = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
        assert_eq!(
            round_trip.properties[&trigger_id],
            PropertyValue::String("Preview details".into())
        );
        assert_eq!(
            round_trip.properties[&open_id],
            PropertyValue::Boolean(true)
        );

        let close_context = PropertyBindingContext {
            property: open_id.clone(),
            stamp: gallery.stamp_lease.clone(),
        };
        assert!(gallery.write_enabled(&close_context, false));
        assert!(!gallery.enabled);
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&open_id],
            PropertyValue::Boolean(false)
        );

        let before = gallery.host.snapshot().unwrap().clone();
        let mut invalid = before.preview.document.clone();
        invalid
            .properties
            .insert(trigger_id, PropertyValue::String(" \t".into()));
        gallery.descriptor_json = encode_preview(&invalid, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        assert!(
            gallery
                .status
                .contains("Popover trigger label must not be blank")
        );
        assert_eq!(gallery.host.snapshot().unwrap().stamp, before.stamp);
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.document,
            before.preview.document
        );
        assert!(!gallery.enabled);
    }

    #[test]
    fn missing_current_text_property_does_not_reuse_previous_component_text() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.text = "Previous component text".into();
        gallery.mount_icon_specimen();

        let binding = gallery.text_binding();

        assert_eq!(binding.read_from(&gallery), Some(String::new()));
    }

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

    #[test]
    fn animated_opacity_specimen_round_trips_bounded_endpoints_without_events() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_animated_opacity_specimen();
        assert!(
            gallery
                .status
                .starts_with("Mounted animated-opacity-specimen"),
            "{}",
            gallery.status
        );
        let component_id = ComponentId::new("animated-opacity-specimen").unwrap();
        let from_id = PropertyId::new("from").unwrap();
        let to_id = PropertyId::new("to").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 2);
        assert!(schema.events.is_empty());
        for id in [&from_id, &to_id] {
            assert_eq!(schema.properties[id].kind, PropertyKind::Number);
            assert_eq!(schema.properties[id].minimum, Some(0.0));
            assert_eq!(schema.properties[id].maximum, Some(1.0));
        }
        let mounted = gallery.host.snapshot().unwrap().clone();
        assert!(mounted.preview.document.actions.is_empty());
        assert_eq!(
            mounted.preview.properties[&from_id],
            PropertyValue::Number(0.0)
        );
        assert_eq!(
            mounted.preview.properties[&to_id],
            PropertyValue::Number(1.0)
        );
        let mut customized = mounted.preview.document.clone();
        customized
            .properties
            .insert(from_id.clone(), PropertyValue::Number(0.25));
        customized
            .properties
            .insert(to_id.clone(), PropertyValue::Number(0.75));
        gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap();
        assert_eq!(
            imported.preview.properties[&from_id],
            PropertyValue::Number(0.25)
        );
        assert_eq!(
            imported.preview.properties[&to_id],
            PropertyValue::Number(0.75)
        );
        let mut invalid = imported.preview.document.clone();
        invalid
            .properties
            .insert(from_id, PropertyValue::Number(-0.01));
        assert!(encode_preview(&invalid, gallery.host.catalog()).is_err());
        invalid.properties.insert(
            PropertyId::new("from").unwrap(),
            PropertyValue::Number(0.25),
        );
        invalid
            .properties
            .insert(to_id, PropertyValue::Number(1.01));
        assert!(encode_preview(&invalid, gallery.host.catalog()).is_err());
    }

    #[test]
    fn spring_opacity_specimen_round_trips_bounded_target_without_events() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_spring_opacity_specimen();
        assert!(
            gallery
                .status
                .starts_with("Mounted spring-opacity-specimen"),
            "{}",
            gallery.status
        );

        let component_id = ComponentId::new("spring-opacity-specimen").unwrap();
        let target_id = PropertyId::new("target").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 1);
        assert!(schema.events.is_empty());
        assert_eq!(schema.properties[&target_id].kind, PropertyKind::Number);
        assert_eq!(schema.properties[&target_id].minimum, Some(0.0));
        assert_eq!(schema.properties[&target_id].maximum, Some(1.0));

        let mounted = gallery.host.snapshot().unwrap().clone();
        assert!(mounted.preview.document.actions.is_empty());
        assert_eq!(
            mounted.preview.properties[&target_id],
            PropertyValue::Number(0.15)
        );
        let mut customized = mounted.preview.document.clone();
        customized
            .properties
            .insert(target_id.clone(), PropertyValue::Number(0.75));
        gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap();
        assert_eq!(
            imported.preview.properties[&target_id],
            PropertyValue::Number(0.75)
        );

        for target in [-0.01, 1.01] {
            let mut invalid = imported.preview.document.clone();
            invalid
                .properties
                .insert(target_id.clone(), PropertyValue::Number(target));
            assert!(encode_preview(&invalid, gallery.host.catalog()).is_err());
        }
    }

    #[test]
    fn shadow_specimen_round_trips_bounded_label_and_elevation_without_events() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_shadow_specimen();
        assert!(
            gallery.status.starts_with("Mounted shadow-specimen"),
            "{}",
            gallery.status
        );
        let component_id = ComponentId::new("shadow-specimen").unwrap();
        let label_id = PropertyId::new("label").unwrap();
        let elevation_id = PropertyId::new("elevation").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 2);
        assert!(schema.events.is_empty());
        assert_eq!(schema.properties[&label_id].kind, PropertyKind::String);
        assert_eq!(
            schema.properties[&label_id].max_length,
            Some(PREVIEW_TREE_LABEL_MAX_LENGTH)
        );
        assert_eq!(
            schema.properties[&elevation_id].allowed_values,
            ["small", "medium", "large"].map(str::to_owned)
        );
        let mounted = gallery.host.snapshot().unwrap().clone();
        assert!(mounted.preview.document.actions.is_empty());
        assert_eq!(
            mounted.preview.properties[&label_id],
            PropertyValue::String("Preview shadow".into())
        );
        assert_eq!(
            mounted.preview.properties[&elevation_id],
            PropertyValue::String("medium".into())
        );
        let mut customized = mounted.preview.document.clone();
        customized.properties.insert(
            label_id.clone(),
            PropertyValue::String("Raised panel".into()),
        );
        customized
            .properties
            .insert(elevation_id.clone(), PropertyValue::String("large".into()));
        gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap();
        assert_eq!(
            imported.preview.properties[&label_id],
            PropertyValue::String("Raised panel".into())
        );
        assert_eq!(
            imported.preview.properties[&elevation_id],
            PropertyValue::String("large".into())
        );
        let mut invalid = imported.preview.document.clone();
        invalid
            .properties
            .insert(elevation_id, PropertyValue::String("custom".into()));
        assert!(encode_preview(&invalid, gallery.host.catalog()).is_err());
    }

    #[test]
    fn background_specimen_round_trips_bounded_label_and_surface_without_events() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_background_specimen();
        assert!(
            gallery.status.starts_with("Mounted background-specimen"),
            "{}",
            gallery.status
        );
        let component_id = ComponentId::new("background-specimen").unwrap();
        let label_id = PropertyId::new("label").unwrap();
        let surface_id = PropertyId::new("surface").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 2);
        assert!(schema.events.is_empty());
        assert_eq!(schema.properties[&label_id].kind, PropertyKind::String);
        assert_eq!(
            schema.properties[&label_id].max_length,
            Some(PREVIEW_TREE_LABEL_MAX_LENGTH)
        );
        assert_eq!(
            schema.properties[&surface_id].allowed_values,
            PREVIEW_CONTAINER_SURFACES.map(str::to_owned)
        );
        let mounted = gallery.host.snapshot().unwrap().clone();
        assert!(mounted.preview.document.actions.is_empty());
        assert_eq!(
            mounted.preview.properties[&label_id],
            PropertyValue::String("Preview background".into())
        );
        assert_eq!(
            mounted.preview.properties[&surface_id],
            PropertyValue::String("panel".into())
        );
        let mut customized = mounted.preview.document.clone();
        customized.properties.insert(
            label_id.clone(),
            PropertyValue::String("Sidebar surface".into()),
        );
        customized
            .properties
            .insert(surface_id.clone(), PropertyValue::String("sidebar".into()));
        gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap();
        assert_eq!(
            imported.preview.properties[&label_id],
            PropertyValue::String("Sidebar surface".into())
        );
        assert_eq!(
            imported.preview.properties[&surface_id],
            PropertyValue::String("sidebar".into())
        );
        let mut invalid = imported.preview.document.clone();
        invalid
            .properties
            .insert(surface_id, PropertyValue::String("custom".into()));
        assert!(encode_preview(&invalid, gallery.host.catalog()).is_err());
    }

    #[test]
    fn border_specimen_round_trips_bounded_width_and_color_without_events() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_border_specimen();
        assert!(
            gallery.status.starts_with("Mounted border-specimen"),
            "{}",
            gallery.status
        );
        let component_id = ComponentId::new("border-specimen").unwrap();
        let width_id = PropertyId::new("width").unwrap();
        let color_id = PropertyId::new("color").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 2);
        assert!(schema.events.is_empty());
        assert_eq!(schema.properties[&width_id].kind, PropertyKind::Number);
        assert_eq!(schema.properties[&width_id].minimum, Some(0.0));
        assert_eq!(schema.properties[&width_id].maximum, Some(8.0));
        assert_eq!(
            schema.properties[&color_id].allowed_values,
            ["border", "accent", "focus-ring"].map(str::to_owned)
        );
        let mounted = gallery.host.snapshot().unwrap().clone();
        assert!(mounted.preview.document.actions.is_empty());
        assert_eq!(
            mounted.preview.properties[&width_id],
            PropertyValue::Number(1.0)
        );
        assert_eq!(
            mounted.preview.properties[&color_id],
            PropertyValue::String("border".into())
        );
        let mut customized = mounted.preview.document.clone();
        customized
            .properties
            .insert(width_id.clone(), PropertyValue::Number(2.5));
        customized
            .properties
            .insert(color_id.clone(), PropertyValue::String("accent".into()));
        gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap();
        assert_eq!(
            imported.preview.properties[&width_id],
            PropertyValue::Number(2.5)
        );
        assert_eq!(
            imported.preview.properties[&color_id],
            PropertyValue::String("accent".into())
        );
        let mut invalid = imported.preview.document.clone();
        invalid
            .properties
            .insert(width_id.clone(), PropertyValue::Number(-0.5));
        assert!(encode_preview(&invalid, gallery.host.catalog()).is_err());
        invalid
            .properties
            .insert(width_id, PropertyValue::Number(2.5));
        invalid
            .properties
            .insert(color_id, PropertyValue::String("custom".into()));
        assert!(encode_preview(&invalid, gallery.host.catalog()).is_err());
    }

    #[test]
    fn semantics_specimen_round_trips_allowlisted_static_role_without_events() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_semantics_specimen();
        assert!(
            gallery.status.starts_with("Mounted semantics-specimen"),
            "{}",
            gallery.status
        );
        let component_id = ComponentId::new("semantics-specimen").unwrap();
        let role_id = PropertyId::new("role").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 1);
        assert!(schema.events.is_empty());
        assert_eq!(schema.properties[&role_id].kind, PropertyKind::String);
        assert_eq!(
            schema.properties[&role_id].allowed_values,
            ["group", "region", "section", "navigation"].map(str::to_owned)
        );
        let mounted = gallery.host.snapshot().unwrap().clone();
        assert!(mounted.preview.document.actions.is_empty());
        assert_eq!(
            mounted.preview.properties[&role_id],
            PropertyValue::String("group".into())
        );
        let mut customized = mounted.preview.document.clone();
        customized
            .properties
            .insert(role_id.clone(), PropertyValue::String("navigation".into()));
        gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap();
        assert_eq!(
            imported.preview.properties[&role_id],
            PropertyValue::String("navigation".into())
        );
        let mut invalid = imported.preview.document.clone();
        invalid
            .properties
            .insert(role_id, PropertyValue::String("button".into()));
        assert!(encode_preview(&invalid, gallery.host.catalog()).is_err());
    }

    #[test]
    fn card_specimen_round_trips_bounded_label_and_border_without_events() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_card_specimen();
        assert!(
            gallery.status.starts_with("Mounted card-specimen"),
            "{}",
            gallery.status
        );
        let component_id = ComponentId::new("card-specimen").unwrap();
        let label_id = PropertyId::new("label").unwrap();
        let border_id = PropertyId::new("border").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 2);
        assert!(schema.events.is_empty());
        assert_eq!(schema.properties[&label_id].kind, PropertyKind::String);
        assert_eq!(schema.properties[&border_id].kind, PropertyKind::Boolean);

        let mounted = gallery.host.snapshot().unwrap().clone();
        assert!(mounted.preview.document.actions.is_empty());
        let mut customized = mounted.preview.document.clone();
        customized.properties.insert(
            label_id.clone(),
            PropertyValue::String("Settings card".into()),
        );
        customized
            .properties
            .insert(border_id.clone(), PropertyValue::Boolean(true));
        gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap();
        assert_eq!(
            imported.preview.properties[&label_id],
            PropertyValue::String("Settings card".into())
        );
        assert_eq!(
            imported.preview.properties[&border_id],
            PropertyValue::Boolean(true)
        );
    }

    #[test]
    fn container_specimen_round_trips_label_and_allowlisted_surface() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_container_specimen();
        assert!(
            gallery.status.starts_with("Mounted container-specimen"),
            "{}",
            gallery.status
        );
        let component_id = ComponentId::new("container-specimen").unwrap();
        let label_id = PropertyId::new("label").unwrap();
        let surface_id = PropertyId::new("surface").unwrap();
        let padding_id = PropertyId::new("padding").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert!(schema.events.is_empty());
        assert_eq!(schema.properties.len(), 3);
        assert_eq!(schema.properties[&padding_id].kind, PropertyKind::Number);
        assert_eq!(
            schema.properties[&padding_id].minimum,
            Some(PREVIEW_CONTAINER_PADDING_MIN)
        );
        assert_eq!(
            schema.properties[&padding_id].maximum,
            Some(PREVIEW_CONTAINER_PADDING_MAX)
        );
        assert_eq!(
            schema.properties[&padding_id].default,
            Some(PropertyValue::Number(PREVIEW_CONTAINER_PADDING_DEFAULT))
        );
        assert_eq!(
            schema.properties[&surface_id].allowed_values,
            PREVIEW_CONTAINER_SURFACES
                .iter()
                .map(|surface| (*surface).to_string())
                .collect::<Vec<String>>()
        );

        let mounted = gallery.host.snapshot().unwrap().clone();
        let mut customized = mounted.preview.document.clone();
        customized.properties.insert(
            label_id.clone(),
            PropertyValue::String("Navigation surface".into()),
        );
        customized
            .properties
            .insert(surface_id.clone(), PropertyValue::String("sidebar".into()));
        customized
            .properties
            .insert(padding_id.clone(), PropertyValue::Number(24.0));
        gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap();
        assert_eq!(
            imported.preview.properties[&label_id],
            PropertyValue::String("Navigation surface".into())
        );
        assert_eq!(
            imported.preview.properties[&surface_id],
            PropertyValue::String("sidebar".into())
        );
        assert_eq!(
            imported.preview.properties[&padding_id],
            PropertyValue::Number(24.0)
        );
    }

    #[test]
    fn split_view_specimen_round_trips_orientation_labels_and_first_size() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_split_view_specimen();
        assert!(
            gallery.status.starts_with("Mounted split-view-specimen"),
            "{}",
            gallery.status
        );
        let component_id = ComponentId::new("split-view-specimen").unwrap();
        let orientation_id = PropertyId::new("orientation").unwrap();
        let first_size_id = PropertyId::new("first_size").unwrap();
        let first_label_id = PropertyId::new("first_label").unwrap();
        let second_label_id = PropertyId::new("second_label").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 4);
        assert!(schema.events.is_empty());
        assert_eq!(
            schema.properties[&orientation_id].allowed_values,
            PREVIEW_SPLIT_ORIENTATIONS
                .iter()
                .map(|orientation| (*orientation).to_string())
                .collect::<Vec<String>>()
        );
        assert_eq!(schema.properties[&first_size_id].kind, PropertyKind::Number);
        assert_eq!(
            schema.properties[&first_size_id].default,
            Some(PropertyValue::Number(180.0))
        );
        assert_eq!(schema.properties[&first_size_id].minimum, Some(120.0));
        assert_eq!(schema.properties[&first_size_id].maximum, Some(360.0));

        let mounted = gallery.host.snapshot().unwrap().clone();
        assert!(mounted.preview.document.actions.is_empty());
        let mut customized = mounted.preview.document.clone();
        customized.properties.insert(
            orientation_id.clone(),
            PropertyValue::String("vertical".into()),
        );
        customized
            .properties
            .insert(first_size_id.clone(), PropertyValue::Number(250.0));
        customized.properties.insert(
            first_label_id.clone(),
            PropertyValue::String("Projects".into()),
        );
        customized.properties.insert(
            second_label_id.clone(),
            PropertyValue::String("Overview".into()),
        );
        gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap();
        assert_eq!(
            imported.preview.properties[&orientation_id],
            PropertyValue::String("vertical".into())
        );
        assert_eq!(
            imported.preview.properties[&first_label_id],
            PropertyValue::String("Projects".into())
        );
        assert_eq!(
            imported.preview.properties[&second_label_id],
            PropertyValue::String("Overview".into())
        );
        assert_eq!(
            imported.preview.properties[&first_size_id],
            PropertyValue::Number(250.0)
        );
        gallery.export_descriptor();
        let exported = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
        assert_eq!(
            exported.properties[&first_size_id],
            PropertyValue::Number(250.0)
        );
    }

    #[test]
    fn label_specimen_round_trips_text_secondary_and_vibrant_descriptor() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_label_specimen();
        assert!(
            gallery.status.starts_with("Mounted label-specimen"),
            "{}",
            gallery.status
        );

        let component_id = ComponentId::new("label-specimen").unwrap();
        let text_id = PropertyId::new("text").unwrap();
        let secondary_id = PropertyId::new("secondary").unwrap();
        let vibrant_id = PropertyId::new("vibrant").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 3);
        assert_eq!(schema.properties[&text_id].kind, PropertyKind::String);
        assert_eq!(schema.properties[&secondary_id].kind, PropertyKind::String);
        assert_eq!(schema.properties[&vibrant_id].kind, PropertyKind::Boolean);
        assert!(schema.properties.values().all(|property| property.editable));
        assert!(schema.events.is_empty());

        let mounted = gallery.host.snapshot().unwrap().clone();
        assert!(mounted.preview.document.actions.is_empty());
        let mut customized = mounted.preview.document.clone();
        customized
            .properties
            .insert(text_id.clone(), PropertyValue::String("Profile".into()));
        customized
            .properties
            .insert(secondary_id.clone(), PropertyValue::String("⌥P".into()));
        customized
            .properties
            .insert(vibrant_id.clone(), PropertyValue::Boolean(true));
        gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        assert!(gallery.status.contains("Imported descriptor"));
        let imported = gallery.host.snapshot().unwrap();
        assert_eq!(
            imported.preview.properties[&text_id],
            PropertyValue::String("Profile".into())
        );
        assert_eq!(
            imported.preview.properties[&secondary_id],
            PropertyValue::String("⌥P".into())
        );
        assert_eq!(
            imported.preview.properties[&vibrant_id],
            PropertyValue::Boolean(true)
        );

        gallery.export_descriptor();
        let exported = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
        assert_eq!(
            exported.properties[&text_id],
            PropertyValue::String("Profile".into())
        );
        assert_eq!(
            exported.properties[&secondary_id],
            PropertyValue::String("⌥P".into())
        );
        assert_eq!(
            exported.properties[&vibrant_id],
            PropertyValue::Boolean(true)
        );
    }

    #[test]
    fn divider_specimen_round_trips_vertical_orientation_descriptor() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_divider_specimen();
        assert!(
            gallery.status.starts_with("Mounted divider-specimen"),
            "{}",
            gallery.status
        );

        let component_id = ComponentId::new("divider-specimen").unwrap();
        let vertical_id = PropertyId::new("vertical").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 1);
        assert_eq!(schema.properties[&vertical_id].kind, PropertyKind::Boolean);
        assert_eq!(
            schema.properties[&vertical_id].default,
            Some(PropertyValue::Boolean(false))
        );
        assert!(schema.properties[&vertical_id].editable);
        assert!(schema.events.is_empty());

        let mounted = gallery.host.snapshot().unwrap().clone();
        assert!(mounted.preview.document.actions.is_empty());
        assert_eq!(
            mounted.preview.properties[&vertical_id],
            PropertyValue::Boolean(false)
        );
        let mut customized = mounted.preview.document.clone();
        customized
            .properties
            .insert(vertical_id.clone(), PropertyValue::Boolean(true));
        gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        assert!(gallery.status.contains("Imported descriptor"));
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&vertical_id],
            PropertyValue::Boolean(true)
        );

        gallery.export_descriptor();
        let exported = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
        assert_eq!(
            exported.properties[&vertical_id],
            PropertyValue::Boolean(true)
        );
    }

    #[test]
    fn skeleton_specimen_round_trips_secondary_descriptor() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_skeleton_specimen();
        assert!(
            gallery.status.starts_with("Mounted skeleton-specimen"),
            "{}",
            gallery.status
        );

        let component_id = ComponentId::new("skeleton-specimen").unwrap();
        let secondary_id = PropertyId::new("secondary").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 1);
        assert_eq!(schema.properties[&secondary_id].kind, PropertyKind::Boolean);
        assert_eq!(
            schema.properties[&secondary_id].default,
            Some(PropertyValue::Boolean(false))
        );
        assert!(schema.properties[&secondary_id].editable);
        assert!(schema.events.is_empty());

        let mounted = gallery.host.snapshot().unwrap().clone();
        assert!(mounted.preview.document.actions.is_empty());
        assert_eq!(
            mounted.preview.properties[&secondary_id],
            PropertyValue::Boolean(false)
        );
        let mut customized = mounted.preview.document.clone();
        customized
            .properties
            .insert(secondary_id.clone(), PropertyValue::Boolean(true));
        gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        assert!(gallery.status.contains("Imported descriptor"));
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&secondary_id],
            PropertyValue::Boolean(true)
        );

        gallery.export_descriptor();
        let exported = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
        assert_eq!(
            exported.properties[&secondary_id],
            PropertyValue::Boolean(true)
        );
    }

    #[test]
    fn scroll_view_specimen_round_trips_bounded_axis_label_and_scrollbars() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_scroll_view_specimen();
        assert!(
            gallery.status.starts_with("Mounted scroll-view-specimen"),
            "{}",
            gallery.status
        );

        let component_id = ComponentId::new("scroll-view-specimen").unwrap();
        let label_id = PropertyId::new("label").unwrap();
        let axis_id = PropertyId::new("axis").unwrap();
        let scrollbars_id = PropertyId::new("scrollbars").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 3);
        assert!(schema.events.is_empty());
        assert_eq!(
            schema.properties[&axis_id].allowed_values,
            PREVIEW_SCROLL_AXES
        );
        assert_eq!(
            schema.properties[&scrollbars_id].kind,
            PropertyKind::Boolean
        );

        let mounted = gallery.host.snapshot().unwrap().clone();
        assert!(mounted.preview.document.actions.is_empty());
        let mut customized = mounted.preview.document.clone();
        customized.properties.insert(
            label_id.clone(),
            PropertyValue::String("Scrollable files".into()),
        );
        customized
            .properties
            .insert(axis_id.clone(), PropertyValue::String("both".into()));
        customized
            .properties
            .insert(scrollbars_id.clone(), PropertyValue::Boolean(false));
        gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap();
        assert_eq!(
            imported.preview.properties[&label_id],
            PropertyValue::String("Scrollable files".into())
        );
        assert_eq!(
            imported.preview.properties[&axis_id],
            PropertyValue::String("both".into())
        );
        assert_eq!(
            imported.preview.properties[&scrollbars_id],
            PropertyValue::Boolean(false)
        );
    }

    #[test]
    fn property_inspector_specimen_round_trips_bounded_labels_and_disabled_state() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_property_inspector_specimen();
        assert!(
            gallery
                .status
                .starts_with("Mounted property-inspector-specimen"),
            "{}",
            gallery.status
        );

        let component_id = ComponentId::new("property-inspector-specimen").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        let text_label_id = PropertyId::new("text_label").unwrap();
        let toggle_label_id = PropertyId::new("toggle_label").unwrap();
        let integer_label_id = PropertyId::new("integer_label").unwrap();
        let disabled_id = PropertyId::new("disabled").unwrap();
        assert_eq!(schema.properties.len(), 4);
        assert!(schema.events.is_empty());
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
        assert_eq!(
            schema.properties[&text_label_id].max_length,
            Some(PREVIEW_TREE_LABEL_MAX_LENGTH)
        );

        let mounted = gallery.host.snapshot().unwrap().clone();
        let mut customized = mounted.preview.document.clone();
        customized.properties.insert(
            text_label_id.clone(),
            PropertyValue::String("Display name".into()),
        );
        customized.properties.insert(
            toggle_label_id.clone(),
            PropertyValue::String("Visible".into()),
        );
        customized.properties.insert(
            integer_label_id.clone(),
            PropertyValue::String("Items".into()),
        );
        customized
            .properties
            .insert(disabled_id.clone(), PropertyValue::Boolean(true));
        gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap();
        assert_eq!(
            imported.preview.properties[&text_label_id],
            PropertyValue::String("Display name".into())
        );
        assert_eq!(
            imported.preview.properties[&toggle_label_id],
            PropertyValue::String("Visible".into())
        );
        assert_eq!(
            imported.preview.properties[&integer_label_id],
            PropertyValue::String("Items".into())
        );
        assert_eq!(
            imported.preview.properties[&disabled_id],
            PropertyValue::Boolean(true)
        );
    }

    #[test]
    fn number_inspector_drafts_follow_property_values_and_imports() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_avatar_specimen();
        let diameter_id = PropertyId::new("diameter").unwrap();
        let avatar_snapshot = gallery.host.snapshot().unwrap().clone();
        let diameter_context = PropertyBindingContext {
            property: diameter_id.clone(),
            stamp: gallery.stamp_lease.clone(),
        };
        let PropertyValue::Number(default_diameter) =
            avatar_snapshot.preview.properties[&diameter_id]
        else {
            panic!("Avatar diameter should be a number");
        };
        assert_eq!(
            gallery.read_number_draft(&diameter_context),
            default_diameter.to_string()
        );
        assert!(gallery.write_number_draft(&diameter_context, "56".into()));
        assert_eq!(gallery.amount, 0.375);
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&diameter_id],
            PropertyValue::Number(56.0)
        );

        gallery.mount_hstack_specimen();
        assert_eq!(gallery.read_number_draft(&diameter_context), "");
        assert!(!gallery.write_number_draft(&diameter_context, "60".into()));

        let spacing_id = PropertyId::new("spacing").unwrap();
        let spacing_context = PropertyBindingContext {
            property: spacing_id.clone(),
            stamp: gallery.stamp_lease.clone(),
        };
        let hstack_snapshot = gallery.host.snapshot().unwrap().clone();
        let PropertyValue::Number(default_spacing) =
            hstack_snapshot.preview.properties[&spacing_id]
        else {
            panic!("HStack spacing should be a number");
        };
        assert_eq!(
            gallery.read_number_draft(&spacing_context),
            default_spacing.to_string()
        );
        assert!(gallery.write_number_draft(&spacing_context, "-".into()));
        assert_eq!(gallery.read_number_draft(&spacing_context), "-");
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&spacing_id],
            PropertyValue::Number(default_spacing)
        );
        assert!(gallery.write_number_draft(&spacing_context, "24.5".into()));
        assert_eq!(gallery.amount, 0.375);
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&spacing_id],
            PropertyValue::Number(24.5)
        );

        let mut imported = gallery.host.snapshot().unwrap().preview.document.clone();
        imported
            .properties
            .insert(spacing_id.clone(), PropertyValue::Number(30.25));
        gallery.descriptor_json = encode_preview(&imported, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        let imported_context = PropertyBindingContext {
            property: spacing_id.clone(),
            stamp: gallery.stamp_lease.clone(),
        };
        assert_eq!(gallery.read_number_draft(&imported_context), "30.25");
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties[&spacing_id],
            PropertyValue::Number(30.25)
        );
    }

    #[test]
    fn frame_specimen_round_trips_bounded_width_without_events() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_frame_specimen();
        assert!(
            gallery.status.starts_with("Mounted frame-specimen"),
            "{}",
            gallery.status
        );
        let component_id = ComponentId::new("frame-specimen").unwrap();
        let width_id = PropertyId::new("width").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 1);
        assert!(schema.events.is_empty());
        let width = &schema.properties[&width_id];
        assert_eq!(width.kind, PropertyKind::Number);
        assert_eq!(width.minimum, Some(PREVIEW_FRAME_WIDTH_MIN));
        assert_eq!(width.maximum, Some(PREVIEW_FRAME_WIDTH_MAX));
        let mounted = gallery.host.snapshot().unwrap().clone();
        assert!(mounted.preview.document.actions.is_empty());
        assert_eq!(
            mounted.preview.properties[&width_id],
            PropertyValue::Number(PREVIEW_FRAME_WIDTH_DEFAULT)
        );

        let mut customized = mounted.preview.document.clone();
        customized
            .properties
            .insert(width_id.clone(), PropertyValue::Number(192.5));
        gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        gallery.export_descriptor();
        let exported = decode_preview(&gallery.descriptor_json, gallery.host.catalog()).unwrap();
        assert_eq!(exported.properties[&width_id], PropertyValue::Number(192.5));
        let accepted = gallery.host.snapshot().unwrap().clone();
        let valid_descriptor = gallery.descriptor_json.clone();
        for width in [PREVIEW_FRAME_WIDTH_MIN - 1.0, PREVIEW_FRAME_WIDTH_MAX + 1.0] {
            let mut invalid = accepted.preview.document.clone();
            invalid
                .properties
                .insert(width_id.clone(), PropertyValue::Number(width));
            assert!(encode_preview(&invalid, gallery.host.catalog()).is_err());
            gallery.descriptor_json = valid_descriptor.replacen("192.5", &width.to_string(), 1);
            gallery.import_descriptor();
            let current = gallery.host.snapshot().unwrap();
            assert_eq!(current.stamp, accepted.stamp);
            assert_eq!(current.preview.document, accepted.preview.document);
        }
    }

    #[test]
    fn hstack_specimen_round_trips_bounded_spacing_without_events() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_hstack_specimen();
        assert!(
            gallery.status.starts_with("Mounted hstack-specimen"),
            "{}",
            gallery.status
        );

        let component_id = ComponentId::new("hstack-specimen").unwrap();
        let spacing_id = PropertyId::new("spacing").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 1);
        assert!(schema.events.is_empty());
        assert_eq!(schema.properties[&spacing_id].kind, PropertyKind::Number);
        assert_eq!(
            schema.properties[&spacing_id].minimum,
            Some(PREVIEW_HSTACK_SPACING_MIN)
        );
        assert_eq!(
            schema.properties[&spacing_id].maximum,
            Some(PREVIEW_HSTACK_SPACING_MAX)
        );

        let mounted = gallery.host.snapshot().unwrap().clone();
        assert!(mounted.preview.document.actions.is_empty());
        assert_eq!(
            mounted.preview.properties[&spacing_id],
            PropertyValue::Number(PREVIEW_HSTACK_SPACING_DEFAULT)
        );
        let mut customized = mounted.preview.document.clone();
        customized
            .properties
            .insert(spacing_id.clone(), PropertyValue::Number(24.5));
        gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap();
        assert_eq!(
            imported.preview.properties[&spacing_id],
            PropertyValue::Number(24.5)
        );
    }

    #[test]
    fn vstack_specimen_round_trips_bounded_spacing_without_events() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_vstack_specimen();
        assert!(
            gallery.status.starts_with("Mounted vstack-specimen"),
            "{}",
            gallery.status
        );

        let component_id = ComponentId::new("vstack-specimen").unwrap();
        let spacing_id = PropertyId::new("spacing").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 1);
        assert!(schema.events.is_empty());
        assert_eq!(schema.properties[&spacing_id].kind, PropertyKind::Number);
        assert_eq!(
            schema.properties[&spacing_id].minimum,
            Some(PREVIEW_VSTACK_SPACING_MIN)
        );
        assert_eq!(
            schema.properties[&spacing_id].maximum,
            Some(PREVIEW_VSTACK_SPACING_MAX)
        );

        let mounted = gallery.host.snapshot().unwrap().clone();
        assert!(mounted.preview.document.actions.is_empty());
        assert_eq!(
            mounted.preview.properties[&spacing_id],
            PropertyValue::Number(PREVIEW_VSTACK_SPACING_DEFAULT)
        );

        let mut customized = mounted.preview.document.clone();
        customized
            .properties
            .insert(spacing_id.clone(), PropertyValue::Number(24.5));
        gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap();
        assert_eq!(
            imported.preview.properties[&spacing_id],
            PropertyValue::Number(24.5)
        );
    }

    #[test]
    fn grid_specimen_round_trips_bounded_columns_and_spacing_without_events() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_grid_specimen();
        assert!(
            gallery.status.starts_with("Mounted grid-specimen"),
            "{}",
            gallery.status
        );

        let component_id = ComponentId::new("grid-specimen").unwrap();
        let columns_id = PropertyId::new("columns").unwrap();
        let spacing_id = PropertyId::new("spacing").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 2);
        assert!(schema.events.is_empty());
        assert_eq!(schema.properties[&columns_id].kind, PropertyKind::Integer);
        assert_eq!(
            schema.properties[&columns_id].minimum,
            Some(PREVIEW_GRID_COLUMNS_MIN)
        );
        assert_eq!(
            schema.properties[&columns_id].maximum,
            Some(PREVIEW_GRID_COLUMNS_MAX)
        );
        assert_eq!(schema.properties[&spacing_id].kind, PropertyKind::Number);
        assert_eq!(
            schema.properties[&spacing_id].minimum,
            Some(PREVIEW_GRID_SPACING_MIN)
        );
        assert_eq!(
            schema.properties[&spacing_id].maximum,
            Some(PREVIEW_GRID_SPACING_MAX)
        );

        let mounted = gallery.host.snapshot().unwrap().clone();
        assert!(mounted.preview.document.actions.is_empty());
        assert_eq!(
            mounted.preview.properties[&columns_id],
            PropertyValue::Integer(PREVIEW_GRID_COLUMNS_DEFAULT)
        );
        assert_eq!(
            mounted.preview.properties[&spacing_id],
            PropertyValue::Number(PREVIEW_GRID_SPACING_DEFAULT)
        );
        let mut customized = mounted.preview.document.clone();
        customized
            .properties
            .insert(columns_id.clone(), PropertyValue::Integer(3));
        customized
            .properties
            .insert(spacing_id.clone(), PropertyValue::Number(24.5));
        gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap();
        assert_eq!(
            imported.preview.properties[&columns_id],
            PropertyValue::Integer(3)
        );
        assert_eq!(
            imported.preview.properties[&spacing_id],
            PropertyValue::Number(24.5)
        );
    }

    #[test]
    fn lazy_grid_specimen_round_trips_bounded_columns_and_gaps_without_events() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_lazy_grid_specimen();
        assert!(
            gallery.status.starts_with("Mounted lazy-grid-specimen"),
            "{}",
            gallery.status
        );

        let component_id = ComponentId::new("lazy-grid-specimen").unwrap();
        let columns_id = PropertyId::new("columns").unwrap();
        let spacing_id = PropertyId::new("spacing").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 2);
        assert!(schema.events.is_empty());
        assert_eq!(schema.properties[&columns_id].kind, PropertyKind::Integer);
        assert_eq!(
            schema.properties[&columns_id].minimum,
            Some(PREVIEW_LAZY_GRID_COLUMNS_MIN)
        );
        assert_eq!(
            schema.properties[&columns_id].maximum,
            Some(PREVIEW_LAZY_GRID_COLUMNS_MAX)
        );
        assert_eq!(schema.properties[&spacing_id].kind, PropertyKind::Number);
        assert_eq!(
            schema.properties[&spacing_id].minimum,
            Some(PREVIEW_LAZY_GRID_SPACING_MIN)
        );
        assert_eq!(
            schema.properties[&spacing_id].maximum,
            Some(PREVIEW_LAZY_GRID_SPACING_MAX)
        );

        let mounted = gallery.host.snapshot().unwrap().clone();
        assert!(mounted.preview.document.actions.is_empty());
        assert_eq!(
            mounted.preview.properties[&columns_id],
            PropertyValue::Integer(PREVIEW_LAZY_GRID_COLUMNS_DEFAULT)
        );
        assert_eq!(
            mounted.preview.properties[&spacing_id],
            PropertyValue::Number(PREVIEW_LAZY_GRID_SPACING_DEFAULT)
        );
        let mut customized = mounted.preview.document.clone();
        customized
            .properties
            .insert(columns_id.clone(), PropertyValue::Integer(3));
        customized
            .properties
            .insert(spacing_id.clone(), PropertyValue::Number(24.5));
        gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap();
        assert_eq!(
            imported.preview.properties[&columns_id],
            PropertyValue::Integer(3)
        );
        assert_eq!(
            imported.preview.properties[&spacing_id],
            PropertyValue::Number(24.5)
        );
    }

    #[test]
    fn spacer_specimen_round_trips_bounded_axis_and_minimum_without_events() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_spacer_specimen();
        assert!(
            gallery.status.starts_with("Mounted spacer-specimen"),
            "{}",
            gallery.status
        );

        let component_id = ComponentId::new("spacer-specimen").unwrap();
        let axis_id = PropertyId::new("axis").unwrap();
        let min_length_id = PropertyId::new("min_length").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 2);
        assert!(schema.events.is_empty());
        assert_eq!(schema.properties[&axis_id].kind, PropertyKind::String);
        assert_eq!(
            schema.properties[&axis_id].allowed_values,
            PREVIEW_SPACER_AXES
                .iter()
                .map(|axis| (*axis).into())
                .collect::<Vec<String>>()
        );
        assert_eq!(schema.properties[&min_length_id].kind, PropertyKind::Number);
        assert_eq!(
            schema.properties[&min_length_id].minimum,
            Some(PREVIEW_SPACER_MIN_LENGTH_MIN)
        );
        assert_eq!(
            schema.properties[&min_length_id].maximum,
            Some(PREVIEW_SPACER_MIN_LENGTH_MAX)
        );

        let mounted = gallery.host.snapshot().unwrap().clone();
        assert!(mounted.preview.document.actions.is_empty());
        assert_eq!(
            mounted.preview.properties[&axis_id],
            PropertyValue::String("horizontal".into())
        );
        assert_eq!(
            mounted.preview.properties[&min_length_id],
            PropertyValue::Number(PREVIEW_SPACER_MIN_LENGTH_DEFAULT)
        );
        let mut customized = mounted.preview.document.clone();
        customized
            .properties
            .insert(axis_id.clone(), PropertyValue::String("vertical".into()));
        customized
            .properties
            .insert(min_length_id.clone(), PropertyValue::Number(32.5));
        gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap();
        assert_eq!(
            imported.preview.properties[&axis_id],
            PropertyValue::String("vertical".into())
        );
        assert_eq!(
            imported.preview.properties[&min_length_id],
            PropertyValue::Number(32.5)
        );
    }

    #[test]
    fn zstack_specimen_round_trips_constrained_alignment_without_events() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount("zstack-specimen");
        assert!(
            gallery.status.starts_with("Mounted zstack-specimen"),
            "{}",
            gallery.status
        );

        let component_id = ComponentId::new("zstack-specimen").unwrap();
        let alignment_id = PropertyId::new("alignment").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        assert_eq!(schema.properties.len(), 1);
        assert!(schema.events.is_empty());
        assert_eq!(schema.properties[&alignment_id].kind, PropertyKind::String);
        assert_eq!(
            schema.properties[&alignment_id].allowed_values,
            PREVIEW_ZSTACK_ALIGNMENTS
                .iter()
                .map(|alignment| String::from(*alignment))
                .collect::<Vec<_>>()
        );

        let mounted = gallery.host.snapshot().unwrap().clone();
        assert!(mounted.preview.document.actions.is_empty());
        assert_eq!(
            mounted.preview.properties[&alignment_id],
            PropertyValue::String("center".into())
        );
        let mut customized = mounted.preview.document.clone();
        customized.properties.insert(
            alignment_id.clone(),
            PropertyValue::String("top-leading".into()),
        );
        gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();
        let imported = gallery.host.snapshot().unwrap();
        assert_eq!(
            imported.preview.properties[&alignment_id],
            PropertyValue::String("top-leading".into())
        );

        customized
            .properties
            .insert(alignment_id, PropertyValue::String("arbitrary".into()));
        assert!(encode_preview(&customized, gallery.host.catalog()).is_err());
    }

    #[test]
    fn geometry_reader_specimen_round_trips_bounded_size() {
        let mut gallery = CompiledPreviewGallery::default();
        gallery.mount_geometry_reader_specimen();
        assert!(
            gallery
                .status
                .starts_with("Mounted geometry-reader-specimen"),
            "{}",
            gallery.status
        );

        let component_id = ComponentId::new("geometry-reader-specimen").unwrap();
        let schema = &gallery.host.catalog().components[&component_id];
        let width_id = PropertyId::new("width").unwrap();
        let height_id = PropertyId::new("height").unwrap();
        assert_eq!(schema.properties.len(), 2);
        assert!(schema.events.is_empty());
        assert_eq!(schema.properties[&width_id].minimum, Some(80.0));
        assert_eq!(schema.properties[&width_id].maximum, Some(320.0));
        assert_eq!(schema.properties[&height_id].minimum, Some(64.0));
        assert_eq!(schema.properties[&height_id].maximum, Some(240.0));

        let mounted = gallery.host.snapshot().unwrap().clone();
        assert_eq!(
            mounted.preview.properties[&width_id],
            PropertyValue::Number(280.0)
        );
        assert_eq!(
            mounted.preview.properties[&height_id],
            PropertyValue::Number(120.0)
        );
        let mut customized = mounted.preview.document.clone();
        customized
            .properties
            .insert(width_id.clone(), PropertyValue::Number(312.5));
        customized
            .properties
            .insert(height_id.clone(), PropertyValue::Number(192.0));
        gallery.descriptor_json = encode_preview(&customized, gallery.host.catalog()).unwrap();
        gallery.import_descriptor();

        let imported = gallery.host.snapshot().unwrap();
        assert_eq!(
            imported.preview.properties[&width_id],
            PropertyValue::Number(312.5)
        );
        assert_eq!(
            imported.preview.properties[&height_id],
            PropertyValue::Number(192.0)
        );
    }
}
