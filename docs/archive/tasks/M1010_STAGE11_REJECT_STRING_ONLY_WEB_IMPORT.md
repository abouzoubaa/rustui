# M1010 — Stage 11: reject string-only Web module imports

The Web packager previously accepted JavaScript string content containing the expected `rustui_playground.js` import text as if it were an executable module bootstrap. The shell guard now recognizes only the expected default-import statement, outside block comments.

The package harness includes a string-only fixture and verifies that rejection preserves the previous bundle and removes staging output. Validation reported for this change: `sh scripts/tests/test_build_playground_web.sh`, `bash -n scripts/build-playground-web.sh`, and `sh -n scripts/tests/test_build_playground_web.sh` passed.

This closes a packaging-validation false positive only. Browser startup, rendering, input, asset loading, accessibility, and platform acceptance remain open.
