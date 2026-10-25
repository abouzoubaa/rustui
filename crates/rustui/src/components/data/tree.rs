//! Controlled virtualized hierarchy backed by retained GPUI TreeState.

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use gpui::{
    AnyElement, App, AppContext as _, ElementId, InteractiveElement as _, IntoElement,
    ParentElement as _, SharedString, Styled as _, div, px,
};
use gpui_base::actions::{Confirm, SelectDown, SelectLeft, SelectRight, SelectUp};
use gpui_component::{
    Icon, IconName, Sizable as _,
    list::ListItem,
    tree::{Tree as GpuiTree, TreeEntry, TreeItem, TreeState},
};
use rustui_core::{Binding, Identity};

use crate::{
    components::theme::{required_style, selector, styled_container},
    view::{ControlAction, EmptyView, RenderContext, View},
};

#[derive(Clone)]
enum TreeAccessibilityLabel {
    Literal(SharedString),
    Message {
        id: SharedString,
        fallback: SharedString,
    },
}

fn resolved_tree_accessibility_label(
    label: &TreeAccessibilityLabel,
    localized: &str,
) -> SharedString {
    match label {
        TreeAccessibilityLabel::Literal(label) => label.clone(),
        TreeAccessibilityLabel::Message { id, fallback }
            if localized.trim().is_empty() || localized.trim() == id.as_ref() =>
        {
            fallback.clone()
        }
        TreeAccessibilityLabel::Message { .. } => localized.into(),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SelectionRequest {
    expected_owner: Option<String>,
    selection: Option<String>,
}

fn apply_selection_request(current: &mut Option<String>, request: &SelectionRequest) -> bool {
    if *current != request.expected_owner || *current == request.selection {
        return false;
    }
    *current = request.selection.clone();
    true
}

fn apply_pending_selection_request(
    current: &mut Option<String>,
    pending: &RefCell<Option<SelectionRequest>>,
) -> bool {
    pending
        .borrow_mut()
        .take()
        .is_some_and(|request| apply_selection_request(current, &request))
}

type SelectionUpdater<T> = Rc<dyn Fn(&mut T)>;

fn replace_selection_updater<T>(
    slot: &RefCell<Option<SelectionUpdater<T>>>,
    updater: SelectionUpdater<T>,
) {
    *slot.borrow_mut() = Some(updater);
}

fn apply_selection_updater<T>(slot: &RefCell<Option<SelectionUpdater<T>>>, context: &mut T) {
    let updater = slot.borrow().clone();
    if let Some(updater) = updater {
        updater(context);
    }
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
        let id = id.into();
        assert!(
            !id.trim().is_empty(),
            "RustUI Tree node ID must be nonempty"
        );
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "RustUI Tree node label must be nonempty"
        );
        Self {
            id,
            label,
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

fn selected_enabled_leaf(entry: Option<&TreeEntry>, expected_id: Option<&SharedString>) -> bool {
    entry.is_some_and(|entry| {
        !entry.is_folder()
            && !entry.is_disabled()
            && expected_id.is_none_or(|expected| &entry.item().id == expected)
    })
}

fn selected_disabled_folder(entry: Option<&TreeEntry>) -> bool {
    entry.is_some_and(|entry| entry.is_folder() && entry.is_disabled())
}

fn next_enabled_index(
    enabled: &[bool],
    selected: Option<usize>,
    direction: isize,
) -> Option<usize> {
    if enabled.is_empty() {
        return None;
    }
    let Some(selected) = selected else {
        return if direction > 0 {
            (0..enabled.len()).find(|index| enabled[*index])
        } else {
            (0..enabled.len()).rev().find(|index| enabled[*index])
        };
    };
    let current = selected.min(enabled.len() - 1) as isize;
    (1..=enabled.len())
        .map(|offset| {
            (current + direction * offset as isize).rem_euclid(enabled.len() as isize) as usize
        })
        .find(|index| enabled[*index])
}

fn next_enabled_tree_index(state: &TreeState, direction: isize) -> Option<usize> {
    let enabled = (0..)
        .map_while(|index| state.entry(index).map(|entry| !entry.is_disabled()))
        .collect::<Vec<_>>();
    next_enabled_index(&enabled, state.selected_index(), direction)
}

fn should_consume_tree_arrow() -> bool {
    // GPUI's fallback handler moves by raw index and can select disabled rows.
    true
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ExpansionRequest {
    expected_owner: HashSet<String>,
    expanded: HashSet<String>,
}

fn apply_expansion_request(current: &mut HashSet<String>, request: &ExpansionRequest) -> bool {
    if *current != request.expected_owner || *current == request.expanded {
        return false;
    }
    *current = request.expanded.clone();
    true
}

fn apply_pending_expansion_request(
    current: &mut HashSet<String>,
    pending: &RefCell<Option<ExpansionRequest>>,
) -> bool {
    pending
        .borrow_mut()
        .take()
        .is_some_and(|request| apply_expansion_request(current, &request))
}

fn valid_expanded_branches(nodes: &[TreeNode], requested: &HashSet<String>) -> HashSet<String> {
    fn collect(nodes: &[TreeNode], requested: &HashSet<String>, valid: &mut HashSet<String>) {
        for node in nodes {
            if !node.children.is_empty() && requested.contains(&node.id) {
                valid.insert(node.id.clone());
            }
            collect(&node.children, requested, valid);
        }
    }
    let mut valid = HashSet::new();
    collect(nodes, requested, &mut valid);
    valid
}

fn controlled_expansion(nodes: &[TreeNode], expanded: &HashSet<String>) -> HashMap<String, bool> {
    declared_expansion(nodes)
        .into_keys()
        .map(|id| {
            let value = expanded.contains(&id);
            (id, value)
        })
        .collect()
}

fn current_expanded_branches(state: &TreeState) -> HashSet<String> {
    fn collect(item: &TreeItem, expanded: &mut HashSet<String>) {
        if item.is_folder() && item.is_expanded() {
            expanded.insert(item.id.to_string());
        }
        for child in &item.children {
            collect(child, expanded);
        }
    }
    let mut expanded = HashSet::new();
    let mut index = 0;
    while let Some(entry) = state.entry(index) {
        if entry.is_root() {
            collect(entry.item(), &mut expanded);
        }
        index += 1;
    }
    expanded
}

fn visible_controlled_selection(
    nodes: &[TreeNode],
    selection: Option<&str>,
    expanded: &HashSet<String>,
) -> Option<String> {
    for node in nodes {
        if selection == Some(node.id.as_str()) {
            return Some(node.id.clone());
        }
        if expanded.contains(&node.id) {
            if let Some(selected) =
                visible_controlled_selection(&node.children, selection, expanded)
            {
                return Some(selected);
            }
        }
    }
    None
}

struct RetainedTree {
    tree: gpui::Entity<TreeState>,
    nodes: Vec<TreeNode>,
    observed_selection: Rc<RefCell<Option<String>>>,
    owner_selection: Rc<RefCell<Option<String>>>,
    pending_selection: Rc<RefCell<Option<SelectionRequest>>>,
    selection_update: Rc<RefCell<Option<SelectionUpdater<App>>>>,
    observed_expansion: Rc<RefCell<HashSet<String>>>,
    owner_expansion: Rc<RefCell<Option<HashSet<String>>>>,
    pending_expansion: Rc<RefCell<Option<ExpansionRequest>>>,
    expansion_update: Rc<RefCell<Option<SelectionUpdater<App>>>>,
}

/// A virtualized hierarchy with controlled stable-ID selection.
pub struct TreeView {
    nodes: Vec<TreeNode>,
    selection: Binding<Option<String>>,
    expansion: Option<Binding<HashSet<String>>>,
    on_activate: Option<ControlAction>,
    id: Option<Identity>,
    height: Option<f32>,
    accessibility_label: TreeAccessibilityLabel,
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
            expansion: None,
            on_activate: None,
            id: None,
            height: None,
            accessibility_label: TreeAccessibilityLabel::Message {
                id: "rustui-tree".into(),
                fallback: "Tree".into(),
            },
        }
    }

    /// Set a nonempty accessible name for the outer tree.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "Tree accessible name must be nonempty"
        );
        self.accessibility_label = TreeAccessibilityLabel::Literal(label);
        self
    }

    /// Resolve the outer tree name from a catalog message, falling back to
    /// the current literal name or the localized generic "Tree" name.
    pub fn localized_accessibility_label(mut self, id: impl Into<SharedString>) -> Self {
        let id = id.into();
        assert!(
            !id.trim().is_empty(),
            "Tree accessible-name message ID must be nonempty"
        );
        let fallback = match &self.accessibility_label {
            TreeAccessibilityLabel::Literal(label) => label.clone(),
            TreeAccessibilityLabel::Message { fallback, .. } => fallback.clone(),
        };
        self.accessibility_label = TreeAccessibilityLabel::Message { id, fallback };
        self
    }

    fn resolved_accessibility_label(&self, context: &RenderContext<'_>) -> SharedString {
        match &self.accessibility_label {
            TreeAccessibilityLabel::Literal(label) => label.clone(),
            TreeAccessibilityLabel::Message { id, .. } => resolved_tree_accessibility_label(
                &self.accessibility_label,
                &context.localized_message(id.as_ref()),
            ),
        }
    }

    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "RustUI Tree ID must be nonempty"
        );
        self.id = Some(id);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        assert!(height.is_finite(), "Tree height must be finite");
        self.height = Some(height.max(1.0));
        self
    }

    /// Invokes this action when an enabled leaf is selected and activated.
    /// Pointer activation runs after row selection; keyboard confirmation
    /// captures the existing Tree Confirm action only for enabled leaves.
    /// Enabled folder expansion remains owned by GPUI Tree.
    pub fn on_activate(mut self, action: impl Into<ControlAction>) -> Self {
        let action = action.into();
        assert!(
            !matches!(&action, ControlAction::Application(action) if !action.keyboard_compatible()),
            "Tree leaf activation requires a keyboard-compatible action"
        );
        self.on_activate = Some(action);
        self
    }

    pub(crate) fn controlled_expansion(mut self, expanded: Binding<HashSet<String>>) -> Self {
        self.expansion = Some(expanded);
        self
    }

    fn identity_with(&self, allocate: impl FnOnce(Identity) -> Identity) -> Identity {
        self.id.clone().unwrap_or_else(|| {
            allocate(Identity::from(format!(
                "tree:{}:{}",
                self.nodes.len(),
                self.nodes.first().map_or("", |node| node.id.as_str())
            )))
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
        let identity = self.identity_with(|base| context.next_implicit_retained_identity(base));
        let root_accessibility_label = self.resolved_accessibility_label(context);
        let nodes = self.nodes.clone();
        let initial_selection = self.selection.value();
        let requested_expansion = self.expansion.as_ref().map(Binding::value);
        let desired_expansion = requested_expansion
            .as_ref()
            .map(|requested| valid_expanded_branches(&self.nodes, requested));
        let initial_expansion = desired_expansion.clone();
        let (retained, created) = context.retained_entity(identity.clone(), move |_window, cx| {
            let tree = cx.new(|cx| {
                let items = if let Some(expanded) = initial_expansion.as_ref() {
                    let expansion = controlled_expansion(&nodes, expanded);
                    nodes
                        .iter()
                        .map(|node| node.to_gpui_with_expansion(&expansion))
                        .collect::<Vec<_>>()
                } else {
                    nodes.iter().map(TreeNode::to_gpui).collect::<Vec<_>>()
                };
                TreeState::new(cx).items(items)
            });
            RetainedTree {
                tree,
                nodes,
                observed_selection: Rc::new(RefCell::new(initial_selection.clone())),
                owner_selection: Rc::new(RefCell::new(initial_selection)),
                pending_selection: Rc::new(RefCell::new(None)),
                selection_update: Rc::new(RefCell::new(None)),
                observed_expansion: Rc::new(RefCell::new(
                    initial_expansion.clone().unwrap_or_default(),
                )),
                owner_expansion: Rc::new(RefCell::new(initial_expansion)),
                pending_expansion: Rc::new(RefCell::new(None)),
                expansion_update: Rc::new(RefCell::new(None)),
            }
        });

        let (
            tree,
            retained_nodes,
            observed_selection,
            owner_selection,
            pending_selection,
            selection_update,
            observed_expansion,
            owner_expansion,
            pending_expansion,
            expansion_update,
        ) = {
            let retained = retained.read(context.app());
            (
                retained.tree.clone(),
                retained.nodes.clone(),
                retained.observed_selection.clone(),
                retained.owner_selection.clone(),
                retained.pending_selection.clone(),
                retained.selection_update.clone(),
                retained.observed_expansion.clone(),
                retained.owner_expansion.clone(),
                retained.pending_expansion.clone(),
                retained.expansion_update.clone(),
            )
        };
        *owner_selection.borrow_mut() = self.selection.value();

        let pending = pending_selection.clone();
        let updater = context.binding_updater(self.selection.clone(), move |current| {
            apply_pending_selection_request(current, &pending)
        });
        replace_selection_updater(&selection_update, updater);
        *owner_expansion.borrow_mut() = requested_expansion.clone();
        if let Some(binding) = self.expansion.as_ref() {
            let pending = pending_expansion.clone();
            let owner_expansion = owner_expansion.clone();
            let updater = context.binding_updater(binding.clone(), move |current| {
                let changed = apply_pending_expansion_request(current, &pending);
                *owner_expansion.borrow_mut() = Some(current.clone());
                changed
            });
            replace_selection_updater(&expansion_update, updater);
        } else {
            // A retained Tree may replace an Outline at the same explicit identity.
            *expansion_update.borrow_mut() = None;
            *pending_expansion.borrow_mut() = None;
        }

        if created {
            let last_selection = observed_selection.clone();
            let event_owner_selection = owner_selection.clone();
            let pending_selection = pending_selection.clone();
            let current_update = selection_update.clone();
            let last_expansion = observed_expansion.clone();
            let event_owner_expansion = owner_expansion.clone();
            let pending_expansion = pending_expansion.clone();
            let expansion_update = expansion_update.clone();
            context.observe_retained(&identity, &tree, move |tree, cx| {
                let selected = tree
                    .read(cx)
                    .selected_item()
                    .map(|item| item.id.to_string());
                if *last_selection.borrow() != selected {
                    *last_selection.borrow_mut() = selected.clone();
                    *pending_selection.borrow_mut() = Some(SelectionRequest {
                        expected_owner: event_owner_selection.borrow().clone(),
                        selection: selected,
                    });
                    apply_selection_updater(&current_update, cx);
                }
                let expected_owner = event_owner_expansion.borrow().clone();
                if let Some(expected_owner) = expected_owner {
                    let expanded = current_expanded_branches(tree.read(cx));
                    if *last_expansion.borrow() != expanded {
                        *last_expansion.borrow_mut() = expanded.clone();
                        *pending_expansion.borrow_mut() = Some(ExpansionRequest {
                            expected_owner,
                            expanded,
                        });
                        apply_selection_updater(&expansion_update, cx);
                    }
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
        if let (Some(binding), Some(requested), Some(desired)) = (
            self.expansion.as_ref(),
            requested_expansion.as_ref(),
            desired_expansion.as_ref(),
        ) {
            if requested != desired {
                let request = ExpansionRequest {
                    expected_owner: requested.clone(),
                    expanded: desired.clone(),
                };
                let owner_expansion = owner_expansion.clone();
                let normalize = context.binding_updater(binding.clone(), move |current| {
                    let changed = apply_expansion_request(current, &request);
                    *owner_expansion.borrow_mut() = Some(current.clone());
                    changed
                });
                context.update_entity(&tree, move |_, _, cx| cx.defer(move |cx| normalize(cx)));
            }
        }
        let native_desired = desired_expansion.as_ref().map_or_else(
            || desired.clone(),
            |expanded| visible_controlled_selection(&self.nodes, desired.as_deref(), expanded),
        );
        if let Some(expanded) = desired_expansion.as_ref() {
            *observed_expansion.borrow_mut() = expanded.clone();
            if requested == desired {
                // A hidden owner selection can already match the empty native
                // selection without requiring a TreeState update this frame.
                *observed_selection.borrow_mut() = native_desired.clone();
            }
        }
        let expansion_changed = desired_expansion
            .as_ref()
            .is_some_and(|desired| current_expanded_branches(tree.read(context.app())) != *desired);
        let selected = tree
            .read(context.app())
            .selected_item()
            .map(|item| item.id.to_string());
        let data_changed = retained_nodes != self.nodes;
        let click_activation = self
            .on_activate
            .clone()
            .map(|action| Rc::new(context.bind_control_action(action)));
        let keyboard_activation = self
            .on_activate
            .clone()
            .map(|action| Rc::new(context.bind_keyboard_action(action)));

        if data_changed || expansion_changed || selected != native_desired || requested != desired {
            if requested == desired {
                *observed_selection.borrow_mut() = native_desired.clone();
            }
            if let Some(expanded) = desired_expansion.as_ref() {
                *observed_expansion.borrow_mut() = expanded.clone();
            }
            let nodes = (data_changed || expansion_changed).then(|| {
                let expansion = desired_expansion.as_ref().map_or_else(
                    || {
                        reconcile_expansion(
                            &retained_nodes,
                            &self.nodes,
                            &current_expansion(&tree.read(context.app())),
                        )
                    },
                    |expanded| controlled_expansion(&self.nodes, expanded),
                );
                self.nodes
                    .iter()
                    .map(|node| node.to_gpui_with_expansion(&expansion))
                    .collect::<Vec<_>>()
            });
            let selected_item = native_desired
                .as_ref()
                .map(|id| TreeItem::new(id.clone(), ""));
            let controlled = self.expansion.is_some();
            context.update_entity(&tree, move |state, _window, cx| {
                if let Some(nodes) = nodes {
                    state.set_items(nodes, cx);
                }
                if controlled {
                    // Upstream set_selected_item reveals hidden ancestors. Controlled Outline
                    // keeps owner expansion authoritative and retains hidden selection only in its binding.
                    let index = selected_item
                        .as_ref()
                        .and_then(|item| state.index_of(&item.id));
                    state.set_selected_index(index, cx);
                } else {
                    state.set_selected_item(selected_item.as_ref(), cx);
                }
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
        let mut tree_container = div().size_full();
        // Preserve disabled-folder keyboard navigation without forwarding expansion actions.
        let keyboard_tree = tree.clone();
        tree_container = tree_container.capture_action(move |_: &Confirm, window, cx| {
            let (disabled_folder, selected_leaf) = {
                let state = keyboard_tree.read(cx);
                let entry = state.selected_entry();
                (
                    selected_disabled_folder(entry),
                    selected_enabled_leaf(entry, None),
                )
            };
            if disabled_folder {
                cx.stop_propagation();
            } else if selected_leaf {
                if let Some(activation) = keyboard_activation.as_ref() {
                    activation(window, cx);
                    cx.stop_propagation();
                }
            }
        });
        let keyboard_tree = tree.clone();
        tree_container = tree_container.capture_action(move |_: &SelectLeft, _, cx| {
            if selected_disabled_folder(keyboard_tree.read(cx).selected_entry()) {
                cx.stop_propagation();
            }
        });
        let keyboard_tree = tree.clone();
        tree_container = tree_container.capture_action(move |_: &SelectRight, _, cx| {
            if selected_disabled_folder(keyboard_tree.read(cx).selected_entry()) {
                cx.stop_propagation();
            }
        });
        let keyboard_tree = tree.clone();
        tree_container = tree_container.capture_action(move |_: &SelectUp, _, cx| {
            let next = next_enabled_tree_index(keyboard_tree.read(cx), -1);
            if let Some(index) = next {
                keyboard_tree.update(cx, |state, cx| {
                    state.set_selected_index(Some(index), cx);
                    state.scroll_to_item(index, gpui::ScrollStrategy::Top);
                });
            }
            if should_consume_tree_arrow() {
                cx.stop_propagation();
            }
        });
        let keyboard_tree = tree.clone();
        tree_container = tree_container.capture_action(move |_: &SelectDown, _, cx| {
            let next = next_enabled_tree_index(keyboard_tree.read(cx), 1);
            if let Some(index) = next {
                keyboard_tree.update(cx, |state, cx| {
                    state.set_selected_index(Some(index), cx);
                    state.scroll_to_item(index, gpui::ScrollStrategy::Bottom);
                });
            }
            if should_consume_tree_arrow() {
                cx.stop_propagation();
            }
        });
        let click_tree = tree.clone();
        container
            .child(
                tree_container.child(
                    GpuiTree::new(&tree, move |_ix, entry, _selected, _window, _cx| {
                        let marker = if entry.is_folder() {
                            Icon::new(if entry.is_expanded() {
                                IconName::ChevronDown
                            } else {
                                IconName::ChevronRight
                            })
                            .with_size(px(12.0))
                            .into_any_element()
                        } else {
                            div().w(px(12.0)).flex_none().into_any_element()
                        };
                        let mut row = ListItem::new(ElementId::Name(
                            format!("rustui-tree-row-{}", entry.item().id).into(),
                        ))
                        .pl(px(8.0 + entry.depth() as f32 * 16.0))
                        .accessibility_label(entry.item().label.clone())
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1()
                                .child(marker)
                                .child(entry.item().label.clone()),
                        );
                        if let Some(activation) = click_activation
                            .clone()
                            .filter(|_| !entry.is_folder() && !entry.is_disabled())
                        {
                            let click_tree = click_tree.clone();
                            let node_id = entry.item().id.clone();
                            row = row.on_click(move |event, window, cx| {
                                let selected_leaf = {
                                    let state = click_tree.read(cx);
                                    selected_enabled_leaf(state.selected_entry(), Some(&node_id))
                                };
                                if selected_leaf {
                                    activation(event, window, cx);
                                }
                            });
                        }
                        row
                    })
                    .accessibility_label(root_accessibility_label),
                ),
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tree_accessibility_names_preserve_localized_fallbacks() {
        struct Owner;
        let selection = Binding::new(None, |_: &Owner| None, |_: &mut Owner, _| {});
        let default = TreeView::new(
            Vec::<TreeNode>::new(),
            Binding::new(None, |_: &Owner| None, |_: &mut Owner, _| {}),
        );
        assert!(matches!(
            default.accessibility_label,
            TreeAccessibilityLabel::Message { ref id, ref fallback }
                if id.as_ref() == "rustui-tree" && fallback.as_ref() == "Tree"
        ));
        let tree = TreeView::new(Vec::<TreeNode>::new(), selection)
            .accessibility_label("Project files")
            .localized_accessibility_label("tree.project-files");
        assert!(matches!(
            tree.accessibility_label,
            TreeAccessibilityLabel::Message { ref id, ref fallback }
                if id.as_ref() == "tree.project-files" && fallback.as_ref() == "Project files"
        ));
        assert_eq!(
            resolved_tree_accessibility_label(&tree.accessibility_label, "Fichiers du projet"),
            "Fichiers du projet"
        );
        assert_eq!(
            resolved_tree_accessibility_label(&tree.accessibility_label, " \ttree.project-files"),
            "Project files"
        );
        assert_eq!(
            resolved_tree_accessibility_label(&tree.accessibility_label, "tree.project-files"),
            "Project files"
        );
        let default = TreeAccessibilityLabel::Message {
            id: "rustui-tree".into(),
            fallback: "Tree".into(),
        };
        assert_eq!(
            resolved_tree_accessibility_label(&default, "Arbre"),
            "Arbre"
        );
        assert_eq!(resolved_tree_accessibility_label(&default, ""), "Tree");
    }

    #[test]
    #[should_panic(expected = "Tree accessible name must be nonempty")]
    fn rejects_blank_tree_accessible_name() {
        struct Owner;
        let selection = Binding::new(None, |_: &Owner| None, |_: &mut Owner, _| {});
        let _ = TreeView::new(Vec::<TreeNode>::new(), selection).accessibility_label(" \t");
    }

    #[test]
    #[should_panic(expected = "Tree accessible-name message ID must be nonempty")]
    fn rejects_blank_tree_accessible_name_message_id() {
        struct Owner;
        let selection = Binding::new(None, |_: &Owner| None, |_: &mut Owner, _| {});
        let _ =
            TreeView::new(Vec::<TreeNode>::new(), selection).localized_accessibility_label(" \t");
    }

    #[test]
    fn implicit_tree_identity_distinguishes_siblings_and_is_stable_by_slot() {
        struct Owner;
        let selection = Binding::new(None, |_: &Owner| None, |_: &mut Owner, _| {});
        let tree = TreeView::new([TreeNode::new("root", "Root")], selection);
        let allocate_at_slot = |slot| {
            move |base: Identity| Identity::from(format!("owner:implicit-retained:{slot}:{base}"))
        };

        let first = tree.identity_with(allocate_at_slot(0));
        let second = tree.identity_with(allocate_at_slot(1));
        let first_after_rerender = tree.identity_with(allocate_at_slot(0));

        assert_ne!(first, second);
        assert_eq!(first, first_after_rerender);
    }

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
    #[should_panic(expected = "RustUI Tree node label must be nonempty")]
    fn blank_tree_node_accessible_names_are_rejected() {
        let _ = TreeNode::new("item", " \t ");
    }

    #[test]
    #[should_panic(expected = "RustUI Tree node ID must be nonempty")]
    fn blank_tree_node_ids_are_rejected() {
        let _ = TreeNode::new(" \t ", "Item");
    }

    #[test]
    #[should_panic(expected = "RustUI Tree ID must be nonempty")]
    fn blank_tree_component_identity_is_rejected() {
        struct Owner;
        fn read(_: &Owner) -> Option<String> {
            None
        }
        fn write(_: &mut Owner, _: Option<String>) {}

        let _ = TreeView::new([], Binding::new(None, read, write)).id(" \t ");
    }

    #[test]
    fn rejects_non_finite_viewport_height() {
        struct Owner;
        fn read(_: &Owner) -> Option<String> {
            None
        }
        fn write(_: &mut Owner, _: Option<String>) {}

        for height in [f32::INFINITY, f32::NEG_INFINITY, f32::NAN] {
            assert!(
                std::panic::catch_unwind(|| {
                    TreeView::new([], Binding::new(None, read, write)).height(height)
                })
                .is_err()
            );
        }
    }

    #[test]
    fn leaf_activation_requires_the_selected_enabled_leaf() {
        let leaf = TreeEntry::new(TreeItem::new("leaf", "Leaf"), 1);
        let disabled_leaf = TreeEntry::new(TreeItem::new("disabled", "Disabled").disabled(true), 1);
        let folder = TreeEntry::new(
            TreeItem::new("folder", "Folder").child(TreeItem::new("child", "Child")),
            0,
        );
        let other_id = SharedString::from("other");

        assert!(selected_enabled_leaf(Some(&leaf), None));
        assert!(selected_enabled_leaf(Some(&leaf), Some(&leaf.item().id)));
        assert!(!selected_enabled_leaf(Some(&leaf), Some(&other_id)));
        assert!(!selected_enabled_leaf(None, None));
        assert!(!selected_enabled_leaf(Some(&disabled_leaf), None));
        assert!(!selected_enabled_leaf(Some(&folder), None));
    }

    #[test]
    fn disabled_folder_expansion_guard_only_matches_disabled_folders() {
        let disabled_collapsed = TreeEntry::new(
            TreeItem::new("disabled-collapsed", "Disabled")
                .child(TreeItem::new("child-a", "Child"))
                .disabled(true),
            0,
        );
        let disabled_expanded = TreeEntry::new(
            TreeItem::new("disabled-expanded", "Disabled")
                .expanded(true)
                .child(TreeItem::new("child-b", "Child"))
                .disabled(true),
            0,
        );
        let enabled_folder = TreeEntry::new(
            TreeItem::new("enabled", "Enabled").child(TreeItem::new("child-c", "Child")),
            0,
        );
        let disabled_leaf = TreeEntry::new(TreeItem::new("leaf", "Leaf").disabled(true), 1);

        assert!(selected_disabled_folder(Some(&disabled_collapsed)));
        assert!(selected_disabled_folder(Some(&disabled_expanded)));
        assert!(!selected_disabled_folder(Some(&enabled_folder)));
        assert!(!selected_disabled_folder(Some(&disabled_leaf)));
        assert!(!selected_disabled_folder(None));
    }

    #[test]
    fn tree_arrow_navigation_skips_disabled_entries_and_wraps() {
        let enabled = [true, false, true];

        assert_eq!(next_enabled_index(&enabled, None, 1), Some(0));
        assert_eq!(next_enabled_index(&enabled, None, -1), Some(2));
        assert_eq!(next_enabled_index(&enabled, Some(0), 1), Some(2));
        assert_eq!(next_enabled_index(&enabled, Some(2), -1), Some(0));
        assert_eq!(next_enabled_index(&enabled, Some(1), -1), Some(0));
        assert_eq!(next_enabled_index(&enabled, Some(1), 1), Some(2));
        assert_eq!(next_enabled_index(&[false, false], Some(0), 1), None);
        assert_eq!(next_enabled_index(&[false, false], None, 1), None);
    }

    #[test]
    fn tree_arrow_key_is_consumed_even_when_no_enabled_entry_is_available() {
        assert!(should_consume_tree_arrow());
    }

    #[test]
    #[should_panic(expected = "Tree leaf activation requires a keyboard-compatible action")]
    fn leaf_activation_rejects_click_only_actions() {
        use crate::runtime::AppAction;

        struct Owner;
        fn read(_: &Owner) -> Option<String> {
            None
        }
        fn write(_: &mut Owner, _: Option<String>) {}

        let click_only = AppAction::with_window_action(|_, _, _| {});
        let _ = TreeView::new([], Binding::new(None, read, write)).on_activate(click_only);
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
        let request = SelectionRequest {
            expected_owner: Some(String::from("first")),
            selection: Some(String::from("stale")),
        };
        let mut owner = Some(String::from("newer"));

        assert!(!apply_selection_request(&mut owner, &request));
        assert_eq!(owner.as_deref(), Some("newer"));
    }

    #[test]
    fn sequential_selection_requests_follow_the_current_owner_value() {
        let mut owner = Some(String::from("first"));

        assert!(apply_selection_request(
            &mut owner,
            &SelectionRequest {
                expected_owner: Some(String::from("first")),
                selection: Some(String::from("second")),
            },
        ));
        assert!(apply_selection_request(
            &mut owner,
            &SelectionRequest {
                expected_owner: Some(String::from("second")),
                selection: None,
            },
        ));
        assert_eq!(owner, None);
    }

    #[test]
    fn queued_selection_is_rejected_after_owner_rerender() {
        let pending = RefCell::new(Some(SelectionRequest {
            expected_owner: Some(String::from("first")),
            selection: Some(String::from("stale")),
        }));
        let mut owner = Some(String::from("newer"));

        assert!(!apply_pending_selection_request(&mut owner, &pending));
        assert_eq!(owner.as_deref(), Some("newer"));
        assert!(pending.borrow().is_none());
    }

    #[test]
    fn retained_selection_observer_uses_latest_owner_binding() {
        let updater = RefCell::new(None);
        let first_owner = Rc::new(RefCell::new(0));
        let second_owner = Rc::new(RefCell::new(0));
        let first = first_owner.clone();
        replace_selection_updater(
            &updater,
            Rc::new(move |value: &mut usize| {
                *first.borrow_mut() = *value;
            }),
        );
        let second = second_owner.clone();
        replace_selection_updater(
            &updater,
            Rc::new(move |value: &mut usize| {
                *second.borrow_mut() = *value;
            }),
        );

        let mut value = 7;
        apply_selection_updater(&updater, &mut value);

        assert_eq!(*first_owner.borrow(), 0);
        assert_eq!(*second_owner.borrow(), 7);
    }
}

#[cfg(test)]
#[path = "outline_tests.rs"]
mod outline_tests;

#[cfg(test)]
mod rendered_accessibility_tests {
    use super::*;
    use crate::runtime::ComponentHost;

    struct TreeSelectionOwner;
    struct RenderedTreeProbe;

    impl View for RenderedTreeProbe {
        fn body(&self) -> impl View {
            EmptyView
        }

        fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
            let selection = Binding::new(
                None,
                |_: &TreeSelectionOwner| None,
                |_: &mut TreeSelectionOwner, _| {},
            );
            let nodes = vec![
                TreeNode::new("workspace", "Workspace")
                    .expanded(true)
                    .child(
                        TreeNode::new("branch", "Branch").expanded(true).child(
                            TreeNode::new("folder", "Folder")
                                .expanded(true)
                                .child(TreeNode::new("leaf", "Leaf")),
                        ),
                    ),
            ];

            TreeView::new(nodes, selection)
                .id("tree-accessibility-regression")
                .into_element(context)
        }
    }

    fn init(cx: &mut gpui::TestAppContext) {
        let theme = crate::theme::parse_theme(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../themes/macos27.json"
        )))
        .unwrap();
        cx.update(|app| {
            gpui_component::init(app);
            crate::theme::activate_theme(&theme, Some(crate::theme::AppearanceMode::Dark), app)
                .unwrap();
        });
    }

    #[gpui::test]
    fn rendered_tree_projects_root_name_and_one_based_row_levels(cx: &mut gpui::TestAppContext) {
        init(cx);
        let (_, window) = cx.add_window_view(|_, app| ComponentHost::new(RenderedTreeProbe, app));
        let snapshot = window
            .update(|window, app| {
                let _ = window.draw(app);
                window.debug_a11y_tree_json()
            })
            .expect("test window accessibility activation projects the rendered Tree");
        let tree: serde_json::Value = serde_json::from_str(&snapshot).unwrap();
        let nodes = tree["nodes"]
            .as_object()
            .expect("debug accessibility nodes are keyed by ephemeral ID")
            .values()
            .collect::<Vec<_>>();

        let root = nodes
            .iter()
            .find(|node| node["aria"]["role"] == "Tree")
            .expect("rendered RustUI Tree root");
        assert_eq!(root["aria"]["label"], "Tree");

        let mut rows = nodes
            .iter()
            .filter(|node| node["aria"]["role"] == "TreeItem")
            .map(|node| {
                (
                    node["aria"]["label"].as_str().unwrap().to_owned(),
                    node["aria"]["level"].as_u64().unwrap(),
                )
            })
            .collect::<Vec<_>>();
        rows.sort_by_key(|(_, level)| *level);
        assert_eq!(
            rows,
            vec![
                ("Workspace".to_owned(), 1),
                ("Branch".to_owned(), 2),
                ("Folder".to_owned(), 3),
                ("Leaf".to_owned(), 4),
            ]
        );
    }
}
