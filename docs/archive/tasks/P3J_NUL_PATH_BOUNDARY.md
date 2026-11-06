# P3J normalized path NUL boundary

`compare_candidate_sources.py` now rejects NUL characters in `workspace_root`
and package `manifest_path` fields. `PurePosixPath` accepts NUL as an ordinary
character, so malformed reports could previously pass P3J validation and be
reported as valid package identities. The focused regression covers both input
fields. This is report-input validation only; it does not add candidate
provenance, GUI evidence, or acceptance.

Reproduction before the fix: a candidate package path of
`source/apps/bad\0name/Cargo.toml` loaded successfully. After the fix, the
focused regression rejects NUL-bearing workspace and package paths.
