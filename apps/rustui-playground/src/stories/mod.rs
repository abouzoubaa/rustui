//! Stateful RustUI stories embedded in the canonical Playground.

use rustui::prelude::*;

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
mod dialog_gallery;
mod disclosure_gallery;
mod event_inspector;
mod feedback_gallery;
mod focus_gallery;
mod foundation_additions;
mod foundation_demos;
mod frame_gallery;
mod grid_overlays;
mod keyed_list_gallery;
mod layout_inspector;
mod lazy_grid_gallery;
mod locale_selector;
mod menu_gallery;
mod navigation_split_gallery;
mod offset_gallery;
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
mod table_reconciliation_gallery;
mod task_release;
mod test_runner;
mod theme_inspector;
mod toolbar_gallery;
mod tooltip_gallery;
mod tree_reconciliation_gallery;

use canvas::{RetentionStoryFields, StoryCanvas};
use child_component::DemoDisplayMode;
pub(crate) use component_catalog::{STORIES, StoryId};
use event_inspector::EventLog;
use locale_selector::LocaleSelector;
pub(crate) use locale_selector::{catalogs, initial_locale};
use properties_panel::ButtonProperties;

/// Returns existing display-control demos for their catalog entries.
pub(crate) fn catalog_demo(key: &str) -> Option<AnyView> {
    match key {
        "combo_box" => Some(AnyView::new(combo_box_reconciliation_gallery::demo())),
        "link" => Some(AnyView::new(supplementary_gallery::demo())),
        "alert_dialog" => Some(AnyView::new(alert_gallery::demo())),
        "save_dialog" => Some(AnyView::new(dialog_gallery::demo())),
        "sheet" => Some(AnyView::new(sheet_gallery::demo())),
        "toolbar" => Some(AnyView::new(toolbar_gallery::demo())),
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
    use super::catalog_demo;

    #[test]
    fn menu_catalog_routes_menu_and_context_menu_demos() {
        assert!(catalog_demo("menu").is_some());
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
    #[state(String::from("GPUI"))]
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

    fn toggle_details_tab(&mut self) {
        self.details_tab_present = !self.details_tab_present;
        if !self.details_tab_present && self.tab_selection == "details" {
            self.tab_selection = String::from("overview");
        }
        self.events.record(if self.details_tab_present {
            "Restored Details tab"
        } else {
            "Removed Details tab"
        });
    }

    fn toggle_tab_conditional_editor(&mut self) {
        self.tab_conditional_visible = !self.tab_conditional_visible;
        self.events.record(if self.tab_conditional_visible {
            "Shown Tabs conditional editor"
        } else {
            "Hidden Tabs conditional editor"
        });
    }

    fn toggle_navigation_conditional_editor(&mut self) {
        self.navigation_conditional_visible = !self.navigation_conditional_visible;
        self.events.record(if self.navigation_conditional_visible {
            "Shown Navigation conditional editor"
        } else {
            "Hidden Navigation conditional editor"
        });
    }

    fn toggle_keyed_order(&mut self) {
        self.keyed_rows_reversed = !self.keyed_rows_reversed;
        self.events.record(if self.keyed_rows_reversed {
            "Reversed keyed rows"
        } else {
            "Restored keyed row order"
        });
    }

    fn toggle_keyed_beta(&mut self) {
        self.keyed_beta_present = !self.keyed_beta_present;
        self.events.record(if self.keyed_beta_present {
            "Restored keyed Beta row"
        } else {
            "Removed keyed Beta row"
        });
    }

    fn toggle_child_titles(&mut self) {
        self.child_titles_prefixed = !self.child_titles_prefixed;
        self.events.record(if self.child_titles_prefixed {
            "Applied parent title prefix to child components"
        } else {
            "Restored child component titles"
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
        self.events
            .record(format!("W1 reset dispatched; run {}", self.w1_epoch));
        println!(
            r#"{{"kind":"rustui_w1","run":{},"step":0,"event":"reset_dispatched","visual":"not_checked"}}"#,
            self.w1_epoch
        );
    }

    fn next_w1_step(&mut self) {
        if !self.w1_parent_matches() {
            self.events
                .record("W1 parent state mismatch; reset W1 before continuing");
            println!(
                r#"{{"kind":"rustui_w1","run":{},"step":{},"event":"parent_state_mismatch"}}"#,
                self.w1_epoch,
                self.w1_step.saturating_sub(1)
            );
            return;
        }
        let (action, event) = match self.w1_step {
            1 => {
                self.w1_alpha_increment = 1;
                ("Alpha increment requested", "increment_alpha_dispatched")
            }
            2 => {
                self.w1_beta_increment = 1;
                ("Beta increment requested", "increment_beta_dispatched")
            }
            3 => {
                self.child_titles_prefixed = true;
                ("Title prefix enabled", "prefix_enabled")
            }
            4 => {
                self.child_titles_prefixed = false;
                ("Title prefix disabled", "prefix_disabled")
            }
            5 => {
                self.keyed_rows_reversed = true;
                ("Keyed order reversed", "order_reversed")
            }
            6 => {
                self.keyed_rows_reversed = false;
                ("Keyed order restored", "order_restored")
            }
            _ => return,
        };
        self.w1_step += 1;
        self.events.record(format!(
            "W1 step {}/6 dispatched: {action}",
            self.w1_step - 1
        ));
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
            self.events.record(format!(
                "W1 run {} visual check marked pass by operator",
                self.w1_epoch
            ));
            println!(
                r#"{{"kind":"rustui_w1","run":{},"step":6,"event":"visual_check","visual":"pass"}}"#,
                self.w1_epoch
            );
        }
    }

    fn fail_w1_manual_check(&mut self) {
        if self.w1_step == 7 {
            self.w1_manual_result = 2;
            self.events.record(format!(
                "W1 run {} visual check marked fail by operator",
                self.w1_epoch
            ));
            println!(
                r#"{{"kind":"rustui_w1","run":{},"step":6,"event":"visual_check","visual":"fail"}}"#,
                self.w1_epoch
            );
        }
    }

    fn toggle_environment_display_mode(&mut self) {
        self.environment_display_mode.toggle();
        self.events.record(format!(
            "RustUI environment display mode set to {}",
            self.environment_display_mode.label()
        ));
    }

    fn breadcrumb_overview(&mut self) {
        self.breadcrumb_page = String::from("Overview");
        self.events.record("Breadcrumb navigated to Overview");
    }

    fn breadcrumb_components(&mut self) {
        self.breadcrumb_page = String::from("Components");
        self.events.record("Breadcrumb navigated to Components");
    }

    fn increment(&mut self) {
        self.count += 1;
        self.events
            .record(format!("Incremented count to {}", self.count));
    }

    fn reset(&mut self) {
        self.count = 0;
        self.events.record("Reset story state");
    }
    fn set_picker_kotlin(&mut self) {
        self.picker_value = String::from("Kotlin");
        self.events.record("Picker owner value set to Kotlin");
    }

    fn cycle_button_style(&mut self) {
        self.button_properties.cycle_style();
        self.events.record(format!(
            "Button style changed to {:?}",
            self.button_properties.style
        ));
    }

    fn toggle_button_disabled(&mut self) {
        self.button_properties.disabled = !self.button_properties.disabled;
        self.events.record(if self.button_properties.disabled {
            "Button story disabled"
        } else {
            "Button story enabled"
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
            StoryId::Layout => HStack::new()
                .child(
                    Button(
                        if self.keyed_rows_reversed() {
                            "Restore row order"
                        } else {
                            "Reverse keyed rows"
                        },
                        self.action(Self::toggle_keyed_order),
                    )
                    .id("lab-toggle-keyed-order"),
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
                    .id("lab-toggle-keyed-beta"),
                )
                .child(
                    Button(
                        if self.child_titles_prefixed() {
                            "Restore child titles"
                        } else {
                            "Prefix child titles"
                        },
                        self.action(Self::toggle_child_titles),
                    )
                    .id("lab-toggle-child-titles"),
                )
                .child(
                    Button(
                        match self.environment_display_mode() {
                            DemoDisplayMode::Compact => "Use detailed environment",
                            DemoDisplayMode::Detailed => "Use compact environment",
                        },
                        self.action(Self::toggle_environment_display_mode),
                    )
                    .id("lab-toggle-environment-display-mode"),
                ),
            StoryId::Tabs => HStack::new()
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
            StoryId::Navigation => HStack::new().child(
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
            _ => HStack::new(),
        };
        let button_controls = if selected == StoryId::Button {
            HStack::new()
                .child(
                    Button("Cycle button style", self.action(Self::cycle_button_style))
                        .id("lab-cycle-style"),
                )
                .child(
                    Button("Toggle disabled", self.action(Self::toggle_button_disabled))
                        .id("lab-toggle-disabled"),
                )
        } else {
            HStack::new()
        };

        let w1_controls = if selected == StoryId::Layout {
            let step = self.w1_step();
            let parent_matches = self.w1_parent_matches();
            let result = match self.w1_manual_result() {
                1 => "operator marked pass",
                2 => "operator marked fail",
                _ => "visual check not recorded",
            };
            VStack::new()
                .child(Text(format!(
                    "W1 run {} · {}/6 actions dispatched · parent state {} · {}. Child counts and presentation require visual check.",
                    self.w1_epoch(),
                    step.saturating_sub(1),
                    if parent_matches { "matches expected" } else { "mismatch; reset required" },
                    result,
                )))
                .child(Text(format!(
                    "Expected after this step: Alpha {}, Beta {}. Inspect each step; avoid other controls during W1. Reset clears child counters and notes.",
                    u8::from(step >= 2),
                    u8::from(step >= 3),
                )))
                .child(
                    HStack::new()
                        .child(Button("Reset W1", self.action(Self::reset_w1)).id("w1-reset"))
                        .child(
                            Button("Next W1 step", self.action(Self::next_w1_step))
                                .id("w1-next-step")
                                .disabled(!(1..7).contains(&step) || !parent_matches),
                        )
                        .child(
                            Button("Mark visual pass", self.action(Self::pass_w1_manual_check))
                                .id("w1-visual-pass")
                                .disabled(step != 7 || !parent_matches),
                        )
                        .child(
                            Button("Mark visual fail", self.action(Self::fail_w1_manual_check))
                                .id("w1-visual-fail")
                                .disabled(step != 7),
                        ),
                )
                .spacing(8.0)
        } else {
            VStack::new()
        };

        VStack! {
        LocalizedText("lab-navigation-locales")
        LocaleSelector::new()
        retention_controls.spacing(8.0)
            w1_controls.spacing(8.0)
            (selected == StoryId::Layout && self.w1_step() == 0).then(|| Component("foundation-demos", foundation_demos::FoundationDemos::default))
        StoryCanvas::new(
            selected,
            self.count(),
            button_properties,
            self.input_text(),
            self.input_text_binding(),
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
        button_controls.spacing(8.0)
        Button("Clear event log", clear_events)
            .id("lab-clear-events")
        Text(layout_inspector::summary_for(selected))
        Text(test_runner::status_for(selected).label())
        Text("Public API source")
        Text(code_preview::source_for(selected))
        Text(self.events.summary())
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
