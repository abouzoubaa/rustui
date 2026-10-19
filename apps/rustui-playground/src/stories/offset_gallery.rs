//! Signed relative translation with a stationary neighboring layout slot.

use rustui::prelude::*;

pub fn demo() -> impl View {
    Component("offset-gallery", OffsetGallery::default)
}

#[component]
struct OffsetGallery {
    #[state(20.0)]
    x: f32,
    #[state(8.0)]
    y: f32,
    #[state(0)]
    shifted_count: i32,
    #[state(String::from("Shifted input"))]
    shifted_text: String,
}

impl OffsetGallery {
    fn left(&mut self) {
        self.x = (self.x - 12.0).max(-48.0);
    }

    fn right(&mut self) {
        self.x = (self.x + 12.0).min(48.0);
    }

    fn up(&mut self) {
        self.y = (self.y - 12.0).max(-48.0);
    }

    fn down(&mut self) {
        self.y = (self.y + 12.0).min(48.0);
    }

    fn increment_shifted(&mut self) {
        self.shifted_count += 1;
    }

    fn bound_shifted_text(&self) -> Binding<String> {
        Binding::new(
            self.shifted_text(),
            Self::read_shifted_text,
            Self::write_shifted_text,
        )
    }

    fn read_shifted_text(owner: &Self) -> String {
        owner.shifted_text()
    }

    fn write_shifted_text(owner: &mut Self, value: String) {
        owner.shifted_text = value;
    }
}

impl View for OffsetGallery {
    fn body(&self) -> impl View {
        VStack::new()
            .spacing(12.0)
            .child(Text("Offset · signed relative translation"))
            .child(
                HStack::new()
                    .spacing(8.0)
                    .child(Button("X − 12", self.action(Self::left)).id("offset-x-negative"))
                    .child(Button("X + 12", self.action(Self::right)).id("offset-x-positive"))
                    .child(Button("Y − 12", self.action(Self::up)).id("offset-y-negative"))
                    .child(Button("Y + 12", self.action(Self::down)).id("offset-y-positive")),
            )
            .child(Text(format!(
                "Translation: x = {:.0} pt · y = {:.0} pt",
                self.x(),
                self.y()
            )))
            .child(
                HStack::new()
                    .spacing(18.0)
                    .child(
                        Container(
                            VStack::new()
                                .spacing(10.0)
                                .child(Text("Reference slot · unchanged"))
                                .child(
                                    Button("Reference action", AppAction::new(|_| {}))
                                        .id("offset-reference-button"),
                                )
                                .child(Text("The layout slot stays here.")),
                        )
                        .width(230.0)
                        .height(220.0)
                        .padding(12.0)
                        .border(),
                    )
                    .child(
                        Container(
                            Offset(
                                VStack::new()
                                    .spacing(8.0)
                                    .child(
                                        Button(
                                            format!("Shifted action · {}", self.shifted_count()),
                                            self.action(Self::increment_shifted),
                                        )
                                        .id("offset-shifted-button")
                                        .accessibility_label(format!(
                                            "Increment shifted action, count {}",
                                            self.shifted_count()
                                        )),
                                    )
                                    .child(
                                        TextField("Shifted note", self.bound_shifted_text())
                                            .id("offset-shifted-input"),
                                    )
                                    .child(Text("Both controls move; this slot does not.")),
                            )
                            .translation(self.x(), self.y()),
                        )
                        .width(330.0)
                        .height(240.0)
                        .padding(12.0)
                        .border(),
                    ),
            )
    }
}
