# M502 — Stage 6 locale subtag ordering

`Locale::parse` now rejects duplicate or out-of-order script and region subtags
in the locale body. The accepted body order is script, region, then variants:
there may be at most one script and one region, and neither may appear after a
variant. Existing case-insensitive duplicate-variant rejection remains in
place.

The parser keeps extension and private-use payloads outside these body-order
checks, so payload subtags that resemble variants do not count as locale-body
variants. This is a bounded structural check; it does not claim full BCP 47
grammar or IANA registry validation.

## Verification

- `./scripts/cargo-headstart test -p rustui-i18n locale::tests --lib --locked --offline`
  passed 4/4 focused locale parser tests.
- Evidence: `target/headstart/evidence/m502-locale-ordering-tests-20261021.log`.
