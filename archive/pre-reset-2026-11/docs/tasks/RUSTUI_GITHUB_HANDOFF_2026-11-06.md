# RustUI GitHub handoff and work history — 2026-11-06

## Current progress

- **MVP evidence checklist: 67.65% (23/34).** No evidence item closed in this handoff.
- **MVP acceptance checklist: 18.18% (2/11).** No acceptance group closed.
- RustUI is **not accepted as MVP or production-ready**. Native acceptance and release gates remain open.

## Work recorded

- On 2026-11-05, a current-source macOS Playground session reused the existing
  Headstart build (1.13 seconds, no code recompilation). The executable and
  disposable app bundle matched SHA-256
  `0aa58c36d9697906392012a894e8b587563d6c104f2996e690a2a61403f4ef5c`.
- That session observed bound text updates, Tab skipping a disabled checkbox,
  radio selection, slider and stepper keyboard changes, picker owner updates,
  button pointer/keyboard activation, disabled-button action suppression, and
  visible Light/Dark switching with story state retained.
- `PYTHONPATH=. python3 scripts/tests/test_gpui_macos_anchored_popup_patch.py`
  passed **7/7 tests** during this handoff. Invoking the same script without
  `PYTHONPATH=.` first failed at import setup (`ModuleNotFoundError: No module
  named 'scripts'`); the test then passed with the repository import path set.
- A separate headless anchored-popup close test passed **1/1**. It is unit-test
  evidence only and does not count as native popup acceptance.
- The relevant native observation, build output, popup-test output, and all four
  compiler crash reports are archived beside this handoff under
  `docs/tasks/evidence/rustui-mvp-2026-11-05/`.
- The focused test's initial invocation error and corrected 7/7 result are
  recorded in `focused-popup-patch-test-2026-11-06.log` in that archive.

## Failed, invalid, or still unverified

- Four compiler ICE reports record unstable-fingerprint panics in the
  experimental Headstart compiler. They are archived verbatim. This is a build
  tool failure; it is not a passing RustUI build or a RustUI source diagnostic.
- The attempted native popup retest attached to a Playground owned by another
  worktree. That run is **invalid for this candidate**; the other process was
  preserved and must not be terminated as part of this handoff.
- A focus ring appeared to remain on a button after the button was disabled by
  automation. Ordinary keyboard reproduction is still needed before calling it
  a RustUI defect or closing the focus requirement.
- Full ordinary text editing, the remaining control/group focus policies, and
  the visual motion path were not verified in the combined controls session.
- The patch fixture test is green, but neither it nor the headless popup test
  substitutes for a fresh, source-bound native popup session.

## Next work and release blockers

1. Re-run native popup and remaining Basic controls checks against the exact
   current-source Playground bundle in an owned, available GUI slot. Verify the
   bundle identity, use ordinary keyboard input for the focus case, and record
   focused results. Do not attach by display name to a different worktree.
2. Finish the remaining evidence and acceptance items using their documented
   closure rules; keep the percentages above unchanged until an item actually
   closes.
3. Obtain trusted CI results and resolve the **five missing dependency
   notices** recorded by the release audit.
4. Record the distribution decision and supported macOS versions.
5. Keep browser, accessibility/VoiceOver, performance, licensing, CI, and final
   release acceptance as separate gates; do not infer any of them from source
   checks or this native session.

## Git history and file handling

- Before this handoff, branch `codex/production-consolidation-01a172cd` was 471
  commits ahead of `origin/codex/production-consolidation-01a172cd`; GitHub's
  branch tip matched the local tracking ref. The requested push publishes that
  existing history along with the new handoff commit.
- The local changes in `vendor/gpui-pre-macos-0.3.9/src/platform.rs` and
  `src/window.rs` are generated working copies matching approved anchored-popup
  patch v4 post-images. The versioned patch is the canonical project change;
  these materialized vendor files are not committed a second time.
- Build caches and generated app bundles remain local. Only the relevant text
  evidence and diagnostic logs listed below are archived in Git.

## Archived files

The `evidence/rustui-mvp-2026-11-05/` directory contains the source-bound
observation note, its build and popup-test logs, the focused test output, and
four unmodified Rustc ICE reports. SHA-256 checksums are recorded in
`SHA256SUMS.txt`.
