# M799 Stage 4 raw-keyword state accessor names

When a component state field uses a raw keyword identifier such as `r#type`, derived method names must omit Rust's raw-identifier prefix. The macro now generates `set_type`, `update_type`, `type_binding`, and `type_state` while preserving the field identifier used to read the state. The same name derivation is used by collision validation and method generation.

Focused verification passed:

```sh
./scripts/cargo-headstart test -p rustui-macros --lib component::tests::generated_state_accessors_normalize_raw_keyword_field_names --locked --offline
```

This closes the generated-name correctness gap for raw keyword fields. Broader macro integration coverage remains a separate acceptance gate.
