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
