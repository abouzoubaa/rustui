//! Native shadow ownership, theme elevation and explicit geometry sample.

use rustui::{component, prelude::*};

pub fn demo() -> impl View {
    Component("shadow-gallery", ShadowGallery::default)
}

#[component]
struct ShadowGallery {
    #[state(false)]
    custom: bool,
    #[state(false)]
    shifted: bool,
    #[state(ShadowElevation::Medium)]
    elevation: ShadowElevation,
    #[state(true)]
    show_a: bool,
    #[state(0)]
    parent_updates: usize,
    #[state(0)]
    owner_hits: usize,
    #[state(String::from("Shadow A"))]
    first: String,
    #[state(String::from("Shadow B"))]
    second: String,
}

impl ShadowGallery {
    fn toggle_source(&mut self) {
        self.custom = !self.custom;
    }
    fn toggle_shape(&mut self) {
        self.shifted = !self.shifted;
        self.custom = true;
    }
    fn cycle_elevation(&mut self) {
        self.custom = false;
        self.elevation = match self.elevation {
            ShadowElevation::Small => ShadowElevation::Medium,
            ShadowElevation::Medium => ShadowElevation::Large,
            ShadowElevation::Large => ShadowElevation::Small,
        };
    }
    fn toggle_a(&mut self) {
        self.show_a = !self.show_a;
    }
    fn update_parent(&mut self) {
        self.parent_updates += 1;
    }
    fn hit(&mut self) {
        self.owner_hits += 1;
    }
}

fn explicit_style(shifted: bool) -> ShadowStyle {
    ShadowStyle::new(
        if shifted { -6.0 } else { 4.0 },
        if shifted { 10.0 } else { 4.0 },
        if shifted { 18.0 } else { 8.0 },
        -2.0,
        Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.0,
            a: 0.35,
        },
    )
    .expect("fixed shadow style is valid")
}

impl View for ShadowGallery {
    fn body(&self) -> impl View {
        let first = if self.show_a() {
            // No nested Keyed/Component/decoration scope hides Shadow's own
            // responsibility for isolating the identical input IDs.
            let shadow = Shadow(
                "shadow-a",
                Container(TextField("Shadow A", self.first_binding()).id("shadow-shared-input"))
                    .width(240.0)
                    .padding(12.0)
                    .background(Surface::Panel)
                    .border()
                    .radius(10.0),
            )
            .radius(10.0);
            AnyView::new(if self.custom() {
                shadow.style(explicit_style(self.shifted()))
            } else {
                shadow.elevation(self.elevation())
            })
        } else {
            AnyView::new(Text("Shadow A omitted"))
        };
        let samples = HStack::new()
            .spacing(24.0)
            .child(
                Shadow(
                    "small",
                    Background(
                        "small",
                        Container(Text("Theme Small")).padding(12.0),
                        Surface::Panel,
                    )
                    .radius(8.0),
                )
                .elevation(ShadowElevation::Small)
                .radius(8.0),
            )
            .child(
                Shadow(
                    "medium",
                    Background(
                        "medium",
                        Container(Text("Theme Medium")).padding(12.0),
                        Surface::Panel,
                    )
                    .radius(8.0),
                )
                .radius(8.0),
            )
            .child(
                Shadow(
                    "large",
                    Background(
                        "large",
                        Container(Text("Theme Large")).padding(12.0),
                        Surface::Panel,
                    )
                    .radius(8.0),
                )
                .elevation(ShadowElevation::Large)
                .radius(8.0),
            );
        VStack::new().spacing(10.0)
            .child(Text("Shadow modifier: native elevation and explicit outer shadow"))
            .child(super::locale_selector::LocaleSelector::new())
            .child(HStack::new().spacing(6.0)
                .child(Button("Theme / explicit", self.action(Self::toggle_source)).id("shadow-source"))
                .child(Button("Change offsets / blur", self.action(Self::toggle_shape)).id("shadow-shape"))
                .child(Button("Cycle theme elevation", self.action(Self::cycle_elevation)).id("shadow-elevation")))
            .child(HStack::new().spacing(6.0)
                .child(Button("Update parent", self.action(Self::update_parent)).id("shadow-parent"))
                .child(Button("Omit / restore A", self.action(Self::toggle_a)).id("shadow-mount")))
            .child(Text(format!("Source {} · elevation {:?} · parent updates {} · owner hits {}",
                if self.custom() { "explicit" } else { "theme" }, self.elevation(), self.parent_updates(), self.owner_hits())))
            .child(Text("Theme elevations follow the current Playground appearance and shadow enable policy. Explicit black shadow keeps its supplied color. Offsets use physical coordinates in either locale direction."))
            .child(Container(samples).padding(24.0))
            .child(Container(HStack::new().spacing(28.0)
                .child(first)
                .child(Shadow("shadow-b", Container(TextField("Shadow B", self.second_binding()).id("shadow-shared-input"))
                    .width(240.0).padding(12.0).background(Surface::Panel).border().radius(10.0))
                    .style(explicit_style(false)).radius(10.0))).padding(24.0))
            .child(Text(format!("Independent mirrors: A [{}] · B [{}]", self.first(), self.second())))
            .child(Container(Shadow("nested", Border("nested", Background("nested",
                Container(VStack::new().spacing(6.0)
                    .child(LocalizedText("rustui-action-close"))
                    .child(Button("Shadowed owner action", self.action(Self::hit)).id("shadow-owner-action")))
                    .padding_edges(LogicalInsets::new(10.0, 10.0, 24.0, 8.0).expect("fixed insets are valid")), Surface::Panel)
                    .radius(10.0)).radius(10.0).color(BorderColor::Accent))
                .style(explicit_style(self.shifted())).radius(10.0)).padding(24.0))
            .child(Text("Edit both fields, then change style/source/parent/locale: state must stay independent. Omit/restore A retains its root string. No clipping is added; surrounding padding merely leaves room to see native paint outside each box."))
    }
}
