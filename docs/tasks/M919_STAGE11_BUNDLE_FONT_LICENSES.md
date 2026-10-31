# M919 — Stage 11 / include embedded font licenses in the Web bundle

## Change

The canonical Playground Web package script now copies each `*-LICENSE` file
from `crates/rustui/assets/fonts/` into `licenses/fonts/` in the output bundle.
The Wasm binary embeds these font assets, so the distributed browser bundle
now carries their license texts too. The existing Web shell and SVG reference
copy paths are unchanged.

## Focused evidence

`sh scripts/tests/test_build_playground_web.sh` passed. The harness runs the
real package script in a disposable fixture with stub Cargo and wasm-bindgen
commands, then checks the shell, generated Wasm/JavaScript, SVG reference, and
byte-identical font licenses. `sh -n` passed for the package script and test.
The disposable fixture was removed; the repository's `target/` bundle was not
changed by the test.

Browser runtime, visual, input, and accessibility acceptance remain open.
