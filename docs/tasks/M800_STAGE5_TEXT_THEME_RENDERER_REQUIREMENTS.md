# M800 Stage 5 Text theme renderer requirements

The Text renderer expects every resolved style rule to contain `style.colors["foreground"]` and `style.typography`. Theme validation previously allowed those properties to be absent, so malformed theme data could pass validation and later panic during rendering. Validation now rejects rules missing either renderer-required property.

Focused verification passed:

```sh
./scripts/cargo-headstart test -p rustui --lib theme::schema::tests::text_theme_requires_renderer_foreground_role --locked --offline
```

The regression removes `foreground` from a text style rule and checks that validation returns an error. This closes the accepted-invalid-theme panic path; broader visual theme acceptance remains open.
