# M836 — Stage 10 P3K rejects unterminated diff rows

`_strict_apply` previously accepted a final unified-diff data row without its
required LF terminator when that row matched a source line without a final
newline. For example, it transformed `before\nkeep` to `after\nkeep` from a
patch ending in ` keep`, with no explicit no-newline marker. This treated an
incomplete diff as a valid exact patch.

P3K now rejects any decoded patch line that is not LF-terminated. Its explicit
no-newline marker remains unsupported. A focused regression verifies rejection
and preservation of the run's failure `result.json`.

This is structural patch validation only. It does not establish candidate
provenance, build or correctness, GUI behavior, performance, approval, or
promotion.
