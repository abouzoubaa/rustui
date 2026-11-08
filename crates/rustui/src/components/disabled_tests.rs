//! R15: one case table and one GPUI-window runner for disabled controls.
use crate::{prelude::*, runtime::ComponentHost, view::EmptyView};
use gpui::{
    App, AppContext as _, Context, Entity, FocusHandle, InteractiveElement as _, IntoElement as _,
    ParentElement as _, Render, StatefulInteractiveElement as _, Styled as _, Window, div, px,
};
use std::{cell::Cell, rc::Rc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Check {
    Tab,
    Mouse,
    Keys,
    Accessibility,
    Recovery,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Button,
    Link,
    Checkbox,
    Radio,
    Toggle,
    Slider,
    Stepper,
    Picker,
    ComboBox,
    TextField,
    SecureField,
    SearchField,
    TextEditor,
    DatePicker,
    ColorPicker,
    TokenField,
    CodeEditor,
    GraphView,
    NodeEditor,
    PropertyText,
    PropertyChoice,
    PropertyToggle,
    PropertyInteger,
    PropertyIntegerValue,
    PropertyInspector,
    ToolbarItem,
    Toolbar,
    BreadcrumbItem,
    Disclosure,
    NavigationLink,
    SidebarItem,
    NavigationSplitItem,
    TabItem,
    TreeItem,
    MenuItem,
    ListRow,
    GridCell,
    DragSource,
    DropTarget,
    FileDropTarget,
}

fn na(kind: Kind, check: Check) -> Option<&'static str> {
    match (kind, check) {
        (Kind::ListRow, _) => Some(
            "List has no per-row disabled API; selectable(false) disables selection, not individual rows",
        ),
        (Kind::GridCell, Check::Tab | Check::Accessibility | Check::Recovery) => Some(
            "DataGridColumn::read_only deliberately keeps cells navigable; it is not a disabled-cell API",
        ),
        (
            Kind::DragSource | Kind::DropTarget | Kind::FileDropTarget,
            Check::Tab | Check::Keys | Check::Recovery,
        ) => Some(
            "drag/drop wrappers have no keyboard activation or independent focus; child is noninteractive",
        ),
        (Kind::DragSource | Kind::DropTarget, Check::Accessibility) => {
            Some("typed drag/drop wrappers expose no independent accessibility node")
        }
        (Kind::MenuItem, Check::Tab) => {
            Some("popup menu items use menu arrow navigation, not independent Tab stops")
        }
        _ => None,
    }
}

#[derive(Clone, Debug)]
struct Slot {
    checked: bool,
    text: String,
    number: f32,
    integer: i32,
    large: i64,
    choice: String,
    selected: Option<String>,
    index: Option<usize>,
    path: Vec<String>,
    tokens: Vec<String>,
    date: Option<chrono::NaiveDate>,
    color: Option<gpui::Hsla>,
    graph: GraphDocument,
    viewport: GraphViewport,
    files: Option<FileDrop>,
    dropped: Option<DropEvent<String>>,
    rows: Vec<Row>,
}
#[derive(Clone, Debug)]
struct Row {
    name: String,
}
impl Default for Slot {
    fn default() -> Self {
        Self {
            checked: false,
            text: "original".into(),
            number: 50.,
            integer: 5,
            large: 5,
            choice: "One".into(),
            selected: None,
            index: None,
            path: vec![],
            tokens: vec!["one".into()],
            date: None,
            color: None,
            files: None,
            dropped: None,
            graph: GraphDocument::new(
                [GraphNode::new(
                    "node",
                    "Target node",
                    CanvasRect::new(10., 10., 80., 40.).unwrap(),
                    [],
                )
                .unwrap()],
                [],
            )
            .unwrap(),
            viewport: GraphViewport::new(CanvasSize::new(260., 120.).unwrap(), 0., 0., 1.).unwrap(),
            rows: vec![Row {
                name: "original".into(),
            }],
        }
    }
}
struct Owner {
    kind: Kind,
    disabled: bool,
    slot: usize,
    values: [Slot; 2],
    writes: [usize; 2],
    actions: usize,
}
impl Owner {
    fn binding<T: Clone + 'static>(
        &self,
        get: fn(&Slot) -> T,
        set: fn(&mut Slot, T),
    ) -> Binding<T> {
        Binding::new_with(
            get(&self.values[self.slot]),
            (self.slot, get, set),
            |owner: &Self, (slot, get, _)| get(&owner.values[*slot]),
            |owner, (slot, _, set), value| {
                set(&mut owner.values[*slot], value);
                owner.writes[*slot] += 1;
                true
            },
        )
    }
    fn action(&self) -> rustui_core::Action {
        rustui_core::Action::new(|owner: &mut Owner| owner.actions += 1)
    }
    fn target(&self) -> AnyView {
        let b = self.binding(|s| s.checked, |s, v| s.checked = v);
        let t = self.binding(|s| s.text.clone(), |s, v| s.text = v);
        let f = self.binding(|s| s.number, |s, v| s.number = v);
        let i = self.binding(|s| s.integer, |s, v| s.integer = v);
        let choice = self.binding(|s| s.choice.clone(), |s, v| s.choice = v);
        let sel = self.binding(|s| s.selected.clone(), |s, v| s.selected = v);
        let d = self.disabled;
        macro_rules! view {
            ($v:expr) => {
                AnyView::new($v)
            };
        }
        match self.kind {
            Kind::Button => view!(Button("Target", self.action()).id("target").disabled(d)),
            Kind::Link => view!(
                Link("Target", "https://example.invalid")
                    .id("target")
                    .on_activate(self.action())
                    .open_url_on_activate(false)
                    .disabled(d)
            ),
            Kind::Checkbox => view!(Checkbox("Target", b).id("target").disabled(d)),
            Kind::Radio => view!(Radio("Target", b).id("target").disabled(d)),
            Kind::Toggle => view!(Toggle("Target", b).id("target").disabled(d)),
            Kind::Slider => view!(
                Slider(f, 0.0..=100.0)
                    .id("target")
                    .accessibility_label("Target")
                    .disabled(d)
            ),
            Kind::Stepper => view!(Stepper("Target", i).id("target").range(0..=10).disabled(d)),
            Kind::Picker => view!(
                Picker("Target", choice, ["One", "Two"])
                    .id("target")
                    .disabled(d)
            ),
            Kind::ComboBox => view!(
                ComboBox("Target", choice, ["One", "Two"])
                    .id("target")
                    .disabled(d)
            ),
            Kind::TextField => view!(TextField("Target", t).id("target").disabled(d)),
            Kind::SecureField => view!(SecureField("Target", t).id("target").disabled(d)),
            Kind::SearchField => view!(SearchField("Target", t).id("target").disabled(d)),
            Kind::TextEditor => view!(TextEditor("Target", t).id("target").height(80.).disabled(d)),
            Kind::CodeEditor => view!(
                CodeEditor("target", t)
                    .height(80.)
                    .accessibility_label("Target")
                    .disabled(d)
            ),
            Kind::DatePicker => view!(
                DatePicker("Target", self.binding(|s| s.date, |s, v| s.date = v))
                    .id("target")
                    .disabled(d)
            ),
            Kind::ColorPicker => view!(
                ColorPicker("Target", self.binding(|s| s.color, |s, v| s.color = v))
                    .id("target")
                    .disabled(d)
            ),
            Kind::TokenField => view!(
                TokenField(
                    "Target",
                    self.binding(|s| s.tokens.clone(), |s, v| s.tokens = v),
                    t
                )
                .id("target")
                .disabled(d)
            ),
            Kind::GraphView => view!(
                GraphView(
                    "target",
                    self.values[self.slot].graph.clone(),
                    sel,
                    self.binding(|s| s.viewport, |s, v| s.viewport = v)
                )
                .accessibility_label("Target")
                .on_activate(self.action())
                .disabled(d)
            ),
            Kind::NodeEditor => view!(
                NodeEditor(
                    "target",
                    self.binding(|s| s.graph.clone(), |s, v| s.graph = v),
                    sel,
                    self.binding(|s| s.viewport, |s, v| s.viewport = v)
                )
                .accessibility_label("Target")
                .on_activate(self.action())
                .disabled(d)
            ),
            Kind::ToolbarItem => view!(Toolbar(
                "target",
                [ToolbarItem::action("item", "Target", self.action()).disabled(d)]
            )),
            Kind::Toolbar => view!(
                Toolbar(
                    "target",
                    [ToolbarItem::action("item", "Target", self.action())]
                )
                .disabled(d)
            ),
            Kind::BreadcrumbItem => view!(
                Breadcrumb([
                    BreadcrumbItem::new("Target")
                        .action(self.action())
                        .disabled(d),
                    BreadcrumbItem::new("Current")
                ])
                .id("target")
            ),
            Kind::Disclosure => view!(Disclosure("target", "Target", b, EmptyView).disabled(d)),
            Kind::NavigationLink => view!(
                NavigationLink(
                    "Target",
                    "page",
                    self.binding(|s| s.path.clone(), |s, v| s.path = v)
                )
                .id("target")
                .disabled(d)
            ),
            Kind::SidebarItem => view!(Sidebar(
                "target",
                [SidebarItem::new("item", "Target").disabled(d)],
                sel
            )),
            Kind::NavigationSplitItem => view!(NavigationSplit(
                "target",
                [NavigationSplitItem::new("item", "Target", EmptyView).disabled(d)],
                sel
            )),
            Kind::TabItem => view!(
                TabView(
                    [TabItem::new("One", "Target", EmptyView).disabled(d)],
                    choice
                )
                .id("target")
            ),
            Kind::TreeItem => view!(
                Tree([TreeNode::new("item", "Target").disabled(d)], sel)
                    .id("target")
                    .height(100.)
                    .on_activate(self.action())
            ),
            Kind::MenuItem => view!(
                Menu(
                    "Open menu",
                    [MenuItem::action("item", "Target", self.action()).disabled(d)]
                )
                .id("target")
            ),
            Kind::ListRow => view!(
                List(["Target"], self.binding(|s| s.index, |s, v| s.index = v))
                    .id("target")
                    .height(80.)
            ),
            Kind::GridCell => {
                let col = DataGridColumn::text(
                    "name",
                    "Target",
                    |r: &Row| r.name.clone(),
                    |v| Ok(v.to_owned()),
                    |r, v| r.name = v,
                );
                view!(
                    EditableDataGrid(
                        "Target",
                        self.binding(|s| s.rows.clone(), |s, v| s.rows = v),
                        |_: &Row| "row".into(),
                        vec![if d { col.read_only() } else { col }]
                    )
                    .id("target")
                )
            }
            Kind::DragSource => {
                view!(DragSource("target", "payload".to_owned(), Text("Target")).enabled(!d))
            }
            Kind::DropTarget => view!(
                DropTarget(
                    "target",
                    self.binding(|s| s.dropped.clone(), |s, v| s.dropped = v),
                    Text("Target")
                )
                .enabled(!d)
            ),
            Kind::FileDropTarget => view!(
                DropTarget::files(
                    "target",
                    self.binding(|s| s.files.clone(), |s, v| s.files = v),
                    Text("Target")
                )
                .enabled(!d)
            ),
            kind => {
                let row = match kind {
                    Kind::PropertyText | Kind::PropertyInspector => {
                        PropertyInspectorRow::text("row", "Target", t)
                    }
                    Kind::PropertyChoice => {
                        PropertyInspectorRow::choice("row", "Target", choice, ["One", "Two"])
                    }
                    Kind::PropertyToggle => PropertyInspectorRow::toggle("row", "Target", b),
                    Kind::PropertyInteger => PropertyInspectorRow::integer("row", "Target", i),
                    Kind::PropertyIntegerValue => PropertyInspectorRow::integer_value(
                        "row",
                        "Target",
                        self.binding(|s| s.large, |s, v| s.large = v),
                    ),
                    _ => unreachable!(),
                };
                view!(
                    PropertyInspector(
                        "target",
                        [row.disabled(d && kind != Kind::PropertyInspector)]
                    )
                    .unwrap()
                    .disabled(d && kind == Kind::PropertyInspector)
                )
            }
        }
    }
    fn snapshot(&self) -> String {
        format!("{:?}:{:?}:{}", self.values, self.writes, self.actions)
    }
}
impl View for Owner {
    fn body(&self) -> impl View {
        self.target()
    }
}
struct Harness {
    owner: Entity<ComponentHost<Owner>>,
    before: FocusHandle,
    after: FocusHandle,
}
impl Render for Harness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl gpui::IntoElement {
        div()
            .flex()
            .flex_col()
            .w(px(580.))
            .gap_2()
            .child(
                div()
                    .id("r15-before")
                    .track_focus(&self.before)
                    .tab_index(0)
                    .role(gpui::Role::Button)
                    .aria_label("Before")
                    .h(px(24.))
                    .child("Before"),
            )
            .child(
                div()
                    .id("r15-region")
                    .debug_selector(|| "r15-region".into())
                    .role(gpui::Role::Group)
                    .aria_label("R15 region")
                    .h(px(260.))
                    .w_full()
                    .child(self.owner.clone()),
            )
            .child(
                div()
                    .id("r15-after")
                    .track_focus(&self.after)
                    .tab_index(0)
                    .role(gpui::Role::Button)
                    .aria_label("After")
                    .h(px(24.))
                    .child("After"),
            )
    }
}
fn draw(window: &mut gpui::VisualTestContext) {
    window.run_until_parked();
    window.update(|window, app| window.draw(app).clear(app));
}
fn keys(window: &mut gpui::VisualTestContext) {
    for key in ["enter", "space"] {
        let stroke = gpui::Keystroke::parse(key).unwrap();
        window.simulate_event(gpui::KeyDownEvent {
            keystroke: stroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        window.simulate_event(gpui::KeyUpEvent { keystroke: stroke });
    }
}
fn tree(window: &mut gpui::VisualTestContext) -> serde_json::Value {
    window.update(|window, _| {
        serde_json::from_str(&window.debug_a11y_tree_json().expect("active AX tree")).unwrap()
    })
}
fn interactive_nodes(tree: &serde_json::Value) -> Vec<&serde_json::Value> {
    tree["nodes"]
        .as_object()
        .unwrap()
        .values()
        .filter(|n| {
            let a = &n["aria"];
            let label = a["label"].as_str().unwrap_or("");
            !["Before", "After", "R15 region"].contains(&label)
                && (a["role"] != "ListItem" || label == "Target")
                && ((a["role"] == "Group" && label.starts_with("Target"))
                    || matches!(
                        a["role"].as_str().unwrap_or(""),
                        "Button"
                            | "CheckBox"
                            | "RadioButton"
                            | "Switch"
                            | "Slider"
                            | "ComboBox"
                            | "TextInput"
                            | "MultilineTextInput"
                            | "TextField"
                            | "Tab"
                            | "TreeItem"
                            | "MenuItem"
                            | "Link"
                            | "SpinButton"
                            | "GridCell"
                            | "Cell"
                            | "ListItem"
                    ))
        })
        .collect()
}
fn click_region(window: &mut gpui::VisualTestContext) {
    let ax = tree(window);
    let scale = ax["frame"]["scale_factor"]
        .as_f64()
        .expect("test window scale");
    let has_menu = interactive_nodes(&ax)
        .iter()
        .any(|n| n["aria"]["role"] == "MenuItem");
    let points = interactive_nodes(&ax)
        .into_iter()
        .filter(|n| !has_menu || n["aria"]["role"] == "MenuItem")
        .filter_map(|node| {
            let b = &node["bounds"];
            Some(gpui::point(
                px(((b["x"].as_f64()? + b["width"].as_f64()? / 2.) / scale) as f32),
                px(((b["y"].as_f64()? + b["height"].as_f64()? / 2.) / scale) as f32),
            ))
        })
        .collect::<Vec<_>>();
    if points.is_empty() {
        let bounds = window.debug_bounds("r15-region").expect("target bounds");
        window.simulate_click(bounds.center(), gpui::Modifiers::default());
    } else {
        for point in points {
            window.simulate_click(point, gpui::Modifiers::default());
        }
    }
}

fn setup(
    cx: &mut gpui::TestAppContext,
    kind: Kind,
) -> (
    Entity<ComponentHost<Owner>>,
    FocusHandle,
    FocusHandle,
    &mut gpui::VisualTestContext,
) {
    let theme = crate::theme::parse_theme(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../themes/macos27.json"
    )))
    .unwrap();
    cx.update(|app| {
        gpui_component::init(app);
        crate::theme::activate_theme(&theme, Some(crate::theme::AppearanceMode::Light), app)
            .unwrap();
    });
    let saved = Rc::new(std::cell::RefCell::new(None));
    let saved_owner = saved.clone();
    let (_, window) = cx.add_window_view(|window, app| {
        let owner = app.new(|cx| {
            ComponentHost::new(
                Owner {
                    kind,
                    disabled: false,
                    slot: 0,
                    values: [Slot::default(), Slot::default()],
                    writes: [0; 2],
                    actions: 0,
                },
                cx,
            )
        });
        let before = app.focus_handle().tab_stop(true);
        let after = app.focus_handle().tab_stop(true);
        *saved_owner.borrow_mut() = Some((owner.clone(), before.clone(), after.clone()));
        let probe = app.new(|_| Harness {
            owner: owner.clone(),
            before: before.clone(),
            after: after.clone(),
        });
        gpui_component::Root::new(probe, window, app)
    });
    let (owner, before, after) = saved.borrow().clone().unwrap();
    (owner, before, after, window)
}
fn run(cx: &mut gpui::TestAppContext, kind: Kind, check: Check) {
    let (owner, before, after, window) = setup(cx, kind);
    draw(window);
    if let Some(reason) = na(kind, check) {
        println!("R15 N/A {kind:?} {check:?}: {reason}");
        return;
    }
    // Capture real enabled focus handles, including all compound-control children.
    let mut handles = vec![];
    window.update(|w, a| w.focus(&before, a));
    for _ in 0..12 {
        window.update(|w, a| w.focus_next(a));
        let focus = window.update(|w, a| w.focused(a).unwrap());
        if focus == after || focus == before {
            break;
        }
        if handles.contains(&focus) {
            break;
        }
        handles.push(focus);
    }
    if check != Check::Recovery {
        window.update(|w, a| w.focus(&before, a));
    }
    owner.update(window, |h, cx| {
        h.component_mut().disabled = true;
        cx.notify();
    });
    draw(window);
    if kind == Kind::MenuItem && matches!(check, Check::Mouse | Check::Keys | Check::Accessibility)
    {
        click_region(window);
        draw(window);
    }
    match check {
        Check::Tab => {
            window.update(|w, a| w.focus(&before, a));
            window.simulate_keystrokes("tab");
            draw(window);
            if kind == Kind::NavigationSplitItem {
                // The divider is an independent enabled control after the sidebar item.
                let ax = tree(window);
                let focus = ax["gpui_focus"].as_str().expect("focused divider");
                assert_eq!(ax["nodes"][focus]["aria"]["role"], "Splitter");
                window.simulate_keystrokes("tab");
                draw(window);
            }
            assert!(
                window.update(|w, _| after.is_focused(w)),
                "{kind:?}: Tab must skip disabled target"
            );
            window.simulate_keystrokes("shift-tab");
            draw(window);
            if kind == Kind::NavigationSplitItem {
                let ax = tree(window);
                let focus = ax["gpui_focus"].as_str().expect("focused divider");
                assert_eq!(ax["nodes"][focus]["aria"]["role"], "Splitter");
                window.simulate_keystrokes("shift-tab");
                draw(window);
            }
            assert!(
                window.update(|w, _| before.is_focused(w)),
                "{kind:?}: Shift-Tab must skip disabled target"
            );
        }
        Check::Mouse | Check::Keys => {
            let snapshot = owner.read_with(window, |h, _| h.component().snapshot());
            if check == Check::Mouse {
                click_region(window);
            } else {
                // Force each old focus handle: stale event/input handlers must also reject disabled input.
                for focus in &handles {
                    window.update(|w, a| w.focus(focus, a));
                    draw(window);
                    keys(window);
                    window.simulate_input("r15");
                }
            }
            draw(window);
            assert_eq!(
                owner.read_with(window, |h, _| h.component().snapshot()),
                snapshot,
                "{kind:?}: disabled {check:?} wrote owner state"
            );
        }
        Check::Accessibility => {
            let ax = tree(window);
            let nodes = interactive_nodes(&ax)
                .into_iter()
                .filter(|n| kind != Kind::MenuItem || n["aria"]["role"] == "MenuItem")
                .collect::<Vec<_>>();
            if kind == Kind::FileDropTarget {
                let node = ax["nodes"]
                    .as_object()
                    .unwrap()
                    .values()
                    .find(|n| n["aria"]["label"] == "File drop target")
                    .expect("file drop AX node");
                assert_eq!(
                    node["aria"]["disabled"], true,
                    "file drop target must report disabled"
                );
            } else {
                assert!(
                    !nodes.is_empty(),
                    "{kind:?}: no interactive AX node in {:?}",
                    ax["nodes"]
                        .as_object()
                        .unwrap()
                        .values()
                        .map(|n| n["aria"].clone())
                        .collect::<Vec<_>>()
                );
                for node in nodes {
                    assert_eq!(
                        node["aria"]["disabled"], true,
                        "{kind:?}: disabled AX node: {}",
                        node["aria"]
                    );
                }
            }
        }
        Check::Recovery => {
            assert!(
                !handles.is_empty(),
                "{kind:?}: enabled control has no focusable element"
            );
            // Focus a real control while enabled, then disable it without pressing Tab.
            owner.update(window, |h, cx| {
                h.component_mut().disabled = false;
                cx.notify();
            });
            draw(window);
            window.update(|w, a| w.focus(&handles[0], a));
            owner.update(window, |h, cx| {
                h.component_mut().disabled = true;
                cx.notify();
            });
            draw(window);
            let focused_next = if kind == Kind::NavigationSplitItem {
                let ax = tree(window);
                ax["gpui_focus"]
                    .as_str()
                    .is_some_and(|id| ax["nodes"][id]["aria"]["role"] == "Splitter")
            } else {
                window.update(|w, _| after.is_focused(w))
            };
            assert!(
                focused_next,
                "{kind:?}: disabling focused target did not immediately focus next element"
            );
            owner.update(window, |h, cx| {
                h.component_mut().disabled = false;
                cx.notify();
            });
            draw(window);
            window.update(|w, a| w.focus(&before, a));
            window.simulate_keystrokes("tab");
            draw(window);
            assert!(
                window.update(|w, a| w.focused(a).is_some_and(|f| f != before && f != after)),
                "{kind:?}: re-enabled control not focusable"
            );
            let snapshot = owner.read_with(window, |h, _| h.component().snapshot());
            let old_tree = tree(window);
            keys(window);
            if matches!(
                kind,
                Kind::TextField
                    | Kind::SecureField
                    | Kind::SearchField
                    | Kind::TextEditor
                    | Kind::CodeEditor
                    | Kind::TokenField
                    | Kind::PropertyText
                    | Kind::PropertyIntegerValue
                    | Kind::PropertyInspector
            ) {
                window.simulate_input("r15");
            }
            if kind == Kind::Slider {
                window.simulate_keystrokes("right");
            }
            click_region(window);
            draw(window);
            assert!(
                owner.read_with(window, |h, _| h.component().snapshot()) != snapshot
                    || tree(window)["nodes"] != old_tree["nodes"],
                "{kind:?}: re-enabled control did not work"
            );
        }
    }
}

macro_rules! case {
    ($name:ident,$kind:ident, [$($tab:meta),*], [$($mouse:meta),*], [$($keys:meta),*], [$($ax:meta),*], [$($recovery:meta),*]) => {
        mod $name {
            use super::*;
            $(#[$tab])*
            #[gpui::test]
            fn tab(cx: &mut gpui::TestAppContext) {
                run(cx, Kind::$kind, Check::Tab)
            }
            $(#[$mouse])*
            #[gpui::test]
            fn mouse(cx: &mut gpui::TestAppContext) {
                run(cx, Kind::$kind, Check::Mouse)
            }
            $(#[$keys])*
            #[gpui::test]
            fn keys(cx: &mut gpui::TestAppContext) {
                run(cx, Kind::$kind, Check::Keys)
            }
            $(#[$ax])*
            #[gpui::test]
            fn accessibility(cx: &mut gpui::TestAppContext) {
                run(cx, Kind::$kind, Check::Accessibility)
            }
            $(#[$recovery])*
            #[gpui::test]
            fn recovery(cx: &mut gpui::TestAppContext) {
                run(cx, Kind::$kind, Check::Recovery)
            }
        }
    };
}
case!(button, Button, [], [], [], [], []);
case!(link, Link, [], [], [], [], []);
case!(checkbox, Checkbox, [], [], [], [], []);
case!(radio, Radio, [], [], [], [], []);
case!(toggle, Toggle, [], [], [], [], []);
case!(slider, Slider, [], [], [], [], []);
case!(stepper, Stepper, [], [], [], [], []);
case!(
    picker,
    Picker,
    [],
    [],
    [],
    [],
    [ignore = "R15b: Picker re-enabled control did not work"]
);
case!(
    combo_box,
    ComboBox,
    [],
    [],
    [],
    [],
    [ignore = "R15b: ComboBox re-enabled control did not work"]
);
case!(text_field, TextField, [], [], [], [], []);
case!(secure_field, SecureField, [], [], [], [], []);
case!(search_field, SearchField, [], [], [], [], []);
case!(text_editor, TextEditor, [], [], [], [], []);
case!(
    date_picker,
    DatePicker,
    [],
    [],
    [],
    [],
    [ignore = "R15b: DatePicker re-enabled control did not work"]
);
case!(
    color_picker,
    ColorPicker,
    [],
    [],
    [],
    [],
    [ignore = "R15b: ColorPicker re-enabled control did not work"]
);
case!(token_field, TokenField, [], [], [], [], []);
case!(code_editor, CodeEditor, [], [], [], [], []);
case!(graph_view, GraphView, [], [], [], [], []);
case!(node_editor, NodeEditor, [], [], [], [], []);
case!(property_text, PropertyText, [], [], [], [], []);
case!(
    property_choice,
    PropertyChoice,
    [],
    [],
    [],
    [],
    [ignore = "R15b: PropertyChoice re-enabled control did not work"]
);
case!(property_toggle, PropertyToggle, [], [], [], [], []);
case!(property_integer, PropertyInteger, [], [], [], [], []);
case!(
    property_integer_value,
    PropertyIntegerValue,
    [],
    [],
    [],
    [],
    []
);
case!(property_inspector, PropertyInspector, [], [], [], [], []);
case!(toolbar_item, ToolbarItem, [], [], [], [], []);
case!(toolbar, Toolbar, [], [], [], [], []);
case!(breadcrumb_item, BreadcrumbItem, [], [], [], [], []);
case!(disclosure, Disclosure, [], [], [], [], []);
case!(navigation_link, NavigationLink, [], [], [], [], []);
case!(sidebar_item, SidebarItem, [], [], [], [], []);
case!(
    navigation_split_item,
    NavigationSplitItem,
    [],
    [],
    [],
    [],
    []
);
case!(tab_item, TabItem, [], [], [], [], []);
case!(tree_item, TreeItem, [], [], [], [], []);
case!(
    menu_item,
    MenuItem,
    [],
    [],
    [],
    [ignore = "R15b: MenuItem accessibility node has no disabled flag"],
    [ignore = "R15b: popup item live-disable recovery not yet exercised"]
);
case!(list_row, ListRow, [], [], [], [], []);
case!(grid_cell, GridCell, [], [], [], [], []);
case!(drag_source, DragSource, [], [], [], [], []);
case!(drop_target, DropTarget, [], [], [], [], []);
case!(file_drop_target, FileDropTarget, [], [], [], [], []);

#[gpui::test]
fn replacing_a_binding_while_disabled_never_writes_the_old_binding(cx: &mut gpui::TestAppContext) {
    let (owner, before, after, window) = setup(cx, Kind::TextField);
    draw(window);
    window.update(|w, a| w.focus(&before, a));
    window.simulate_keystrokes("tab");
    draw(window);
    let old_focus = window.update(|w, a| w.focused(a).unwrap());
    assert_ne!(old_focus, before);
    assert_ne!(old_focus, after);
    let old = owner.read_with(window, |h, _| {
        (
            h.component().values[0].text.clone(),
            h.component().writes[0],
        )
    });
    owner.update(window, |h, cx| {
        let o = h.component_mut();
        o.disabled = true;
        o.slot = 1;
        o.values[1].text = "replacement".into();
        cx.notify();
    });
    draw(window);
    window.update(|w, a| w.focus(&old_focus, a));
    draw(window);
    keys(window);
    window.simulate_input("blocked");
    draw(window);
    assert_eq!(
        owner.read_with(window, |h, _| (
            h.component().values[0].text.clone(),
            h.component().writes[0]
        )),
        old,
        "old binding written during disabled replacement"
    );
    owner.update(window, |h, cx| {
        h.component_mut().disabled = false;
        cx.notify();
    });
    draw(window);
    window.update(|w, a| w.focus(&before, a));
    window.simulate_keystrokes("tab");
    draw(window);
    window.simulate_input("new");
    draw(window);
    owner.read_with(window, |h, _| {
        assert_eq!(
            (
                h.component().values[0].text.clone(),
                h.component().writes[0]
            ),
            old,
            "stale subscription wrote old binding after re-enable"
        );
        assert!(
            h.component().writes[1] > 0,
            "replacement binding must receive new input"
        );
    });
}
