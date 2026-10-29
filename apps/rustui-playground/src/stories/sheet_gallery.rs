//! Trigger-driven side Sheet demonstration using only the public RustUI API.

use rustui::{component, localization::MessageArguments, prelude::*};

pub fn demo() -> impl View {
    Component("sheet-gallery", SheetGallery::default)
}

#[component]
struct SheetGallery {
    #[state(String::from("Owner value"))]
    owner_text: String,
}

impl View for SheetGallery {
    fn body(&self) -> impl View {
        Container(
            VStack::new()
                .spacing(10.0)
                .child(Text(
                    "Side sheet using the upstream default placement and size.",
                ))
                .child(
                    Sheet("playground-details-sheet", "Open sheet")
                        .localized_trigger("lab-sheet-open")
                        .localized_title("lab-sheet-title")
                        .localized_accessibility_label("lab-sheet-name")
                        .content(
                            VStack::new()
                                .spacing(8.0)
                                .child(
                                    TextField("Owner field", self.owner_text_binding())
                                        .id("sheet-owner-field")
                                        .localized_placeholder("lab-sheet-owner-field"),
                                )
                                .child(Component(
                                    "sheet-focus-content",
                                    SheetFocusContent::default,
                                )),
                        ),
                )
                .child(
                    LocalizedText("lab-sheet-owner-value")
                        .with_arguments(MessageArguments::new().with("value", self.owner_text())),
                ),
        )
        .width(560.0)
        .padding(8.0)
        .border()
    }
}

#[component]
struct SheetFocusContent {
    #[state(String::from("Sheet body field"))]
    text: String,
    #[state(false)]
    child_focus_contains_focus: bool,
    #[state(0)]
    activations: usize,
}

impl SheetFocusContent {
    fn activate(&mut self) {
        self.activations += 1;
    }
}

impl View for SheetFocusContent {
    fn body(&self) -> impl View {
        VStack::new()
            .spacing(8.0)
            .child(Text(format!(
                "Sheet child focus scope contains focus: {}",
                self.child_focus_contains_focus()
            )))
            .child(
                FocusScope(
                    "sheet-child-focus-scope",
                    self.child_focus_contains_focus_binding(),
                    VStack::new()
                        .spacing(8.0)
                        .child(
                            TextField("Sheet body field", self.text_binding())
                                .id("sheet-body-field"),
                        )
                        .child(
                            Button(
                                format!("Sheet body action ({})", self.activations()),
                                self.action(Self::activate),
                            )
                            .id("sheet-body-action"),
                        ),
                )
                .tab_stop(false)
                .label("Sheet body controls"),
            )
    }
}
