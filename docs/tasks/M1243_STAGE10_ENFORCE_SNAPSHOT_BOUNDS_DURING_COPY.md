# M1243 — Enforce snapshot bounds during tree copy

Candidate source-tree copying now enforces the P3G limits of 100,000 entries
and 4 GiB while streaming, rather than relying only on pre-copy inventory. A
file that grows after preflight is rejected once the remaining byte budget is
exceeded, and the partial destination file is removed. Existing pre-copy and
post-copy inventories remain in place.

The focused growth regression and four existing copy-mutation/FIFO regressions
passed (5 total).

This closes a local copy-boundary gap. It does not establish candidate build or
correctness, GUI/performance acceptance, independent review, approval, or
promotion.
