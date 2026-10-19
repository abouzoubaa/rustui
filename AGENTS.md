# RustUI working instructions

## Scope and approvals

- Follow the user's authorized task scope and make related file changes needed to complete it without requesting approval for each file.
- Once the user authorizes an objective, proceed with its necessary file edits, implementation, documentation, and validation without pausing for another approval. Keep the work reviewable and report completed milestones as requested.
- Ask before expanding into a materially different objective, changing pinned dependency revisions, promoting an upstream patch, performing an irreversible external action, or committing/pushing. These limits do not create a per-file approval requirement.
- Preserve unrelated work. Do not commit, branch, rebase, push, or initialize Git without explicit authorization.

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
