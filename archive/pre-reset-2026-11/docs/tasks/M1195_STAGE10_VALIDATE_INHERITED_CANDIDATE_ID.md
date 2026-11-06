# M1195 — Stage 10 validate inherited candidate IDs

## Scope

Apply the candidate pipeline's safe single-component identifier contract to
the inherited file-descriptor path in the evidence validator.

## Change

Before opening candidate-relative directories, the validator now rejects IDs
that do not match the pipeline's 1–64 character identifier format, as well as
IDs containing `..`. This closes a validation gap when the caller supplies the
candidate directory descriptors directly.

## Verification

Focused regression command:

```sh
python3 scripts/tests/test_validate_candidate_evidence.py ValidateCandidateEvidenceTests.test_inherited_candidate_id_uses_pipeline_safe_identifier_contract
```

Result: 1 test passed. The regression covers a valid ID plus nested, whitespace,
double-dot, and overlength IDs. Candidate execution, independent review,
approval, promotion, and GUI/performance acceptance remain open.
