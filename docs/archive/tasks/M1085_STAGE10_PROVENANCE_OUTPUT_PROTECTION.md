# M1085 — Stage 10 provenance output protection

`record_build_provenance.py --force --output` now rejects destinations that
alias declared provenance inputs, scanned crate/app sources, the selected
binary, or the supplied build log. This prevents replacing a file after its
bytes have been captured in the record.

Focused regression: `test_force_rejects_output_path_that_is_a_captured_build_input`
passed 1/1. The recorder's existing hard-link-to-build-log regression and
`git diff --check` also passed. No candidate was executed. Independent build
and correctness evidence, GUI/performance evidence, review, approval, and
promotion remain open.
