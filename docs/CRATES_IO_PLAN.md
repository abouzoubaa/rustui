# Crates.io distribution plan

Assessment date: 2026-11-11. This is a plan only: no crate names, manifests, code, tags, or registry releases were changed.

## 1. Publish set and count

The requested full RustUI/GPUI release cohort is **36 package names**: five RustUI library packages, all 29 `gpui-*` packages present in `vendor/`, and two patched third-party forks. `cargo metadata --format-version 1 --locked` found 733 reachable packages and 18 path-source packages for this checkout. Its current `rustui` dependency graph reaches 20 `gpui-*` packages and the two patched forks; it also reaches `cocoa`, `cocoa-foundation`, and `metal` as path packages. Those three have existing upstream registry packages and should become versioned registry dependencies if their source remains unchanged. `rustui-rgpe` is a public RustUI library but is not in `rustui`'s current dependency closure. The broader 36-package cohort includes platform and GPUI Kit packages needed for the whole owned snapshot, not just today's macOS facade.

| Group | Count | Packages |
|---|---:|---|
| RustUI library packages | 5 | `rustui`, `rustui-core`, `rustui-i18n`, `rustui-macros`, `rustui-rgpe` |
| GPUI / GPUI Kit package family | 29 | Every package in the name table below |
| Patched third-party forks | 2 | `block` → `rustui-block`; `cosmic-text` → `rustui-cosmic-text` |
| **Full release cohort** | **36** | 5 + 29 + 2 |

The five RustUI libraries are the publishable library packages among the seven workspace packages reported by `cargo metadata --no-deps`; the other two are Playground/Lab applications. For the strict current `rustui` graph, `rustui`, `rustui-core`, `rustui-i18n`, and `rustui-macros` are required; the GPUI set and both patched forks must also be resolvable. Keep `rustui-rgpe` in the public release cohort only if it remains a supported standalone library.

Commands and measured output:

```text
$ cargo metadata --format-version 1 --no-deps > /tmp/cratesio-metadata.json
$ python3 -c '...'
workspace package count 7
publishable RustUI library packages: rustui, rustui-core, rustui-i18n, rustui-macros, rustui-rgpe

$ cargo metadata --format-version 1 --locked > /tmp/cratesio-full-metadata.json
reachable packages total 733; path packages 18
reachable gpui-* package names 20; patched path forks: block 0.1.6, cosmic-text 0.19.0

$ python3 -c '... scan vendor/*/Cargo.toml for package.name starting gpui-'
29
```

The patched forks are necessary because a consumer without `[patch]` would otherwise resolve the upstream registry source, which does not contain RustUI's approved changes. Give each fork a distinct package name and update its dependants to that name; do not publish a conflicting replacement under `block` or `cosmic-text`.

## 2. Registry names

The crates.io API returned an existing crate record for all 29 upstream names below. Proposed RustUI-prefixed names returned HTTP 404 for each query on 2026-11-11 (available at lookup time, not reserved). The `rustui` crate itself is already taken by this project; `rustui-core`, `rustui-i18n`, `rustui-macros`, and `rustui-rgpe` returned 404 at the same check. Name availability can change before publication.

| Existing upstream package (taken) | Proposed package name (checked free) |
|---|---|
| `gpui-base` | `rustui-gpui-base` |
| `gpui-component` | `rustui-gpui-component` |
| `gpui-component-macros` | `rustui-gpui-component-macros` |
| `gpui-kit` | `rustui-gpui-kit` |
| `gpui-kit-assets` | `rustui-gpui-kit-assets` |
| `gpui-pre` | `rustui-gpui` |
| `gpui-pre-apple` | `rustui-gpui-pre-apple` |
| `gpui-pre-bench-metrics` | `rustui-gpui-pre-bench-metrics` |
| `gpui-pre-collections` | `rustui-gpui-pre-collections` |
| `gpui-pre-derive-refineable` | `rustui-gpui-pre-derive-refineable` |
| `gpui-pre-http-client` | `rustui-gpui-pre-http-client` |
| `gpui-pre-http-client-tls` | `rustui-gpui-pre-http-client-tls` |
| `gpui-pre-ios` | `rustui-gpui-pre-ios` |
| `gpui-pre-linux` | `rustui-gpui-pre-linux` |
| `gpui-pre-macos` | `rustui-gpui-pre-macos` |
| `gpui-pre-macros` | `rustui-gpui-pre-macros` |
| `gpui-pre-platform` | `rustui-gpui-pre-platform` |
| `gpui-pre-refineable` | `rustui-gpui-pre-refineable` |
| `gpui-pre-reqwest` | `rustui-gpui-pre-reqwest` |
| `gpui-pre-reqwest-client` | `rustui-gpui-pre-reqwest-client` |
| `gpui-pre-scheduler` | `rustui-gpui-pre-scheduler` |
| `gpui-pre-shared-string` | `rustui-gpui-pre-shared-string` |
| `gpui-pre-sum-tree` | `rustui-gpui-pre-sum-tree` |
| `gpui-pre-util` | `rustui-gpui-pre-util` |
| `gpui-pre-web` | `rustui-gpui-pre-web` |
| `gpui-pre-wgpu` | `rustui-gpui-pre-wgpu` |
| `gpui-pre-windows` | `rustui-gpui-pre-windows` |
| `gpui-pre-ztracing` | `rustui-gpui-pre-ztracing` |
| `gpui-pre-ztracing-macro` | `rustui-gpui-pre-ztracing-macro` |
| `block` | `rustui-block` |
| `cosmic-text` | `rustui-cosmic-text` |

The three existing Cocoa/Metal packages are not part of this fork-name table: use their upstream registry packages if their published source and version satisfy the owned manifests. Before release, check every target-specific dependency and the proposed names again; the 29-package family table is based on the actual `vendor/*/Cargo.toml` inventory.

## 3. Proc-macro crate-name lookups

`rg -n 'proc_macro_crate|crate_name\(' crates vendor/gpui* --glob '*.rs'` finds the following two lookup implementations in this source set:

| File and lines | Lookup / generated path | Effect of renaming |
|---|---|---|
| `vendor/gpui-component-macros-0.8.1/src/crate_path.rs:6-15` | `gpui()` probes package names `gpui-kit`, `gpui-pre`, and `gpui-fast`. | A renamed Kit or engine package is not discovered unless this macro recognizes the RustUI package name or a tested Cargo alias. |
| `vendor/gpui-component-macros-0.8.1/src/crate_path.rs:31-42` | `component()` resolves `gpui-kit`, then falls back to `gpui-component`. | The `IntoPlot` derive expansion needs the renamed component path as well as the engine path. It is invoked from `derive_into_plot.rs:9-13`; the derive is exported at `src/lib.rs:45`. |
| `vendor/gpui-pre-macros-0.3.10/src/gpui_pre_facade_paths.rs:12-15` | Checks for `gpui-kit` and rewrites `gpui` / `gpui_platform` paths to that facade. | Kit renaming changes macro-generated paths for GPUI derives and attributes. The lookup implementation is in the macro crate, so the fix must be included in its published source. |

Other GPUI proc macros in `gpui-pre-macros/src/gpui_macros.rs` (including `Action`, `IntoElement`, `Render`, `AppContext`, and `VisualContext`) use the facade rewrite helper; they do not make additional `proc_macro_crate` lookups. The project `crates/rustui-macros` has no `proc_macro_crate` lookup. Verify with the search command above after each upstream refresh.

Options:

1. **Patch owned macro crates to accept RustUI package names (recommended).** In the patch-only workflow, update `gpui-component-macros` and `gpui-pre-macros` lookup/rewrite logic to recognize the `rustui-gpui-*` names while preserving upstream names for compatibility. Add integration tests that compile derives under the renamed dependency graph. This gives consumers ordinary dependency names and no special alias setup.
2. **Cargo dependency alias/re-export trick.** Consumers could declare an alias such as `gpui-kit = { package = "rustui-gpui-kit", ... }` and re-export the renamed crates. This is smaller if `proc_macro_crate` resolves that dependency key exactly as assumed, but it is fragile across macro lookup behavior and still asks every downstream app to carry internal GPUI aliases. Prove it with a fresh external consumer before choosing it.
3. **Publish unchanged upstream names to a separate registry.** Upstream names can coexist in a private registry, but users must configure that registry and Cargo source policy. It does not meet the requested default `cargo add rustui` experience on crates.io.

Option 1 is the most predictable for the public registry. Its main risk is maintaining a small divergence in two macro crates; keep that difference as reviewed patches and cover both derive and attribute expansions in the consumer test.

## 4. Licensing and attribution

The sampled GPUI/Kit manifests declare `Apache-2.0` and include `LICENSE-APACHE`; `cosmic-text` declares `MIT OR Apache-2.0` and includes both license texts; the patched `block` crate declares MIT. RustUI's workspace package metadata declares MIT. The sampled GPUI roots did not contain a `NOTICE` file; every package must still be checked before publishing.

For each republished source crate, retain its upstream license text, copyright and attribution notices, and applicable README/repository attribution; mark modified files or document the RustUI patch set. Apache-2.0 section 4 requires the license to accompany source redistribution, preservation of relevant notices, and redistribution of any applicable upstream NOTICE content. MIT requires the copyright and permission notice in copies or substantial portions. Set each package's Cargo `license` accurately, and include RustUI attribution without replacing upstream attribution. Preserve third-party notices generated from the lockfile as a separate distribution artifact. This plan is a process checklist, not legal advice.

Evidence commands:

```text
$ for d in vendor/gpui-pre-0.3.10 vendor/gpui-base-0.8.1 vendor/gpui-component-0.8.1 vendor/gpui-kit-0.8.1 vendor/cosmic-text-0.19.0 vendor/block-0.1.6; do ls "$d" | rg '^(LICENSE|NOTICE)'; done
gpui-pre, gpui-base, gpui-component, gpui-kit: LICENSE-APACHE
cosmic-text: LICENSE-APACHE, LICENSE-MIT
block: no LICENSE file at crate root; Cargo.toml declares MIT (carry the upstream license text/notices from its source package before publishing)
```

Sources: [Apache License 2.0 §4](https://www.apache.org/licenses/LICENSE-2.0.html), [MIT license terms](https://choosealicense.com/licenses/mit/).

## 5. Versioning and A3 updates

Keep a stable RustUI package-name mapping once published. For a renamed GPUI package, publish the initial RustUI fork at the matching upstream version when the source is otherwise identical; where RustUI patches alter the package, use the next compatible patch version and record the source snapshot and patch set. Update internal dependency requirements to the RustUI package names and versions. Do not rely on build metadata to distinguish registry releases because Cargo version requirements ignore it.

For A3, fetch and hash-pin GPUI Kit 0.9.x and its matching gpui-pre snapshot with the existing upgrade harness, rebase patches, rerun proofs and license checks, then publish new versions under the same RustUI names in dependency order. Preserve 0.8.x releases for existing consumers. RustUI's own `0.x` public facade should increment its minor version for breaking public API changes and its patch version for compatible fixes; record each release in the changelog and tag the exact commit. Any package name or package-graph change after first publication requires a migration note.

## 6. Size and publish order

Cargo documents a **10 MB `.crate` archive limit** and recommends `cargo package --list` / `cargo publish --dry-run` to inspect what will be uploaded ([Cargo Book: publishing](https://doc.rust-lang.org/cargo/reference/publishing.html)). Measured extracted snapshot sizes (`du -sk`, KiB) were gpui-pre 8,652; gpui-base 5,292; gpui-component 4,852; gpui-kit 1,152; gpui-kit-assets 7,764. The asset snapshot contains 1,849 files with 1,160,958 bytes of logical file content (about 1.11 MiB); much of its disk allocation comes from many small icons. `cargo package --list --manifest-path crates/rustui/Cargo.toml --allow-dirty` listed 211 RustUI files totaling 188,209 bytes, including its fonts.

These extracted-tree figures are not compressed `.crate` archive sizes. No 10 MB violation is indicated by the observed file totals, but archive sizes for every fork still need to be measured with a successful `cargo package --dry-run` in an isolated packaging fixture. Several current mirror manifests cannot be packaged directly from `vendor/`: Cargo reports the vendor directory as part of the root workspace or rejects Cargo's reserved `.cargo_vcs_info.json`; two mirror attempts also encountered stale vendor-index versions. Fix the packaging fixture without altering vendor snapshots and measure all 36 archives before the first publish.

Recommended dependency order:

1. `rustui-block` and `rustui-cosmic-text` forks.
2. GPUI leaf/helper crates (`gpui-pre-*` collections, refineable, scheduler, sum-tree, shared-string, utilities, and macro crates) in their dependency order.
3. GPUI engine and platform crates (`gpui-pre`, Apple/macOS and the other supported platform packages).
4. GPUI Kit assets and Base.
5. Component macros, Component, then the `gpui-kit` facade.
6. RustUI helper libraries (`rustui-core`, `rustui-i18n`, `rustui-macros`, and `rustui-rgpe` if retained), then `rustui` last.

Use a generated topological publish list from Cargo metadata rather than relying on this family-level ordering alone.

## 7. Git distribution alternative and recommendation

| Route | User experience | Estimated RustUI work | Trade-off |
|---|---|---:|---|
| Git tags only | `rustui = { git = "…", tag = "v0.x.y" }`; the repository already has tag-based consumer checks. | 1–2 hours for a release/consumer documentation pass; about 1–2 hours per later release. | Lowest maintenance; does not make bare `cargo add rustui` work and retains Git checkout overhead. |
| Crates.io | `cargo add rustui`; registry dependency resolution, no Git source and no downstream `[patch]`. | Roughly 16–20 hours, split below, plus an owner-approved publication window. | Better default install path, but requires a 36-crate family, package renaming, macro compatibility, license files, and permanent ordered uploads. |

**Recommendation: proceed with crates.io as the primary install path, while keeping signed/tagged Git dependencies as a supported fallback.** This meets the explicit simple-install goal and avoids making consumers reproduce the current vendor patch graph. Do not begin publication until the renamed-package consumer test and all package dry-runs pass; uploads are permanent and cannot be overwritten.

## 8. Ticket split (each ticket ≤2 hours)

1. **CR-1 — Freeze publish graph and mapping (2h):** generate a target-aware package DAG for macOS and supported target configurations; recheck names and decide whether `rustui-rgpe` is in the release cohort.
2. **CR-2 — Macro-name compatibility (2h):** patch the two upstream macro lookup implementations, add renamed-consumer derive/attribute tests, and prove alias behavior is unnecessary.
3. **CR-3 — Patch the packaging fixture (2h):** create a temporary package workspace that handles path dependencies and reserved Cargo files without editing vendor sources; produce successful dry-run archives and sizes.
4. **CR-4 — Publish patched third-party forks (2h):** package/license-check and publish `rustui-block` and `rustui-cosmic-text`; update the unpublished dependency graph only after both artifacts exist.
5. **CR-5 — GPUI helper wave 1 (2h):** publish the first dependency-ordered GPUI leaf packages with notices and archive checks.
6. **CR-6 — GPUI helper/platform wave 2 (2h):** publish the remaining helper crates and target backends; verify each target resolves from the registry.
7. **CR-7 — Engine and Base (2h):** publish renamed gpui-pre, GPUI Base, and their required dependencies after fresh consumer checks.
8. **CR-8 — Kit graph (2h):** publish assets, macro, Component, and Kit facade in dependency order; run package and registry-resolution checks.
9. **CR-9 — RustUI registry manifests (2h):** switch package manifests to versioned RustUI names and publish helper libraries before the facade; keep the Git consumer passing.
10. **CR-10 — Release rehearsal (2h):** generate the final topological upload script, test `cargo add rustui` plus online consumer builds from a clean temporary project, verify notices and tags, then request owner approval for the irreversible upload.

Existing path-based CI remains usable while these tickets publish registry packages; CI switches to the registry graph only after the clean consumer check succeeds.
