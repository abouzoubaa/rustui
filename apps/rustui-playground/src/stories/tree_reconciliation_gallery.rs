//! Stable-ID Tree updates and native expansion reconciliation examples.

use rustui::localization::MessageArguments;
use rustui::prelude::*;

pub fn demo() -> impl View {
    Component(
        "tree-reconciliation-gallery",
        TreeReconciliationGallery::default,
    )
}

#[component]
struct TreeReconciliationGallery {
    #[state(None)]
    selection: Option<String>,
    #[state(false)]
    reversed: bool,
    #[state(true)]
    branch_present: bool,
    #[state(true)]
    declared_root_expanded: bool,
    #[state(false)]
    relabeled: bool,
    #[state(None)]
    last_activated_leaf: Option<String>,
    #[state(0)]
    leaf_activation_count: u32,
}

impl TreeReconciliationGallery {
    fn reverse(&mut self) {
        self.reversed = !self.reversed;
    }

    fn toggle_branch(&mut self) {
        self.branch_present = !self.branch_present;
    }

    fn toggle_declared_expansion(&mut self) {
        self.declared_root_expanded = !self.declared_root_expanded;
    }

    fn relabel(&mut self) {
        self.relabeled = !self.relabeled;
    }

    fn activate_selected_leaf(&mut self) {
        if let Some(selection) = self.selection.clone() {
            self.last_activated_leaf = Some(selection);
            self.leaf_activation_count = self.leaf_activation_count.saturating_add(1);
        }
    }
}

impl View for TreeReconciliationGallery {
    fn body(&self) -> impl View {
        let workspace_label = if self.relabeled() {
            "Workspace · renamed"
        } else {
            "Workspace"
        };
        let branch_label = if self.relabeled() {
            "Branch · renamed"
        } else {
            "Branch"
        };
        let mut workspace_children = Vec::new();
        let branch = TreeNode::new("branch", branch_label)
            .localized_label(if self.relabeled() {
                "lab-tree-branch-renamed"
            } else {
                "lab-tree-branch"
            })
            .expanded(true)
            .child(
                TreeNode::new("child-folder", "Nested child folder")
                    .localized_label("lab-tree-nested")
                    .expanded(true)
                    .child(
                        TreeNode::new("grandchild", "Grandchild leaf")
                            .localized_label("lab-tree-grandchild"),
                    ),
            );
        let sibling =
            TreeNode::new("sibling", "Stable sibling").localized_label("lab-tree-sibling");

        if self.branch_present() {
            workspace_children.push(branch);
        }
        workspace_children.push(sibling);
        if self.reversed() {
            workspace_children.reverse();
        }

        let workspace = TreeNode::new("workspace", workspace_label)
            .localized_label(if self.relabeled() {
                "lab-tree-workspace-renamed"
            } else {
                "lab-tree-workspace"
            })
            .expanded(self.declared_root_expanded())
            .children(workspace_children);
        let archive = TreeNode::new("archive", "Archive")
            .localized_label("lab-tree-archive")
            .expanded(true)
            .child(
                TreeNode::new("archive-item", "Archived item")
                    .localized_label("lab-tree-archived-item"),
            );
        let roots = if self.reversed() {
            vec![archive, workspace]
        } else {
            vec![workspace, archive]
        };

        VStack::new()
            .spacing(8.0)
            .child(LocalizedText("lab-tree-demo-heading"))
            .child(
                VStack::new()
                    .spacing(6.0)
                    .child(
                        HStack::new()
                            .spacing(6.0)
                            .child(
                                Button("Reverse order", self.action(Self::reverse))
                                    .localized_label("lab-tree-reverse")
                                    .id("tree-reconcile-reverse"),
                            )
                            .child(
                                Button("Relabel", self.action(Self::relabel))
                                    .localized_label("lab-tree-relabel")
                                    .id("tree-reconcile-relabel"),
                            ),
                    )
                    .child(
                        HStack::new()
                            .spacing(6.0)
                            .child(
                                Button(
                                    if self.branch_present() {
                                        "Remove branch"
                                    } else {
                                        "Restore branch"
                                    },
                                    self.action(Self::toggle_branch),
                                )
                                .localized_label(if self.branch_present() {
                                    "lab-tree-remove-branch"
                                } else {
                                    "lab-tree-restore-branch"
                                })
                                .id("tree-reconcile-branch"),
                            )
                            .child(
                                Button(
                                    "Toggle declared expansion",
                                    self.action(Self::toggle_declared_expansion),
                                )
                                .localized_label("lab-tree-toggle-declared")
                                .id("tree-reconcile-declared-expansion"),
                            ),
                    ),
            )
            .child(
                LocalizedText(if self.declared_root_expanded() {
                    "lab-tree-status-expanded"
                } else {
                    "lab-tree-status-collapsed"
                })
                .with_arguments(
                    MessageArguments::new()
                        .with(
                            "selection",
                            self.selection().as_deref().unwrap_or("none").to_owned(),
                        )
                        .with(
                            "last",
                            self.last_activated_leaf()
                                .as_deref()
                                .unwrap_or("none")
                                .to_owned(),
                        )
                        .with("count", self.leaf_activation_count().to_string()),
                ),
            )
            .child(LocalizedText("lab-tree-demo-help"))
            .child(
                Tree(roots, self.selection_binding())
                    .id("tree-reconciliation-tree")
                    .on_activate(self.action(Self::activate_selected_leaf))
                    .height(280.0),
            )
    }
}
