# M1005 — describe the Text catalog route accurately

The Text catalog row said “RustUI baseline demo” even though it routes to the
shared catalog-message localization gallery. Update it to identify the
interpolation and plural/ordinal/select examples and clarify that a dedicated
standalone Text specimen is not implemented.

Focused catalog assertion passed 1/1:

```text
./scripts/cargo-headstart test -p rustui-playground text_catalog_describes_the_story_it_routes_to --lib
```

Native interaction and visual acceptance remain open.
