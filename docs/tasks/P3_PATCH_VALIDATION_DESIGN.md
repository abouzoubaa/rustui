# P3: Isolated upstream patch validation design

**Status: design only.** This document does not implement a pipeline, enable
GPUI Fast, or authorize a fetch, build, test, benchmark, dependency change, or
patch promotion. Enabling Fast or beginning advanced GPU/native composition
requires explicit phase approval. Any upstream revision change and each patch
set also require explicit approval.

## Goal and boundary

Evaluate one explicitly approved upstream candidate without modifying the
accepted source snapshot under `vendor/` or using its Cargo cache. Preserve source
identity, licenses, lock data, patches, and validation evidence for review and a
separate promotion decision. The accepted snapshot remains the fallback until
promotion is approved and the promoted state is revalidated.

The approved direct stack is `gpui-pre 0.3.9`, `gpui-pre-platform 0.3.9`, and
`gpui-component 0.7.3`; its GPUI snapshot identifies Zed revision
`93d745668805029476e6d7fbb19652116fed51de`. This is Longbridge snapshot
packaging, not a Zed-published release or byte-for-byte Zed workspace. The
historical GPUI Fast review is investigation input, not an implementation
recommendation.

## Minimal proposed layout

```text
scripts/validate_upstream_candidate.py        # proposed single orchestrator
patches/<candidate-id>/                       # approved ordered patches and notes
target/headstart/candidates/<candidate-id>/
  manifest.json                               # immutable input manifest
  source/                                      # isolated candidate workspace snapshot
  target/                                      # isolated CARGO_TARGET_DIR
  evidence/                                   # append-only events and result reports
```

The operator supplies a safe candidate ID that does not already exist. The
source snapshot is a copy of the RustUI workspace inputs needed to build the
application: manifests and lockfile, workspace crate and app sources, required
assets, locales, themes, build scripts/configuration, and the candidate upstream
source closure. It must not symlink to live mutable workspace sources, active
`.cargo/config.toml`, `vendor/`, or `target/headstart/`. Candidate configuration
resolves only to the copied workspace and candidate source tree; all candidate
build output goes to its own `target/`. Keep candidate state inside RustUI and
do not copy unrelated files.

`manifest.json` is immutable after intake. It records schema version, candidate
ID and purpose, parent accepted manifest identity, copied workspace input paths
and hashes, candidate Cargo manifests/lock/config, toolchain, target triple,
profile, features and compiler flags. Record corresponding baseline inputs
separately. For each direct and transitive package, record name/version, registry
or repository URL, exact revision when applicable, Cargo lock source/checksum,
source manifest/tree hashes, license expression, and preserved license and
copyright file paths/hashes. Unknown provenance remains `unknown`; a registry
checksum is not a Git revision.

Record pristine upstream package tree hashes separately from patched tree hashes.
Preserve upstream `.cargo-checksum.json` files unchanged as provenance for the
pristine vendored bytes; never rewrite them to bless modified bytes. Patched
crates become candidate-only path sources selected by candidate manifest
resolution or candidate-only `[patch]` entries. Record pristine package identity
and checksum, ordered patch paths and hashes, expected preimages, and resulting
patched tree hashes. No active workspace resolution may point to a candidate
patched source.

Execution and result state belongs in append-only `evidence/` records, not the
input manifest. Each record references the manifest hash and includes command,
observed exit status, log/evidence paths and hashes, provenance and gate state.
Missing checks are `not_run`, never pass. Keep promotion decisions in a separate
attributable human decision record referencing the candidate and last accepted
manifest for rollback.

## Proposed ordered pipeline

The future `scripts/validate_upstream_candidate.py` is one standard-library
Python orchestrator. Invoke `./scripts/cargo-headstart` with the candidate
workspace as cwd, candidate-only Cargo source configuration, and
`CARGO_TARGET_DIR` under the candidate directory. Preserve
`-Zmetadata-crate-hash=no`, the selected toolchain, and caller flags. Network
access is an intake option only after exact candidate revision approval. Later
stages use locked offline/frozen inputs where possible. The orchestrator records
evidence and never promotes automatically.

1. **Intake and pin.** After explicit authorization to validate the candidate,
   require approved exact revisions, package identities, source locations, and
   patch approval references. Copy/fetch required workspace inputs and the
   upstream source closure into the candidate directory. Preserve licenses,
   copyright notices, pristine checksums, and lock provenance. Hash the complete
   snapshot and atomically write the immutable manifest. Fail closed on missing
   identity, mutable revision, checksum mismatch, omitted license, unexpected
   destination, or symlink to live mutable inputs.
2. **Patch application.** Apply only listed approved patches in manifest order
   to candidate-only path sources. Verify patch hashes and expected preimages;
   reject fuzzy or unexpected application. Preserve pristine sources and
   checksum metadata unchanged, and hash patched trees separately. Append an
   application event; do not rewrite the input manifest.
3. **Build and correctness.** Require baseline and candidate workspace snapshots
   to use the same RustUI workload/source inputs except for the reviewed upstream
   candidate and patch. Run only separately authorized build/correctness
   commands in the candidate target directory. Record raw logs, exit status,
   effective inputs, and hashes. Correctness must pass before GUI or performance
   results are interpreted.
4. **GUI and visual acceptance.** Require human-operated native Playground checks for
   scenarios and platform agreed in the workload plan. Record scenarios,
   reviewer, result, and visual evidence. Compilation and headless checks cannot
   satisfy this gate.
5. **Performance comparison.** After correctness and GUI gates, compare the
   candidate and accepted baseline with the same approved workload, machine
   conditions, warmup/repetition/variance method, and regression thresholds.
   Store raw measurements and analysis. Cargo cache freshness or build duration
   alone is not performance evidence.
6. **Review and promotion.** Produce a result report referencing immutable input
   identity and all gate events. Failed, incomplete, or inconclusive candidates
   stay isolated and cannot alter working vendor files, accepted manifests, or
   accepted caches. Promotion is a separate human-approved operation: apply the
   approved revision/patch through the repository's versioned process, update
   active resolution only under explicit revision approval, then revalidate that
   state. Record the previous accepted manifest as rollback identity. The
   candidate script must never perform these steps implicitly.

Create candidate directories and evidence only after explicit candidate
validation authorization. Use unique append-only event names and retain partial
logs on failure; never clean or overwrite a candidate directory.

## Existing tools and missing pieces

- `scripts/record_build_provenance.py` hashes an existing log and captures
  current workspace inputs. It does not parse the log for success, query Cargo or
  rustc, capture vendored source identities, or bind captured state to build
  time. It cannot identify an isolated candidate snapshot by itself.
- `scripts/verify_dependency_reuse.py` validates a successful Cargo JSON build
  event and artifact freshness for named packages. This is a narrow cache fact,
  not executable correctness, GUI acceptance, performance, or isolation.
- `scripts/cargo-headstart` selects the experimental compiler, preserves the
  metadata hash workaround, and defaults outputs to `target/headstart/`. It does
  not accept candidate workspace identity, isolate source resolution or target
  cache, or guard promotion.
- `patches/README.md` says no patches are installed and lists review fields. It
  does not provide candidate snapshotting, immutable manifests, deterministic
  patch application, result events, acceptance gates, or promotion controls.

Missing pieces are the candidate orchestrator, isolated workspace snapshot and
source resolver, input hashing, pristine-versus-patched source accounting,
candidate-specific Cargo configuration and target directory, deterministic
patch application, baseline/candidate pairing, and evidence capture for
correctness, native GUI acceptance, and controlled performance comparison.

## Dependencies and unresolved choices

Order: define and approve a workload/acceptance plan; approve the exact candidate
revision and patches; snapshot and pin inputs; capture the accepted baseline;
apply patches; run separately authorized correctness/build checks; complete GUI
acceptance; compare performance; review evidence; separately approve promotion
and post-promotion validation.

The W1 baseline is no longer pending: Milestone 14 and
`P2E_W1_ACCEPTANCE_BASELINE.md` record native acceptance, mounted-state readback,
and 90 measured cycles across three fresh processes after warm-up. That is a
same-build CPU/RSS repeatability baseline, not a candidate comparison or speedup
result. Numerical regression thresholds remain unapproved, and presented-frame,
GPU, and main-thread timings remain unavailable. The P2 workload plan and
calibration evidence are inputs to future candidate work; they do not authorize
running this proposed pipeline. Resolve these choices before implementing the
orchestrator:

1. **Source closure:** copy the full locked vendor closure or only the proven
   transitive closure for the selected platform/features. Full closure is easier
   to audit; minimal closure needs a completeness proof.
2. **Promotion representation:** use a versioned patch against the accepted
   snapshot or a newly pinned upstream snapshot plus patch series. Preserve both
   upstream and local patch identity either way.
3. **Acceptance ownership:** define Playground scenarios, platform, visual reviewer,
   workload, and numerical thresholds under P2 before collecting acceptance
   evidence.

These choices do not enable GPUI Fast or advanced GPU/native composition; both
remain behind separate explicit phase approval.


## Milestone 12 prerequisite update

The external process collector, per-run summarizer, and timing-hook review are
implemented. They provide CPU/RSS evidence, not a candidate-validation pipeline,
workload completion timing, or presented-frame timing. At this milestone, W1
deterministic reset/actions and repeated calibrated measurements remained
pending. Milestone 14 and `P2E_W1_ACCEPTANCE_BASELINE.md` later superseded that
W1 status with native acceptance and a three-process, 90-cycle CPU/RSS baseline;
they did not approve numerical regression thresholds or add cycle/presentation,
GPU, or main-thread timing. No candidate patch, revision change, or GPUI Fast
activation is authorized by this tooling milestone.

## Milestone 14 baseline availability

The bounded W1 CPU/RSS baseline and native GUI acceptance now exist; see
`P2E_W1_ACCEPTANCE_BASELINE.md` and the hashed machine-readable evidence. Future
candidate comparisons must repeat the same compact-view protocol with a
contemporary baseline. This does not supply presented-frame/GPU/main-thread
timing, approve numerical regression thresholds, or implement the P3 pipeline.
GPUI Fast features remain candidates for RustUI additions or controlled upstream
patches; the fork itself is not adopted.
