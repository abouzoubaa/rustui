//! Deterministic GPUI-owned task states without a service dependency.

use std::{future::Future, time::Duration};

use rustui::prelude::*;

pub fn demo() -> impl View {
    Component("owner-task-demo", TaskDemo::default)
}

#[component]
struct TaskDemo {
    #[state(AsyncState::Idle)]
    result: AsyncState<String, String>,
}

impl View for TaskDemo {
    fn body(&self) -> impl View {
        let status = match self.result() {
            AsyncState::Idle => "Idle".to_owned(),
            AsyncState::Loading => "Loading".to_owned(),
            AsyncState::Success(value) => format!("Success: {value}"),
            AsyncState::Error(error) => format!("Error: {error}"),
        };

        OwnerTaskReader(move |tasks: OwnerTask<TaskDemo>| {
            VStack::new()
                .child(Text("Owner task: shared request ID"))
                .child(Text(format!("Current state: {status}")))
                .child(
                    Button(
                        "Start slow success",
                        tasks.start(
                            "request",
                            || result_after(900, Ok("original result")),
                            TaskDemo::set_result,
                        ),
                    )
                    .id("task-start")
                    .accessibility_label(format!("Start slow success; current state {status}")),
                )
                .child(
                    Button(
                        "Restart with fast success",
                        tasks.start(
                            "request",
                            || result_after(150, Ok("restarted result")),
                            TaskDemo::set_result,
                        ),
                    )
                    .id("task-restart")
                    .accessibility_label(format!("Restart request; current state {status}")),
                )
                .child(
                    Button(
                        "Start error",
                        tasks.start(
                            "request",
                            || result_after(250, Err("sample failure")),
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
    delay_ms: u64,
    result: Result<&'static str, &'static str>,
) -> impl Future<Output = Result<String, String>> + Send {
    async move {
        // This future is polled only by RustUI's GPUI background executor.
        std::thread::sleep(Duration::from_millis(delay_ms));
        result.map(str::to_owned).map_err(str::to_owned)
    }
}
