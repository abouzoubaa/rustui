# M854 — Stage 7 DatePicker identity across localized labels

Implicit DatePicker identity previously included its visible label. A locale
change therefore created a new retained DatePicker entity and reset calendar
navigation state even though the owner kept the same control in the same slot.

Implicit identity now uses the stable owner slot and DatePicker base. Explicit
IDs are unchanged. The regression confirms same-slot identity survives an
English-to-French label change while sibling slots remain distinct.

Focused evidence:

```text
./scripts/cargo-headstart test -p rustui --lib components::input::date_picker::tests --locked --offline --quiet
14 passed
```

Scoped rustfmt and `git diff --check` passed. This is source-level retained
identity evidence; native selection, VoiceOver, popup accessibility,
keyboard/IME, and cross-platform acceptance remain open.
