# M928 — Stage 3 / add Save Dialog reference controls

The Playground `save_dialog` composition now places editable Save As and Tags
fields and a File Format ComboBox inside the existing public `Dialog`. Save
reports the selected name and format; Cancel retains its existing result.

Focused regression passed:

```text
./scripts/cargo-headstart test -p rustui-playground save_dialog_form_controls_update_their_owner_bindings --lib
1 passed
```

`rustfmt --edition 2024 --check apps/rustui-playground/src/stories/dialog_gallery.rs`
passed. Native visual and accessibility acceptance remain open.
