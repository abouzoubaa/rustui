# M914 — Stage 9 / initialize optional Toast title inspector value correctly

The editable optional `Toast.title` property has no default and is omitted from
the mounted preview until set. Its inspector binding previously captured the
gallery's unrelated shared text as its initial value, even though later reads
returned an empty string. The inspector now initializes missing string values
as empty, matching the mounted preview.

The focused regression
`optional_toast_title_inspector_initial_value_does_not_use_shared_text` passed
1/1 with:

```text
./scripts/cargo-headstart test -p rustui-playground --lib stories::compiled_preview_gallery::tests::optional_toast_title_inspector_initial_value_does_not_use_shared_text --locked --offline
```

This is source-level inspector coverage only. Broad native rendering,
interaction, accessibility, locale, theme, RTL, and cross-platform acceptance
remain open.
