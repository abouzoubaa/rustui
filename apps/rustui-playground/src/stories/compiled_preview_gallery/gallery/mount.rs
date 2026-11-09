use super::*;

impl CompiledPreviewGallery {
    pub(in super::super) fn mount(&mut self, component: &str) {
        // Secure values are ephemeral and never enter the serializable preview
        // document. A remount clears the previous field value.
        self.secure_value.clear();
        let mut document = PreviewDocument::new(
            PreviewId::new("compiled-gallery").unwrap(),
            ComponentId::new(component).unwrap(),
            1,
        );
        if component == "color-picker-specimen" {
            document.properties.extend([
                (
                    PropertyId::new("label").unwrap(),
                    PropertyValue::String("Accent color".into()),
                ),
                (
                    PropertyId::new("has_color").unwrap(),
                    PropertyValue::Boolean(true),
                ),
                (
                    PropertyId::new("hue").unwrap(),
                    PropertyValue::Number(PREVIEW_COLOR_HUE),
                ),
                (
                    PropertyId::new("saturation").unwrap(),
                    PropertyValue::Number(PREVIEW_COLOR_SATURATION),
                ),
                (
                    PropertyId::new("lightness").unwrap(),
                    PropertyValue::Number(PREVIEW_COLOR_LIGHTNESS),
                ),
                (
                    PropertyId::new("alpha").unwrap(),
                    PropertyValue::Number(PREVIEW_COLOR_ALPHA),
                ),
            ]);
        } else if component == "picker" {
            self.text = "comfortable".into();
            document.properties.insert(
                PropertyId::new("selection").unwrap(),
                PropertyValue::String(self.text.clone()),
            );
        } else if component == "combo-box-specimen" {
            self.text = "rust".into();
            document.properties.insert(
                PropertyId::new("selection").unwrap(),
                PropertyValue::String(self.text.clone()),
            );
        } else if component == "token-field-specimen" {
            self.token_values = PREVIEW_TOKEN_FIELD_DEFAULTS
                .iter()
                .map(|token| (*token).to_owned())
                .collect();
            self.token_field_placeholder = "Add recipients".into();
            self.text = "Engineering".into();
            document.properties.extend([
                (
                    PropertyId::new("placeholder").unwrap(),
                    PropertyValue::String(self.token_field_placeholder.clone()),
                ),
                (
                    PropertyId::new("tokens_json").unwrap(),
                    PropertyValue::String(serde_json::to_string(&self.token_values).unwrap()),
                ),
                (
                    PropertyId::new("draft").unwrap(),
                    PropertyValue::String(self.text.clone()),
                ),
            ]);
        } else if component == "tab-view-specimen" {
            self.text = "overview".into();
            document.properties.insert(
                PropertyId::new("selection").unwrap(),
                PropertyValue::String(self.text.clone()),
            );
        } else if component == "stepper" {
            self.stepper_count = PREVIEW_STEPPER_DEFAULT;
            document.properties.insert(
                PropertyId::new("value").unwrap(),
                PropertyValue::Integer(i64::from(self.stepper_count)),
            );
        } else if component == "slider" {
            self.amount = PREVIEW_SLIDER_DEFAULT;
            self.amount_draft = self.amount.to_string();
            document.properties.insert(
                PropertyId::new("value").unwrap(),
                PropertyValue::Number(self.amount),
            );
        } else if component == "text-field-specimen" {
            self.text = "Preview text input".into();
            self.text_field_placeholder = "Preview text input".into();
            document.properties.insert(
                PropertyId::new("placeholder").unwrap(),
                PropertyValue::String(self.text_field_placeholder.clone()),
            );
            document.properties.insert(
                PropertyId::new("value").unwrap(),
                PropertyValue::String(self.text.clone()),
            );
        } else if component == "search-field-specimen" {
            self.text = "Search RustUI".into();
            self.search_field_placeholder = "Search RustUI".into();
            document.properties.insert(
                PropertyId::new("placeholder").unwrap(),
                PropertyValue::String(self.search_field_placeholder.clone()),
            );
            document.properties.insert(
                PropertyId::new("query").unwrap(),
                PropertyValue::String(self.text.clone()),
            );
        } else if component == "text-editor-specimen" {
            self.text = "First line\nSecond line".into();
            self.text_editor_placeholder = "Write a note".into();
            document.properties.insert(
                PropertyId::new("placeholder").unwrap(),
                PropertyValue::String(self.text_editor_placeholder.clone()),
            );
            document.properties.insert(
                PropertyId::new("value").unwrap(),
                PropertyValue::String(self.text.clone()),
            );
        } else if component == "code-editor-specimen" {
            self.text = "Plain text preview".into();
            document.properties.insert(
                PropertyId::new("value").unwrap(),
                PropertyValue::String(self.text.clone()),
            );
        } else if component == "secure-field-specimen" {
            self.secure_field_placeholder = "Enter a password".into();
            document.properties.insert(
                PropertyId::new("placeholder").unwrap(),
                PropertyValue::String(self.secure_field_placeholder.clone()),
            );
        } else if component == "progress" {
            self.text = "Preview upload".into();
            self.amount = PREVIEW_PROGRESS_DEFAULT;
            self.amount_draft = self.amount.to_string();
            document.properties.insert(
                PropertyId::new("label").unwrap(),
                PropertyValue::String(self.text.clone()),
            );
            document.properties.insert(
                PropertyId::new("value").unwrap(),
                PropertyValue::Number(self.amount),
            );
        } else if component == "text-specimen" {
            document.properties.insert(
                PropertyId::new("value").unwrap(),
                PropertyValue::String("Preview text".into()),
            );
        } else if component == "button-specimen" {
            self.text = "Preview action".into();
            self.enabled = true;
            document.properties.insert(
                PropertyId::new("label").unwrap(),
                PropertyValue::String(self.text.clone()),
            );
            document.properties.insert(
                PropertyId::new("enabled").unwrap(),
                PropertyValue::Boolean(self.enabled),
            );
        } else if component == "tap-region-specimen" {
            self.text = "Tap this text".into();
            self.enabled = true;
            document.properties.insert(
                PropertyId::new("label").unwrap(),
                PropertyValue::String(self.text.clone()),
            );
            document.properties.insert(
                PropertyId::new("enabled").unwrap(),
                PropertyValue::Boolean(self.enabled),
            );
        } else if component == "hover-region-specimen" {
            document.properties.insert(
                PropertyId::new("label").unwrap(),
                PropertyValue::String("Hover over this text".into()),
            );
        } else if component == "toolbar-specimen" {
            document.properties.extend([
                (
                    PropertyId::new("primary_label").unwrap(),
                    PropertyValue::String("Save".into()),
                ),
                (
                    PropertyId::new("secondary_label").unwrap(),
                    PropertyValue::String("Cancel".into()),
                ),
                (
                    PropertyId::new("secondary_disabled").unwrap(),
                    PropertyValue::Boolean(false),
                ),
            ]);
        } else if component == "image-specimen" {
            document.properties.extend([
                (
                    PropertyId::new("accessibility_label").unwrap(),
                    PropertyValue::String("Purple and cyan sample image".into()),
                ),
                (
                    PropertyId::new("fit").unwrap(),
                    PropertyValue::String("contain".into()),
                ),
            ]);
        } else if component == "breadcrumb-specimen" {
            document.properties.extend([
                (
                    PropertyId::new("parent_label").unwrap(),
                    PropertyValue::String("Components".into()),
                ),
                (
                    PropertyId::new("current_label").unwrap(),
                    PropertyValue::String("Image".into()),
                ),
            ]);
        } else if component == "context-menu-specimen" {
            document.properties.extend([
                (
                    PropertyId::new("target_label").unwrap(),
                    PropertyValue::String("Preview target".into()),
                ),
                (
                    PropertyId::new("action_label").unwrap(),
                    PropertyValue::String("Open preview".into()),
                ),
            ]);
        } else if component == "menu-specimen" {
            self.text = "Preview menu".into();
            document.properties.insert(
                PropertyId::new("trigger_label").unwrap(),
                PropertyValue::String(self.text.clone()),
            );
        } else if component == "link-specimen" {
            document.properties.extend([
                (
                    PropertyId::new("label").unwrap(),
                    PropertyValue::String("RustUI documentation".into()),
                ),
                (
                    PropertyId::new("url").unwrap(),
                    PropertyValue::String("https://example.com/docs".into()),
                ),
            ]);
        } else if component == "dialog-specimen" {
            self.text = "Open preview dialog".into();
            document.properties.insert(
                PropertyId::new("trigger_label").unwrap(),
                PropertyValue::String(self.text.clone()),
            );
        } else if component == "popover-specimen" {
            self.text = "Preview details".into();
            self.enabled = false;
            document.properties.extend([
                (
                    PropertyId::new("trigger_label").unwrap(),
                    PropertyValue::String(self.text.clone()),
                ),
                (
                    PropertyId::new("open").unwrap(),
                    PropertyValue::Boolean(self.enabled),
                ),
            ]);
        } else if component == "badge-specimen" {
            document.properties.extend([
                (
                    PropertyId::new("label").unwrap(),
                    PropertyValue::String("Inbox".into()),
                ),
                (PropertyId::new("count").unwrap(), PropertyValue::Integer(7)),
                (
                    PropertyId::new("dot").unwrap(),
                    PropertyValue::Boolean(false),
                ),
            ]);
        } else if component == "empty-state-specimen" {
            document.properties.extend([
                (
                    PropertyId::new("title").unwrap(),
                    PropertyValue::String("No results".into()),
                ),
                (
                    PropertyId::new("detail").unwrap(),
                    PropertyValue::String("Try changing your filters.".into()),
                ),
            ]);
        } else if component == "error-state-specimen" {
            document.properties.extend([
                (
                    PropertyId::new("title").unwrap(),
                    PropertyValue::String("Could not load".into()),
                ),
                (
                    PropertyId::new("detail").unwrap(),
                    PropertyValue::String("Check your connection and try again.".into()),
                ),
            ]);
        } else if component == "alert-specimen" {
            self.text = "Connection lost".into();
            self.enabled = true;
            document.properties.extend([
                (
                    PropertyId::new("message").unwrap(),
                    PropertyValue::String(self.text.clone()),
                ),
                (
                    PropertyId::new("visible").unwrap(),
                    PropertyValue::Boolean(self.enabled),
                ),
            ]);
        } else if component == "toast-specimen" {
            self.text = "Changes saved".into();
            document.properties.insert(
                PropertyId::new("message").unwrap(),
                PropertyValue::String(self.text.clone()),
            );
        } else if component == "sheet-specimen" {
            self.text = "Open details".into();
            document.properties.extend([
                (
                    PropertyId::new("trigger_label").unwrap(),
                    PropertyValue::String(self.text.clone()),
                ),
                (
                    PropertyId::new("title").unwrap(),
                    PropertyValue::String("Details".into()),
                ),
                (
                    PropertyId::new("body_text").unwrap(),
                    PropertyValue::String("Preview sheet content".into()),
                ),
            ]);
        } else if component == "drawer-specimen" {
            self.text = "Open details".into();
            document.properties.extend([
                (
                    PropertyId::new("trigger_label").unwrap(),
                    PropertyValue::String(self.text.clone()),
                ),
                (
                    PropertyId::new("title").unwrap(),
                    PropertyValue::String("Details".into()),
                ),
                (
                    PropertyId::new("body_text").unwrap(),
                    PropertyValue::String("Preview sheet content".into()),
                ),
            ]);
        } else if component == "sidebar-specimen" {
            document.properties.extend([
                (
                    PropertyId::new("home_label").unwrap(),
                    PropertyValue::String("Home".into()),
                ),
                (
                    PropertyId::new("settings_label").unwrap(),
                    PropertyValue::String("Settings".into()),
                ),
                (
                    PropertyId::new("selected_id").unwrap(),
                    PropertyValue::String("home".into()),
                ),
            ]);
        } else if component == "keyed-list-specimen" {
            document.properties.extend([
                (
                    PropertyId::new("first_label").unwrap(),
                    PropertyValue::String("First item".into()),
                ),
                (
                    PropertyId::new("second_label").unwrap(),
                    PropertyValue::String("Second item".into()),
                ),
                (
                    PropertyId::new("selected_id").unwrap(),
                    PropertyValue::String("first".into()),
                ),
            ]);
        } else if component == "data-grid-specimen" {
            document.properties.extend([
                (
                    PropertyId::new("label").unwrap(),
                    PropertyValue::String("Quarterly revenue".into()),
                ),
                (
                    PropertyId::new("selected_id").unwrap(),
                    PropertyValue::String("none".into()),
                ),
            ]);
        } else if component == "list-specimen" {
            document.properties.insert(
                PropertyId::new("selected_id").unwrap(),
                PropertyValue::String("none".into()),
            );
        } else if component == "table-specimen" {
            document.properties.insert(
                PropertyId::new("selected_id").unwrap(),
                PropertyValue::String("none".into()),
            );
        } else if matches!(component, "tree-specimen" | "outline-specimen") {
            document.properties.extend([
                (
                    PropertyId::new("root_label").unwrap(),
                    PropertyValue::String("Project".into()),
                ),
                (
                    PropertyId::new("src_label").unwrap(),
                    PropertyValue::String("src".into()),
                ),
                (
                    PropertyId::new("readme_label").unwrap(),
                    PropertyValue::String("README.md".into()),
                ),
                (
                    PropertyId::new("tests_label").unwrap(),
                    PropertyValue::String("tests".into()),
                ),
                (
                    PropertyId::new("selected_id").unwrap(),
                    PropertyValue::String("none".into()),
                ),
            ]);
        } else if component == "navigation-split-specimen" {
            document.properties.insert(
                PropertyId::new("selected_id").unwrap(),
                PropertyValue::String("none".into()),
            );
        } else if component == "navigation-stack-specimen" {
            document.properties.insert(
                PropertyId::new("selected_id").unwrap(),
                PropertyValue::String("none".into()),
            );
        } else if component == "collection-specimen" {
            document.properties.insert(
                PropertyId::new("selected_id").unwrap(),
                PropertyValue::String("none".into()),
            );
        } else if component == "canvas-specimen" {
            document.properties.insert(
                PropertyId::new("composition").unwrap(),
                PropertyValue::String("basic".into()),
            );
        } else if component == "localized-text-specimen" {
            document.properties.insert(
                PropertyId::new("message_id").unwrap(),
                PropertyValue::String(PREVIEW_LOCALIZED_TEXT_DEFAULT.into()),
            );
        } else if component == "disclosure-specimen" {
            self.text = "More information".into();
            self.enabled = false;
            document.properties.extend([
                (
                    PropertyId::new("label").unwrap(),
                    PropertyValue::String(self.text.clone()),
                ),
                (
                    PropertyId::new("open").unwrap(),
                    PropertyValue::Boolean(self.enabled),
                ),
            ]);
        } else if component == "icon-specimen" {
            document.properties.insert(
                PropertyId::new("accessibility_label").unwrap(),
                PropertyValue::String("Search".into()),
            );
        } else if component == "date-picker-specimen" {
            // Label and empty optional date come from the catalog defaults.
        } else if component == "avatar-specimen" {
            document.properties.extend([
                (
                    PropertyId::new("name").unwrap(),
                    PropertyValue::String("Ada Lovelace".into()),
                ),
                (
                    PropertyId::new("diameter").unwrap(),
                    PropertyValue::Number(40.0),
                ),
            ]);
        } else if matches!(
            component,
            "animated-opacity-specimen"
                | "spring-opacity-specimen"
                | "group-specimen"
                | "shadow-specimen"
                | "background-specimen"
                | "border-specimen"
                | "semantics-specimen"
                | "card-specimen"
                | "container-specimen"
                | "split-view-specimen"
                | "frame-specimen"
                | "hstack-specimen"
                | "vstack-specimen"
                | "grid-specimen"
                | "lazy-grid-specimen"
                | "spacer-specimen"
                | "zstack-specimen"
                | "geometry-reader-specimen"
                | "property-inspector-specimen"
        ) {
            // These specimens take their initial values from the schema defaults.
        } else if component == "scroll-view-specimen" {
            // The specimen's initial properties come from its bounded schema defaults.
        }

        let component_schema = self.host.catalog().components.get(&document.component_id);
        let mut seed_generic_property = |name: &str, value: PropertyValue| {
            let property = PropertyId::new(name).unwrap();
            if component_schema.is_some_and(|schema| schema.properties.contains_key(&property)) {
                document.properties.entry(property).or_insert(value);
            }
        };
        if matches!(component, "toggle" | "checkbox" | "radio") {
            let value_property = if component == "radio" {
                "selected"
            } else {
                "checked"
            };
            seed_generic_property("label", PropertyValue::String(self.text.clone()));
            seed_generic_property(value_property, PropertyValue::Boolean(self.enabled));
        } else {
            seed_generic_property("text", PropertyValue::String(self.text.clone()));
            seed_generic_property("enabled", PropertyValue::Boolean(self.enabled));
            seed_generic_property("amount", PropertyValue::Number(self.amount));
        }
        if matches!(component, "button" | "button-specimen") {
            document.actions.insert(
                EventId::new("activate").unwrap(),
                ActionRef::new("increment").unwrap(),
            );
        }
        if component == "tap-region-specimen" {
            document.actions.insert(
                EventId::new("tap").unwrap(),
                ActionRef::new("increment").unwrap(),
            );
        }
        if component == "hover-region-specimen" {
            document.actions.extend([
                (
                    EventId::new("enter").unwrap(),
                    ActionRef::new("increment").unwrap(),
                ),
                (
                    EventId::new("exit").unwrap(),
                    ActionRef::new("increment").unwrap(),
                ),
            ]);
        }
        if component == "menu-specimen" {
            document.actions.insert(
                EventId::new("select").unwrap(),
                ActionRef::new("increment").unwrap(),
            );
        }
        if component == "link-specimen" {
            document.actions.insert(
                EventId::new("activate").unwrap(),
                ActionRef::new("increment").unwrap(),
            );
        }
        if component == "alert-specimen" {
            document.actions.insert(
                EventId::new("dismiss").unwrap(),
                ActionRef::new("dismiss").unwrap(),
            );
        }
        if component == "disclosure-specimen" {
            document.actions.insert(
                EventId::new("toggle").unwrap(),
                ActionRef::new("toggle").unwrap(),
            );
        }
        if component == "toolbar-specimen" {
            document.actions.extend([
                (
                    EventId::new("primary").unwrap(),
                    ActionRef::new("toolbar_primary").unwrap(),
                ),
                (
                    EventId::new("secondary").unwrap(),
                    ActionRef::new("toolbar_secondary").unwrap(),
                ),
            ]);
        }
        if component == "breadcrumb-specimen" {
            document.actions.insert(
                EventId::new("parent").unwrap(),
                ActionRef::new("breadcrumb_parent").unwrap(),
            );
        }
        if component == "context-menu-specimen" {
            document.actions.insert(
                EventId::new("activate").unwrap(),
                ActionRef::new("context_menu_open_preview").unwrap(),
            );
        }
        if component == "toast-specimen" {
            document.actions.insert(
                EventId::new("show").unwrap(),
                ActionRef::new("toast_show").unwrap(),
            );
        }
        if component == "dialog-specimen" {
            for event in ["confirm", "cancel"] {
                document.actions.insert(
                    EventId::new(event).unwrap(),
                    ActionRef::new("increment").unwrap(),
                );
            }
        }
        self.status = match self.host.mount(document) {
            Ok(stamp) => {
                // A new lease per generation keeps callbacks from an older mount stale.
                self.stamp_lease = Rc::new(RefCell::new(Some(stamp.clone())));
                self.refresh_number_drafts();
                let status = format!("Mounted {component}, generation {}", stamp.generation);
                self.status_presentation = Some(PreviewStatusPresentation {
                    status: status.clone(),
                    key: "lab-preview-status-mounted",
                    subject: component.into(),
                    generation: stamp.generation.to_string(),
                });
                status
            }
            Err(error) => error.to_string(),
        };
    }
    pub(super) fn mount_text(&mut self) {
        self.mount("text");
    }
    pub(super) fn mount_text_specimen(&mut self) {
        self.mount("text-specimen");
    }

    pub(super) fn mount_localized_text_specimen(&mut self) {
        self.mount("localized-text-specimen");
    }
    pub(super) fn mount_button(&mut self) {
        self.mount("button");
    }
    pub(super) fn mount_field(&mut self) {
        self.mount("text-field");
    }
    pub(super) fn mount_toggle(&mut self) {
        self.mount("toggle");
    }
    pub(super) fn mount_checkbox(&mut self) {
        self.mount("checkbox");
    }
    pub(super) fn mount_radio(&mut self) {
        self.mount("radio");
    }
    pub(super) fn mount_picker(&mut self) {
        self.mount("picker");
    }
    pub(super) fn mount_combo_box_specimen(&mut self) {
        self.mount("combo-box-specimen");
    }
    pub(super) fn mount_tab_view_specimen(&mut self) {
        self.mount("tab-view-specimen");
    }
    pub(super) fn mount_slider(&mut self) {
        self.mount("slider");
    }

    pub(super) fn mount_progress(&mut self) {
        self.mount("progress");
    }

    pub(super) fn mount_stepper(&mut self) {
        self.mount("stepper");
    }

    pub(super) fn mount_text_field_specimen(&mut self) {
        self.mount("text-field-specimen");
    }
    pub(super) fn mount_search_field_specimen(&mut self) {
        self.mount("search-field-specimen");
    }
    pub(super) fn mount_token_field_specimen(&mut self) {
        self.mount("token-field-specimen");
    }
    pub(super) fn mount_text_editor_specimen(&mut self) {
        self.mount("text-editor-specimen");
    }
    pub(super) fn mount_code_editor_specimen(&mut self) {
        self.mount("code-editor-specimen");
    }

    pub(super) fn mount_secure_field_specimen(&mut self) {
        self.mount("secure-field-specimen");
    }
    pub(super) fn mount_button_specimen(&mut self) {
        self.mount("button-specimen");
    }
    pub(super) fn mount_toolbar_specimen(&mut self) {
        self.mount("toolbar-specimen");
    }
    pub(super) fn mount_image_specimen(&mut self) {
        self.mount("image-specimen");
    }
    pub(super) fn mount_breadcrumb_specimen(&mut self) {
        self.mount("breadcrumb-specimen");
    }
    pub(super) fn mount_context_menu_specimen(&mut self) {
        self.mount("context-menu-specimen");
    }
    pub(super) fn mount_menu_specimen(&mut self) {
        self.mount("menu-specimen");
    }
    pub(super) fn mount_link_specimen(&mut self) {
        self.mount("link-specimen");
    }
    pub(super) fn mount_alert_specimen(&mut self) {
        self.mount("alert-specimen");
    }
    pub(super) fn mount_toast_specimen(&mut self) {
        self.mount("toast-specimen");
    }
    pub(super) fn mount_sheet_specimen(&mut self) {
        self.mount("sheet-specimen");
    }
    pub(super) fn mount_drawer_specimen(&mut self) {
        self.mount("drawer-specimen");
    }
    pub(super) fn mount_sidebar_specimen(&mut self) {
        self.mount("sidebar-specimen");
    }
    pub(super) fn mount_keyed_list_specimen(&mut self) {
        self.mount("keyed-list-specimen");
    }

    pub(super) fn mount_data_grid_specimen(&mut self) {
        self.mount("data-grid-specimen");
    }

    pub(super) fn mount_list_specimen(&mut self) {
        self.mount("list-specimen");
    }

    pub(super) fn mount_table_specimen(&mut self) {
        self.mount("table-specimen");
    }

    pub(super) fn mount_tree_specimen(&mut self) {
        self.mount("tree-specimen");
    }

    pub(super) fn mount_outline_specimen(&mut self) {
        self.mount("outline-specimen");
    }
    pub(super) fn mount_navigation_split_specimen(&mut self) {
        self.mount("navigation-split-specimen");
    }

    pub(super) fn mount_navigation_stack_specimen(&mut self) {
        self.mount("navigation-stack-specimen");
    }

    pub(super) fn mount_collection_specimen(&mut self) {
        self.mount("collection-specimen");
    }
    pub(super) fn mount_canvas_specimen(&mut self) {
        self.mount("canvas-specimen");
    }
    pub(super) fn mount_disclosure_specimen(&mut self) {
        self.mount("disclosure-specimen");
    }
    pub(super) fn mount_dialog_specimen(&mut self) {
        self.mount("dialog-specimen");
    }
    pub(super) fn mount_popover_specimen(&mut self) {
        self.mount("popover-specimen");
    }
    pub(super) fn mount_date_picker_specimen(&mut self) {
        self.mount("date-picker-specimen");
    }

    pub(super) fn mount_property_inspector_specimen(&mut self) {
        self.mount("property-inspector-specimen");
    }

    pub(super) fn mount_group_specimen(&mut self) {
        self.mount("group-specimen");
    }

    pub(super) fn mount_animated_opacity_specimen(&mut self) {
        self.mount("animated-opacity-specimen");
    }

    pub(super) fn mount_spring_opacity_specimen(&mut self) {
        self.mount("spring-opacity-specimen");
    }

    pub(super) fn mount_shadow_specimen(&mut self) {
        self.mount("shadow-specimen");
    }

    pub(super) fn mount_background_specimen(&mut self) {
        self.mount("background-specimen");
    }

    pub(super) fn mount_border_specimen(&mut self) {
        self.mount("border-specimen");
    }

    pub(super) fn mount_semantics_specimen(&mut self) {
        self.mount("semantics-specimen");
    }

    pub(super) fn mount_card_specimen(&mut self) {
        self.mount("card-specimen");
    }

    pub(super) fn mount_container_specimen(&mut self) {
        self.mount("container-specimen");
    }

    pub(super) fn mount_split_view_specimen(&mut self) {
        self.mount("split-view-specimen");
    }

    pub(super) fn mount_frame_specimen(&mut self) {
        self.mount("frame-specimen");
    }

    pub(super) fn mount_hstack_specimen(&mut self) {
        self.mount("hstack-specimen");
    }

    pub(super) fn mount_vstack_specimen(&mut self) {
        self.mount("vstack-specimen");
    }

    pub(super) fn mount_grid_specimen(&mut self) {
        self.mount("grid-specimen");
    }

    pub(super) fn mount_lazy_grid_specimen(&mut self) {
        self.mount("lazy-grid-specimen");
    }

    pub(super) fn mount_spacer_specimen(&mut self) {
        self.mount("spacer-specimen");
    }

    pub(super) fn mount_geometry_reader_specimen(&mut self) {
        self.mount("geometry-reader-specimen");
    }

    #[cfg(test)]
    pub(super) fn mount_scroll_view_specimen(&mut self) {
        self.mount("scroll-view-specimen");
    }
    pub(super) fn mount_avatar_specimen(&mut self) {
        self.mount("avatar-specimen");
    }
    pub(super) fn mount_empty_state_specimen(&mut self) {
        self.mount("empty-state-specimen");
    }
    pub(super) fn mount_error_state_specimen(&mut self) {
        self.mount("error-state-specimen");
    }

    pub(super) fn mount_badge_specimen(&mut self) {
        self.mount("badge-specimen");
    }

    pub(super) fn mount_label_specimen(&mut self) {
        self.mount("label-specimen");
    }

    pub(super) fn mount_tooltip_specimen(&mut self) {
        self.mount("tooltip-specimen");
    }

    pub(super) fn mount_spinner_specimen(&mut self) {
        self.mount("spinner-specimen");
    }

    pub(super) fn mount_divider_specimen(&mut self) {
        self.mount("divider-specimen");
    }

    pub(super) fn mount_skeleton_specimen(&mut self) {
        self.mount("skeleton-specimen");
    }
    pub(super) fn mount_icon_specimen(&mut self) {
        self.mount("icon-specimen");
    }
    pub(super) fn mount_color_picker_specimen(&mut self) {
        self.mount("color-picker-specimen");
    }
    pub(super) fn rerender(&mut self) {
        self.renders += 1;
    }

    pub(super) fn finish_delayed_rerender(&mut self, state: AsyncState<(), String>) -> bool {
        match state {
            AsyncState::Success(()) => {
                self.renders += 1;
                self.status = "Completed delayed parent rerender".into();
                true
            }
            AsyncState::Error(error) => {
                self.status = format!("Delayed parent rerender failed: {error}");
                true
            }
            AsyncState::Idle | AsyncState::Loading => false,
        }
    }
    pub(in super::super) fn export_descriptor(&mut self) {
        let Some(snapshot) = self.host.snapshot() else {
            self.status = "No mounted preview to export".into();
            return;
        };
        // The descriptor is the editable/importable representation; export
        // resolved values from the live host so accepted binding feedback is
        // included even if a compiled factory rendered an older snapshot.
        let mut document = snapshot.preview.document.clone();
        document.properties.clone_from(&snapshot.preview.properties);
        match encode_preview(&document, self.host.catalog()) {
            Ok(json) => {
                self.descriptor_json = json;
                self.status = "Exported normalized preview descriptor".into();
            }
            Err(error) => self.status = format!("Export rejected: {error}"),
        }
    }
    pub(super) fn import_descriptor(&mut self) {
        let parsed = match decode_preview(&self.descriptor_json, self.host.catalog()) {
            Ok(preview) => preview.document,
            Err(error) => {
                self.status = format!("Import rejected: {error}");
                return;
            }
        };
        if parsed.component_id.as_str() == "date-picker-specimen"
            && let Some(PropertyValue::String(value)) =
                parsed.properties.get(&PropertyId::new("date").unwrap())
            && parse_preview_date(value).is_err()
        {
            self.status =
                "Import rejected: date must be empty or a canonical YYYY-MM-DD date".into();
            return;
        }
        if parsed.component_id.as_str() == "tooltip-specimen"
            && let Some(PropertyValue::String(text)) =
                parsed.properties.get(&PropertyId::new("text").unwrap())
            && text.trim().is_empty()
        {
            self.status = "Import rejected: Tooltip text must not be blank".into();
            return;
        }
        if parsed.component_id.as_str() == "stepper"
            && let Some(PropertyValue::String(label)) =
                parsed.properties.get(&PropertyId::new("label").unwrap())
            && (label.trim().is_empty() || label.len() > PREVIEW_STEPPER_LABEL_MAX_LENGTH)
        {
            self.status = format!(
                "Import rejected: Stepper label must be nonblank and at most {PREVIEW_STEPPER_LABEL_MAX_LENGTH} bytes"
            );
            return;
        }
        if parsed.component_id.as_str() == "sidebar-specimen" {
            for name in ["home_label", "settings_label"] {
                if let Some(PropertyValue::String(label)) =
                    parsed.properties.get(&PropertyId::new(name).unwrap())
                    && label.trim().is_empty()
                {
                    self.status = format!("Import rejected: Sidebar {name} must not be blank");
                    return;
                }
            }
        }
        if parsed.component_id.as_str() == "data-grid-specimen"
            && let Some(PropertyValue::String(label)) =
                parsed.properties.get(&PropertyId::new("label").unwrap())
            && label.trim().is_empty()
        {
            self.status = "Import rejected: DataGrid label must not be blank".into();
            return;
        }
        if matches!(
            parsed.component_id.as_str(),
            "tree-specimen" | "outline-specimen"
        ) {
            for name in ["root_label", "src_label", "readme_label", "tests_label"] {
                if let Some(PropertyValue::String(label)) =
                    parsed.properties.get(&PropertyId::new(name).unwrap())
                    && label.trim().is_empty()
                {
                    self.status = format!(
                        "Import rejected: {} {name} must not be blank",
                        if parsed.component_id.as_str() == "outline-specimen" {
                            "Outline"
                        } else {
                            "Tree"
                        }
                    );
                    return;
                }
            }
        }
        let uses_label_checked = matches!(parsed.component_id.as_str(), "toggle" | "checkbox");
        let uses_button_specimen = parsed.component_id.as_str() == "button-specimen";
        let uses_tap_region_specimen = parsed.component_id.as_str() == "tap-region-specimen";
        let uses_hover_region_specimen = parsed.component_id.as_str() == "hover-region-specimen";
        let uses_menu_specimen = parsed.component_id.as_str() == "menu-specimen";
        let uses_dialog_specimen = parsed.component_id.as_str() == "dialog-specimen";
        let uses_popover_specimen = parsed.component_id.as_str() == "popover-specimen";
        let uses_alert_specimen = parsed.component_id.as_str() == "alert-specimen";
        let uses_toast_specimen = parsed.component_id.as_str() == "toast-specimen";
        let uses_sheet_specimen = matches!(
            parsed.component_id.as_str(),
            "sheet-specimen" | "drawer-specimen"
        );
        let uses_disclosure_specimen = parsed.component_id.as_str() == "disclosure-specimen";
        let uses_toolbar_specimen = parsed.component_id.as_str() == "toolbar-specimen";
        let uses_image_specimen = parsed.component_id.as_str() == "image-specimen";
        let uses_breadcrumb_specimen = parsed.component_id.as_str() == "breadcrumb-specimen";
        let uses_context_menu_specimen = parsed.component_id.as_str() == "context-menu-specimen";
        let uses_text_specimen = parsed.component_id.as_str() == "text-specimen";
        let uses_radio = parsed.component_id.as_str() == "radio";
        let uses_selection = matches!(
            parsed.component_id.as_str(),
            "picker" | "combo-box-specimen" | "tab-view-specimen"
        );
        let uses_slider = parsed.component_id.as_str() == "slider";
        let uses_progress = parsed.component_id.as_str() == "progress";
        let uses_stepper = parsed.component_id.as_str() == "stepper";
        let uses_text_field_specimen = parsed.component_id.as_str() == "text-field-specimen";
        let uses_search_field_specimen = parsed.component_id.as_str() == "search-field-specimen";
        let uses_text_editor_specimen = parsed.component_id.as_str() == "text-editor-specimen";
        let uses_code_editor_specimen = parsed.component_id.as_str() == "code-editor-specimen";
        let uses_secure_field_specimen = parsed.component_id.as_str() == "secure-field-specimen";
        let uses_token_field = parsed.component_id.as_str() == "token-field-specimen";
        let uses_color_picker_specimen = parsed.component_id.as_str() == "color-picker-specimen";
        if uses_token_field
            && let Some(PropertyValue::String(tokens_json)) = parsed
                .properties
                .get(&PropertyId::new("tokens_json").unwrap())
            && parse_preview_token_values(tokens_json).is_err()
        {
            self.status =
                "Import rejected: TokenField tokens_json must be a JSON string array".into();
            return;
        }
        if parsed.component_id.as_str() == "empty-state-specimen"
            && let Some(PropertyValue::String(title)) =
                parsed.properties.get(&PropertyId::new("title").unwrap())
            && title.trim().is_empty()
        {
            self.status = "Import rejected: EmptyState title must not be blank".into();
            return;
        }
        if parsed.component_id.as_str() == "error-state-specimen"
            && let Some(PropertyValue::String(title)) =
                parsed.properties.get(&PropertyId::new("title").unwrap())
            && title.trim().is_empty()
        {
            self.status = "Import rejected: ErrorState title must not be blank".into();
            return;
        }
        if parsed.component_id.as_str() == "alert-specimen"
            && let Some(PropertyValue::String(message)) =
                parsed.properties.get(&PropertyId::new("message").unwrap())
            && message.trim().is_empty()
        {
            self.status = "Import rejected: Alert message must not be blank".into();
            return;
        }
        if uses_toast_specimen {
            for name in ["message", "title"] {
                if let Some(PropertyValue::String(value)) =
                    parsed.properties.get(&PropertyId::new(name).unwrap())
                    && value.trim().is_empty()
                {
                    self.status = format!("Import rejected: Toast {name} must not be blank");
                    return;
                }
            }
        }
        if uses_sheet_specimen {
            for name in ["trigger_label", "title", "body_text"] {
                if let Some(PropertyValue::String(value)) =
                    parsed.properties.get(&PropertyId::new(name).unwrap())
                    && value.trim().is_empty()
                {
                    self.status = format!(
                        "Import rejected: {} {name} must not be blank",
                        if parsed.component_id.as_str() == "drawer-specimen" {
                            "Drawer"
                        } else {
                            "Sheet"
                        }
                    );
                    return;
                }
            }
        }
        if uses_popover_specimen
            && let Some(PropertyValue::String(label)) = parsed
                .properties
                .get(&PropertyId::new("trigger_label").unwrap())
            && label.trim().is_empty()
        {
            self.status = "Import rejected: Popover trigger label must not be blank".into();
            return;
        }
        if uses_disclosure_specimen
            && let Some(PropertyValue::String(label)) =
                parsed.properties.get(&PropertyId::new("label").unwrap())
            && label.trim().is_empty()
        {
            self.status = "Import rejected: Disclosure label must not be blank".into();
            return;
        }
        if uses_toolbar_specimen {
            for name in ["primary_label", "secondary_label"] {
                if let Some(PropertyValue::String(label)) =
                    parsed.properties.get(&PropertyId::new(name).unwrap())
                    && label.trim().is_empty()
                {
                    self.status = format!("Import rejected: Toolbar {name} must not be blank");
                    return;
                }
            }
        }
        if uses_image_specimen
            && let Some(PropertyValue::String(label)) = parsed
                .properties
                .get(&PropertyId::new("accessibility_label").unwrap())
            && label.trim().is_empty()
        {
            self.status = "Import rejected: Image accessibility label must not be blank".into();
            return;
        }
        if uses_breadcrumb_specimen {
            for name in ["parent_label", "current_label"] {
                if let Some(PropertyValue::String(label)) =
                    parsed.properties.get(&PropertyId::new(name).unwrap())
                    && label.trim().is_empty()
                {
                    self.status = format!("Import rejected: Breadcrumb {name} must not be blank");
                    return;
                }
            }
        }
        if uses_context_menu_specimen {
            for name in ["target_label", "action_label"] {
                if let Some(PropertyValue::String(label)) =
                    parsed.properties.get(&PropertyId::new(name).unwrap())
                    && label.trim().is_empty()
                {
                    self.status = format!("Import rejected: ContextMenu {name} must not be blank");
                    return;
                }
            }
        }
        if uses_text_specimen
            && let Some(PropertyValue::String(value)) =
                parsed.properties.get(&PropertyId::new("value").unwrap())
            && value.trim().is_empty()
        {
            self.status = "Import rejected: Text value must not be blank".into();
            return;
        }
        match self.host.mount(parsed) {
            Ok(stamp) => {
                self.secure_value.clear();
                self.refresh_number_drafts();
                if let Some(snapshot) = self.host.snapshot() {
                    if !uses_secure_field_specimen {
                        self.text = property_string(
                            snapshot,
                            if uses_label_checked
                                || uses_radio
                                || uses_progress
                                || uses_button_specimen
                                || uses_tap_region_specimen
                                || uses_hover_region_specimen
                                || uses_color_picker_specimen
                            {
                                "label"
                            } else if uses_alert_specimen || uses_toast_specimen {
                                "message"
                            } else if uses_sheet_specimen {
                                "trigger_label"
                            } else if uses_disclosure_specimen {
                                "label"
                            } else if uses_toolbar_specimen {
                                "primary_label"
                            } else if uses_image_specimen {
                                "accessibility_label"
                            } else if uses_breadcrumb_specimen {
                                "current_label"
                            } else if uses_context_menu_specimen {
                                "target_label"
                            } else if uses_text_specimen {
                                "value"
                            } else if uses_menu_specimen
                                || uses_dialog_specimen
                                || uses_popover_specimen
                            {
                                "trigger_label"
                            } else if uses_selection {
                                "selection"
                            } else if uses_search_field_specimen {
                                "query"
                            } else if uses_token_field {
                                "draft"
                            } else if uses_text_field_specimen
                                || uses_text_editor_specimen
                                || uses_code_editor_specimen
                            {
                                "value"
                            } else {
                                "text"
                            },
                        )
                        .unwrap_or_default();
                    } else {
                        self.secure_field_placeholder =
                            property_string(snapshot, "placeholder").unwrap_or_default();
                    }
                    if uses_text_field_specimen {
                        self.text_field_placeholder =
                            property_string(snapshot, "placeholder").unwrap_or_default();
                    }
                    if uses_search_field_specimen {
                        self.search_field_placeholder =
                            property_string(snapshot, "placeholder").unwrap_or_default();
                    }
                    if uses_text_editor_specimen {
                        self.text_editor_placeholder =
                            property_string(snapshot, "placeholder").unwrap_or_default();
                    }
                    if uses_token_field {
                        self.token_field_placeholder =
                            property_string(snapshot, "placeholder").unwrap_or_default();
                        self.token_values = property_string(snapshot, "tokens_json")
                            .and_then(|value| parse_preview_token_values(&value).ok())
                            .unwrap_or_default();
                    }
                    self.enabled = property_boolean(
                        snapshot,
                        if uses_radio {
                            "selected"
                        } else if uses_label_checked {
                            "checked"
                        } else if uses_alert_specimen {
                            "visible"
                        } else if uses_disclosure_specimen {
                            "open"
                        } else if uses_toolbar_specimen {
                            "secondary_disabled"
                        } else if uses_popover_specimen {
                            "open"
                        } else {
                            "enabled"
                        },
                    )
                    .unwrap_or(true);
                    self.amount = property_number(
                        snapshot,
                        if uses_slider || uses_progress {
                            "value"
                        } else {
                            "amount"
                        },
                    )
                    .unwrap_or(0.375);
                    self.amount_draft = self.amount.to_string();
                    if uses_stepper {
                        if let Some(count) = property_integer(snapshot, "value")
                            .and_then(|value| i32::try_from(value).ok())
                        {
                            self.stepper_count = count;
                        }
                    }
                }
                self.stamp_lease = Rc::new(RefCell::new(Some(stamp.clone())));
                self.status = format!(
                    "Imported descriptor for {} · generation {}",
                    stamp.preview_id, stamp.generation
                );
                let status = self.status.clone();
                self.export_descriptor();
                self.status_presentation = Some(PreviewStatusPresentation {
                    status: status.clone(),
                    key: "lab-preview-status-imported",
                    subject: stamp.preview_id.to_string(),
                    generation: stamp.generation.to_string(),
                });
                self.status = status;
            }
            Err(error) => self.status = format!("Import rejected: {error}"),
        }
    }
}
