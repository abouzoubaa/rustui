# M880 — Stage 6 private-use-only locale tags

## Change

The BCP 47 parser now accepts private-use-only tags such as
`X-Company-Tag`, normalizes them to lowercase, and preserves the tag through
`primary()` and `without_extensions()`. It rejects empty and overlong
private-use payloads.

## Focused evidence

- `accepts_private_use_only_tags_and_rejects_empty_or_oversized_payloads` passed (1 passed, 60 filtered).
- Scoped `git diff --check` passed.

## Acceptance boundary

This adds private-use-only structural parsing. It does not establish full
IANA registry or ICU/CLDR validation.
