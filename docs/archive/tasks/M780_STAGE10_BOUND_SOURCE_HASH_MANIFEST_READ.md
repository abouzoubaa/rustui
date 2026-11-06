# M780 — Stage 10: bound source-hash manifest reads

## Gap

P3L limited `manifest.json` to the P3G snapshot-manifest limit of 32 MiB, but read `source-hash-manifest.json` without a bound. The producer can emit paired snapshot inventories, so a malformed or oversized record could be fully allocated before parsing.

## Change

Apply the same 32 MiB limit to both manifest reads. Add a sparse-file regression larger than the limit and update its recorded digest, so the assertion proves rejection by the size bound before digest mismatch handling.

## Focused evidence

- `scripts/tests/test_aggregate_candidate_evidence.py`: focused oversized source-hash manifest test passed 1/1.
- `scripts/aggregate_candidate_evidence.py`: `py_compile` passed.
- The regression checks only the input-size guard. It does not accept candidate provenance or close build/correctness, GUI/performance, approval, or promotion gates.
