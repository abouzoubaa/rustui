# M600 — Stage 6 ordinal Playground sample

## Acceptance

The canonical Localization story includes one `selectordinal` message in each
shipped English, French, and Arabic catalog. The sample formats `3` as `3rd`,
`3e`, and `الترتيب ٣`, respectively, exercising locale selection and ordinal
`#` substitution. The story test also checks that catalogs remain complete and
their message contracts agree.

## Verification

- Ordinal gallery regression passed 1/1.
- The locked/offline Playground library suite passed 100/100.
- Locked/offline Playground build passed.
- Locale validation passed for all three runtime catalogs.
- Scoped rustfmt check passed for the formatter and changed Playground files.

Evidence: `target/headstart/evidence/m600-playground-localization-tests-20261021.log`,
`target/headstart/evidence/m600-playground-build-20261021.log`, and
`target/headstart/evidence/m600-playground-ordinal-locales-20261021.log`.

This is a localized source/gallery sample. It does not establish full CLDR
ordinal support, native GUI locale switching, VoiceOver, RTL layout, or
cross-platform acceptance.
