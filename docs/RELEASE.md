# RustUI release verification and packaging

RustUI is not yet cleared for production distribution. Use the current
[remaining-work inventory](tasks/ROADMAP_REMAINING_AUDIT.md) and component
contracts as the acceptance checklist. A successful build or a bounded GUI
exercise does not close unrelated requirements.

## Targets and evidence

| Target | Current evidence | Outstanding release scope |
| --- | --- | --- |
| macOS native | Optimized arm64 builds and disposable Playground packages; bounded functional checks on macOS 27 | Supported OS/version policy, wider functional/visual/IME/theme/RTL/workload checks, clean build reproduction, compiled CI, signing and notarization |
| Web/Wasm browser | Optimized Playground bundle, hashed build inputs/assets, bounded functional checks in one Chromium profile | Supported browser/version policy, wider browser/IME/clipboard/font/theme/RTL/workload checks, clean build reproduction and compiled CI |

Linux CI checks the source tooling. It does not establish a Linux product target.
Do not infer Windows, Linux, mobile, or other platform support from GPUI's APIs.
Accessibility acceptance and VoiceOver are **undone and deferred** by the user.
Keep existing behavior and evidence; the deferral is not a passed check.

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

Dependency notice collection currently has five native and four Web gaps.
See the [notice provenance and limitations](release/dependency-notices/README.md).
Complete the license/notice and compatibility review, including runtime and
build-only attribution, before claiming the distribution is cleared. Do not
invent copyright statements or substitute an unrelated upstream revision.

Preserve candidate archives, executable/bundle hashes, notice reports, source
provenance, validation results and a manifest together under
`target/headstart/release-artifacts/`. A hash of one archive establishes its
identity; it does not establish an independent reproducible build. Compare
artifacts from the required clean compiler/bootstrap reproduction separately.

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
