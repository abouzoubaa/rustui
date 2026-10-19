//! Basic native vector drawing through public RustUI Canvas commands.

use rustui::{component, prelude::*};

pub fn demo() -> impl View {
    Component("canvas-gallery", CanvasGallery::default)
}

#[component]
struct CanvasGallery {
    #[state(false)]
    shifted: bool,
}

impl CanvasGallery {
    fn toggle_clip(&mut self) {
        self.shifted = !self.shifted;
    }
}

impl View for CanvasGallery {
    fn body(&self) -> impl View {
        let blue = Hsla {
            h: 0.58,
            s: 0.82,
            l: 0.58,
            a: 1.0,
        };
        let amber = Hsla {
            h: 0.11,
            s: 0.9,
            l: 0.58,
            a: 1.0,
        };
        let mut builder = CanvasScene::builder();
        builder
            .fill_rect(
                CanvasRect::new(0.0, 0.0, 320.0, 180.0).unwrap(),
                Hsla {
                    h: 0.62,
                    s: 0.2,
                    l: 0.15,
                    a: 1.0,
                },
            )
            .unwrap();
        let x = if self.shifted() { -30.0 } else { 20.0 };
        builder
            .fill_rect(CanvasRect::new(x, 38.0, 105.0, 60.0).unwrap(), blue)
            .unwrap();
        builder
            .stroke_rect(
                CanvasRect::new(124.0, 34.0, 112.0, 75.0).unwrap(),
                3.0,
                amber,
            )
            .unwrap();
        builder
            .stroke_line(
                CanvasPoint::new(8.0, 155.0).unwrap(),
                CanvasPoint::new(280.0, 15.0).unwrap(),
                5.0,
                amber,
            )
            .unwrap();
        let scene = builder.build();

        VStack::new()
            .spacing(10.0)
            .child(Text("Canvas: rectangles and lines"))
            .child(Button("Move blue rectangle across clip edge", self.action(Self::toggle_clip))
                .id("canvas-toggle-clip"))
            .child(Text(if self.shifted() {
                "Blue rectangle crosses the left edge and is clipped. An amber outline and diagonal line remain visible."
            } else {
                "Blue rectangle sits inside the canvas. An amber outline and diagonal line remain visible."
            }))
            .child(Semantics("canvas-main", Canvas(
                CanvasSize::new(320.0, 180.0).unwrap(), scene.clone(),
            )).role(SemanticsRole::Group)
                .label("Canvas illustration: blue rectangle, amber outline, and diagonal amber line"))
            .child(Text("The same immutable scene in a smaller canvas, clipped to its own bounds:"))
            .child(Semantics("canvas-small", Canvas(
                CanvasSize::new(160.0, 90.0).unwrap(), scene,
            )).role(SemanticsRole::Group)
                .label("Clipped small copy of the canvas illustration"))
    }
}
