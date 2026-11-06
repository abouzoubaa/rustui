# M1017 — Stage 7: release the ComboBox writer borrow before confirmation

ComboBox confirmation clones the current retained binding writer and drops the
`RefCell` borrow before invoking the writer. This allows a synchronous rerender
to refresh the writer slot without reentrant borrowing failure.

The focused regression `retained_confirmation_releases_writer_slot_before_applying`
passed 1/1.

Current option-change tests cover reconciliation decision helpers, but do not
exercise retained Select behavior when options are replaced or the selected
value is removed, or when disabled state changes. Broader keyboard, focus,
VoiceOver, and cross-platform acceptance remain open.
