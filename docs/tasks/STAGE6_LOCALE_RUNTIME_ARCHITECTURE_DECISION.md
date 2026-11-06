# Stage 6 — locale runtime architecture decision

Date: 2026-10-23

## Decision needed

Stage 6 already has a bounded locale parser, catalog/fallback support, logical
direction, and a RustUI-owned message formatter. Completing the roadmap's full
CLDR/ICU/Fluent behavior requires selecting a runtime and data policy before
implementation. The existing gap audit, `STAGE6_FULL_LOCALE_RUNTIME_GAP_AUDIT.md`,
identified the same approval gate; this note makes the choices and target/API
boundary explicit.

## Current boundary and evidence

- `crates/rustui-i18n/Cargo.toml` has no direct dependencies. Its public surface
  exports `Locale`, catalog/fallback/validation types, `interpolate`,
  `interpolate_with_locale`, `format_integer`, and `format_decimal`.
- `crates/rustui-i18n/src/formatting.rs` documents bounded English, French, and
  Arabic cardinal behavior; other languages use `other` except exact integer
  branches. It has no date/time formatting API.
- The lockfile contains ICU locale-core, normalization, properties, and provider
  support packages, but no ICU number/date/plural formatter packages. They are
  transitive packages, not an approved RustUI localization runtime. `fluent-uri`
  is a URI parser, not Project Fluent.
- `crates/rustui/src/localization/` coordinates active locale state and adapts
  the selected locale to GPUI Components; the Playground also sets browser
  document `lang`/`dir` on Wasm. Keep this coordination boundary separate from
  catalog parsing and locale formatting.
- The repository has evidence for native macOS work and a `wasm32-unknown-unknown`
  browser target, but no authoritative exhaustive supported-platform matrix was
  found. The Stage 6 decision must name the targets to support; it must not
  silently imply Linux, Windows, iOS, or Android runtime support.

Sources: `crates/rustui-i18n/Cargo.toml`,
`crates/rustui-i18n/src/{lib.rs,formatting.rs}`, `Cargo.lock`,
`crates/rustui/src/localization/`, `apps/rustui-playground/src/stories/locale_selector.rs`,
and `docs/tasks/STAGE6_FULL_LOCALE_RUNTIME_GAP_AUDIT.md`.

## Viable strategies

| Strategy | Target and API shape | Benefits | Costs and risks |
| --- | --- | --- | --- |
| **A. Rust-owned ICU4X plus Fluent runtime** | Add a cross-target locale formatting and message runtime behind `rustui-i18n`; keep `Locale` and current interpolation entry points where their contracts remain compatible. Use one explicit data provider policy across each approved target. | One implementation path for native and Wasm; results do not depend on the host OS/browser's ICU version. | Requires approved direct dependencies and versions, data packaging/coverage, and a CLDR/ICU update policy. Embedded data can affect binary size and build time; provider-loaded data adds distribution and failure policy. Exact size impact and licensing must be checked against the selected packages/data before approval. Fluent syntax and term/overlay behavior need a defined compatibility target. |
| **B. Host-native formatting adapters** | Define a Rust adapter contract and implement a native adapter for each named OS plus a browser `Intl` adapter for Wasm. Keep message syntax/runtime separately defined in `rustui-i18n`. | Can use platform facilities and avoid bundling all locale data into each application. | Results and available locale data can vary by OS/browser version. Requires per-target FFI/JS integration, error/fallback behavior, and conformance tests. It does not supply Fluent message parsing by itself. No complete target matrix or adapter contract currently exists. |
| **C. Hybrid: shared message runtime, target-selected data backend** | Use a single Fluent-compatible message runtime and API, with ICU4X for targets needing reproducible output and native/`Intl` backends only where explicitly selected. | Allows a common message/catalog contract while making data-size and host-integration choices per target. | Highest policy surface: backend differences can leak into output, so every backend needs pinned compatibility expectations and cross-backend conformance. Adds maintenance for multiple backends and does not remove the dependency/data approval gate. |

The repository does not contain measurements sufficient to rank binary-size,
startup, or runtime costs. Those need a small, approved candidate comparison;
they should not be estimated from transitive lockfile entries.

## Approval required before implementation

The owner needs to explicitly approve:

1. **Strategy:** A, B, or C (or a replacement with equivalent target parity).
2. **Initial target matrix:** at minimum decide whether Stage 6 must cover both
   native macOS and Web/Wasm, and name any additional supported targets.
3. **Data/version policy:** CLDR/ICU compatibility version, data coverage and
   delivery model (embedded, provider-loaded, or host supplied), update cadence,
   and licensing review responsibility.
4. **Message contract:** the Fluent version/features that count as supported
   (terms, select/plural, references/overlays as applicable), and whether
   `rustui-i18n` gains public number/date/time formatting APIs or keeps those
   behind a later separately scoped API decision.
5. **Dependency authorization:** for a Rust-owned or hybrid strategy, authorize
   selecting and adding the named crates/features and updating `Cargo.lock`;
   versions and feature set must be proposed for review before the dependency
   edit. For host adapters, authorize the target-specific native/JS integration
   and specify which targets may use it.

After that decision, implementation should add conformance evidence for message
syntax, cardinal and ordinal selection, number/date/time formatting, locale
negotiation, and the chosen CLDR/ICU compatibility version on every approved
target. Until then, adding hand-written locale branches would only expand the
bounded formatter and would not complete Stage 6.

## Status

Decision pending. This memo records options and repository constraints only; it
does not change dependencies, runtime behavior, roadmap summaries, or acceptance
status.
