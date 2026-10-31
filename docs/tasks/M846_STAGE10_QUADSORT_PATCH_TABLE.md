# M846 — Stage 10: reuse the existing Cargo patch table

The quad-sort candidate preparer appended a second `[patch.crates-io]` table to each copied workspace manifest. The repository manifest already defines that table, so the resulting Cargo.toml was invalid TOML and could not faithfully prepare a buildable baseline/candidate pair. Preparation now redirects the existing `gpui-pre` entry to the copied path source, preserving the other patch entries and validating the rewritten TOML.

Validation: `python3 -m unittest scripts.tests.test_prepare_quad_sort_candidate` passed (1 test). The current workspace manifest also rewrites with exactly one patch table. This closes a candidate-preparation manifest defect only; candidate build/correctness, GUI/performance acceptance, review, approval, and promotion remain open.
