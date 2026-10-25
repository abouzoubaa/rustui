# M1134 — avoid duplicate Choice editor labels

## Change

PropertyInspector does not render a separate row label when the Choice editor
already renders its own visible Picker label. The editor classification and
its focused regression now cover Choice alongside Toggle and Stepper.

## Focused evidence

```text
./scripts/cargo-headstart test -p rustui --lib avoids_repeating_labels_owned_by_choice_toggle_and_stepper_editors
1 passed
```

Broader UI Studio composition and native accessibility acceptance remain open.
