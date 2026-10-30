//! Cancellable work owned by an existing GPUI component host.

use std::{collections::HashMap, future::Future};

use gpui::{App, BackgroundExecutor, Task, WeakEntity};
use rustui_core::{Identity, async_state::AsyncState};

use crate::{
    runtime::{AppAction, ComponentHost},
    view::View,
};

struct RunningTask {
    generation: u64,
    task: Option<Task<()>>,
}

/// GPUI task handles retained by one component host. Dropping the host cancels them.
#[derive(Default)]
pub(crate) struct TaskRegistry {
    next_generation: u64,
    running: HashMap<Identity, RunningTask>,
}

impl TaskRegistry {
    fn begin(&mut self, id: Identity) -> u64 {
        self.next_generation = self
            .next_generation
            .checked_add(1)
            .expect("owner task generation exhausted");
        self.running.insert(
            id,
            RunningTask {
                generation: self.next_generation,
                task: None,
            },
        );
        self.next_generation
    }

    fn attach(&mut self, id: &Identity, generation: u64, task: Task<()>) {
        if let Some(entry) = self.running.get_mut(id)
            && entry.generation == generation
        {
            entry.task = Some(task);
        }
    }

    fn cancel(&mut self, id: &Identity) {
        self.running.remove(id);
    }

    fn is_current(&self, id: &Identity, generation: u64) -> bool {
        self.running
            .get(id)
            .is_some_and(|entry| entry.generation == generation)
    }

    fn finish(&mut self, id: &Identity, generation: u64) -> bool {
        if !self.is_current(id, generation) {
            return false;
        }
        self.running.remove(id);
        true
    }
}

/// Starts and cancels tasks for one exact GPUI component owner.
pub struct OwnerTask<C: View> {
    owner: WeakEntity<ComponentHost<C>>,
}

impl<C: View> OwnerTask<C> {
    pub(crate) fn new(owner: WeakEntity<ComponentHost<C>>) -> Self {
        Self { owner }
    }

    /// Restart a named task and report loading, success, or failure to its owner.
    ///
    /// `work` creates a Send future on the UI thread; GPUI polls it on the
    /// background executor. `apply` runs on the UI thread and returns whether
    /// the owner needs notification.
    pub fn start<T, E, F, Fut>(
        &self,
        id: impl Into<Identity>,
        work: F,
        apply: fn(&mut C, AsyncState<T, E>) -> bool,
    ) -> AppAction
    where
        T: Send + 'static,
        E: Send + 'static,
        F: Fn() -> Fut + 'static,
        Fut: Future<Output = Result<T, E>> + Send + 'static,
    {
        self.start_with_executor(id, move |_| work(), apply)
    }

    /// Restart a named task whose work needs the owning GPUI background executor.
    pub fn start_with_executor<T, E, F, Fut>(
        &self,
        id: impl Into<Identity>,
        work: F,
        apply: fn(&mut C, AsyncState<T, E>) -> bool,
    ) -> AppAction
    where
        T: Send + 'static,
        E: Send + 'static,
        F: Fn(&BackgroundExecutor) -> Fut + 'static,
        Fut: Future<Output = Result<T, E>> + Send + 'static,
    {
        let owner = self.owner.clone();
        let id = id.into();
        AppAction::new(move |cx: &mut App| {
            let id = id.clone();
            owner
                .update(cx, |host, cx| {
                    let generation = host.tasks.begin(id.clone());
                    if apply(&mut host.component, AsyncState::Loading) {
                        cx.notify();
                    }

                    let executor = cx.background_executor().clone();
                    let background = cx.background_executor().spawn(work(&executor));
                    let completion_id = id.clone();
                    let task = cx.spawn(async move |weak_host, cx| {
                        let result = background.await;
                        let completed = weak_host
                            .update(cx, |host, cx| {
                                if !host.tasks.is_current(&completion_id, generation) {
                                    return false;
                                }
                                let state = match result {
                                    Ok(value) => AsyncState::Success(value),
                                    Err(error) => AsyncState::Error(error),
                                };
                                if apply(&mut host.component, state) {
                                    cx.notify();
                                }
                                true
                            })
                            .unwrap_or(false);
                        if completed {
                            // Drop this task's registry handle in a later foreground poll,
                            // after its own future has returned from polling.
                            let cleanup_host = weak_host.clone();
                            let cleanup_id = completion_id.clone();
                            cx.spawn(async move |cx| {
                                cleanup_host
                                    .update(cx, |host, _| {
                                        host.tasks.finish(&cleanup_id, generation);
                                    })
                                    .ok();
                            })
                            .detach();
                        }
                    });
                    host.tasks.attach(&id, generation, task);
                })
                .ok();
        })
    }

    /// Cancel a named task and report idle to its owner.
    pub fn cancel<T: 'static, E: 'static>(
        &self,
        id: impl Into<Identity>,
        apply: fn(&mut C, AsyncState<T, E>) -> bool,
    ) -> AppAction {
        let owner = self.owner.clone();
        let id = id.into();
        AppAction::new(move |cx| {
            owner
                .update(cx, |host, cx| {
                    host.tasks.cancel(&id);
                    if apply(&mut host.component, AsyncState::Idle) {
                        cx.notify();
                    }
                })
                .ok();
        })
    }
}

impl<C: View> Clone for OwnerTask<C> {
    fn clone(&self) -> Self {
        Self {
            owner: self.owner.clone(),
        }
    }
}

impl<C: View> PartialEq for OwnerTask<C> {
    fn eq(&self, other: &Self) -> bool {
        self.owner == other.owner
    }
}

impl<C: View> Eq for OwnerTask<C> {}

#[cfg(test)]
mod tests {
    use super::*;

    struct DropProbe(std::sync::Arc<std::sync::atomic::AtomicUsize>);

    impl Drop for DropProbe {
        fn drop(&mut self) {
            self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
    }

    #[derive(Default)]
    struct TaskProbe {
        state: AsyncState<String, String>,
        other_state: AsyncState<String, String>,
    }

    impl View for TaskProbe {
        fn body(&self) -> impl View {
            crate::view::EmptyView
        }
    }

    fn apply_probe(probe: &mut TaskProbe, state: AsyncState<String, String>) -> bool {
        probe.state = state;
        true
    }

    #[gpui::test]
    fn owner_actions_replace_cancel_and_complete_work(cx: &mut gpui::TestAppContext) {
        cx.update(gpui_component::init);
        let (host, window) =
            cx.add_window_view(|_, cx| ComponentHost::new(TaskProbe::default(), cx));
        let owner = OwnerTask::new(host.downgrade());
        let dropped = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let work_dropped = dropped.clone();
        let pending = owner.start(
            "request",
            move || {
                let guard = DropProbe(work_dropped.clone());
                async move {
                    let _guard = guard;
                    std::future::pending::<Result<String, String>>().await
                }
            },
            apply_probe,
        );
        window.update(|window, cx| pending.dispatch_keyboard(window, cx));
        window.cx.run_until_parked();
        assert_eq!(
            window.update(|_, cx| host.read(cx).component().state.clone()),
            AsyncState::Loading
        );

        let replacement = owner.start(
            "request",
            || async { Ok::<_, String>("new".to_owned()) },
            apply_probe,
        );
        window.update(|window, cx| replacement.dispatch_keyboard(window, cx));
        window.cx.run_until_parked();
        assert_eq!(
            window.update(|_, cx| host.read(cx).component().state.clone()),
            AsyncState::Success("new".to_owned())
        );
        assert_eq!(dropped.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert!(window.update(|_, cx| host.read(cx).tasks.running.is_empty()));

        window.update(|window, cx| pending.dispatch_keyboard(window, cx));
        let cancel = owner.cancel("request", apply_probe);
        window.update(|window, cx| cancel.dispatch_keyboard(window, cx));
        window.cx.run_until_parked();
        assert_eq!(
            window.update(|_, cx| host.read(cx).component().state.clone()),
            AsyncState::Idle
        );
        assert_eq!(dropped.load(std::sync::atomic::Ordering::SeqCst), 2);

        let failure = owner.start(
            "request",
            || async { Err::<String, _>("failed".to_owned()) },
            apply_probe,
        );
        window.update(|window, cx| failure.dispatch_keyboard(window, cx));
        window.cx.run_until_parked();
        assert_eq!(
            window.update(|_, cx| host.read(cx).component().state.clone()),
            AsyncState::Error("failed".to_owned())
        );
        assert!(window.update(|_, cx| host.read(cx).tasks.running.is_empty()));
    }

    #[derive(Default)]
    struct RaceProbe {
        events: Vec<AsyncState<String, String>>,
    }

    impl View for RaceProbe {
        fn body(&self) -> impl View {
            crate::view::EmptyView
        }
    }

    fn apply_race(probe: &mut RaceProbe, state: AsyncState<String, String>) -> bool {
        probe.events.push(state);
        true
    }

    #[gpui::test]
    fn completed_old_work_cannot_overwrite_a_replacement(cx: &mut gpui::TestAppContext) {
        check_completed_old_work_cannot_overwrite_a_replacement(cx, false);
    }

    #[gpui::test]
    fn completed_old_error_cannot_overwrite_a_replacement(cx: &mut gpui::TestAppContext) {
        check_completed_old_work_cannot_overwrite_a_replacement(cx, true);
    }

    fn check_completed_old_work_cannot_overwrite_a_replacement(
        cx: &mut gpui::TestAppContext,
        old_fails: bool,
    ) {
        use std::sync::{
            Arc, Mutex,
            atomic::{AtomicBool, Ordering},
        };
        use std::task::{Poll, Waker};

        cx.update(gpui_component::init);
        let (host, window) =
            cx.add_window_view(|_, cx| ComponentHost::new(RaceProbe::default(), cx));
        let owner = OwnerTask::new(host.downgrade());
        let gate = Arc::new(Mutex::new((false, None::<Waker>)));
        let completed = Arc::new(AtomicBool::new(false));
        let work_gate = gate.clone();
        let work_completed = completed.clone();
        let old = owner.start(
            "request",
            move || {
                let gate = work_gate.clone();
                let completed = work_completed.clone();
                async move {
                    std::future::poll_fn(|cx| {
                        let mut state = gate.lock().unwrap();
                        if state.0 {
                            completed.store(true, Ordering::SeqCst);
                            Poll::Ready(if old_fails {
                                Err("old error".to_owned())
                            } else {
                                Ok("old".to_owned())
                            })
                        } else {
                            state.1 = Some(cx.waker().clone());
                            Poll::Pending
                        }
                    })
                    .await
                }
            },
            apply_race,
        );
        window.update(|window, cx| old.dispatch_keyboard(window, cx));
        window.cx.run_until_parked();
        let wake = {
            let mut state = gate.lock().unwrap();
            state.0 = true;
            state.1.take().expect("old task was polled")
        };
        wake.wake();
        // Poll only until the background result exists; do not run the queued
        // foreground completion before the replacement action starts.
        for _ in 0..100 {
            if completed.load(Ordering::SeqCst) {
                break;
            }
            assert!(window.cx.executor().tick(), "old background task must run");
        }
        assert!(completed.load(Ordering::SeqCst));
        assert_eq!(
            window.update(|_, cx| host.read(cx).component().events.clone()),
            vec![AsyncState::Loading]
        );
        let replacement = owner.start(
            "request",
            || async { Ok::<_, String>("new".to_owned()) },
            apply_race,
        );
        window.update(|window, cx| replacement.dispatch_keyboard(window, cx));
        window.cx.run_until_parked();
        assert_eq!(
            window.update(|_, cx| host.read(cx).component().events.clone()),
            vec![
                AsyncState::Loading,
                AsyncState::Loading,
                AsyncState::Success("new".to_owned())
            ]
        );
        assert!(window.update(|_, cx| host.read(cx).tasks.running.is_empty()));
    }

    #[gpui::test]
    fn canceled_completed_work_cannot_deliver_a_stale_result(cx: &mut gpui::TestAppContext) {
        use std::sync::{
            Arc, Mutex,
            atomic::{AtomicBool, Ordering},
        };
        use std::task::{Poll, Waker};

        cx.update(gpui_component::init);
        let (host, window) =
            cx.add_window_view(|_, cx| ComponentHost::new(RaceProbe::default(), cx));
        let owner = OwnerTask::new(host.downgrade());
        let gate = Arc::new(Mutex::new((false, None::<Waker>)));
        let completed = Arc::new(AtomicBool::new(false));
        let work_gate = gate.clone();
        let work_completed = completed.clone();
        let start = owner.start(
            "request",
            move || {
                let gate = work_gate.clone();
                let completed = work_completed.clone();
                async move {
                    std::future::poll_fn(|cx| {
                        let mut state = gate.lock().unwrap();
                        if state.0 {
                            completed.store(true, Ordering::SeqCst);
                            Poll::Ready(Ok::<_, String>("stale".to_owned()))
                        } else {
                            state.1 = Some(cx.waker().clone());
                            Poll::Pending
                        }
                    })
                    .await
                }
            },
            apply_race,
        );
        window.update(|window, cx| start.dispatch_keyboard(window, cx));
        window.cx.run_until_parked();
        let wake = {
            let mut state = gate.lock().unwrap();
            state.0 = true;
            state.1.take().expect("background task was polled")
        };
        wake.wake();
        for _ in 0..100 {
            if completed.load(Ordering::SeqCst) {
                break;
            }
            assert!(window.cx.executor().tick(), "background task must run");
        }
        assert!(completed.load(Ordering::SeqCst));
        assert_eq!(
            window.update(|_, cx| host.read(cx).component().events.clone()),
            vec![AsyncState::Loading]
        );
        let cancel = owner.cancel("request", apply_race);
        window.update(|window, cx| cancel.dispatch_keyboard(window, cx));
        window.cx.run_until_parked();
        assert_eq!(
            window.update(|_, cx| host.read(cx).component().events.clone()),
            vec![AsyncState::Loading, AsyncState::Idle]
        );
        assert!(window.update(|_, cx| host.read(cx).tasks.running.is_empty()));
    }

    #[gpui::test]
    fn canceling_one_owner_task_keeps_another_running(cx: &mut gpui::TestAppContext) {
        use std::sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        };

        fn apply_other(probe: &mut TaskProbe, state: AsyncState<String, String>) -> bool {
            probe.other_state = state;
            true
        }
        cx.update(gpui_component::init);
        let (host, window) =
            cx.add_window_view(|_, cx| ComponentHost::new(TaskProbe::default(), cx));
        let owner = OwnerTask::new(host.downgrade());
        let first_dropped = Arc::new(AtomicUsize::new(0));
        let second_dropped = Arc::new(AtomicUsize::new(0));
        let first_work = first_dropped.clone();
        let second_work = second_dropped.clone();
        let gate = Arc::new(std::sync::Mutex::new((false, None::<std::task::Waker>)));
        let work_gate = gate.clone();
        let first = owner.start(
            "first",
            move || {
                let guard = DropProbe(first_work.clone());
                async move {
                    let _guard = guard;
                    std::future::pending::<Result<String, String>>().await
                }
            },
            apply_probe,
        );
        let second = owner.start(
            "second",
            move || {
                let guard = DropProbe(second_work.clone());
                let gate = work_gate.clone();
                async move {
                    let _guard = guard;
                    std::future::poll_fn(|cx| {
                        let mut state = gate.lock().unwrap();
                        if state.0 {
                            std::task::Poll::Ready(Ok::<_, String>("finished".to_owned()))
                        } else {
                            state.1 = Some(cx.waker().clone());
                            std::task::Poll::Pending
                        }
                    })
                    .await
                }
            },
            apply_other,
        );
        window.update(|window, cx| {
            first.dispatch_keyboard(window, cx);
            second.dispatch_keyboard(window, cx);
        });
        window.cx.run_until_parked();
        let cancel_first = owner.cancel("first", apply_probe);
        window.update(|window, cx| cancel_first.dispatch_keyboard(window, cx));
        window.cx.run_until_parked();
        assert_eq!(first_dropped.load(Ordering::SeqCst), 1);
        assert_eq!(second_dropped.load(Ordering::SeqCst), 0);
        window.update(|_, cx| {
            let host = host.read(cx);
            assert_eq!(host.component().state, AsyncState::Idle);
            assert_eq!(host.component().other_state, AsyncState::Loading);
            assert_eq!(host.tasks.running.len(), 1);
            assert!(host.tasks.running.contains_key(&Identity::from("second")));
        });
        let wake = {
            let mut state = gate.lock().unwrap();
            state.0 = true;
            state.1.take().expect("second task was polled")
        };
        wake.wake();
        window.cx.run_until_parked();
        assert_eq!(second_dropped.load(Ordering::SeqCst), 1);
        window.update(|_, cx| {
            let host = host.read(cx);
            assert_eq!(host.component().state, AsyncState::Idle);
            assert_eq!(
                host.component().other_state,
                AsyncState::Success("finished".to_owned())
            );
            assert!(host.tasks.running.is_empty());
        });
    }

    #[gpui::test]
    fn releasing_owner_cancels_pending_work(cx: &mut gpui::TestAppContext) {
        use gpui::AppContext as _;
        use std::sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        };

        cx.update(gpui_component::init);
        let (_root, window) =
            cx.add_window_view(|_, cx| ComponentHost::new(TaskProbe::default(), cx));
        let host = window.update(|_, cx| cx.new(|cx| ComponentHost::new(TaskProbe::default(), cx)));
        let weak = host.downgrade();
        let owner = OwnerTask::new(weak.clone());
        let dropped = Arc::new(AtomicUsize::new(0));
        let work_dropped = dropped.clone();
        let pending = owner.start(
            "release",
            move || {
                let guard = DropProbe(work_dropped.clone());
                async move {
                    let _guard = guard;
                    std::future::pending::<Result<String, String>>().await
                }
            },
            apply_probe,
        );
        window.update(|window, cx| pending.dispatch_keyboard(window, cx));
        window.cx.run_until_parked();
        assert_eq!(dropped.load(Ordering::SeqCst), 0);
        assert!(weak.upgrade().is_some());

        drop(host);
        window.update(|_, _| {});
        window.cx.run_until_parked();
        assert!(weak.upgrade().is_none());
        assert_eq!(dropped.load(Ordering::SeqCst), 1);

        // A retained action must not resurrect or update its released owner.
        window.update(|window, cx| pending.dispatch_keyboard(window, cx));
        window.cx.run_until_parked();
        assert!(weak.upgrade().is_none());
        assert_eq!(dropped.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn replacement_and_cancel_invalidate_old_generations() {
        let mut tasks = TaskRegistry::default();
        let id = Identity::from("load");
        let other_id = Identity::from("other");
        let first = tasks.begin(id.clone());
        let other = tasks.begin(other_id.clone());
        let second = tasks.begin(id.clone());
        assert!(!tasks.is_current(&id, first));
        assert!(tasks.is_current(&id, second));
        tasks.cancel(&id);
        assert!(!tasks.is_current(&id, second));
        assert!(tasks.is_current(&other_id, other));
        let third = tasks.begin(id.clone());
        assert!(third > second);
        assert!(!tasks.is_current(&id, second));
    }

    #[test]
    fn terminal_cleanup_only_removes_the_matching_generation() {
        let mut tasks = TaskRegistry::default();
        let id = Identity::from("load");
        let old = tasks.begin(id.clone());
        let current = tasks.begin(id.clone());
        assert!(!tasks.finish(&id, old));
        assert!(tasks.is_current(&id, current));
        assert!(tasks.finish(&id, current));
        assert!(!tasks.is_current(&id, current));
        let next = tasks.begin(id.clone());
        assert!(next > current);
        assert!(!tasks.finish(&id, current));
        assert!(tasks.is_current(&id, next));
    }
}
