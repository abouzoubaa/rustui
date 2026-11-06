//! D1c schema-driven local preview inspector; all controls use public RustUI APIs.
use rustui::{localization::MessageArguments, prelude::*, preview::*, view::EmptyView};
use std::{cell::RefCell, collections::BTreeMap, rc::Rc, time::Duration};

mod gallery;
mod registry;

fn preview_action(label: &'static str, action: rustui::runtime::Action) -> impl View {
    let component = label.strip_prefix("Preview ").unwrap_or(label);
    let (component, message) = match component.strip_suffix(" specimen") {
        Some(component) => (component, "lab-compiled-preview-specimen"),
        None => (component, "lab-compiled-preview-show"),
    };
    Button(label, action).localized_label_with_arguments(
        message,
        MessageArguments::new().with("component", component),
    )
}

pub fn demo() -> impl View {
    Component("compiled-preview-gallery", CompiledPreviewGallery::default)
}

#[cfg(target_family = "wasm")]
pub(crate) fn reference_demo() -> impl View {
    Component("browser-compiled-preview-reference", || {
        let mut owner = CompiledPreviewGallery::default();
        owner.teardown();
        owner.reference_session = Some(crate::preview_transport::Session::default());
        owner.reference_bridge = Some(crate::preview_transport_web::Bridge::new(
            CompiledPreviewGallery::reference_request,
            CompiledPreviewGallery::reference_close,
        ));
        owner
    })
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

fn preview_status_summary_key(status: &str) -> &'static str {
    match status {
        "Exported normalized preview descriptor" => "lab-preview-status-exported",
        "Completed delayed parent rerender" => "lab-preview-status-rerendered",
        _ => "lab-compiled-preview-status-summary",
    }
}

struct PreviewStatusPresentation {
    status: String,
    key: &'static str,
    subject: String,
    generation: String,
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
    reference_session: Option<crate::preview_transport::Session>,
    #[cfg(target_family = "wasm")]
    reference_bridge: Option<crate::preview_transport_web::Bridge<Self>>,
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
    status_presentation: Option<PreviewStatusPresentation>,
    descriptor_json: String,
}
