# M1026 — Reject boolean theme format versions in validation

Python considers `True == 1`. The theme validation script therefore treated a
runtime-shaped document with `format_version: true` as version 1 and skipped
it, although the Rust `ThemeDocument` deserializer requires an integer format
version.

`scripts/validate_themes.py` now requires the JSON value to be an integer
before classifying a runtime pack as version 1. The focused regression passed:

```text
python3 scripts/tests/test_validate_themes.py
Ran 1 test, OK
```

Scoped whitespace validation passed. This closes a schema-classification gap
only; native visual theme acceptance remains open.
