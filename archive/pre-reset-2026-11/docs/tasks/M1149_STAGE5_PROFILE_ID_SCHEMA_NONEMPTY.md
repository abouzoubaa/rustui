# M1149 — Stage 5: reject whitespace-only profile IDs in the theme schema

`ThemeProfile::validate` rejects profile IDs whose trimmed value is empty, but
the published JSON Schema previously accepted whitespace-only IDs because it
only required a string of length one. The profile `id` schema now also requires
a non-whitespace character with the `\\S` pattern.

The focused regression checks for the schema pattern and confirms the Rust
runtime rejects a whitespace-only profile ID:

```sh
./scripts/cargo-headstart test -p rustui theme_schema_rejects_whitespace_only_profile_ids_like_runtime --lib
```

This aligns the schema constraint with runtime validation. Broader native
visual, accessibility, and cross-platform theme acceptance remain open.
