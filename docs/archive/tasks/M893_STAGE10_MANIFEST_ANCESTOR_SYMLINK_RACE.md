# M893 — Stage 10 / reject manifest ancestor symlink swaps

P3G manifest loading now walks from the filesystem anchor with descriptor-relative `O_NOFOLLOW` directory opens. This prevents ancestor-component symlink swaps from bypassing the lexical path checks used by inventory loading.

The focused validator suite passed 22/22, including `test_manifest_open_rejects_symlink_swap_in_root_ancestor`. No broad suite or candidate acceptance was run. This closes one snapshot-path safety gap; candidate provenance, build/correctness, GUI/performance acceptance, review, approval, and promotion remain open.
