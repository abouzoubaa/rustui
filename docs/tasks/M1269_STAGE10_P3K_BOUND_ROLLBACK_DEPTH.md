# M1269 — Stage 10 bound P3K rollback traversal

P3K rollback now reuses `PinnedDir.remove_tree` for iterative cleanup, passes
the captured disposable-package identity, and applies the existing
100,000-entry package limit. Removal enumerates entries incrementally under
that budget and rechecks non-directory identities before unlinking. This
preserves M1251's replacement protection while avoiding recursive cleanup and
unbounded per-directory entry lists.

The focused P3K rollback regressions passed 4/4, including 1,100-level cleanup
under a low descriptor limit, entry-limit rejection, and directory/file
replacement preservation. The P3G filesystem suite passed 23/23. Python
bytecode compilation and scoped diff checks passed.

This closes a local rollback resource and integrity gap only. Candidate
execution, correctness, GUI/performance acceptance, independent review,
approval, and promotion remain open.
