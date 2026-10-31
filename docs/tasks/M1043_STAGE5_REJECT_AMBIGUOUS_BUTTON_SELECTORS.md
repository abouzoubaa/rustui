# M1043 — Stage 5 / reject ambiguous Button selectors

Button theme validation checked only the default light/dark queries. Two
equal-specificity rules could therefore overlap for a non-default selector,
pass schema validation, and fail only when the runtime resolver was asked for
that variant.

Button validation now uses the shared selector specificity and overlap rules
also used for typed component themes. The focused regression constructs a
dark-wide rule and a wildcard-appearance opaque rule, confirms runtime
resolution reports their ambiguity for dark/opaque, and confirms schema
validation rejects the theme before activation.

The focused locked/offline regression passed 1/1:

```text
./scripts/cargo-headstart test -p rustui --lib button_theme_rejects_variant_only_ambiguous_selectors --locked --offline
```

This closes ambiguous Button selector validation only. Native visual,
accessibility, and cross-platform theme acceptance remain open.
