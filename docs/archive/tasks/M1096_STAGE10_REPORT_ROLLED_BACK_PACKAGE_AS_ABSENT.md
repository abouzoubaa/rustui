# M1096 — Stage 10: report a rolled-back package as absent

When pinned schema-three patch application fails and rollback succeeds, its
result now reports `output_package: null`. The package directory has been
removed, so the report no longer points to a nonexistent output path.

The focused pinned-descriptor rollback regression passed. No candidate was
run; independent build/correctness and GUI/performance evidence, review,
approval, and promotion remain open.
