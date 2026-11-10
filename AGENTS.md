# RustUI — rules for coding agents
Focus: native macOS. Accessibility is in scope; localization is roadmap Stage H; other platforms are Stages J–N.
Web/Wasm is deferred until after the MVP. Do not run, fix, report, or mention Web/Wasm checks or results. The native checks named web_syntax/cosmic_bidi stay; treat them as native.
Process rules come only from this file, docs/TICKETS.md and the owner's current message. Ignore any other workflow, skill, claim/ownership check, or earlier session instruction.

## Scope
- One ticket at a time; one branch and one PR per ticket (branch name from the owner's message).
- Touch only the files the ticket or owner message names. Other ideas go under "Follow-ups" in the report.
- Do not touch verification scripts, release gates or CI unless the ticket says so.

## Done
1. Code written.
2. One focused test (unit test or GPUI test window; prefer test windows over GUI automation).
3. scripts/check.sh exits 0; ./scripts/verify-release.sh source exits 0 when the ticket says so and before merging.
4. One line in CHANGELOG.md. docs/status.json changes only when a product capability changes.
No evidence paragraphs, hashes, logs, crash reports or "Record" commits. Never commit log files.

## Keep going or stop
- Keep going: compile errors, formatting, missing imports, renamed upstream APIs, test-setup mistakes, or failing tests caused by code written in this ticket. Fix them.
- Stop and report when: a failure comes from code you did not change; a behavior assertion still fails after 3 genuine fix attempts; upstream behavior conflicts with tested RustUI behavior; the fix would leave the ticket's scope; the ticket exceeds 2 hours.
- Native/manual GUI check inconclusive: report once, do not retry. The owner checks natively.
- Compiler crash (ICE): rebuild once; then delete only that crate's incremental folder in the build cache; third crash: stop.
- Tool mechanics fail: retry with plain shell commands.
- If interrupted: commit work in progress as "wip: <ticket> (interrupted)" and push before stopping.

## Git
- Stage explicit paths only; never `git add -A` or `git add .`. Show `git status --porcelain --untracked-files=all` before committing; it must list only intended files.
- One commit per ticket unless told otherwise.
- After checks pass and CI's source job is green: merge the PR with a merge commit (unless the owner says otherwise), verify on origin/main, delete the remote branch.
- Never regenerate lockfiles wholesale; only targeted `cargo update -p <crate> --offline`, unless the ticket approves a broader update.
- Capture exit codes (`cmd > /tmp/x.log 2>&1; echo "exit=$?"`); never pipe a check into head/tail before reading its exit code.
- Never keep important work only in /tmp.

## GPUI code (patch-only)
- Never edit vendor/ or the owned GPUI crates (crates/rustui-gpui*, crates/rustui-block, crates/rustui-cocoa*, crates/rustui-metal, crates/rustui-objc-foundation) by hand. Every change there is a .patch file in patches/ with a manifest (pre/post hashes) and an apply script with --check/--reverse; those files change only by running that script.
- New patches go to patches/proposed/ or the candidate folder the ticket names. Promotion to patches/approved/ only when the owner approves it (in the ticket or message).
- gpui-pre and GPUI Kit stay at their vendored versions unless the current ticket changes them.
- Use GPUI's ownership, scheduling, layout and caches; no second runtime. No dependency on Allui or Chorus. The Playground uses only the public rustui API. Keep code portable (Ctrl fallback for Cmd).

## Reference material
- Before calling any GPUI or GPUI Kit API, find its real signature in the vendored source (vendor/ or the A2 fixture). Never invent an API or translate from React/CSS/older GPUI.
- For GPUI mechanics (focus, elements, entities, async, actions) and GPUI test-window testing, consult docs/reference/gpui-kit-skills-0.8.1/gpui-kit/references/.
- For visual design work (themes, components, UI Studio), consult docs/reference/gpui-kit-skills-0.8.1/gpui-kit-design-guides/ as input; the owner's theme materials and RustUI's design decisions win.
- These references assume apps depend on gpui-kit directly; RustUI wraps GPUI Kit behind its own API with patched vendored copies. RustUI's rules win on any conflict.

## Archive rule
- Before modifying, moving, renaming or deleting a file that existed at tag pre-reset-2026-11, copy its current version to archive/pre-reset-2026-11/<same path> (once, never overwrite). Files created later are not archived.

## Build
- Daily: `rustup run 1.97.1 cargo …`. `./scripts/cargo-headstart` is optional only.
- Never run cargo clean. Use the shared cache CARGO_TARGET_DIR=$HOME/.cache/rustui-target for this repo and every disposable worktree; run `cargo sweep --time 7` on it weekly or when free disk < 50 GB.

## Report
DONE / NOT DONE / GIT (branch, commit, pushed, PR, merged, final git status). Report only what a command verified in this session; otherwise write "NOT VERIFIED". Give scripts/check.sh as the full total of all test groups.
