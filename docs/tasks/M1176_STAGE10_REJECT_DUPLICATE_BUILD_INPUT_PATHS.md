# M1176 — Stage 10 / reject duplicate build-provenance input paths

The cache-comparison provenance validator built a path-to-record dictionary
from `input_file_hashes`, silently discarding duplicate paths. This could let
an ambiguous source-input inventory pass when the last duplicate matched.

The validator now requires a list of object records, nonempty string paths,
unique paths, and an exact match with the expected input inventory before
checking the captured hashes.

The regression duplicates the valid `Cargo.toml` record and recomputes the
snapshot digest; validation rejects it. The focused test passed 1/1, and scoped
`git diff --check` passed. No candidate was executed.
