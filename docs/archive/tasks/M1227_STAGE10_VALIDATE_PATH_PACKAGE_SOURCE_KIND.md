# M1227 — Validate P3I path package source kind

P3I now requires a package with `source_kind: "path"` to have the exact raw
`source` value `"path"`. This rejects contradictory source metadata instead
of normalizing it as a path dependency.

```sh
python -m unittest \
  scripts.tests.test_compare_candidate_sources.CompareCandidateSourcesTests.test_path_package_source_must_match_source_kind
```

Result: passed (1/1). This is source-inventory validation; broader graph and
candidate acceptance gates remain open.
