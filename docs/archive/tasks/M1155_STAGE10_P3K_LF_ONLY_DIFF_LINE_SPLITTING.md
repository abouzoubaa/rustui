# M1155 — Stage 10 P3K LF-only diff line splitting

`_strict_apply` previously used `str.splitlines(keepends=True)` for unified
diffs and source text. Python treats Unicode NEL (U+0085) as a line separator,
even though unified diffs use LF-delimited physical lines. Consequently, a
valid patch changing a UTF-8 Rust source line containing U+0085 was rejected
as an unterminated diff line.

P3K now splits decoded text only on LF, preserving all other Unicode
characters as line content. A focused regression applies an LF-delimited
patch to a source line containing U+0085 and checks the exact resulting bytes.
The P3K focused suite passed 44/44 tests with:

`python3 -m unittest scripts.tests.test_apply_candidate_patch -v`

This verifies the local strict parser and synthetic patch application only; no
candidate was applied, run, or promoted.
