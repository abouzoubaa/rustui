# Build workflow

## Recording current build provenance

After an observed build/check, record its log and the current workspace inputs:

```sh
python3 scripts/record_build_provenance.py target/headstart/roadmap-milestone2-check-final.log \
  --command './scripts/cargo-headstart check --workspace --all-targets --locked' \
  --exit-status 0 --profile dev
```

Use the exit status actually observed for that command. Omitting it records an
unverified result. The script creates a unique JSON file under
`target/headstart/evidence/`; it does not execute a build or infer success from log
text. Hashes describe files at capture time, so record evidence promptly and retain
the log. Compiler version strings are optional caller-supplied metadata; unknown
provenance remains unknown. Dependency reuse still requires the separate reuse
checker and Cargo JSON artifact evidence. See
`docs/tasks/P2A_BUILD_PROVENANCE.md` for the complete contract.

## Headstart

Use the project-local launcher for Cargo commands:

```sh
./scripts/cargo-headstart build --workspace --locked
./scripts/cargo-headstart check --workspace --locked
./scripts/cargo-headstart playground
rustup run 1.97.1 cargo fmt --all -- --check
```

The launcher selects installed Headstart Cargo and stage1 rustc from
`$HOME/.local/opt/headstart` (override with `RUSTUI_HEADSTART_ROOT`).
It enables `-Zheadstart` and keeps `-Zmetadata-crate-hash=no` visible to Cargo.
That approved workaround addresses early/full metadata SVH disagreement. It is
experimental; compilation success is not evidence of production compiler stability.

A second narrow workaround disables incremental compilation only for the local
`rustui-macros` proc-macro crate in dev/headstart profiles. On 2026-10-18, changed
proc-macro source still triggered the same SVH assertion even with the metadata-hash
flag. GPUI and all other workspace crates retain incremental compilation. ICE reports
are preserved under `target/headstart/evidence/rustc-ice-*.txt`.
No speedup measurements are claimed for this workspace.

Existing `CARGO_ENCODED_RUSTFLAGS` or `RUSTFLAGS` are preserved and appended.
Otherwise the workaround is supplied via Cargo's `build.rustflags` configuration.
Keep flags consistent across warmup and development. Do not silently change
compiler binaries, features, profiles, targets, source pins, or wrapper settings.

`rust-toolchain.toml` pins 1.97.1 for host tooling such as rustfmt and clippy.
The Headstart launcher deliberately chooses a different experimental compiler.
It does not download or upgrade that installation.
Use `rustup run` explicitly: the current PATH resolves `cargo` to Homebrew, whose
binary does not accept the `cargo +toolchain` shorthand.

## Warm, development, and audit aliases

The Cargo aliases use the shared `headstart` profile and `target/headstart/` cache:

```sh
./scripts/cargo-headstart warm
./scripts/cargo-headstart dev
./scripts/cargo-headstart audit
```

`warm` builds the workspace, `dev` runs RustUI Playground, and `audit` checks all workspace targets.
`playground` is the canonical application alias; `lab` remains a compatibility alias
that launches the same UI through the thin `rustui-lab` package.

## Local dependency sources and persistent build cache

Both live inside this RustUI folder, as requested:

- `vendor/`: unmodified Cargo-vendored source packages, including GPUI and Components.
- `target/headstart/`: compiled dependencies, incremental state, binaries, and evidence.

The approved direct pins are `gpui-pre = 0.3.9`, `gpui-pre-platform = 0.3.9`, and
`gpui-component = 0.7.3`. Rust uses the dependency aliases `gpui` and `gpui_platform`.
The snapshot records Zed revision `93d745668805029476e6d7fbb19652116fed51de`.
Platform features enable font rendering, Linux windowing, and runtime Metal shader
compilation. Optional GPUI Fast, speech, and extra language grammars are not enabled.

`.cargo/config.toml` replaces crates.io with `vendor/`. The full lockfile source set
is retained, including packages for other targets and inactive optional features;
presence in `vendor/` does not mean a package is compiled or enabled. Preserve
upstream license files and `.cargo-checksum.json` files, and do not edit vendored
code directly. Controlled source patches require their own approval.

The launcher and Cargo configuration default to `target/headstart/`. A caller can
explicitly override `CARGO_TARGET_DIR`, but keep it inside RustUI for this workflow.
`RUSTUI_CACHE_DIR` and `XDG_CACHE_HOME` no longer select RustUI's build location.
Deleting this RustUI folder or running `cargo clean` deletes its compiled cache.
Cargo may retain duplicate download copies in its usual registry cache, but this
workspace resolves and compiles its dependency sources from `vendor/`.

Cargo already decides which artifacts are reusable. A fixed dependency graph,
compiler, flags, profile, target, and features allows unchanged GPUI and Components
to stay fresh while RustUI recompiles. Changes to those inputs legitimately require
rebuilding. Build scripts may also invalidate dependencies. No cache can guarantee
a binary remains valid forever.

Do not run `cargo clean` during routine work. Do not share this cache with Chorus:
its patched sources/features/compiler configuration can differ. Sharing the same
cache path alone does not make those artifacts compatible.

## Warmup and evidence

Warm the full native development build, then verify an unchanged rebuild:

```sh
mkdir -p target/headstart/evidence
./scripts/cargo-headstart build --workspace --frozen
./scripts/cargo-headstart build --workspace --frozen --message-format=json > target/headstart/evidence/reuse.jsonl
python3 scripts/verify_dependency_reuse.py target/headstart/evidence/reuse.jsonl
```

The verification script requires one successful Cargo build event and fresh
artifacts for `gpui-pre`, `gpui-pre-platform`, and `gpui-component`. It fails if any
is absent or rebuilt. For another approved package name, repeat `--package NAME`.
Check-only artifacts do not replace a full build warmup. Debug/release and target
architectures have different artifacts; warm the configuration actually used.

`--frozen` requires the existing lockfile and disables Cargo network access.
After a future approved dependency update, regenerate the lockfile and vendored
sources with `./scripts/cargo-headstart vendor --locked --versioned-dirs vendor`,
then repeat the native build and freshness check. Keep the same source location
and feature set for warmup and normal development.

## Planning metadata


```sh
python3 scripts/validate_themes.py --plans-only
python3 scripts/validate_locales.py
```

The theme command validates planning records only. The locale command validates the
three runtime locale records, the supported catalog syntax, duplicate/empty messages,
and matching message-ID sets across the framework and Playground catalog groups. It does not
claim pluralization, interpolation, formatting, bidi, or visual acceptance.

## Structured dependency reuse evidence (P2b)

Pass `--report PATH` to the existing reuse checker to save its result with the
source log hash and per-artifact Cargo metadata:

```sh
python3 scripts/verify_dependency_reuse.py target/headstart/evidence/reuse.jsonl \
  --report target/headstart/evidence/reuse-report.json
```

The default package set and success criteria are unchanged. Exit 0 means all
required package artifacts were present and fresh; exit 1 means a required package
was missing or rebuilt; exit 2 means invalid input or an I/O error. The report
cannot overwrite its input log. Keep the report beside the original Cargo JSON
log and a provenance record when recording a build.

Cargo check evidence describes check artifacts. It does not establish executable
build-cache reuse, GUI correctness, or a performance improvement. Use a full build
log when assessing the development executable cache.

## Hot reload

Data-only themes/catalogs may eventually reload with explicit validation and
invalidation. Rust source changes use incremental Cargo rebuilds. Arbitrary Rust
hot reload is not implemented or promised.

## Initial scaffold validation — 2026-10-16

- Headstart Cargo: `1.101.0 (4f3fb2428 2026-09-30)`.
- Headstart rustc: `1.101.0-dev`, host `aarch64-apple-darwin`, LLVM `23.1.1`.
- `build --workspace --locked` succeeded for all six local packages.
- Verbose compiler output included the stage1 compiler, `-Zearly-metadata`, and
  `-Zmetadata-crate-hash=no` with the persistent target path.
- A subsequent unchanged build reported every local package artifact as fresh;
  `verify_dependency_reuse.py` passed with all six package names supplied.
- `rustup run 1.97.1 cargo fmt --all -- --check` and launcher shell syntax passed.
- All 12 theme and 3 locale planning records passed their planning checks.
- Utility checks exercised fresh/missing/rebuilt/malformed build evidence,
  unsupported/incomplete planning records, and flag/cache override preservation.
- At that date, Lab printed its scaffold notice and no native GUI was opened. This
  historical result was superseded by the later native Lab implementation.

Headstart reports four unused dependency warnings for the reserved local package
edges. Their code is not implemented yet; warnings were not suppressed.
No Rust component tests, visual tests, accessibility tests, or benchmarks ran.
That initial check covered only local scaffold packages. The user subsequently
approved the snapshot pins and moved both sources and artifacts inside RustUI.
The dependency build and its cache verification are recorded separately below.

## Approved dependency build — 2026-10-16

- `cargo vendor --locked --versioned-dirs vendor` retained all 797 external
  packages from `Cargo.lock`. Their package checksums match the lockfile.
- Cargo metadata confirms external source paths under `rustui/vendor/`, build
  outputs under `rustui/target/headstart/`, and the selected snapshot revision.
- The full macOS/aarch64 build succeeded through Headstart with `--frozen -j 8`.
  The verbose log confirms the stage1 compiler and metadata-hash workaround.
- The first repeat reused every third-party artifact and rebuilt two local
  scaffold packages. A further unchanged build reused all 488 artifact records,
  including all 482 third-party records; no workspace package rebuilt.
- The dependency-reuse checker passed for `gpui-pre`, `gpui-pre-platform`, and
  `gpui-component`. Cargo reported 1m 32s for warmup and 1.33s for the final repeat.
  These are observed build durations, not a controlled Headstart benchmark.
- Formatting and shell syntax checks passed. No component/native GUI tests ran.

Evidence is retained in `target/headstart/evidence/`: `warmup.log`,
`warmup.jsonl`, `metadata.json`, `reuse-confirmed.jsonl`, and
`reuse-confirmed.log`. The generated tree includes optional/cross-platform sources;
Allui is absent from the entire lockfile and GPUI Fast is absent from the active graph.

Warnings remain for unused scaffold dependency edges and a future-compatibility
warning in upstream `block 0.1.6`. The latter is recorded in
`target/headstart/evidence/future-incompatibilities.log`. Neither warning was
suppressed and no vendored source was patched. At the time, Lab still had no native
window; this historical evidence does not describe the current Lab source.

## Runtime ownership validation — 2026-10-18

- `rustup run 1.97.1 cargo fmt --all -- --check` passed.
- `./scripts/cargo-headstart check --workspace --all-targets --locked` passed.
- Lab now depends only on the public `rustui` crate and starts through `RustUI`.
- No interaction, visual, accessibility, or performance tests ran in this milestone.

## State and identity validation — 2026-10-18

- `rustup run 1.97.1 cargo fmt --all -- --check` passed.
- `./scripts/cargo-headstart check --workspace --all-targets --locked` passed.
- The check covered generated state/binding methods in Lab, Playground, and examples.
- No interaction, visual, accessibility, or performance tests ran in this milestone.

## Lab story-system validation — 2026-10-18

- `rustup run 1.97.1 cargo fmt --all -- --check` passed after formatting.
- `./scripts/cargo-headstart check --workspace --all-targets --locked` passed.
- The check covered the typed story registry, canvases, controls, event log, source
  snippets, layout facts, and verification-status wiring.
- The Lab was not launched, and no interaction, visual, accessibility, or performance
  tests ran in this milestone.

## Initial-component validation — 2026-10-18

- `rustup run 1.97.1 cargo fmt --all -- --check` passed after formatting.
- The first full locked check exposed the Headstart proc-macro SVH ICE described above.
- After the local proc-macro incremental override,
  `./scripts/cargo-headstart check --workspace --all-targets --locked` passed twice.
- No interaction, visual, keyboard, accessibility, or performance tests ran, so those
  acceptance statuses remain pending.

## Typed Button theme validation — 2026-10-18

- `python3 -m json.tool themes/theme.schema.json` passed.
- `./scripts/cargo-headstart check --workspace --all-targets --locked` passed.
- `./scripts/cargo-headstart run -p rustui --example theme_loader --locked --
  themes/macos27.json` succeeded and loaded profile `macos27` in Dark appearance.
- Lab and Playground compile against complete-document `activate_theme`.
- No GUI or visual comparison ran; macOS 27 appearance acceptance remains pending.

## Localization foundation validation — 2026-10-18

- `python3 scripts/validate_locales.py` passed for the three runtime locale records
  and both catalog groups.
- `rustup run 1.97.1 cargo fmt --all -- --check` passed.
- `./scripts/cargo-headstart check --workspace --all-targets --locked` passed.
- The check covered the RustUI i18n crate, GPUI global invalidation integration,
  localized text, logical RTL stacks, application actions, and the Lab selector.
- No localization interaction, GUI, visual, bidi, accessibility, or formatting tests
  ran; those acceptance claims remain pending.

## Retained text-input foundation validation — 2026-10-18

- `rustup run 1.97.1 cargo fmt --all -- --check` passed.
- `./scripts/cargo-headstart check --workspace --all-targets --locked` passed.
- The check covered binding snapshots, keyed retained GPUI entities, retained event
  subscriptions, `TextField`, `SecureField`, `SearchField`, and the fourth Lab story.
- No GUI, typing, focus, IME, keyboard, accessibility, or visual interaction was run;
  runtime acceptance remains pending.

## Multiline and Boolean control validation — 2026-10-18

- `rustup run 1.97.1 cargo fmt --all -- --check` passed.
- `./scripts/cargo-headstart check --workspace --all-targets --locked` passed.
- The check covered `TextEditor`, `Toggle`, `Checkbox`, `Radio`, their
  binding event paths, and the expanded Lab input story.
- No GUI, typing, pointer, keyboard, focus, IME, accessibility, or visual interaction
  was run; runtime acceptance remains pending.

## Slider and Stepper validation — 2026-10-18

- `rustup run 1.97.1 cargo fmt --all -- --check` passed.
- `./scripts/cargo-headstart check --workspace --all-targets --locked` passed.
- The check covered retained `SliderState`, two-way `Binding<f32>` synchronization,
  bounded numeric `Stepper` actions, public exports, and the expanded Lab story.
- No GUI, pointer, keyboard, focus, accessibility, or visual interaction was run;
  runtime acceptance remains pending.

## Picker and ComboBox validation — 2026-10-18

- `rustup run 1.97.1 cargo fmt --all -- --check` passed.
- `./scripts/cargo-headstart check --workspace --all-targets --locked` passed.
- The check covered retained GPUI `SelectState` and `ComboboxState`, two-way
  `Binding<String>` synchronization, dynamic option replacement, public exports,
  and the expanded Lab story.
- No GUI, pointer, keyboard, focus, search, accessibility, or visual interaction was
  run; runtime acceptance remains pending.

## DatePicker and ColorPicker validation — 2026-10-18

- The first locked check correctly stopped because the new direct `chrono` declaration
  required lock metadata refresh.
- `./scripts/cargo-headstart check --workspace --all-targets --offline` refreshed the
  lock without network access and then passed after one ownership fix.
- `rustup run 1.97.1 cargo fmt --all -- --check` passed.
- `./scripts/cargo-headstart check --workspace --all-targets --locked` passed.
- The check covered retained GPUI `DatePickerState` and `ColorPickerState`, optional
  date/color binding synchronization, public exports, and the expanded Lab story.
- No GUI, calendar, palette, keyboard, focus, accessibility, localization, or visual
  interaction was run; runtime acceptance remains pending.

## Virtualized List validation — 2026-10-18

- `rustup run 1.97.1 cargo fmt --all -- --check` passed.
- `./scripts/cargo-headstart check --workspace --all-targets --locked` passed.
- The check covered retained GPUI `ListState`, string-row delegation, filtering,
  source-index selection synchronization, public exports, and the dedicated Lab story.
- No GUI, scrolling, search, pointer, keyboard, focus, accessibility, large-data, or
  visual interaction was run; runtime acceptance remains pending.

## Virtualized Table validation — 2026-10-18

- `rustup run 1.97.1 cargo fmt --all -- --check` passed.
- `./scripts/cargo-headstart check --workspace --all-targets --locked` passed.
- The check covered retained GPUI `TableState`, `DataTable` virtual rows, validated
  string row shapes, controlled row-selection synchronization, public exports, and the
  dedicated Lab story.
- No GUI, scrolling, pointer, keyboard, focus, accessibility, large-data, or visual
  interaction was run; runtime acceptance remains pending.

## Virtualized Tree validation — 2026-10-18

- `rustup run 1.97.1 cargo fmt --all -- --check` passed.
- `./scripts/cargo-headstart check --workspace --all-targets --locked` passed.
- The check covered retained GPUI `TreeState`, virtualized hierarchy rendering,
  unique node-ID validation, GPUI notification observation, controlled stable-ID
  selection, public exports, and the dedicated Lab story.
- No GUI, expansion, scrolling, pointer, keyboard, focus, accessibility, large-data,
  or visual interaction was run; runtime acceptance remains pending.

## TabView validation — 2026-10-18

- `rustup run 1.97.1 cargo fmt --all -- --check` passed.
- `./scripts/cargo-headstart check --workspace --all-targets --locked` passed.
- The check covered unique stable tab IDs, `Binding<String>` selection, GPUI Components
  `TabBar` integration, selected RustUI child lowering, public exports, and the
  dedicated Lab story.
- No GUI, pointer, keyboard, focus, accessibility, overflow, indicator-animation, or
  visual interaction was run; runtime acceptance remains pending.

## Breadcrumb validation — 2026-10-18

- `rustup run 1.97.1 cargo fmt --all -- --check` passed.
- `./scripts/cargo-headstart check --workspace --all-targets --locked` passed.
- The check covered optional RustUI component actions, stable path identity, disabled
  items, GPUI Components breadcrumb presentation, public exports, and the dedicated
  Lab story.
- No GUI, pointer, focus, accessibility, navigation-flow, or visual interaction was
  run; runtime acceptance remains pending.

## AnyView and heterogeneous TabView validation — 2026-10-18

- `rustup run 1.97.1 cargo fmt --all -- --check` passed.
- `./scripts/cargo-headstart check --workspace --all-targets --locked` passed.
- The check covered RustUI-owned `AnyView` erasure and a Lab `TabView` containing
  different concrete child view types.
- No GUI, interaction, lifecycle, accessibility, or visual verification ran; runtime
  acceptance remains pending.

## NavigationStack validation — 2026-10-18

- `rustup run 1.97.1 cargo fmt --all -- --check` passed.
- `./scripts/cargo-headstart check --workspace --all-targets --locked` passed.
- The check covered a bound `Vec<String>` route path, heterogeneous destinations,
  forward links, back navigation, public exports, and the dedicated Lab story.
- No GUI, pointer, keyboard, focus, accessibility, deep-link, state-restoration, or
  visual interaction was run; runtime acceptance remains pending.
## External runtime measurements

Sample an already running Playground PID. This does not launch the app. Choose a
new run ID and summary filename each time; evidence cannot be overwritten.

```sh
python3 scripts/collect_runtime_metrics.py --pid PID --duration 10 --interval 0.25 --label playground-idle --workload idle --run-id idle-001
python3 scripts/summarize_runtime_metrics.py target/headstart/performance/process-idle-001.json --output target/headstart/performance/summary-idle-001.json
```

Runtime JSON files must be direct children of `target/headstart/performance/`.
Optional `--provenance PATH` hashes an existing provenance file; it does not
establish which executable the process loaded. Inspect completion status and
errors before interpreting a report. CPU/RSS and sampling overhead are external
process observations, separate from Cargo artifact freshness/build duration.
They do not establish FPS, frame latency, GPU memory, or workload completion.

## W1 calibration tools

`collect_w1_baseline.py` uses the existing process sampler with a bounded maximum
capture duration. Supply a fresh run ID and stop path, run the fixed native CUA
sequence, then create the stop file (`touch UNIQUE_STOP_PATH`). The recorder takes
a final sample and saves exclusive evidence. It never launches or signals the app.
`w1_native_benchmark.js` is CUA REPL code, not a standalone Node launcher.
`summarize_w1_baseline.py` validates the recorded three-session evidence and
recomputes descriptive CPU/RSS calibration. Use a unique report filename.
See `tasks/P2E_W1_ACCEPTANCE_BASELINE.md` for exact inputs, results and limitations.


## P3D paired cache comparison compile validation — 2026-10-18

- The final comparison-tool suite passed 21 Python tests (`target/headstart/p3d-final-python-tests.log`); its final guards exclude runs when accessibility or native drag is detected, including late observations. No native comparison or benchmark measurement has run; these checks make no performance claim. See `docs/tasks/P3D_CACHE_COMPARISON.md` for the native protocol and its remaining documentation refresh.
## GPUI ownership and event-path tests

RustUI's test-only GPUI dependency enables the pinned upstream `test-support`
feature. Eighteen additional registry packages were restored from the retained
worktree after verifying official registry checksums, archive contents, source
checksums and license files. No existing package versions or checksums changed.
The macOS and Wasm Playground normal/build dependency trees were compared before
and after this addition and are unchanged; the application does not enable the
test harness. All sources remain under `vendor/`.

```sh
./scripts/cargo-headstart test -p rustui --lib component_lifecycle_tests --profile headstart --locked --offline
./scripts/cargo-headstart test -p rustui --lib focused_divider_routes --profile headstart --locked --offline
```

These tests use GPUI's fake windows and production render/event/owner paths. They
do not launch Playground or establish native/browser accessibility, VoiceOver,
IME or visual acceptance. The fake backend does not project AX trees; the
SplitView event test observes the retained native `ResizableState` after key
dispatch. Its read-only retained-slot inspector is compiled only for tests.

## Stable generated-source paths for native release artifacts

`RUSTUI_REMAP_BUILD_PATHS=1` opts the Headstart launcher into stable source/build
path mappings. It maps the physical repository root to `/rustui/src` and the
selected target directory to `/rustui/build`, preserving caller flags and the
approved `-Zmetadata-crate-hash=no` workaround. Encoded caller flags retain
precedence over unencoded `RUSTFLAGS`; paths containing spaces remain single
encoded arguments. Python 3 is required for this opt-in mode. Development defaults
are unchanged. Values other than `0` and `1` are rejected.

The native optimized build in `scripts/verify-release.sh macos` enables the mode.
Select and record the same installed SDK and toolchain when comparing builds.
Two initially absent native target directories on the current Mac produced
byte-identical executables after this mapping. Without it, html5ever embeds its
absolute generated `OUT_DIR/rules.rs` filename and changes executable bytes.
This establishes same-host, same-toolchain native target-directory reproduction;
cross-machine and complete native archive reproduction remain separate. The
approved installed compiler is identified by its binary hash and configuration;
binding that binary to an exact source snapshot or reproducing its bootstrap is
a disclosed provenance follow-up, not an original RustUI release requirement.
Evidence: `target/headstart/evidence/native-clean-reproduction-20261026/`.

Web reproduction was also checked on source `39da1d7b` using two initially absent
`RUSTUI_WASM_TARGET_DIR` directories, the same approved Wasm compiler and
wasm-bindgen, and explicit `RUSTUI_REMAP_BUILD_PATHS=1`. Each build recorded 496
rebuilt / 0 fresh Cargo artifacts. Raw Wasm, processed Wasm and all 770 package
files (including the manifest) match byte-for-byte. Keep each copied bundle before
the next build replaces `target/headstart/playground-web`.
Evidence: `target/headstart/evidence/web-clean-reproduction-20261026/`.
