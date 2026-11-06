# M1049 — Reject blank FocusScope accessible labels

`FocusScopeView::label` could previously store a whitespace-only accessible name. That allowed the focus container to emit an empty `aria-label` when assistive technology focused it.

The builder now rejects labels whose trimmed value is empty and preserves nonblank labels verbatim. A focused regression covers whitespace-only input.

Focused check:

```text
./scripts/cargo-headstart test -p rustui --lib interaction::focus::tests::blank_accessible_label_is_rejected -- --exact
```

Result: passed (1 test). Native GUI and assistive-technology acceptance remain open.
