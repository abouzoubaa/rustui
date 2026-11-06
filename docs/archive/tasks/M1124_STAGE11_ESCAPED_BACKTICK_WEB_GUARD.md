# M1124 — Stage 11 / ignore escaped backticks in Web shell checks

The Web packager tracks template literals while looking for executable module
imports and initializer calls. Its line-based scans previously treated an
escaped backtick as a template boundary, allowing decoy bootstrap code later in
the literal to satisfy the package guard.

Both scans now toggle template state only on unescaped backticks. The focused
fixture puts the apparent import and initializer after an escaped backtick
inside a template literal and checks that packaging is rejected while the
previous bundle is preserved.

Focused validation reported passing: `bash -n scripts/build-playground-web.sh`,
`sh -n scripts/tests/test_build_playground_web.sh`, and
`sh scripts/tests/test_build_playground_web.sh`.

This closes a packaging false-positive only. WebAssembly browser runtime,
pointer and keyboard interaction, asset fetching, accessibility, and
font/glyph acceptance remain open.
