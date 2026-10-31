//! Demonstrates a task completing after its conditional child owner is released.

use std::{
    future::Future,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use rustui::prelude::*;

pub fn demo() -> impl View {
    Component("task-release-demo", TaskReleaseDemo::new)
}

#[derive(Default)]
struct TaskReleaseProbe {
    background_started: AtomicUsize,
    background_finished: AtomicUsize,
    completion_delivered: AtomicUsize,
    mounts: AtomicUsize,
    unmounts: AtomicUsize,
}

#[component]
struct TaskReleaseDemo {
    #[state(true)]
    child_visible: bool,
    #[state(0)]
    observations: usize,
    probe: Arc<TaskReleaseProbe>,
}

impl TaskReleaseDemo {
    fn new() -> Self {
        Self {
            probe: Arc::default(),
            ..Self::default()
        }
    }

    fn hide_child(&mut self) {
        self.child_visible = false;
    }

    fn restore_child(&mut self) {
        self.child_visible = true;
    }

    fn observe(&mut self) {
        self.observations += 1;
    }
}

impl View for TaskReleaseDemo {
    fn body(&self) -> impl View {
        let child_probe = self.probe.clone();
        let visible = self.child_visible();
        VStack! {
            Text("Owner release while work is pending")
            Text("Start the child task, remove the child, wait 3 seconds, then observe.")
            HStack! {
                Button("Remove child", self.action(Self::hide_child))
                    .id("task-release-remove")
                    .disabled(!visible)
                Button("Restore child", self.action(Self::restore_child))
                    .id("task-release-restore")
                    .disabled(visible)
                Button("Observe counters", self.action(Self::observe))
                    .id("task-release-observe")
            }
            Text(format!(
                "Background started: {} · results: {} · delivered to owners: {}",
                self.probe.background_started.load(Ordering::SeqCst),
                self.probe.background_finished.load(Ordering::SeqCst),
                self.probe.completion_delivered.load(Ordering::SeqCst),
            ))
            Text(format!(
                "Child owner mounts: {} · releases: {} · observations: {}",
                self.probe.mounts.load(Ordering::SeqCst),
                self.probe.unmounts.load(Ordering::SeqCst),
                self.observations(),
            ))
            visible.then(|| {
                Component("pending-task-child", move || {
                    PendingTaskChild::new(child_probe.clone())
                })
            })
        }
        .spacing(8.0)
    }
}

#[component]
struct PendingTaskChild {
    #[state(AsyncState::Idle)]
    result: AsyncState<String, String>,
    probe: Arc<TaskReleaseProbe>,
}

impl PendingTaskChild {
    fn new(probe: Arc<TaskReleaseProbe>) -> Self {
        Self {
            probe,
            ..Self::default()
        }
    }

    fn apply_result(&mut self, result: AsyncState<String, String>) -> bool {
        if matches!(&result, AsyncState::Success(_) | AsyncState::Error(_)) {
            self.probe
                .completion_delivered
                .fetch_add(1, Ordering::SeqCst);
        }
        self.set_result(result);
        true
    }
}

impl View for PendingTaskChild {
    fn body(&self) -> impl View {
        let result = match self.result() {
            AsyncState::Idle => "Idle".to_owned(),
            AsyncState::Loading => "Loading".to_owned(),
            AsyncState::Success(value) => format!("Success: {value}"),
            AsyncState::Error(error) => format!("Error: {error}"),
        };
        let probe = self.probe.clone();
        let can_start = matches!(self.result(), AsyncState::Idle);

        OwnerTaskReader(move |tasks: OwnerTask<PendingTaskChild>| {
            #[cfg(not(target_arch = "wasm32"))]
            let native_work = {
                let probe = probe.clone();
                Some(
                    Button(
                        "Start non-cancellable native work",
                        tasks.start(
                            "pending-release",
                            move || native_result(probe.clone(), Duration::from_secs(3)),
                            PendingTaskChild::apply_result,
                        ),
                    )
                    .id("task-release-native-start")
                    .disabled(!can_start),
                )
            };
            #[cfg(target_arch = "wasm32")]
            let native_work = None::<rustui::view::EmptyView>;
            VStack::new()
                .child(Text(format!("Child task state: {result}")))
                .child(
                    Button(
                        "Start 3 second task",
                        tasks.start_with_executor(
                            "pending-release",
                            {
                                let task_probe = probe.clone();
                                move |executor| {
                                    delayed_result(
                                        task_probe.clone(),
                                        executor.timer(Duration::from_secs(3)),
                                    )
                                }
                            },
                            PendingTaskChild::apply_result,
                        ),
                    )
                    .id("task-release-start")
                    .disabled(!can_start),
                )
                .child(native_work)
                .spacing(8.0)
        })
    }

    fn on_mount(&mut self, _: &mut rustui::App) {
        self.probe.mounts.fetch_add(1, Ordering::SeqCst);
    }

    fn on_unmount(&mut self, _: &mut rustui::App) {
        self.probe.unmounts.fetch_add(1, Ordering::SeqCst);
    }
}

fn delayed_result(
    probe: Arc<TaskReleaseProbe>,
    delay: impl Future<Output = ()> + Send + 'static,
) -> impl Future<Output = Result<String, String>> + Send {
    async move {
        probe.background_started.fetch_add(1, Ordering::SeqCst);
        delay.await;
        probe.background_finished.fetch_add(1, Ordering::SeqCst);
        Ok(String::from("finished after child removal"))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn native_result(
    probe: Arc<TaskReleaseProbe>,
    delay: Duration,
) -> impl Future<Output = Result<String, String>> + Send {
    async move {
        probe.background_started.fetch_add(1, Ordering::SeqCst);
        // A bounded acceptance probe on GPUI's background executor. Unlike the
        // timer case, dropping the task cannot interrupt this in-flight poll.
        std::thread::sleep(delay);
        probe.background_finished.fetch_add(1, Ordering::SeqCst);
        Ok(String::from("native work finished after child removal"))
    }
}

#[cfg(test)]
mod tests {
    use std::task::{Context, Poll, Waker};

    use super::*;

    #[test]
    fn delayed_result_yields_while_its_executor_delay_is_pending() {
        let probe = Arc::new(TaskReleaseProbe::default());
        let mut result = Box::pin(delayed_result(probe.clone(), std::future::pending()));
        let waker = Waker::noop();
        let mut context = Context::from_waker(waker);

        assert!(matches!(result.as_mut().poll(&mut context), Poll::Pending));
        assert_eq!(probe.background_started.load(Ordering::SeqCst), 1);
        assert_eq!(probe.background_finished.load(Ordering::SeqCst), 0);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_probe_records_work_without_claiming_owner_delivery() {
        let probe = Arc::new(TaskReleaseProbe::default());
        let mut result = Box::pin(native_result(probe.clone(), Duration::ZERO));
        let waker = Waker::noop();
        let mut context = Context::from_waker(waker);
        assert!(matches!(
            result.as_mut().poll(&mut context),
            Poll::Ready(Ok(_))
        ));
        assert_eq!(probe.background_started.load(Ordering::SeqCst), 1);
        assert_eq!(probe.background_finished.load(Ordering::SeqCst), 1);
        assert_eq!(probe.completion_delivered.load(Ordering::SeqCst), 0);
    }
}
