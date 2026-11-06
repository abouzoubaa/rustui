//! Compact public-API examples for fixed grids and overlay ordering.

use std::num::NonZeroU16;

use rustui::prelude::*;

pub fn demo() -> impl View {
    HStack::new()
        .child(
            Container(
                VStack::new()
                    .spacing(8.0)
                    .child(LocalizedText("lab-grid-demo-title"))
                    .child(
                        Grid::new(NonZeroU16::new(3).expect("three is nonzero"))
                            .spacing(4.0)
                            .children(["01", "02", "03", "04", "05", "06"].map(|label| {
                                Container(Text(label))
                                    .width(54.0)
                                    .height(28.0)
                                    .padding(4.0)
                                    .background(Surface::Sidebar)
                                    .border()
                            })),
                    )
                    .child(LocalizedText("lab-zstack-demo-title"))
                    .child(
                        Container(
                            ZStack::new()
                                .alignment(ZStackAlignment::new(
                                    HorizontalAlignment::Center,
                                    VerticalAlignment::Center,
                                ))
                                .child(
                                    Container(LocalizedText("lab-zstack-back-layer"))
                                        .width(176.0)
                                        .height(58.0)
                                        .padding(8.0)
                                        .background(Surface::Panel)
                                        .border(),
                                )
                                .child(
                                    Container(LocalizedText("lab-zstack-front-layer"))
                                        .padding(6.0)
                                        .background(Surface::Sidebar)
                                        .border(),
                                ),
                        )
                        .width(192.0)
                        .height(72.0)
                        .border(),
                    ),
            )
            .width(204.0)
            .padding(8.0)
            .border(),
        )
        .child(Spacer())
}
