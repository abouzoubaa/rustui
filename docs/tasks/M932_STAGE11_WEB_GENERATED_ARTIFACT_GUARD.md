# M932 — Stage 11 / reject incomplete generated Web bundles

The Web packaging script now requires non-empty `rustui_playground.js` and
`rustui_playground_bg.wasm` outputs before copying the shell or publishing the
staging directory. If the generator exits successfully without either browser
entrypoint, the script fails and its cleanup preserves the existing bundle.

The focused fake-tool package harness verified a successful bundle, then
simulated a successful `wasm-bindgen` exit with the Wasm artifact missing. The
packager rejected the incomplete output, retained the previous bundle, and
removed its staging directory. Browser startup, rendering, input, asset
loading, accessibility, and platform acceptance remain open.
