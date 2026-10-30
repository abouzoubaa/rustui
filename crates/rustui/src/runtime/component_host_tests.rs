// Recovered owner lifecycle/event-path regressions. Uses GPUI test windows.
use super::*;
use crate::components::navigation::{NavigationDestination, NavigationStack, TabItem, TabView};
use crate::runtime::AppAction;
use crate::view::{ActionHandler, AnyView, Component, EmptyView, OwnerActionReader, RenderContext};
use gpui::AppContext;
use rustui_core::{Action, Binding};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

/// Repeatable test-window workload, not a native frame or GPU benchmark.
#[gpui::test]
#[ignore = "manual performance workload"]
fn owner_update_and_draw_workload(cx: &mut gpui::TestAppContext) {
    use crate::prelude::{Text, VStack};
    use std::time::Instant;

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

    struct Workload {
        value: u64,
        rows: usize,
        renders: Arc<AtomicUsize>,
    }

    impl View for Workload {
        fn body(&self) -> impl View {
            self.renders.fetch_add(1, Ordering::Relaxed);
            let mut stack = VStack::new();
            for row in 0..self.rows {
                stack = stack.child(Text(format!("row {row}: {}", self.value)));
            }
            stack
        }
    }

    println!("rows,sample,updates,render_calls,elapsed_ns,ns_per_update_and_draw");
    for rows in [10, 100, 1000] {
        let renders = Arc::new(AtomicUsize::new(0));
        let (owner, window) = cx.add_window_view(|_, app| {
            ComponentHost::new(
                Workload {
                    value: 0,
                    rows,
                    renders: renders.clone(),
                },
                app,
            )
        });
        window.update(|window, app| {
            let _ = window.draw(app);
        });
        let mut total_updates = 0;
        for sample in 0..=10 {
            let before_renders = renders.load(Ordering::Relaxed);
            let start = Instant::now();
            for _ in 0..10 {
                owner.update(window, |host, cx| {
                    host.component_mut().value += 1;
                    cx.notify();
                });
                window.update(|window, app| {
                    window.refresh();
                    let _ = window.draw(app);
                });
                total_updates += 1;
            }
            let elapsed = start.elapsed().as_nanos();
            let render_calls = renders.load(Ordering::Relaxed) - before_renders;
            assert!(render_calls >= 10);
            window.update(|_, app| assert_eq!(owner.read(app).component().value, total_updates));
            if sample > 0 {
                println!(
                    "{rows},{sample},10,{render_calls},{elapsed},{:.1}",
                    elapsed as f64 / 10.0
                );
            }
        }
    }
}

struct LifecycleChild {
    mounts: Arc<AtomicUsize>,
    releases: Arc<AtomicUsize>,
}

impl View for LifecycleChild {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn on_mount(&mut self, _cx: &mut App) {
        self.mounts.fetch_add(1, Ordering::SeqCst);
    }

    fn on_unmount(&mut self, _cx: &mut App) {
        self.releases.fetch_add(1, Ordering::SeqCst);
    }
}

struct ReplacementChild {
    mounts: Arc<AtomicUsize>,
    releases: Arc<AtomicUsize>,
}

impl View for ReplacementChild {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn on_mount(&mut self, _cx: &mut App) {
        self.mounts.fetch_add(1, Ordering::SeqCst);
    }

    fn on_unmount(&mut self, _cx: &mut App) {
        self.releases.fetch_add(1, Ordering::SeqCst);
    }
}

struct ConditionalParent {
    child_visible: bool,
    replacement: bool,
    mounts: Arc<AtomicUsize>,
    releases: Arc<AtomicUsize>,
}

impl View for ConditionalParent {
    fn body(&self) -> impl View {
        let mounts = self.mounts.clone();
        let releases = self.releases.clone();
        let child = if self.replacement {
            AnyView::new(Component("conditional-child", move || ReplacementChild {
                mounts: mounts.clone(),
                releases: releases.clone(),
            }))
        } else {
            AnyView::new(Component("conditional-child", move || LifecycleChild {
                mounts: mounts.clone(),
                releases: releases.clone(),
            }))
        };
        AnyView::new(if self.child_visible {
            child
        } else {
            AnyView::new(EmptyView)
        })
    }
}

struct ReorderedParent {
    reverse: bool,
    mounts: [Arc<AtomicUsize>; 2],
    releases: [Arc<AtomicUsize>; 2],
}

impl View for ReorderedParent {
    fn body(&self) -> impl View {
        let first_mounts = self.mounts[0].clone();
        let first_releases = self.releases[0].clone();
        let first = AnyView::new(Component("first-child", move || LifecycleChild {
            mounts: first_mounts.clone(),
            releases: first_releases.clone(),
        }));
        let second_mounts = self.mounts[1].clone();
        let second_releases = self.releases[1].clone();
        let second = AnyView::new(Component("second-child", move || LifecycleChild {
            mounts: second_mounts.clone(),
            releases: second_releases.clone(),
        }));
        let children = if self.reverse {
            [second, first]
        } else {
            [first, second]
        };
        crate::layout::VStack::new().children(children)
    }
}

struct TabLifecycleParent {
    selected: String,
    activity_declared: bool,
    mounts: [Arc<AtomicUsize>; 2],
    releases: [Arc<AtomicUsize>; 2],
}

fn read_tab_selection(parent: &TabLifecycleParent) -> String {
    parent.selected.clone()
}

fn write_tab_selection(parent: &mut TabLifecycleParent, selected: String) {
    parent.selected = selected;
}

impl View for TabLifecycleParent {
    fn body(&self) -> impl View {
        let overview_mounts = self.mounts[0].clone();
        let overview_releases = self.releases[0].clone();
        let overview = TabItem::new(
            "overview",
            "Overview",
            AnyView::new(Component("overview-content", move || LifecycleChild {
                mounts: overview_mounts.clone(),
                releases: overview_releases.clone(),
            })),
        );
        let mut tabs = vec![overview];
        if self.activity_declared {
            let activity_mounts = self.mounts[1].clone();
            let activity_releases = self.releases[1].clone();
            tabs.push(TabItem::new(
                "activity",
                "Activity",
                AnyView::new(Component("activity-content", move || LifecycleChild {
                    mounts: activity_mounts.clone(),
                    releases: activity_releases.clone(),
                })),
            ));
        }
        TabView(
            tabs,
            Binding::new(
                self.selected.clone(),
                read_tab_selection,
                write_tab_selection,
            ),
        )
        .id("lifecycle-tabs")
    }
}

struct NavigationLifecycleParent {
    path: Vec<String>,
    mounts: [Arc<AtomicUsize>; 3],
    releases: [Arc<AtomicUsize>; 3],
}

fn read_navigation_path(parent: &NavigationLifecycleParent) -> Vec<String> {
    parent.path.clone()
}

fn write_navigation_path(parent: &mut NavigationLifecycleParent, path: Vec<String>) {
    parent.path = path;
}

impl View for NavigationLifecycleParent {
    fn body(&self) -> impl View {
        let root_mounts = self.mounts[0].clone();
        let root_releases = self.releases[0].clone();
        let root = AnyView::new(Component("root-page", move || LifecycleChild {
            mounts: root_mounts.clone(),
            releases: root_releases.clone(),
        }));
        let details_mounts = self.mounts[1].clone();
        let details_releases = self.releases[1].clone();
        let details = NavigationDestination::new(
            "details",
            AnyView::new(Component("details-page", move || LifecycleChild {
                mounts: details_mounts.clone(),
                releases: details_releases.clone(),
            })),
        );
        let settings_mounts = self.mounts[2].clone();
        let settings_releases = self.releases[2].clone();
        let settings = NavigationDestination::new(
            "settings",
            AnyView::new(Component("settings-page", move || LifecycleChild {
                mounts: settings_mounts.clone(),
                releases: settings_releases.clone(),
            })),
        );
        NavigationStack(
            root,
            [details, settings],
            Binding::new(
                self.path.clone(),
                read_navigation_path,
                write_navigation_path,
            ),
        )
        .id("lifecycle-navigation")
    }
}

type ActionHandlerSlot = Rc<Mutex<Option<ActionHandler>>>;
type BindingWriter = Rc<dyn Fn(String, &mut App)>;
type BindingWriterSlot = Rc<Mutex<Option<BindingWriter>>>;

struct ActionBindingChild {
    key: &'static str,
    action_calls: Arc<AtomicUsize>,
    value: Arc<Mutex<String>>,
    action_slot: ActionHandlerSlot,
    binding_writer_slot: BindingWriterSlot,
}

struct ActionBindingOwner {
    action_calls: Arc<AtomicUsize>,
    value: Arc<Mutex<String>>,
    action_slot: ActionHandlerSlot,
    binding_writer_slot: BindingWriterSlot,
    children: Vec<ActionBindingChild>,
}

fn increment_action_binding_owner(owner: &mut ActionBindingOwner) {
    owner.action_calls.fetch_add(1, Ordering::SeqCst);
}

fn read_action_binding_owner_value(owner: &ActionBindingOwner) -> String {
    owner.value.lock().unwrap().clone()
}

fn write_action_binding_owner_value(owner: &mut ActionBindingOwner, value: String) {
    *owner.value.lock().unwrap() = value;
}

#[derive(Default, Debug, PartialEq, Eq)]
struct OwnerActionState {
    actions: usize,
    captured: Option<String>,
    conditional_actions: usize,
}

struct OwnerActionProbe {
    state: Arc<Mutex<OwnerActionState>>,
    captured_actions: Rc<std::cell::RefCell<Option<CapturedOwnerActions>>>,
    releases: Arc<AtomicUsize>,
}

struct CapturedOwnerActions {
    direct: crate::view::OwnerAction<OwnerActionProbe>,
    action: AppAction,
    action_with: AppAction,
    action_if_changed: AppAction,
}

struct OwnerActionCapture {
    actions: CapturedOwnerActions,
    slot: Rc<std::cell::RefCell<Option<CapturedOwnerActions>>>,
}

fn increment_owner_action(owner: &mut OwnerActionProbe) {
    owner.state.lock().unwrap().actions += 1;
}

fn capture_owner_action_value(owner: &mut OwnerActionProbe, value: &String) {
    owner.state.lock().unwrap().captured = Some(value.clone());
}

fn count_changed_owner_action(owner: &mut OwnerActionProbe) -> bool {
    owner.state.lock().unwrap().conditional_actions += 1;
    true
}

impl View for OwnerActionProbe {
    fn body(&self) -> impl View {
        let slot = self.captured_actions.clone();
        OwnerActionReader::<Self, _, _>(move |owner| OwnerActionCapture {
            actions: CapturedOwnerActions {
                direct: owner.clone(),
                action: owner.action(increment_owner_action),
                action_with: owner
                    .action_with("captured value".to_owned(), capture_owner_action_value),
                action_if_changed: owner.action_if_changed(count_changed_owner_action),
            },
            slot: slot.clone(),
        })
    }

    fn on_unmount(&mut self, _cx: &mut App) {
        self.releases.fetch_add(1, Ordering::SeqCst);
    }
}

struct OwnerActionReleaseParent {
    child_visible: bool,
    state: Arc<Mutex<OwnerActionState>>,
    captured_actions: Rc<std::cell::RefCell<Option<CapturedOwnerActions>>>,
    releases: Arc<AtomicUsize>,
}

impl View for OwnerActionReleaseParent {
    fn body(&self) -> impl View {
        let state = self.state.clone();
        let captured_actions = self.captured_actions.clone();
        let releases = self.releases.clone();
        let child = AnyView::new(Component("owner-action-child", move || OwnerActionProbe {
            state: state.clone(),
            captured_actions: captured_actions.clone(),
            releases: releases.clone(),
        }));
        if self.child_visible {
            child
        } else {
            AnyView::new(EmptyView)
        }
    }
}

impl View for OwnerActionCapture {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> gpui::AnyElement {
        *self.slot.borrow_mut() = Some(CapturedOwnerActions {
            direct: self.actions.direct.clone(),
            action: self.actions.action.clone(),
            action_with: self.actions.action_with.clone(),
            action_if_changed: self.actions.action_if_changed.clone(),
        });
        EmptyView.into_element(context)
    }
}

struct ActionBindingCapture {
    action: Action,
    binding: Binding<String>,
    action_slot: ActionHandlerSlot,
    binding_writer_slot: BindingWriterSlot,
}

impl View for ActionBindingCapture {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> gpui::AnyElement {
        *self.action_slot.lock().unwrap() = Some(context.bind_action(self.action.clone()));
        *self.binding_writer_slot.lock().unwrap() =
            Some(context.binding_writer(self.binding.clone()));
        EmptyView.into_element(context)
    }
}

impl View for ActionBindingOwner {
    fn body(&self) -> impl View {
        let capture = AnyView::new(ActionBindingCapture {
            action: Action::new(increment_action_binding_owner),
            binding: Binding::new(
                self.value.lock().unwrap().clone(),
                read_action_binding_owner_value,
                write_action_binding_owner_value,
            ),
            action_slot: self.action_slot.clone(),
            binding_writer_slot: self.binding_writer_slot.clone(),
        });
        let mut children = vec![capture];
        for child in &self.children {
            let key = child.key;
            let action_calls = child.action_calls.clone();
            let value = child.value.clone();
            let action_slot = child.action_slot.clone();
            let binding_writer_slot = child.binding_writer_slot.clone();
            children.push(AnyView::new(Component(key, move || ActionBindingOwner {
                action_calls: action_calls.clone(),
                value: value.clone(),
                action_slot: action_slot.clone(),
                binding_writer_slot: binding_writer_slot.clone(),
                children: Vec::new(),
            })));
        }
        crate::layout::VStack::new().children(children)
    }
}

struct SubscriptionEvent;

struct SubscriptionSource;

impl gpui::EventEmitter<SubscriptionEvent> for SubscriptionSource {}

struct SubscriptionChild {
    source: Entity<SubscriptionSource>,
    observed: Arc<AtomicUsize>,
    received: Arc<AtomicUsize>,
    mounts: Arc<AtomicUsize>,
    releases: Arc<AtomicUsize>,
}

impl View for SubscriptionChild {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> gpui::AnyElement {
        let subscription_id = Identity::from("subscription-owner");
        let (_, created) = context.retained_entity(subscription_id.clone(), |_, _| ());
        if created {
            let observed = self.observed.clone();
            context.observe_retained(&subscription_id, &self.source, move |_, _| {
                observed.fetch_add(1, Ordering::SeqCst);
            });
            let received = self.received.clone();
            context.subscribe_retained(&subscription_id, &self.source, move |_, _, _| {
                received.fetch_add(1, Ordering::SeqCst);
            });
        }
        EmptyView.into_element(context)
    }

    fn on_mount(&mut self, _cx: &mut App) {
        self.mounts.fetch_add(1, Ordering::SeqCst);
    }

    fn on_unmount(&mut self, _cx: &mut App) {
        self.releases.fetch_add(1, Ordering::SeqCst);
    }
}

struct SubscriptionParent {
    child_visible: bool,
    source: Entity<SubscriptionSource>,
    observed: Arc<AtomicUsize>,
    received: Arc<AtomicUsize>,
    mounts: Arc<AtomicUsize>,
    releases: Arc<AtomicUsize>,
}

impl View for SubscriptionParent {
    fn body(&self) -> impl View {
        let source = self.source.clone();
        let observed = self.observed.clone();
        let received = self.received.clone();
        let mounts = self.mounts.clone();
        let releases = self.releases.clone();
        AnyView::new(if self.child_visible {
            AnyView::new(Component("subscription-child", move || SubscriptionChild {
                source: source.clone(),
                observed: observed.clone(),
                received: received.clone(),
                mounts: mounts.clone(),
                releases: releases.clone(),
            }))
        } else {
            AnyView::new(EmptyView)
        })
    }
}

struct PlainLifecycleDescription {
    mounts: Arc<AtomicUsize>,
    releases: Arc<AtomicUsize>,
}

impl View for PlainLifecycleDescription {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn on_mount(&mut self, _cx: &mut App) {
        self.mounts.fetch_add(1, Ordering::SeqCst);
    }

    fn on_unmount(&mut self, _cx: &mut App) {
        self.releases.fetch_add(1, Ordering::SeqCst);
    }
}

struct LifecycleBoundaryParent {
    plain_visible: bool,
    owned_visible: bool,
    plain_mounts: Arc<AtomicUsize>,
    plain_releases: Arc<AtomicUsize>,
    owned_mounts: Arc<AtomicUsize>,
    owned_releases: Arc<AtomicUsize>,
}

impl View for LifecycleBoundaryParent {
    fn body(&self) -> impl View {
        let plain_mounts = self.plain_mounts.clone();
        let plain_releases = self.plain_releases.clone();
        let plain = if self.plain_visible {
            AnyView::new(PlainLifecycleDescription {
                mounts: plain_mounts,
                releases: plain_releases,
            })
        } else {
            AnyView::new(EmptyView)
        };
        let owned_mounts = self.owned_mounts.clone();
        let owned_releases = self.owned_releases.clone();
        let owned = if self.owned_visible {
            AnyView::new(Component("explicit-lifecycle-owner", move || {
                LifecycleChild {
                    mounts: owned_mounts.clone(),
                    releases: owned_releases.clone(),
                }
            }))
        } else {
            AnyView::new(EmptyView)
        };
        crate::layout::VStack::new().child(plain).child(owned)
    }
}

#[gpui::test]
fn keyed_children_keep_their_owners_when_sibling_order_changes(cx: &mut gpui::TestAppContext) {
    let mounts = [Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0))];
    let releases = [Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0))];
    cx.update(gpui_component::init);

    let (parent, window_cx) = cx.add_window_view(|_, cx| {
        ComponentHost::new(
            ReorderedParent {
                reverse: false,
                mounts: mounts.clone(),
                releases: releases.clone(),
            },
            cx,
        )
    });
    assert_eq!(
        std::array::from_fn(|index| mounts[index].load(Ordering::SeqCst)),
        [1, 1]
    );
    assert_eq!(
        std::array::from_fn(|index| releases[index].load(Ordering::SeqCst)),
        [0, 0]
    );

    parent.update(window_cx, |host, cx| {
        host.component_mut().reverse = true;
        cx.notify();
    });
    window_cx.update(|window, app| {
        window.refresh();
        let _ = window.draw(app);
    });
    window_cx.cx.run_until_parked();

    assert_eq!(
        std::array::from_fn(|index| mounts[index].load(Ordering::SeqCst)),
        [1, 1]
    );
    assert_eq!(
        std::array::from_fn(|index| releases[index].load(Ordering::SeqCst)),
        [0, 0]
    );
}

#[gpui::test]
fn inactive_tab_owner_is_retained_until_its_tab_is_removed(cx: &mut gpui::TestAppContext) {
    let mounts = [Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0))];
    let releases = [Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0))];
    cx.update(gpui_component::init);

    let (parent, window_cx) = cx.add_window_view(|_, cx| {
        ComponentHost::new(
            TabLifecycleParent {
                selected: "overview".to_owned(),
                activity_declared: true,
                mounts: mounts.clone(),
                releases: releases.clone(),
            },
            cx,
        )
    });
    let counts = |values: &[Arc<AtomicUsize>; 2]| {
        [
            values[0].load(Ordering::SeqCst),
            values[1].load(Ordering::SeqCst),
        ]
    };
    assert_eq!(counts(&mounts), [1, 0]);
    assert_eq!(counts(&releases), [0, 0]);

    parent.update(window_cx, |host, cx| {
        host.component_mut().selected = "activity".to_owned();
        cx.notify();
    });
    window_cx.update(|window, app| {
        window.refresh();
        let _ = window.draw(app);
    });
    window_cx.cx.run_until_parked();
    assert_eq!(counts(&mounts), [1, 1]);
    assert_eq!(counts(&releases), [0, 0]);

    parent.update(window_cx, |host, cx| {
        host.component_mut().selected = "overview".to_owned();
        cx.notify();
    });
    window_cx.update(|window, app| {
        window.refresh();
        let _ = window.draw(app);
    });
    window_cx.cx.run_until_parked();
    assert_eq!(counts(&mounts), [1, 1]);
    assert_eq!(counts(&releases), [0, 0]);

    parent.update(window_cx, |host, cx| {
        host.component_mut().activity_declared = false;
        cx.notify();
    });
    window_cx.update(|window, app| {
        window.refresh();
        let _ = window.draw(app);
    });
    window_cx.cx.run_until_parked();
    assert_eq!(counts(&releases), [0, 1]);

    parent.update(window_cx, |host, cx| {
        host.component_mut().activity_declared = true;
        host.component_mut().selected = "activity".to_owned();
        cx.notify();
    });
    window_cx.update(|window, app| {
        window.refresh();
        let _ = window.draw(app);
    });
    window_cx.cx.run_until_parked();
    assert_eq!(counts(&mounts), [1, 2]);
    assert_eq!(counts(&releases), [0, 1]);
}

#[gpui::test]
fn covered_navigation_pages_are_retained_until_pop_then_remount_on_push(
    cx: &mut gpui::TestAppContext,
) {
    let mounts = [
        Arc::new(AtomicUsize::new(0)),
        Arc::new(AtomicUsize::new(0)),
        Arc::new(AtomicUsize::new(0)),
    ];
    let releases = [
        Arc::new(AtomicUsize::new(0)),
        Arc::new(AtomicUsize::new(0)),
        Arc::new(AtomicUsize::new(0)),
    ];
    cx.update(gpui_component::init);

    let (parent, window_cx) = cx.add_window_view(|_, cx| {
        ComponentHost::new(
            NavigationLifecycleParent {
                path: Vec::new(),
                mounts: mounts.clone(),
                releases: releases.clone(),
            },
            cx,
        )
    });
    let counts = |values: &[Arc<AtomicUsize>; 3]| {
        [
            values[0].load(Ordering::SeqCst),
            values[1].load(Ordering::SeqCst),
            values[2].load(Ordering::SeqCst),
        ]
    };
    assert_eq!(counts(&mounts), [1, 0, 0]);
    assert_eq!(counts(&releases), [0, 0, 0]);

    parent.update(window_cx, |host, cx| {
        host.component_mut().path.push("details".to_owned());
        cx.notify();
    });
    window_cx.update(|window, app| {
        window.refresh();
        let _ = window.draw(app);
    });
    window_cx.cx.run_until_parked();
    assert_eq!(counts(&mounts), [1, 1, 0]);
    assert_eq!(counts(&releases), [0, 0, 0]);

    parent.update(window_cx, |host, cx| {
        host.component_mut().path.push("settings".to_owned());
        cx.notify();
    });
    window_cx.update(|window, app| {
        window.refresh();
        let _ = window.draw(app);
    });
    window_cx.cx.run_until_parked();
    assert_eq!(counts(&mounts), [1, 1, 1]);
    assert_eq!(counts(&releases), [0, 0, 0]);

    parent.update(window_cx, |host, cx| {
        host.component_mut().path.pop();
        cx.notify();
    });
    window_cx.update(|window, app| {
        window.refresh();
        let _ = window.draw(app);
    });
    window_cx.cx.run_until_parked();
    assert_eq!(counts(&mounts), [1, 1, 1]);
    assert_eq!(counts(&releases), [0, 0, 1]);

    parent.update(window_cx, |host, cx| {
        host.component_mut().path.pop();
        cx.notify();
    });
    window_cx.update(|window, app| {
        window.refresh();
        let _ = window.draw(app);
    });
    window_cx.cx.run_until_parked();
    assert_eq!(counts(&mounts), [1, 1, 1]);
    assert_eq!(counts(&releases), [0, 1, 1]);

    parent.update(window_cx, |host, cx| {
        host.component_mut().path.push("details".to_owned());
        cx.notify();
    });
    window_cx.update(|window, app| {
        window.refresh();
        let _ = window.draw(app);
    });
    window_cx.cx.run_until_parked();
    assert_eq!(counts(&mounts), [1, 2, 1]);
    assert_eq!(counts(&releases), [0, 1, 1]);
}

#[gpui::test]
fn local_actions_and_bindings_update_only_their_same_type_owner(cx: &mut gpui::TestAppContext) {
    let action_calls = [
        Arc::new(AtomicUsize::new(0)),
        Arc::new(AtomicUsize::new(0)),
        Arc::new(AtomicUsize::new(0)),
    ];
    let values = [
        Arc::new(Mutex::new("parent".to_owned())),
        Arc::new(Mutex::new("first child".to_owned())),
        Arc::new(Mutex::new("second child".to_owned())),
    ];
    let action_counts = || {
        action_calls
            .each_ref()
            .map(|calls| calls.load(Ordering::SeqCst))
    };
    let current_values = || values.each_ref().map(|value| value.lock().unwrap().clone());
    let action_slots: [ActionHandlerSlot; 3] = std::array::from_fn(|_| Rc::new(Mutex::new(None)));
    let binding_writer_slots: [BindingWriterSlot; 3] =
        std::array::from_fn(|_| Rc::new(Mutex::new(None)));
    cx.update(gpui_component::init);

    let (parent, window_cx) = cx.add_window_view(|_, cx| {
        ComponentHost::new(
            ActionBindingOwner {
                action_calls: action_calls[0].clone(),
                value: values[0].clone(),
                action_slot: action_slots[0].clone(),
                binding_writer_slot: binding_writer_slots[0].clone(),
                children: [1, 2]
                    .into_iter()
                    .map(|index| ActionBindingChild {
                        key: if index == 1 {
                            "first-child"
                        } else {
                            "second-child"
                        },
                        action_calls: action_calls[index].clone(),
                        value: values[index].clone(),
                        action_slot: action_slots[index].clone(),
                        binding_writer_slot: binding_writer_slots[index].clone(),
                    })
                    .collect(),
            },
            cx,
        )
    });
    assert!(
        action_slots
            .iter()
            .all(|slot| slot.lock().unwrap().is_some())
    );
    assert!(
        binding_writer_slots
            .iter()
            .all(|slot| slot.lock().unwrap().is_some())
    );

    let first_action = action_slots[1].lock().unwrap().take().unwrap();
    let first_binding = binding_writer_slots[1]
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .clone();
    window_cx.update(|window, app| {
        first_action(&gpui::ClickEvent::default(), window, app);
        first_binding("first child updated".to_owned(), app);
    });
    window_cx.cx.run_until_parked();
    assert_eq!(action_counts(), [0, 1, 0]);
    assert_eq!(
        current_values(),
        [
            "parent".to_owned(),
            "first child updated".to_owned(),
            "second child".to_owned()
        ]
    );

    let second_action = action_slots[2].lock().unwrap().take().unwrap();
    let second_binding = binding_writer_slots[2]
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .clone();
    window_cx.update(|window, app| {
        second_action(&gpui::ClickEvent::default(), window, app);
        second_binding("second child updated".to_owned(), app);
    });
    window_cx.cx.run_until_parked();
    assert_eq!(action_counts(), [0, 1, 1]);
    assert_eq!(
        current_values(),
        [
            "parent".to_owned(),
            "first child updated".to_owned(),
            "second child updated".to_owned()
        ]
    );
    drop(parent);
}

fn external_owner_request(owner: &mut OwnerActionProbe, value: String) -> (String, bool) {
    owner.state.lock().unwrap().captured = Some(value.clone());
    (value, true)
}

fn external_owner_read(_owner: &mut OwnerActionProbe, value: u64) -> (u64, bool) {
    (value, false)
}

#[gpui::test]
fn external_owner_dispatch_distinguishes_same_type_owners_and_unchanged_reply(
    cx: &mut gpui::TestAppContext,
) {
    cx.update(gpui_component::init);
    let first_state = Arc::new(Mutex::new(OwnerActionState::default()));
    let second_state = Arc::new(Mutex::new(OwnerActionState::default()));
    let first_slot = Rc::new(std::cell::RefCell::new(None));
    let second_slot = Rc::new(std::cell::RefCell::new(None));
    let releases = Arc::new(AtomicUsize::new(0));
    let first = {
        let (_, window_cx) = cx.add_window_view(|_, cx| {
            ComponentHost::new(
                OwnerActionProbe {
                    state: first_state.clone(),
                    captured_actions: first_slot.clone(),
                    releases: releases.clone(),
                },
                cx,
            )
        });
        window_cx.cx.run_until_parked();
        first_slot.borrow_mut().take().unwrap()
    };
    let (_, window_cx) = cx.add_window_view(|_, cx| {
        ComponentHost::new(
            OwnerActionProbe {
                state: second_state.clone(),
                captured_actions: second_slot.clone(),
                releases: releases.clone(),
            },
            cx,
        )
    });
    let second = second_slot.borrow_mut().take().unwrap();
    window_cx.update(|_, app| {
        assert_eq!(
            first
                .direct
                .dispatch_with("first".to_owned(), external_owner_request, app),
            Some("first".to_owned())
        );
        assert_eq!(
            second
                .direct
                .dispatch_with("second".to_owned(), external_owner_request, app),
            Some("second".to_owned())
        );
        assert_eq!(
            first.direct.dispatch_with(42, external_owner_read, app),
            Some(42)
        );
    });
    assert_eq!(
        first_state.lock().unwrap().captured.as_deref(),
        Some("first")
    );
    assert_eq!(
        second_state.lock().unwrap().captured.as_deref(),
        Some("second")
    );
}

#[gpui::test]
fn owner_action_reader_dispatches_each_callback_to_its_host(cx: &mut gpui::TestAppContext) {
    let state = Arc::new(Mutex::new(OwnerActionState::default()));
    let captured_actions = Rc::new(std::cell::RefCell::new(None));
    let releases = Arc::new(AtomicUsize::new(0));
    cx.update(gpui_component::init);

    let (_, window_cx) = cx.add_window_view(|_, cx| {
        ComponentHost::new(
            OwnerActionProbe {
                state: state.clone(),
                captured_actions: captured_actions.clone(),
                releases,
            },
            cx,
        )
    });
    let actions = captured_actions
        .borrow_mut()
        .take()
        .expect("OwnerActionReader captured its callback actions");

    window_cx.update(|window, app| {
        let event = gpui::ClickEvent::default();
        actions.action.dispatch(&event, window, app);
        actions.action_with.dispatch(&event, window, app);
        actions.action_if_changed.dispatch(&event, window, app);
    });
    window_cx.cx.run_until_parked();

    assert_eq!(
        *state.lock().unwrap(),
        OwnerActionState {
            actions: 1,
            captured: Some("captured value".to_owned()),
            conditional_actions: 1,
        }
    );
}

#[gpui::test]
fn escaped_owner_action_callbacks_are_inert_after_host_release(cx: &mut gpui::TestAppContext) {
    let state = Arc::new(Mutex::new(OwnerActionState::default()));
    let captured_actions = Rc::new(std::cell::RefCell::new(None));
    let releases = Arc::new(AtomicUsize::new(0));
    cx.update(gpui_component::init);

    let (parent, window_cx) = cx.add_window_view(|_, cx| {
        ComponentHost::new(
            OwnerActionReleaseParent {
                child_visible: true,
                state: state.clone(),
                captured_actions: captured_actions.clone(),
                releases: releases.clone(),
            },
            cx,
        )
    });
    let actions = captured_actions
        .borrow_mut()
        .take()
        .expect("OwnerActionReader captured its callback actions");

    parent.update(window_cx, |host, cx| {
        host.component_mut().child_visible = false;
        cx.notify();
    });
    window_cx.update(|window, app| {
        window.refresh();
        let _ = window.draw(app);
    });
    window_cx.cx.run_until_parked();
    assert_eq!(releases.load(Ordering::SeqCst), 1);

    window_cx.update(|window, app| {
        let event = gpui::ClickEvent::default();
        actions.action.dispatch(&event, window, app);
        actions.action_with.dispatch(&event, window, app);
        actions.action_if_changed.dispatch(&event, window, app);
    });
    window_cx.cx.run_until_parked();

    window_cx.update(|_, app| {
        assert_eq!(
            actions
                .direct
                .dispatch_with("released".to_owned(), external_owner_request, app),
            None
        );
        assert_eq!(
            actions.direct.dispatch_with(42, external_owner_read, app),
            None
        );
    });
    assert_eq!(*state.lock().unwrap(), OwnerActionState::default());
    drop(parent);
}

#[gpui::test]
fn plain_nested_views_do_not_receive_component_lifecycle_callbacks(cx: &mut gpui::TestAppContext) {
    let plain_mounts = Arc::new(AtomicUsize::new(0));
    let plain_releases = Arc::new(AtomicUsize::new(0));
    let owned_mounts = Arc::new(AtomicUsize::new(0));
    let owned_releases = Arc::new(AtomicUsize::new(0));
    cx.update(gpui_component::init);

    let (parent, window_cx) = cx.add_window_view(|_, cx| {
        ComponentHost::new(
            LifecycleBoundaryParent {
                plain_visible: true,
                owned_visible: true,
                plain_mounts: plain_mounts.clone(),
                plain_releases: plain_releases.clone(),
                owned_mounts: owned_mounts.clone(),
                owned_releases: owned_releases.clone(),
            },
            cx,
        )
    });
    assert_eq!(plain_mounts.load(Ordering::SeqCst), 0);
    assert_eq!(plain_releases.load(Ordering::SeqCst), 0);
    assert_eq!(owned_mounts.load(Ordering::SeqCst), 1);
    assert_eq!(owned_releases.load(Ordering::SeqCst), 0);

    parent.update(window_cx, |host, cx| {
        host.component_mut().plain_visible = false;
        host.component_mut().owned_visible = false;
        cx.notify();
    });
    window_cx.update(|window, app| {
        window.refresh();
        let _ = window.draw(app);
    });
    window_cx.cx.run_until_parked();
    assert_eq!(plain_mounts.load(Ordering::SeqCst), 0);
    assert_eq!(plain_releases.load(Ordering::SeqCst), 0);
    assert_eq!(owned_mounts.load(Ordering::SeqCst), 1);
    assert_eq!(owned_releases.load(Ordering::SeqCst), 1);

    parent.update(window_cx, |host, cx| {
        host.component_mut().plain_visible = true;
        host.component_mut().owned_visible = true;
        cx.notify();
    });
    window_cx.update(|window, app| {
        window.refresh();
        let _ = window.draw(app);
    });
    window_cx.cx.run_until_parked();
    assert_eq!(plain_mounts.load(Ordering::SeqCst), 0);
    assert_eq!(plain_releases.load(Ordering::SeqCst), 0);
    assert_eq!(owned_mounts.load(Ordering::SeqCst), 2);
    assert_eq!(owned_releases.load(Ordering::SeqCst), 1);
    drop(parent);
}

#[gpui::test]
fn removed_child_keeps_subscriptions_until_its_last_gpui_entity_handle_is_dropped(
    cx: &mut gpui::TestAppContext,
) {
    let observed = Arc::new(AtomicUsize::new(0));
    let received = Arc::new(AtomicUsize::new(0));
    let mounts = Arc::new(AtomicUsize::new(0));
    let releases = Arc::new(AtomicUsize::new(0));
    let source_slot: Rc<Mutex<Option<Entity<SubscriptionSource>>>> = Rc::new(Mutex::new(None));
    cx.update(gpui_component::init);

    let parent_observed = observed.clone();
    let parent_received = received.clone();
    let parent_mounts = mounts.clone();
    let parent_releases = releases.clone();
    let (parent, window_cx) = cx.add_window_view({
        let source_slot = source_slot.clone();
        move |_, cx| {
            let source = cx.new(|_| SubscriptionSource);
            *source_slot.lock().unwrap() = Some(source.clone());
            ComponentHost::new(
                SubscriptionParent {
                    child_visible: true,
                    source,
                    observed: parent_observed.clone(),
                    received: parent_received.clone(),
                    mounts: parent_mounts.clone(),
                    releases: parent_releases.clone(),
                },
                cx,
            )
        }
    });
    let source = source_slot.lock().unwrap().take().unwrap();
    assert_eq!(mounts.load(Ordering::SeqCst), 1);
    assert_eq!(releases.load(Ordering::SeqCst), 0);

    source.update(window_cx, |_, cx| {
        cx.emit(SubscriptionEvent);
        cx.notify();
    });
    window_cx.cx.run_until_parked();
    assert_eq!(observed.load(Ordering::SeqCst), 1);
    assert_eq!(received.load(Ordering::SeqCst), 1);

    let external_child = parent.update(window_cx, |host, _| {
        host.retained.begin_frame();
        let (child, created) = host.retained.component::<SubscriptionChild>(
            Identity::from("subscription-child"),
            || unreachable!(),
        );
        assert!(!created);
        host.retained.finish_frame();
        child
    });
    let weak_child = external_child.downgrade();

    parent.update(window_cx, |host, cx| {
        host.component_mut().child_visible = false;
        cx.notify();
    });
    window_cx.update(|window, app| {
        window.refresh();
        let _ = window.draw(app);
    });
    window_cx.cx.run_until_parked();
    assert_eq!(releases.load(Ordering::SeqCst), 0);

    source.update(window_cx, |_, cx| {
        cx.emit(SubscriptionEvent);
        cx.notify();
    });
    window_cx.cx.run_until_parked();
    assert_eq!(observed.load(Ordering::SeqCst), 2);
    assert_eq!(received.load(Ordering::SeqCst), 2);

    drop(external_child);
    window_cx.update(|window, app| {
        window.refresh();
        let _ = window.draw(app);
    });
    window_cx.cx.run_until_parked();
    assert_eq!(releases.load(Ordering::SeqCst), 1);
    weak_child.assert_released();

    source.update(window_cx, |_, cx| {
        cx.emit(SubscriptionEvent);
        cx.notify();
    });
    window_cx.cx.run_until_parked();
    assert_eq!(observed.load(Ordering::SeqCst), 2);
    assert_eq!(received.load(Ordering::SeqCst), 2);

    parent.update(window_cx, |host, cx| {
        host.component_mut().child_visible = true;
        cx.notify();
    });
    window_cx.update(|window, app| {
        window.refresh();
        let _ = window.draw(app);
    });
    window_cx.cx.run_until_parked();
    assert_eq!(mounts.load(Ordering::SeqCst), 2);

    source.update(window_cx, |_, cx| {
        cx.emit(SubscriptionEvent);
        cx.notify();
    });
    window_cx.cx.run_until_parked();
    assert_eq!(observed.load(Ordering::SeqCst), 3);
    assert_eq!(received.load(Ordering::SeqCst), 3);
    drop(parent);
}

#[gpui::test]
fn keyed_child_lifecycle_tracks_retention_type_replacement_and_removal(
    cx: &mut gpui::TestAppContext,
) {
    let mounts = Arc::new(AtomicUsize::new(0));
    let releases = Arc::new(AtomicUsize::new(0));
    cx.update(gpui_component::init);

    let (parent, window_cx) = cx.add_window_view(|_, cx| {
        ComponentHost::new(
            ConditionalParent {
                child_visible: true,
                replacement: false,
                mounts: mounts.clone(),
                releases: releases.clone(),
            },
            cx,
        )
    });
    assert_eq!(mounts.load(Ordering::SeqCst), 1);
    assert_eq!(releases.load(Ordering::SeqCst), 0);

    parent.update(window_cx, |host, cx| {
        host.component_mut().child_visible = true;
        cx.notify();
    });
    window_cx.update(|window, app| {
        window.refresh();
        let _ = window.draw(app);
    });
    window_cx.cx.run_until_parked();
    assert_eq!(mounts.load(Ordering::SeqCst), 1);
    assert_eq!(releases.load(Ordering::SeqCst), 0);

    parent.update(window_cx, |host, cx| {
        host.component_mut().replacement = true;
        cx.notify();
    });
    window_cx.update(|window, app| {
        window.refresh();
        let _ = window.draw(app);
    });
    window_cx.cx.run_until_parked();
    assert_eq!(mounts.load(Ordering::SeqCst), 2);
    assert_eq!(releases.load(Ordering::SeqCst), 1);

    parent.update(window_cx, |host, cx| {
        host.component_mut().child_visible = false;
        cx.notify();
    });
    window_cx.update(|window, app| {
        window.refresh();
        let _ = window.draw(app);
    });
    window_cx.cx.run_until_parked();
    assert_eq!(releases.load(Ordering::SeqCst), 2);

    parent.update(window_cx, |host, cx| {
        host.component_mut().child_visible = true;
        cx.notify();
    });
    window_cx.update(|window, app| {
        window.refresh();
        let _ = window.draw(app);
    });
    window_cx.cx.run_until_parked();
    assert_eq!(mounts.load(Ordering::SeqCst), 3);
    assert_eq!(releases.load(Ordering::SeqCst), 2);

    parent.update(window_cx, |host, cx| {
        host.component_mut().child_visible = false;
        cx.notify();
    });
    window_cx.update(|window, app| {
        window.refresh();
        let _ = window.draw(app);
    });
    window_cx.cx.run_until_parked();
    assert_eq!(releases.load(Ordering::SeqCst), 3);
}

#[gpui::test]
fn retained_child_unmount_waits_for_the_last_gpui_entity_handle(cx: &mut gpui::TestAppContext) {
    let mounts = Arc::new(AtomicUsize::new(0));
    let releases = Arc::new(AtomicUsize::new(0));
    cx.update(gpui_component::init);

    let (parent, window_cx) = cx.add_window_view(|_, cx| {
        ComponentHost::new(
            ConditionalParent {
                child_visible: false,
                replacement: false,
                mounts: mounts.clone(),
                releases: releases.clone(),
            },
            cx,
        )
    });
    let external_child = parent.update(window_cx, |host, cx| {
        host.retained.begin_frame();
        let (child, created) = host.retained.component(Identity::from("external"), || {
            cx.new(|cx| {
                ComponentHost::new(
                    LifecycleChild {
                        mounts: mounts.clone(),
                        releases: releases.clone(),
                    },
                    cx,
                )
            })
        });
        assert!(created);
        host.retained.finish_frame();
        child
    });
    let weak_child = external_child.downgrade();
    assert_eq!(mounts.load(Ordering::SeqCst), 1);
    assert_eq!(releases.load(Ordering::SeqCst), 0);

    parent.update(window_cx, |host, _| {
        host.retained.begin_frame();
        host.retained.finish_frame();
    });
    window_cx.update(|window, app| {
        window.refresh();
        let _ = window.draw(app);
    });
    window_cx.cx.run_until_parked();
    assert_eq!(releases.load(Ordering::SeqCst), 0);

    drop(external_child);
    window_cx.update(|window, app| {
        window.refresh();
        let _ = window.draw(app);
    });
    window_cx.cx.run_until_parked();
    assert_eq!(releases.load(Ordering::SeqCst), 1);
    weak_child.assert_released();
}
