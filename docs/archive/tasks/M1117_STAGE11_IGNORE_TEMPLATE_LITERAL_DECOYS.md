# M1117 — Stage 11 / ignore template-literal decoys in Web shell checks

The Web bundle validator searched the generated HTML for Wasm module imports
and initialization calls. Its line-based checks could mistake matching text
inside a multiline JavaScript template literal for executable bootstrap code.

Both checks now track multiline template-literal state and exclude literal
content from import/initializer matches. The regression places apparent import
and initialization code only inside a template literal and verifies rejection
while preserving the previous bundle.

The regression failed before the source fix. Afterward, `bash -n
scripts/build-playground-web.sh`, `sh -n
scripts/tests/test_build_playground_web.sh`, and
`sh scripts/tests/test_build_playground_web.sh` passed.

This improves packaging validation only. Browser runtime, pointer/keyboard,
asset fetching, accessibility, and font/glyph acceptance remain open.
