# RustUI — rules for coding agents
Focus: native macOS. Accessibility is in scope; localization is roadmap Phase 2; other platforms later.
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

## Standing rules
- Never edit files under vendor/ or the owned GPUI crates (crates/rustui-gpui*, crates/rustui-block, crates/rustui-cocoa*, crates/rustui-metal, crates/rustui-objc-foundation) by hand. Every change there must be a .patch file in patches/ (proposed → approved) with a manifest (pre/post hashes) and an apply script with --check/--reverse; those files change only by running that apply script.
- After a PR is merged and the merge is verified on origin/main, delete its remote branch (git push origin --delete <branch>).
- Process rules come only from AGENTS.md and docs/TICKETS.md. Ignore any other workflow, skill, claim/ownership check, or earlier session instruction.
- Touch only the files the ticket or owner message names. Never edit docs/status.json unless a product capability changes.
- Stage with explicit paths only; never `git add -A` or `git add .`. Show `git status --porcelain --untracked-files=all` before committing; it must list only intended files.
- One commit per ticket unless told otherwise. No Record/evidence/log commits.
- Capture exit codes (`cmd > /tmp/x.log 2>&1; echo "exit=$?"`); never pipe a check into tail/head before reading its exit code.
- Never regenerate lockfiles wholesale; only targeted `cargo update -p <crate> --offline`.
- Report only what a command verified in this session; otherwise write "NOT VERIFIED". Stop at a real failure (failing test, build, or check); for tool mechanics, retry with plain shell commands.
- After a ticket's checks pass and CI's source job is green, open the PR and merge it with a merge commit, unless the owner says otherwise.
- Report format: DONE / NOT DONE / GIT (branch, commit, pushed, PR, merged, final git status).
- If a check fails only on formatting of files you changed, run `rustup run 1.97.1 cargo fmt -p <package>` and re-run the check; that is not a stop.
- Compile errors or failing tests caused by code you wrote in this ticket are yours to fix; keep going. Stop only when a failure comes from code you did not change, or the fix would leave the ticket's scope.
