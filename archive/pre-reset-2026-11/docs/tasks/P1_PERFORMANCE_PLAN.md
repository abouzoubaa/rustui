# P1: RGPE evidence and controlled upstream candidates

Status: planning only. This document inventories current evidence and proposes
work; it does not enable an optimization, change dependencies, or authorize a
build, patch, or upstream revision.

## Current state: observed

- `crates/rustui-rgpe/` is an infrastructure scaffold. Its `lib.rs` calls it a
  scaffold, and `config.rs`, `compatibility.rs`, and `diagnostics.rs` reserve
  planned locations without runtime behavior or public contracts.
- `docs/IMPLEMENTATION_ROADMAP.md` puts reproducible RGPE workloads, profiling,
  approved optimizations, and fallbacks in Stage 10. `docs/UPSTREAM_AUDIT.md`
  identifies existing GPUI retained views, scene replay, text caches and
  virtualization as capabilities to measure before adding RGPE caches.
- `docs/UPSTREAM_AUDIT.md` records source/provenance review, not integration or
  GUI acceptance. It identifies the approved Longbridge snapshot as
  `gpui-pre = 0.3.9`, `gpui-pre-platform = 0.3.9`, and `gpui-component = 0.7.3`,
  with Zed source revision
  `93d745668805029476e6d7fbb19652116fed51de`. This packaging is not a Zed
  published release. Keep the selected snapshot unless a separately reviewed
  and approved candidate replaces it.
- The audit says GPUI Fast is not a RustUI dependency, is disabled in the active
  macOS graph, and must remain disabled. The historical review of
  `longbridge/gpui-fast@fa133a1dd970532dce4ffaee8577438971e5dec9` reports that
  its reviewed path disables retained-subtree and scroll-layer reuse during
  active drag. This is investigation input only, not a patch recommendation or
  acceptance evidence.
- The same audit says UI Studio drag-retention work is reference-only: retained
  content/overlay separation was noted, but no separate current GPUI drag patch
  was identified. Do not treat that reference as a RustUI implementation.
- `docs/BUILDING.md` documents `./scripts/cargo-headstart`, its experimental
  Headstart compiler and metadata workaround, the persistent in-repository
  `target/headstart/` directory, and dependency reuse checks. It explicitly
  says no speedup measurements are claimed, and Cargo cache freshness does not
  guarantee permanent compilation reuse. Cache reuse evidence is not runtime
  performance evidence.
- `scripts/verify_dependency_reuse.py` checks successful Cargo JSON build
  evidence and fresh artifacts for named dependencies. It explicitly infers no
  speedup. `patches/README.md` says no patches are installed and requires source
  identity, checksums, purpose, dependencies, compatibility, fallback, removal
  condition, and validation evidence for any future patch set.

## Not established yet

There is no RGPE runtime contract, measured baseline, benchmark workload, GPU
optimization, selective drag-retention patch, visual acceptance result, or
controlled upstream candidate pipeline recorded by these inspected files.
No conclusion about performance benefit, compatibility, or current runtime
behavior should be made until the relevant work below produces evidence.

## Staged work and delegation boundaries

Do the documentation and inventory work first; defer implementation until an
owner has an accepted workload and baseline. Each row is an independent task
that can be assigned to a capable low-cost model for bounded inventory or
documentation, with a Rust/GPUI specialist reviewing implementation proposals.
Do not delegate unbounded GPU design to an inventory task.

| Stage | Task | Deliverable / dependency | Gate |
| --- | --- | --- | --- |
| P1.1 | Inventory RGPE scope and evidence paths | Path-backed index of scaffold, roadmap, audit, build scripts, patch policy, and existing evidence; mark each fact observed or unknown. | Documentation only; no runtime claims. |
| P1.2 | Define reproducible workload and measurement record | Proposed workload dimensions, machine/toolchain/source identity, warmup and repetition rules, correctness checks, visual comparison inputs, and raw result locations. Depends on P1.1. | Agree on workload before optimization implementation. |
| P1.3 | Establish standard-path baseline | Run only after separate implementation/measurement authorization; capture correctness and performance for unchanged official GPUI paths. | Record failures and variance; do not claim gains from cache reuse. |
| P1.4 | Evaluate one optimization candidate at a time | Candidate proposal using the template below, then isolated implementation/patch and comparative evidence. Depends on accepted P1.2 and baseline. | Human approval before enabling or promoting. |
| P1.5 | Upstream candidate intake and promotion | Isolated pinned candidate workflow below, with validated lock and rollback identity. | Human approval for any dependency revision or promoted patch. |
| P1.6 | Advanced GPU/native composition study | Specialist research plan and safety/correctness constraints, separately scoped. | Separate phase approval before implementation; no implied RGPE authorization. |

### Optimization proposal template

Every candidate, including a no-change result, records:

1. **Name and purpose:** workload bottleneck and expected user-visible effect.
2. **Evidence:** baseline source/revision, workload, correctness result, raw
   measurements, repetitions, variance, and hardware/toolchain identity. Mark
   unavailable data as unverified.
3. **Mechanism and ownership:** smallest proposed change; identify the GPUI-owned
   scheduling, layout, notification, cache, or rendering path it uses. Do not
   add another reactive or rendering runtime.
4. **Compatibility:** exact pinned GPUI/Components versions and APIs, platform,
   features, interaction states (including drag if relevant), and known limits.
5. **Fallback:** how to disable/revert the candidate and retain standard upstream
   behavior without losing correctness or user content.
6. **Validation and removal condition:** correctness and visual checks, measured
   comparison threshold, regressions that reject it, and the condition to remove
   the optimization.
7. **Approval:** named human decision before implementation where scope requires
   it, and again before dependency/patch promotion.

Candidate inventory (hypotheses only; no benefit is established):

| Candidate area | Purpose to investigate | Compatibility focus | Required fallback |
| --- | --- | --- | --- |
| Retained view / scene reuse | Avoid repeating work for unchanged retained content. | GPUI invalidation and identity rules; correctness after state changes. | Use the official GPUI rendering path without the added reuse. |
| Text/layout cache behavior | Reduce repeated shaping or layout work where upstream does not already cover it. | Text, font, locale, theme, scale, and invalidation changes. | Recompute through GPUI and its existing caches. |
| Virtualized collection workloads | Keep large list/table/tree workloads responsive. | Stable identity, selection, scrolling, and data replacement. | Use the official GPUI Components virtualization path or bounded data. |
| Active-drag retention | Investigate whether stable content can be retained while overlays move. | Pointer capture, transforms, invalidation, scrolling, and platform behavior. | Standard GPUI drag/render path; no GPUI Fast enablement. |
| GPU/native composition | Investigate only under the separate advanced phase. | Renderer correctness, resource lifetime, platform support, and fallback rendering. | Standard official GPUI composition. |

## Isolated upstream candidate workflow (proposed)

1. Keep the approved snapshot lock and vendored sources as the last validated
   version. Record its revision/checksums and evidence before creating a
   candidate.
2. After explicit approval to review a particular candidate, fetch that pinned
   candidate into an isolated work area. Do not overwrite the accepted vendor
   tree or change the active dependency graph during evaluation.
3. Apply only approved, versioned patch files in deterministic order. Record
   candidate source identity/checksum, patch checksums, and toolchain/features.
   Fail closed on patch mismatch or conflicts; preserve the last validated lock.
4. Produce candidate-specific build and correctness evidence, then visual
   acceptance evidence and controlled performance comparisons against the same
   workload and baseline. Keep raw logs/results with candidate identity. A
   successful compile alone does not establish GUI or performance acceptance.
5. Compare correctness first, then visual behavior, then performance and
   variance. Reject or revise candidates that regress correctness or fail the
   agreed workload criteria. Record a clear fallback to the accepted snapshot.
6. Present the complete candidate record for human approval. Only after approval
   may a separate authorized change update the dependency revision or promote
   patches. Revalidate the promoted source and retain rollback identity/evidence.

This workflow is a design proposal, not an existing automation. It does not
promise persistent compilation-cache reuse; Cargo may rebuild when source,
compiler, flags, target, profile, features, or build-script inputs change.

## Scope and approval limits

- GPUI Fast stays disabled. Its dragging review evidence is historical and
  exploratory only.
- No GPUI/Components revision change or patch promotion without human approval.
- Advanced RGPE GPU work and native composition require separate phase approval.
- This roadmap creates no runtime feature contract and authorizes no build,
  benchmark, visual run, download, vendor edit, or implementation.
