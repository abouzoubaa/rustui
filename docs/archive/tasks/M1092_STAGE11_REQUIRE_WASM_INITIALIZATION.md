# M1092 — Stage 11: require the Web shell to initialize Wasm

The Web packager now requires its executable module script to call `await
init()` after importing the generated RustUI Playground module. A shell that
imports the module but never starts the Wasm application is rejected before
replacing the existing bundle.

The focused packaging harness passed, including rejection, preservation of
the prior bundle, and staging cleanup. No Wasm build or browser acceptance was
run; those Stage 11 gates remain open.
