# M755 — Stage 10 / pin the exact P3 intake bytes parsed

Candidate intake validation now checks SHA-256 against the exact raw manifest
and patch-plan bytes returned by the parser, closing the gap where a file could
change between a separate hash read and parse read.

The focused regressions `test_hash_pins_the_exact_manifest_and_plan_bytes_parsed`
and `test_copies_snapshot_pins_hashes_and_runs_stages_in_order` passed (2 tests).
This validates intake byte pinning only; it does not establish candidate
provenance, build/correctness, GUI/performance acceptance, approval, or
promotion.
