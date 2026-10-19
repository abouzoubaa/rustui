//! Public Frame modifier examples in the layout gallery.

use rustui::prelude::*;
use rustui::view::EmptyView;

pub fn demo() -> impl View {
    Container(Component("frame-gallery", FrameGallery::default))
        .width(330.0)
        .padding(8.0)
        .border()
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum FrameMode {
    #[default]
    Center,
    Stretch,
    Baseline,
}

impl FrameMode {
    fn next(self) -> Self {
        match self {
            Self::Center => Self::Stretch,
            Self::Stretch => Self::Baseline,
            Self::Baseline => Self::Center,
        }
    }

    fn alignment(self) -> (HorizontalAlignment, VerticalAlignment) {
        match self {
            Self::Center => (HorizontalAlignment::Center, VerticalAlignment::Center),
            Self::Stretch => (HorizontalAlignment::Stretch, VerticalAlignment::Stretch),
            Self::Baseline => (HorizontalAlignment::Center, VerticalAlignment::Baseline),
        }
    }
}

#[component]
struct FrameGallery {
    #[state(0)]
    activations: u32,
    #[state(FrameMode::Center)]
    mode: FrameMode,
    #[state(String::from("Retained Frame editor"))]
    editor: String,
}

impl FrameGallery {
    fn activate(&mut self) {
        self.activations += 1;
    }

    fn cycle_mode(&mut self) {
        self.mode = self.mode.next();
    }
}

impl View for FrameGallery {
    fn body(&self) -> impl View {
        let activations = self.activations();
        let mode = self.mode();
        let editor = self.editor_binding();
        let (horizontal, vertical) = mode.alignment();
        OwnerActionReader(move |target: OwnerAction<FrameGallery>| {
            let sample = |label| {
                Container(Text(label))
                    .width(72.0)
                    .height(28.0)
                    .padding(4.0)
                    .background(Surface::Sidebar)
            };

            VStack::new()
                .spacing(8.0)
                .alignment(HorizontalAlignment::Start)
                .child(Text("Frame · native size and alignment").font_weight(TextWeight::Semibold))
                .child(Text(format!("Editor alignment: {mode:?}")))
                .child(
                    Button(
                        "Center → Stretch → Baseline",
                        target.action(Self::cycle_mode),
                    )
                    .id("frame-cycle-alignment"),
                )
                .child(Text(
                    "Baseline uses one child; there is no peer baseline to compare.",
                ))
                .child(
                    Container(
                        Frame(
                            Container(
                                Frame(
                                    TextField("Retained Frame editor", editor.clone())
                                        .id("frame-retained-editor")
                                        .accessibility_label("Retained Frame editor"),
                                )
                                .width(160.0)
                                .alignment(HorizontalAlignment::Stretch, VerticalAlignment::Center),
                            )
                            .fill()
                            .background(Surface::Sidebar),
                        )
                        .width(240.0)
                        .height(52.0)
                        .alignment(horizontal, vertical),
                    )
                    .border(),
                )
                .child(Text("Fixed 240 × 48 · centered child"))
                .child(Container(Frame(sample("Centered")).width(240.0).height(48.0)).border())
                .child(Text("Logical Start and End · follows active locale"))
                .child(
                    Container(
                        Frame(sample("Start"))
                            .width(240.0)
                            .height(40.0)
                            .alignment(HorizontalAlignment::Start, VerticalAlignment::Center),
                    )
                    .border(),
                )
                .child(
                    Container(
                        Frame(sample("End"))
                            .width(240.0)
                            .height(40.0)
                            .alignment(HorizontalAlignment::End, VerticalAlignment::Center),
                    )
                    .border(),
                )
                .child(Text("Requested width 240 · max width 160"))
                .child(
                    Container(
                        Frame(Text("Native clamp"))
                            .width(240.0)
                            .max_width(160.0)
                            .height(36.0),
                    )
                    .border(),
                )
                .child(Text("Zero-width frame (label sits outside it):"))
                .child(
                    HStack::new()
                        .spacing(4.0)
                        .child(Frame(EmptyView).width(0.0).height(24.0))
                        .child(Text("← width 0")),
                )
                .child(Text("Fixed frame containing a clickable child button"))
                .child(
                    Container(
                        Frame(Button(
                            format!("Click · count: {activations}"),
                            target.action(Self::activate),
                        ))
                        .width(240.0)
                        .height(48.0),
                    )
                    .border(),
                )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alignment_control_cycles_center_stretch_and_baseline() {
        assert_eq!(
            FrameMode::Center.alignment(),
            (HorizontalAlignment::Center, VerticalAlignment::Center)
        );
        assert_eq!(FrameMode::Center.next(), FrameMode::Stretch);
        assert_eq!(
            FrameMode::Stretch.alignment(),
            (HorizontalAlignment::Stretch, VerticalAlignment::Stretch)
        );
        assert_eq!(FrameMode::Stretch.next(), FrameMode::Baseline);
        assert_eq!(
            FrameMode::Baseline.alignment(),
            (HorizontalAlignment::Center, VerticalAlignment::Baseline)
        );
        assert_eq!(FrameMode::Baseline.next(), FrameMode::Center);
    }
}
