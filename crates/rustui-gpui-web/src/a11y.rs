//! Retained semantic projection of GPUI's authoritative AccessKit tree.
//! This cache owns no application value, layout, selection or event scheduler.
use crate::a11y_dom::{ImeHost, SemanticDom};
use gpui::{
    A11yCallbacks,
    accesskit::{self, Action, ActionData, Node, NodeId, Role, TreeId, TreeUpdate},
};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};
use wasm_bindgen::{JsCast as _, prelude::*};

struct State {
    nodes: HashMap<NodeId, Node>,
    root: Option<NodeId>,
    focus: NodeId,
    visible: HashSet<NodeId>,
    disabled: HashSet<NodeId>,
    healthy: bool,
    protected: HashSet<NodeId>,
    callbacks: Rc<A11yCallbacks>,
}
pub(crate) struct Adapter {
    state: Rc<RefCell<State>>,
    dom: SemanticDom,
    ime: ImeHost,
    _dispatch: Closure<dyn FnMut(String, String, JsValue)>,
    pending: Vec<TreeUpdate>,
}
impl Adapter {
    pub(crate) fn new(
        document: &web_sys::Document,
        body: &web_sys::HtmlElement,
        canvas: &web_sys::HtmlCanvasElement,
        ime: ImeHost,
        callbacks: A11yCallbacks,
    ) -> Result<Self, JsValue> {
        let callbacks = Rc::new(callbacks);
        let state = Rc::new(RefCell::new(State {
            nodes: HashMap::new(),
            root: None,
            focus: NodeId(0),
            visible: HashSet::new(),
            disabled: HashSet::new(),
            healthy: false,
            protected: HashSet::new(),
            callbacks: callbacks.clone(),
        }));
        let weak = Rc::downgrade(&state);
        let dispatch = Closure::new(move |id: String, name: String, data: JsValue| {
            let Some(state) = weak.upgrade() else {
                return;
            };
            let Ok(id) = id.parse::<u64>() else {
                return;
            };
            let id = NodeId(id);
            let Some(action) = action(&name) else {
                return;
            };
            let (callback, payload) = {
                let state = state.borrow();
                if !state.healthy || !state.visible.contains(&id) || state.disabled.contains(&id) {
                    return;
                }
                let Some(node) = state.nodes.get(&id) else {
                    return;
                };
                if !node.supports_action(action)
                    || (action == Action::SetValue
                        && (node.is_read_only()
                            || state.protected.contains(&id)
                            || matches!(node.role(), Role::PasswordInput)
                            || node.numeric_value().is_none()))
                {
                    return;
                }
                let payload = if action == Action::SetValue {
                    let Some(value) = data.as_f64().filter(|v| v.is_finite()) else {
                        return;
                    };
                    Some(ActionData::NumericValue(value))
                } else {
                    None
                };
                (state.callbacks.clone(), payload)
            };
            (callback.action)(accesskit::ActionRequest {
                action,
                target_tree: TreeId::ROOT,
                target_node: id,
                data: payload,
            });
        });
        let dom = match crate::a11y_dom::create_semantic_dom(
            document,
            body,
            canvas,
            &ime,
            dispatch.as_ref().unchecked_ref(),
        ) {
            Ok(dom) => dom,
            Err(error) => {
                (callbacks.deactivation)();
                return Err(error);
            }
        };
        let mut adapter = Self {
            state,
            dom,
            ime,
            _dispatch: dispatch,
            pending: Vec::new(),
        };
        if let Some(initial) = (callbacks.activation)() {
            adapter.update(initial, false);
        }
        Ok(adapter)
    }
    pub(crate) fn update(&mut self, update: TreeUpdate, _composing: bool) {
        // Painting precedes native input installation. Buffer AccessKit updates;
        // serialize values/focus only after this frame's protected owner is ready.
        self.pending.push(update);
    }
    pub(crate) fn frame_ready(&mut self, composing: bool) {
        let mut snapshot = None;
        for update in std::mem::take(&mut self.pending) {
            let result = {
                let mut state = self.state.borrow_mut();
                project(&mut state, update, self.ime.is_secure())
            };
            match result {
                Ok(next) => snapshot = Some(next),
                Err(error) => {
                    self.state.borrow_mut().healthy = false;
                    self.dom.fail_closed();
                    log::warn!("Web semantic tree rejected: {error}");
                    return;
                }
            }
        }
        if let Some(snapshot) = snapshot {
            if self.dom.update(&snapshot, composing).is_err() {
                self.state.borrow_mut().healthy = false;
                self.dom.fail_closed();
                return;
            }
        }
        if self.dom.frame_ready().is_err() {
            self.state.borrow_mut().healthy = false;
            self.dom.fail_closed();
        }
    }
    pub(crate) fn composition_started(&self) {
        self.dom.composition_started();
    }
    pub(crate) fn owner_changed(&self) {
        self.dom.semantic_owner_changed();
    }
    pub(crate) fn owns_target(&self, target: &web_sys::EventTarget) -> bool {
        self.dom.owns_target(target)
    }
    pub(crate) fn native_keyboard(&self, event: &web_sys::KeyboardEvent) -> bool {
        self.dom.native_keyboard(event)
    }
    pub(crate) fn composition_ended(&self) {
        if self.dom.composition_ended().is_err() {
            self.state.borrow_mut().healthy = false;
            self.dom.fail_closed();
        }
    }
}
impl Drop for Adapter {
    fn drop(&mut self) {
        self.state.borrow_mut().healthy = false;
        self.dom.close();
        let callbacks = self.state.borrow().callbacks.clone();
        (callbacks.deactivation)();
    }
}

fn action(name: &str) -> Option<Action> {
    Some(match name {
        "Focus" => Action::Focus,
        "Click" => Action::Click,
        "SetValue" => Action::SetValue,
        "Increment" => Action::Increment,
        "Decrement" => Action::Decrement,
        "Expand" => Action::Expand,
        "Collapse" => Action::Collapse,
        "ScrollIntoView" => Action::ScrollIntoView,
        "ShowContextMenu" => Action::ShowContextMenu,
        _ => return None,
    })
}
fn put(object: &js_sys::Object, key: &str, value: impl Into<JsValue>) {
    js_sys::Reflect::set(object, &JsValue::from_str(key), &value.into())
        .expect("fresh semantic descriptor object");
}
fn attr(object: &js_sys::Object, key: &str, value: Option<String>) {
    if let Some(value) = value {
        put(object, key, value);
    }
}
fn text_role(role: Role) -> bool {
    matches!(
        role,
        Role::TextInput
            | Role::MultilineTextInput
            | Role::SearchInput
            | Role::EmailInput
            | Role::PhoneNumberInput
            | Role::UrlInput
            | Role::PasswordInput
            | Role::EditableComboBox
            | Role::NumberInput
            | Role::DateInput
            | Role::DateTimeInput
            | Role::WeekInput
            | Role::MonthInput
            | Role::TimeInput
    )
}
fn role(role: Role) -> Option<&'static str> {
    Some(match role {
        Role::Window | Role::Application | Role::RootWebArea => "application",
        Role::Button | Role::DefaultButton => "button",
        Role::Link => "link",
        Role::CheckBox => "checkbox",
        Role::RadioButton => "radio",
        Role::Switch => "switch",
        Role::TextInput
        | Role::MultilineTextInput
        | Role::PasswordInput
        | Role::EmailInput
        | Role::PhoneNumberInput
        | Role::UrlInput => "textbox",
        Role::SearchInput => "searchbox",
        Role::NumberInput => "spinbutton",
        Role::DateInput
        | Role::DateTimeInput
        | Role::WeekInput
        | Role::MonthInput
        | Role::TimeInput => "textbox",
        Role::ComboBox | Role::EditableComboBox => "combobox",
        Role::Tree => "tree",
        Role::TreeItem => "treeitem",
        Role::TreeGrid => "treegrid",
        Role::List => "list",
        Role::ListItem => "listitem",
        Role::ListBox => "listbox",
        Role::ListBoxOption | Role::MenuListOption => "option",
        Role::Menu | Role::MenuListPopup => "menu",
        Role::MenuBar => "menubar",
        Role::MenuItem => "menuitem",
        Role::MenuItemCheckBox => "menuitemcheckbox",
        Role::MenuItemRadio => "menuitemradio",
        Role::Tab => "tab",
        Role::TabList => "tablist",
        Role::TabPanel => "tabpanel",
        Role::Table => "table",
        Role::Grid => "grid",
        Role::Row => "row",
        Role::RowGroup => "rowgroup",
        Role::Cell => "cell",
        Role::GridCell => "gridcell",
        Role::RowHeader => "rowheader",
        Role::ColumnHeader => "columnheader",
        Role::Slider => "slider",
        Role::SpinButton => "spinbutton",
        Role::ProgressIndicator => "progressbar",
        Role::Meter => "meter",
        Role::ScrollBar => "scrollbar",
        Role::Splitter => "separator",
        Role::Group | Role::Pane => "group",
        Role::Region | Role::Section => "region",
        Role::Navigation => "navigation",
        Role::Toolbar => "toolbar",
        Role::Heading => "heading",
        Role::Image => "img",
        Role::Alert => "alert",
        Role::Status => "status",
        Role::Log => "log",
        Role::Dialog => "dialog",
        Role::AlertDialog => "alertdialog",
        Role::Tooltip => "tooltip",
        Role::Form => "form",
        Role::Article => "article",
        Role::Document => "document",
        Role::Main => "main",
        Role::Banner => "banner",
        Role::ContentInfo => "contentinfo",
        Role::Complementary => "complementary",
        _ => return None,
    })
}
fn project(state: &mut State, update: TreeUpdate, secure: bool) -> Result<JsValue, &'static str> {
    if update.tree_id != TreeId::ROOT {
        return Err("embedded AccessKit trees are not supported");
    }
    let root = update
        .tree
        .as_ref()
        .map(|tree| tree.root)
        .or(state.root)
        .ok_or("missing initial root")?;
    let mut nodes = state.nodes.clone();
    let mut changed = HashSet::new();
    for (id, node) in update.nodes {
        if !changed.insert(id) {
            return Err("duplicate update ID");
        }
        nodes.insert(id, node);
    }
    let mut visited = HashSet::new();
    let mut visible = HashSet::new();
    let mut disabled = HashSet::new();
    let mut protected_ids = HashSet::new();
    let mut order = Vec::new();
    let mut stack = vec![(root, false, false, false, accesskit::Affine::IDENTITY)];
    let output = js_sys::Array::new();
    while let Some((id, hidden, parent_disabled, parent_protected, transform)) = stack.pop() {
        if !visited.insert(id) {
            return Err("cyclic or multiply parented node");
        }
        let node = nodes.get(&id).ok_or("missing child node")?;
        let transform = node
            .transform()
            .map_or(transform, |local| transform * (*local));
        let hidden = hidden || node.is_hidden();
        let is_disabled = parent_disabled || node.is_disabled();
        let protected = parent_protected
            || node.role() == Role::PasswordInput
            || (id == update.focus && secure);
        if protected {
            protected_ids.insert(id);
        }
        for child in node.children().iter().rev() {
            stack.push((*child, hidden, is_disabled, protected, transform));
        }
        if hidden || parent_protected {
            continue;
        }
        visible.insert(id);
        if is_disabled {
            disabled.insert(id);
        }
        order.push((id, transform, is_disabled));
    }
    if changed.iter().any(|id| !visited.contains(id)) {
        return Err("orphan update ID");
    }
    for (id, transform, is_disabled) in order {
        let node = nodes.get(&id).ok_or("missing projected node")?;
        let attrs = js_sys::Object::new();
        attr(&attrs, "role", role(node.role()).map(str::to_owned));
        attr(
            &attrs,
            "aria-label",
            node.label()
                .map(str::to_owned)
                .or_else(|| relation_text(node.labelled_by(), &nodes, &protected_ids)),
        );
        attr(
            &attrs,
            "aria-description",
            node.description()
                .map(str::to_owned)
                .or_else(|| relation_text(node.described_by(), &nodes, &protected_ids)),
        );
        if is_disabled {
            put(&attrs, "aria-disabled", "true");
        }
        if node.is_read_only() {
            put(&attrs, "aria-readonly", "true");
        }
        if node.is_required() {
            put(&attrs, "aria-required", "true");
        }
        if node.is_busy() {
            put(&attrs, "aria-busy", "true");
        }
        if node.is_modal() {
            put(&attrs, "aria-modal", "true");
        }
        if node.is_multiselectable() {
            put(&attrs, "aria-multiselectable", "true");
        }
        attr(
            &attrs,
            "aria-selected",
            node.is_selected().map(|v| v.to_string()),
        );
        attr(
            &attrs,
            "aria-expanded",
            node.is_expanded().map(|v| v.to_string()),
        );
        attr(
            &attrs,
            "aria-checked",
            node.toggled().map(|value| {
                match value {
                    accesskit::Toggled::True => "true",
                    accesskit::Toggled::False => "false",
                    accesskit::Toggled::Mixed => "mixed",
                }
                .into()
            }),
        );
        attr(
            &attrs,
            "aria-live",
            node.live().map(|value| {
                match value {
                    accesskit::Live::Off => "off",
                    accesskit::Live::Polite => "polite",
                    accesskit::Live::Assertive => "assertive",
                }
                .into()
            }),
        );
        if node.is_live_atomic() {
            put(&attrs, "aria-atomic", "true");
        }
        attr(
            &attrs,
            "aria-orientation",
            node.orientation().map(|value| {
                match value {
                    accesskit::Orientation::Horizontal => "horizontal",
                    accesskit::Orientation::Vertical => "vertical",
                }
                .into()
            }),
        );
        for (name, value) in [
            ("aria-level", node.level()),
            ("aria-posinset", node.position_in_set()),
            ("aria-setsize", node.size_of_set()),
            ("aria-rowcount", node.row_count()),
            ("aria-colcount", node.column_count()),
            ("aria-rowindex", node.row_index()),
            ("aria-colindex", node.column_index()),
            ("aria-rowspan", node.row_span()),
            ("aria-colspan", node.column_span()),
        ] {
            attr(&attrs, name, value.map(|v| v.to_string()));
        }
        for (name, ids) in [
            ("aria-labelledby", node.labelled_by()),
            ("aria-describedby", node.described_by()),
            ("aria-controls", node.controls()),
        ] {
            if !ids.is_empty() && ids.iter().all(|id| visible.contains(id)) {
                put(
                    &attrs,
                    name,
                    ids.iter()
                        .map(|id| format!("gpui-ax-{}", id.0))
                        .collect::<Vec<_>>()
                        .join(" "),
                );
            }
        }
        if text_role(node.role()) {
            put(
                &attrs,
                "aria-multiline",
                if node.role() == Role::MultilineTextInput {
                    "true"
                } else {
                    "false"
                },
            );
        }
        attr(
            &attrs,
            "aria-invalid",
            node.invalid().map(|invalid| {
                match invalid {
                    accesskit::Invalid::True => "true",
                    accesskit::Invalid::Grammar => "grammar",
                    accesskit::Invalid::Spelling => "spelling",
                }
                .into()
            }),
        );
        attr(
            &attrs,
            "aria-activedescendant",
            node.active_descendant()
                .filter(|id| visible.contains(id))
                .map(|id| format!("gpui-ax-{}", id.0)),
        );
        let protected = protected_ids.contains(&id);
        let numeric = if protected {
            None
        } else {
            node.numeric_value().filter(|v| v.is_finite())
        };
        let minimum = if protected {
            None
        } else {
            node.min_numeric_value().filter(|v| v.is_finite())
        };
        let maximum = if protected {
            None
        } else {
            node.max_numeric_value().filter(|v| v.is_finite())
        };
        for (name, value) in [
            ("aria-valuenow", numeric),
            ("aria-valuemin", minimum),
            ("aria-valuemax", maximum),
        ] {
            attr(&attrs, name, value.map(|v| v.to_string()));
        }
        if !protected && !text_role(node.role()) {
            attr(&attrs, "aria-valuetext", node.value().map(str::to_owned));
        }
        let kind = if protected {
            "protected"
        } else if text_role(node.role()) {
            "text"
        } else if node.role() == Role::SpinButton {
            "spin"
        } else if matches!(
            node.role(),
            Role::CheckBox | Role::Switch | Role::MenuItemCheckBox
        ) {
            "checkbox"
        } else if matches!(node.role(), Role::RadioButton | Role::MenuItemRadio) {
            "radio"
        } else if node.role() == Role::Slider
            && numeric.is_some()
            && minimum.zip(maximum).is_some_and(|(min, max)| min < max)
        {
            "range"
        } else if matches!(
            node.role(),
            Role::Button
                | Role::DefaultButton
                | Role::Link
                | Role::MenuItem
                | Role::Tab
                | Role::DisclosureTriangle
                | Role::ComboBox
        ) {
            "button"
        } else {
            "generic"
        };
        let descriptor = js_sys::Object::new();
        put(&descriptor, "id", id.0.to_string());
        put(&descriptor, "kind", kind);
        put(&descriptor, "attrs", attrs);
        put(&descriptor, "disabled", is_disabled);
        put(&descriptor, "readOnly", node.is_read_only());
        put(
            &descriptor,
            "focusable",
            node.supports_action(Action::Focus),
        );
        let actions = js_sys::Array::new();
        for name in [
            "Focus",
            "Click",
            "SetValue",
            "Increment",
            "Decrement",
            "Expand",
            "Collapse",
            "ScrollIntoView",
            "ShowContextMenu",
        ] {
            if node.supports_action(action(name).unwrap()) {
                actions.push(&JsValue::from_str(name));
            }
        }
        put(&descriptor, "actions", actions);
        let children = js_sys::Array::new();
        for child in node.children() {
            if visible.contains(child) {
                children.push(&JsValue::from_str(&child.0.to_string()));
            }
        }
        put(&descriptor, "children", children);
        // Protected values are never materialized in semantic descriptors. A
        // focused text field's only DOM value is the existing native IME host.
        let text = if protected || (text_role(node.role()) && id == update.focus) {
            ""
        } else if text_role(node.role())
            || matches!(
                node.role(),
                Role::Label | Role::TextRun | Role::Paragraph | Role::Heading
            )
        {
            node.value().unwrap_or("")
        } else {
            ""
        };
        put(&descriptor, "text", text);
        put(
            &descriptor,
            "checked",
            node.toggled()
                .map(|v| match v {
                    accesskit::Toggled::True => "true",
                    accesskit::Toggled::False => "false",
                    accesskit::Toggled::Mixed => "mixed",
                })
                .unwrap_or("false"),
        );
        if let Some(value) = numeric {
            put(&descriptor, "numeric", value);
        }
        if let Some(value) = minimum {
            put(&descriptor, "min", value);
        }
        if let Some(value) = maximum {
            put(&descriptor, "max", value);
        }
        put(
            &descriptor,
            "step",
            node.numeric_value_step()
                .filter(|v| v.is_finite() && *v > 0.0)
                .map(|v| v.to_string())
                .unwrap_or_else(|| "any".into()),
        );
        if let Some(bounds) = node.bounds() {
            let bounds = transform.transform_rect_bbox(bounds);
            let values = [
                bounds.x0,
                bounds.y0,
                bounds.x1 - bounds.x0,
                bounds.y1 - bounds.y0,
            ];
            if !values.iter().all(|value| value.is_finite()) || values[2] < 0.0 || values[3] < 0.0 {
                return Err("nonfinite semantic bounds");
            }
            let array = js_sys::Array::new();
            for value in values {
                array.push(&JsValue::from_f64(value));
            }
            put(&descriptor, "bounds", array);
        }
        output.push(&descriptor);
    }
    if changed.iter().any(|id| !visited.contains(id)) {
        return Err("orphan update ID");
    }
    nodes.retain(|id, _| visited.contains(id));
    state.nodes = nodes;
    state.root = Some(root);
    state.focus = update.focus;
    state.visible = visible;
    state.disabled = disabled;
    state.protected = protected_ids;
    state.healthy = true;
    let snapshot = js_sys::Object::new();
    put(&snapshot, "root", root.0.to_string());
    put(
        &snapshot,
        "focus",
        if state.visible.contains(&update.focus) {
            update.focus.0.to_string()
        } else {
            root.0.to_string()
        },
    );
    put(&snapshot, "nodes", output);
    Ok(snapshot.into())
}

fn relation_text(
    ids: &[NodeId],
    nodes: &HashMap<NodeId, Node>,
    protected: &HashSet<NodeId>,
) -> Option<String> {
    let mut result = Vec::new();
    let mut seen = HashSet::new();
    let mut stack = ids.iter().rev().copied().collect::<Vec<_>>();
    while let Some(id) = stack.pop() {
        if !seen.insert(id) || protected.contains(&id) {
            continue;
        }
        let Some(node) = nodes.get(&id) else {
            continue;
        };
        if let Some(text) = node
            .label()
            .or(node.value())
            .filter(|text| !text.is_empty())
        {
            result.push(text.to_string());
        } else {
            stack.extend(node.children().iter().rev().copied());
        }
    }
    if result.is_empty() {
        None
    } else {
        Some(result.join(" "))
    }
}
