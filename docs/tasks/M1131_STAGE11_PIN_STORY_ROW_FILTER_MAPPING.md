# M1131 — pin story selection to the rendered filter

## Change

`Playground::story_binding` now captures the filtered story indices associated
with the row list that created the binding. A row event arriving after the
query changes maps through its original rendered list instead of using the
new query's row order.

## Focused evidence

The regression changes the query between binding creation and row selection
and confirms the event selects the story displayed at that row:

```text
./scripts/cargo-headstart test -p rustui-playground --lib stale_story_binding_keeps_the_row_mapping_from_its_rendered_filter
1 passed, 174 filtered out
```

The scoped source diff check passed. This does not establish Web browser
runtime acceptance.
