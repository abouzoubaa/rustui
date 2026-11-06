# M1045 — Stage 7 ColorPicker localized implicit identity

ColorPicker's implicit retained identity included its visible label. A locale
change at the same owner slot therefore allocated a different retained picker
and discarded retained interaction state. The fallback identity now uses the
stable component kind; owner slot allocation still distinguishes sibling
pickers, and explicit IDs remain authoritative.

The focused regression verifies both same-slot stability across a translated
label and distinct identities for same-label siblings:

```sh
./scripts/cargo-headstart test -p rustui --lib components::input::color_picker::implicit_identity_tests::same_label_siblings_get_distinct_stable_implicit_identities -- --exact
```

Result: 1 passed. Native popup continuity, focus, and accessibility acceptance
remain separate runtime gates.
