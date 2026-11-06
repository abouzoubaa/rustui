# M1056 — Stage 5 / reject invalid component appearance selectors

Typed non-Button component theme rules accepted arbitrary non-empty appearance
strings. Because validation resolved each rule against its own selector, a rule
using an unsupported value such as `system` passed validation even though
runtime appearance queries use `light` or `dark` and could never select it.

Component theme validation now accepts only `any`, `light`, or `dark` for the
appearance field. The focused regression changes a Divider rule to `system`
and confirms that theme validation rejects it.

The focused locked/offline regression passed 1/1:

```text
./scripts/cargo-headstart test -p rustui --lib component_theme_rejects_unresolvable_appearance_selectors --locked --offline
```

This closes unsupported appearance selector validation only. Native visual,
accessibility, and cross-platform theme acceptance remain open.
