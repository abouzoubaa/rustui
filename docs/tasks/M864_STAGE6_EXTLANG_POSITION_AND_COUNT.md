# M864 — Stage 6 / enforce extlang position and count

## Contract

Locale parsing permits up to three three-letter extlang subtags only directly
after a two- or three-letter primary language and before script, region, or
variant subtags. Extension and private-use payloads retain their separate
grammar.

## Implementation

`Locale::parse` tracks whether the parser remains in the extlang position and
rejects extlang-shaped subtags after other core subtags or beyond the limit.
No registry lookup or locale dependency was introduced.

## Evidence

- Focused extlang regression passed 1/1.
- `./scripts/cargo-headstart test -p rustui-i18n locale::tests --lib --locked --offline` passed 8/8.
- `git diff --check -- crates/rustui-i18n/src/locale.rs` passed.

The parser enforces bounded structural order only; full IANA registry
validation and full ICU/CLDR behavior remain out of scope.
