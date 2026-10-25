# RustUI release preparation

RustUI is not currently release-ready. The canonical blockers and acceptance
counts are in [the remaining-work audit](tasks/ROADMAP_REMAINING_AUDIT.md).
The declared targets are macOS on Apple Silicon and Web/Wasm in a browser;
no additional platform support is certified. Package manifests currently set
`publish = false`. Do not publish, sign for distribution, upload artifacts or
create a public release without the user's approval.

## Repeatable checks

Run from the registered implementation checkout. Preserve persistent caches;
do not run `cargo clean` to repair compiler/artifact mismatches.

```sh
./scripts/verify-release.sh source
./scripts/verify-release.sh macos
./scripts/verify-release.sh web
```

The source checks run Python tooling regressions, packaging fixtures, locale
records, theme plans and static adapter coverage. They do not compile Rust.
The macOS checks use host rustfmt 1.97.1 and the approved Headstart launcher
for the dependency boundary, all targets, library and public integration
tests, macro doctests, and an optimized Playground package. The Web check
produces the optimized JavaScript/Wasm bundle, SVG references and font/icon
license notices. Fixture packaging tests use stubs and are separate evidence
from the real target builds.

CI runs source checks on Ubuntu. The manual build job requires a dedicated,
trusted self-hosted macOS ARM64 runner labeled `rustui-headstart`, provisioned
with the approved compiler, SDK and matching Wasm sysroot. The workflow is
prepared; runner availability and a successful remote CI run remain unverified.
Do not automatically run untrusted pull-request code on this runner. The
checkout preserves the runner's caches; use a dedicated checkout and never
point it at a user's dirty development tree. The build job verifies the selected
Git revision and rejects tracked changes or nonignored untracked files after
checkout, while retaining ignored build caches.

## Compiler and dependency inputs

Use the approved `gpui-pre 0.3.9` snapshot and Components `0.7.3`. Follow
[the patch policy](../patches/README.md) to reproduce only approved patches
before building. Preserve vendored checksums and licenses; do not modify
vendored sources directly or substitute an upstream revision.

Headstart is experimental. It must be provisioned before CI: the launcher does
not bootstrap or download it. The inspected local source revisions on
2026-10-25 were:

| Input | Revision |
| --- | --- |
| Headstart checkout | `ff81bfecda2116d54776a7fec538fc2f4e349d1f` |
| Cargo checkout | `4f3fb2428282fb2467c4bb5be0163d2f42a31491` |
| rustc checkout | `6006fd03947bd7b04ae5cb5f7f9bb66b44dc8c6e` |

These are observed source identities, not proof that compiler binaries were
reproducibly built from clean sources. A release still needs a captured
compiler/bootstrap configuration, SDK identity and clean-build reproduction.
Keep `-Zmetadata-crate-hash=no`, caller flags and the scoped proc-macro
incremental workaround described in [BUILDING.md](BUILDING.md).

For native builds, `RUSTUI_HEADSTART_ROOT` defaults to
`$HOME/.local/opt/headstart`. For Web, `RUSTUI_WASM_HEADSTART_ROOT` defaults to
`target/headstart/wasm-root`; it must contain matching host and Wasm standard
libraries. The established bootstrap command and evidence are recorded in
[X1A_WASM_COMPILE.md](tasks/X1A_WASM_COMPILE.md). The Web script requires
`wasm-bindgen 0.2.129` and `rust-lld`; it does not install tools. Use
`RUSTUI_WASM_TARGET_DIR` for a persistent Web cache override. Keep all build
artifacts under `target/headstart`.

## Artifacts and acceptance

The native package is `target/headstart/debug/RustUI Playground.app` (or the
same path beneath `CARGO_TARGET_DIR`). The packaging script accepts
`RUSTUI_PLAYGROUND_BINARY` because Headstart artifact locations differ from
ordinary Cargo layouts. The release-check driver obtains that path from Cargo
JSON rather than guessing. Native build JSON is retained at
`target/headstart/evidence/release-checks/native-build.jsonl`.

The Web bundle is `target/headstart/playground-web/`. Serve this whole
directory, including references and license notices, over HTTP. Its
`bundle-manifest.json` records source and artifact hashes. The build rejects
source changes during compilation and missing, unreadable or symlinked inputs.
The manifest binds these files; it does not prove deterministic compiler output
or native/browser acceptance. This browser host permits the upstream
single-thread dispatcher fallback; cross-origin isolation/threaded operation
has separate acceptance requirements.

Before release, bind macOS GUI/keyboard/IME/VoiceOver evidence and browser
interaction/semantic-accessibility/font/IME evidence to current artifacts.
Reserve the shared GUI slot. Reuse the canonical disposable validation bundle,
then quit and remove its bundle and standalone executable after GUI testing,
preserving dependency caches and evidence. Do not stop user applications or
unowned processes. Performance budgets require actual measurements; source
tests and compilation are not substitutes.

## Distribution checkpoint

The macOS and Web/Wasm release checks save locked/offline Cargo metadata and
successful compiler-artifact JSON, then collect notices into unique directories
under target/headstart/evidence/. Native collections use the
native-dependency-notices-checked-* prefix; Web collections use
web-dependency-notices-checked-*. Both bind verbatim files, package identities,
hashes, explicit gaps, and review notes. The collector includes observed build
dependencies; it does not establish exact runtime closure or legal completeness
and writes only a fresh output directory. Each packaged validation bundle includes
the verified collection and the verbatim embedded font and icon license files.
Bundle manifests hash these files. Packaged reports retain input hashes, package
sources, manifest hashes, review notes, provenance, and unresolved gaps, while
replacing machine-local paths with stable repository or Cargo-source labels; the
original report remains in external evidence. Ordinary developer packaging does
not run the collection step. This records evidence and does not create a release
approval gate. Complete security dependency/license review, notices for all
distributed code/assets, supported macOS/browser versions, native
signing/notarization, artifact checksums, and rollback plan before requesting
publication approval. Native bundle and Web directory are local validation
outputs, not signed release distributions. Retain the previous approved artifact
manifest until replacement acceptance is complete; on failed validation, keep
serving the previous artifact. External distribution and package publishing
remain unperformed.
