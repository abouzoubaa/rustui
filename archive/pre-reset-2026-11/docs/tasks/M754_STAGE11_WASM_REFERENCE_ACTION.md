# M754 — Stage 11 / omit native reference launcher in WebAssembly

The Playground reference panel now keeps packaged browser links in WebAssembly
and omits the adjacent native-only action that launches the operating system's
`open` command. Native reference launching remains available on desktop.

Validation: `./scripts/cargo-headstart check -p rustui-playground --target
wasm32-unknown-unknown --locked --offline` passed. Scoped `git diff --check`
passed. Browser link click and asset-fetch behavior remain unverified; see
`docs/tasks/X1B_BROWSER_HOST.md`.
