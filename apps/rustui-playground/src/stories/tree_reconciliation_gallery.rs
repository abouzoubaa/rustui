//! Stable-ID Tree updates and native expansion reconciliation examples.

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
        let branch = TreeNode::new("branch", branch_label).expanded(true).child(
            TreeNode::new("child-folder", "Nested child folder")
                .expanded(true)
                .child(TreeNode::new("grandchild", "Grandchild leaf")),
        );
        let sibling = TreeNode::new("sibling", "Stable sibling");

        if self.branch_present() {
            workspace_children.push(branch);
        }
        workspace_children.push(sibling);
        if self.reversed() {
            workspace_children.reverse();
        }

        let workspace = TreeNode::new("workspace", workspace_label)
            .expanded(self.declared_root_expanded())
            .children(workspace_children);
        let archive = TreeNode::new("archive", "Archive")
            .expanded(true)
            .child(TreeNode::new("archive-item", "Archived item"));
        let roots = if self.reversed() {
            vec![archive, workspace]
        } else {
            vec![workspace, archive]
        };

        VStack::new()
            .spacing(8.0)
            .child(Text("Tree · stable IDs across owner updates"))
            .child(
                VStack::new()
                    .spacing(6.0)
                    .child(
                        HStack::new()
                            .spacing(6.0)
                            .child(
                                Button("Reverse order", self.action(Self::reverse))
                                    .id("tree-reconcile-reverse"),
                            )
                            .child(
                                Button("Relabel", self.action(Self::relabel))
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
                                .id("tree-reconcile-branch"),
                            )
                            .child(
                                Button(
                                    "Toggle declared expansion",
                                    self.action(Self::toggle_declared_expansion),
                                )
                                .id("tree-reconcile-declared-expansion"),
                            ),
                    ),
            )
            .child(Text(format!(
                "Declared Workspace expansion: {} · selected stable ID: {}",
                self.declared_root_expanded(),
                self.selection().as_deref().unwrap_or("none"),
            )))
            .child(Text(
                "Expand the nested folders, then collapse Workspace and update labels or order. Removing and restoring Branch starts it at its declared default.",
            ))
            .child(
                Tree(roots, self.selection_binding())
                    .id("tree-reconciliation-tree")
                    .height(280.0),
            )
    }
}
