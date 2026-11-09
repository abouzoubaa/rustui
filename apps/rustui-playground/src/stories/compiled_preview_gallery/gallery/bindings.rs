use super::*;

impl CompiledPreviewGallery {
    pub(super) fn read_descriptor(owner: &Self) -> String {
        owner.descriptor_json.clone()
    }
    pub(super) fn write_descriptor(owner: &mut Self, value: String) {
        owner.descriptor_json = value;
    }
    pub(super) fn teardown(&mut self) {
        self.secure_value.clear();
        if let Some(stamp) = self.host.snapshot().map(|s| s.stamp.clone()) {
            self.status = format!("Teardown: {:?}", self.host.teardown(&stamp));
        }
    }
    pub(super) fn string_selection_binding(
        selected_id: Option<String>,
        context: PropertyBindingContext,
    ) -> Binding<Option<String>> {
        Binding::new_with(
            selected_id,
            context,
            Self::read_string_selection,
            Self::write_string_selection,
        )
    }

    pub(super) fn read_string_selection(&self, context: &PropertyBindingContext) -> Option<String> {
        let snapshot = self.host.snapshot()?;
        if context.stamp.borrow().as_ref() != Some(&snapshot.stamp) {
            return None;
        }
        property_string(&snapshot, context.property.as_str())
    }

    pub(super) fn write_string_selection(
        &mut self,
        context: &PropertyBindingContext,
        selected_id: Option<String>,
    ) -> bool {
        selected_id.is_some_and(|selected_id| self.write_text(context, selected_id))
    }

    pub(super) fn navigation_stack_path_binding(
        path: Vec<String>,
        context: PropertyBindingContext,
    ) -> Binding<Vec<String>> {
        Binding::new_with(
            path,
            context,
            Self::read_navigation_stack_path,
            Self::write_navigation_stack_path,
        )
    }

    pub(super) fn read_navigation_stack_path(
        &self,
        context: &PropertyBindingContext,
    ) -> Vec<String> {
        match self.read_string_selection(context).as_deref() {
            Some("details") => vec!["details".into()],
            _ => Vec::new(),
        }
    }

    pub(super) fn write_navigation_stack_path(
        &mut self,
        context: &PropertyBindingContext,
        path: Vec<String>,
    ) -> bool {
        let selected_id = match path.as_slice() {
            [] => "none",
            [route] if route == "details" => "details",
            _ => return false,
        };
        self.write_text(context, selected_id.into())
    }

    pub(super) fn collection_selection_binding(
        selected_id: Option<Identity>,
        context: PropertyBindingContext,
    ) -> Binding<Option<Identity>> {
        Binding::new_with(
            selected_id,
            context,
            Self::read_collection_selection,
            Self::write_collection_selection,
        )
    }

    pub(super) fn read_collection_selection(
        &self,
        context: &PropertyBindingContext,
    ) -> Option<Identity> {
        self.read_string_selection(context)
            .filter(|selected_id| selected_id != "none")
            .map(Identity::from)
    }

    pub(super) fn write_collection_selection(
        &mut self,
        context: &PropertyBindingContext,
        selected_id: Option<Identity>,
    ) -> bool {
        let selected_id = selected_id
            .map(|id| id.as_str().to_owned())
            .unwrap_or_else(|| "none".into());
        if !PREVIEW_COLLECTION_SELECTIONS.contains(&selected_id.as_str()) {
            return false;
        }
        self.write_text(context, selected_id)
    }

    pub(super) fn data_grid_selection_binding(
        selected_row: Option<usize>,
        context: PropertyBindingContext,
    ) -> Binding<Option<usize>> {
        Binding::new_with(
            selected_row,
            context,
            Self::read_data_grid_selection,
            Self::write_data_grid_selection,
        )
    }

    pub(super) fn list_selection_binding(
        selected_index: Option<usize>,
        context: PropertyBindingContext,
    ) -> Binding<Option<usize>> {
        Binding::new_with(
            selected_index,
            context,
            Self::read_list_selection,
            Self::write_list_selection,
        )
    }

    pub(super) fn read_list_selection(&self, context: &PropertyBindingContext) -> Option<usize> {
        self.read_string_selection(context).and_then(|selection| {
            selection
                .strip_prefix("row-")
                .and_then(|row| row.parse::<usize>().ok())
        })
    }

    pub(super) fn write_list_selection(
        &mut self,
        context: &PropertyBindingContext,
        selected_index: Option<usize>,
    ) -> bool {
        if selected_index.is_some_and(|index| index >= PREVIEW_LIST_ROWS.len()) {
            return false;
        }
        let selected_id =
            selected_index.map_or_else(|| "none".into(), |index| format!("row-{index}"));
        self.write_text(context, selected_id)
    }

    pub(super) fn read_data_grid_selection(
        &self,
        context: &PropertyBindingContext,
    ) -> Option<usize> {
        self.read_string_selection(context).and_then(|selection| {
            selection
                .strip_prefix("row-")
                .and_then(|row| row.parse::<usize>().ok())
        })
    }

    pub(super) fn write_data_grid_selection(
        &mut self,
        context: &PropertyBindingContext,
        selected_row: Option<usize>,
    ) -> bool {
        if selected_row.is_some_and(|row| row >= 3) {
            return false;
        }
        let selected_id = selected_row.map_or_else(|| "none".into(), |row| format!("row-{row}"));
        self.write_text(context, selected_id)
    }

    pub(super) fn tree_selection_binding(
        selected_id: Option<String>,
        context: PropertyBindingContext,
    ) -> Binding<Option<String>> {
        Binding::new_with(
            selected_id,
            context,
            Self::read_tree_selection,
            Self::write_tree_selection,
        )
    }

    pub(super) fn outline_expansion_binding(
        value: String,
        context: PropertyBindingContext,
    ) -> Binding<std::collections::HashSet<String>> {
        fn expanded(value: &str) -> std::collections::HashSet<String> {
            let ids: &[&str] = match value {
                "all" => &["root", "src"],
                "root" => &["root"],
                "nested" => &["src"],
                _ => &[],
            };
            ids.iter().map(|id| (*id).to_owned()).collect()
        }
        Binding::new_with(
            expanded(&value),
            context,
            |owner: &Self, context| expanded(&owner.read_text(context)),
            |owner: &mut Self, context, ids| {
                if ids.iter().any(|id| !["root", "src"].contains(&id.as_str()))
                    || owner
                        .preview_snapshot_for_component(context, "outline-specimen")
                        .is_none()
                {
                    return false;
                }
                let value = match (ids.contains("root"), ids.contains("src")) {
                    (false, false) => "collapsed",
                    (true, false) => "root",
                    (false, true) => "nested",
                    (true, true) => "all",
                };
                owner.write_text(context, value.into())
            },
        )
    }

    pub(super) fn read_tree_selection(&self, context: &PropertyBindingContext) -> Option<String> {
        self.read_string_selection(context)
            .filter(|selection| selection != "none")
    }

    pub(super) fn write_tree_selection(
        &mut self,
        context: &PropertyBindingContext,
        selected_id: Option<String>,
    ) -> bool {
        if selected_id
            .as_deref()
            .is_some_and(|id| !PREVIEW_TREE_NODE_IDS.contains(&id))
        {
            return false;
        }
        self.write_text(context, selected_id.unwrap_or_else(|| "none".into()))
    }

    pub(super) fn read_disclosure_open(&self, context: &PropertyBindingContext) -> bool {
        self.preview_snapshot_for_component(context, "disclosure-specimen")
            .and_then(|snapshot| property_boolean(&snapshot, "open"))
            .unwrap_or(false)
    }

    pub(super) fn write_disclosure_open(
        &mut self,
        context: &PropertyBindingContext,
        open: bool,
    ) -> bool {
        let Some(stamp) = context.stamp.borrow().clone() else {
            return false;
        };
        let Some(snapshot) = self.host.snapshot() else {
            return false;
        };
        if snapshot.stamp != stamp
            || snapshot.preview.document.component_id.as_str() != "disclosure-specimen"
        {
            return false;
        }
        let signal = snapshot.signal(EventId::new("toggle").unwrap());
        if !matches!(self.host.dispatch(&signal), Ok(target) if target.as_str() == "toggle") {
            return false;
        }
        match self.host.binding_feedback(
            &stamp,
            context.property.clone(),
            PropertyValue::Boolean(open),
        ) {
            Ok(next) => {
                self.enabled = open;
                *context.stamp.borrow_mut() = Some(next);
                self.status = "Signal-only disclosure toggle accepted".into();
                true
            }
            Err(error) => {
                self.status = error.to_string();
                false
            }
        }
    }

    pub(super) fn dismiss_alert(&mut self, signal: &PreviewSignal) {
        match self.host.dispatch(signal) {
            Ok(target) if target.as_str() == "dismiss" => {
                let context = PropertyBindingContext {
                    property: PropertyId::new("visible").unwrap(),
                    stamp: self.stamp_lease.clone(),
                };
                if self.write_enabled(&context, false) {
                    self.status = "Signal-only alert dismiss action".into();
                }
            }
            result => self.status = format!("Rejected alert signal: {result:?}"),
        }
    }

    pub(super) fn show_toast(&mut self, signal: &PreviewSignal) {
        match self.host.dispatch(signal) {
            Ok(target) if target.as_str() == "toast_show" => {
                self.activations += 1;
                self.status = "Allowlisted Toast show action".into();
            }
            result => self.status = format!("Rejected Toast signal: {result:?}"),
        }
    }

    pub(super) fn toolbar_primary(&mut self, signal: &PreviewSignal) {
        match self.host.dispatch(signal) {
            Ok(target) if target.as_str() == "toolbar_primary" => {
                self.activations += 1;
                self.status = "Allowlisted Toolbar primary action".into();
            }
            result => self.status = format!("Rejected Toolbar primary signal: {result:?}"),
        }
    }

    pub(super) fn toolbar_secondary(&mut self, signal: &PreviewSignal) {
        match self.host.dispatch(signal) {
            Ok(target) if target.as_str() == "toolbar_secondary" => {
                self.activations += 1;
                self.status = "Allowlisted Toolbar secondary action".into();
            }
            result => self.status = format!("Rejected Toolbar secondary signal: {result:?}"),
        }
    }

    pub(super) fn navigate_breadcrumb_parent(&mut self, signal: &PreviewSignal) {
        match self.host.dispatch(signal) {
            Ok(target) if target.as_str() == "breadcrumb_parent" => {
                self.activations += 1;
                self.status = "Allowlisted Breadcrumb parent action".into();
            }
            result => self.status = format!("Rejected Breadcrumb parent signal: {result:?}"),
        }
    }

    pub(super) fn activate_context_menu_item(&mut self, signal: &PreviewSignal) {
        match self.host.dispatch(signal) {
            Ok(target) if target.as_str() == "context_menu_open_preview" => {
                self.activations += 1;
                self.status = "Allowlisted ContextMenu action".into();
            }
            result => self.status = format!("Rejected ContextMenu signal: {result:?}"),
        }
    }

    pub(super) fn activate(&mut self, signal: &PreviewSignal) {
        match self.host.dispatch(signal) {
            Ok(target) if target.as_str() == "increment" => {
                self.activations += 1;
                self.status = "Allowlisted increment action".into();
            }
            result => self.status = format!("Rejected signal: {result:?}"),
        }
    }

    pub(super) fn record_hover_enter(&mut self, signal: &PreviewSignal) {
        match self.host.dispatch(signal) {
            Ok(target) if target.as_str() == "increment" => {
                self.activations += 1;
                self.status = "Allowlisted HoverRegion enter action".into();
            }
            result => self.status = format!("Rejected HoverRegion enter signal: {result:?}"),
        }
    }

    pub(super) fn record_hover_exit(&mut self, signal: &PreviewSignal) {
        match self.host.dispatch(signal) {
            Ok(target) if target.as_str() == "increment" => {
                self.activations += 1;
                self.status = "Allowlisted HoverRegion exit action".into();
            }
            result => self.status = format!("Rejected HoverRegion exit signal: {result:?}"),
        }
    }
    pub(super) fn reject_stale(&mut self) {
        let Some(current) = self.host.snapshot().map(|snapshot| snapshot.stamp.clone()) else {
            return;
        };
        let stale = PreviewStamp {
            generation: 0,
            ..current
        };
        let signal = PreviewSignal {
            stamp: stale.clone(),
            event: EventId::new("activate").unwrap(),
        };
        let action_rejected = self.host.dispatch(&signal).is_err();
        let feedback_rejected = self
            .host
            .binding_feedback(
                &stale,
                PropertyId::new("text").unwrap(),
                PropertyValue::String("stale write".into()),
            )
            .is_err();
        self.status = format!(
            "Stale action rejected: {action_rejected}; stale binding rejected: {feedback_rejected}"
        );
    }
    pub(super) fn capture_stale_amount_callback(&mut self) {
        let Some(stamp) = self.host.snapshot().map(|snapshot| snapshot.stamp.clone()) else {
            self.status = "No mounted preview to capture".into();
            return;
        };
        self.stale_amount_callback = Some(Binding::new_with(
            self.amount_draft.clone(),
            PropertyBindingContext {
                property: PropertyId::new("amount").unwrap(),
                stamp: Rc::new(RefCell::new(Some(stamp.clone()))),
            },
            Self::read_amount_draft,
            Self::write_amount_draft,
        ));
        self.status = format!(
            "Captured amount callback at generation {} revision {}",
            stamp.generation, stamp.revision
        );
    }
    pub(super) fn replay_stale_amount_callback(&mut self) {
        let Some(callback) = self.stale_amount_callback.clone() else {
            self.status = "Capture an amount callback before replay".into();
            return;
        };
        let draft_before = self.amount_draft.clone();
        let amount_before = self
            .host
            .snapshot()
            .and_then(|snapshot| property_number(snapshot, "amount"));
        let accepted = callback.write_to(self, "0.987".into());
        let amount_after = self
            .host
            .snapshot()
            .and_then(|snapshot| property_number(snapshot, "amount"));
        let unchanged = self.amount_draft == draft_before && amount_after == amount_before;
        self.status =
            format!("Stale amount callback accepted={accepted}; state unchanged={unchanged}");
    }
    pub(super) fn reject_wrong_type(&mut self) {
        let Some(stamp) = self.host.snapshot().map(|snapshot| snapshot.stamp.clone()) else {
            return;
        };
        let rejected = self
            .host
            .edit(
                &stamp,
                [(
                    PropertyId::new("text").unwrap(),
                    PropertyValue::Boolean(true),
                )],
            )
            .is_err();
        self.status = format!("Wrong property type rejected: {rejected}");
    }
    pub(super) fn read_text(&self, context: &PropertyBindingContext) -> String {
        let snapshot = self.host.snapshot();
        if snapshot.as_ref().map(|snapshot| &snapshot.stamp) != context.stamp.borrow().as_ref() {
            return String::new();
        }
        let Some(snapshot) = snapshot else {
            return self.text.clone();
        };
        if context.property.as_str() == "title"
            && snapshot.preview.document.component_id.as_str() == "toast-specimen"
            && property_string(&snapshot, "title").is_none()
        {
            return String::new();
        }
        if context.property.as_str() == "placeholder" {
            match snapshot.preview.document.component_id.as_str() {
                "text-field-specimen" => self.text_field_placeholder.clone(),
                "search-field-specimen" => self.search_field_placeholder.clone(),
                "text-editor-specimen" => self.text_editor_placeholder.clone(),
                "secure-field-specimen" => self.secure_field_placeholder.clone(),
                "token-field-specimen" => self.token_field_placeholder.clone(),
                _ => String::new(),
            }
        } else {
            property_string(&snapshot, context.property.as_str()).unwrap_or_default()
        }
    }
    pub(super) fn read_token_values(&self, context: &PropertyBindingContext) -> Vec<String> {
        let Some(snapshot) = self.host.snapshot() else {
            return Vec::new();
        };
        if context.stamp.borrow().as_ref() != Some(&snapshot.stamp)
            || snapshot.preview.document.component_id.as_str() != "token-field-specimen"
        {
            return Vec::new();
        }
        property_string(&snapshot, context.property.as_str())
            .and_then(|value| parse_preview_token_values(&value).ok())
            .unwrap_or_default()
    }
    pub(super) fn write_token_values(
        &mut self,
        context: &PropertyBindingContext,
        values: Vec<String>,
    ) -> bool {
        let Some(stamp) = context.stamp.borrow().clone() else {
            return false;
        };
        let Some(snapshot) = self.host.snapshot() else {
            return false;
        };
        if snapshot.stamp != stamp
            || snapshot.preview.document.component_id.as_str() != "token-field-specimen"
        {
            return false;
        }
        let Ok(tokens_json) = serde_json::to_string(&values) else {
            self.status = "Rejected: TokenField token list could not be serialized".into();
            return false;
        };
        if tokens_json.len() > PREVIEW_TOKEN_VALUES_MAX_LENGTH {
            self.status = "Rejected: TokenField token list exceeds its preview limit".into();
            return false;
        }
        match self.host.binding_feedback(
            &stamp,
            context.property.clone(),
            PropertyValue::String(tokens_json),
        ) {
            Ok(next) => {
                self.token_values = values;
                *context.stamp.borrow_mut() = Some(next);
                self.status = "TokenField token list accepted".into();
                true
            }
            Err(error) => {
                self.status = format!("Rejected: {error}");
                false
            }
        }
    }
    pub(super) fn write_text(&mut self, context: &PropertyBindingContext, text: String) -> bool {
        let Some(stamp) = context.stamp.borrow().clone() else {
            return false;
        };
        let Some(snapshot) = self.host.snapshot() else {
            return false;
        };
        if snapshot.stamp != stamp {
            return false;
        }

        let is_date_picker_date = snapshot.preview.document.component_id.as_str()
            == "date-picker-specimen"
            && context.property.as_str() == "date";
        if is_date_picker_date && parse_preview_date(&text).is_err() {
            self.status = DATE_PICKER_DATE_REJECTION.into();
            return false;
        }
        let is_token_values = snapshot.preview.document.component_id.as_str()
            == "token-field-specimen"
            && context.property.as_str() == "tokens_json";
        if is_token_values && parse_preview_token_values(&text).is_err() {
            self.status = "Rejected: TokenField tokens_json must be a JSON string array".into();
            return false;
        }
        let requires_nonblank = self
            .host
            .catalog()
            .components
            .get(&snapshot.preview.document.component_id)
            .and_then(|component| component.properties.get(&context.property))
            .is_some_and(|property| property.min_length.is_some_and(|minimum| minimum > 0));
        if requires_nonblank && text.trim().is_empty() {
            self.status = "Rejected blank value for a nonempty preview property".into();
            return false;
        }
        match self.host.binding_feedback(
            &stamp,
            context.property.clone(),
            PropertyValue::String(text.clone()),
        ) {
            Ok(next) => {
                if context.property.as_str() == "placeholder" {
                    match self
                        .host
                        .snapshot()
                        .map(|snapshot| snapshot.preview.document.component_id.as_str().to_owned())
                    {
                        Some(component_id) if component_id == "text-field-specimen" => {
                            self.text_field_placeholder = text;
                        }
                        Some(component_id) if component_id == "search-field-specimen" => {
                            self.search_field_placeholder = text;
                        }
                        Some(component_id) if component_id == "text-editor-specimen" => {
                            self.text_editor_placeholder = text;
                        }
                        Some(component_id) if component_id == "secure-field-specimen" => {
                            self.secure_field_placeholder = text;
                        }
                        Some(component_id) if component_id == "token-field-specimen" => {
                            self.token_field_placeholder = text;
                        }
                        _ => self.text = text,
                    }
                } else if is_token_values {
                    self.token_values = parse_preview_token_values(&text).unwrap_or_default();
                } else {
                    self.text = text;
                }
                self.status = "Binding feedback accepted".into();
                // The same retained InputState subscription will serve the next
                // edit, so advance its lease only after this exact revision accepts.
                *context.stamp.borrow_mut() = Some(next);
                true
            }
            Err(error) => {
                self.status = format!("Rejected: {error}");
                false
            }
        }
    }
    pub(super) fn read_secure_value(&self, context: &PropertyBindingContext) -> String {
        if self
            .preview_snapshot_for_component(context, "secure-field-specimen")
            .is_none()
        {
            return String::new();
        }
        self.secure_value.clone()
    }
    pub(super) fn write_secure_value(
        &mut self,
        context: &PropertyBindingContext,
        secret: String,
    ) -> bool {
        let Some(stamp) = context.stamp.borrow().clone() else {
            return false;
        };
        let Some(snapshot) = self.host.snapshot() else {
            return false;
        };
        if snapshot.stamp != stamp
            || snapshot.preview.document.component_id.as_str() != "secure-field-specimen"
        {
            return false;
        }

        self.secure_value = secret;
        true
    }
    pub(super) fn preview_snapshot_for_binding(
        &self,
        context: &PropertyBindingContext,
    ) -> Option<PreviewSnapshot> {
        let stamp = context.stamp.borrow().clone()?;
        self.host
            .snapshot()
            .filter(|snapshot| snapshot.stamp == stamp)
            .cloned()
    }

    pub(super) fn preview_snapshot_for_component(
        &self,
        context: &PropertyBindingContext,
        component_id: &str,
    ) -> Option<PreviewSnapshot> {
        self.preview_snapshot_for_binding(context)
            .filter(|snapshot| snapshot.preview.document.component_id.as_str() == component_id)
    }

    pub(super) fn read_enabled(&self, context: &PropertyBindingContext) -> bool {
        let Some(snapshot) = self.preview_snapshot_for_binding(context) else {
            return false;
        };
        property_boolean(&snapshot, context.property.as_str()).unwrap_or(self.enabled)
    }

    pub(super) fn preview_inspector_boolean_binding(
        &self,
        context: PropertyBindingContext,
    ) -> Binding<bool> {
        let initial = self.read_enabled(&context);
        Binding::new_with(initial, context, Self::read_enabled, Self::write_enabled)
    }

    pub(super) fn read_date_value(&self, context: &PropertyBindingContext) -> Option<NaiveDate> {
        let snapshot = self.preview_snapshot_for_component(context, "date-picker-specimen")?;
        property_date(&snapshot)
    }

    pub(super) fn write_date_value(
        &mut self,
        context: &PropertyBindingContext,
        value: Option<NaiveDate>,
    ) -> bool {
        let Some(stamp) = context.stamp.borrow().clone() else {
            return false;
        };
        if !self.host.snapshot().is_some_and(|snapshot| {
            snapshot.stamp == stamp
                && snapshot.preview.document.component_id.as_str() == "date-picker-specimen"
        }) {
            return false;
        }
        let encoded = value.map(|date| date.to_string()).unwrap_or_default();
        if parse_preview_date(&encoded).is_err() {
            self.status = DATE_PICKER_DATE_REJECTION.into();
            return false;
        }
        match self.host.binding_feedback(
            &stamp,
            context.property.clone(),
            PropertyValue::String(encoded),
        ) {
            Ok(next) => {
                *context.stamp.borrow_mut() = Some(next);
                self.status = "DatePicker feedback accepted".into();
                true
            }
            Err(error) => {
                self.status = error.to_string();
                false
            }
        }
    }

    pub(super) fn read_color_value(&self, context: &PropertyBindingContext) -> Option<Hsla> {
        let snapshot = self.preview_snapshot_for_component(context, "color-picker-specimen")?;
        property_color(&snapshot).flatten()
    }

    pub(super) fn read_color_presence(&self, context: &PropertyBindingContext) -> bool {
        self.preview_snapshot_for_component(context, "color-picker-specimen")
            .and_then(|snapshot| property_boolean(&snapshot, "has_color"))
            .unwrap_or(false)
    }

    pub(super) fn write_color_presence(
        &mut self,
        context: &PropertyBindingContext,
        has_color: bool,
    ) -> bool {
        let color = self
            .preview_snapshot_for_component(context, "color-picker-specimen")
            .and_then(|snapshot| {
                Some(Hsla {
                    h: property_number(&snapshot, "hue")? as f32,
                    s: property_number(&snapshot, "saturation")? as f32,
                    l: property_number(&snapshot, "lightness")? as f32,
                    a: property_number(&snapshot, "alpha")? as f32,
                })
            })
            .unwrap_or_else(|| Hsla {
                h: PREVIEW_COLOR_HUE as f32,
                s: PREVIEW_COLOR_SATURATION as f32,
                l: PREVIEW_COLOR_LIGHTNESS as f32,
                a: PREVIEW_COLOR_ALPHA as f32,
            });
        self.write_color_value(context, has_color.then_some(color))
    }

    pub(super) fn write_color_value(
        &mut self,
        context: &PropertyBindingContext,
        value: Option<Hsla>,
    ) -> bool {
        let Some(stamp) = context.stamp.borrow().clone() else {
            return false;
        };
        let Some((current, color)) = self.host.snapshot().and_then(|snapshot| {
            (snapshot.stamp == stamp
                && snapshot.preview.document.component_id.as_str() == "color-picker-specimen")
                .then(|| (snapshot.stamp.clone(), property_color(snapshot).flatten()))
        }) else {
            return false;
        };
        if current != stamp {
            return false;
        }

        let has_color = value.is_some();
        let color = value.or(color).unwrap_or_else(|| Hsla {
            h: PREVIEW_COLOR_HUE as f32,
            s: PREVIEW_COLOR_SATURATION as f32,
            l: PREVIEW_COLOR_LIGHTNESS as f32,
            a: PREVIEW_COLOR_ALPHA as f32,
        });
        let channels = [color.h, color.s, color.l, color.a];
        if channels
            .iter()
            .any(|channel| !channel.is_finite() || !(0.0..=1.0).contains(channel))
        {
            self.status = "Color channels must be finite values from 0 to 1".into();
            return false;
        }

        match self.host.edit(
            &stamp,
            [
                (
                    PropertyId::new("has_color").unwrap(),
                    PropertyValue::Boolean(has_color),
                ),
                (
                    PropertyId::new("hue").unwrap(),
                    PropertyValue::Number(f64::from(color.h)),
                ),
                (
                    PropertyId::new("saturation").unwrap(),
                    PropertyValue::Number(f64::from(color.s)),
                ),
                (
                    PropertyId::new("lightness").unwrap(),
                    PropertyValue::Number(f64::from(color.l)),
                ),
                (
                    PropertyId::new("alpha").unwrap(),
                    PropertyValue::Number(f64::from(color.a)),
                ),
            ],
        ) {
            Ok(next) => {
                *context.stamp.borrow_mut() = Some(next);
                self.status = "ColorPicker feedback accepted".into();
                true
            }
            Err(error) => {
                self.status = error.to_string();
                false
            }
        }
    }

    pub(super) fn write_enabled(
        &mut self,
        context: &PropertyBindingContext,
        enabled: bool,
    ) -> bool {
        let Some(stamp) = context.stamp.borrow().clone() else {
            return false;
        };
        if self.host.snapshot().map(|snapshot| &snapshot.stamp) != Some(&stamp) {
            return false;
        }
        match self.host.binding_feedback(
            &stamp,
            context.property.clone(),
            PropertyValue::Boolean(enabled),
        ) {
            Ok(next) => {
                self.enabled = enabled;
                *context.stamp.borrow_mut() = Some(next);
                self.status = "Boolean property accepted".into();
                true
            }
            Err(error) => {
                self.status = error.to_string();
                false
            }
        }
    }
    pub(super) fn refresh_number_drafts(&mut self) {
        let drafts = self
            .host
            .snapshot()
            .map(|snapshot| {
                snapshot
                    .preview
                    .properties
                    .iter()
                    .filter_map(|(property, value)| match value {
                        PropertyValue::Number(value) => Some((property.clone(), value.to_string())),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default();
        self.number_drafts = drafts;
    }

    pub(super) fn read_number_draft(&self, context: &PropertyBindingContext) -> String {
        let Some(stamp) = context.stamp.borrow().clone() else {
            return String::new();
        };
        let Some(snapshot) = self.host.snapshot() else {
            return String::new();
        };
        if snapshot.stamp != stamp {
            return String::new();
        }
        self.number_drafts
            .get(&context.property)
            .cloned()
            .or_else(|| {
                property_number(&snapshot, context.property.as_str()).map(|value| value.to_string())
            })
            .unwrap_or_default()
    }

    pub(super) fn write_number_draft(
        &mut self,
        context: &PropertyBindingContext,
        draft: String,
    ) -> bool {
        let Some(stamp) = context.stamp.borrow().clone() else {
            return false;
        };
        let Some(snapshot) = self.host.snapshot() else {
            return false;
        };
        if snapshot.stamp != stamp {
            return false;
        }

        self.number_drafts
            .insert(context.property.clone(), draft.clone());
        let Ok(value) = draft.trim().parse::<f64>() else {
            self.status = "Enter a finite number".into();
            return true;
        };
        if !value.is_finite() {
            self.status = "Enter a finite number".into();
            return true;
        }

        let component_id = snapshot.preview.document.component_id.as_str().to_owned();
        match self.host.binding_feedback(
            &stamp,
            context.property.clone(),
            PropertyValue::Number(value),
        ) {
            Ok(next) => {
                if context.property.as_str() == "amount"
                    || (matches!(component_id.as_str(), "slider" | "progress")
                        && context.property.as_str() == "value")
                {
                    self.amount = value;
                    self.amount_draft = value.to_string();
                }
                *context.stamp.borrow_mut() = Some(next);
                self.status = format!("Accepted f64 value {value}");
                true
            }
            Err(error) => {
                self.status = format!("Rejected: {error}");
                true
            }
        }
    }

    pub(super) fn read_amount_draft(&self, context: &PropertyBindingContext) -> String {
        self.preview_snapshot_for_binding(context)
            .map(|_| self.amount_draft.clone())
            .unwrap_or_default()
    }
    pub(super) fn write_amount_draft(
        &mut self,
        context: &PropertyBindingContext,
        draft: String,
    ) -> bool {
        let Some(stamp) = context.stamp.borrow().clone() else {
            return false;
        };
        if self.host.snapshot().map(|snapshot| &snapshot.stamp) != Some(&stamp) {
            return false;
        }
        self.amount_draft = draft.clone();
        let Ok(amount) = draft.trim().parse::<f64>() else {
            self.status = "Enter a finite number from 0 to 1".into();
            return true;
        };
        if !amount.is_finite() {
            self.status = "Enter a finite number from 0 to 1".into();
            return true;
        }
        match self.host.binding_feedback(
            &stamp,
            context.property.clone(),
            PropertyValue::Number(amount),
        ) {
            Ok(next) => {
                self.amount = amount;
                *context.stamp.borrow_mut() = Some(next);
                self.number_drafts
                    .insert(context.property.clone(), amount.to_string());
                self.status = format!("Accepted f64 value {}", amount);
                true
            }
            Err(error) => {
                self.status = format!("Rejected: {error}");
                true
            }
        }
    }
    pub(super) fn read_stepper_value(&self, context: &PropertyBindingContext) -> i32 {
        self.preview_snapshot_for_component(context, "stepper")
            .map(|_| self.stepper_count)
            .unwrap_or(PREVIEW_STEPPER_MIN)
    }

    pub(super) fn write_stepper_value(
        &mut self,
        context: &PropertyBindingContext,
        value: i32,
    ) -> bool {
        let Some(stamp) = context.stamp.borrow().clone() else {
            return false;
        };
        if self.host.snapshot().map(|snapshot| &snapshot.stamp) != Some(&stamp)
            || !(PREVIEW_STEPPER_MIN..=PREVIEW_STEPPER_MAX).contains(&value)
        {
            return false;
        }

        match self.host.binding_feedback(
            &stamp,
            context.property.clone(),
            PropertyValue::Integer(i64::from(value)),
        ) {
            Ok(next) => {
                self.stepper_count = value;
                *context.stamp.borrow_mut() = Some(next);
                self.status = format!("Accepted integer value {value}");
                true
            }
            Err(error) => {
                self.status = error.to_string();
                false
            }
        }
    }

    pub(super) fn read_integer_property(&self, context: &PropertyBindingContext) -> i64 {
        let Some(stamp) = context.stamp.borrow().clone() else {
            return 0;
        };
        self.host
            .snapshot()
            .filter(|snapshot| snapshot.stamp == stamp)
            .and_then(|snapshot| property_integer(&snapshot, context.property.as_str()))
            .unwrap_or_default()
    }

    pub(super) fn write_integer_property(
        &mut self,
        context: &PropertyBindingContext,
        value: i64,
    ) -> bool {
        let Some(stamp) = context.stamp.borrow().clone() else {
            return false;
        };
        let Some(snapshot) = self.host.snapshot() else {
            return false;
        };
        if snapshot.stamp != stamp {
            return false;
        }
        let stepper_value = if snapshot.preview.document.component_id.as_str() == "stepper" {
            let Ok(stepper_value) = i32::try_from(value) else {
                self.status = "Stepper value must fit its control".into();
                return false;
            };
            Some(stepper_value)
        } else {
            None
        };

        match self.host.binding_feedback(
            &stamp,
            context.property.clone(),
            PropertyValue::Integer(value),
        ) {
            Ok(next) => {
                if let Some(stepper_value) = stepper_value {
                    self.stepper_count = stepper_value;
                }
                *context.stamp.borrow_mut() = Some(next);
                self.status = format!("Accepted integer property {value}");
                true
            }
            Err(error) => {
                self.status = format!("Rejected: {error}");
                false
            }
        }
    }

    pub(super) fn read_slider_value(&self, context: &PropertyBindingContext) -> f32 {
        self.preview_snapshot_for_component(context, "slider")
            .map(|_| self.amount as f32)
            .unwrap_or(PREVIEW_SLIDER_MIN)
    }
    pub(super) fn write_slider_value(
        &mut self,
        context: &PropertyBindingContext,
        value: f32,
    ) -> bool {
        let Some(stamp) = context.stamp.borrow().clone() else {
            return false;
        };
        if self.host.snapshot().map(|snapshot| &snapshot.stamp) != Some(&stamp)
            || !value.is_finite()
            || !(PREVIEW_SLIDER_MIN..=PREVIEW_SLIDER_MAX).contains(&value)
        {
            return false;
        }
        let amount = f64::from(value);
        match self.host.binding_feedback(
            &stamp,
            context.property.clone(),
            PropertyValue::Number(amount),
        ) {
            Ok(next) => {
                self.amount = amount;
                self.amount_draft = amount.to_string();
                self.number_drafts
                    .insert(context.property.clone(), amount.to_string());
                *context.stamp.borrow_mut() = Some(next);
                self.status = format!("Accepted slider value {amount}");
                true
            }
            Err(error) => {
                self.status = error.to_string();
                false
            }
        }
    }
    pub(super) fn text_binding(&self) -> Binding<String> {
        let context = PropertyBindingContext {
            property: self
                .host
                .snapshot()
                .map(
                    |snapshot| match snapshot.preview.document.component_id.as_str() {
                        "picker" | "combo-box-specimen" | "tab-view-specimen" => {
                            PropertyId::new("selection").unwrap()
                        }
                        "token-field-specimen" => PropertyId::new("draft").unwrap(),
                        "button-specimen" | "tap-region-specimen" | "hover-region-specimen" => {
                            PropertyId::new("label").unwrap()
                        }
                        "toast-specimen" => PropertyId::new("message").unwrap(),
                        "sheet-specimen" | "drawer-specimen" => {
                            PropertyId::new("trigger_label").unwrap()
                        }
                        "search-field-specimen" => PropertyId::new("query").unwrap(),
                        "text-field-specimen" => PropertyId::new("value").unwrap(),
                        "text-editor-specimen" | "text-specimen" | "code-editor-specimen" => {
                            PropertyId::new("value").unwrap()
                        }
                        _ => PropertyId::new("text").unwrap(),
                    },
                )
                .unwrap_or_else(|| PropertyId::new("text").unwrap()),
            stamp: self.stamp_lease.clone(),
        };
        Binding::new_with(
            self.text.clone(),
            context,
            Self::read_text,
            Self::write_text,
        )
    }
}
