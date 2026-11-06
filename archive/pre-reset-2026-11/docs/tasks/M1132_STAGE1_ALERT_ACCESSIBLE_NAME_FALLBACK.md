# M1132 — keep unresolved Alert keys out of accessible names

## Change

When an Alert's localized title or message is unresolved, the component no
longer uses the message identifier as visible content or as its accessible
name. It prefers a resolved title and otherwise uses the validated Alert
identity as a stable fallback.

## Focused evidence

```text
./scripts/cargo-headstart test -p rustui --lib components::overlay::alert::tests
11 passed
```

`rustfmt --edition 2024 --check crates/rustui/src/components/overlay/alert.rs`
and the scoped diff check passed. Native screen-reader acceptance remains
open.
