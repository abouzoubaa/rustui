# M1233 — Bind every path-source file to the P3G tree hash

P3G now rejects files placed directly under `path-sources/`. The previous
inventory included those files while the package tree hash covered only files
inside `path-sources/<package>/`, leaving a path-source file outside the tree
binding. The regression adds such a file while preserving the old tree hash
and confirms rejection.

```sh
python3 scripts/tests/test_validate_candidate_snapshot.py
```

Result: passed (33/33). This closes one snapshot-integrity gap; package
provenance, graph equivalence, candidate execution/correctness, GUI/performance,
independent review, approval, and promotion remain separate gates.
