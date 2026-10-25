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
