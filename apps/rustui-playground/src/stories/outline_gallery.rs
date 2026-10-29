//! A public-API document outline with controlled expansion independent of node declarations.

use rustui::localization::MessageArguments;
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
        let mut children =
            vec![TreeNode::new("summary", "Summary").localized_label("lab-outline-summary")];
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
                .localized_label(if self.relabeled() {
                    "lab-outline-section-renamed"
                } else {
                    "lab-outline-section"
                })
                .child(
                    TreeNode::new("heading", "Heading").localized_label("lab-outline-heading-node"),
                ),
            );
        }
        children.push(
            TreeNode::new("disabled", "Disabled branch")
                .localized_label("lab-outline-disabled-node")
                .disabled(true)
                .child(
                    TreeNode::new("disabled-heading", "Unavailable heading")
                        .localized_label("lab-outline-unavailable-node"),
                ),
        );
        if self.reversed() {
            children.reverse();
        }
        let nodes = vec![
            TreeNode::new("document", "Document")
                .localized_label("lab-outline-document")
                .children(children),
        ];
        let mut expanded = self.expanded().iter().cloned().collect::<Vec<_>>();
        expanded.sort();
        VStack::new()
            .spacing(8.0)
            .child(LocalizedText("lab-outline-demo-heading"))
            .child(
                HStack::new()
                    .spacing(6.0)
                    .child(
                        Button("Collapse all", self.action(Self::collapse_all))
                            .localized_label("lab-outline-collapse")
                            .id("outline-collapse"),
                    )
                    .child(
                        Button("Expand all", self.action(Self::expand_all))
                            .localized_label("lab-outline-expand")
                            .id("outline-expand"),
                    )
                    .child(
                        Button("Select heading", self.action(Self::select_heading))
                            .localized_label("lab-outline-select")
                            .id("outline-select"),
                    ),
            )
            .child(
                HStack::new()
                    .spacing(6.0)
                    .child(
                        Button("Remove/restore section", self.action(Self::toggle_branch))
                            .localized_label("lab-outline-branch")
                            .id("outline-branch"),
                    )
                    .child(
                        Button("Reverse order", self.action(Self::reverse))
                            .localized_label("lab-outline-reverse")
                            .id("outline-reverse"),
                    )
                    .child(
                        Button("Relabel", self.action(Self::relabel))
                            .localized_label("lab-outline-relabel")
                            .id("outline-relabel"),
                    )
                    .child(
                        Button("Unmount/remount", self.action(Self::toggle_visible))
                            .localized_label("lab-outline-visible")
                            .id("outline-visible"),
                    ),
            )
            .child(
                LocalizedText("lab-outline-status").with_arguments(
                    MessageArguments::new()
                        .with(
                            "selection",
                            self.selected().as_deref().unwrap_or("none").to_owned(),
                        )
                        .with("expanded", expanded.join(", "))
                        .with("count", self.activations().to_string()),
                ),
            )
            .child(LocalizedText("lab-outline-help"))
            .child(if self.visible() {
                AnyView::new(
                    Outline(
                        "document-outline",
                        nodes,
                        self.selected_binding(),
                        self.expanded_binding(),
                    )
                    .accessibility_label("Document outline")
                    .height(280.0)
                    .on_activate(self.action(Self::activate)),
                )
            } else {
                AnyView::new(LocalizedText("lab-outline-unmounted"))
            })
    }
}
