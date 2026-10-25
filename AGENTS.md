# RustUI working instructions

## Scope and approvals

- Follow the user's authorized task scope and make related file changes needed to complete it without requesting approval for each file.
- Once the user authorizes an objective, proceed with its necessary file edits, implementation, documentation, and validation without pausing for another approval. Keep the work reviewable and report completed milestones as requested.
- Ask before expanding into a materially different objective, changing pinned dependency revisions, promoting an upstream patch, or performing an irreversible external action. These limits do not create a per-file approval requirement.
- The user has granted standing authorization to create branches, commit, rebase, push, and initialize Git as needed to complete the active RustUI roadmap objective. Keep Git changes scoped and reviewable, and preserve unrelated work.

## Architectural boundaries

- No direct, transitive, optional, or public API dependency on Allui.
- No dependency on Chorus applications, business logic, or local Chorus paths.
- GPUI and GPUI Components remain official upstream sources of truth.
- The user selected Longbridge's `gpui-pre = 0.3.9` snapshot of official Zed
  revision `93d745668805029476e6d7fbb19652116fed51de`, with Components `0.7.3`.
  The approved snapshot packaging is distinct from a Zed-published release.
- Keep dependency sources under `vendor/` and build artifacts under
  `target/headstart/` inside this repository. Preserve upstream checksums/licenses;
  do not edit vendored code directly. GPUI Fast remains disabled.
- Add or change upstream revisions only after approval. Internal changes belong
  in a reproducible, versioned, explicitly approved patch process.
- Use GPUI ownership, scheduling, layout, notifications, and existing caches.
  Do not introduce a second reactive or rendering runtime without approval.
- RustUI Playground is the canonical component development application and uses
  only the public `rustui` API. Keep direct GPUI and GPUI Components dependencies
  inside framework implementation crates.
- `rustui-lab` is a compatibility launcher that delegates to Playground; keep one
  application and story implementation.
- Theme files and locale records marked planned are planning metadata, not runtime
  packs or translations. Empty modules do not establish feature support.
- Advanced RGPE GPU work and native composition require separate phase approval.
- Keep copied upstream code licensed and attributed. No upstream code is copied
  into this initial scaffold.

## Tools and evidence

- For multi-session roadmap work, read `docs/tasks/multi_session_coordination.md`
  and use the canonical shared coordinator from your own linked worktree. The
  `--thread` argument must be your own Codex thread ID, supplied literally in
  your assignment; never use the orchestrator's ID. Before editing, confirm the
  ledger shows your own thread ID, actual worktree, unique `codex/` branch, and
  exact assigned paths. If the row is missing or any value differs, stop and
  report it before writing.

- Reuse one disposable Playground validation bundle and bundle identifier; do not
  create a separate application for each milestone.
- After each GUI test, quit the test application and remove its disposable app
  bundle and standalone test executable. Apply the same cleanup to isolated
  probe apps. Preserve source, validation evidence, and shared Cargo dependency
  and incremental caches. Verify that no test app remains running.

- Follow /Users/Abium/.codex/RTK.md. Use RTK for routine exploration; use raw commands
  for exact Git state, diagnostics, failures, reviews, and acceptance decisions.
- Retrieve original content for Headroom CCR/compression markers before relying
  on exact details.
- Apply /Users/Abium/Developer/OmniRoute/skills/ponytail/SKILL.md at full intensity
  for coding, review, architecture, and implementation tasks.
- Apply /Users/Abium/Developer/OmniRoute/skills/omni-compression/SKILL.md when working
  with prompt, message, tool-output, or context compression.
- Apply /Users/Abium/.agents/skills/graphify/SKILL.md for codebase/architecture
  questions. Query an existing graph when present; do not generate one during a
  read-only task.
- Use ./scripts/cargo-headstart for Cargo build/check/run/metadata commands.
  The user approved Headstart with -Zmetadata-crate-hash=no. It is experimental.
  Keep that flag visible to Cargo, honor caller flags, and reuse the persistent
  target directory. The dev/headstart profiles disable incremental compilation only
  for `rustui-macros` because that proc-macro still triggered the SVH ICE. Preserve
  incremental compilation for the rest of the workspace. Do not run cargo clean
  during normal iteration.
- Rust 1.97.1 supplies rustfmt/host tooling; the launcher selects installed
  Headstart Cargo/rustc. Dependency revisions still require approval.
- Use the least expensive reliable model and reasoning level available. Prefer `gpt-6-luna` Low for documentation, tests, simple fixes, and routine implementation; use Luna Medium when routine work needs more reasoning. Use `gpt-6.1-sol` Low or Medium for complex Rust/GPUI integration, and Sol High for difficult architecture, rendering internals, concurrency, or performance investigations. Use `gpt-6-astra` only for exceptional problems after decomposition and cheaper-model attempts are insufficient.
- When model selection is available, run the orchestrator on `gpt-6-luna` High and assign each delegated task its own cheapest suitable model and reasoning level. The active chat model cannot be changed retroactively; do not claim it was switched. Prefer a small number of independent agents, avoid duplicate investigations, and use deterministic checks instead of extra model reviews.
- Do not invent cost or token-savings figures. Report model assignments and outcomes; quantify savings only when reliable usage telemetry is available.
- Never claim a build, test, benchmark, visual check, or accessibility check passed
  unless it ran. Compile success of this scaffold is not GUI acceptance.

## Multi-session roadmap coordination

- Before any roadmap edit, create or use one isolated Git worktree for the
  writing session and claim its task, worktree, and exact repository-relative
  files with the canonical coordinator at
  `/Users/Abium/.codex/worktrees/session-coordination/rustui/scripts/roadmap_claims.py`,
  invoked while the shell is in the session's own worktree. Follow
  `docs/tasks/MULTI_SESSION_COORDINATION.md`.
- Use `advance` for same-session task changes or scope expansion; it preserves
  the existing claim atomically. Use `handoff` only after independently
  confirming that the old session is terminal.
- Start claims only from a clean registered worktree on a unique `codex/`
  branch. Record Stage, milestone, and next action; run `audit` at turn
  boundaries and before integration. Never assign overlapping paths or the
  same worktree/branch to simultaneous writers.
- Do not write in the shared primary checkout or outside claimed paths. A claim
  on a directory conflicts with claims on any child or parent path. Claim shared
  ledgers as `@shared/<name>`; only the coordinator edits them, one at a time.
- Renew claims at the start and end of each work turn. Expired claims are
  suspect and remain locked; never reassign them based only on timeout, idle,
  blocked, or not-loaded session status.
- Release or transfer a claim only after the coordinator checks the old thread
  is terminal and records evidence. If ownership is uncertain, preserve the
  claim and assign disjoint files in a different worktree.
- The registry is cooperative enforcement: every session must claim before
  writing, and the coordinator must audit claims before assigning sessions.
  A registry entry is not proof that an unregistered process stopped; verify
  terminal session state before release or handoff.
