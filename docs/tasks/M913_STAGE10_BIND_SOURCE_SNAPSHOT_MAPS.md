# M913 — Stage 10 / bind source hash snapshot maps to the candidate manifest

## Gap

The P3 evidence aggregator checked the source hash manifest's byte digest and
candidate identity, but did not compare its per-snapshot file maps with the
candidate manifest. A source hash manifest could therefore contain unrelated
file hashes and still be reported as valid after its enclosing digest references
were updated. Its `tree_sha256` values were also not checked against the maps.

## Change

The aggregator now requires baseline and candidate file maps to match between
the two manifests and validates each source hash manifest tree digest against
the corresponding file map. This keeps the separate immutable hash record bound
to the candidate snapshot record used by P3G.

## Evidence

The focused regression altered the candidate snapshot map in the source hash
manifest and updated its referenced digest. Before the fix, aggregation returned
success; after the fix, the focused test rejects the inconsistency. The single
focused test passed after the change.

The tree-hash regression keeps the candidate file maps equal, corrupts the
source hash manifest's candidate `tree_sha256`, and updates its digest
references. `test_source_hash_manifest_tree_hash_matches_snapshot_files` rejects
the inconsistent tree hash and passed when run by itself. Broader Stage 10
acceptance remains open.
