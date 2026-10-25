# M1100 — Stage 10: bind snapshot candidate identity to the pinned root

The inherited snapshot validator now rejects a manifest whose `candidate_id`
does not match the candidate ID bound to its pinned candidate directory. The
focused regression constructs a coherent paired snapshot under candidate-1
with a manifest naming candidate-2.

The focused test passed. No candidate was run; Stage 10 candidate build and
correctness results, GUI/performance evidence, independent review, approval,
and promotion remain open.
