//! R20: the same deterministic window matrix for every public async starter.
use super::*;
use crate::view::{Component, EmptyView, OwnerTaskReader};
use gpui::{TestAppContext, VisualTestContext};
use std::{
    cell::{Cell, RefCell},
    pin::Pin,
    rc::Rc,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    task::{Context, Poll, Waker},
};

type State = AsyncState<u32, &'static str>;

#[derive(Clone, Default)]
struct Observed {
    events: Rc<RefCell<Vec<State>>>,
    controller: Rc<RefCell<Option<OwnerTask<Probe>>>>,
    renders: Rc<Cell<usize>>,
    unmounts: Rc<Cell<usize>>,
}
struct Probe {
    observed: Observed,
    unrelated: u32,
    state: State,
}
impl View for Probe {
    fn body(&self) -> impl View {
        self.observed.renders.set(self.observed.renders.get() + 1);
        let slot = self.observed.controller.clone();
        OwnerTaskReader::<Self, _, _>(move |owner| {
            *slot.borrow_mut() = Some(owner);
            EmptyView
        })
    }
    fn on_unmount(&mut self, _: &mut App) {
        self.observed.unmounts.set(self.observed.unmounts.get() + 1);
    }
}
struct Parent {
    mounted: bool,
    observed: Observed,
}
impl View for Parent {
    fn body(&self) -> impl View {
        let observed = self.observed.clone();
        self.mounted.then(|| {
            Component("async-child", move || Probe {
                observed: observed.clone(),
                unrelated: 0,
                state: State::Idle,
            })
        })
    }
}
fn apply(probe: &mut Probe, state: State) -> bool {
    probe.observed.events.borrow_mut().push(state.clone());
    probe.state = state;
    true
}

#[derive(Clone, Copy, Debug)]
enum Api {
    Start,
    WithExecutor,
}
#[derive(Clone, Copy, Debug)]
enum Check {
    Unmount,
    Restart,
    Error,
    OwnerUpdate,
    NoLeak,
}

#[derive(Default)]
struct GateState {
    result: Option<Result<u32, &'static str>>,
    waker: Option<Waker>,
}
#[derive(Clone, Default)]
struct Gate {
    state: Arc<Mutex<GateState>>,
    started: Arc<AtomicUsize>,
    polled: Arc<AtomicUsize>,
    completed: Arc<AtomicUsize>,
    dropped: Arc<AtomicUsize>,
}
impl Gate {
    fn release(&self, result: Result<u32, &'static str>) {
        let wake = {
            let mut state = self.state.lock().unwrap();
            state.result = Some(result);
            state.waker.take()
        };
        if let Some(wake) = wake {
            wake.wake();
        }
    }
    fn work(&self) -> Work {
        self.started.fetch_add(1, Ordering::SeqCst);
        Work(self.clone())
    }
}
struct Work(Gate);
impl Future for Work {
    type Output = Result<u32, &'static str>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.0.polled.fetch_add(1, Ordering::SeqCst);
        let mut state = self.0.state.lock().unwrap();
        if let Some(result) = state.result.take() {
            self.0.completed.fetch_add(1, Ordering::SeqCst);
            Poll::Ready(result)
        } else {
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
impl Drop for Work {
    fn drop(&mut self) {
        self.0.dropped.fetch_add(1, Ordering::SeqCst);
    }
}
fn start(
    api: Api,
    owner: &OwnerTask<Probe>,
    gate: &Gate,
    window: &mut VisualTestContext,
) -> AppAction {
    let gate = gate.clone();
    let action = match api {
        Api::Start => owner.start("request", move || gate.work(), apply),
        Api::WithExecutor => owner.start_with_executor(
            "request",
            move |executor| {
                // Exercise the supplied GPUI executor, retaining its child task
                // inside the owner task rather than detaching it.
                executor.spawn(gate.work())
            },
            apply,
        ),
    };
    window.update(|window, app| action.dispatch_keyboard(window, app));
    action
}
fn draw(window: &mut VisualTestContext) {
    window.run_until_parked();
    window.update(|window, app| window.draw(app).clear(app));
    window.run_until_parked();
}
fn run(cx: &mut TestAppContext, api: Api, check: Check) {
    cx.update(gpui_component::init);
    let observed = Observed::default();
    let (root, window) = cx.add_window_view(|_, app| {
        ComponentHost::new(
            Parent {
                mounted: true,
                observed: observed.clone(),
            },
            app,
        )
    });
    draw(window);
    let owner = observed
        .controller
        .borrow()
        .clone()
        .expect("reader rendered in child");
    let weak = owner.owner.clone();
    let old = Gate::default();
    let retained_action = start(api, &owner, &old, window);
    draw(window);
    assert_eq!(old.started.load(Ordering::SeqCst), 1);
    assert!(
        old.polled.load(Ordering::SeqCst) > 0,
        "work must be pending before check"
    );
    assert_eq!(*observed.events.borrow(), vec![State::Loading]);
    match check {
        Check::Unmount | Check::NoLeak => {
            // Remove the actual retained child from its parent's rendered tree.
            window.update(|_, app| {
                root.update(app, |root, cx| {
                    root.component_mut().mounted = false;
                    cx.notify();
                })
            });
            draw(window);
            assert_eq!(observed.unmounts.get(), 1, "child unmounted exactly once");
            assert!(
                weak.upgrade().is_none(),
                "task/controller must not retain owner"
            );
            assert_eq!(
                old.dropped.load(Ordering::SeqCst),
                1,
                "unmount drops pending work"
            );
            old.release(Ok(99));
            draw(window);
            assert_eq!(
                old.completed.load(Ordering::SeqCst),
                0,
                "cancelled work cannot complete"
            );
            assert_eq!(
                *observed.events.borrow(),
                vec![State::Loading],
                "no callback/state write after unmount"
            );
            // Retained actions and readers also cannot resurrect their owner.
            window.update(|window, app| retained_action.dispatch_keyboard(window, app));
            draw(window);
            assert!(weak.upgrade().is_none());
        }
        Check::Restart => {
            // First cancel a genuinely pending run with changed input.
            let newest = Gate::default();
            start(api, &owner, &newest, window);
            draw(window);
            assert_eq!(
                old.dropped.load(Ordering::SeqCst),
                1,
                "restart drops old future"
            );
            old.release(Ok(11));
            newest.release(Ok(22));
            draw(window);
            assert_eq!(
                *observed.events.borrow(),
                vec![State::Loading, State::Loading, State::Success(22)]
            );
            // Then race a completed background result (success AND error)
            // against replacement before its foreground apply is polled.
            for result in [Ok(33), Err("stale")] {
                let stale = Gate::default();
                start(api, &owner, &stale, window);
                draw(window);
                stale.release(result);
                for _ in 0..100 {
                    if stale.completed.load(Ordering::SeqCst) == 1 {
                        break;
                    }
                    assert!(
                        window.cx.executor().tick(),
                        "queued background work must run"
                    );
                }
                assert_eq!(stale.completed.load(Ordering::SeqCst), 1);
                let before = observed.events.borrow().clone();
                assert_eq!(
                    before.last(),
                    Some(&State::Loading),
                    "old foreground callback is still queued"
                );
                let replacement = Gate::default();
                start(api, &owner, &replacement, window);
                replacement.release(Ok(44));
                draw(window);
                let mut expected = before;
                expected.extend([State::Loading, State::Success(44)]);
                assert_eq!(
                    *observed.events.borrow(),
                    expected,
                    "old generation cannot apply success or error"
                );
            }
            assert!(window.update(|_, app| {
                weak.update(app, |host, _| host.tasks.running.is_empty())
                    .unwrap()
            }));
        }
        Check::Error => {
            old.release(Err("failed"));
            draw(window);
            draw(window);
            assert_eq!(
                *observed.events.borrow(),
                vec![State::Loading, State::Error("failed")],
                "error delivered exactly once"
            );
            window.update(|_, app| {
                weak.update(app, |host, cx| {
                    assert_eq!(host.component().state, State::Error("failed"));
                    host.component_mut().unrelated += 1;
                    cx.notify();
                })
                .unwrap()
            });
            draw(window);
            let success = Gate::default();
            start(api, &owner, &success, window);
            assert_eq!(
                window.update(|_, app| weak
                    .update(app, |host, _| host.component().state.clone())
                    .unwrap()),
                State::Loading,
                "new run clears prior error"
            );
            success.release(Ok(7));
            draw(window);
            assert_eq!(
                *observed.events.borrow(),
                vec![
                    State::Loading,
                    State::Error("failed"),
                    State::Loading,
                    State::Success(7)
                ]
            );
            window.update(|_, app| {
                weak.update(app, |host, _| {
                    assert_eq!(
                        host.component().unrelated,
                        1,
                        "view remains usable after error"
                    );
                    assert_eq!(host.component().state, State::Success(7));
                })
                .unwrap()
            });
        }
        Check::OwnerUpdate => {
            let renders = observed.renders.get();
            window.update(|_, app| {
                weak.update(app, |host, cx| {
                    host.component_mut().unrelated += 1;
                    cx.notify();
                })
                .unwrap()
            });
            draw(window);
            assert!(observed.renders.get() > renders, "owner really re-rendered");
            assert_eq!(
                old.started.load(Ordering::SeqCst),
                1,
                "unrelated render must not restart work"
            );
            assert_eq!(old.dropped.load(Ordering::SeqCst), 0);
            assert_eq!(*observed.events.borrow(), vec![State::Loading]);
            let replacement = Gate::default();
            start(api, &owner, &replacement, window);
            draw(window);
            assert_eq!(old.dropped.load(Ordering::SeqCst), 1);
            assert_eq!(replacement.started.load(Ordering::SeqCst), 1);
            old.release(Ok(1));
            replacement.release(Ok(2));
            draw(window);
            assert_eq!(
                *observed.events.borrow(),
                vec![State::Loading, State::Loading, State::Success(2)]
            );
        }
    }
}
macro_rules! cases {
    ($module:ident, $api:expr) => {
        mod $module {
            use super::*;
            #[gpui::test]
            fn unmount(cx: &mut TestAppContext) {
                run(cx, $api, Check::Unmount);
            }
            #[gpui::test]
            fn restart(cx: &mut TestAppContext) {
                run(cx, $api, Check::Restart);
            }
            #[gpui::test]
            fn error(cx: &mut TestAppContext) {
                run(cx, $api, Check::Error);
            }
            #[gpui::test]
            fn owner_update(cx: &mut TestAppContext) {
                run(cx, $api, Check::OwnerUpdate);
            }
            #[gpui::test]
            fn no_leak(cx: &mut TestAppContext) {
                run(cx, $api, Check::NoLeak);
            }
        }
    };
}
cases!(start, Api::Start);
cases!(start_with_executor, Api::WithExecutor);
