# D1c TextField preview specimen

The compiled preview gallery offers an opt-in `text-field-specimen` backed by
RustUI's public `TextField` API. Its schema exposes the actual controlled
`value` and `placeholder` strings. The placeholder inspector binding updates
the owner's placeholder independently from the value binding. The compiled
factory passes the placeholder to `TextField` and includes it in the retained
identity so the retained input state initializes with the edited placeholder.
Changing the placeholder therefore replaces `InputState` and resets transient
cursor, selection, and IME composition state for that field.
The accessible name remains a fixed specimen label; disabled state is not
exposed. Accepted property edits use the existing stamp-checked preview
feedback path, and descriptor import/export preserves both strings. The event
map stays empty under the signal-only contract, and the descriptor format is
unchanged.

The focused test verifies both schema defaults, signal-only events, independent
value and placeholder feedback, stale value-stamp rejection, and descriptor
round-trip. It passed via:

`./scripts/cargo-headstart test -p rustui-playground --lib stories::compiled_preview_gallery::tests::text_field_specimen_uses_stamp_checked_value_and_round_trips --locked --offline`

No GUI run was performed, so native text editing, accessibility, locale, IME,
and cross-platform behavior for this specimen remain unverified.
