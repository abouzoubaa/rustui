# Stage 6 — full locale runtime gap audit

Date: 2026-10-23

## Finding

Full CLDR/ICU/Fluent behavior cannot be completed faithfully within the current
approved dependency and implementation boundary. No source change was made:
adding a few hand-written locale cases would extend the existing bounded
formatter without meeting the roadmap's full-coverage requirement.

## Current source and dependency evidence

- `crates/rustui-i18n/Cargo.toml` declares no dependencies, and the locked
  offline dependency tree for `rustui-i18n` contains only the crate itself.
- `crates/rustui-i18n/src/formatting.rs` implements a RustUI-owned message
  subset: named interpolation, `plural`, `selectordinal`, and `select`, with
  explicit category logic and locale-specific number-formatting branches.
  The module documents bounded English, French, and Arabic cardinal rules and
  uses `other` for other languages unless an exact integer branch matches.
- `Cargo.lock` contains transitive ICU4X locale-core, normalizer, properties,
  and provider packages used elsewhere in the workspace. It contains no ICU4X
  message, number, date, or plural formatter packages; those transitive crates
  are not declared as RustUI localization dependencies and do not supply the
  required complete formatter/data surface here.
- `fluent-uri` in `Cargo.lock` is a URI parser, not Project Fluent. No Fluent
  runtime or Fluent syntax implementation is present in `rustui-i18n`.
- `chrono` is an approved workspace dependency of `rustui` for date types, but
  `rustui-i18n` has no date/time formatting API and `chrono` does not supply the
  missing CLDR locale formatting rules.
- `docs/tasks/STAGE0_LOCALIZATION_ARCHITECTURE_TRUTH.md` already distinguishes
  implemented bounded behavior from incomplete full CLDR, date formatting,
  Fluent terms, and broader locale behavior.

## Required next decision

Choose and approve the Stage 6 runtime architecture before implementation:

1. Pin an ICU4X/Fluent dependency and locale-data strategy for every supported
   target, including version/update policy, data size/licensing, and whether
   the full data is compiled in or supplied by a provider; or
2. Define a host-native formatting adapter per target and specify how message
   syntax, plural operands, and results stay consistent across targets.

Then define conformance evidence for message syntax, cardinal/ordinal rules,
number/date/time formatting, locale negotiation, and supported CLDR/ICU
versions. Until that decision exists, adding manual locale branches would not
close Stage 6 and must not be recorded as full locale support.

## Acceptance status

This is a source/dependency audit only. No build, test, locale GUI, RTL, or
cross-platform acceptance was performed. Stage 6 full CLDR/ICU/Fluent runtime
coverage remains open.
