//! Deterministic GPUI-owned task states without a service dependency.

use std::{future::Future, time::Duration};

use rustui::localization::MessageArguments;
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

fn localized_status(result: &AsyncState<String, String>) -> (&'static str, MessageArguments) {
    let args = MessageArguments::new();
    match result {
        AsyncState::Idle => ("lab-task-status-idle", args),
        AsyncState::Loading => ("lab-task-status-loading", args),
        AsyncState::Success(value) if value == "original result" => {
            ("lab-task-status-original", args)
        }
        AsyncState::Success(value) if value == "restarted result" => {
            ("lab-task-status-restarted", args)
        }
        AsyncState::Error(error) if error == "sample failure" => ("lab-task-status-failure", args),
        AsyncState::Success(value) => {
            ("lab-task-status-success", args.with("value", value.clone()))
        }
        AsyncState::Error(error) => ("lab-task-status-error", args.with("error", error.clone())),
    }
}

#[component]
struct TaskDemo {
    #[state(AsyncState::Idle)]
    result: AsyncState<String, String>,
}

impl View for TaskDemo {
    fn body(&self) -> impl View {
        let result = self.result();
        let (status_key, status_args) = localized_status(&result);
        let status = task_result_status(result);

        OwnerTaskReader(move |tasks: OwnerTask<TaskDemo>| {
            VStack::new()
                .child(LocalizedText("lab-task-title"))
                .child(
                    Semantics(
                        "owner-task-status",
                        LocalizedText(status_key).with_arguments(status_args.clone()),
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
                    .localized_label("lab-task-start")
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
                    .localized_label("lab-task-restart")
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
                    .localized_label("lab-task-error")
                    .accessibility_label(format!("Start failing request; current state {status}")),
                )
                .child(
                    Button("Cancel", tasks.cancel("request", TaskDemo::set_result))
                        .id("task-cancel")
                        .localized_label("lab-task-cancel")
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
    fn owner_task_states_have_messages_for_all_runtime_locales() {
        let cases = [
            (AsyncState::Idle, "lab-task-status-idle"),
            (AsyncState::Loading, "lab-task-status-loading"),
            (
                AsyncState::Success("original result".into()),
                "lab-task-status-original",
            ),
            (
                AsyncState::Success("restarted result".into()),
                "lab-task-status-restarted",
            ),
            (
                AsyncState::Error("sample failure".into()),
                "lab-task-status-failure",
            ),
            (
                AsyncState::Success("custom".into()),
                "lab-task-status-success",
            ),
            (AsyncState::Error("custom".into()), "lab-task-status-error"),
        ];
        let catalogs = super::super::locale_selector::catalogs();
        for (state, expected_key) in cases {
            let (key, _) = localized_status(&state);
            assert_eq!(key, expected_key);
            for language in ["en", "fr", "ar"] {
                let locale = rustui::localization::Locale::parse(language).unwrap();
                assert!(catalogs.get(&locale).unwrap().get(key).is_some());
            }
        }
    }

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
