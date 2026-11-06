# M1099 — Stage 10: reject hard-linked follow-up artifacts

The evidence aggregator's shared file reader now checks regular-file link
counts before and after reading and after reopening the recorded path. This
rejects a follow-up artifact hard-linked outside the pinned candidate root,
including the M287 screenshot case.

The focused M287 regression passed. No candidate was run; Stage 10 candidate
build and correctness results, GUI/performance evidence, independent review,
approval, and promotion remain open.
