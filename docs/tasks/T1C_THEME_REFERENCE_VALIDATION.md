# T1C: Validate format-v1 Button theme references

## Contract

`ThemeDocument::validate`, and therefore `parse_theme` and `serialize_theme`,
resolves the existing typed Button theme for both Light and Dark appearance
modes. This checks that each mode has a matching Button style and that its
height, spacing, and corner-radius references resolve against the existing
platform snapshot and `token_sources`.

The validation calls the current `resolve_button` implementation after the
document's structural and platform checks. The resolver reads `ThemeDocument`
and does not call `validate`, so this introduces no validation cycle. It does
not change format version 1, add dependencies, or change vendor sources.

Subsequent Stage 5 contracts added object-shape validation for `source`,
`token_sources`, `derivation`, and component entries. Typed non-Button component
records are now validated through the shared resolver; opaque object-valued
custom components, Button `parts`, and additional metadata remain preserved.
This does not add general semantic token types, serialize view/state trees, or
add UI Studio integration.

## Acceptance

- A missing Button spacing alias is rejected while parsing.
- A Button missing a Light or Dark resolver match is rejected while parsing.
- A Button style whose height disagrees with its metric reference is rejected
  during validated export.
- Existing format-v1 `macos27.json`, normalized round-trip, and arbitrary
  custom-component metadata remain accepted.

Validation is covered by focused unit tests in `theme/loader.rs`. The root task
ran 12 focused loader tests and six focused resolver tests under the persistent
Headstart profile. One default test-profile attempt hit the known Headstart
incremental compiler ICE before tests ran; the dedicated profile completed both
suites successfully.

## Executed validation

`./scripts/cargo-headstart test --workspace --lib --tests --locked --offline` passed 90 tests across 12 binaries with zero failures. The three new rejection cases ran. Exact log: `target/headstart/milestone31-32-tests-fixed.log`. Earlier compilation errors in test setup were corrected using mutable map access; the original failed log is retained.
