# M899 — Stage 3 / add a controlled TokenField component

RustUI now exports `TokenField` and `TokenFieldView`. The caller controls the
committed `Vec<String>` and text draft through separate bindings. The explicit
Add action ignores blank drafts and trims committed values; each token exposes
an accessible remove action. The existing Input story uses TokenField for its
editable recipient-token specimen.

Focused checks passed:

```text
./scripts/cargo-headstart test -p rustui token_field::tests --lib
4 passed
./scripts/cargo-headstart test -p rustui-playground search_field_reference_gallery_starts_with_the_six_reference_states --lib
1 passed
```

The catalog remains Partial for unrepresented reference appearance/state
combinations. Native visual and accessibility acceptance remain open.
