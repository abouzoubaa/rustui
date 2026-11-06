# M1203 — Stage 5 apply the badge background token

RustUI's Badge wrapper now passes the active theme profile's
`badge_background` color to the pinned GPUI Components Badge. Previously the
upstream default used GPUI's error color. If no RustUI theme is active, the
wrapper keeps the existing GPUI fallback.

The focused regression checks the profile token in both light and dark
appearances and confirms it differs from the error token.

Verification: `./scripts/cargo-headstart test -p rustui
badge_background_uses_the_profile_token_for_both_appearances` passed (1 test).
`git diff --check -- crates/rustui/src/components/basic/badge.rs` passed.
