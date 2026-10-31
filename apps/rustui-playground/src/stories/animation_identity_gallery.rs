//! Native acceptance sample for animation-owned child identity scopes.

use rustui::{component, localization::MessageArguments, prelude::*};
use std::time::Duration;

const SHARED_CHILD_ID: &str = "shared-animation-input";

pub fn demo() -> impl View {
    Component(
        "animation-identity-gallery",
        AnimationIdentityGallery::default,
    )
}

#[component]
struct AnimationIdentityGallery {
    #[state(String::from("Spring A"))]
    spring_a: String,
    #[state(String::from("Spring B"))]
    spring_b: String,
    #[state(String::from("Fade A"))]
    fade_a: String,
    #[state(String::from("Fade B"))]
    fade_b: String,
    #[state(true)]
    show_a: bool,
    #[state(0)]
    parent_updates: usize,
}

impl AnimationIdentityGallery {
    fn update_parent(&mut self) {
        self.parent_updates += 1;
    }
    fn toggle_a(&mut self) {
        self.show_a = !self.show_a;
    }
    fn replace_a(&mut self) {
        self.spring_a = format!("Spring A external {}", self.parent_updates);
        self.fade_a = format!("Fade A external {}", self.parent_updates);
    }
}

impl View for AnimationIdentityGallery {
    fn body(&self) -> impl View {
        // Containers establish layout only: no Component/Keyed boundary masks
        // the animation wrapper's responsibility for the shared child ID.
        let spring_a = if self.show_a() {
            AnyView::new(
                SpringOpacity::new(
                    "identity-a",
                    Container(
                        TextField("Spring A", self.spring_a_binding())
                            .id(SHARED_CHILD_ID)
                            .localized_placeholder("lab-identity-spring-a"),
                    )
                    .width(240.0),
                    1.0,
                )
                .expect("fixed opacity is valid"),
            )
        } else {
            AnyView::new(LocalizedText("lab-identity-spring-omitted"))
        };
        let fade_a = if self.show_a() {
            AnyView::new(
                AnimatedOpacity::new(
                    "identity-a",
                    Container(
                        TextField("Fade A", self.fade_a_binding())
                            .id(SHARED_CHILD_ID)
                            .localized_placeholder("lab-identity-fade-a"),
                    )
                    .width(240.0),
                    0.65,
                    1.0,
                    Duration::from_secs(1),
                )
                .expect("fixed animation parameters are valid"),
            )
        } else {
            AnyView::new(LocalizedText("lab-identity-fade-omitted"))
        };
        VStack::new()
            .spacing(8.0)
            .child(LocalizedText("lab-identity-title"))
            .child(LocalizedText("lab-identity-description"))
            .child(
                HStack::new()
                    .spacing(6.0)
                    .child(
                        Button("Update parent", self.action(Self::update_parent))
                            .id("animation-identity-parent")
                            .localized_label("lab-identity-update"),
                    )
                    .child(
                        Button("Omit / restore A fields", self.action(Self::toggle_a))
                            .id("animation-identity-mount")
                            .localized_label("lab-identity-mount"),
                    )
                    .child(
                        Button("Replace A values externally", self.action(Self::replace_a))
                            .id("animation-identity-replace")
                            .localized_label("lab-identity-replace"),
                    ),
            )
            .child(
                LocalizedText(if self.show_a() {
                    "lab-identity-visible"
                } else {
                    "lab-identity-hidden"
                })
                .with_arguments(
                    MessageArguments::new().with("count", self.parent_updates() as i64),
                ),
            )
            .child(LocalizedText("lab-identity-spring-fields"))
            .child(
                HStack::new().spacing(8.0).child(spring_a).child(
                    SpringOpacity::new(
                        "identity-b",
                        Container(
                            TextField("Spring B", self.spring_b_binding())
                                .id(SHARED_CHILD_ID)
                                .localized_placeholder("lab-identity-spring-b"),
                        )
                        .width(240.0),
                        1.0,
                    )
                    .expect("fixed opacity is valid"),
                ),
            )
            .child(
                LocalizedText("lab-identity-spring-mirrors").with_arguments(
                    MessageArguments::new()
                        .with("a", self.spring_a())
                        .with("b", self.spring_b()),
                ),
            )
            .child(LocalizedText("lab-identity-fade-fields"))
            .child(
                HStack::new().spacing(8.0).child(fade_a).child(
                    AnimatedOpacity::new(
                        "identity-b",
                        Container(
                            TextField("Fade B", self.fade_b_binding())
                                .id(SHARED_CHILD_ID)
                                .localized_placeholder("lab-identity-fade-b"),
                        )
                        .width(240.0),
                        0.65,
                        1.0,
                        Duration::from_secs(1),
                    )
                    .expect("fixed animation parameters are valid"),
                ),
            )
            .child(
                LocalizedText("lab-identity-fade-mirrors").with_arguments(
                    MessageArguments::new()
                        .with("a", self.fade_a())
                        .with("b", self.fade_b()),
                ),
            )
            .child(LocalizedText("lab-identity-help"))
    }
}
