//! Toolbar action, order, disabled, and localization review through public RustUI.

use rustui::localization::MessageArguments;
use rustui::prelude::*;

pub fn demo() -> impl View {
    Component("toolbar-gallery", ToolbarGallery::default)
}

const BUTTON_GROUP_ACTIONS: [(&str, &str); 3] =
    [("previous", "Previous"), ("next", "Next"), ("more", "More")];

pub fn button_group_demo() -> impl View {
    Component("button-group-gallery", ButtonGroupGallery::default)
}

const SEGMENTED_CONTROL_MODES: [(&str, &str); 4] = [
    ("icons", "Icons"),
    ("list", "List"),
    ("columns", "Columns"),
    ("gallery", "Gallery"),
];

pub fn segmented_control_demo() -> impl View {
    Component(
        "segmented-control-gallery",
        SegmentedControlGallery::default,
    )
}

#[component]
struct SegmentedControlGallery {
    #[state(String::from("icons"))]
    selected_mode: String,
}

impl SegmentedControlGallery {
    fn mode_is_selected(&self, mode: &str) -> bool {
        self.selected_mode() == mode
    }

    fn selected_label(&self) -> &'static str {
        SEGMENTED_CONTROL_MODES
            .iter()
            .find_map(|(id, label)| self.mode_is_selected(id).then_some(*label))
            .unwrap_or("Icons")
    }

    fn show_icons(&mut self) {
        self.selected_mode = String::from("icons");
    }

    fn show_list(&mut self) {
        self.selected_mode = String::from("list");
    }

    fn show_columns(&mut self) {
        self.selected_mode = String::from("columns");
    }

    fn show_gallery(&mut self) {
        self.selected_mode = String::from("gallery");
    }
}

impl View for SegmentedControlGallery {
    fn body(&self) -> impl View {
        let selected_key = match self.selected_label() {
            "List" => "lab-segment-list",
            "Columns" => "lab-segment-columns",
            "Gallery" => "lab-segment-gallery",
            _ => "lab-segment-icons",
        };
        let selected_style = if self.mode_is_selected("icons") {
            ButtonStyle::BorderedProminent
        } else {
            ButtonStyle::Plain
        };
        let list_style = if self.mode_is_selected("list") {
            ButtonStyle::BorderedProminent
        } else {
            ButtonStyle::Plain
        };
        let columns_style = if self.mode_is_selected("columns") {
            ButtonStyle::BorderedProminent
        } else {
            ButtonStyle::Plain
        };
        let gallery_style = if self.mode_is_selected("gallery") {
            ButtonStyle::BorderedProminent
        } else {
            ButtonStyle::Plain
        };

        let segments = Container(
            HStack::new()
                .spacing(0.0)
                .child(
                    Button(SEGMENTED_CONTROL_MODES[0].1, self.action(Self::show_icons))
                        .id("segmented-control-icons")
                        .localized_label("lab-segment-icons")
                        .button_style(selected_style)
                        .toggled(self.mode_is_selected("icons")),
                )
                .child(
                    Button(SEGMENTED_CONTROL_MODES[1].1, self.action(Self::show_list))
                        .id("segmented-control-list")
                        .localized_label("lab-segment-list")
                        .button_style(list_style)
                        .toggled(self.mode_is_selected("list")),
                )
                .child(
                    Button(
                        SEGMENTED_CONTROL_MODES[2].1,
                        self.action(Self::show_columns),
                    )
                    .id("segmented-control-columns")
                    .localized_label("lab-segment-columns")
                    .button_style(columns_style)
                    .toggled(self.mode_is_selected("columns")),
                )
                .child(
                    Button(
                        SEGMENTED_CONTROL_MODES[3].1,
                        self.action(Self::show_gallery),
                    )
                    .id("segmented-control-gallery")
                    .localized_label("lab-segment-gallery")
                    .button_style(gallery_style)
                    .toggled(self.mode_is_selected("gallery")),
                ),
        )
        .padding(4.0)
        .background(Surface::Panel)
        .border()
        .radius(12.0);

        VStack::new()
            .spacing(8.0)
            .child(LocalizedText("lab-segment-title"))
            .child(segments)
            .child(
                HStack::new()
                    .spacing(4.0)
                    .child(LocalizedText("lab-segment-selected"))
                    .child(LocalizedText(selected_key)),
            )
    }
}

#[component]
struct ButtonGroupGallery {
    #[state(String::from("None"))]
    last_action: String,
}

impl ButtonGroupGallery {
    fn choose_previous(&mut self) {
        self.last_action = String::from("Previous");
    }

    fn choose_next(&mut self) {
        self.last_action = String::from("Next");
    }

    fn choose_more(&mut self) {
        self.last_action = String::from("More");
    }
}

impl View for ButtonGroupGallery {
    fn body(&self) -> impl View {
        let group = Container(
            HStack::new()
                .spacing(0.0)
                .child(
                    Button(
                        BUTTON_GROUP_ACTIONS[0].1,
                        self.action(Self::choose_previous),
                    )
                    .id("button-group-previous")
                    .localized_label("lab-button-group-previous"),
                )
                .child(
                    Button(BUTTON_GROUP_ACTIONS[1].1, self.action(Self::choose_next))
                        .id("button-group-next")
                        .localized_label("lab-button-group-next"),
                )
                .child(
                    Button(BUTTON_GROUP_ACTIONS[2].1, self.action(Self::choose_more))
                        .id("button-group-more")
                        .localized_label("lab-button-group-more"),
                ),
        )
        .padding(4.0)
        .background(Surface::Panel)
        .border()
        .radius(18.0);
        VStack::new()
            .spacing(8.0)
            .child(LocalizedText("lab-button-group-title"))
            .child(group)
            .child(
                HStack::new()
                    .spacing(4.0)
                    .child(LocalizedText("lab-button-group-last-action"))
                    .child(LocalizedText(match self.last_action().as_str() {
                        "Previous" => "lab-button-group-previous",
                        "Next" => "lab-button-group-next",
                        "More" => "lab-button-group-more",
                        _ => "lab-button-group-none",
                    })),
            )
    }
}

#[cfg(test)]
mod button_group_tests {
    use super::{
        BUTTON_GROUP_ACTIONS, ButtonGroupGallery, SEGMENTED_CONTROL_MODES, SegmentedControlGallery,
    };

    fn assert_selection(
        gallery: &SegmentedControlGallery,
        expected_id: &str,
        expected_label: &str,
    ) {
        let selected_modes = SEGMENTED_CONTROL_MODES
            .iter()
            .filter_map(|(id, _)| gallery.mode_is_selected(id).then_some(*id))
            .collect::<Vec<_>>();
        assert_eq!(selected_modes, vec![expected_id]);
        assert_eq!(gallery.selected_label(), expected_label);
    }

    #[test]
    fn navigation_story_messages_exist_for_all_runtime_locales() {
        let catalogs = super::super::locale_selector::catalogs();
        for language in ["en", "fr", "ar"] {
            let locale = rustui::localization::Locale::parse(language).unwrap();
            let catalog = catalogs.get(&locale).unwrap();
            for key in [
                "lab-segment-title",
                "lab-segment-selected",
                "lab-segment-icons",
                "lab-segment-list",
                "lab-segment-columns",
                "lab-segment-gallery",
                "lab-button-group-title",
                "lab-button-group-last-action",
                "lab-button-group-none",
                "lab-button-group-previous",
                "lab-button-group-next",
                "lab-button-group-more",
            ] {
                assert!(
                    catalog.get(key).is_some_and(|text| !text.trim().is_empty()),
                    "{language}: {key}"
                );
            }
        }
    }

    #[test]
    fn button_group_composes_the_reference_actions_and_reports_activation() {
        assert_eq!(
            BUTTON_GROUP_ACTIONS,
            [("previous", "Previous"), ("next", "Next"), ("more", "More"),]
        );

        let mut gallery = ButtonGroupGallery::default();
        assert_eq!(gallery.last_action(), "None");
        gallery.choose_more();
        assert_eq!(gallery.last_action(), "More");
    }

    #[test]
    fn button_group_generated_actions_update_last_action_feedback() {
        let mut gallery = ButtonGroupGallery::default();

        let actions: [(fn(&mut ButtonGroupGallery), &str); 3] = [
            (ButtonGroupGallery::choose_previous, "Previous"),
            (ButtonGroupGallery::choose_next, "Next"),
            (ButtonGroupGallery::choose_more, "More"),
        ];
        for (action, expected) in actions {
            assert!(
                gallery
                    .action(action)
                    .dispatch(&mut gallery)
                    .should_notify()
            );
            assert_eq!(gallery.last_action(), expected);
        }
    }

    #[test]
    fn segmented_control_composes_reference_modes_and_tracks_one_selection() {
        assert_eq!(
            SEGMENTED_CONTROL_MODES,
            [
                ("icons", "Icons"),
                ("list", "List"),
                ("columns", "Columns"),
                ("gallery", "Gallery"),
            ]
        );

        let mut gallery = SegmentedControlGallery::default();
        assert_selection(&gallery, "icons", "Icons");
        gallery.show_list();
        assert_selection(&gallery, "list", "List");
        gallery.show_columns();
        assert_selection(&gallery, "columns", "Columns");
        gallery.show_gallery();
        assert_selection(&gallery, "gallery", "Gallery");
        gallery.show_icons();
        assert_selection(&gallery, "icons", "Icons");
    }
}

#[component]
struct ToolbarGallery {
    #[state(0)]
    count: u32,
    #[state(false)]
    reverse: bool,
    #[state(false)]
    disabled: bool,
    #[state(0)]
    disabled_hits: u32,
}

impl ToolbarGallery {
    fn increment(&mut self) {
        self.count += 1;
    }

    fn reset(&mut self) {
        self.count = 0;
    }

    fn toggle_order(&mut self) {
        self.reverse = !self.reverse;
    }

    fn toggle_disabled(&mut self) {
        self.disabled = !self.disabled;
    }

    fn disabled_hit(&mut self) {
        self.disabled_hits += 1;
    }
}

impl View for ToolbarGallery {
    fn body(&self) -> impl View {
        let mut items = vec![
            ToolbarItem::action("increment", "Increment", self.action(Self::increment))
                .localized("lab-toolbar-increment"),
            ToolbarItem::action("reset", "Reset", self.action(Self::reset))
                .localized("lab-toolbar-reset"),
            ToolbarItem::action(
                "unavailable",
                "Unavailable",
                self.action(Self::disabled_hit),
            )
            .disabled(true)
            .localized("lab-toolbar-unavailable"),
            ToolbarItem::action("localized-close", "Close", self.action(Self::reset))
                .localized("rustui-action-close"),
        ];
        if self.reverse() {
            items.reverse();
        }

        VStack::new()
            .spacing(8.0)
            .child(LocalizedText("lab-toolbar-title"))
            .child(
                LocalizedText("lab-toolbar-counts").with_arguments(
                    MessageArguments::new()
                        .with("count", i64::from(self.count()))
                        .with("hits", i64::from(self.disabled_hits())),
                ),
            )
            .child(LocalizedText(if self.reverse() {
                "lab-toolbar-order-reversed"
            } else {
                "lab-toolbar-order-normal"
            }))
            .child(LocalizedText(if self.disabled() {
                "lab-toolbar-disabled"
            } else {
                "lab-toolbar-enabled"
            }))
            .child(Toolbar("toolbar-gallery-actions", items).disabled(self.disabled()))
            .child(
                HStack::new().child(
                    Button("Reverse order", self.action(Self::toggle_order))
                        .id("toolbar-reverse")
                        .localized_label("lab-toolbar-reverse-action"),
                ),
            )
            .child(
                HStack::new().child(
                    Button("Toggle disabled", self.action(Self::toggle_disabled))
                        .id("toolbar-disabled")
                        .localized_label("lab-toolbar-disabled-action"),
                ),
            )
            .child(LocalizedText("lab-toolbar-help"))
    }
}
