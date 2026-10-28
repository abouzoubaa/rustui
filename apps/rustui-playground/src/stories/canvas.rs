//! Isolated rendering for the stories currently supported by RustUI Playground.

use rustui::localization::MessageArguments;
use rustui::{
    component,
    prelude::{
        Binding, Breadcrumb, BreadcrumbItem, Button, Card, Checkbox, ColorPicker, ComboBox,
        Component, DatePicker, Environment, FontFallbackPolicy, HStack, Hsla, Keyed, List,
        LocalizedText, NaiveDate, NavigationDestination, NavigationLink, NavigationStack, Picker,
        Popover, Radio, SearchField, SecureField, Slider, Stepper, TabItem, TabView, TabViewStyle,
        Text, TextEditor, TextField, Toggle, TokenField, Tree, TreeNode, VStack, View,
    },
    runtime::Action,
};

use super::child_component::{ChildComponent, ChildProps, DemoDisplayMode};
use super::{component_catalog::StoryId, properties_panel::ButtonProperties};

pub struct RetentionStoryFields {
    pub tab_overview: Binding<String>,
    pub tab_details: Binding<String>,
    pub tab_conditional: Binding<String>,
    pub navigation_root: Binding<String>,
    pub navigation_details: Binding<String>,
    pub navigation_inspector: Binding<String>,
    pub navigation_conditional: Binding<String>,
    pub details_tab_present: bool,
    pub tab_conditional_visible: bool,
    pub navigation_conditional_visible: bool,
}

pub struct KeyedCollectionFields {
    pub alpha: Binding<String>,
    pub beta: Binding<String>,
    pub reversed: bool,
    pub beta_present: bool,
    pub child_title_prefix: Option<String>,
    pub display_mode: DemoDisplayMode,
    pub workload_active: bool,
    pub workload_epoch: u64,
    pub alpha_workload_increment: u8,
    pub beta_workload_increment: u8,
}

pub struct StoryCanvas {
    story: StoryId,
    count: i32,
    button: ButtonProperties,
    input_text: String,
    input_binding: Binding<String>,
    secure_binding: Binding<String>,
    toggle_binding: Binding<bool>,
    checkbox_binding: Binding<bool>,
    radio_binding: Binding<bool>,
    slider_binding: Binding<f32>,
    stepper_binding: Binding<i32>,
    picker_binding: Binding<String>,
    combo_box_binding: Binding<String>,
    date_binding: Binding<Option<NaiveDate>>,
    color_binding: Binding<Option<Hsla>>,
    _list_binding: Binding<Option<usize>>,
    _table_binding: Binding<Option<usize>>,
    tree_binding: Binding<Option<String>>,
    tab_binding: Binding<String>,
    breadcrumb_page: String,
    breadcrumb_overview: Action,
    breadcrumb_components: Action,
    navigation_path: Binding<Vec<String>>,
    retention: RetentionStoryFields,
    keyed_collection: KeyedCollectionFields,
    set_picker_kotlin: Action,
    increment: Action,
    reset: Action,
}

impl StoryCanvas {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        story: StoryId,
        count: i32,
        button: ButtonProperties,
        input_text: String,
        input_binding: Binding<String>,
        secure_binding: Binding<String>,
        toggle_binding: Binding<bool>,
        checkbox_binding: Binding<bool>,
        radio_binding: Binding<bool>,
        slider_binding: Binding<f32>,
        stepper_binding: Binding<i32>,
        picker_binding: Binding<String>,
        combo_box_binding: Binding<String>,
        date_binding: Binding<Option<NaiveDate>>,
        color_binding: Binding<Option<Hsla>>,
        list_binding: Binding<Option<usize>>,
        table_binding: Binding<Option<usize>>,
        tree_binding: Binding<Option<String>>,
        tab_binding: Binding<String>,
        breadcrumb_page: String,
        breadcrumb_overview: Action,
        breadcrumb_components: Action,
        navigation_path: Binding<Vec<String>>,
        retention: RetentionStoryFields,
        keyed_collection: KeyedCollectionFields,
        set_picker_kotlin: Action,
        increment: Action,
        reset: Action,
    ) -> Self {
        Self {
            story,
            count,
            button,
            input_text,
            input_binding,
            secure_binding,
            toggle_binding,
            checkbox_binding,
            radio_binding,
            slider_binding,
            stepper_binding,
            picker_binding,
            combo_box_binding,
            date_binding,
            color_binding,
            _list_binding: list_binding,
            _table_binding: table_binding,
            tree_binding,
            tab_binding,
            breadcrumb_page,
            breadcrumb_overview,
            breadcrumb_components,
            navigation_path,
            retention,
            keyed_collection,
            set_picker_kotlin,
            increment,
            reset,
        }
    }
}

impl View for StoryCanvas {
    fn body(&self) -> impl View {
        match self.story {
            StoryId::Button => VStack::new()
                .child(
                    LocalizedText("lab-button-count")
                        .with_arguments(MessageArguments::new().with("count", self.count)),
                )
                .child(
                    HStack::new()
                        .child(
                            Button("Increment", self.increment.clone())
                                .id("story-button-increment")
                                .localized_label("lab-button-increment-visible")
                                .localized_accessibility_label("lab-button-increment-accessibility")
                                .button_style(self.button.style)
                                .disabled(self.button.disabled),
                        )
                        .child(
                            Button("Reset", self.reset.clone())
                                .id("story-button-reset")
                                .localized_label("lab-button-reset-visible")
                                .button_style(self.button.style)
                                .disabled(self.button.disabled),
                        )
                        .spacing(8.0)
                        .alignment(rustui::prelude::VerticalAlignment::Center),
                )
                .child(
                    Button("Unavailable", self.reset.clone())
                        .id("story-button-disabled")
                        .localized_label("lab-button-unavailable-visible")
                        .button_style(self.button.style)
                        .disabled(true),
                )
                .spacing(8.0)
                .alignment(rustui::prelude::HorizontalAlignment::Start),
            StoryId::Text => {
                let mixed_script_fallbacks =
                    FontFallbackPolicy::new(["Geeza Pro", "PingFang SC", "Apple Symbols"])
                        .expect("the example fallback family list is nonempty");
                let localized_fallbacks = mixed_script_fallbacks.clone();

                VStack::new()
                    .child(Text("RustUI Text"))
                    .child(Text(format!("Dynamic value: {}", self.count)))
                    .child(Text("Rendered by GPUI through the RustUI View contract"))
                    .child(Text("Caller supplied macOS family example: Geeza Pro → PingFang SC → Apple Symbols"))
                    .child(
                        Text("Latin: Hello · Arabic: مرحبًا · Chinese: 你好")
                            .font_fallbacks(mixed_script_fallbacks),
                    )
                    .child(
                        LocalizedText("rustui-action-close")
                            .font_fallbacks(localized_fallbacks),
                    )
                    .child(Text("Repeated text label"))
                    .child(Environment(
                        "label-scope",
                        Text("Repeated text label"),
                    ))
                    .child(Text(
                        "AX check: expect two nodes named Repeated text label; verify LocalizedText updates after a locale change.",
                    ))
                    .child(Text("Family names are caller examples; availability is not checked."))
                    .child(super::semantics_gallery::demo())
                    .spacing(8.0)
                    .alignment(rustui::prelude::HorizontalAlignment::Start)
            }
            StoryId::Layout => {
                let alpha = Keyed(
                    "alpha",
                    TextEditor("Alpha row", self.keyed_collection.alpha.clone())
                        .id("row-editor")
                        .height(80.0)
                        .localized_placeholder("lab-layout-alpha-row"),
                );
                let beta = Keyed(
                    "beta",
                    TextEditor("Beta row", self.keyed_collection.beta.clone())
                        .id("row-editor")
                        .height(80.0)
                        .localized_placeholder("lab-layout-beta-row"),
                );
                let beta = self.keyed_collection.beta_present.then_some(beta);
                let rows = if self.keyed_collection.reversed {
                    vec![beta, Some(alpha)]
                } else {
                    vec![Some(alpha), beta]
                };
                let child_alpha_label = String::from("Alpha");
                let child_beta_label = String::from("Beta");
                let alpha_title = self
                    .keyed_collection
                    .child_title_prefix
                    .as_deref()
                    .map_or_else(
                        || child_alpha_label.clone(),
                        |prefix| format!("{prefix}{child_alpha_label}"),
                    );
                let beta_title = self
                    .keyed_collection
                    .child_title_prefix
                    .as_deref()
                    .map_or_else(
                        || child_beta_label.clone(),
                        |prefix| format!("{prefix}{child_beta_label}"),
                    );
                let child_alpha = Component(
                    format!("child-alpha-{}", self.keyed_collection.workload_epoch),
                    move || ChildComponent::named(child_alpha_label.clone()),
                )
                .props(
                    ChildProps {
                        title: alpha_title,
                        workload_increment: self.keyed_collection.alpha_workload_increment,
                    },
                    ChildComponent::apply_props,
                );
                let child_beta = Component(
                    format!("child-beta-{}", self.keyed_collection.workload_epoch),
                    move || ChildComponent::named(child_beta_label.clone()),
                )
                .props(
                    ChildProps {
                        title: beta_title,
                        workload_increment: self.keyed_collection.beta_workload_increment,
                    },
                    ChildComponent::apply_props,
                );
                let children = if self.keyed_collection.reversed {
                    vec![
                        self.keyed_collection.beta_present.then_some(child_beta),
                        Some(child_alpha),
                    ]
                } else {
                    vec![
                        Some(child_alpha),
                        self.keyed_collection.beta_present.then_some(child_beta),
                    ]
                };

                if self.keyed_collection.workload_active {
                    VStack::new()
                        .child(Environment(
                            self.keyed_collection.display_mode,
                            HStack::new().children(children).spacing(24.0),
                        ))
                        .spacing(8.0)
                } else {
                    let child_stack = VStack::new().children(children);

                    VStack::new()
                        .child(LocalizedText("lab-layout-outer-stack"))
                        .child(
                            HStack::new()
                                .child(LocalizedText("lab-layout-row-item"))
                                .child(
                                    Button("Layout action", self.increment.clone())
                                        .id("story-layout-action")
                                        .button_style(self.button.style)
                                        .localized_label("lab-layout-action"),
                                )
                                .spacing(8.0)
                                .alignment(rustui::prelude::VerticalAlignment::Center),
                        )
                        .child(LocalizedText("lab-layout-keyed-retention-note"))
                        .children(rows)
                        .child(LocalizedText("lab-layout-child-retention-note"))
                        .child(Environment(self.keyed_collection.display_mode, child_stack))
                        .spacing(8.0)
                        .alignment(rustui::prelude::HorizontalAlignment::Start)
                }
            }
            StoryId::Input => VStack::new()
                .child(Component(
                    "search-field-reference-gallery",
                    SearchFieldReferenceGallery::default,
                ))
                .child(
                    LocalizedText("lab-input-bound-value").with_arguments(
                        MessageArguments::new().with("value", self.input_text.clone()),
                    ),
                )
                .child(
                    TextField("Name", self.input_binding.clone())
                        .id("story-input-name")
                        .localized_placeholder("lab-input-name")
                        .localized_accessibility_label("lab-input-name"),
                )
                .child(
                    SecureField("Password", self.secure_binding.clone())
                        .id("story-input-password")
                        .localized_placeholder("lab-input-password")
                        .localized_accessibility_label("lab-input-password"),
                )
                .child(
                    SearchField("Search", self.input_binding.clone())
                        .id("story-input-search")
                        .localized_placeholder("lab-input-search")
                        .localized_accessibility_label("lab-input-search"),
                )
                .child(
                    TextEditor("Notes", self.input_binding.clone())
                        .id("story-input-notes")
                        .height(96.0)
                        .localized_placeholder("lab-input-notes")
                        .localized_accessibility_label("lab-input-notes"),
                )
                .child(
                    HStack::new()
                        .child(
                            Toggle("Enabled", self.toggle_binding.clone())
                                .id("story-input-toggle")
                                .localized_label("lab-input-enabled"),
                        )
                        .child(
                            Checkbox("Accepted", self.checkbox_binding.clone())
                                .id("story-input-checkbox")
                                .localized_label("lab-input-accepted"),
                        )
                        // Native acceptance probe: sharing the value makes an
                        // accidental disabled activation visible on both controls.
                        .child(
                            Checkbox("Accepted (disabled)", self.checkbox_binding.clone())
                                .id("story-input-checkbox-disabled")
                                .localized_label("lab-input-accepted-disabled")
                                .accessibility_label("Accepted option (disabled)")
                                .localized_accessibility_label(
                                    "lab-input-accepted-disabled-accessibility",
                                )
                                .disabled(true),
                        )
                        .child(
                            Radio("Selected", self.radio_binding.clone())
                                .id("story-input-radio")
                                .localized_label("lab-input-selected"),
                        )
                        .spacing(12.0)
                        .alignment(rustui::prelude::VerticalAlignment::Center),
                )
                .child(CheckboxReferenceGallery::default())
                .child(
                    Slider(self.slider_binding.clone(), 0.0..=100.0)
                        .id("story-input-slider")
                        .accessibility_label("Input value")
                        .step(1.0),
                )
                .child(
                    Stepper("Quantity", self.stepper_binding.clone())
                        .id("story-input-stepper")
                        .range(0..=10),
                )
                .child(Text(format!(
                    "Picker owner value: {}",
                    self.picker_binding.value()
                )))
                .child(
                    Button("Set Picker to Kotlin", self.set_picker_kotlin.clone())
                        .id("story-input-picker-set-kotlin"),
                )
                .child(
                    Picker(
                        "Framework",
                        self.picker_binding.clone(),
                        ["Rust", "Swift", "Kotlin"],
                    )
                    .id("story-input-picker")
                    .disabled(!self.toggle_binding.value()),
                )
                .child(
                    ComboBox(
                        "Rendering engine",
                        self.combo_box_binding.clone(),
                        ["GPUI", "AppKit", "Skia"],
                    )
                    .id("story-input-combo-box")
                    .localized_placeholder("lab-input-combo-placeholder")
                    .search_placeholder("Find an engine")
                    .localized_search_placeholder("lab-input-combo-search")
                    .width(220.0)
                    .menu_width(220.0),
                )
                .child(
                    DatePicker("Choose a date", self.date_binding.clone()).id("story-input-date"),
                )
                .child(
                    ColorPicker("Choose a color", self.color_binding.clone())
                        .id("story-input-color"),
                )
                .child(super::code_editor_gallery::demo())
                .spacing(8.0)
                .alignment(rustui::prelude::HorizontalAlignment::Start),
            StoryId::Card => VStack::new().child(super::card_gallery::demo()),
            StoryId::List => VStack::new().child(Component(
                "list-reconciliation-gallery",
                ListReconciliationGallery::default,
            )),
            StoryId::Table => VStack::new()
                .child(super::table_reconciliation_gallery::demo())
                .child(super::data_grid_accessibility::demo())
                .spacing(8.0)
                .alignment(rustui::prelude::HorizontalAlignment::Start),
            StoryId::Outline => VStack::new().child(super::outline_gallery::demo()),
            StoryId::Drawer => VStack::new().child(super::drawer_gallery::demo()),
            StoryId::SplitView => VStack::new().child(super::split_view_gallery::demo()),
            StoryId::GraphView => VStack::new().child(super::graph_gallery::graph_demo()),
            StoryId::NodeEditor => VStack::new().child(super::graph_gallery::editor_demo()),
            StoryId::Tree => VStack::new()
                .child(Text(format!(
                    "Selected node: {:?}",
                    self.tree_binding.value()
                )))
                .child(
                    Tree(
                        [
                            TreeNode::new("controls", "Controls")
                                .expanded(true)
                                .children([
                                    TreeNode::new("button", "Button"),
                                    TreeNode::new("text-field", "TextField"),
                                ]),
                            TreeNode::new("collections", "Collections")
                                .expanded(true)
                                .children([
                                    TreeNode::new("list", "List"),
                                    TreeNode::new("table", "Table"),
                                    TreeNode::new("tree", "Tree"),
                                ]),
                        ],
                        self.tree_binding.clone(),
                    )
                    .id("story-tree-components")
                    .height(300.0),
                )
                .spacing(8.0)
                .alignment(rustui::prelude::HorizontalAlignment::Start),
            StoryId::Tabs => {
                let mut overview = VStack::new().child(Text("Overview editor")).child(
                    TextEditor("Overview notes", self.retention.tab_overview.clone())
                        .id("retention-editor")
                        .height(80.0),
                );
                if self.retention.tab_conditional_visible {
                    overview = overview.child(
                        TextEditor("Conditional notes", self.retention.tab_conditional.clone())
                            .id("conditional-editor")
                            .height(80.0),
                    );
                }
                let mut tabs = vec![
                    TabItem::new("overview", "Overview", overview)
                        .localized_accessibility_label("lab-tab-overview-accessibility"),
                ];
                if self.retention.details_tab_present {
                    tabs.push(TabItem::new(
                        "details",
                        "Details",
                        VStack::new().child(Text("Details editor")).child(
                            TextEditor("Details notes", self.retention.tab_details.clone())
                                .id("retention-editor")
                                .height(80.0),
                        ),
                    ));
                }
                tabs.push(
                    TabItem::new("disabled", "Unavailable", Text("Disabled tab content"))
                        .disabled(true),
                );
                VStack::new()
                    .child(Text(format!("Selected tab: {}", self.tab_binding.value())))
                    .child(
                        TabView(tabs, self.tab_binding.clone())
                            .id("story-tabs")
                            .style(TabViewStyle::Segmented),
                    )
                    .spacing(8.0)
                    .alignment(rustui::prelude::HorizontalAlignment::Start)
            }
            StoryId::Breadcrumb => VStack::new()
                .child(Text(format!("Current page: {}", self.breadcrumb_page)))
                .child(
                    Breadcrumb([
                        BreadcrumbItem::new("Overview").action(self.breadcrumb_overview.clone()),
                        BreadcrumbItem::new("Components")
                            .action(self.breadcrumb_components.clone()),
                        BreadcrumbItem::new(self.breadcrumb_page.clone()),
                    ])
                    .id("story-breadcrumb"),
                )
                .spacing(8.0)
                .alignment(rustui::prelude::HorizontalAlignment::Start),
            StoryId::Navigation => {
                let mut root = VStack::new().child(Text("Navigation root")).child(
                    TextEditor("Root notes", self.retention.navigation_root.clone())
                        .id("retention-editor")
                        .height(80.0),
                );
                if self.retention.navigation_conditional_visible {
                    root = root.child(
                        TextEditor(
                            "Conditional root notes",
                            self.retention.navigation_conditional.clone(),
                        )
                        .id("conditional-editor")
                        .height(80.0),
                    );
                }
                VStack::new().child(
                    NavigationStack(
                        root.child(
                            NavigationLink("Open details", "details", self.navigation_path.clone())
                                .id("story-navigation-details-link"),
                        ),
                        [
                            NavigationDestination::new(
                                "details",
                                VStack::new()
                                    .child(Text("Details destination"))
                                    .child(
                                        TextEditor(
                                            "Details notes",
                                            self.retention.navigation_details.clone(),
                                        )
                                        .id("retention-editor")
                                        .height(80.0),
                                    )
                                    .child(
                                        NavigationLink(
                                            "Open inspector",
                                            "inspector",
                                            self.navigation_path.clone(),
                                        )
                                        .id("story-navigation-inspector-link"),
                                    ),
                            ),
                            NavigationDestination::new(
                                "inspector",
                                VStack::new().child(Text("Inspector destination")).child(
                                    TextEditor(
                                        "Inspector notes",
                                        self.retention.navigation_inspector.clone(),
                                    )
                                    .id("retention-editor")
                                    .height(80.0),
                                ),
                            ),
                        ],
                        self.navigation_path.clone(),
                    )
                    .id("story-navigation-stack"),
                )
            }
        }
    }
}

#[component]
struct CheckboxReferenceGallery {
    #[state(true)]
    checked: bool,
    #[state(false)]
    unchecked: bool,
}

impl View for CheckboxReferenceGallery {
    fn body(&self) -> impl View {
        VStack::new()
            .spacing(6.0)
            .child(LocalizedText("lab-input-checkbox-reference-states"))
            .child(
                HStack::new()
                    .spacing(12.0)
                    .child(
                        Checkbox("Label", self.checked_binding())
                            .id("checkbox-reference-checked")
                            .localized_label("lab-input-checkbox-reference-label")
                            .accessibility_label("Checked checkbox reference state")
                            .localized_accessibility_label("lab-input-checkbox-reference-checked"),
                    )
                    .child(
                        Checkbox("Label", self.unchecked_binding())
                            .id("checkbox-reference-unchecked")
                            .localized_label("lab-input-checkbox-reference-label")
                            .accessibility_label("Unchecked checkbox reference state")
                            .localized_accessibility_label(
                                "lab-input-checkbox-reference-unchecked",
                            ),
                    ),
            )
    }
}

#[component]
struct SearchFieldReferenceGallery {
    #[state(String::new())]
    empty_query: String,
    #[state(String::from("Design"))]
    active_query: String,
    #[state(String::from("Design"))]
    disabled_query: String,
    #[state(String::new())]
    dark_focused_empty_query: String,
    #[state(String::from("Design"))]
    dark_focused_populated_query: String,
    #[state(String::new())]
    dark_idle_empty_query: String,
    #[state(String::from("Design"))]
    dark_idle_populated_query: String,
    #[state(String::new())]
    recent_query: String,
    #[state(true)]
    recent_menu_open: bool,
    #[state(initial_recent_searches())]
    recent_searches: Vec<String>,
    #[state(String::from("Design"))]
    token_query: String,
    #[state(initial_recipient_tokens())]
    recipient_tokens: Vec<String>,
    #[state(String::from("Studio"))]
    combo_selection: String,
    #[state(String::from("Atlas"))]
    disabled_combo_selection: String,
    #[state(3)]
    disabled_count: i32,
}

fn initial_recent_searches() -> Vec<String> {
    ["Design", "Studio", "Atlas", "Notes"]
        .into_iter()
        .map(String::from)
        .collect()
}

fn initial_recipient_tokens() -> Vec<String> {
    ["Atlas", "Studio", "Notes"]
        .into_iter()
        .map(String::from)
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DarkSearchFieldState {
    Disabled,
    Focused,
    Idle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DarkSearchFieldCase {
    id: &'static str,
    state: DarkSearchFieldState,
    text_present: bool,
}

const DARK_SEARCH_FIELD_CASES: [DarkSearchFieldCase; 6] = [
    DarkSearchFieldCase {
        id: "search-field-reference-disabled",
        state: DarkSearchFieldState::Disabled,
        text_present: false,
    },
    DarkSearchFieldCase {
        id: "search-field-reference-disabled-populated",
        state: DarkSearchFieldState::Disabled,
        text_present: true,
    },
    DarkSearchFieldCase {
        id: "search-field-dark-focused-empty",
        state: DarkSearchFieldState::Focused,
        text_present: false,
    },
    DarkSearchFieldCase {
        id: "search-field-dark-focused-populated",
        state: DarkSearchFieldState::Focused,
        text_present: true,
    },
    DarkSearchFieldCase {
        id: "search-field-dark-idle-empty",
        state: DarkSearchFieldState::Idle,
        text_present: false,
    },
    DarkSearchFieldCase {
        id: "search-field-dark-idle-populated",
        state: DarkSearchFieldState::Idle,
        text_present: true,
    },
];

fn dark_search_field_specimen(case: DarkSearchFieldCase, value: Binding<String>) -> impl View {
    let title_message = match (case.state, case.text_present) {
        (DarkSearchFieldState::Disabled, false) => "lab-search-gallery-dark-disabled-empty",
        (DarkSearchFieldState::Disabled, true) => "lab-search-gallery-dark-disabled-populated",
        (DarkSearchFieldState::Focused, false) => "lab-search-gallery-dark-focused-empty",
        (DarkSearchFieldState::Focused, true) => "lab-search-gallery-dark-focused-populated",
        (DarkSearchFieldState::Idle, false) => "lab-search-gallery-dark-idle-empty",
        (DarkSearchFieldState::Idle, true) => "lab-search-gallery-dark-idle-populated",
    };
    Card(
        format!("{}-card", case.id),
        VStack::new()
            .spacing(6.0)
            .child(LocalizedText(title_message))
            .child(if case.state == DarkSearchFieldState::Focused {
                LocalizedText("lab-search-gallery-dark-focus-help")
            } else {
                Text("")
            })
            .child(
                SearchField("Search components...", value)
                    .id(case.id)
                    .localized_placeholder("lab-search-gallery-placeholder")
                    .localized_accessibility_label(title_message)
                    .disabled(case.state == DarkSearchFieldState::Disabled),
            ),
    )
}

impl SearchFieldReferenceGallery {
    fn clear_recent_searches(&mut self) {
        self.recent_searches.clear();
    }

    fn select_recent_design(&mut self) {
        self.recent_query = "Design".into();
        self.recent_menu_open = false;
    }

    fn select_recent_studio(&mut self) {
        self.recent_query = "Studio".into();
        self.recent_menu_open = false;
    }

    fn select_recent_atlas(&mut self) {
        self.recent_query = "Atlas".into();
        self.recent_menu_open = false;
    }

    fn select_recent_notes(&mut self) {
        self.recent_query = "Notes".into();
        self.recent_menu_open = false;
    }
}

impl View for SearchFieldReferenceGallery {
    fn body(&self) -> impl View {
        let mut recent_rows = VStack::new().spacing(4.0);
        if self
            .recent_searches()
            .iter()
            .any(|search| search == "Design")
        {
            recent_rows = recent_rows.child(
                Button("Design", self.action(Self::select_recent_design))
                    .id("search-field-reference-recent-design"),
            );
        }
        if self
            .recent_searches()
            .iter()
            .any(|search| search == "Studio")
        {
            recent_rows = recent_rows.child(
                Button("Studio", self.action(Self::select_recent_studio))
                    .id("search-field-reference-recent-studio"),
            );
        }
        if self
            .recent_searches()
            .iter()
            .any(|search| search == "Atlas")
        {
            recent_rows = recent_rows.child(
                Button("Atlas", self.action(Self::select_recent_atlas))
                    .id("search-field-reference-recent-atlas"),
            );
        }
        if self
            .recent_searches()
            .iter()
            .any(|search| search == "Notes")
        {
            recent_rows = recent_rows.child(
                Button("Notes", self.action(Self::select_recent_notes))
                    .id("search-field-reference-recent-notes"),
            );
        }

        VStack::new()
            .spacing(8.0)
            .child(
                HStack::new()
                    .spacing(8.0)
                    .child(Card(
                        "search-field-empty",
                        VStack::new()
                            .spacing(6.0)
                            .child(LocalizedText("lab-search-gallery-empty-title"))
                            .child(LocalizedText("lab-search-gallery-empty-summary"))
                            .child(
                                SearchField("Search components...", self.empty_query_binding())
                                    .id("search-field-reference-empty")
                                    .localized_placeholder("lab-search-gallery-placeholder")
                                    .localized_accessibility_label("lab-search-gallery-label"),
                            ),
                    ))
                    .child(Card(
                        "search-field-active",
                        VStack::new()
                            .spacing(6.0)
                            .child(LocalizedText("lab-search-gallery-active-title"))
                            .child(LocalizedText("lab-search-gallery-active-summary"))
                            .child(
                                SearchField("Search components...", self.active_query_binding())
                                    .id("search-field-reference-active")
                                    .localized_placeholder("lab-search-gallery-placeholder")
                                    .localized_accessibility_label("lab-search-gallery-label"),
                            ),
                    ))
                    .child(Card(
                        "search-field-recent-searches",
                        VStack::new()
                            .spacing(6.0)
                            .child(LocalizedText("lab-search-gallery-recent-title"))
                            .child(LocalizedText("lab-search-gallery-recent-summary"))
                            .child(
                                SearchField("Search components...", self.recent_query_binding())
                                    .id("search-field-reference-recent")
                                    .localized_placeholder("lab-search-gallery-placeholder")
                                    .localized_accessibility_label("lab-search-gallery-label"),
                            )
                            .child(
                                Popover(
                                    "search-field-reference-recent-menu",
                                    "Recent Searches",
                                    self.recent_menu_open_binding(),
                                    VStack::new().spacing(6.0).child(recent_rows).child(
                                        Button(
                                            "Clear Recent Searches",
                                            self.action(Self::clear_recent_searches),
                                        )
                                        .id("search-field-reference-clear-recent")
                                        .localized_label("lab-search-gallery-clear-recent"),
                                    ),
                                )
                                .localized_label("lab-search-gallery-recent-menu"),
                            ),
                    )),
            )
            .child(LocalizedText("lab-search-gallery-dark-note"))
            .child(
                VStack::new()
                    .spacing(8.0)
                    .child(
                        HStack::new()
                            .spacing(8.0)
                            .child(dark_search_field_specimen(
                                DARK_SEARCH_FIELD_CASES[0],
                                Binding::new(
                                    String::new(),
                                    |value: &String| value.clone(),
                                    |_, _| {},
                                ),
                            ))
                            .child(dark_search_field_specimen(
                                DARK_SEARCH_FIELD_CASES[1],
                                Binding::new(
                                    self.disabled_query(),
                                    |value: &String| value.clone(),
                                    |_, _| {},
                                ),
                            )),
                    )
                    .child(
                        HStack::new()
                            .spacing(8.0)
                            .child(dark_search_field_specimen(
                                DARK_SEARCH_FIELD_CASES[2],
                                self.dark_focused_empty_query_binding(),
                            ))
                            .child(dark_search_field_specimen(
                                DARK_SEARCH_FIELD_CASES[3],
                                self.dark_focused_populated_query_binding(),
                            )),
                    )
                    .child(
                        HStack::new()
                            .spacing(8.0)
                            .child(dark_search_field_specimen(
                                DARK_SEARCH_FIELD_CASES[4],
                                self.dark_idle_empty_query_binding(),
                            ))
                            .child(dark_search_field_specimen(
                                DARK_SEARCH_FIELD_CASES[5],
                                self.dark_idle_populated_query_binding(),
                            )),
                    ),
            )
            .child(
                HStack::new()
                    .spacing(8.0)
                    .child(Card(
                        "search-field-token-recipients",
                        VStack::new()
                            .spacing(6.0)
                            .child(LocalizedText("lab-search-gallery-token-title"))
                            .child(LocalizedText("lab-search-gallery-token-summary"))
                            .child(
                                TokenField(
                                    "Add recipients",
                                    self.recipient_tokens_binding(),
                                    self.token_query_binding(),
                                )
                                .id("search-field-reference-token-query")
                                .localized_placeholder("lab-input-token-recipients"),
                            ),
                    ))
                    .child(Card(
                        "search-field-editable-combo-box",
                        VStack::new()
                            .spacing(6.0)
                            .child(LocalizedText("lab-search-gallery-combo-title"))
                            .child(LocalizedText("lab-search-gallery-combo-summary"))
                            .child(
                                ComboBox(
                                    "Framework",
                                    self.combo_selection_binding(),
                                    ["Studio", "Atlas", "Notes", "Design"],
                                )
                                .id("search-field-reference-editable-combo")
                                .search_placeholder("Choose a workspace"),
                            ),
                    ))
                    .child(Card(
                        "search-field-disabled-examples",
                        VStack::new()
                            .spacing(6.0)
                            .child(LocalizedText("lab-search-gallery-disabled-title"))
                            .child(LocalizedText("lab-search-gallery-disabled-summary"))
                            .child(
                                ComboBox(
                                    "Framework",
                                    self.disabled_combo_selection_binding(),
                                    ["Atlas", "Studio", "Notes"],
                                )
                                .id("search-field-reference-disabled-combo")
                                .disabled(true),
                            )
                            .child(
                                Stepper("Count (disabled)", self.disabled_count_binding())
                                    .id("search-field-reference-disabled-count")
                                    .range(0..=10)
                                    .disabled(true),
                            ),
                    )),
            )
    }
}

#[cfg(test)]
mod search_field_reference_tests {
    use super::{DARK_SEARCH_FIELD_CASES, DarkSearchFieldState, SearchFieldReferenceGallery};

    #[test]
    fn search_field_reference_gallery_starts_with_the_six_reference_states() {
        let gallery = SearchFieldReferenceGallery::default();

        assert!(gallery.empty_query.is_empty());
        assert_eq!(gallery.active_query, "Design");
        assert_eq!(gallery.disabled_query, "Design");
        assert!(gallery.dark_focused_empty_query.is_empty());
        assert_eq!(gallery.dark_focused_populated_query, "Design");
        assert!(gallery.dark_idle_empty_query.is_empty());
        assert_eq!(gallery.dark_idle_populated_query, "Design");
        assert_eq!(
            gallery.recent_searches,
            ["Design", "Studio", "Atlas", "Notes"]
        );
        assert_eq!(gallery.recipient_tokens, ["Atlas", "Studio", "Notes"]);
        assert!(gallery.recent_menu_open);
        assert_eq!(gallery.combo_selection, "Studio");
        assert_eq!(gallery.disabled_combo_selection, "Atlas");
        assert_eq!(gallery.disabled_count, 3);
    }

    #[test]
    fn recipient_token_field_bindings_update_the_gallery_state() {
        let mut gallery = SearchFieldReferenceGallery::default();
        let tokens = gallery.recipient_tokens_binding();
        let draft = gallery.token_query_binding();

        assert!(draft.write_to(&mut gallery, "Research".into()));
        assert_eq!(gallery.token_query(), "Research");
        assert!(tokens.write_to(&mut gallery, vec!["Atlas".into(), "Notes".into()]));
        assert_eq!(
            tokens.read_from(&gallery),
            Some(vec!["Atlas".into(), "Notes".into()])
        );
    }

    #[test]
    fn dark_search_field_matrix_covers_each_reference_state_and_text_value() {
        assert_eq!(
            DARK_SEARCH_FIELD_CASES.map(|case| (case.state, case.text_present)),
            [
                (DarkSearchFieldState::Disabled, false),
                (DarkSearchFieldState::Disabled, true),
                (DarkSearchFieldState::Focused, false),
                (DarkSearchFieldState::Focused, true),
                (DarkSearchFieldState::Idle, false),
                (DarkSearchFieldState::Idle, true),
            ]
        );
        assert_eq!(
            DARK_SEARCH_FIELD_CASES[0].id,
            "search-field-reference-disabled"
        );
        assert_eq!(
            DARK_SEARCH_FIELD_CASES[1].id,
            "search-field-reference-disabled-populated"
        );
    }
}

#[cfg(test)]
mod checkbox_reference_tests {
    use super::CheckboxReferenceGallery;

    #[test]
    fn checkbox_reference_gallery_starts_with_checked_and_unchecked_states() {
        let gallery = CheckboxReferenceGallery::default();

        assert!(gallery.checked());
        assert!(!gallery.unchecked());
    }

    #[test]
    fn checkbox_reference_state_bindings_update_independently() {
        let mut gallery = CheckboxReferenceGallery::default();

        assert!(gallery.checked_binding().write_to(&mut gallery, false));
        assert!(!gallery.checked());
        assert!(!gallery.unchecked());

        assert!(gallery.unchecked_binding().write_to(&mut gallery, true));
        assert!(!gallery.checked());
        assert!(gallery.unchecked());
    }
}

#[component]
struct ListReconciliationGallery {
    #[state(list_rows())]
    rows: Vec<String>,
    #[state(None)]
    selection: Option<usize>,
}

fn list_rows() -> Vec<String> {
    [
        "Button",
        "Text",
        "TextField",
        "Slider",
        "Picker",
        "DatePicker",
        "ColorPicker",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

impl ListReconciliationGallery {
    fn remove_last_row(&mut self) {
        self.rows.pop();
    }

    fn restore_rows(&mut self) {
        self.rows = list_rows();
    }
}

impl View for ListReconciliationGallery {
    fn body(&self) -> impl View {
        VStack::new()
            .spacing(8.0)
            .child(Text(format!("Rows: {}", self.rows().len())))
            .child(Text(format!("Selected source row: {:?}", self.selection())))
            .child(
                Button("Remove last row", self.action(Self::remove_last_row))
                    .id("list-reconciliation-remove-last")
                    .disabled(self.rows().is_empty()),
            )
            .child(
                Button("Restore rows", self.action(Self::restore_rows))
                    .id("list-reconciliation-restore-rows"),
            )
            .child(
                List(self.rows().clone(), self.selection_binding())
                    .id("story-list-components")
                    .height(260.0)
                    .searchable(true)
                    .search_placeholder("Find a component"),
            )
    }
}
