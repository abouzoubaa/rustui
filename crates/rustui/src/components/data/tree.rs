//! Controlled virtualized hierarchy backed by retained GPUI TreeState.

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use gpui::{
    AnyElement, AppContext as _, ElementId, IntoElement, ParentElement as _, SharedString,
    Styled as _, div, px,
};
use gpui_component::{
    list::ListItem,
    tree::{Tree as GpuiTree, TreeItem, TreeState},
};
use rustui_core::{Binding, Identity};

use crate::{
    components::theme::{required_style, selector, styled_container},
    view::{EmptyView, RenderContext, View},
};

fn apply_selection_request(
    current: &mut Option<String>,
    expected: &RefCell<Option<String>>,
    requested: Option<String>,
) -> bool {
    if *current != *expected.borrow() || *current == requested {
        return false;
    }
    *expected.borrow_mut() = requested.clone();
    *current = requested;
    true
}

/// A stable RustUI tree node.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeNode {
    id: String,
    label: String,
    children: Vec<TreeNode>,
    expanded: bool,
    disabled: bool,
}

impl TreeNode {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            children: Vec::new(),
            expanded: false,
            disabled: false,
        }
    }

    pub fn child(mut self, child: TreeNode) -> Self {
        self.children.push(child);
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = TreeNode>) -> Self {
        self.children.extend(children);
        self
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    fn contains_id(&self, id: &str) -> bool {
        self.id == id || self.children.iter().any(|child| child.contains_id(id))
    }

    fn collect_ids<'a>(&'a self, ids: &mut HashSet<&'a str>) -> bool {
        ids.insert(&self.id) && self.children.iter().all(|child| child.collect_ids(ids))
    }

    fn to_gpui(&self) -> TreeItem {
        TreeItem::new(self.id.clone(), self.label.clone())
            .children(self.children.iter().map(Self::to_gpui))
            .expanded(self.expanded)
            .disabled(self.disabled)
    }

    fn to_gpui_with_expansion(&self, expansion: &HashMap<String, bool>) -> TreeItem {
        TreeItem::new(self.id.clone(), self.label.clone())
            .children(
                self.children
                    .iter()
                    .map(|child| child.to_gpui_with_expansion(expansion)),
            )
            .expanded(expansion.get(&self.id).copied().unwrap_or(self.expanded))
            .disabled(self.disabled)
    }
}

fn collect_item_expansion(item: &TreeItem, expansion: &mut HashMap<String, bool>) {
    expansion.insert(item.id.to_string(), item.is_expanded());
    for child in &item.children {
        collect_item_expansion(child, expansion);
    }
}

fn current_expansion(state: &TreeState) -> HashMap<String, bool> {
    let mut expansion = HashMap::new();
    let mut index = 0;
    while let Some(entry) = state.entry(index) {
        if entry.is_root() {
            collect_item_expansion(entry.item(), &mut expansion);
        }
        index += 1;
    }
    expansion
}

fn declared_expansion(nodes: &[TreeNode]) -> HashMap<String, bool> {
    fn collect(nodes: &[TreeNode], expansion: &mut HashMap<String, bool>) {
        for node in nodes {
            expansion.insert(node.id.clone(), node.expanded);
            collect(&node.children, expansion);
        }
    }

    let mut expansion = HashMap::new();
    collect(nodes, &mut expansion);
    expansion
}

fn reconcile_expansion(
    previous_nodes: &[TreeNode],
    next_nodes: &[TreeNode],
    current: &HashMap<String, bool>,
) -> HashMap<String, bool> {
    fn collect(
        nodes: &[TreeNode],
        previous: &HashMap<String, bool>,
        current: &HashMap<String, bool>,
        expansion: &mut HashMap<String, bool>,
    ) {
        for node in nodes {
            let unchanged_declaration = previous.get(&node.id) == Some(&node.expanded);
            let value = if unchanged_declaration {
                current.get(&node.id).copied().unwrap_or(node.expanded)
            } else {
                node.expanded
            };
            expansion.insert(node.id.clone(), value);
            collect(&node.children, previous, current, expansion);
        }
    }

    let previous = declared_expansion(previous_nodes);
    let mut expansion = HashMap::new();
    collect(next_nodes, &previous, current, &mut expansion);
    expansion
}

fn valid_selection(nodes: &[TreeNode], requested: Option<&str>) -> Option<String> {
    requested
        .filter(|id| nodes.iter().any(|node| node.contains_id(id)))
        .map(str::to_owned)
}

fn clear_missing_selection(current: &mut Option<String>, missing: &str) -> bool {
    if current.as_deref() != Some(missing) {
        return false;
    }
    *current = None;
    true
}

struct RetainedTree {
    tree: gpui::Entity<TreeState>,
    nodes: Vec<TreeNode>,
    observed_selection: Rc<RefCell<Option<String>>>,
    owner_selection: Rc<RefCell<Option<String>>>,
}

/// A virtualized hierarchy with controlled stable-ID selection.
pub struct TreeView {
    nodes: Vec<TreeNode>,
    selection: Binding<Option<String>>,
    id: Option<Identity>,
    height: Option<f32>,
}

impl TreeView {
    pub fn new(
        nodes: impl IntoIterator<Item = TreeNode>,
        selection: Binding<Option<String>>,
    ) -> Self {
        let nodes = nodes.into_iter().collect::<Vec<_>>();
        let mut ids = HashSet::new();
        assert!(
            nodes.iter().all(|node| node.collect_ids(&mut ids)),
            "every RustUI Tree node must have a unique ID"
        );

        Self {
            nodes,
            selection,
            id: None,
            height: None,
        }
    }

    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height.max(1.0));
        self
    }

    fn identity(&self) -> Identity {
        self.id.clone().unwrap_or_else(|| {
            Identity::from(format!(
                "tree:{}:{}",
                self.nodes.len(),
                self.nodes.first().map_or("", |node| node.id.as_str())
            ))
        })
    }
}

#[allow(non_snake_case)]
pub fn Tree(
    nodes: impl IntoIterator<Item = TreeNode>,
    selection: Binding<Option<String>>,
) -> TreeView {
    TreeView::new(nodes, selection)
}

impl View for TreeView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let style = required_style(context, "outline_view", "outline_view", &selector(context));
        let identity = self.identity();
        let nodes = self.nodes.clone();
        let initial_selection = self.selection.value();
        let (retained, created) = context.retained_entity(identity.clone(), move |_window, cx| {
            let tree = cx.new(|cx| {
                TreeState::new(cx).items(nodes.iter().map(TreeNode::to_gpui).collect::<Vec<_>>())
            });
            RetainedTree {
                tree,
                nodes,
                observed_selection: Rc::new(RefCell::new(initial_selection.clone())),
                owner_selection: Rc::new(RefCell::new(initial_selection)),
            }
        });

        let (tree, retained_nodes, observed_selection, owner_selection) = {
            let retained = retained.read(context.app());
            (
                retained.tree.clone(),
                retained.nodes.clone(),
                retained.observed_selection.clone(),
                retained.owner_selection.clone(),
            )
        };
        *owner_selection.borrow_mut() = self.selection.value();

        if created {
            let requested = observed_selection.clone();
            let expected_selection = owner_selection.clone();
            let update = context.binding_updater(self.selection.clone(), move |current| {
                let selected = requested.borrow();
                apply_selection_request(current, &expected_selection, selected.clone())
            });
            let last_selection = observed_selection.clone();
            context.observe_retained(&identity, &tree, move |tree, cx| {
                let selected = tree
                    .read(cx)
                    .selected_item()
                    .map(|item| item.id.to_string());
                if *last_selection.borrow() != selected {
                    *last_selection.borrow_mut() = selected;
                    update(cx);
                }
            });
        }

        let requested = self.selection.value();
        let desired = valid_selection(&self.nodes, requested.as_deref());
        if desired.is_none() {
            if let Some(missing) = requested.clone() {
                let normalize = context.binding_updater(self.selection.clone(), move |current| {
                    clear_missing_selection(current, &missing)
                });
                context.update_entity(&tree, move |_state, _window, cx| {
                    cx.defer(move |cx| normalize(cx));
                });
            }
        }
        let selected = tree
            .read(context.app())
            .selected_item()
            .map(|item| item.id.to_string());
        let data_changed = retained_nodes != self.nodes;

        if data_changed || selected != desired || requested != desired {
            if requested == desired {
                *observed_selection.borrow_mut() = desired.clone();
            }
            let nodes = data_changed.then(|| {
                let expansion = reconcile_expansion(
                    &retained_nodes,
                    &self.nodes,
                    &current_expansion(&tree.read(context.app())),
                );
                self.nodes
                    .iter()
                    .map(|node| node.to_gpui_with_expansion(&expansion))
                    .collect::<Vec<_>>()
            });
            let selected_item = desired.as_ref().map(|id| TreeItem::new(id.clone(), ""));
            context.update_entity(&tree, move |state, _window, cx| {
                if let Some(nodes) = nodes {
                    state.set_items(nodes, cx);
                }
                state.set_selected_item(selected_item.as_ref(), cx);
            });

            if data_changed {
                let nodes = self.nodes.clone();
                context.update_entity(&retained, move |retained, _window, _cx| {
                    retained.nodes = nodes;
                });
            }
        }

        let mut container = styled_container(div(), &style);
        if let Some(height) = self.height {
            container = container.h(px(height));
        }
        container
            .child(GpuiTree::new(
                &tree,
                |_ix, entry, _selected, _window, _cx| {
                    let marker = if entry.is_folder() {
                        if entry.is_expanded() { "▾ " } else { "▸ " }
                    } else {
                        "  "
                    };
                    let label = SharedString::from(format!("{marker}{}", entry.item().label));
                    ListItem::new(ElementId::Name(
                        format!("rustui-tree-row-{}", entry.item().id).into(),
                    ))
                    .pl(px(8.0 + entry.depth() as f32 * 16.0))
                    .accessibility_label(entry.item().label.clone())
                    .child(label)
                },
            ))
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expansion_reconciliation_preserves_unchanged_nodes_and_uses_new_declarations() {
        let previous = vec![TreeNode::new("root", "Root").expanded(true).children([
            TreeNode::new("branch", "Branch").expanded(false),
            TreeNode::new("removed", "Removed").expanded(true),
        ])];
        let next = vec![TreeNode::new("root", "Root").expanded(true).children([
            TreeNode::new("branch", "Branch").expanded(true),
            TreeNode::new("new", "New").expanded(true),
        ])];
        let current = HashMap::from([
            (String::from("root"), false),
            (String::from("branch"), false),
            (String::from("removed"), false),
            (String::from("new"), false),
        ]);

        let expansion = reconcile_expansion(&previous, &next, &current);

        assert_eq!(expansion.get("root"), Some(&false));
        assert_eq!(expansion.get("branch"), Some(&true));
        assert_eq!(expansion.get("new"), Some(&true));
        assert!(!expansion.contains_key("removed"));
    }

    #[test]
    #[should_panic(expected = "every RustUI Tree node must have a unique ID")]
    fn duplicate_ids_are_rejected_across_nested_nodes() {
        struct Owner(Option<String>);
        fn read(owner: &Owner) -> Option<String> {
            owner.0.clone()
        }
        fn write(owner: &mut Owner, selection: Option<String>) {
            owner.0 = selection;
        }

        let nodes = [TreeNode::new("same", "Root").child(TreeNode::new("same", "Child"))];

        let _ = TreeView::new(nodes, Binding::new(None, read, write));
    }

    #[test]
    fn expansion_collection_includes_descendants_hidden_by_collapsed_ancestors() {
        let root = TreeItem::new("root", "Root").expanded(false).child(
            TreeItem::new("branch", "Branch")
                .expanded(true)
                .child(TreeItem::new("leaf", "Leaf")),
        );
        let mut expansion = HashMap::new();

        collect_item_expansion(&root, &mut expansion);

        assert_eq!(expansion.get("root"), Some(&false));
        assert_eq!(expansion.get("branch"), Some(&true));
        assert_eq!(expansion.get("leaf"), Some(&false));
    }

    #[test]
    fn initial_missing_selection_is_normalized() {
        let nodes = [TreeNode::new("root", "Root")];
        let mut owner = Some(String::from("missing"));

        assert_eq!(valid_selection(&nodes, owner.as_deref()), None);
        assert!(clear_missing_selection(&mut owner, "missing"));
        assert_eq!(owner, None);
    }

    #[test]
    fn removed_selection_is_normalized() {
        let previous =
            [TreeNode::new("branch", "Branch").child(TreeNode::new("grandchild", "Grandchild"))];
        let current = [TreeNode::new("root", "Root")];
        let mut owner = Some(String::from("grandchild"));

        assert_eq!(
            valid_selection(&previous, owner.as_deref()),
            Some(String::from("grandchild"))
        );
        assert_eq!(valid_selection(&current, owner.as_deref()), None);
        assert!(clear_missing_selection(&mut owner, "grandchild"));
        assert_eq!(owner, None);
    }

    #[test]
    fn hidden_selected_id_remains_valid() {
        let nodes = [TreeNode::new("root", "Root").expanded(false).child(
            TreeNode::new("branch", "Branch")
                .expanded(true)
                .child(TreeNode::new("grandchild", "Grandchild")),
        )];

        assert_eq!(
            valid_selection(&nodes, Some("grandchild")),
            Some(String::from("grandchild"))
        );
    }

    #[test]
    fn stale_normalizer_preserves_newer_valid_selection() {
        let nodes = [TreeNode::new("still-present", "Still present")];
        let mut owner = Some(String::from("still-present"));

        assert_eq!(
            valid_selection(&nodes, owner.as_deref()),
            Some(String::from("still-present"))
        );
        assert!(!clear_missing_selection(&mut owner, "removed"));
        assert_eq!(owner.as_deref(), Some("still-present"));
    }

    #[test]
    fn selection_observer_preserves_newer_owner_selection() {
        let expected = RefCell::new(Some(String::from("first")));
        let mut owner = Some(String::from("newer"));

        assert!(!apply_selection_request(
            &mut owner,
            &expected,
            Some(String::from("stale")),
        ));
        assert_eq!(owner.as_deref(), Some("newer"));
        assert_eq!(expected.borrow().as_deref(), Some("first"));
    }

    #[test]
    fn accepted_selection_advances_expected_owner_id() {
        let expected = RefCell::new(Some(String::from("first")));
        let mut owner = Some(String::from("first"));

        assert!(apply_selection_request(
            &mut owner,
            &expected,
            Some(String::from("second")),
        ));
        assert!(apply_selection_request(&mut owner, &expected, None));
        assert_eq!(owner, None);
        assert_eq!(*expected.borrow(), None);
    }
}
