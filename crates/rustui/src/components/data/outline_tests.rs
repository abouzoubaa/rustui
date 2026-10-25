use super::*;
use crate::{components::data::outline::Outline, runtime::ComponentHost};

fn ids(ids: &[&str]) -> HashSet<String> {
    ids.iter().map(|id| (*id).to_owned()).collect()
}

fn nodes() -> Vec<TreeNode> {
    vec![
        TreeNode::new("root", "Document")
            .child(TreeNode::new("nested", "Section").child(TreeNode::new("leaf", "Heading"))),
    ]
}

#[test]
fn controlled_expansion_overrides_declarations_and_keeps_hidden_branch_ids() {
    let nodes = vec![
        TreeNode::new("root", "Document").expanded(true).child(
            TreeNode::new("nested", "Section")
                .expanded(true)
                .child(TreeNode::new("leaf", "Heading")),
        ),
    ];
    let expanded = valid_expanded_branches(&nodes, &ids(&["nested", "leaf", "gone"]));
    assert_eq!(expanded, ids(&["nested"]));
    let expansion = controlled_expansion(&nodes, &expanded);
    assert_eq!(expansion["root"], false);
    assert_eq!(expansion["nested"], true);
    assert_eq!(expansion["leaf"], false);
    assert_eq!(
        visible_controlled_selection(&nodes, Some("leaf"), &expanded),
        None
    );
    assert_eq!(valid_selection(&nodes, Some("leaf")), Some("leaf".into()));
}

#[test]
fn queued_expansion_rejects_stale_owner_state_and_does_not_repeat_an_update() {
    let request = ExpansionRequest {
        expected_owner: ids(&[]),
        expanded: ids(&["root"]),
    };
    let mut current = ids(&["nested"]);
    assert!(!apply_expansion_request(&mut current, &request));
    assert_eq!(current, ids(&["nested"]));
    current.clear();
    let pending = RefCell::new(Some(request));
    assert!(apply_pending_expansion_request(&mut current, &pending));
    assert_eq!(current, ids(&["root"]));
    assert!(!apply_pending_expansion_request(&mut current, &pending));
    let identical = ExpansionRequest {
        expected_owner: current.clone(),
        expanded: current.clone(),
    };
    assert!(!apply_expansion_request(&mut current, &identical));
}

struct OutlineProbe {
    selected: Option<String>,
    expanded: HashSet<String>,
    replacement_expanded: HashSet<String>,
    replacement_binding: bool,
    expansion_writes: usize,
    replacement_writes: usize,
    visible: bool,
}

impl OutlineProbe {
    fn new() -> Self {
        Self {
            selected: Some("root".into()),
            expanded: HashSet::new(),
            replacement_expanded: HashSet::new(),
            replacement_binding: false,
            expansion_writes: 0,
            replacement_writes: 0,
            visible: true,
        }
    }
}

impl View for OutlineProbe {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let selected = Binding::new(
            self.selected.clone(),
            |owner: &Self| owner.selected.clone(),
            |owner, value| owner.selected = value,
        );
        let expanded = if self.replacement_binding {
            Binding::new(
                self.replacement_expanded.clone(),
                |owner: &Self| owner.replacement_expanded.clone(),
                |owner, value| {
                    owner.replacement_expanded = value;
                    owner.replacement_writes += 1;
                },
            )
        } else {
            Binding::new(
                self.expanded.clone(),
                |owner: &Self| owner.expanded.clone(),
                |owner, value| {
                    owner.expanded = value;
                    owner.expansion_writes += 1;
                },
            )
        };
        let mut frame = div().w(px(320.0)).h(px(280.0));
        if self.visible {
            frame = frame.child(
                Outline("outline-event", nodes(), selected, expanded)
                    .accessibility_label("Document outline")
                    .height(260.0)
                    .into_element(context),
            );
        }
        frame.into_any_element()
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

fn native_tree(
    owner: &gpui::Entity<ComponentHost<OutlineProbe>>,
    window: &mut gpui::VisualTestContext,
) -> gpui::Entity<TreeState> {
    window.update(|window, app| {
        let _ = window.draw(app);
        let retained = owner
            .read(app)
            .retained_entity_for_test::<RetainedTree>(vec![], Identity::from("outline-event"))
            .expect("production Outline render retained the shared Tree engine");
        retained.read(app).tree.clone()
    })
}

#[gpui::test]
fn physical_outline_keys_update_the_latest_expansion_binding_once(cx: &mut gpui::TestAppContext) {
    init(cx);
    let (owner, window) = cx.add_window_view(|_, app| ComponentHost::new(OutlineProbe::new(), app));
    let native = native_tree(&owner, window);
    window.update(|window, app| native.update(app, |tree, cx| tree.focus(window, cx)));
    window.simulate_keystrokes("right");
    window.update(|window, app| {
        let _ = window.draw(app);
        assert_eq!(owner.read(app).component().expanded, ids(&["root"]));
        assert_eq!(owner.read(app).component().expansion_writes, 1);
    });
    window.simulate_keystrokes("right");
    window.update(|window, app| {
        let _ = window.draw(app);
        assert_eq!(owner.read(app).component().expansion_writes, 1);
        owner.update(app, |host, cx| {
            host.component_mut().replacement_expanded = ids(&["root"]);
            host.component_mut().replacement_binding = true;
            cx.notify();
        });
        let _ = window.draw(app);
    });
    let same_native = native_tree(&owner, window);
    assert_eq!(native.entity_id(), same_native.entity_id());
    window.simulate_keystrokes("left");
    window.update(|window, app| {
        let _ = window.draw(app);
        let probe = owner.read(app).component();
        assert_eq!(
            probe.expansion_writes, 1,
            "retained observer must not retain the old Binding"
        );
        assert_eq!(probe.expanded, ids(&["root"]));
        assert!(probe.replacement_expanded.is_empty());
        assert_eq!(probe.replacement_writes, 1);
        assert!(!native.read(app).entry(0).unwrap().is_expanded());
    });
}

#[gpui::test]
fn owner_expansion_is_render_authoritative_without_revealing_hidden_selection(
    cx: &mut gpui::TestAppContext,
) {
    init(cx);
    let (owner, window) = cx.add_window_view(|_, app| ComponentHost::new(OutlineProbe::new(), app));
    let native = native_tree(&owner, window);
    window.update(|window, app| {
        owner.update(app, |host, cx| {
            host.component_mut().expanded = ids(&["root", "nested"]);
            host.component_mut().selected = Some("leaf".into());
            cx.notify();
        });
        let _ = window.draw(app);
        assert_eq!(
            current_expanded_branches(native.read(app)),
            ids(&["root", "nested"])
        );
        assert_eq!(
            native.read(app).selected_item().unwrap().id.as_ref(),
            "leaf"
        );
        owner.update(app, |host, cx| {
            host.component_mut().expanded = ids(&["nested"]);
            cx.notify();
        });
        let _ = window.draw(app);
        assert_eq!(
            current_expanded_branches(native.read(app)),
            ids(&["nested"])
        );
        assert!(native.read(app).selected_item().is_none());
        assert_eq!(
            owner.read(app).component().selected.as_deref(),
            Some("leaf")
        );
        assert_eq!(owner.read(app).component().expansion_writes, 0);
    });
    // An old tree handle may outlive its removed render scope. Its events must no
    // longer write the former owner's binding, even before GPUI releases it.
    window.update(|window, app| {
        owner.update(app, |host, cx| {
            host.component_mut().visible = false;
            cx.notify();
        });
        let _ = window.draw(app);
        assert!(
            owner
                .read(app)
                .retained_entity_for_test::<RetainedTree>(vec![], Identity::from("outline-event"))
                .is_none()
        );
        native.update(app, |tree, cx| {
            tree.set_items(
                vec![
                    TreeItem::new("stale", "Stale")
                        .child(TreeItem::new("old", "Old"))
                        .expanded(true),
                ],
                cx,
            )
        });
        let _ = window.draw(app);
        assert_eq!(owner.read(app).component().expanded, ids(&["nested"]));
        assert_eq!(owner.read(app).component().expansion_writes, 0);
        owner.update(app, |host, cx| {
            host.component_mut().visible = true;
            cx.notify();
        });
        let _ = window.draw(app);
    });
    let remounted = native_tree(&owner, window);
    assert_ne!(native.entity_id(), remounted.entity_id());
    window.update(|window, app| {
        // Initial hidden selection needs an observation baseline even when no
        // native state change is necessary during its first render.
        remounted.update(app, |tree, cx| tree.set_selected_index(None, cx));
        let _ = window.draw(app);
        assert_eq!(
            current_expanded_branches(remounted.read(app)),
            ids(&["nested"])
        );
        assert_eq!(
            owner.read(app).component().selected.as_deref(),
            Some("leaf")
        );
    });
}
