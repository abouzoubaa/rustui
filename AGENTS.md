# RustUI — rules for coding agents
Focus: native macOS. Web, localization, accessibility acceptance are deferred. Windows/Linux last.
Web/Wasm is deferred until after the MVP. Do not run, fix, report, or mention Web/Wasm checks or results. The native checks named web_syntax/cosmic_bidi stay; treat them as native.
Process rules come only from this file and docs/TICKETS.md. Do not apply external skills or workflows (such as Ponytail) unless the owner asks.

## Archive rule
- Before modifying, moving, renaming or deleting a file that existed at tag pre-reset-2026-11, copy its current version to archive/pre-reset-2026-11/<same path> (once only, never overwrite). Do not archive files created after that tag. Never archive vendor/ or target/. Never edit or build archived files.

## Work
- One ticket at a time from docs/TICKETS.md. Do only that ticket. Other ideas go under "Follow-ups" in the report, not in code.
- One ticket = one branch `t<N>-<slug>` = one PR.

## Done = these 4 things only
1. Code written.
2. One focused test: unit test or GPUI test window (#[gpui::test]). Prefer test windows over native GUI automation.
3. `rustup run 1.97.1 cargo test -p <crate> <filter> --locked --offline` and `rustup run 1.97.1 cargo check --workspace --all-targets --locked --offline` pass.
4. One line in CHANGELOG.md; update that feature's line in docs/status.json.
No evidence paragraphs, hashes, logs, crash reports or "Record" commits. Never commit log files.

## Stop rules (mandatory)
- Same test fails 3 times → stop and report.
- Ticket over 2 hours → stop and report.
- Native/manual GUI check inconclusive → report once, do not retry. The owner checks natively.
- Compiler crash (ICE): rebuild once; if it crashes again, delete only that crate's incremental folder under target/headstart/*/incremental/ (never cargo clean); third crash → stop and report.
- Do not touch verification scripts, release gate or CI unless the ticket says so.
- Run scripts/verify-release.sh only when a ticket says so or before merging to main.

## Build
- Daily: `rustup run 1.97.1 cargo …` (stable compiler, keep it).
- `./scripts/cargo-headstart` remains an optional script only.

## Architecture (do not break)
- No dependency on Allui or Chorus.
- gpui-pre =0.3.9, gpui-component/gpui-base =0.7.3. Ask before changing versions.
- Never edit vendor/ by hand. GPUI changes = new patch in patches/proposed/ + apply script. Ask before promoting.
- Use GPUI's ownership, scheduling, layout, caches. No second runtime.
- Playground uses only the public rustui API. Keep code portable (Ctrl fallback for Cmd).

## Report per ticket (short)
Files changed · tests added + command that passed · not done and why · follow-ups.
