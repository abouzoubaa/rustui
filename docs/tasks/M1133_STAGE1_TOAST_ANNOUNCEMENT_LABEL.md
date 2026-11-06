# M1133 — expose Toast announcement labels

## Change

Toast status and alert nodes now expose their announcement text as an
accessible label as well as a live value. The status role remains polite and
the alert role remains assertive.

## Focused evidence

```text
./scripts/cargo-headstart test -p rustui --lib toast_announcement_uses_status_or_assertive_alert_semantics
1 passed
```

The file-scoped diff check passed. Native screen-reader acceptance remains
open.
