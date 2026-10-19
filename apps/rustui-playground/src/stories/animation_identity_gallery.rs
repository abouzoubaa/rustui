//! Native acceptance sample for animation-owned child identity scopes.

use rustui::{component, prelude::*};
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
                    Container(TextField("Spring A", self.spring_a_binding()).id(SHARED_CHILD_ID))
                        .width(240.0),
                    1.0,
                )
                .expect("fixed opacity is valid"),
            )
        } else {
            AnyView::new(Text("Spring A omitted"))
        };
        let fade_a = if self.show_a() {
            AnyView::new(
                AnimatedOpacity::new(
                    "identity-a",
                    Container(TextField("Fade A", self.fade_a_binding()).id(SHARED_CHILD_ID))
                        .width(240.0),
                    0.65,
                    1.0,
                    Duration::from_secs(1),
                )
                .expect("fixed animation parameters are valid"),
            )
        } else {
            AnyView::new(Text("Fade A omitted"))
        };
        VStack::new().spacing(8.0)
            .child(Text("Animation child identity acceptance"))
            .child(Text("Four fields intentionally share one explicit child ID. Each wrapper has its own typed identity scope."))
            .child(HStack::new().spacing(6.0)
                .child(Button("Update parent", self.action(Self::update_parent)).id("animation-identity-parent"))
                .child(Button("Omit / restore A fields", self.action(Self::toggle_a)).id("animation-identity-mount"))
                .child(Button("Replace A values externally", self.action(Self::replace_a)).id("animation-identity-replace")))
            .child(Text(format!("Parent updates {} · A fields visible {}", self.parent_updates(), self.show_a())))
            .child(Text("Spring A / Spring B"))
            .child(HStack::new().spacing(8.0)
                .child(spring_a)
                .child(SpringOpacity::new("identity-b",
                    Container(TextField("Spring B", self.spring_b_binding()).id(SHARED_CHILD_ID))
                        .width(240.0), 1.0).expect("fixed opacity is valid")))
            .child(Text(format!("Spring mirrors: A [{}] · B [{}]", self.spring_a(), self.spring_b())))
            .child(Text("Fade A / Fade B"))
            .child(HStack::new().spacing(8.0)
                .child(fade_a)
                .child(AnimatedOpacity::new("identity-b",
                    Container(TextField("Fade B", self.fade_b_binding()).id(SHARED_CHILD_ID))
                        .width(240.0), 0.65, 1.0, Duration::from_secs(1))
                    .expect("fixed animation parameters are valid")))
            .child(Text(format!("Fade mirrors: A [{}] · B [{}]", self.fade_a(), self.fade_b())))
            .child(Text("Edit each field with a distinct value, then update parent: all mirrors must remain separate. Omit/restore A retains root-bound text; B remains mounted. External replacement changes only A. Animation completion must not reset input state."))
    }
}
