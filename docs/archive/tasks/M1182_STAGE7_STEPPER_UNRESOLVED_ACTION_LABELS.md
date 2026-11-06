# M1182 — Stage 7 / fall back from unresolved Stepper action names

The localization resolver returns a missing message key as its own string.
Stepper treated that successful interpolation as a translated action name,
exposing IDs such as `rustui-stepper-increment` to assistive technology.

The Stepper label resolver now treats failed, blank, and unchanged-key results
as unresolved and uses its existing “Increase/Decrease {label}” fallback.
Valid translated labels remain unchanged. The focused regression passed 1/1:

```sh
./scripts/cargo-headstart test -p rustui --lib components::input::stepper::tests::unresolved_action_names_use_english_fallbacks
```

Native screen-reader acceptance remains open.
