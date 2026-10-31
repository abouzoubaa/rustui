# Refresh retained TextField placeholders

`TextFieldView::into_element` initializes the retained GPUI `InputState` with
the placeholder only when the field is first created. Later renders previously
left that state using the old placeholder even when the view's placeholder
changed. The field now compares the retained `InputState` placeholder with the
current view value and calls the existing `InputState::set_placeholder` API
when they differ.

The focused regression covers changed and unchanged placeholder reconciliation
in `changed_placeholder_is_refreshed_in_retained_input_state`. This is a
RustUI-side synchronization fix; it does not require a GPUI API change.

Focused evidence: `./scripts/cargo-headstart test -p rustui --lib components::input::text_field::tests::changed_placeholder_is_refreshed_in_retained_input_state --offline` passed (1/1).
