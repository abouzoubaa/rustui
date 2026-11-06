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

<!-- trail:start -->
## Trail — repo context graph

This repo is indexed in `.trail/cache/`: small linked markdown nodes that explain each
system and carry exact file:line spans, kept in sync with the code through git.

For ANY task here — understanding how something works, finding where code lives,
or scoping a change — get context from the graph before grepping or opening
source files. Re-ask freely (it's cheap) and reuse literal identifiers you
already have (symbol, error string, file name) as the query. New to this repo?
Run `trail map` first — a token-budgeted orientation (dir clusters, hubs,
hotspots), no LLM, no key.

- Run `trail ask "<your question>" --source` → ranked nodes with the relevant
  code spans inlined (each hit's ≤8-line crux by default; `--full` for whole
  definitions when the crux isn't enough). Match the tool to the task shape:
  for understanding or editing, the top node IS the answer — cite its
  `covers:` file:line spans and edit straight from `--source`. For
  exhaustive tasks ("every occurrence / every caller of this pattern"), ranked
  results are top-N, not complete — run `trail grep "<literal>"` instead
  (exhaustive over indexed files, grouped by enclosing symbol), falling back
  to raw `grep -rn` only for unindexed files.
- `trail skeleton <file>` → every definition's signature + span, ~10× cheaper
  than reading the file; use it to skim an API surface.
- `trail callers <symbol>` gives precomputed, exact edges — who calls this.
  Add `--direction out` for what it calls, or `--depth N` to walk
  transitively for the full blast radius. For structural questions, skip
  ranking and use this directly.
- Or browse: `.trail/cache/INDEX.md` lists every node; follow the links.
- Monorepos and folders of multiple repos rank fairly across sub-projects —
  hits carry `[scope/]` labels naming which one they're from. Narrow with
  `trail ask "<task>" --in <scope>/` once you know where you're working.

If a returned span is truncated ("+N more lines"), open the file at that exact
range before finalizing. Only open source files when a node genuinely lacks a
needed detail, and then at the exact file:line the node points to — never
re-read whole files.

After big code changes, refresh the graph with `trail build` (deterministic,
no API key, $0).
<!-- trail:end -->

## Trail learnings

Use durable notes in `.trail/learnings/` to carry forward non-obvious RustUI
decisions and findings. When a task surfaces one that will help future work,
save it with `trail note --title "..." --touches path/to/file --body "..."`.
Keep notes specific to this repository; skip routine task summaries.
