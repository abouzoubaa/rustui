# Proposed V2 quad ordering policy (unapplied)

Status: source patch proposal only. No V2 build, test, native check, benchmark, or promotion has run. Applying it requires explicit approval and a fresh isolated candidate. The active vendor, approved V1 patch, baseline/candidate snapshots and release benchmark results remain unchanged.

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
