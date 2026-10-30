//! Stateful RustUI stories embedded in the canonical Playground.

use rustui::localization::MessageArguments;
use rustui::prelude::*;
use std::time::Duration;

const MENU_BAR_REFERENCE_LABELS: [&str; 9] = [
    "Finder",
    "File",
    "Edit",
    "View",
    "Item",
    "Window",
    "Help",
    "Wed Apr 1",
    "9:41 AM",
];

mod compiled_preview_gallery;

#[cfg(target_family = "wasm")]
pub(crate) fn browser_reference(profile: &str) -> Option<rustui::view::AnyView> {
    match profile {
        "preview" => Some(rustui::view::AnyView::new(
            compiled_preview_gallery::reference_demo(),
        )),
        "composition" => Some(rustui::view::AnyView::new(
            composition_gallery::reference_demo(),
        )),
        _ => None,
    }
}
mod composition_gallery;

mod alert_gallery;
mod animation_gallery;
mod animation_identity_gallery;
mod cache_gallery;
mod canvas;
mod child_component;
mod code_editor_gallery;
mod code_preview;
mod collection_gallery;
mod combo_box_reconciliation_gallery;
mod component_catalog;
mod context_menu_gallery;
pub(super) mod data_grid_accessibility;
mod dialog_gallery;
mod disclosure_gallery;
mod drawer_gallery;
mod event_inspector;
mod feedback_gallery;
mod focus_gallery;
mod form_accessories_gallery;
mod foundation_additions;
mod foundation_demos;
mod frame_gallery;
mod grid_overlays;
mod keyed_list_gallery;
mod layout_inspector;
mod lazy_grid_gallery;
mod locale_selector;
mod localization_gallery;
mod menu_bar_reference_gallery;
mod menu_gallery;
mod navigation_split_gallery;
mod offset_gallery;
mod outline_gallery;
mod owner_callbacks;
mod owner_tasks;
mod popover_gallery;
mod primitive_gallery;
mod properties_panel;
mod render_timings;
mod semantics_gallery;
mod sheet_gallery;
mod sidebar_gallery;
mod split_view_gallery;
mod spring_gallery;
mod supplementary_gallery;
mod tab_bar_reference_gallery;
mod table_reconciliation_gallery;
mod task_release;
mod test_runner;
mod text_specimen_gallery;
mod text_style_gallery;
mod theme_inspector;
mod toolbar_gallery;
mod tooltip_gallery;
mod tree_reconciliation_gallery;
mod utility_window_gallery;
mod window_chrome_gallery;
mod window_controls_gallery;

use canvas::{RetentionStoryFields, StoryCanvas};
use child_component::DemoDisplayMode;
pub(crate) use component_catalog::{STORIES, StoryId, filtered_story_indices};
use event_inspector::EventLog;
pub(crate) use locale_selector::LocaleSelector;
pub(crate) use locale_selector::{catalogs, initial_locale};
use properties_panel::ButtonProperties;

/// Returns existing runtime demos for their catalog entries.
pub(crate) fn catalog_demo(key: &str) -> Option<AnyView> {
    match key {
        "text" => Some(AnyView::new(text_specimen_gallery::demo())),
        "text_style_guide" => Some(AnyView::new(text_style_gallery::demo())),
        "combo_box" => Some(AnyView::new(combo_box_reconciliation_gallery::demo())),
        "link" => Some(AnyView::new(supplementary_gallery::demo())),
        "divider" => Some(AnyView::new(foundation_additions::foundation_additions())),
        "notification" => Some(AnyView::new(feedback_gallery::demo())),
        "button_group" => Some(AnyView::new(toolbar_gallery::button_group_demo())),
        "segmented_control" => Some(AnyView::new(toolbar_gallery::segmented_control_demo())),
        "window_chrome" => Some(AnyView::new(window_chrome_gallery::demo())),
        "utility_window" => Some(AnyView::new(utility_window_gallery::demo())),
        "form" => Some(AnyView::new(form_accessories_gallery::demo())),
        "form_accessories" => Some(AnyView::new(form_accessories_gallery::demo())),
        "menu_bar" => Some(AnyView::new(menu_bar_reference_gallery::demo())),
        "window_controls" => Some(AnyView::new(window_controls_gallery::demo())),
        "alert_dialog" => Some(AnyView::new(alert_gallery::demo())),
        "save_dialog" => Some(AnyView::new(dialog_gallery::demo())),
        "sheet" => Some(AnyView::new(
            VStack::new()
                .spacing(16.0)
                .child(sheet_gallery::demo())
                .child(drawer_gallery::demo()),
        )),
        "toolbar" => Some(AnyView::new(toolbar_gallery::demo())),
        "tab_bar" => Some(AnyView::new(tab_bar_reference_gallery::demo())),
        "menu_item" => Some(AnyView::new(menu_gallery::demo())),
        "menu_separator" => Some(AnyView::new(menu_gallery::demo())),
        "menu" => Some(AnyView::new(
            VStack::new()
                .spacing(20.0)
                .child(popover_gallery::demo())
                .child(menu_gallery::demo())
                .child(context_menu_gallery::demo())
                .child(tooltip_gallery::demo())
                .child(animation_gallery::demo())
                .child(spring_gallery::demo())
                .child(animation_identity_gallery::demo()),
        )),
        "list" => Some(AnyView::new(keyed_list_gallery::demo())),
        "sidebar" => Some(AnyView::new(sidebar_gallery::demo())),
        "navigation_split" => Some(AnyView::new(navigation_split_gallery::demo())),
        "offset" => Some(AnyView::new(offset_gallery::demo())),
        "collection" => Some(AnyView::new(collection_gallery::demo())),
        "drag_drop" => Some(AnyView::new(drag_drop_gallery::demo())),
        "disclosure_control" => Some(AnyView::new(disclosure_gallery::demo())),
        "label" | "icon" | "image" => Some(AnyView::new(primitive_gallery::demo())),
        _ => None,
    }
}

pub(crate) fn split_view_demo() -> impl View {
    split_view_gallery::demo()
}

#[cfg(test)]
mod catalog_demo_tests {
    use super::{MENU_BAR_REFERENCE_LABELS, catalog_demo};

    #[test]
    fn divider_catalog_reaches_horizontal_and_vertical_demo() {
        assert!(catalog_demo("divider").is_some());
    }

    #[test]
    fn button_group_catalog_reaches_public_button_composition() {
        assert!(catalog_demo("button_group").is_some());
    }

    #[test]
    fn tab_bar_catalog_reaches_public_tab_view_composition() {
        assert!(catalog_demo("tab_bar").is_some());
    }

    #[test]
    fn segmented_control_catalog_reaches_public_button_composition() {
        assert!(catalog_demo("segmented_control").is_some());
    }

    #[test]
    fn menu_catalog_routes_menu_and_context_menu_demos() {
        assert!(catalog_demo("menu").is_some());
    }

    #[test]
    fn menu_item_catalog_entry_reaches_menu_demo() {
        assert!(catalog_demo("menu_item").is_some());
    }

    #[test]
    fn menu_separator_catalog_entry_reaches_menu_demo() {
        assert!(catalog_demo("menu_separator").is_some());
    }

    #[test]
    fn list_catalog_reaches_keyed_list_gallery() {
        assert!(catalog_demo("list").is_some());
    }

    #[test]
    fn sidebar_catalog_reaches_stable_selection_gallery() {
        assert!(catalog_demo("sidebar").is_some());
    }

    #[test]
    fn disclosure_catalog_reaches_controlled_disclosure_gallery() {
        assert!(catalog_demo("disclosure_control").is_some());
    }

    #[test]
    fn window_chrome_catalog_routes_to_composed_preview() {
        assert!(catalog_demo("window_chrome").is_some());
    }

    #[test]
    fn primitive_composition_catalog_entries_reach_previews() {
        for key in ["utility_window", "form", "menu_bar", "window_controls"] {
            assert!(
                catalog_demo(key).is_some(),
                "missing composition preview: {key}"
            );
        }
    }

    #[test]
    fn menu_bar_composition_matches_the_reference_labels_and_order() {
        assert_eq!(
            MENU_BAR_REFERENCE_LABELS,
            [
                "Finder",
                "File",
                "Edit",
                "View",
                "Item",
                "Window",
                "Help",
                "Wed Apr 1",
                "9:41 AM",
            ]
        );
        assert!(catalog_demo("menu_bar").is_some());
    }

    #[test]
    fn form_accessories_catalog_routes_to_its_public_api_composition() {
        assert!(catalog_demo("form_accessories").is_some());
    }

    #[test]
    fn text_style_guide_catalog_routes_to_its_public_typography_gallery() {
        assert!(catalog_demo("text_style_guide").is_some());
    }

    #[test]
    fn text_catalog_routes_to_its_public_text_specimen() {
        assert!(catalog_demo("text").is_some());
    }
}

#[component]
pub struct StoryHost {
    selected: StoryId,
    #[state(0)]
    count: i32,
    #[state(ButtonProperties::default())]
    button_properties: ButtonProperties,
    #[state(String::new())]
    input_text: String,
    #[state(0)]
    external_input_updates: u32,
    #[state(String::new())]
    external_input_status: String,
    #[state(String::new())]
    secure_text: String,
    #[state(false)]
    toggle_value: bool,
    #[state(false)]
    checkbox_value: bool,
    #[state(false)]
    radio_value: bool,
    #[state(35.0)]
    slider_value: f32,
    #[state(1)]
    stepper_value: i32,
    #[state(String::from("Rust"))]
    picker_value: String,
    #[state(String::new())]
    combo_box_value: String,
    #[state(None)]
    date_value: Option<NaiveDate>,
    #[state(None)]
    color_value: Option<Hsla>,
    #[state(None)]
    list_selection: Option<usize>,
    #[state(None)]
    table_selection: Option<usize>,
    #[state(None)]
    tree_selection: Option<String>,
    #[state(String::from("overview"))]
    tab_selection: String,
    #[state(true)]
    details_tab_present: bool,
    #[state(false)]
    tab_conditional_visible: bool,
    #[state(false)]
    navigation_conditional_visible: bool,
    #[state(String::from("Alpha row"))]
    keyed_alpha_text: String,
    #[state(String::from("Beta row"))]
    keyed_beta_text: String,
    #[state(false)]
    keyed_rows_reversed: bool,
    #[state(true)]
    keyed_beta_present: bool,
    #[state(false)]
    child_titles_prefixed: bool,
    #[state(DemoDisplayMode::Compact)]
    environment_display_mode: DemoDisplayMode,
    #[state(0)]
    w1_epoch: u64,
    #[state(0)]
    w1_step: u8,
    #[state(0)]
    w1_alpha_increment: u8,
    #[state(0)]
    w1_beta_increment: u8,
    #[state(0)]
    w1_manual_result: u8,
    #[state(String::new())]
    tab_overview_notes: String,
    #[state(String::new())]
    tab_details_notes: String,
    #[state(String::new())]
    tab_conditional_notes: String,
    #[state(String::new())]
    navigation_root_notes: String,
    #[state(String::new())]
    navigation_details_notes: String,
    #[state(String::new())]
    navigation_inspector_notes: String,
    #[state(String::new())]
    navigation_conditional_notes: String,
    #[state(String::from("Components"))]
    breadcrumb_page: String,
    #[state(Vec::<String>::new())]
    navigation_path: Vec<String>,
    events: EventLog,
}

impl StoryHost {
    pub fn new(story: StoryId) -> Self {
        Self {
            selected: story,
            ..Self::default()
        }
    }

    pub fn select_story(&mut self, story: StoryId) {
        self.selected = story;
    }

    pub fn apply_story(&mut self, story: &StoryId) {
        self.select_story(*story);
    }

    fn finish_external_input_update(&mut self, state: AsyncState<(), String>) -> bool {
        match state {
            AsyncState::Success(()) => {
                self.external_input_updates += 1;
                self.input_text = format!("External owner update {}", self.external_input_updates);
                self.external_input_status = format!(
                    "Owner changed the bound value to {:?}; check the focused field and composition",
                    self.input_text
                );
                true
            }
            AsyncState::Error(error) => {
                self.external_input_status = format!("Owner update failed: {error}");
                true
            }
            AsyncState::Idle | AsyncState::Loading => false,
        }
    }

    fn toggle_details_tab(&mut self) {
        self.details_tab_present = !self.details_tab_present;
        if !self.details_tab_present && self.tab_selection == "details" {
            self.tab_selection = String::from("overview");
        }
        self.events.record_simple(if self.details_tab_present {
            "lab-event-details-restored"
        } else {
            "lab-event-details-removed"
        });
    }

    fn toggle_tab_conditional_editor(&mut self) {
        self.tab_conditional_visible = !self.tab_conditional_visible;
        self.events.record_simple(if self.tab_conditional_visible {
            "lab-event-tabs-editor-shown"
        } else {
            "lab-event-tabs-editor-hidden"
        });
    }

    fn toggle_navigation_conditional_editor(&mut self) {
        self.navigation_conditional_visible = !self.navigation_conditional_visible;
        self.events
            .record_simple(if self.navigation_conditional_visible {
                "lab-event-navigation-editor-shown"
            } else {
                "lab-event-navigation-editor-hidden"
            });
    }

    fn toggle_keyed_order(&mut self) {
        self.keyed_rows_reversed = !self.keyed_rows_reversed;
        self.events.record_simple(if self.keyed_rows_reversed {
            "lab-event-rows-reversed"
        } else {
            "lab-event-rows-restored"
        });
    }

    fn toggle_keyed_beta(&mut self) {
        self.keyed_beta_present = !self.keyed_beta_present;
        self.events.record_simple(if self.keyed_beta_present {
            "lab-event-beta-restored"
        } else {
            "lab-event-beta-removed"
        });
    }

    fn toggle_child_titles(&mut self) {
        self.child_titles_prefixed = !self.child_titles_prefixed;
        self.events.record_simple(if self.child_titles_prefixed {
            "lab-event-titles-prefixed"
        } else {
            "lab-event-titles-restored"
        });
    }

    fn w1_parent_matches(&self) -> bool {
        self.keyed_beta_present
            && self.w1_alpha_increment == u8::from(self.w1_step >= 2)
            && self.w1_beta_increment == u8::from(self.w1_step >= 3)
            && self.child_titles_prefixed == (self.w1_step == 4)
            && self.keyed_rows_reversed == (self.w1_step == 6)
    }

    fn reset_w1(&mut self) {
        self.w1_epoch += 1;
        self.w1_step = 1;
        self.w1_alpha_increment = 0;
        self.w1_beta_increment = 0;
        self.w1_manual_result = 0;
        self.keyed_beta_present = true;
        self.keyed_rows_reversed = false;
        self.child_titles_prefixed = false;
        self.keyed_alpha_text = String::from("Alpha row");
        self.keyed_beta_text = String::from("Beta row");
        self.events.record(
            "lab-event-w1-reset",
            MessageArguments::new().with("run", self.w1_epoch as i64),
        );
        println!(
            r#"{{"kind":"rustui_w1","run":{},"step":0,"event":"reset_dispatched","visual":"not_checked"}}"#,
            self.w1_epoch
        );
    }

    fn next_w1_step(&mut self) {
        if !self.w1_parent_matches() {
            self.events.record_simple("lab-event-w1-parent-mismatch");
            println!(
                r#"{{"kind":"rustui_w1","run":{},"step":{},"event":"parent_state_mismatch"}}"#,
                self.w1_epoch,
                self.w1_step.saturating_sub(1)
            );
            return;
        }
        let (message, event) = match self.w1_step {
            1 => {
                self.w1_alpha_increment = 1;
                ("lab-event-w1-alpha-increment", "increment_alpha_dispatched")
            }
            2 => {
                self.w1_beta_increment = 1;
                ("lab-event-w1-beta-increment", "increment_beta_dispatched")
            }
            3 => {
                self.child_titles_prefixed = true;
                ("lab-event-w1-prefix-enabled", "prefix_enabled")
            }
            4 => {
                self.child_titles_prefixed = false;
                ("lab-event-w1-prefix-disabled", "prefix_disabled")
            }
            5 => {
                self.keyed_rows_reversed = true;
                ("lab-event-w1-order-reversed", "order_reversed")
            }
            6 => {
                self.keyed_rows_reversed = false;
                ("lab-event-w1-order-restored", "order_restored")
            }
            _ => return,
        };
        self.w1_step += 1;
        self.events.record(
            message,
            MessageArguments::new().with("step", (self.w1_step - 1) as i64),
        );
        println!(
            r#"{{"kind":"rustui_w1","run":{},"step":{},"event":"{}","sequence_dispatched":{},"visual":"not_checked"}}"#,
            self.w1_epoch,
            self.w1_step - 1,
            event,
            self.w1_step == 7
        );
    }

    fn pass_w1_manual_check(&mut self) {
        if self.w1_step == 7 && self.w1_parent_matches() {
            self.w1_manual_result = 1;
            self.events.record(
                "lab-event-w1-pass",
                MessageArguments::new().with("run", self.w1_epoch as i64),
            );
            println!(
                r#"{{"kind":"rustui_w1","run":{},"step":6,"event":"visual_check","visual":"pass"}}"#,
                self.w1_epoch
            );
        }
    }

    fn fail_w1_manual_check(&mut self) {
        if self.w1_step == 7 {
            self.w1_manual_result = 2;
            self.events.record(
                "lab-event-w1-fail",
                MessageArguments::new().with("run", self.w1_epoch as i64),
            );
            println!(
                r#"{{"kind":"rustui_w1","run":{},"step":6,"event":"visual_check","visual":"fail"}}"#,
                self.w1_epoch
            );
        }
    }

    fn toggle_environment_display_mode(&mut self) {
        self.environment_display_mode.toggle();
        self.events
            .record_simple(match self.environment_display_mode {
                DemoDisplayMode::Compact => "lab-event-display-compact",
                DemoDisplayMode::Detailed => "lab-event-display-detailed",
            });
    }

    fn breadcrumb_overview(&mut self) {
        self.breadcrumb_page = String::from("Overview");
        self.events.record_simple("lab-event-breadcrumb-overview");
    }

    fn breadcrumb_components(&mut self) {
        self.breadcrumb_page = String::from("Components");
        self.events.record_simple("lab-event-breadcrumb-components");
    }

    fn increment(&mut self) {
        self.count += 1;
        self.events.record(
            "lab-event-count-incremented",
            MessageArguments::new().with("count", self.count),
        );
    }

    fn reset(&mut self) {
        self.count = 0;
        self.events.record_simple("lab-event-story-reset");
    }
    fn set_picker_kotlin(&mut self) {
        self.picker_value = String::from("Kotlin");
        self.events.record_simple("lab-event-picker-kotlin");
    }

    fn cycle_button_style(&mut self) {
        self.button_properties.cycle_style();
        self.events.record(
            "lab-event-button-style",
            MessageArguments::new().with("style", format!("{:?}", self.button_properties.style)),
        );
    }

    fn toggle_button_disabled(&mut self) {
        self.button_properties.disabled = !self.button_properties.disabled;
        self.events
            .record_simple(if self.button_properties.disabled {
                "lab-event-button-disabled"
            } else {
                "lab-event-button-enabled"
            });
    }

    fn clear_events(&mut self) {
        self.events.clear();
    }
}

impl View for StoryHost {
    fn body(&self) -> impl View {
        let selected = self.selected;
        let button_properties = self.button_properties();

        let retention_controls = match selected {
            StoryId::Layout => VStack::new()
                .child(
                    HStack::new()
                        .child(
                            Button(
                                if self.keyed_rows_reversed() {
                                    "Restore row order"
                                } else {
                                    "Reverse keyed rows"
                                },
                                self.action(Self::toggle_keyed_order),
                            )
                            .id("lab-toggle-keyed-order")
                            .localized_label(
                                if self.keyed_rows_reversed() {
                                    "lab-layout-restore-row-order"
                                } else {
                                    "lab-layout-reverse-keyed-rows"
                                },
                            ),
                        )
                        .child(
                            Button(
                                if self.keyed_beta_present() {
                                    "Remove Beta row"
                                } else {
                                    "Restore Beta row"
                                },
                                self.action(Self::toggle_keyed_beta),
                            )
                            .id("lab-toggle-keyed-beta")
                            .localized_label(
                                if self.keyed_beta_present() {
                                    "lab-layout-remove-beta-row"
                                } else {
                                    "lab-layout-restore-beta-row"
                                },
                            ),
                        ),
                )
                .child(
                    HStack::new()
                        .child(
                            Button(
                                if self.child_titles_prefixed() {
                                    "Restore child titles"
                                } else {
                                    "Prefix child titles"
                                },
                                self.action(Self::toggle_child_titles),
                            )
                            .id("lab-toggle-child-titles")
                            .localized_label(
                                if self.child_titles_prefixed() {
                                    "lab-layout-restore-child-titles"
                                } else {
                                    "lab-layout-prefix-child-titles"
                                },
                            ),
                        )
                        .child(
                            Button(
                                match self.environment_display_mode() {
                                    DemoDisplayMode::Compact => "Use detailed environment",
                                    DemoDisplayMode::Detailed => "Use compact environment",
                                },
                                self.action(Self::toggle_environment_display_mode),
                            )
                            .id("lab-toggle-environment-display-mode")
                            .localized_label(
                                match self.environment_display_mode() {
                                    DemoDisplayMode::Compact => {
                                        "lab-layout-use-detailed-environment"
                                    }
                                    DemoDisplayMode::Detailed => {
                                        "lab-layout-use-compact-environment"
                                    }
                                },
                            ),
                        ),
                ),
            StoryId::Tabs => VStack::new().child(
                HStack::new()
                    .child(
                        Button(
                            if self.details_tab_present() {
                                "Remove Details tab"
                            } else {
                                "Restore Details tab"
                            },
                            self.action(Self::toggle_details_tab),
                        )
                        .id("lab-toggle-details-tab"),
                    )
                    .child(
                        Button(
                            if self.tab_conditional_visible() {
                                "Hide extra Overview editor"
                            } else {
                                "Show extra Overview editor"
                            },
                            self.action(Self::toggle_tab_conditional_editor),
                        )
                        .id("lab-toggle-tab-editor")
                        .disabled(self.tab_selection() != "overview"),
                    ),
            ),
            StoryId::Navigation => VStack::new().child(
                HStack::new().child(
                    Button(
                        if self.navigation_conditional_visible() {
                            "Hide extra root editor"
                        } else {
                            "Show extra root editor"
                        },
                        self.action(Self::toggle_navigation_conditional_editor),
                    )
                    .id("lab-toggle-navigation-editor")
                    .disabled(!self.navigation_path().is_empty()),
                ),
            ),
            _ => VStack::new(),
        };
        let button_controls = if selected == StoryId::Button {
            HStack::new()
                .child(
                    Button("Cycle button style", self.action(Self::cycle_button_style))
                        .id("lab-cycle-style")
                        .localized_label("lab-button-cycle-style-visible"),
                )
                .child(
                    Button("Toggle disabled", self.action(Self::toggle_button_disabled))
                        .id("lab-toggle-disabled")
                        .localized_label("lab-button-toggle-disabled-visible"),
                )
        } else {
            HStack::new()
        };

        let w1_controls = if selected == StoryId::Layout {
            let step = self.w1_step();
            let parent_matches = self.w1_parent_matches();
            VStack::new()
                .child(
                    HStack::new().child(
                        LocalizedText("lab-w1-progress").with_arguments(
                            MessageArguments::new()
                                .with("run", self.w1_epoch() as i64)
                                .with("actions", step.saturating_sub(1) as i64),
                        ),
                    ),
                )
                .child(HStack::new().child(LocalizedText(if parent_matches {
                    "lab-w1-parent-matches"
                } else {
                    "lab-w1-parent-mismatch"
                })))
                .child(
                    HStack::new().child(LocalizedText(match self.w1_manual_result() {
                        1 => "lab-w1-visual-pass-status",
                        2 => "lab-w1-visual-fail-status",
                        _ => "lab-w1-visual-pending-status",
                    })),
                )
                .child(
                    HStack::new().child(
                        LocalizedText("lab-w1-expected-counts").with_arguments(
                            MessageArguments::new()
                                .with("alpha", u8::from(step >= 2) as i64)
                                .with("beta", u8::from(step >= 3) as i64),
                        ),
                    ),
                )
                .child(HStack::new().child(LocalizedText("lab-w1-guidance-inspect")))
                .child(HStack::new().child(LocalizedText("lab-w1-guidance-controls")))
                .child(HStack::new().child(LocalizedText("lab-w1-guidance-reset")))
                .child(
                    HStack::new()
                        .child(
                            Button("Reset W1", self.action(Self::reset_w1))
                                .id("w1-reset")
                                .localized_label("lab-w1-reset"),
                        )
                        .child(
                            Button("Next W1 step", self.action(Self::next_w1_step))
                                .id("w1-next-step")
                                .localized_label("lab-w1-next-step")
                                .disabled(!(1..7).contains(&step) || !parent_matches),
                        )
                        .child(
                            Button("Mark visual pass", self.action(Self::pass_w1_manual_check))
                                .id("w1-visual-pass")
                                .localized_label("lab-w1-mark-pass")
                                .disabled(step != 7 || !parent_matches),
                        )
                        .child(
                            Button("Mark visual fail", self.action(Self::fail_w1_manual_check))
                                .id("w1-visual-fail")
                                .localized_label("lab-w1-mark-fail")
                                .disabled(step != 7),
                        ),
                )
                .spacing(8.0)
        } else {
            VStack::new()
        };

        VStack! {
        retention_controls.spacing(8.0)
            w1_controls.spacing(8.0)
            (selected == StoryId::Input).then(|| {
                VStack::new()
                    .child(OwnerTaskReader(|tasks: OwnerTask<StoryHost>| {
                        Button(
                            "Change bound text from owner in 5 seconds",
                            tasks.start_with_executor(
                                "story-input-external-owner-update",
                                |executor| {
                                    let timer = executor.timer(Duration::from_secs(5));
                                    async move {
                                        timer.await;
                                        Ok::<(), String>(())
                                    }
                                },
                                StoryHost::finish_external_input_update,
                            ),
                        )
                        .id("story-input-external-owner-update")
                        .localized_label("lab-input-owner-update")
                    }))
                    .child(Text(self.external_input_status()))
                    .spacing(8.0)
            })
            (selected == StoryId::Layout && self.w1_step() == 0).then(|| Component("foundation-demos", foundation_demos::FoundationDemos::default))
            (selected == StoryId::Layout && self.w1_step() == 0).then(compiled_preview_gallery::demo)
            (selected == StoryId::Layout && self.w1_step() == 0).then(composition_gallery::demo)
        StoryCanvas::new(
            selected,
            self.count(),
            button_properties,
            self.input_text(),
            self.input_text_binding(),
            self.secure_text_binding(),
            self.toggle_value_binding(),
            self.checkbox_value_binding(),
            self.radio_value_binding(),
            self.slider_value_binding(),
            self.stepper_value_binding(),
            self.picker_value_binding(),
            self.combo_box_value_binding(),
            self.date_value_binding(),
            self.color_value_binding(),
            self.list_selection_binding(),
            self.table_selection_binding(),
            self.tree_selection_binding(),
            self.tab_selection_binding(),
            self.breadcrumb_page(),
            self.action(Self::breadcrumb_overview),
            self.action(Self::breadcrumb_components),
            self.navigation_path_binding(),
            RetentionStoryFields {
                tab_overview: self.tab_overview_notes_binding(),
                tab_details: self.tab_details_notes_binding(),
                tab_conditional: self.tab_conditional_notes_binding(),
                navigation_root: self.navigation_root_notes_binding(),
                navigation_details: self.navigation_details_notes_binding(),
                navigation_inspector: self.navigation_inspector_notes_binding(),
                navigation_conditional: self.navigation_conditional_notes_binding(),
                details_tab_present: self.details_tab_present(),
                tab_conditional_visible: self.tab_conditional_visible(),
                navigation_conditional_visible: self.navigation_conditional_visible(),
            },
            canvas::KeyedCollectionFields {
                alpha: self.keyed_alpha_text_binding(),
                beta: self.keyed_beta_text_binding(),
                reversed: self.keyed_rows_reversed(),
                beta_present: self.keyed_beta_present(),
                child_title_prefix: self.child_titles_prefixed().then(|| String::from("Parent · ")),
                display_mode: self.environment_display_mode(),
                    workload_active: self.w1_step() > 0,
                    workload_epoch: self.w1_epoch(),
                    alpha_workload_increment: self.w1_alpha_increment(),
                    beta_workload_increment: self.w1_beta_increment(),
            },
            self.action(Self::set_picker_kotlin),
            self.action(Self::increment),
            self.action(Self::reset),
        )
        (selected == StoryId::Tree).then(tree_reconciliation_gallery::demo)
            (selected == StoryId::Text).then(localization_gallery::demo)
        button_controls.spacing(8.0)
            Button("Clear event log", clear_events)
            .id("lab-clear-events")
            .localized_label("lab-clear-events-visible")
            (selected == StoryId::Button).then(|| LocalizedText("lab-button-layout-summary"))
            (selected != StoryId::Button).then(|| LocalizedText(layout_inspector::summary_key_for(selected)))
            (selected == StoryId::Button).then(|| LocalizedText("lab-button-test-status"))
            (selected != StoryId::Button).then(|| LocalizedText(test_runner::status_for(selected).localization_key()))
            (selected == StoryId::Button).then(|| LocalizedText("lab-button-source-disclaimer"))
            (selected != StoryId::Button).then(|| LocalizedText("lab-story-source-disclaimer"))
        Text(code_preview::source_for(selected))
        self.events.view()
        }
        .spacing(12.0)
        .alignment(HorizontalAlignment::Start)
    }
}

mod keyboard_gallery;

mod geometry_gallery;
mod hover_gallery;

mod canvas_gallery;

mod scroll_gallery;

mod group_gallery;

mod decoration_gallery;

mod drag_drop_gallery;

mod tap_gallery;

mod property_inspector_gallery;

mod conditional_gallery;

mod shadow_gallery;

mod card_gallery;

mod graph_gallery;
