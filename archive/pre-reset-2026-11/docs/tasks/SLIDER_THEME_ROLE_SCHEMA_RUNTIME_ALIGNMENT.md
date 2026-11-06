# Slider Theme Role Schema/Runtime Alignment

## Finding

`ThemeDocument::validate` checked the slider's typed style records but left its flattened `metadata.theme_roles` unchecked. The slider renderer reads `track`, `thumb`, and `disabled` from that map and panics when a role is absent or does not name one of the supported profile color roles (`secondary`, `primary`, or `muted_foreground`). Thus a document could pass theme validation and fail later while rendering a slider.

## Change

Schema validation now checks those three role entries for typed slider records. Extension metadata remains preserved, and extra role entries remain allowed.

## Focused evidence

`slider_theme_roles_must_be_supported_by_runtime` mutates the valid macOS 27 slider's thumb mapping to an unsupported runtime role and confirms `ThemeDocument::validate` rejects it.

Command: `./scripts/cargo-headstart test -p rustui --lib theme::schema::tests::slider_theme_roles_must_be_supported_by_runtime`

Result: 1 passed, 0 failed.
