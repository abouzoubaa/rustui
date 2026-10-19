//! Compact gallery for empty, error, and loading feedback.

use rustui::prelude::*;

pub fn demo() -> impl View {
    Container(Component("feedback-gallery", FeedbackGallery::default))
        .width(650.0)
        .padding(8.0)
        .border()
}

#[component]
struct FeedbackGallery {
    #[state(0)]
    retry_activations: u32,
    #[state(0)]
    toast_body_activations: u32,
    #[state(0)]
    toast_action_activations: u32,
}

impl FeedbackGallery {
    fn retry(&mut self) {
        self.retry_activations += 1;
    }

    fn record_toast_body_activation(&mut self) {
        self.toast_body_activations += 1;
    }

    fn record_toast_action_activation(&mut self) {
        self.toast_action_activations += 1;
    }
}

impl View for FeedbackGallery {
    fn body(&self) -> impl View {
        let retry_activations = self.retry_activations();

        let toast_body_activations = self.toast_body_activations();
        let toast_action_activations = self.toast_action_activations();
        OwnerActionReader(move |target: OwnerAction<FeedbackGallery>| {
            let persistent_action_toast = Toast("Saved changes")
                .title("Saved")
                .kind(ToastKind::Success)
                .id("feedback-action-toast")
                .autohide(false)
                .on_click(target.action(Self::record_toast_body_activation))
                .action("Undo", target.action(Self::record_toast_action_activation))
                .show();

            VStack::new()
                .spacing(10.0)
                .child(Text("Feedback states").font_weight(TextWeight::Semibold))
                .child(
                    Button("Show persistent action toast", persistent_action_toast)
                        .id("feedback-show-toast"),
                )
                .child(Text(format!(
                    "Toast body activations: {toast_body_activations}; action activations: {toast_action_activations}"
                )))
                .child(
                    HStack::new()
                        .spacing(6.0)
                        .child(Button("Info", Toast("Info notice").id("feedback-kind-toast").kind(ToastKind::Info).show()).id("feedback-kind-info"))
                        .child(Button("Success", Toast("Success notice").id("feedback-kind-toast").kind(ToastKind::Success).show()).id("feedback-kind-success"))
                        .child(Button("Warning", Toast("Warning notice").id("feedback-kind-toast").kind(ToastKind::Warning).show()).id("feedback-kind-warning"))
                        .child(Button("Error", Toast("Error notice").id("feedback-kind-toast").kind(ToastKind::Error).show()).id("feedback-kind-error")),
                )
                .child(
                    HStack::new()
                        .spacing(10.0)
                        .child(
                            Container(
                                EmptyState("No saved items")
                                    .detail("Saved items will appear here.")
                                    .id("feedback-empty-state")
                                    .accessibility_label("Saved items are empty"),
                            )
                            .width(196.0)
                            .height(150.0)
                            .padding(10.0)
                            .border(),
                        )
                        .child(
                            Container(
                                VStack::new()
                                    .spacing(8.0)
                                    .child(
                                        ErrorState("Could not load items")
                                            .detail("Check your connection and retry.")
                                            .id("feedback-error-state")
                                            .accessibility_label("Item loading failed"),
                                    )
                                    .child(
                                        Button("Retry", target.action(Self::retry))
                                            .id("feedback-retry-button")
                                            .accessibility_label(format!(
                                                "Retry loading items; activated {retry_activations} times"
                                            )),
                                    )
                                    .child(Text(format!(
                                        "Retry activations: {retry_activations}"
                                    ))),
                            )
                            .width(196.0)
                            .height(150.0)
                            .padding(10.0)
                            .border(),
                        )
                        .child(
                            Container(
                                VStack::new()
                                    .spacing(12.0)
                                    .child(
                                        Skeleton()
                                            .size(144.0, 18.0)
                                            .secondary()
                                            .id("feedback-skeleton-title")
                                            .accessibility_label("Loading item title"),
                                    )
                                    .child(
                                        Skeleton()
                                            .size(172.0, 12.0)
                                            .secondary()
                                            .id("feedback-skeleton-detail")
                                            .accessibility_label("Loading item details"),
                                    ),
                            )
                            .width(196.0)
                            .height(150.0)
                            .padding(10.0)
                            .border(),
                        ),
                )
        })
    }
}
