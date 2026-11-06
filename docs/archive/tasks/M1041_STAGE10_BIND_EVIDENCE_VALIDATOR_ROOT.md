# M1041 — Bind the P3F evidence validator to the repository root

## Problem

The validator required an inherited repository descriptor but did not verify
that the inherited candidate-parent descriptor was actually
`target/headstart/candidates` beneath that repository. A matching decoy parent
and candidate could satisfy the prior identity checks.

## Change

The validator now opens the repository-relative candidate parent through the
inherited repository descriptor and checks that it matches the supplied
candidate-parent descriptor before validating the candidate.

## Focused evidence

`python3 scripts/tests/test_validate_candidate_evidence.py`

Result: 16 passed, 0 failed. A regression supplies a matching decoy parent and
candidate and verifies explicit rejection.

## Existing candidate gates

The Quad V2 candidate pipeline record is partial but contains passing P3E,
P3G–P3K, and P3F steps. Its separate M274 build gate passed, and M276
correctness passed 10/10. GUI/performance acceptance, independent evidence
review, approval, and promotion remain open. This validator change does not
rerun those gates.
