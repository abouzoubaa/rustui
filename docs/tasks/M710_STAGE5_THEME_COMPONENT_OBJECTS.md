# M710 — Stage 5 component record object contract

The theme JSON Schema requires every `components` entry to be an object, and
the M596 runtime contract preserves unknown/custom component objects as opaque
extensions. `ThemeDocument::validate` previously skipped semantic validation
for every non-object non-Button value, allowing a theme that the schema rejected
to pass the Rust loader.

The runtime validator now rejects non-object component records with the
component name in the error. Unknown component objects remain opaque and
round-trip unchanged; typed `parts`/`styles` records retain M596 validation.

Focused regression:

```sh
./scripts/cargo-headstart test --locked --offline -p rustui --lib \
  theme::loader::tests::parser_rejects_non_object_custom_component_records -- --exact
```

Result: passed, 1/1. This aligns runtime acceptance with the existing JSON
Schema and documented extension contract. Adapter behavior is unaffected;
native visual theme acceptance remains a separate open gate.
