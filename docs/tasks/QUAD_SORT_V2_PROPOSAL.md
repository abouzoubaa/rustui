# Proposed V2 quad ordering policy (unapplied)

Status (updated 2026-10-20): the V2 patch was applied and built only in an isolated candidate. The test harness and paired release probe passed their execution gates, but measurements are noisy and include a large ordered-input regression; V2 remains unpromoted. The active vendor and approved V1 fallback remain unchanged. Native visual acceptance is still open.

## Measured reason for a narrower counting path

The paired release reset-copy-plus-`Scene::finish` batch comparison in `target/headstart/candidates/quad-stable-sort-20261018-a/evidence/quad-batched-release-comparison.json` found candidate/baseline median ratios of about 1.02–1.03 on four length-16 kinds and 1.06–1.11 on all five length-131072 kinds. Length-16 reverse was faster at about 0.64, so a small-size fallback deliberately gives up that case. At length 1024 and 16384, dense and reverse cases were substantially faster, while sparse-max was about 1.04 and 1.05. The comparison measures one instrumented CPU workload, with copy/reset cost reported separately; it is not frame, GPU, general application, or native visual evidence. The strengthened run-index/manifest provenance check is a separate gate before these results are relied on for a promotion decision.

## Exact proposed artifact

| Item | Identity |
| --- | --- |
| Proposed full replacement patch | `patches/proposed/gpui-pre-quad-stable-sort-v2.patch` |
| Target source | `vendor/gpui-pre-0.3.9/src/scene.rs` in a **new isolated snapshot**, never direct editing of active `vendor/` |
| Pinned pristine preimage SHA-256 | `3addca59aab12ca0e00acb1e39d652863a8904146db2e28192275660aa5b8880` |
| In-memory V2 postimage SHA-256 | `1da784711db7148374fb357652e01e2e8b21050a79127124f25dc18a570ae549` |
| V2 patch SHA-256 | `75d876d572664df312091145de445eefa0641316f6640b5be8f7c2854a975ae7` |
| Prior V1 postimage / patch SHA-256 | `08d04c593184213f433a046c01a77640a0ad83fa0c2d63d7588daee3ba485845` / `2e1b35252aa120e5f7a7138f2c8a4b92aa9035f6edce7c4294333368d5909853` |

The V2 file is a complete unified diff against the pristine preimage, not an incremental patch on V1. It was constructed from the existing V1 candidate source in memory and diffed against pristine source. The existing Apache-2.0 attribution is retained. The hashes identify the proposal bytes; they do not prove it compiles or performs better.

## Small policy change

Keep V1's stable counting algorithm, payload-preserving permutation, 65,536-bin limit, 1 MiB retained scratch budget, allocation-failure stock fallback and no-GPUI-Fast dependency. Route lengths below 1024 or at least 131072 directly to GPUI's existing stable `sort_by_key`, releasing any retained counting scratch. The thresholds bracket only the measured sizes; behavior between sampled sizes still needs measurement.

For counting-eligible lengths, compute the largest range permitted by the existing bin and density caps once. While scanning orders, if a disorder has already been seen and the current maximum makes counting ineligible, return immediately to stable stock sorting. An order sequence still known to be sorted continues through the scan and may return `AlreadyOrdered`, including a uniform `u32::MAX` sequence. The early guard targets the alternating `u32::MAX`/zero sparse workload; it does not claim a new sorting algorithm or a different stable-order contract.

The patch adjusts V1's focused test expectation so the counting-path assertion uses a 1024-element dense input, and adds full-payload oracle checks for small stock fallback, sparse early stock fallback, ordered maximum values and large stock fallback. These tests are present in the proposal but have not run.

## Approval and validation before any replacement

1. Review the V2 diff and hashes against the pristine package; reject a preimage mismatch and preserve the V1 patch/snapshots as the current fallback. Apply only in a new versioned isolated candidate after approval.
2. Compile through `./scripts/cargo-headstart` with the same locked dependency graph, then run the focused stable-order tests and `Scene` integration/production builds. Retain raw diagnostics. The prior upstream full-library test build failed before executing quad tests, so a build failure is not a passed test.
3. Re-run the identical paired release probe protocol against a fresh V2 candidate and pristine baseline with a fresh run index, binary/source/output digests and snapshot manifest. The existing `compare_batched.py` is bound to V1's manifest and cannot certify V2 results unchanged. Include unsampled sizes around both thresholds and dense, reverse, ordered, duplicates, and sparse-max cases; assess absolute times, reset cost, allocations and pair spread, not only ratios.
4. Repeat native visual/stable blending and clipping checks and preserve the stock fallback. Make any promotion decision only after correctness, native and performance evidence is reviewed; no automatic replacement follows from this proposal.


## Isolated V2 candidate validation and release probe (2026-10-20)

The exact V2 patch (`patches/proposed/gpui-pre-quad-stable-sort-v2.patch`) was applied only to the disposable candidate `target/headstart/candidates/quad-stable-sort-v2-20261020-a/`. Snapshot validation, zero-fuzz patch application, the extracted-code harness (9/9), candidate workspace all-targets check, and production Playground build passed. No canonical vendor source was changed.

The upstream `gpui-pre` unit-test build did not reach test execution: it failed on absent font assets and test-support dependencies/APIs (`futures_concurrency`, `postage`, `proptest`, `util_macros`, and `FakeHttpClient`). This is a failed compilation attempt, not an upstream unit-test pass.

The release probe completed five fresh baseline/candidate pairs (10/10 process exits were 0). The comparison covered 40 cases and 200 paired ratios using `reset_copy_finish_batch_v1`; median case ratio was 0.98534874 and median pair ratio was 0.97819485. Median within-case pair spread was 0.26950231; case medians ranged from 0.23704258 to 1.73561411. The ordered 131,071-element case regressed to 1.73561411. Allocation-call and byte totals matched in 27/40 cases. These process CPU measurements are noisy workload evidence, not frame, GPU, or whole-application results, and do not support promotion.

Provenance records: `quad-v2-release-20261020-a-run-index.json` SHA-256 `6f19c12a5d7d92fd3466f27853a999341d8895707c7dbe711c7fc3b11c25bc44`; comparison `quad-v2-release-20261020-a-comparison.json` SHA-256 `f36c6c20b112e301d1a089c6f77ecb86028f57e80dc815bb6d5dc53eece5ec16`; verification `quad-v2-release-20261020-a-provenance-verification.json` SHA-256 `46a6aba37e2b0ea034928fa3a9de3d40c02b3af1cf1a0660f205507c965e6119`. All are under the candidate `evidence/` directory. Native visual acceptance and any promotion decision remain separate.

## Fresh paired rerun (M276, 2026-10-20)

A second isolated release run repeated five fresh baseline/candidate pairs.
Both builds and all ten probe processes exited successfully; differential
oracle checks passed for all 40 cases (200 paired ratios). Median case ratio
was 1.00330173 and median pair ratio was 1.00321695. Median within-case pair
spread was 0.37279880, indicating substantial measurement variability. The
ordered 131,071-element case again showed a material regression: median ratio
1.73022114, with paired ratios from 1.69727552 to 1.96525942. The rerun summary
SHA-256 is
`3d097f97a5fbc600e01858b78f4f7603f685c04d973331f5aa42cd0d92fd798d`; its run
index SHA-256 is
`26baf48d8f397a714596a2fcdd42fb7936ad8bb5a83c31f27f14766c82c69206`.
No numeric acceptance thresholds are defined. The ordered-input regression is
reproduced, so performance acceptance and promotion remain unsupported. Keep
the active vendor and V1 fallback unchanged pending a revised candidate,
defined thresholds, and new correctness, performance, and native evidence.


## M288 — Stage 10 / P3 dispatch-boundary experiment

Created isolated candidate `quad-stable-sort-v2-m288-20261020-a` with only the candidate counting-path upper bound lowered to 16,384; 16,384 and larger inputs use stock stable sorting in this experiment. This value is an experiment boundary, not an acceptance threshold. The candidate source is `target/headstart/candidates/quad-stable-sort-v2-m288-20261020-a/candidate/source/path-sources/gpui-pre-0.3.9/src/scene.rs` (SHA-256 `10c9351330a99cadfdebae828c8c65150c163d8f6ed310d9d9dbdb7d1d54fda1`); the baseline source hash is the pinned pristine `3addca59aab12ca0e00acb1e39d652863a8904146db2e28192275660aa5b8880`. The updated focused candidate correctness run passed 9/9, including 16,383 → Counting and 16,384 → Stock assertions through the stable-order/full-payload oracle; both isolated locked/offline release-probe builds passed. The original 9/9 log predates those boundary assertions and is superseded for boundary coverage. A stale inherited Cargo source configuration was caught before timing; the rebound probe configuration resolved the baseline and candidate to their own source roots. Timed pairs were then deferred before the idle gate because WindowServer, cloudd, and a Graphify watcher sustained material unrelated CPU load. No run index or timed comparison was created. Timing and performance acceptance remain open; no vendor source or dependency revision was changed. Evidence: `target/headstart/candidates/quad-stable-sort-v2-m288-20261020-a/evidence/m288-focused-tests-only.log`, `target/headstart/candidates/quad-stable-sort-v2-m288-20261020-a/evidence/m288-focused-test-boundary-retest.json`, `target/headstart/candidates/quad-stable-sort-v2-m288-20261020-a/evidence/m288-release/m288-builds-ready.json`, and `target/headstart/candidates/quad-stable-sort-v2-m288-20261020-a/evidence/m288-release/m288-timing-deferred.json`.


## M294 — Stage 10 / P3 ordered-Quad cutoff experiment

Fresh candidate `quad-stable-sort-v2-m294-20261020-a` changes only the exclusive counting cutoff from 131,072 to 131,071. Candidate source SHA-256 is `eb707355dabd76bce37fd3214894e5e2b2dbb207e5d455750213c4cbb4dfa97e`; the baseline is `3addca59aab12ca0e00acb1e39d652863a8904146db2e28192275660aa5b8880`. The isolated focused harness passed 10/10, proving the 131,071 ordered case uses Stock with full-payload stability and the 1,024 dense case still uses Counting. Locked/offline release `cargo check -p gpui-pre` passed. The full test command's 10 library tests passed but its doctest failed with E0514 because Headstart-built GPUI metadata was read by host rustdoc; a retry with available Headstart stage0 rustdoc also failed because it does not match the stage1-built metadata. The wrapper's expected stage1 rustdoc is not installed. Evidence: `evidence/m294-correctness-lib-test.log`, `evidence/m294-candidate-release-check.log`, `evidence/m294-correctness-test.log`, and `evidence/m294-doctest-headstart-stage0.log` under `target/headstart/candidates/quad-stable-sort-v2-m294-20261020-a/`. No timed pairs ran, so this experiment does not establish a performance win or acceptance. Active vendor code and dependency pins remain unchanged.
