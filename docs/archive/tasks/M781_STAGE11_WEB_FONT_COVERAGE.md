# M781 — Stage 11: Web font weight and Hebrew coverage

## Gap

The Web host registered regular Noto Sans and Arabic Noto Sans, plus only the Medium Fira Mono face; it did not include the vendored Inter variable font or Hebrew Noto Sans face. The Web theme therefore lacked the variable sans weight range and explicit Hebrew font coverage.

## Change

Copy the existing licensed `InterVariable.ttf` and `NotoSansHebrew.ttf` files from `vendor/cosmic-text-0.19.0/fonts/` into RustUI's font assets, preserving their license files. Register both for Wasm and select the Inter family in the Web theme. Native font selection continues to use the platform family.

## Focused evidence

- `cargo-headstart test -p rustui --lib web_font_registration_includes_variable_weights_and_hebrew --locked --offline`: passed.
- `cargo-headstart check -p rustui-playground --target wasm32-unknown-unknown --locked --offline`: passed.
- Both added font files match their pinned vendor copies byte-for-byte.
- `rustfmt --check` still reports pre-existing formatting differences in the shared dirty Rust files; those unrelated differences were left untouched.
- Browser weight selection, rendered glyph coverage, screen-reader output, and broader Stage 11 acceptance remain open.
