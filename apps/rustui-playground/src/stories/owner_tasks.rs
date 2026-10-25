//! Deterministic GPUI-owned task states without a service dependency.

use std::{future::Future, time::Duration};

use rustui::prelude::*;

pub fn demo() -> impl View {
    Component("owner-task-demo", TaskDemo::default)
}

fn task_result_status(result: AsyncState<String, String>) -> String {
    match result {
        AsyncState::Idle => "Idle".to_owned(),
        AsyncState::Loading => "Loading".to_owned(),
        AsyncState::Success(value) => format!("Success: {value}"),
        AsyncState::Error(error) => format!("Error: {error}"),
    }
}

#[component]
struct TaskDemo {
    #[state(AsyncState::Idle)]
    result: AsyncState<String, String>,
}

impl View for TaskDemo {
    fn body(&self) -> impl View {
        let status = task_result_status(self.result());

        OwnerTaskReader(move |tasks: OwnerTask<TaskDemo>| {
            VStack::new()
                .child(Text("Owner task: shared request ID"))
                .child(
                    Semantics(
                        "owner-task-status",
                        Text(format!("Current state: {status}")),
                    )
                    .live_region_value(status.clone(), LivePoliteness::Polite),
                )
                .child(
                    Button(
                        "Start slow success",
                        tasks.start_with_executor(
                            "request",
                            |executor| {
                                result_after(
                                    executor.timer(Duration::from_millis(900)),
                                    Ok("original result"),
                                )
                            },
                            TaskDemo::set_result,
                        ),
                    )
                    .id("task-start")
                    .accessibility_label(format!("Start slow success; current state {status}")),
                )
                .child(
                    Button(
                        "Restart with fast success",
                        tasks.start_with_executor(
                            "request",
                            |executor| {
                                result_after(
                                    executor.timer(Duration::from_millis(150)),
                                    Ok("restarted result"),
                                )
                            },
                            TaskDemo::set_result,
                        ),
                    )
                    .id("task-restart")
                    .accessibility_label(format!("Restart request; current state {status}")),
                )
                .child(
                    Button(
                        "Start error",
                        tasks.start_with_executor(
                            "request",
                            |executor| {
                                result_after(
                                    executor.timer(Duration::from_millis(250)),
                                    Err("sample failure"),
                                )
                            },
                            TaskDemo::set_result,
                        ),
                    )
                    .id("task-error")
                    .accessibility_label(format!("Start failing request; current state {status}")),
                )
                .child(
                    Button("Cancel", tasks.cancel("request", TaskDemo::set_result))
                        .id("task-cancel")
                        .accessibility_label(format!("Cancel request; current state {status}")),
                )
                .spacing(8.0)
        })
    }
}

fn result_after(
    delay: impl Future<Output = ()> + Send + 'static,
    result: Result<&'static str, &'static str>,
) -> impl Future<Output = Result<String, String>> + Send {
    async move {
        delay.await;
        result.map(str::to_owned).map_err(str::to_owned)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_task_status_text_covers_idle_loading_success_and_error() {
        assert_eq!(task_result_status(AsyncState::Idle), "Idle");
        assert_eq!(task_result_status(AsyncState::Loading), "Loading");
        assert_eq!(
            task_result_status(AsyncState::Success("saved".into())),
            "Success: saved"
        );
        assert_eq!(
            task_result_status(AsyncState::Error("offline".into())),
            "Error: offline"
        );
    }
}
