//! A public-API document outline with controlled expansion independent of node declarations.

use rustui::prelude::*;
use std::collections::HashSet;

pub fn demo() -> impl View {
    Component("controlled-outline-gallery", OutlineGallery::default)
}

#[component]
struct OutlineGallery {
    #[state(None)]
    selected: Option<String>,
    #[state(HashSet::from([String::from("document")]))]
    expanded: HashSet<String>,
    #[state(true)]
    branch_present: bool,
    #[state(false)]
    reversed: bool,
    #[state(false)]
    relabeled: bool,
    #[state(true)]
    visible: bool,
    #[state(0)]
    activations: usize,
}

impl OutlineGallery {
    fn collapse_all(&mut self) {
        self.expanded.clear();
    }
    fn expand_all(&mut self) {
        self.expanded = HashSet::from([String::from("document"), String::from("section")]);
    }
    fn select_heading(&mut self) {
        self.selected = Some(String::from("heading"));
    }
    fn toggle_branch(&mut self) {
        self.branch_present = !self.branch_present;
    }
    fn reverse(&mut self) {
        self.reversed = !self.reversed;
    }
    fn relabel(&mut self) {
        self.relabeled = !self.relabeled;
    }
    fn toggle_visible(&mut self) {
        self.visible = !self.visible;
    }
    fn activate(&mut self) {
        self.activations += 1;
    }
}

impl View for OutlineGallery {
    fn body(&self) -> impl View {
        let mut children = vec![TreeNode::new("summary", "Summary")];
        if self.branch_present() {
            children.push(
                TreeNode::new(
                    "section",
                    if self.relabeled() {
                        "Renamed section"
                    } else {
                        "Section"
                    },
                )
                .child(TreeNode::new("heading", "Heading")),
            );
        }
        children.push(
            TreeNode::new("disabled", "Disabled branch")
                .disabled(true)
                .child(TreeNode::new("disabled-heading", "Unavailable heading")),
        );
        if self.reversed() {
            children.reverse();
        }
        let nodes = vec![TreeNode::new("document", "Document").children(children)];
        let mut expanded = self.expanded().iter().cloned().collect::<Vec<_>>();
        expanded.sort();
        VStack::new().spacing(8.0)
            .child(Text("Outline: owner-controlled document hierarchy"))
            .child(HStack::new().spacing(6.0)
                .child(Button("Collapse all", self.action(Self::collapse_all)).id("outline-collapse"))
                .child(Button("Expand all", self.action(Self::expand_all)).id("outline-expand"))
                .child(Button("Select heading", self.action(Self::select_heading)).id("outline-select")))
            .child(HStack::new().spacing(6.0)
                .child(Button("Remove/restore section", self.action(Self::toggle_branch)).id("outline-branch"))
                .child(Button("Reverse order", self.action(Self::reverse)).id("outline-reverse"))
                .child(Button("Relabel", self.action(Self::relabel)).id("outline-relabel"))
                .child(Button("Unmount/remount", self.action(Self::toggle_visible)).id("outline-visible")))
            .child(Text(format!("Selected: {} | Expanded IDs: {} | Activations: {}",
                self.selected().as_deref().unwrap_or("none"), expanded.join(", "), self.activations())))
            .child(Text("Collapse keeps a hidden selection without reopening ancestors. Removed branch IDs normalize away."))
            .child(if self.visible() {
                AnyView::new(Outline("document-outline", nodes, self.selected_binding(), self.expanded_binding())
                    .accessibility_label("Document outline").height(280.0).on_activate(self.action(Self::activate)))
            } else {
                AnyView::new(Text("Outline unmounted; owner bindings are retained."))
            })
    }
}
