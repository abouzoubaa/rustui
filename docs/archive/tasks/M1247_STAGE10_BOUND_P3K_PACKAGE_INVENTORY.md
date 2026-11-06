# M1247 — Stage 10: bound P3K package inventory copy

P3K's descriptor-pinned package inventory and streaming copy use P3G's shared
100,000-entry and 4 GiB limits. Focused regressions passed 2/2. This closes the
local inventory/copy bounds issue only; candidate execution, current-source GUI
and performance evidence, independent review, approval, and promotion remain
open.
