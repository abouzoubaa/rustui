//! Card surface, elevation, direction, and retained-child demonstration.

use rustui::{
    component,
    localization::MessageArguments,
    prelude::*,
    theme::{AppearanceMode, activate_theme, parse_theme},
};

const SHARED_CHILD_ID: &str = "card-shared-input";

pub fn demo() -> impl View {
    Component("card-gallery", CardGallery::default)
}

#[component]
struct CardGallery {
    #[state(false)]
    custom: bool,
    #[state(false)]
    bordered: bool,
    #[state(true)]
    show_a: bool,
    #[state(0)]
    parent_updates: usize,
    #[state(String::new())]
    first_value: String,
    #[state(String::new())]
    second_value: String,
}

impl CardGallery {
    fn toggle_custom(&mut self) {
        self.custom = !self.custom;
    }

    fn toggle_border(&mut self) {
        self.bordered = !self.bordered;
    }

    fn toggle_a(&mut self) {
        self.show_a = !self.show_a;
    }

    fn update_parent(&mut self) {
        self.parent_updates += 1;
    }
}

impl View for CardGallery {
    fn body(&self) -> impl View {
        let custom = self.custom();
        let bordered = self.bordered();
        let show_a = self.show_a();
        let parent_updates = self.parent_updates();
        let first_value = self.first_value();
        let second_value = self.second_value();

        let appearance = |mode| {
            AppAction::new(move |cx| {
                let document = parse_theme(include_str!("../../../../themes/macos27.json"))
                    .expect("checked-in theme is valid");
                activate_theme(&document, mode, cx).expect("theme activation succeeds");
            })
        };

        let first_card = if show_a {
            let card = Card(
                "first-card",
                TextField("First card", self.first_value_binding())
                    .id(SHARED_CHILD_ID)
                    .localized_placeholder("lab-card-first-placeholder"),
            );
            let card = if custom {
                card.padding(28.0)
                    .radius(18.0)
                    .elevation(ShadowElevation::Large)
            } else {
                card
            };
            let card = card.border(bordered);
            AnyView::new(Container(card).width(260.0))
        } else {
            AnyView::new(Container(LocalizedText("lab-card-first-omitted")).width(260.0))
        };

        let second = Card(
            "second-card",
            TextField("Second card", self.second_value_binding())
                .id(SHARED_CHILD_ID)
                .localized_placeholder("lab-card-second-placeholder"),
        );
        let second = if custom {
            second
                .padding(28.0)
                .radius(18.0)
                .elevation(ShadowElevation::Small)
        } else {
            second
        }
        .border(bordered);

        VStack::new()
            .spacing(10.0)
            .child(LocalizedText("lab-card-title"))
            .child(
                VStack::new()
                    .spacing(4.0)
                    .child(
                        HStack::new()
                            .spacing(8.0)
                            .child(
                                Button("Default / custom", self.action(Self::toggle_custom))
                                    .id("card-custom")
                                    .localized_label("lab-card-custom"),
                            )
                            .child(
                                Button("Toggle border", self.action(Self::toggle_border))
                                    .id("card-border")
                                    .localized_label("lab-card-border"),
                            ),
                    )
                    .child(
                        HStack::new()
                            .spacing(8.0)
                            .child(
                                Button("Omit / restore first", self.action(Self::toggle_a))
                                    .id("card-mount")
                                    .localized_label("lab-card-mount"),
                            )
                            .child(
                                Button("Update parent", self.action(Self::update_parent))
                                    .id("card-parent")
                                    .localized_label("lab-card-parent"),
                            ),
                    ),
            )
            .child(
                HStack::new()
                    .spacing(8.0)
                    .child(
                        Button("Light appearance", appearance(Some(AppearanceMode::Light)))
                            .id("card-light")
                            .localized_label("lab-card-light"),
                    )
                    .child(
                        Button("Dark appearance", appearance(Some(AppearanceMode::Dark)))
                            .id("card-dark")
                            .localized_label("lab-card-dark"),
                    ),
            )
            .child(super::locale_selector::LocaleSelector::new())
            .child(
                LocalizedText("lab-card-summary").with_arguments(
                    MessageArguments::new()
                        .with("count", parent_updates as i64)
                        .with("first", first_value)
                        .with("second", second_value),
                ),
            )
            .child(
                HStack::new()
                    .spacing(10.0)
                    .child(first_card)
                    .child(Container(second).width(260.0)),
            )
    }
}
