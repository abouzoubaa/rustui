# P1 — native workspace lints

Base: `p-integration` at `263dc318eb9efa482e3ced1ef2928bacfa869ada`. The seven owned workspace packages inherit one explicit Clippy policy. GPUI/Kit sources and their patch stacks are unchanged.

## Baseline and result

Counted before source or policy edits, using pinned Rust/Clippy 1.98.0, the shared build cache and native workspace/all-targets selection. The initial invocation resolved Homebrew Clippy 1.97.1 and was rejected by the minimum Rust version; it produced no valid baseline and is excluded. The valid run resolved Cargo subcommands beside the pinned compiler, as the check script now does.

A unique diagnostic is identified by package, lint code, message and primary source span; repeated library/test-target emissions are deduplicated. The valid baseline emitted 296 workspace diagnostics, representing 195 unique diagnostics: 123 Clippy and 72 rustc. The final direct run with `-D warnings` emitted zero workspace diagnostics and exited 0. Dependency diagnostics are outside the seven-package workspace count; their existing source ownership and lint policies remain unchanged.

| Diagnostic group | Unique before | After |
|---|---:|---:|
| Clippy correctness | 0 | 0 |
| Clippy suspicious | 1 | 0 |
| Clippy performance | 2 | 0 |
| Clippy complexity | 43 | 0 |
| Clippy style | 77 | 0 |
| rustc warnings | 72 | 0 |
| **Total** | **195** | **0** |

## Policy and changes

`[workspace.lints.clippy]` lists correctness as deny, and suspicious, perf, complexity and style as warn, with group priority -1. Each workspace manifest explicitly inherits this policy. `scripts/check.sh` runs `cargo clippy --workspace --all-targets --locked --offline -- -D warnings`; CI installs the pinned Clippy component and invokes that same script. The script pins external Cargo subcommands through the compiler's bin directory, avoiding a different Homebrew Clippy.

The two performance diagnostics were handled first: menu entries share one immutable resolved-style value per binding, and the fixed tab list uses an array. Other changes use callback/cache aliases, equivalent current Fast scroll names, mechanical control-flow simplifications, direct test initializers, and removal or correct build guards for unused bookkeeping/support code. Test modules move after production items without dropping their tests.

Exceptions are local and have reasons. Parser fixtures retain deliberately nonminimal expressions; callbacks keep exact String function-pointer contracts; the public borrowed View lowering method and existing renderer constructors keep their signatures. Native support data retains required model shapes while optional consumers are inactive. There is no workspace-wide allowance for these diagnostics.

A focused generic-payload test checks that deriving AsyncState's unit default does not require its payload types to implement Default. The existing conditional-default macro fixture and both optional-child parser fixtures also pass.

## Validation

- `scripts/check.sh`: exit 0, including pinned Clippy with `-D warnings`; all seven library groups total 1,875 tests: 1,868 passed and 7 intentionally ignored (RustUI 1,456 + 7 ignored; core 11; i18n 86; macros 3; Lab 40; Playground 272; RGPE 0).
- `./scripts/verify-release.sh source`: exit 0; 695 tests passed.
- Active damage stack `--check`: exact postimages pass, 76 ordered patches plus one alternative, 95 files.
- Focused generic-default test: 1 passed; conditional-default macro test: 1 passed; optional-child parser integration tests: 2 passed. Menu/overlay test-window selection: 115 passed.
- The PR must pass the source CI job and review before merging into `p-integration`.

This ticket makes no measured runtime-performance verdict and adds no latency or damage behavior feature.

## Stage P v4 supplemental lint gate

The baseline is the G3 merge at `46bf8878c4648dab7ad9a434154b69afce35ae15`. The supplemental gate runs the pinned Clippy 1.98.0 with `--lib --bins` on the five RustUI-owned crates. It promotes the performance group and twelve named lints to errors. Tests continue through the existing all-targets Clippy command and library test groups in `scripts/check.sh`.

| Distinct production findings | G3 baseline | Amended P1 |
|---|---:|---:|
| Redundant clone or string copy | 43 | 0 |
| Small copyable value passed by reference | 3 | 0 |
| Eager fallback expression | 1 | 0 |
| **Total** | **47** | **0** |

The baseline counts unique primary source spans from the library targets in the saved pre-change Clippy inventory. The five selected packages have no production binary targets. The resumed deny-level command exited 0. The earlier inventory included 21 test-only findings. Those findings stay outside the supplemental gate. No owned crate contained a `.context(format!(...))` call.

### Test edit

One file contains test changes, `crates/rustui/src/components/data/collection.rs`. The Collection keyboard filter now takes its five-byte `Modifiers` value by value. The following is the complete diff inside its test module.

```diff
@@ -1566,4 +1565,4 @@ mod tests {
-        assert!(keyboard_event_eligible(true, false, false, &plain));
-        assert!(!keyboard_event_eligible(false, false, false, &plain));
-        assert!(!keyboard_event_eligible(true, true, false, &plain));
-        assert!(!keyboard_event_eligible(true, false, true, &plain));
+        assert!(keyboard_event_eligible(true, false, false, plain));
+        assert!(!keyboard_event_eligible(false, false, false, plain));
+        assert!(!keyboard_event_eligible(true, true, false, plain));
+        assert!(!keyboard_event_eligible(true, false, true, plain));
@@ -1586 +1585 @@ mod tests {
-            assert!(!keyboard_event_eligible(true, false, false, &modifiers));
+            assert!(!keyboard_event_eligible(true, false, false, modifiers));
```

### Verification and measurement

The amended production lint gate passed. The seven library groups in `scripts/check.sh` total 1,875 passed, 7 ignored and 0 failed. All nine native Metal proofs passed.

The original source check rejected Kit candidates inherited from the G3 merge. With owner approval, the source job now uses the same exact candidate-chain verification for both integration branches and PRs targeting them. The release verifier and the approved-only source checks for main are unchanged.

### P1 paired captures

Source CI passed on [the measured code revision](https://github.com/abouzoubaa/rustui/actions/runs/46035864999/job/140630981721). The release comparison used Rust 1.98.0, 60-second G0 windows after warmup, damage off and buffered logging. Each arm ran directly from its frozen executable with the same logging flags. Before was the G3 merge and after was the amended P1 code. The executable SHA-256 values were `06a796fca7e43435649b51a9a5bdc98e1b68f87da17ba5bb3b3288d04841592a` and `797a68d0e1e0fd46485609620496ac31b413ab608fddd45470b3ce92ed9f6e5f`, respectively.

**0 of 29 completed pair attempts qualified.** An owner-requested pause interrupted one additional pair. On resume, that pair restarted from its first arm. The original timebox ended before the remaining G0 pairs and the four F0 workloads could finish. A dash denotes no qualified observation. The 100%-of-one-core arm limit and 15-point pair difference apply to background CPU after excluding the app, WindowServer and kernel_task.

| G0 scene | Qualified | Metric per attempted present | Median before → after | Paired changes 1–5 (%) | Median change | Min / max | Verdict |
|---|---:|---|---:|---|---:|---:|---|
| idle | 0/5 | Draw + present (ms) | — | —, —, —, —, — | — | — | INCONCLUSIVE |
| idle | 0/5 | Process user + system (s) | — | —, —, —, —, — | — | — | INCONCLUSIVE |
| typing | 0/5 | Draw + present (ms) | — | —, —, —, —, — | — | — | INCONCLUSIVE |
| typing | 0/5 | Process user + system (s) | — | —, —, —, —, — | — | — | INCONCLUSIVE |
| hover | 0/5 | Draw + present (ms) | — | —, —, —, —, — | — | — | INCONCLUSIVE |
| hover | 0/5 | Process user + system (s) | — | —, —, —, —, — | — | — | INCONCLUSIVE |
| log | 0/5 | Draw + present (ms) | — | —, —, —, —, — | — | — | INCONCLUSIVE |
| log | 0/5 | Process user + system (s) | — | —, —, —, —, — | — | — | INCONCLUSIVE |
| drag | 0/5 | Draw + present (ms) | — | —, —, —, —, — | — | — | INCONCLUSIVE |
| drag | 0/5 | Process user + system (s) | — | —, —, —, —, — | — | — | INCONCLUSIVE |

CPU load in percent of one core, before / after. An empty CPU speed limit means pmset reported no limit.

| Scene | Pair / attempt | CPU speed limit | Background | WindowServer | kernel_task | Qualified |
|---|---|---|---:|---:|---:|---|
| idle | 1 / 1 | — | 118.91 / 104.06 | 49.73 / 51.36 | 16.58 / 16.57 | no |
| idle | 1 / 2 | — | 101.92 / 93.55 | 51.35 / 51.39 | 16.57 / 14.92 | no |
| idle | 1 / 3 | — | 101.75 / 138.05 | 51.66 / 51.36 | 16.66 / 18.23 | no |
| idle | 1 / 4 | — | 154.76 / 169.43 | 51.38 / 51.35 | 18.23 / 18.22 | no |
| idle | 2 / 1 | — | 124.86 / 167.15 | 51.35 / 51.64 | 18.22 / 18.33 | no |
| idle | 2 / 2 | — | 180.93 / 143.51 | 51.36 / 51.42 | 28.17 / 21.56 | no |
| idle | 2 / 3 | — | 165.21 / 197.94 | 51.36 / 49.92 | 21.54 / 26.63 | no |
| idle | 2 / 4 | — | 158.30 / 148.63 | 51.65 / 49.98 | 21.66 / 21.66 | no |
| idle | 3 / 1 | — | 178.58 / 164.86 | 51.64 / 51.40 | 23.32 / 21.56 | no |
| idle | 3 / 2 | — | 153.45 / 160.95 | 49.98 / 49.71 | 21.66 / 21.54 | no |
| idle | 3 / 3 | — | 159.38 / 180.95 | 49.71 / 49.98 | 19.88 / 21.66 | no |
| idle | 3 / 4 | — | 159.71 / 153.72 | 49.98 / 49.69 | 21.66 / 21.53 | no |
| idle | 4 / 1 | — | 170.95 / 160.94 | 49.71 / 51.36 | 21.54 / 21.54 | no |
| idle | 4 / 2 | — | 156.77 / 186.36 | 49.70 / 49.69 | 21.54 / 23.19 | no |
| idle | 4 / 3 | — | 177.53 / 159.12 | 49.70 / 49.70 | 21.54 / 21.54 | no |
| idle | 4 / 4 | — | 162.90 / 163.67 | 49.99 / 49.70 | 21.66 / 21.54 | no |
| idle | 5 / 1 | — | 166.56 / 158.79 | 50.00 / 49.72 | 20.00 / 19.89 | no |
| idle | 5 / 2 | — | 223.90 / 318.56 | 48.06 / 46.66 | 21.54 / 20.00 | no |
| typing | 1 / 1 | — | 364.50 / 346.74 | 48.24 / 49.89 | 18.30 / 18.29 | no |
| typing | 2 / 1 | — | 200.11 / 199.42 | 46.64 / 48.29 | 19.99 / 18.32 | no |
| typing | 3 / 1 | — | 206.93 / 158.80 | 46.57 / 48.25 | 18.29 / 19.97 | no |
| hover | 1 / 1 | — | 348.10 / 242.21 | 53.33 / 53.33 | 21.67 / 23.33 | no |
| hover | 2 / 1 | — | 189.72 / 249.92 | 51.66 / 51.66 | 23.33 / 23.33 | no |
| hover | 3 / 1 | — | 127.25 / 93.55 | 50.00 / 51.66 | 21.67 / 20.00 | no |
| log | 1 / 1 | — | 236.50 / 204.31 | 49.98 / 49.98 | 24.99 / 21.66 | no |
| log | 2 / 1 | — | 202.93 / 213.99 | 48.30 / 48.30 | 23.32 / 21.65 | no |
| log | 3 / 1 | — | 86.64 / 179.60 | 49.99 / 48.30 | 16.66 / 21.65 | no |
| drag | 1 / 1 | — | 188.92 / 193.94 | 51.66 / 51.66 | 23.33 / 23.33 | no |
| drag | 2 / 1 | — | 182.61 / 196.46 | 51.66 / 51.66 | 25.00 / 23.33 | no |

The four F0 workloads (5,000-row scroll, 10,000-line typing, idle redraw with 60 panels, and forced draw with 1,000 rows) each have 0/5 qualified pairs and no CPU measurements. No new crates were added.

### Amended P1 measurement

The November 28 run used a 150% per-arm load ceiling and a maximum 15-point difference. CPU load is expressed as a percentage of one core. Earlier captures under the 100% ceiling are reported separately. All captures used release builds with damage off. Pairs alternated before/after order. The RustUI Lab recorder collected 60 seconds after warmup and flushed its frame log on exit. The before and after Lab SHA-256 hashes are recorded above.

The four F0 comparisons used separate measurement binaries built from the original headless workloads. Each measured 300 list scrolls, 100 editor inputs, 60 idle draws, or 100 forced draws at 1,000 rows. The first CPU metric is dispatch/update plus draw duration for F0, and draw plus present duration for G0. The second is process user plus system CPU sampled around the timed workload. F0 background load covers each complete test invocation, including setup. No existing test changed for instrumentation.

Top background processes over 10.12 seconds before pair 1.

| Process | PID | CPU (% of one core) |
|---|---:|---:|
| signpost_reporter | 39522 | 70.17 |
| Google Chrome Helper (Aperitif) | 40729 | 36.17 |
| Codex (Aperitif Renderer) | 63259 | 20.06 |
| Codex (Aperitif) | 63244 | 17.89 |
| Google Chrome Helper (Aperitif Renderer) | 69847 | 7.21 |

| Scene | Metric per attempt | Qualified | Median before → after | Paired changes 1–5 (%) | Median (%) | Min / max (%) | Verdict |
|---|---|---:|---:|---|---:|---:|---|
| typing | Frame CPU (ms) | 1/5 | 1.34107 → 1.34914 | —, —, +0.60, —, — | +0.60 | +0.60 / +0.60 | INCONCLUSIVE |
| typing | Process CPU (s) | 1/5 | 0.01961744 → 0.01959223 | —, —, -0.13, —, — | -0.13 | -0.13 / -0.13 | INCONCLUSIVE |
| hover | Frame CPU (ms) | 0/5 | — | —, —, —, —, — | — | — | INCONCLUSIVE |
| hover | Process CPU (s) | 0/5 | — | —, —, —, —, — | — | — | INCONCLUSIVE |
| log | Frame CPU (ms) | 0/5 | — | —, —, —, —, — | — | — | INCONCLUSIVE |
| log | Process CPU (s) | 0/5 | — | —, —, —, —, — | — | — | INCONCLUSIVE |
| drag | Frame CPU (ms) | 0/5 | — | —, —, —, —, — | — | — | INCONCLUSIVE |
| drag | Process CPU (s) | 0/5 | — | —, —, —, —, — | — | — | INCONCLUSIVE |
| idle | Frame CPU (ms) | 0/5 | — | —, —, —, —, — | — | — | INCONCLUSIVE |
| idle | Process CPU (s) | 0/5 | — | —, —, —, —, — | — | — | INCONCLUSIVE |
| f0-scroll | Frame CPU (ms) | 2/5 | 1.27064 → 1.25221 | -2.96, —, —, +0.10, — | -1.43 | -2.96 / +0.10 | INCONCLUSIVE |
| f0-scroll | Process CPU (s) | 2/5 | 0.00126313 → 0.00124792 | -2.42, —, —, +0.04, — | -1.19 | -2.42 / +0.04 | INCONCLUSIVE |
| f0-typing | Frame CPU (ms) | 2/5 | 0.58885 → 0.55930 | —, -3.13, —, —, -6.82 | -4.97 | -6.82 / -3.13 | INCONCLUSIVE |
| f0-typing | Process CPU (s) | 2/5 | 0.00058760 → 0.00055823 | —, -3.09, —, —, -6.82 | -4.96 | -6.82 / -3.09 | INCONCLUSIVE |
| f0-idle | Frame CPU (ms) | 0/5 | — | —, —, —, —, — | — | — | INCONCLUSIVE |
| f0-idle | Process CPU (s) | 0/5 | — | —, —, —, —, — | — | — | INCONCLUSIVE |
| f0-forced | Frame CPU (ms) | 2/5 | 5.23150 → 5.16177 | -1.10, -1.57, —, —, — | -1.33 | -1.57 / -1.10 | INCONCLUSIVE |
| f0-forced | Process CPU (s) | 2/5 | 0.00521646 → 0.00515020 | -1.03, -1.51, —, —, — | -1.27 | -1.51 / -1.03 | INCONCLUSIVE |

<details><summary>Thermal and per-arm CPU qualification for each attempt</summary>

| Scene | Pair / try | CPU speed limit | Background before / after | WindowServer before / after | kernel_task before / after | Qualified |
|---|---|---|---:|---:|---:|---|
| typing | 1 / 1 | No limit reported | 176.10 / 154.05 | 48.29 / 49.95 | 18.32 / 16.65 | no |
| typing | 1 / 2 | No limit reported | 154.37 / 155.54 | 48.26 / 49.93 | 16.64 / 16.64 | no |
| typing | 1 / 3 | No limit reported | 157.25 / 168.38 | 48.28 / 48.30 | 16.65 / 16.66 | no |
| typing | 2 / 1 | No limit reported | 156.04 / 157.14 | 48.28 / 48.30 | 18.31 / 16.66 | no |
| typing | 2 / 2 | No limit reported | 157.59 / 154.90 | 48.30 / 48.23 | 18.32 / 18.29 | no |
| typing | 3 / 1 | No limit reported | 139.19 / 126.56 | 48.29 / 48.27 | 16.65 / 18.31 | yes |
| typing | 4 / 1 | No limit reported | 168.57 / 157.57 | 48.26 / 48.27 | 16.64 / 18.31 | no |
| typing | 4 / 2 | No limit reported | 159.80 / 156.98 | 46.63 / 48.24 | 18.32 / 18.30 | no |
| typing | 5 / 1 | No limit reported | 185.47 / 151.46 | 48.30 / 48.28 | 18.32 / 18.31 | no |
| typing | 5 / 2 | No limit reported | 170.52 / 156.70 | 48.22 / 48.23 | 18.29 / 18.29 | no |
| hover | 1 / 1 | No limit reported | 174.40 / 151.23 | 53.32 / 51.67 | 21.66 / 21.67 | no |
| hover | 1 / 2 | No limit reported | 154.50 / 160.49 | 51.67 / 51.66 | 21.67 / 20.00 | no |
| hover | 2 / 1 | No limit reported | 153.49 / 148.66 | 51.66 / 51.66 | 21.66 / 21.67 | no |
| hover | 2 / 2 | No limit reported | 153.48 / 167.14 | 53.33 / 51.66 | 21.67 / 21.66 | no |
| hover | 3 / 1 | No limit reported | 165.75 / 155.32 | 51.66 / 53.33 | 21.66 / 20.00 | no |
| hover | 3 / 2 | No limit reported | 157.29 / 153.97 | 51.66 / 51.66 | 21.66 / 21.66 | no |
| hover | 4 / 1 | No limit reported | 155.53 / 159.31 | 53.33 / 51.66 | 21.67 / 21.66 | no |
| hover | 4 / 2 | No limit reported | 157.45 / 156.67 | 51.66 / 53.32 | 21.66 / 21.66 | no |
| hover | 5 / 1 | No limit reported | 164.21 / 156.53 | 53.33 / 51.66 | 20.00 / 21.66 | no |
| hover | 5 / 2 | No limit reported | 157.74 / 163.26 | 53.33 / 51.66 | 21.67 / 21.66 | no |
| log | 1 / 1 | No limit reported | 176.90 / 157.87 | 49.96 / 49.95 | 19.99 / 19.98 | no |
| log | 1 / 2 | No limit reported | 161.65 / 165.65 | 49.97 / 49.97 | 19.99 / 19.99 | no |
| log | 2 / 1 | No limit reported | 157.89 / 164.58 | 49.98 / 48.29 | 19.99 / 19.98 | no |
| log | 2 / 2 | No limit reported | 160.22 / 167.22 | 49.96 / 48.31 | 19.99 / 19.99 | no |
| log | 3 / 1 | No limit reported | 157.35 / 155.37 | 49.97 / 49.99 | 19.99 / 20.00 | no |
| log | 3 / 2 | No limit reported | 169.73 / 162.85 | 49.96 / 49.96 | 19.98 / 19.98 | no |
| log | 4 / 1 | No limit reported | 161.41 / 156.58 | 48.31 / 48.31 | 19.99 / 19.99 | no |
| log | 4 / 2 | No limit reported | 160.38 / 181.54 | 49.97 / 49.95 | 19.99 / 18.32 | no |
| log | 5 / 1 | No limit reported | 174.41 / 171.13 | 49.96 / 49.96 | 19.98 / 18.32 | no |
| log | 5 / 2 | No limit reported | 161.05 / 157.81 | 49.97 / 49.98 | 18.32 / 19.99 | no |
| drag | 1 / 1 | No limit reported | 150.71 / 104.03 | 53.33 / 51.67 | 23.33 / 20.00 | no |
| drag | 1 / 2 | No limit reported | 151.02 / 179.76 | 51.66 / 51.66 | 21.67 / 23.33 | no |
| drag | 2 / 1 | No limit reported | 155.65 / 186.00 | 53.33 / 51.67 | 21.67 / 25.00 | no |
| drag | 2 / 2 | No limit reported | 150.46 / 174.14 | 53.33 / 51.66 | 21.67 / 20.00 | no |
| drag | 3 / 1 | No limit reported | 170.93 / 154.79 | 53.33 / 53.33 | 20.00 / 21.66 | no |
| drag | 3 / 2 | No limit reported | 150.80 / 150.02 | 51.66 / 51.67 | 21.67 / 20.00 | no |
| drag | 4 / 1 | No limit reported | 151.96 / 153.20 | 53.33 / 53.33 | 21.67 / 20.00 | no |
| drag | 4 / 2 | No limit reported | 151.97 / 153.02 | 53.33 / 51.66 | 21.67 / 20.00 | no |
| drag | 5 / 1 | No limit reported | 153.67 / 153.89 | 51.66 / 51.66 | 23.33 / 21.66 | no |
| drag | 5 / 2 | No limit reported | 179.80 / 153.90 | 53.33 / 51.67 | 21.67 / 21.67 | no |
| idle | 1 / 1 | No limit reported | 178.54 / 164.15 | 49.73 / 49.73 | 18.23 / 18.24 | no |
| idle | 1 / 2 | No limit reported | 160.74 / 160.55 | 48.06 / 48.07 | 19.89 / 19.89 | no |
| idle | 2 / 1 | No limit reported | 152.64 / 158.57 | 49.71 / 49.71 | 19.89 / 18.23 | no |
| idle | 2 / 2 | No limit reported | 158.59 / 187.63 | 49.69 / 48.06 | 19.88 / 21.55 | no |
| idle | 3 / 1 | No limit reported | 153.58 / 163.62 | 49.72 / 48.05 | 19.89 / 18.23 | no |
| idle | 3 / 2 | No limit reported | 161.13 / 159.11 | 49.71 / 49.99 | 18.23 / 18.33 | no |
| idle | 4 / 1 | No limit reported | 162.87 / 160.04 | 48.04 / 48.06 | 18.22 / 18.23 | no |
| idle | 4 / 2 | No limit reported | 162.53 / 176.92 | 48.33 / 48.06 | 18.33 / 18.23 | no |
| idle | 5 / 1 | No limit reported | 174.63 / 160.71 | 48.04 / 48.31 | 18.22 / 18.33 | no |
| idle | 5 / 2 | No limit reported | 162.12 / 161.14 | 48.04 / 48.33 | 18.22 / 18.33 | no |
| f0-scroll | 1 / 1 | No limit reported | 269.72 / 0.00 | 0.00 / 251.09 | 0.00 / 251.09 | no |
| f0-scroll | 1 / 2 | No limit reported | 177.53 / 179.89 | 0.00 / 0.00 | 0.00 / 0.00 | no |
| f0-scroll | 1 / 3 | No limit reported | 0.00 / 175.91 | 251.13 / 0.00 | 0.00 / 0.00 | no |
| f0-scroll | 1 / 4 | No limit reported | 0.00 / 0.00 | 245.08 / 0.00 | 0.00 / 252.07 | yes |
| f0-scroll | 2 / 1 | No limit reported | 0.00 / 172.65 | 254.70 / 0.00 | 0.00 / 0.00 | no |
| f0-scroll | 2 / 2 | No limit reported | 178.95 / 179.11 | 0.00 / 0.00 | 0.00 / 0.00 | no |
| f0-scroll | 2 / 3 | No limit reported | 187.87 / 0.00 | 0.00 / 254.02 | 0.00 / 0.00 | no |
| f0-scroll | 2 / 4 | No limit reported | 180.04 / 0.00 | 0.00 / 249.87 | 0.00 / 0.00 | no |
| f0-scroll | 3 / 1 | No limit reported | 181.36 / 0.00 | 0.00 / 252.79 | 0.00 / 252.79 | no |
| f0-scroll | 3 / 2 | No limit reported | 0.00 / 174.37 | 0.00 / 0.00 | 251.99 / 0.00 | no |
| f0-scroll | 3 / 3 | No limit reported | 0.00 / 186.01 | 253.76 / 0.00 | 0.00 / 0.00 | no |
| f0-scroll | 3 / 4 | No limit reported | 0.00 / 214.62 | 251.40 / 0.00 | 0.00 / 0.00 | no |
| f0-scroll | 4 / 1 | No limit reported | 0.00 / 194.10 | 255.63 / 0.00 | 0.00 / 0.00 | no |
| f0-scroll | 4 / 2 | No limit reported | 232.51 / 179.12 | 0.00 / 0.00 | 0.00 / 0.00 | no |
| f0-scroll | 4 / 3 | No limit reported | 0.00 / 0.00 | 0.00 / 250.69 | 250.78 / 0.00 | yes |
| f0-scroll | 5 / 1 | No limit reported | 176.55 / 0.00 | 0.00 / 253.83 | 0.00 / 0.00 | no |
| f0-scroll | 5 / 2 | No limit reported | 0.00 / 173.72 | 0.00 / 0.00 | 251.59 / 0.00 | no |
| f0-scroll | 5 / 3 | No limit reported | 0.00 / 278.47 | 250.64 / 0.00 | 0.00 / 0.00 | no |
| f0-scroll | 5 / 4 | No limit reported | 197.86 / 179.58 | 0.00 / 0.00 | 0.00 / 0.00 | no |
| f0-typing | 1 / 1 | No limit reported | 177.29 / 188.78 | 0.00 / 0.00 | 0.00 / 0.00 | no |
| f0-typing | 1 / 2 | No limit reported | 177.24 / 187.94 | 0.00 / 0.00 | 0.00 / 0.00 | no |
| f0-typing | 1 / 3 | No limit reported | 233.55 / 191.87 | 0.00 / 0.00 | 0.00 / 0.00 | no |
| f0-typing | 1 / 4 | No limit reported | 176.78 / 181.95 | 0.00 / 0.00 | 0.00 / 0.00 | no |
| f0-typing | 2 / 1 | No limit reported | 184.00 / 0.00 | 0.00 / 0.00 | 0.00 / 689.14 | no |
| f0-typing | 2 / 2 | No limit reported | 233.54 / 184.99 | 0.00 / 0.00 | 0.00 / 0.00 | no |
| f0-typing | 2 / 3 | No limit reported | 0.00 / 0.00 | 0.00 / 706.51 | 685.16 / 0.00 | yes |
| f0-typing | 3 / 1 | No limit reported | 0.00 / 169.68 | 683.04 / 0.00 | 0.00 / 0.00 | no |
| f0-typing | 3 / 2 | No limit reported | 217.07 / 0.00 | 0.00 / 680.52 | 0.00 / 0.00 | no |
| f0-typing | 3 / 3 | No limit reported | 0.00 / 185.94 | 698.60 / 0.00 | 0.00 / 0.00 | no |
| f0-typing | 3 / 4 | No limit reported | 177.96 / 196.76 | 0.00 / 0.00 | 0.00 / 0.00 | no |
| f0-typing | 4 / 1 | No limit reported | 158.97 / 179.17 | 0.00 / 0.00 | 0.00 / 0.00 | no |
| f0-typing | 4 / 2 | No limit reported | 0.00 / 182.77 | 691.82 / 0.00 | 0.00 / 0.00 | no |
| f0-typing | 4 / 3 | No limit reported | 0.00 / 194.75 | 677.30 / 0.00 | 0.00 / 0.00 | no |
| f0-typing | 4 / 4 | No limit reported | 203.03 / 0.00 | 0.00 / 668.93 | 0.00 / 0.00 | no |
| f0-typing | 5 / 1 | No limit reported | 146.69 / 0.00 | 0.00 / 687.95 | 0.00 / 0.00 | no |
| f0-typing | 5 / 2 | No limit reported | 0.00 / 177.95 | 0.00 / 0.00 | 697.25 / 0.00 | no |
| f0-typing | 5 / 3 | No limit reported | 0.00 / 0.00 | 0.00 / 702.08 | 688.64 / 0.00 | yes |
| f0-idle | 1 / 1 | No limit reported | 0.00 / 163.64 | 5223.35 / 0.00 | 0.00 / 0.00 | no |
| f0-idle | 1 / 2 | No limit reported | 0.00 / 300.41 | 5129.17 / 0.00 | 0.00 / 0.00 | no |
| f0-idle | 1 / 3 | No limit reported | 163.37 / 0.00 | 0.00 / 5046.98 | 0.00 / 0.00 | no |
| f0-idle | 1 / 4 | No limit reported | 66.83 / 0.00 | 0.00 / 4964.36 | 0.00 / 0.00 | no |
| f0-idle | 2 / 1 | No limit reported | 300.24 / 150.26 | 0.00 / 0.00 | 0.00 / 0.00 | no |
| f0-idle | 2 / 2 | No limit reported | 122.46 / 268.70 | 0.00 / 0.00 | 0.00 / 0.00 | no |
| f0-idle | 2 / 3 | No limit reported | 0.00 / 233.68 | 0.00 / 0.00 | 5057.86 / 0.00 | no |
| f0-idle | 2 / 4 | No limit reported | 300.28 / 163.31 | 0.00 / 0.00 | 0.00 / 0.00 | no |
| f0-idle | 3 / 1 | No limit reported | 163.26 / 0.00 | 0.00 / 5176.70 | 0.00 / 0.00 | no |
| f0-idle | 3 / 2 | No limit reported | 233.54 / 248.01 | 0.00 / 0.00 | 0.00 / 0.00 | no |
| f0-idle | 3 / 3 | No limit reported | 194.40 / 122.47 | 0.00 / 0.00 | 0.00 / 0.00 | no |
| f0-idle | 3 / 4 | No limit reported | 186.09 / 321.88 | 0.00 / 0.00 | 0.00 / 0.00 | no |
| f0-idle | 4 / 1 | No limit reported | 122.44 / 172.94 | 0.00 / 0.00 | 0.00 / 0.00 | no |
| f0-idle | 4 / 2 | No limit reported | 216.05 / 0.00 | 0.00 / 5090.85 | 0.00 / 0.00 | no |
| f0-idle | 4 / 3 | No limit reported | 200.29 / 0.00 | 0.00 / 5090.33 | 0.00 / 0.00 | no |
| f0-idle | 4 / 4 | No limit reported | 233.76 / 0.00 | 0.00 / 5154.85 | 0.00 / 0.00 | no |
| f0-idle | 5 / 1 | No limit reported | 0.00 / 194.28 | 0.00 / 0.00 | 5160.55 / 0.00 | no |
| f0-idle | 5 / 2 | No limit reported | 0.00 / 122.47 | 0.00 / 0.00 | 5107.58 / 0.00 | no |
| f0-idle | 5 / 3 | No limit reported | 100.35 / 0.00 | 0.00 / 0.00 | 0.00 / 5155.65 | no |
| f0-idle | 5 / 4 | No limit reported | 138.24 / 233.71 | 0.00 / 0.00 | 0.00 / 0.00 | no |
| f0-forced | 1 / 1 | No limit reported | 153.71 / 182.88 | 0.00 / 0.00 | 90.27 / 0.00 | no |
| f0-forced | 1 / 2 | No limit reported | 96.28 / 182.05 | 94.96 / 0.00 | 0.00 / 0.00 | no |
| f0-forced | 1 / 3 | No limit reported | 91.99 / 81.91 | 95.93 / 96.68 | 0.00 / 0.00 | yes |
| f0-forced | 2 / 1 | No limit reported | 0.00 / 91.47 | 96.42 / 96.42 | 96.42 / 0.00 | no |
| f0-forced | 2 / 2 | No limit reported | 94.33 / 93.88 | 96.47 / 0.00 | 0.00 / 97.21 | yes |
| f0-forced | 3 / 1 | No limit reported | 192.55 / 263.35 | 0.00 / 0.00 | 0.00 / 0.00 | no |
| f0-forced | 3 / 2 | No limit reported | 87.57 / 129.13 | 95.77 / 0.00 | 0.00 / 94.66 | no |
| f0-forced | 3 / 3 | No limit reported | 92.20 / 181.62 | 0.00 / 0.00 | 96.12 / 0.00 | no |
| f0-forced | 3 / 4 | No limit reported | 0.87 / 182.07 | 96.27 / 0.00 | 96.27 / 0.00 | no |
| f0-forced | 4 / 1 | No limit reported | 87.16 / 0.00 | 96.72 / 97.50 | 0.00 / 97.50 | no |
| f0-forced | 4 / 2 | No limit reported | 91.37 / 184.70 | 97.63 / 0.00 | 0.00 / 0.00 | no |
| f0-forced | 4 / 3 | No limit reported | 95.58 / 117.33 | 0.00 / 96.11 | 95.17 / 0.00 | no |
| f0-forced | 4 / 4 | No limit reported | 3.41 / 192.71 | 96.20 / 0.00 | 96.20 / 0.00 | no |
| f0-forced | 5 / 1 | No limit reported | 199.14 / 87.07 | 0.00 / 0.00 | 0.00 / 96.12 | no |
| f0-forced | 5 / 2 | No limit reported | 185.81 / 185.12 | 0.00 / 0.00 | 0.00 / 0.00 | no |
| f0-forced | 5 / 3 | No limit reported | 187.08 / 95.69 | 0.00 / 95.48 | 0.00 / 0.00 | no |
| f0-forced | 5 / 4 | No limit reported | 121.16 / 208.81 | 95.01 / 0.00 | 0.00 / 0.00 | no |

</details>

**P1 qualified scenes = 0/9.** Seven of 124 pair attempts qualified individually. No scene reached four qualified pairs. All nine process CPU verdicts are INCONCLUSIVE. Under Amendment 6 this satisfies the performance condition for the behavior-preserving P1 lint ticket.
