# M564: M276 Quad V2 measurement provenance validator

**Status: implemented as a read-only validator.** This task validates the existing M276 paired release measurement bundle for candidate identity, artifact integrity, run pairing, provenance links, and raw timing consistency. It does not run Cargo or the benchmark, modify candidate evidence, set acceptance thresholds, promote the patch, or change the canonical GPUI source.

## Inputs and checks

Run `scripts/validate_quad_sort_performance.py` with the M276 rerun summary and the isolated candidate root. The validator is intentionally bound to candidate `quad-stable-sort-v2-20261020-a` and run `quad-v2-release-20261020-b`.

It rejects paths outside the candidate root, symlinks and non-regular files; verifies the candidate manifest, selected baseline/candidate source snapshots, probe source and release binary digests; cross-checks the run index, provenance verification, comparison, summary records, process/build logs, and all ten raw output files; and checks the five alternating baseline/candidate pairs, all 40 input cases, trial/sample dimensions, correctness hashes, positive finite timing values, normalized durations, and comparison ratios against the raw samples.

The reader uses descriptor-relative `O_NOFOLLOW` access and a bounded JSON read. It reports `measurement_status: valid` only for internally consistent local evidence. That result is distinct from performance acceptance: `acceptance_status` remains `not_assessed`, `performance_gate` remains `not_run`, and `promotion_gate` remains `not_run`. No numeric performance threshold is defined by this validator.

## Verification

The six synthetic tests cover a valid record, path traversal and symlinks, digest drift, candidate transplant, missing or duplicate runs, and invalid timing. The focused suite and Python bytecode compilation passed. The live M276 bundle validated successfully with no errors.

Evidence:

- `target/headstart/evidence/m564-quad-sort-performance-validator-tests-20261021.log`
- `target/headstart/evidence/m564-quad-sort-performance-validator-pycompile-20261021.log`
- `target/headstart/evidence/m564-m276-live-validation-20261021.json`

This is local provenance and measurement-integrity evidence only. It does not establish a performance win, define acceptance, close the native visual gate, or authorize promotion. The noisy M276 measurements still include the ordered 131,071-element regression documented in `QUAD_SORT_V2_PROPOSAL.md`.
