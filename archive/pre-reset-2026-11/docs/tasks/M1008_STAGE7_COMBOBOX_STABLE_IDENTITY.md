# M1008 — keep ComboBox identity stable across label changes

ComboBox previously derived implicit retained identity from its visible label.
Localized labels can change between renders, which changed the identity and
could discard retained Select/search state. Use the stable `combo-box` base
instead; explicit `.id(...)` continues to take precedence.

Extended the sibling identity regression to check that `Choice` and `Choix`
at the same owner slot retain the same identity.

Focused check passed 1/1:

```text
./scripts/cargo-headstart test -p rustui components::input::combo_box::implicit_identity_tests::implicit_identity_survives_label_changes_and_distinguishes_siblings -- --exact
```

`git diff --check -- crates/rustui/src/components/input/combo_box.rs` passed.
Native interaction, focus, and accessibility acceptance remain open.
