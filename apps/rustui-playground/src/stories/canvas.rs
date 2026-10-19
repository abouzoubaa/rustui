//! Isolated rendering for the stories currently supported by RustUI Playground.

use rustui::{
    prelude::{
        Binding, Breadcrumb, BreadcrumbItem, Button, Checkbox, ColorPicker, ComboBox, Component,
        DatePicker, Environment, FontFallbackPolicy, HStack, Hsla, Keyed, List, LocalizedText,
        NaiveDate, NavigationDestination, NavigationLink, NavigationStack, Picker, Radio,
        SearchField, SecureField, Slider, Stepper, TabItem, TabView, TabViewStyle, Table, Text,
        TextEditor, TextField, Toggle, Tree, TreeNode, VStack, View,
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
        increment: Action,
        reset: Action,
    ) -> Self {
        Self {
            story,
            count,
            button,
            input_text,
            input_binding,
            toggle_binding,
            checkbox_binding,
            radio_binding,
            slider_binding,
            stepper_binding,
            picker_binding,
            combo_box_binding,
            date_binding,
            color_binding,
            list_binding,
            table_binding,
            tree_binding,
            tab_binding,
            breadcrumb_page,
            breadcrumb_overview,
            breadcrumb_components,
            navigation_path,
            retention,
            keyed_collection,
            increment,
            reset,
        }
    }
}

impl View for StoryCanvas {
    fn body(&self) -> impl View {
        match self.story {
            StoryId::Button => VStack::new()
                .child(Text(format!("Count: {}", self.count)))
                .child(
                    HStack::new()
                        .child(
                            Button("Increment", self.increment.clone())
                                .id("story-button-increment")
                                .button_style(self.button.style)
                                .disabled(self.button.disabled),
                        )
                        .child(
                            Button("Reset", self.reset.clone())
                                .id("story-button-reset")
                                .button_style(self.button.style)
                                .disabled(self.button.disabled),
                        )
                        .spacing(8.0)
                        .alignment(rustui::prelude::VerticalAlignment::Center),
                )
                .child(
                    Button("Unavailable", self.reset.clone())
                        .id("story-button-disabled")
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
                        .height(80.0),
                );
                let beta = Keyed(
                    "beta",
                    TextEditor("Beta row", self.keyed_collection.beta.clone())
                        .id("row-editor")
                        .height(80.0),
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
                    .child(Text("Outer VStack"))
                    .child(
                        HStack::new()
                            .child(Text("HStack item"))
                            .child(
                                Button("Layout action", self.increment.clone())
                                    .id("story-layout-action")
                                    .button_style(self.button.style),
                            )
                            .spacing(8.0)
                            .alignment(rustui::prelude::VerticalAlignment::Center),
                    )
                    .child(Text(
                        "Reorder keeps local cursor/undo by key; removal resets it; text stays bound",
                    ))
                    .children(rows)
                    .child(Text("Retained child components: change parent titles without resetting counters or notes"))
                    .child(Environment(self.keyed_collection.display_mode, child_stack))
                    .spacing(8.0)
                    .alignment(rustui::prelude::HorizontalAlignment::Start)
                }
            }
            StoryId::Input => VStack::new()
                .child(Text(format!("Bound value: {}", self.input_text)))
                .child(
                    TextField("Name", self.input_binding.clone())
                        .id("story-input-name")
                        .accessibility_label("Name"),
                )
                .child(
                    SecureField("Password", self.input_binding.clone())
                        .id("story-input-password")
                        .accessibility_label("Password"),
                )
                .child(
                    SearchField("Search", self.input_binding.clone())
                        .id("story-input-search")
                        .accessibility_label("Search"),
                )
                .child(
                    TextEditor("Notes", self.input_binding.clone())
                        .id("story-input-notes")
                        .height(96.0)
                        .accessibility_label("Notes"),
                )
                .child(
                    HStack::new()
                        .child(
                            Toggle("Enabled", self.toggle_binding.clone()).id("story-input-toggle"),
                        )
                        .child(
                            Checkbox("Accepted", self.checkbox_binding.clone())
                                .id("story-input-checkbox"),
                        )
                        .child(
                            Radio("Selected", self.radio_binding.clone()).id("story-input-radio"),
                        )
                        .spacing(12.0)
                        .alignment(rustui::prelude::VerticalAlignment::Center),
                )
                .child(
                    Slider(self.slider_binding.clone(), 0.0..=100.0)
                        .id("story-input-slider")
                        .step(1.0),
                )
                .child(
                    Stepper("Quantity", self.stepper_binding.clone())
                        .id("story-input-stepper")
                        .range(0..=10),
                )
                .child(
                    Picker(
                        "Framework",
                        self.picker_binding.clone(),
                        ["Rust", "Swift", "Kotlin"],
                    )
                    .id("story-input-picker"),
                )
                .child(
                    ComboBox(
                        "Rendering engine",
                        self.combo_box_binding.clone(),
                        ["GPUI", "AppKit", "Skia"],
                    )
                    .id("story-input-combo-box")
                    .search_placeholder("Find an engine"),
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
            StoryId::List => VStack::new()
                .child(Text(format!(
                    "Selected source row: {:?}",
                    self.list_binding.value()
                )))
                .child(
                    List(
                        [
                            "Button",
                            "Text",
                            "TextField",
                            "Slider",
                            "Picker",
                            "DatePicker",
                            "ColorPicker",
                        ],
                        self.list_binding.clone(),
                    )
                    .id("story-list-components")
                    .height(260.0)
                    .searchable(true)
                    .search_placeholder("Find a component"),
                )
                .spacing(8.0)
                .alignment(rustui::prelude::HorizontalAlignment::Start),
            StoryId::Table => VStack::new()
                .child(super::table_reconciliation_gallery::demo())
                .spacing(8.0)
                .alignment(rustui::prelude::HorizontalAlignment::Start),
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
                let mut tabs = vec![TabItem::new("overview", "Overview", overview)];
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
