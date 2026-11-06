# M1183 — Stage 6: Imperial Aramaic explicit-script direction

Date: 2026-10-23

Status: Implemented; focused regression passed. This closes the identified source-owned RTL direction gap, not Stage 6 acceptance as a whole.

## Finding

The explicit BCP 47 script `Armi` (Imperial Aramaic) was absent from the RTL script allowlist. Unicode 17 UCD assigns `Armi` to `U+10840..U+10855`, `U+10857`, and `U+10858..U+1085F` (`U+10856` is unassigned); DerivedBidiClass assigns the script characters bidi class `R`. Consequently, `und-Armi` fell through to the default left-to-right direction.

## Change

- Added `Armi` to `is_right_to_left_script` in `crates/rustui-i18n/src/direction.rs`.
- Added `und-Armi` to the explicit-script direction regression.

## Verification

- `./scripts/cargo-headstart test -p rustui-i18n --lib explicit_script_overrides_language_direction --locked --offline` — passed (1 test, 72 filtered out).

## Remaining

Stage 6 still requires its broader locale, shaping, and native/browser RTL acceptance. This milestone establishes the locale-direction result for the explicit `Armi` script only.
