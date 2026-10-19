//! Pointer tap region and keyboard-equivalent actions through public RustUI API.

use rustui::prelude::*;

pub fn demo() -> impl View {
    Component("tap-gallery", TapGallery::default)
}

#[component]
struct TapGallery {
    #[state(0)]
    left_hits: usize,
    #[state(0)]
    right_hits: usize,
    #[state(true)]
    show_right: bool,
    #[state(true)]
    taps_enabled: bool,
    #[state(String::from("Left editor"))]
    left: String,
    #[state(String::from("Right editor"))]
    right: String,
}

impl TapGallery {
    fn hit_left(&mut self) {
        self.left_hits += 1;
    }

    fn hit_right(&mut self) {
        self.right_hits += 1;
    }

    fn toggle_right(&mut self) {
        self.show_right = !self.show_right;
    }

    fn toggle_enabled(&mut self) {
        self.taps_enabled = !self.taps_enabled;
    }
}

impl View for TapGallery {
    fn body(&self) -> impl View {
        let left = TapRegion(
            "left-tap",
            Container(
                VStack::new()
                    .spacing(6.0)
                    .child(Text("Left pointer region"))
                    .child(TextField("Left editor", self.left_binding()).id("shared-tap-editor")),
            )
            .width(260.0)
            .padding(8.0)
            .border(),
            self.action(Self::hit_left),
        )
        .enabled(self.taps_enabled());
        let right = self.show_right().then(|| {
            TapRegion(
                "right-tap",
                Container(
                    VStack::new()
                        .spacing(6.0)
                        .child(Text("Right pointer region"))
                        .child(
                            TextField("Right editor", self.right_binding()).id("shared-tap-editor"),
                        ),
                )
                .width(260.0)
                .padding(8.0)
                .border(),
                self.action(Self::hit_right),
            )
            .enabled(self.taps_enabled())
        });

        VStack::new()
            .spacing(8.0)
            .child(Text("TapRegion: native pointer click around arbitrary child views"))
            .child(Text(format!(
                "Left clicks: {} · Right clicks: {} · pointer enabled: {}",
                self.left_hits(),
                self.right_hits(),
                self.taps_enabled()
            )))
            .child(HStack::new().spacing(8.0).child(left).child(right))
            .child(
                HStack::new()
                    .spacing(6.0)
                    .child(Button("Keyboard: left action", self.action(Self::hit_left)).id("tap-keyboard-left"))
                    .child(Button("Keyboard: right action", self.action(Self::hit_right)).id("tap-keyboard-right"))
                    .child(Button("Enable / disable pointer", self.action(Self::toggle_enabled)).id("tap-enabled"))
                    .child(Button("Remove / restore right", self.action(Self::toggle_right)).id("tap-right-toggle")),
            )
            .child(Text("The two editors share a native ID under distinct TapRegion scopes. Removing the right region retains the component owner."))
    }
}
