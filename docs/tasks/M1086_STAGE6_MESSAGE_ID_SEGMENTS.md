# M1086 — Stage 6 message ID segments

`MessageId::parse` accepts dotted identifiers but now rejects a trailing dot
or consecutive dots, which would create an empty segment. `app.title` remains
valid.

Focused regression:
`./scripts/cargo-headstart test -p rustui-i18n --lib message::tests::rejects_empty_dotted_identifier_segments --locked --offline`
passed 1/1. This covers identifier validation only; locale direction,
formatting, fallback, runtime switching, RTL layout, and native screen-reader
acceptance remain separate.
