# M1127 — Stage 10 / reject candidate ID path traversal

In pinned mode, candidate patch application used the inherited candidate ID
when opening a child directory beneath the candidates-parent descriptor. The
ID was also used to bind the manifest path, but it was not checked against the
safe identifier rule before the child lookup. A traversal ID could therefore
address a directory outside the candidates root.

Pinned mode now validates the inherited candidate ID with the existing safe
run-ID rule before opening the child. The regression supplies an ID that
escapes to a sibling fixture and verifies that no patch-check output is
created there.

Focused validation passed:
`python3 -m unittest scripts.tests.test_apply_candidate_patch.ApplyCandidatePatchTests.test_inherited_candidate_id_cannot_escape_candidates_root`
(1 test). No candidate patch was applied to vendored sources, and candidate
execution, review, approval, and promotion remain open.
