# M998 — describe the Table catalog route accurately

The `table` catalog note now matches its `StoryId::Table` route: a public
RustUI Table story with controlled row selection, reordering, removal and
restoration, plus a DataGrid accessibility example. A focused catalog test
checks the note and route.

The focused check passed:

```text
./scripts/cargo-headstart test -p rustui-playground table_catalog_describes_the_controlled_table_story --lib
1 test passed
```

Native visual, keyboard, and accessibility acceptance remain pending.
