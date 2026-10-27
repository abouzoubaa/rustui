//! Public RustUI source shown for each Playground story.

use super::component_catalog::StoryId;

pub fn source_for(story: StoryId) -> &'static str {
    match story {
        StoryId::Button => {
            r#"Button("Increment", increment)
    .button_style(ButtonStyle::BorderedProminent)"#
        }
        StoryId::Text => {
            r#"use rustui::localization::MessageArguments;

let arguments = MessageArguments::new().with("name", "Ada").with("count", 3);
LocalizedText("lab-i18n-sample").with_arguments(arguments)"#
        }
        StoryId::Card => {
            r#"Card("sample", TextField("First card", value_binding).localized_placeholder("lab-card-first-placeholder"))"#
        }
        StoryId::Layout => {
            r#"let alpha = Keyed(
    "alpha",
    TextEditor("Alpha row", self.keyed_alpha_text_binding())
        .id("row-editor")
        .height(80.0),
);
let beta = Keyed(
    "beta",
    TextEditor("Beta row", self.keyed_beta_text_binding())
        .id("row-editor")
        .height(80.0),
);
let rows = if self.keyed_rows_reversed() {
    vec![self.keyed_beta_present().then_some(beta), Some(alpha)]
} else {
    vec![Some(alpha), self.keyed_beta_present().then_some(beta)]
};

// apps/rustui-playground/src/stories/child_component.rs
let child_alpha = Component("child-alpha", move || {
    ChildComponent::named(alpha_label.clone())
})
.props(alpha_title, ChildComponent::apply_title);
let child_beta = Component("child-beta", move || {
    ChildComponent::named(beta_label.clone())
})
.props(beta_title, ChildComponent::apply_title);
let children = if self.keyed_rows_reversed() {
    vec![self.keyed_beta_present().then_some(child_beta), Some(child_alpha)]
} else {
    vec![Some(child_alpha), self.keyed_beta_present().then_some(child_beta)]
};
let child_stack = VStack::new().children(children);
Environment(display_mode, child_stack)

// Child notes and counters remain owned by their keyed components.
// EnvironmentReader samples the scoped RustUI value when the parent renders."#
        }
        StoryId::Input => {
            r#"VStack!
    TextField("Name", self.input_text_binding()).id("name")
    SecureField("Password", self.secure_text_binding()).id("password")
    SearchField("Search", self.input_text_binding()).id("search")
    TextEditor("Notes", self.input_text_binding()).id("notes")
    Toggle("Enabled", self.toggle_value_binding()).id("enabled")
    Checkbox("Accepted", self.checkbox_value_binding()).id("accepted")
    Radio("Selected", self.radio_value_binding()).id("selected")
    Slider(self.slider_value_binding(), 0.0..=100.0).id("volume")
    Stepper("Quantity", self.stepper_value_binding()).range(0..=10)
    Picker("Language", self.picker_value_binding(), ["Rust", "Swift"])
    ComboBox("Rendering", self.combo_box_value_binding(), ["GPUI", "AppKit"])
    DatePicker("Choose a date", self.date_value_binding())
    ColorPicker("Choose a color", self.color_value_binding())"#
        }
        StoryId::List => {
            r#"List(
    ["Button", "Text", "TextField", "Slider", "Picker"],
    self.list_selection_binding(),
)
.id("components")
.height(260.0)
.searchable(true)"#
        }
        StoryId::Table => {
            r#"Table(
    ["Component", "Status"],
    [["Button", "Implemented"], ["List", "Implemented"]],
    self.table_selection_binding(),
)
.id("component-table")
.height(300.0)"#
        }
        StoryId::Outline => {
            r#"Outline(
    "document-outline",
    [TreeNode::new("document", "Document")
        .child(TreeNode::new("heading", "Heading"))],
    self.selected_binding(),
    self.expanded_binding(),
)
.accessibility_label("Document outline")
.height(280.0)"#
        }
        StoryId::GraphView => {
            r#"GraphView(
    "dependency-graph",
    self.document().clone(),
    self.selection_binding(),
    self.viewport_binding(),
)
.accessibility_label("Dependency graph")
.on_activate(self.action(Self::activate))"#
        }
        StoryId::NodeEditor => {
            r#"NodeEditor(
    "node-editor",
    self.document_binding(),
    self.selection_binding(),
    self.viewport_binding(),
)
.accessibility_label("Node editor")
.on_activate(self.action(Self::activate))"#
        }
        StoryId::SplitView => {
            r#"SplitView::horizontal("editor")
    .panel(ResizablePanel::new("sidebar", Text("Files")).initial_size(180.0))
    .panel(ResizablePanel::new("document", Text("Document")))"#
        }
        StoryId::Drawer => {
            r#"Drawer("details", self.visible_binding())
    .title("Details")
    .edge(DrawerEdge::Right)
    .size(350.0)
    .accessibility_label("Details drawer")
    .localized_close_label("rustui-action-close")
    .content(TextField("Notes", self.notes_binding()))"#
        }
        StoryId::Tree => {
            r#"Tree(
    [TreeNode::new("components", "Components")
        .expanded(true)
        .children([
            TreeNode::new("button", "Button"),
            TreeNode::new("table", "Table"),
        ])],
    self.tree_selection_binding(),
)
.id("component-tree")
.height(300.0)"#
        }
        StoryId::Tabs => {
            r#"let mut overview = VStack::new().child(
    TextEditor("Overview notes", self.tab_overview_notes_binding())
        .id("retention-editor")
        .height(80.0),
);
if self.tab_conditional_visible() {
    overview = overview.child(
        TextEditor("Conditional notes", self.tab_conditional_notes_binding())
            .id("conditional-editor")
            .height(80.0),
    );
}

let mut tabs = vec![TabItem::new("overview", "Overview", overview)];
if self.details_tab_present() {
    tabs.push(TabItem::new(
        "details",
        "Details",
        TextEditor("Details notes", self.tab_details_notes_binding())
            .id("retention-editor")
            .height(80.0),
    ));
}

TabView(tabs, self.tab_selection_binding())
    .id("story-tabs")
    .style(TabViewStyle::Segmented)"#
        }
        StoryId::Breadcrumb => {
            r#"Breadcrumb([
    BreadcrumbItem::new("Overview").action(self.breadcrumb_overview.clone()),
    BreadcrumbItem::new("Components").action(self.breadcrumb_components.clone()),
    BreadcrumbItem::new(self.breadcrumb_page.clone()),
])
.id("story-breadcrumb")"#
        }
        StoryId::Navigation => {
            r#"let path = self.navigation_path_binding();
let root = VStack::new()
    .child(TextEditor("Root notes", self.navigation_root_notes_binding()))
    .child(NavigationLink("Open details", "details", path.clone()));
let details = VStack::new()
    .child(TextEditor("Details notes", self.navigation_details_notes_binding()))
    .child(NavigationLink("Open inspector", "inspector", path.clone()));
let inspector = TextEditor(
    "Inspector notes",
    self.navigation_inspector_notes_binding(),
);
NavigationStack(
    root,
    [
        NavigationDestination::new("details", details),
        NavigationDestination::new("inspector", inspector),
    ],
    path,
)
.id("story-navigation-stack")"#
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::component_catalog::STORIES;
    use super::source_for;

    #[test]
    fn every_rendered_story_has_illustrative_source() {
        for story in STORIES {
            assert!(
                !source_for(story.id).trim().is_empty(),
                "missing source preview for {}",
                story.title
            );
        }
    }
}
