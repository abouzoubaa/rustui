//! Controlled searchable ComboBox backed by a retained GPUI SelectState.

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

type RetainedComboBoxState = SelectState<SearchableVec<String>>;

struct RetainedComboBox {
    select: Entity<RetainedComboBoxState>,
    options: Vec<String>,
}

/// A controlled searchable single-selection ComboBox.
pub struct ComboBoxView {
    label: SharedString,
    selection: Binding<String>,
    options: Vec<String>,
    id: Option<Identity>,
    disabled: bool,
    search_placeholder: Option<SharedString>,
    appearance: bool,
}

impl ComboBoxView {
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
            search_placeholder: None,
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

    pub fn search_placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.search_placeholder = Some(placeholder.into());
        self
    }

    pub fn appearance(mut self, appearance: bool) -> Self {
        self.appearance = appearance;
        self
    }

    fn identity(&self) -> Identity {
        self.id
            .clone()
            .unwrap_or_else(|| Identity::from(format!("combo-box:{}", self.label)))
    }
}

#[allow(non_snake_case)]
pub fn ComboBox<I, S>(
    label: impl Into<SharedString>,
    selection: Binding<String>,
    options: I,
) -> ComboBoxView
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    ComboBoxView::new(label, selection, options)
}

impl View for ComboBoxView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let identity = self.identity();
        let options = self.options.clone();
        let initial_value = self.selection.value();
        let initial_index = option_index(&options, &initial_value);
        let (retained, created) =
            context.retained_entity(identity.clone(), move |window, cx| RetainedComboBox {
                select: cx.new(|cx| {
                    SelectState::new(
                        SearchableVec::new(options.clone()),
                        initial_index,
                        window,
                        cx,
                    )
                    .searchable(true)
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

        let mut combo_box = GpuiSelect::new(&select)
            .id(identity.to_string())
            .placeholder(self.label.clone())
            .accessibility_label(self.label.clone())
            .disabled(self.disabled)
            .appearance(self.appearance);
        if let Some(placeholder) = &self.search_placeholder {
            combo_box = combo_box.search_placeholder(placeholder.clone());
        }
        if !self.appearance {
            return combo_box.into_any_element();
        }

        // Keep GPUI Select's searchable menu and focus behavior. Theme only
        // the 16 px arrow affordance described by the component record.
        let style = required_style(context, "combo_box", "body", &selector(context));
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
            .child(combo_box.icon(Icon::empty()))
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
        "ComboBox options must be unique"
    );
    options
}

fn option_index(options: &[String], selection: &str) -> Option<IndexPath> {
    options
        .iter()
        .position(|option| option == selection)
        .map(IndexPath::new)
}
