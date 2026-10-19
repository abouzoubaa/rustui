//! Native checks for controlled ComboBox selection and option replacement.

use rustui::prelude::*;

pub fn demo() -> impl View {
    Component(
        "combo-box-reconciliation-gallery",
        ComboBoxReconciliationGallery::default,
    )
}

#[component]
struct ComboBoxReconciliationGallery {
    #[state(String::from("GPUI"))]
    selection: String,
    #[state(initial_options())]
    options: Vec<String>,
    #[state(false)]
    disabled: bool,
}

fn initial_options() -> Vec<String> {
    ["GPUI", "AppKit", "Skia"]
        .into_iter()
        .map(String::from)
        .collect()
}

impl ComboBoxReconciliationGallery {
    fn select_skia(&mut self) {
        self.selection = String::from("Skia");
    }

    fn replace_options(&mut self) {
        self.options = ["Skia", "Metal", "CoreGraphics"]
            .into_iter()
            .map(String::from)
            .collect();
    }

    fn remove_selected_option(&mut self) {
        let selected = self.selection.clone();
        self.options.retain(|option| option != &selected);
    }

    fn restore_options(&mut self) {
        self.options = initial_options();
    }

    fn toggle_disabled(&mut self) {
        self.disabled = !self.disabled;
    }
}

impl View for ComboBoxReconciliationGallery {
    fn body(&self) -> impl View {
        VStack::new()
            .spacing(8.0)
            .child(Text(
                "ComboBox · controlled selection and option replacement",
            ))
            .child(
                HStack::new()
                    .spacing(6.0)
                    .child(
                        Button("Select Skia", self.action(Self::select_skia))
                            .id("combo-reconcile-select-skia"),
                    )
                    .child(
                        Button("Replace options", self.action(Self::replace_options))
                            .id("combo-reconcile-replace"),
                    )
                    .child(
                        Button(
                            "Remove selected option",
                            self.action(Self::remove_selected_option),
                        )
                        .id("combo-reconcile-remove"),
                    ),
            )
            .child(
                HStack::new()
                    .spacing(6.0)
                    .child(
                        Button("Restore options", self.action(Self::restore_options))
                            .id("combo-reconcile-restore"),
                    )
                    .child(
                        Button("Toggle disabled", self.action(Self::toggle_disabled))
                            .id("combo-reconcile-disabled"),
                    ),
            )
            .child(Text(format!(
                "Bound selection: {} · options: {} · disabled: {}",
                self.selection(),
                self.options().join(", "),
                self.disabled(),
            )))
            .child(
                ComboBox("Rendering engine", self.selection_binding(), self.options())
                    .id("combo-box-reconciliation-control")
                    .search_placeholder("Find an engine")
                    .disabled(self.disabled()),
            )
    }
}
