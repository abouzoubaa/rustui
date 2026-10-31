# M890 — Stage 6 English India number grouping

Integer and finite decimal formatting now use Indian digit grouping for English
India locale tags such as `en-IN`: the rightmost group has three digits and
preceding groups have two. Other currently supported locale formatting keeps
its existing grouping behavior.

Focused regression command and result:

```text
./scripts/cargo-headstart test -p rustui-i18n --lib --locked --offline formats_english_indian_integers_and_decimals_with_indian_grouping
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 61 filtered out; finished in 0.00s
```

This covers `format_integer` and `format_decimal` for one `en-IN` sample. It
does not establish full CLDR/ICU behavior, formatting for other Indian locales,
or native locale, RTL, accessibility, and cross-platform acceptance.
