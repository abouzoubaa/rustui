//! Controlled single-selection picker backed by a retained GPUI SelectState.

use std::collections::HashSet;

use gpui::{
    AnyElement, AppContext as _, Entity, IntoElement, ParentElement as _, SharedString, Styled,
    div, px,
};
use gpui_component::{
    Icon, IconName, IndexPath, Sizable, Size,
    select::{SearchableVec, Select as GpuiSelect, SelectEvent, SelectState},
};
use rustui_core::{Binding, Identity};

use crate::{
    components::theme::{required_style, selector, styled_element},
    view::{EmptyView, RenderContext, View},
};

type PickerSelectState = SelectState<SearchableVec<String>>;

struct RetainedPicker {
    select: Entity<PickerSelectState>,
    options: Vec<String>,
}

/// A controlled picker whose selection and dropdown state survive body rebuilds.
pub struct PickerView {
    label: SharedString,
    selection: Binding<String>,
    options: Vec<String>,
    id: Option<Identity>,
    disabled: bool,
    searchable: bool,
    search_placeholder: Option<SharedString>,
    accessibility_label: Option<SharedString>,
    appearance: bool,
}

impl PickerView {
    pub fn new<I, S>(label: impl Into<SharedString>, selection: Binding<String>, options: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            label: label.into(),
            selection,
            options: collect_unique_options(options),
            id: None,
            disabled: false,
            searchable: false,
            search_placeholder: None,
            accessibility_label: None,
            appearance: true,
        }
    }

    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn searchable(mut self, searchable: bool) -> Self {
        self.searchable = searchable;
        self
    }

    pub fn search_placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.search_placeholder = Some(placeholder.into());
        self
    }

    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    pub fn appearance(mut self, appearance: bool) -> Self {
        self.appearance = appearance;
        self
    }

    fn identity(&self) -> Identity {
        self.id
            .clone()
            .unwrap_or_else(|| Identity::from(format!("picker:{}", self.label)))
    }
}

#[allow(non_snake_case)]
pub fn Picker<I, S>(
    label: impl Into<SharedString>,
    selection: Binding<String>,
    options: I,
) -> PickerView
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    PickerView::new(label, selection, options)
}

impl View for PickerView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let identity = self.identity();
        let options = self.options.clone();
        let initial_value = self.selection.value();
        let initial_index = option_index(&options, &initial_value);
        let searchable = self.searchable;
        let (retained, created) =
            context.retained_entity(identity.clone(), move |window, cx| RetainedPicker {
                select: cx.new(|cx| {
                    SelectState::new(
                        SearchableVec::new(options.clone()),
                        initial_index,
                        window,
                        cx,
                    )
                    .searchable(searchable)
                }),
                options,
            });

        let (select, retained_options) = {
            let retained = retained.read(context.app());
            (retained.select.clone(), retained.options.clone())
        };

        let options_changed = retained_options != self.options;
        if options_changed {
            let options = self.options.clone();
            context.update_entity(&select, |state, window, cx| {
                state.set_items(SearchableVec::new(options.clone()), window, cx);
            });
            context.update_entity(&retained, move |retained, _window, _cx| {
                retained.options = options;
            });
        }

        let current = self.selection.value();
        let retained_value = select.read(context.app()).selected_value().cloned();
        if super::select_needs_reconciliation(options_changed, retained_value.as_deref(), &current)
        {
            context.update_entity(&select, move |state, window, cx| {
                state.set_selected_value(&current, window, cx);
            });
        }

        if created {
            let write = context.binding_writer(self.selection.clone());
            context.subscribe_retained(
                &identity,
                &select,
                move |_state, event: &SelectEvent<SearchableVec<String>>, cx| {
                    let SelectEvent::Confirm(value) = event;
                    if let Some(value) = value {
                        write(value.clone(), cx);
                    }
                },
            );
        }

        let mut picker = GpuiSelect::new(&select)
            .id(identity.to_string())
            .placeholder(self.label.clone())
            .accessibility_label(
                self.accessibility_label
                    .clone()
                    .unwrap_or_else(|| self.label.clone()),
            )
            .disabled(self.disabled)
            .appearance(self.appearance);
        if let Some(placeholder) = &self.search_placeholder {
            picker = picker.search_placeholder(placeholder.clone());
        }
        if !self.appearance {
            return picker.into_any_element();
        }

        // GPUI Select owns focus, keyboard handling, and the menu. Its icon slot
        // has no background styling hook, so draw the themed affordance over an
        // empty icon while leaving the native trigger fully interactive.
        let style = required_style(context, "picker", "body", &selector(context));
        let mut affordance = styled_element(div().flex().items_center().justify_center(), &style)
            .absolute()
            .right(px(9.))
            .top_0()
            .bottom_0()
            .my_auto();
        if self.disabled {
            affordance = affordance.opacity(0.5);
        }
        let affordance = affordance.child(Icon::new(IconName::ChevronDown).with_size(Size::XSmall));
        div()
            .relative()
            .child(picker.icon(Icon::empty()))
            .child(affordance)
            .into_any_element()
    }
}

fn collect_unique_options<I, S>(options: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let options: Vec<String> = options.into_iter().map(Into::into).collect();
    let mut seen = HashSet::new();
    assert!(
        options.iter().all(|option| seen.insert(option.as_str())),
        "Picker options must be unique"
    );
    options
}

fn option_index(options: &[String], selection: &str) -> Option<IndexPath> {
    options
        .iter()
        .position(|option| option == selection)
        .map(IndexPath::new)
}
