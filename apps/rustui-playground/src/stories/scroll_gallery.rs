//! Native axis, scrollbar, retained-child and bounds acceptance sample.

use rustui::{component, prelude::*};

pub fn demo() -> impl View {
    Component("scroll-gallery", ScrollGallery::default)
}

#[component]
struct ScrollGallery {
    #[state(true)]
    bars: bool,
    #[state(false)]
    compact: bool,
    #[state(true)]
    visible: bool,
    #[state(0)]
    updates: usize,
    #[state(0)]
    root_hits: usize,
    #[state(ScrollAxis::Both)]
    plane_axis: ScrollAxis,
    #[state(String::from("Vertical value"))]
    vertical_text: String,
    #[state(String::from("Horizontal value"))]
    horizontal_text: String,
    #[state(String::from("Both value"))]
    both_text: String,
}

impl ScrollGallery {
    fn toggle_bars(&mut self) {
        self.bars = !self.bars;
    }
    fn toggle_content(&mut self) {
        self.compact = !self.compact;
    }
    fn toggle_visible(&mut self) {
        self.visible = !self.visible;
    }
    fn update_parent(&mut self) {
        self.updates += 1;
    }
    fn root_hit(&mut self) {
        self.root_hits += 1;
    }
    fn cycle_axis(&mut self) {
        self.plane_axis = match self.plane_axis {
            ScrollAxis::Both => ScrollAxis::Vertical,
            ScrollAxis::Vertical => ScrollAxis::Horizontal,
            ScrollAxis::Horizontal => ScrollAxis::Both,
        };
    }

    fn vertical_pane(&self) -> impl View {
        let mut rows = VStack::new()
            .spacing(4.0)
            .child(LocalizedText("rustui-action-close"))
            .child(
                TextField("Vertical binding", self.vertical_text_binding())
                    .id("shared-scroll-input"),
            )
            .child(Button("Vertical owner action", self.action(Self::root_hit)).id("owner-action"));
        // Orthogonal nested viewport uses the same native gesture facilities.
        let mut nested = HStack::new().spacing(4.0);
        for index in 0..8 {
            nested = nested.child(
                Container(Text(format!("Nested {index}")))
                    .width(100.0)
                    .height(50.0)
                    .border(),
            );
        }
        rows = rows.child(
            ScrollView(nested)
                .id("nested-horizontal")
                .axis(ScrollAxis::Horizontal)
                .width(480.0)
                .height(62.0)
                .scrollbars(self.bars()),
        );
        for index in 0..if self.compact() { 1 } else { 18 } {
            rows = rows.child(
                Container(Text(format!("Vertical row {index:02}")))
                    .height(28.0)
                    .border(),
            );
        }
        ScrollView(rows)
            .id("vertical-pane")
            .width(520.0)
            .height(200.0)
            .scrollbars(self.bars())
    }

    fn horizontal_pane(&self) -> impl View {
        let mut cards = HStack::new().spacing(6.0).child(
            Container(
                VStack::new()
                    .spacing(4.0)
                    .child(
                        TextField("Horizontal binding", self.horizontal_text_binding())
                            .id("shared-scroll-input"),
                    )
                    .child(
                        Button("Horizontal owner action", self.action(Self::root_hit))
                            .id("owner-action"),
                    ),
            )
            .width(220.0)
            .height(120.0)
            .border(),
        );
        for index in 1..if self.compact() { 2 } else { 9 } {
            cards = cards.child(
                Container(Text(format!("Horizontal card {index}")))
                    .width(150.0)
                    .height(120.0)
                    .border(),
            );
        }
        ScrollView(cards)
            .id("horizontal-pane")
            .axis(ScrollAxis::Horizontal)
            .width(520.0)
            .height(150.0)
            .scrollbars(self.bars())
    }

    fn both_pane(&self) -> impl View {
        let axis = self.plane_axis();
        let (width, height) = if self.compact() {
            (400.0, 140.0)
        } else {
            (
                if axis == ScrollAxis::Vertical {
                    500.0
                } else {
                    1100.0
                },
                if axis == ScrollAxis::Horizontal {
                    170.0
                } else {
                    560.0
                },
            )
        };
        let mut board = VStack::new()
            .spacing(6.0)
            .child(Text(
                "Plane top-left: horizontal and vertical positions are physical native coordinates",
            ))
            .child(TextField("Both binding", self.both_text_binding()).id("shared-scroll-input"))
            .child(Button("Plane owner action", self.action(Self::root_hit)).id("owner-action"));
        for index in 0..if self.compact() { 0 } else { 10 } {
            board = board.child(
                HStack::new()
                    .spacing(12.0)
                    .child(
                        Container(Text(format!("Plane row {index} left")))
                            .width(200.0)
                            .height(30.0)
                            .border(),
                    )
                    .child(
                        Container(Text(format!("Plane row {index} right")))
                            .width(width - 220.0)
                            .height(30.0)
                            .border(),
                    ),
            );
        }
        ScrollView(Container(board).width(width).height(height))
            .id("plane-pane")
            .axis(axis)
            .width(520.0)
            .height(200.0)
            .scrollbars(self.bars())
    }
}

impl View for ScrollGallery {
    fn body(&self) -> impl View {
        let panes = if self.visible() {
            AnyView::new(
                VStack::new()
                    .spacing(8.0)
                    .child(Text("Vertical, with a nested horizontal viewport"))
                    .child(self.vertical_pane())
                    .child(Text("Horizontal"))
                    .child(self.horizontal_pane())
                    .child(Text(format!(
                        "Plane axis {:?} (Both uses native dominant-axis deltas)",
                        self.plane_axis()
                    )))
                    .child(self.both_pane()),
            )
        } else {
            AnyView::new(Text(
                "Scroll panes omitted; restore starts fresh native positions with retained root-bound text.",
            ))
        };
        VStack::new().spacing(8.0)
            .child(Text("ScrollView axes and native overlay bars"))
            .child(super::locale_selector::LocaleSelector::new())
            .child(HStack::new().spacing(6.0)
                .child(Button("Toggle scrollbars", self.action(Self::toggle_bars)).id("scroll-bars"))
                .child(Button("Shrink / expand content", self.action(Self::toggle_content)).id("scroll-content"))
                .child(Button("Hide / restore panes", self.action(Self::toggle_visible)).id("scroll-visible")))
            .child(HStack::new().spacing(6.0)
                .child(Button("Update parent", self.action(Self::update_parent)).id("scroll-parent"))
                .child(Button("Cycle plane axis", self.action(Self::cycle_axis)).id("scroll-axis")))
            .child(Text(format!("Bars {} · compact {} · parent updates {} · owner actions {}", self.bars(), self.compact(), self.updates(), self.root_hits())))
            .child(Text(format!("Binding mirrors: vertical [{}] · horizontal [{}] · both [{}]", self.vertical_text(), self.horizontal_text(), self.both_text())))
            .child(Text("Scroll each pane independently. Toggle bars/update parent without resetting position. Shrink clamps native offsets; expand keeps the clamped position. Inputs intentionally share child IDs across distinct viewport scopes. Axis changes clear disabled coordinates. Locale changes child direction without synthesizing mirrored scroll offsets."))
            .child(panes)
    }
}
