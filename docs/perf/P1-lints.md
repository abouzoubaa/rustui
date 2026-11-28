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

**0 of 29 completed pair attempts qualified.** An owner-requested pause interrupted one additional pair. On resume, that pair restarted from its first arm. The timebox ended before the remaining G0 pairs and the four F0 workloads could finish. A dash denotes no qualified observation. The 100%-of-one-core arm limit and 15-point pair difference apply to background CPU after excluding the app, WindowServer and kernel_task.

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

The four F0 workloads (5,000-row scroll, 10,000-line typing, idle redraw with 60 panels, and forced draw with 1,000 rows) each have 0/5 qualified pairs and no CPU measurements. P1 stays draft. No new crates were added.
