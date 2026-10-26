# RustUI release verification and packaging

RustUI is not yet cleared for production distribution. Use the current
[remaining-work inventory](tasks/ROADMAP_REMAINING_AUDIT.md) and component
contracts as the acceptance checklist. A successful build or a bounded GUI
exercise does not close unrelated requirements.

## Targets and evidence

| Target | Current evidence | Outstanding release scope |
| --- | --- | --- |
| macOS native | Optimized arm64 builds, same-host executable reproduction and disposable Playground packages; bounded functional and 8/8 English/French/Arabic input checks on macOS 27 | Supported OS/version policy, wider functional/visual/theme/RTL/workload checks, cross-machine/archive reproduction, compiled CI, signing and notarization |
| Web/Wasm browser | Optimized Playground bundle, hashed build inputs/assets, bounded functional and 6/6 English/French/Arabic input checks in one Chromium profile | Supported browser/version policy, wider browser/real OS IME/clipboard/font/theme/RTL/workload checks and compiled CI |

Linux CI checks the source tooling. It does not establish a Linux product target.
Do not infer Windows, Linux, mobile, or other platform support from GPUI's APIs.
Accessibility acceptance and VoiceOver are **undone and deferred** by the user.
Keep existing behavior and evidence; the deferral is not a passed check.

## Current language scope

The user selected **English, French and Arabic** for the current release scope
on 2026-10-26. Validate their input, text rendering, translations and locale
behavior on the declared macOS and Web targets, including French accents and
Arabic RTL/mixed-direction text. Japanese and all other languages/scripts are
**undone and deferred** for a later release. Native Japanese IME acceptance is
unrun/deferred; it is not counted as a passed check. Preserve existing code,
fonts and evidence without claiming broader acceptance.

## Build the reviewed source

Use a clean registered isolated worktree, its verified owner claim, and the
approved compiler configuration. Record the commit, source-input hashes, lockfile,
compiler identities, host/SDK, command output and artifact hashes. Preserve
unrelated work and persistent dependency caches.

```sh
./scripts/verify-release.sh source
./scripts/verify-release.sh macos
./scripts/verify-release.sh web
```

The scripts use `./scripts/cargo-headstart`, locked/offline dependencies and the
reproducible approved upstream patches. Headstart remains experimental with the
approved `-Zmetadata-crate-hash=no` configuration. Preserve `gpui-pre` 0.3.9 and
Components 0.7.3. A missing compiler, tool, or locked source requires diagnosis;
do not change versions or clear caches to make the command pass.

The Web command requires the matching `wasm-bindgen` CLI 0.2.129 and approved
Wasm Headstart installation. `RUSTUI_WASM_HEADSTART_ROOT` selects that installed
compiler; add the approved tools directory to `PATH` when it is not already
available. Outputs remain under this checkout's `target/headstart/`.

The native command collects dependency notices and packages the exact executable
reported by Cargo. Its output identifies the disposable `RustUI Playground.app`.
The Web package is `target/headstart/playground-web`, with
`bundle-manifest.json`. Independently verify every packaged file and build-input
hash before testing or distributing it. Do not bind an old executable to a new
commit by changing a label.

## Validate and preserve a candidate

Reserve the shared Playground GUI slot. Run the outstanding functional,
error/recovery, ownership, keyboard/IME, theme/RTL, localization, browser and
representative workload checks against the exact candidate. Record actual
results and exclusions separately for macOS, browser, source tests and
performance. Keep accessibility and VoiceOver marked undone while deferred.

The `a8290b24` candidate passed 8/8 bounded native and 6/6 bounded Web
English/French/Arabic input checks in the live compiled preview. Native French
marked accent and Arabic keyboard entry passed. Web `typeText` checked Unicode
entry and selection, while browser automation did not supply real macOS dead-key
composition; Web OS IME acceptance remains open. See
`target/headstart/evidence/native-en-fr-ar-selection-20261026/` and
`target/headstart/evidence/web-en-fr-ar-input-20261026/` for raw observations,
manifest hashes and cleanup.

Dependency notice collection currently has five native and four Web gaps.
See the [notice provenance and limitations](release/dependency-notices/README.md).
Complete the license/notice and compatibility review, including runtime and
build-only attribution, before claiming the distribution is cleared. Do not
invent copyright statements or substitute an unrelated upstream revision.

Preserve candidate archives, executable/bundle hashes, notice reports, source
provenance, validation results and a manifest together under
`target/headstart/release-artifacts/`. A hash of one archive establishes its
identity; it does not establish an independent reproducible build. Compare
artifacts from independent fresh-target RustUI builds separately. Rebuilding the
compiler from source is a separate provenance follow-up, not an original RustUI
release requirement; preserve and record the approved installed toolchain.

After each GUI exercise, quit only the owned test application, verify its process
is gone, remove its disposable bundle and standalone test executable, and release
the GUI slot. Preserve evidence, candidate archives and shared Cargo caches.

## Distribution checkpoint

The workflow in `.github/workflows/ci.yml` runs source checks on GitHub-hosted
Ubuntu. Compiled macOS/Web checks require an explicitly dispatched trusted
self-hosted Mac runner with the approved compiler/cache. Green source CI is not
compiled CI or runtime acceptance.

Before publication, resolve the supported-version matrix and all undeferred
release blockers; retain the deferred requirements explicitly in the release
scope. Provision the actual signing/notarization identity, verify the final
signed package on a clean target, publish its checksums, and retain the previous
verified artifact plus rollback instructions. Record the exact candidate and
distribution destination for the user's final publication approval. Pushing a
reviewable branch does not authorize publishing a release.

## Current reproduction checkpoint — 2026-10-26

The native optimized release command now enables `RUSTUI_REMAP_BUILD_PATHS=1`.
Two independent target directories, each with 555 Cargo artifacts reported rebuilt
and none fresh, produced the same executable SHA-256:
`d857e6b5b687951bcb20fd4fff0e1be2abecf7e2eb3b0c8ff696186e8acbffcf`.
The installed compiler and SDK 26.1 were held fixed. This fixes absolute generated
source paths affecting native binary identity; it does not prove compiler
bootstrap, cross-machine or complete native archive reproduction. Compiler-source
binding remains disclosed separately from reproduction of RustUI artifacts.

An unsigned local candidate is retained at
`target/headstart/release-artifacts/RustUI-Playground-macos-arm64-unsigned-repro-d857e6b5b687.tar.gz`,
with its adjacent JSON file manifest and checksum. The exact candidate passed one
W1 functional smoke cycle (seven expected states). Its CPU/RSS performance is not
substituted for the separate 90-cycle baseline on executable `15c71ff4…`.
Five native dependency-notice gaps remain. The bundle declares macOS 13.0 as its
minimum; current native runtime evidence covers macOS 27 arm64. A supported-version
policy and acceptance across its declared versions are still required.

Two independent Web builds of source `39da1d7b`, each in an initially absent
target directory with 496 rebuilt / 0 fresh Cargo artifacts, produced identical
raw Wasm, processed Wasm, JavaScript, assets and manifest: **770/770 packaged
files match** (769 payload files plus the manifest). Both builds explicitly used
`RUSTUI_REMAP_BUILD_PATHS=1` and the same approved Wasm compiler, Cargo and
wasm-bindgen 0.2.129. Raw Wasm SHA-256:
`8bfd3ffc6057d7fe845449f8509cbc547d9639f8cc841a7b780337737325a449`;
processed Wasm SHA-256:
`f12e4eee03a7392c37d14f822eab2899542a9102034aa5334262c7972591d078`.
Evidence: `target/headstart/evidence/web-clean-reproduction-20261026/`.
This is same-host, same-toolchain reproduction, with no new browser acceptance
claimed for these candidates. Four Web dependency-notice gaps remain.

Normalized archives independently produced from both Web packages also match.
Each archive contains 770 verified files, with sorted paths, fixed modes and
ownership, zero tar/gzip timestamps, and the same package-root name. The retained
candidate is
`target/headstart/release-artifacts/RustUI-Playground-web-unsigned-repro-f479c2d07e77.tar.gz`;
SHA-256 `f479c2d07e774af5b5dba5d799a47e8981e28644fd1aa841ec9f202c6e420b27`.
Its adjacent JSON records provenance and open gates. This artifact is local and
has not been published.
