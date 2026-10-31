# P3I: Read-only baseline/candidate source inventory comparison

**Status: bounded inventory comparison implemented.** M179/P3I consumes two
independently valid P3H reports and compares their package name, version,
source-kind, and stable workspace-relative path identity. Both reports must
use the same locked-offline feature/platform selection. M181/P3J extended the
same comparator to compare normalized edges and enabled features only when
both reports carry P3H schema version 2 graph data. The comparator does not
invoke Cargo and does not mutate either report or candidate.

## Boundary

`scripts/compare_candidate_sources.py` rejects invalid P3H reports, duplicate
JSON keys, mismatched selections, malformed package inventories, unsafe package
paths, and duplicate normalized identities. It reports packages added to or
removed from the candidate inventory. M179 preserved the then-opaque v1 graph
digests for reference; M181 now validates normalized schema v2 graph digests
against their edge/feature payload before comparing them.

P3H schema version 1 does not expose resolved dependency edges or per-package
feature sets. A v1 or mixed-version pair therefore remains **inventory-only**;
its digest equality or difference is not graph evidence. For two schema version
2 reports, graph comparison includes normalized package identities, dependency
aliases and kinds/targets, and enabled per-package features. Neither mode
establishes source provenance, license correctness, build or runtime
correctness, GUI or performance acceptance, or permission to apply patches or
promote a candidate.

## Invocation

```sh
python3 scripts/compare_candidate_sources.py \
  target/headstart/evidence/p3h-baseline-source-resolution.json \
  target/headstart/evidence/p3h-candidate-source-resolution.json
```

The JSON report contains `inventory_equal`, sorted `added` and `removed`
package identities, selection-match status, and opaque graph digests. Invalid
or mismatched inputs produce a nonzero exit status.

## Verification

`python3 scripts/tests/test_compare_candidate_sources.py` covers root-independent
path package IDs, package-version deltas, selection mismatch, report integrity,
and workspace containment; all five tests pass. A live locked-offline P3H
baseline report was generated from the prepared Quad-sort baseline snapshot and
compared with the existing candidate report. Both inventories contain 804
packages with no added or removed package identities. The P3H graph digests
differ; P3I does not infer graph equivalence or explain that difference. Raw
reports are `target/headstart/evidence/p3h-quad-baseline-source-resolution-20261020.json`
and `target/headstart/evidence/p3h-quad-source-resolution-20261020.json`; the
comparison is `target/headstart/evidence/p3i-quad-source-inventory-comparison-20261020.json`.
Focused test output is `target/headstart/evidence/p3i-source-comparison-tests-m179-20261020.log`.

This is a read-only comparison stage only; it does not constitute candidate
intake, deterministic patch application, build or correctness execution, native
acceptance, performance evaluation, or promotion.
