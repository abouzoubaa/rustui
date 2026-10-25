# M1159 — Stage 10 / normalize P3I Cargo feature selectors

P3I sorted and deduplicated `request.features` array entries but compared each
entry as an opaque string. Cargo accepts comma- or whitespace-separated feature
names within each selector, so equivalent requests such as `first, second`
and separate `first` / `second` entries were reported as a selection mismatch.
Whitespace-only and comma-only selectors also represent an empty selection in
Cargo, but P3I treated them as feature names and mismatched them with an empty
selection.

P3I now tokenizes each selector on Cargo's comma and whitespace separators,
discards empty tokens, and compares the resulting sorted unique feature set.
Other request fields remain exact. The regression covers both equivalent
separator forms and empty-selector equivalence.

Focused validation passed:

- `python3 -m unittest scripts.tests.test_compare_candidate_sources.CompareCandidateSourcesTests.test_feature_selector_separators_and_empty_tokens_are_normalized`.
- `git diff --check` scoped to the P3I comparator, its test, and this task note.

This normalizes requested feature names for selection matching. It does not
prove source provenance or that the selected features resolve to the same graph;
schema-v2 graph comparison reports resolved edge and enabled-feature differences
separately.
