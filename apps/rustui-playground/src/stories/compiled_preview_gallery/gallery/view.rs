impl View for CompiledPreviewGallery {
    fn on_mount(&mut self, cx: &mut rustui::App) {
        #[cfg(target_family = "wasm")]
        if let Some(bridge) = &self.reference_bridge {
            bridge.install(cx);
        }
        #[cfg(not(target_family = "wasm"))]
        let _ = cx;
    }
    fn on_unmount(&mut self, _cx: &mut rustui::App) {
        if let Some(mut session) = self.reference_session.take() {
            session.close(crate::preview_transport::Host::Preview(&mut self.host));
            *self.stamp_lease.borrow_mut() = None;
            self.secure_value.clear();
        }
        #[cfg(target_family = "wasm")]
        if let Some(bridge) = self.reference_bridge.take() {
            bridge.close();
        }
    }
    fn body(&self) -> impl View {
        let render_preview = self.host.render_factory();
        let inspector_rows = self
            .host
            .snapshot()
            .and_then(|snapshot| {
                self.host
                    .catalog()
                    .components
                    .get(&snapshot.preview.document.component_id)
            })
            .map(|schema| {
                let generation = self
                    .host
                    .snapshot()
                    .map(|snapshot| snapshot.stamp.generation)
                    .unwrap_or_default();
                schema
                    .properties
                    .iter()
                    .filter(|(_, property)| preview_inspector_property_is_visible(property))
                    .filter_map(|(id, property)| {
                        // A remount is a new binding lease. Scope each editor
                        // to that generation so retained input state cannot
                        // keep the previous generation's captured lease.
                        let row_id = format!("{}-generation-{generation}", id.as_str());
                        let context = PropertyBindingContext {
                            property: id.clone(),
                            stamp: self.stamp_lease.clone(),
                        };
                        match property.kind {
                            PropertyKind::String => {
                                let snapshot = self.host.snapshot();
                                let value = Binding::new_with(
                                    preview_inspector_string_initial_value(snapshot, id.as_str()),
                                    context,
                                    Self::read_text,
                                    Self::write_text,
                                );
                                if property.allowed_values.is_empty() {
                                    Some(PropertyInspectorRow::text(
                                        row_id.clone(),
                                        id.as_str(),
                                        value,
                                    ))
                                } else {
                                    Some(PropertyInspectorRow::choice(
                                        row_id.clone(),
                                        id.as_str(),
                                        value,
                                        property.allowed_values.clone(),
                                    ))
                                }
                            }
                            PropertyKind::Boolean => {
                                let value = if schema.id.as_str() == "color-picker-specimen"
                                    && id.as_str() == "has_color"
                                {
                                    Binding::new_with(
                                        self.read_color_presence(&context),
                                        context,
                                        Self::read_color_presence,
                                        Self::write_color_presence,
                                    )
                                } else {
                                    self.preview_inspector_boolean_binding(context)
                                };
                                Some(PropertyInspectorRow::toggle(
                                    row_id.clone(),
                                    id.as_str(),
                                    value,
                                ))
                            }
                            PropertyKind::Integer => Some(PropertyInspectorRow::integer_value(
                                row_id,
                                format!("{} (integer)", id.as_str()),
                                Binding::new_with(
                                    self.host
                                        .snapshot()
                                        .and_then(|snapshot| {
                                            property_integer(&snapshot, id.as_str())
                                        })
                                        .unwrap_or_default(),
                                    context,
                                    Self::read_integer_property,
                                    Self::write_integer_property,
                                ),
                            )),
                            PropertyKind::Number => Some(PropertyInspectorRow::text(
                                row_id,
                                format!("{} (f64)", id.as_str()),
                                Binding::new_with(
                                    self.read_number_draft(&context),
                                    context,
                                    Self::read_number_draft,
                                    Self::write_number_draft,
                                ),
                            )),
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let descriptor_binding = Binding::new::<Self>(
            self.descriptor_json.clone(),
            Self::read_descriptor,
            Self::write_descriptor,
        );
        let binding = self.text_binding();
        let token_values_binding = Binding::new_with(
            self.token_values.clone(),
            PropertyBindingContext {
                property: PropertyId::new("tokens_json").unwrap(),
                stamp: self.stamp_lease.clone(),
            },
            Self::read_token_values,
            Self::write_token_values,
        );
        let secure_value_binding = Binding::new_with(
            self.secure_value.clone(),
            PropertyBindingContext {
                property: PropertyId::new("placeholder").unwrap(),
                stamp: self.stamp_lease.clone(),
            },
            CompiledPreviewGallery::read_secure_value,
            CompiledPreviewGallery::write_secure_value,
        );
        let checked_binding = Binding::new_with(
            self.enabled,
            PropertyBindingContext {
                property: if self.host.snapshot().is_some_and(|snapshot| {
                    snapshot.preview.document.component_id.as_str() == "radio"
                }) {
                    PropertyId::new("selected").unwrap()
                } else if self.host.snapshot().is_some_and(|snapshot| {
                    snapshot.preview.document.component_id.as_str() == "alert-specimen"
                }) {
                    PropertyId::new("visible").unwrap()
                } else if self.host.snapshot().is_some_and(|snapshot| {
                    snapshot.preview.document.component_id.as_str() == "popover-specimen"
                }) {
                    PropertyId::new("open").unwrap()
                } else {
                    PropertyId::new("checked").unwrap()
                },
                stamp: self.stamp_lease.clone(),
            },
            Self::read_enabled,
            Self::write_enabled,
        );
        let slider_binding = Binding::new_with(
            self.amount as f32,
            PropertyBindingContext {
                property: PropertyId::new("value").unwrap(),
                stamp: self.stamp_lease.clone(),
            },
            CompiledPreviewGallery::read_slider_value,
            CompiledPreviewGallery::write_slider_value,
        );
        let stepper_binding = Binding::new_with(
            self.stepper_count,
            PropertyBindingContext {
                property: PropertyId::new("value").unwrap(),
                stamp: self.stamp_lease.clone(),
            },
            CompiledPreviewGallery::read_stepper_value,
            CompiledPreviewGallery::write_stepper_value,
        );
        let date_value_binding = Binding::new_with(
            self.host.snapshot().and_then(property_date),
            PropertyBindingContext {
                property: PropertyId::new("date").unwrap(),
                stamp: self.stamp_lease.clone(),
            },
            Self::read_date_value,
            Self::write_date_value,
        );
        let color_value_binding = Binding::new_with(
            self.host.snapshot().and_then(property_color).flatten(),
            PropertyBindingContext {
                property: PropertyId::new("hue").unwrap(),
                stamp: self.stamp_lease.clone(),
            },
            CompiledPreviewGallery::read_color_value,
            CompiledPreviewGallery::write_color_value,
        );
        let stamp_lease = self.stamp_lease.clone();
        let secure_preview = self.host.snapshot().is_some_and(|snapshot| {
            snapshot.preview.document.component_id.as_str() == "secure-field-specimen"
        });
        let owner_message = if secure_preview {
            "lab-compiled-preview-owner-secure"
        } else {
            "lab-compiled-preview-owner-text"
        };
        let owner_arguments = MessageArguments::new()
            .with("text", self.text.clone())
            .with(
                "secure_present",
                (!self.secure_value.is_empty()).to_string(),
            )
            .with("enabled", self.enabled.to_string())
            .with("amount", self.amount)
            .with(
                "stamp",
                format!("{:?}", self.host.snapshot().map(|snapshot| &snapshot.stamp)),
            );
        let presentation = self.current_status_presentation();
        let status_key =
            presentation.map_or_else(|| preview_status_summary_key(&self.status), |p| p.key);
        let status_arguments = MessageArguments::new()
            .with("status", self.status.clone())
            .with("subject", presentation.map_or("", |p| &p.subject))
            .with("generation", presentation.map_or("", |p| &p.generation))
            .with("activations", self.activations.to_string())
            .with("renders", self.renders.to_string());
        let rejection_announcement =
            preview_inspector_rejection_live_region_value(&self.status).unwrap_or_default();
        let mounted_preview =
            OwnerActionReader(move |owner: OwnerAction<CompiledPreviewGallery>| {
                render_preview(&FactoryContext {
                    owner,
                    stamp_lease: stamp_lease.clone(),
                    text: binding.clone(),
                    secure_value: secure_value_binding.clone(),
                    checked: checked_binding.clone(),
                    slider_value: slider_binding.clone(),
                    stepper_value: stepper_binding.clone(),
                    color_value: color_value_binding.clone(),
                    date_value: date_value_binding.clone(),
                    token_values: token_values_binding.clone(),
                })
                .expect("registered compiled factory")
            });
        #[cfg(target_family = "wasm")]
        if self.reference_bridge.is_some() {
            return AnyView::new(
                VStack::new()
                    .child(self.reference_owner_binding())
                    .child(mounted_preview),
            );
        }

        let view = VStack::new()
            .spacing(8.0)
            .child(self.reference_owner_binding())
            .child(LocalizedText("lab-compiled-preview-title"))
            .child(LocalizedText(owner_message).with_arguments(owner_arguments))
            .child(
                Semantics(
                    "compiled-preview-status",
                    LocalizedText(status_key).with_arguments(status_arguments),
                )
                .live_region_value(rejection_announcement, LivePoliteness::Polite),
            )
            .child(
                VStack::new()
                    .spacing(6.0)
                    .child(preview_action(
                        "Preview Text specimen",
                        rustui::runtime::Action::new(Self::mount_text_specimen),
                    ))
                    .child(preview_action(
                        "Preview LocalizedText specimen",
                        rustui::runtime::Action::new(Self::mount_localized_text_specimen),
                    ))
                    .child(preview_action(
                        "Preview Text",
                        rustui::runtime::Action::new(Self::mount_text),
                    ))
                    .child(preview_action(
                        "Preview Button",
                        rustui::runtime::Action::new(Self::mount_button),
                    ))
                    .child(preview_action(
                        "Preview TextField",
                        rustui::runtime::Action::new(Self::mount_field),
                    ))
                    .child(preview_action(
                        "Preview Toggle",
                        rustui::runtime::Action::new(Self::mount_toggle),
                    ))
                    .child(preview_action(
                        "Preview Checkbox",
                        rustui::runtime::Action::new(Self::mount_checkbox),
                    ))
                    .child(preview_action(
                        "Preview Radio",
                        rustui::runtime::Action::new(Self::mount_radio),
                    ))
                    .child(preview_action(
                        "Preview Picker",
                        rustui::runtime::Action::new(Self::mount_picker),
                    ))
                    .child(preview_action(
                        "Preview Slider",
                        rustui::runtime::Action::new(Self::mount_slider),
                    ))
                    .child(preview_action(
                        "Preview Progress",
                        rustui::runtime::Action::new(Self::mount_progress),
                    ))
                    .child(preview_action(
                        "Preview Stepper",
                        rustui::runtime::Action::new(Self::mount_stepper),
                    )),
            )
            .child(LocalizedText("lab-compiled-preview-inspector"))
            .child(
                PropertyInspector("compiled-preview-inspector", inspector_rows)
                    .expect("schema property IDs are unique and valid"),
            )
            .child(
                VStack::new()
                    .spacing(4.0)
                    .child(
                        Button(
                            "Rerender preview parent",
                            rustui::runtime::Action::new(Self::rerender),
                        )
                        .localized_label("lab-compiled-preview-rerender"),
                    )
                    .child(preview_action(
                        "Preview TextField specimen",
                        rustui::runtime::Action::new(Self::mount_text_field_specimen),
                    ))
                    .child(preview_action(
                        "Preview TokenField specimen",
                        rustui::runtime::Action::new(Self::mount_token_field_specimen),
                    ))
                    .child(preview_action(
                        "Preview SearchField specimen",
                        rustui::runtime::Action::new(Self::mount_search_field_specimen),
                    ))
                    .child(preview_action(
                        "Preview TextEditor specimen",
                        rustui::runtime::Action::new(Self::mount_text_editor_specimen),
                    ))
                    .child(preview_action(
                        "Preview CodeEditor specimen",
                        rustui::runtime::Action::new(Self::mount_code_editor_specimen),
                    ))
                    .child(preview_action(
                        "Preview SecureField specimen",
                        rustui::runtime::Action::new(Self::mount_secure_field_specimen),
                    ))
                    .child(preview_action(
                        "Preview ComboBox specimen",
                        rustui::runtime::Action::new(Self::mount_combo_box_specimen),
                    ))
                    .child(preview_action(
                        "Preview TabView specimen",
                        rustui::runtime::Action::new(Self::mount_tab_view_specimen),
                    ))
                    .child(preview_action(
                        "Preview Button specimen",
                        rustui::runtime::Action::new(Self::mount_button_specimen),
                    ))
                    .child(preview_action(
                        "Preview Toolbar specimen",
                        rustui::runtime::Action::new(Self::mount_toolbar_specimen),
                    ))
                    .child(preview_action(
                        "Preview Image specimen",
                        rustui::runtime::Action::new(Self::mount_image_specimen),
                    ))
                    .child(preview_action(
                        "Preview Breadcrumb specimen",
                        rustui::runtime::Action::new(Self::mount_breadcrumb_specimen),
                    ))
                    .child(preview_action(
                        "Preview ContextMenu specimen",
                        rustui::runtime::Action::new(Self::mount_context_menu_specimen),
                    ))
                    .child(preview_action(
                        "Preview Menu specimen",
                        rustui::runtime::Action::new(Self::mount_menu_specimen),
                    ))
                    .child(preview_action(
                        "Preview Link specimen",
                        rustui::runtime::Action::new(Self::mount_link_specimen),
                    ))
                    .child(preview_action(
                        "Preview Dialog specimen",
                        rustui::runtime::Action::new(Self::mount_dialog_specimen),
                    ))
                    .child(preview_action(
                        "Preview Popover specimen",
                        rustui::runtime::Action::new(Self::mount_popover_specimen),
                    ))
                    .child(preview_action(
                        "Preview EmptyState specimen",
                        rustui::runtime::Action::new(Self::mount_empty_state_specimen),
                    ))
                    .child(preview_action(
                        "Preview Alert specimen",
                        rustui::runtime::Action::new(Self::mount_alert_specimen),
                    ))
                    .child(preview_action(
                        "Preview Toast specimen",
                        rustui::runtime::Action::new(Self::mount_toast_specimen),
                    ))
                    .child(preview_action(
                        "Preview Sheet specimen",
                        rustui::runtime::Action::new(Self::mount_sheet_specimen),
                    ))
                    .child(preview_action(
                        "Preview Drawer specimen",
                        rustui::runtime::Action::new(Self::mount_drawer_specimen),
                    ))
                    .child(preview_action(
                        "Preview Sidebar specimen",
                        rustui::runtime::Action::new(Self::mount_sidebar_specimen),
                    ))
                    .child(preview_action(
                        "Preview KeyedList specimen",
                        rustui::runtime::Action::new(Self::mount_keyed_list_specimen),
                    ))
                    .child(preview_action(
                        "Preview DataGrid specimen",
                        rustui::runtime::Action::new(Self::mount_data_grid_specimen),
                    ))
                    .child(preview_action(
                        "Preview List specimen",
                        rustui::runtime::Action::new(Self::mount_list_specimen),
                    ))
                    .child(preview_action(
                        "Preview Table specimen",
                        rustui::runtime::Action::new(Self::mount_table_specimen),
                    ))
                    .child(preview_action(
                        "Preview Tree specimen",
                        rustui::runtime::Action::new(Self::mount_tree_specimen),
                    ))
                    .child(preview_action(
                        "Preview Outline specimen",
                        rustui::runtime::Action::new(Self::mount_outline_specimen),
                    ))
                    .child(preview_action(
                        "Preview NavigationSplitView specimen",
                        rustui::runtime::Action::new(Self::mount_navigation_split_specimen),
                    ))
                    .child(preview_action(
                        "Preview NavigationStack specimen",
                        rustui::runtime::Action::new(Self::mount_navigation_stack_specimen),
                    ))
                    .child(preview_action(
                        "Preview Collection specimen",
                        rustui::runtime::Action::new(Self::mount_collection_specimen),
                    ))
                    .child(preview_action(
                        "Preview Canvas specimen",
                        rustui::runtime::Action::new(Self::mount_canvas_specimen),
                    ))
                    .child(preview_action(
                        "Preview Group specimen",
                        rustui::runtime::Action::new(Self::mount_group_specimen),
                    ))
                    .child(preview_action(
                        "Preview AnimatedOpacity specimen",
                        rustui::runtime::Action::new(Self::mount_animated_opacity_specimen),
                    ))
                    .child(preview_action(
                        "Preview SpringOpacity specimen",
                        rustui::runtime::Action::new(Self::mount_spring_opacity_specimen),
                    ))
                    .child(preview_action(
                        "Preview Shadow specimen",
                        rustui::runtime::Action::new(Self::mount_shadow_specimen),
                    ))
                    .child(preview_action(
                        "Preview Background specimen",
                        rustui::runtime::Action::new(Self::mount_background_specimen),
                    ))
                    .child(preview_action(
                        "Preview Border specimen",
                        rustui::runtime::Action::new(Self::mount_border_specimen),
                    ))
                    .child(preview_action(
                        "Preview Semantics specimen",
                        rustui::runtime::Action::new(Self::mount_semantics_specimen),
                    ))
                    .child(preview_action(
                        "Preview Card specimen",
                        rustui::runtime::Action::new(Self::mount_card_specimen),
                    ))
                    .child(preview_action(
                        "Preview Container specimen",
                        rustui::runtime::Action::new(Self::mount_container_specimen),
                    ))
                    .child(preview_action(
                        "Preview SplitView specimen",
                        rustui::runtime::Action::new(Self::mount_split_view_specimen),
                    ))
                    .child(preview_action(
                        "Preview Frame specimen",
                        rustui::runtime::Action::new(Self::mount_frame_specimen),
                    ))
                    .child(preview_action(
                        "Preview Spacer specimen",
                        rustui::runtime::Action::new(Self::mount_spacer_specimen),
                    ))
                    .child(preview_action(
                        "Preview HStack specimen",
                        rustui::runtime::Action::new(Self::mount_hstack_specimen),
                    ))
                    .child(preview_action(
                        "Preview VStack specimen",
                        rustui::runtime::Action::new(Self::mount_vstack_specimen),
                    ))
                    .child(preview_action(
                        "Preview Grid specimen",
                        rustui::runtime::Action::new(Self::mount_grid_specimen),
                    ))
                    .child(preview_action(
                        "Preview LazyGrid specimen",
                        rustui::runtime::Action::new(Self::mount_lazy_grid_specimen),
                    ))
                    .child(preview_action(
                        "Preview GeometryReader specimen",
                        rustui::runtime::Action::new(Self::mount_geometry_reader_specimen),
                    ))
                    .child(preview_action(
                        "Preview PropertyInspector specimen",
                        rustui::runtime::Action::new(Self::mount_property_inspector_specimen),
                    ))
                    .child(preview_action(
                        "Preview Disclosure specimen",
                        rustui::runtime::Action::new(Self::mount_disclosure_specimen),
                    ))
                    .child(preview_action(
                        "Preview ErrorState specimen",
                        rustui::runtime::Action::new(Self::mount_error_state_specimen),
                    ))
                    .child(preview_action(
                        "Preview DatePicker specimen",
                        rustui::runtime::Action::new(Self::mount_date_picker_specimen),
                    ))
                    .child(preview_action(
                        "Preview Badge specimen",
                        rustui::runtime::Action::new(Self::mount_badge_specimen),
                    ))
                    .child(preview_action(
                        "Preview Label specimen",
                        rustui::runtime::Action::new(Self::mount_label_specimen),
                    ))
                    .child(preview_action(
                        "Preview Tooltip specimen",
                        rustui::runtime::Action::new(Self::mount_tooltip_specimen),
                    ))
                    .child(preview_action(
                        "Preview Spinner specimen",
                        rustui::runtime::Action::new(Self::mount_spinner_specimen),
                    ))
                    .child(preview_action(
                        "Preview Divider specimen",
                        rustui::runtime::Action::new(Self::mount_divider_specimen),
                    ))
                    .child(preview_action(
                        "Preview Skeleton specimen",
                        rustui::runtime::Action::new(Self::mount_skeleton_specimen),
                    ))
                    .child(preview_action(
                        "Preview Icon specimen",
                        rustui::runtime::Action::new(Self::mount_icon_specimen),
                    ))
                    .child(preview_action(
                        "Preview Avatar specimen",
                        rustui::runtime::Action::new(Self::mount_avatar_specimen),
                    ))
                    .child(preview_action(
                        "Preview ColorPicker specimen",
                        rustui::runtime::Action::new(Self::mount_color_picker_specimen),
                    ))
                    .child(
                        Button(
                            "Teardown preview",
                            rustui::runtime::Action::new(Self::teardown),
                        )
                        .localized_label("lab-preview-teardown"),
                    ),
            )
            .child(LocalizedText("lab-preview-descriptor-heading"))
            .child(
                HStack::new()
                    .spacing(6.0)
                    .child(
                        Button(
                            "Import descriptor",
                            rustui::runtime::Action::new(Self::import_descriptor),
                        )
                        .localized_label("lab-preview-import"),
                    )
                    .child(
                        Button(
                            "Export normalized descriptor",
                            rustui::runtime::Action::new(Self::export_descriptor),
                        )
                        .localized_label("lab-preview-export"),
                    ),
            )
            .child(OwnerActionReader(
                move |owner: OwnerAction<CompiledPreviewGallery>| {
                    KeyboardShortcuts(
                        "compiled-preview-import-shortcut",
                        TextEditor(
                            "Import or export normalized preview JSON",
                            descriptor_binding.clone(),
                        )
                        .localized_placeholder("lab-preview-descriptor-editor")
                        .id("compiled-preview-descriptor-json")
                        .height(150.0),
                    )
                    .shortcut(
                        "secondary-shift-enter",
                        owner.action(Self::import_descriptor),
                    )
                },
            ))
            .child(
                VStack::new()
                    .spacing(6.0)
                    .child(
                        Button(
                            "Capture amount callback",
                            rustui::runtime::Action::new(Self::capture_stale_amount_callback),
                        )
                        .localized_label("lab-preview-capture-callback"),
                    )
                    .child(
                        Button(
                            "Replay captured amount callback",
                            rustui::runtime::Action::new(Self::replay_stale_amount_callback),
                        )
                        .localized_label("lab-preview-replay-callback"),
                    )
                    .child(
                        Button(
                            "Reject stale preview callbacks",
                            rustui::runtime::Action::new(Self::reject_stale),
                        )
                        .localized_label("lab-preview-reject-stale"),
                    )
                    .child(
                        Button(
                            "Reject wrong preview type",
                            rustui::runtime::Action::new(Self::reject_wrong_type),
                        )
                        .localized_label("lab-preview-reject-type"),
                    ),
            )
            .child(OwnerTaskReader(
                |tasks: OwnerTask<CompiledPreviewGallery>| {
                    Button(
                        "Schedule no-blur parent rerender (5 seconds)",
                        tasks.start_with_executor(
                            "preview-parent-rerender",
                            |executor| {
                                let timer = executor.timer(Duration::from_secs(5));
                                async move {
                                    timer.await;
                                    Ok::<(), String>(())
                                }
                            },
                            CompiledPreviewGallery::finish_delayed_rerender,
                        ),
                    )
                    .localized_label("lab-preview-delayed-rerender")
                    .id("compiled-preview-no-blur-rerender")
                },
            ))
            .child(mounted_preview);
        AnyView::new(view)
    }
}
