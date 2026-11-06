# P3G: Read-only paired candidate snapshot validator

**Status: bounded local validator implemented.** P3G checks the source
directories recorded by the existing candidate preparation manifests. It does
not create or repair snapshots, select Cargo sources, run Cargo, establish
provenance or approval, apply patches, or accept/promote a candidate.

## Checks

`scripts/validate_candidate_snapshot.py` reads one JSON manifest and verifies
that the preparation manifest uses supported schema version 1 and that baseline
and candidate source paths are distinct directories beneath the candidate root.
The schema version must be an integer; JSON booleans are rejected. It rejects
symlinked path components including the snapshot root, symlinks or non-regular
files inside snapshots, malformed file inventories, and files whose recorded
SHA-256 values differ from disk. It also checks the recorded path-source tree
hash and requires paired snapshots to differ only below `path-sources/`.

The report names each observed source delta and file count. A `valid` result
means only that the two local snapshots match their inventory and preserve
workspace-input parity under this rule. It does not prove that paths point to
approved upstream revisions, that license/provenance claims are true, that
Cargo resolves to those sources, or that any patch is approved.

## Invocation

```sh
python3 scripts/validate_candidate_snapshot.py \
  target/headstart/candidates/example/manifest.json \
  --root target/headstart/candidates/example
```

The validator is read-only and uses only Python's standard library.

## Verification

`python3 scripts/tests/test_validate_candidate_snapshot.py` covers valid paired
inventories, unsupported/missing/Boolean manifest schema versions, snapshot
hash drift, a non-patch workspace delta, symlink rejection and swap races,
source-root escape, and invalid candidate IDs. The current focused suite has
11 passing tests, including source-root, manifest, and snapshot-file race
regressions. See
`target/headstart/evidence/p3g-inventory-no-follow-m451-tests.log` and
`target/headstart/evidence/p3g-inventory-no-follow-m451-pycompile.log`. This is
a local validator, not a source intake or Cargo-resolution stage.

## Remaining P3 pipeline work

P3H schema 2 records effective locked-offline Cargo package sources, dependency edges, and enabled per-package features; P3I compares source inventories and P3J compares normalized dependency graphs. M181 verified the prepared Quad-sort pair with the P3J graph comparator; see `docs/tasks/P3J_CANDIDATE_GRAPH_COMPARISON.md` and its evidence records. This validates only the prepared local snapshots and does not establish candidate intake or promotion safety. Candidate snapshot creation, deterministic patch application
orchestration, build/correctness, native GUI acceptance, performance
comparison, evidence aggregation, and separate human promotion remain open.

## M377 — Reject candidate-root symlink aliases

`_resolve_source_path` now rejects symlinked lexical ancestors before accepting a resolved path as the candidate root. Regression coverage for a root-alias path is included in the focused suite, which passed 8/8. Static symlink components are covered; concurrent path swaps are not claimed. Evidence: `target/headstart/evidence/p3g-symlink-root-alias-m377-20261021.log`.

## M429 — Reject symlink aliases for manifest input

The CLI now walks the lexical manifest path beneath the candidate root and
rejects any symlink component before resolving the file. This closes a path
alias that previously reached the same in-root manifest by a symlink. Focused
tests passed 9/9 and `py_compile` passed. This validates path integrity only;
it does not establish provenance, candidate acceptance, or patch promotion.
Evidence: `target/headstart/evidence/p3g-manifest-symlink-m429-tests.log` and
`target/headstart/evidence/p3g-manifest-symlink-m429-pycompile.log`.

## M434 — no-follow open for manifest path components

The validator now walks and opens the manifest beneath the resolved
candidate-root directory descriptor, applying no-follow flags at every
component. This closes the race between lexical symlink checks and path-based
resolve/read. A regression swaps the manifest for an outside symlink after the
lexical check. Focused tests passed 10/10 and `py_compile` passed. This
validates local path integrity only; provenance, candidate acceptance, and
promotion remain open. Evidence:
`target/headstart/evidence/p3g-manifest-symlink-race-m434-tests.log` and
`target/headstart/evidence/p3g-manifest-symlink-race-m434-pycompile.log`.

## M451 — No-follow snapshot inventory reads

`_inventory` previously checked entries by pathname and later reopened regular
files by pathname to hash them. A concurrent replacement could therefore
substitute an outside symlink after the checks. Inventory now walks and opens
snapshot entries relative to pinned directory descriptors, uses `O_NOFOLLOW`
for each directory and file open, checks the opened file descriptor is regular,
and hashes bytes read once from that descriptor. A deterministic regression
swaps a checked snapshot file for an outside symlink immediately before its
open; validation rejects it. The focused suite passed 11/11 and `py_compile`
passed. Evidence:
`target/headstart/evidence/p3g-inventory-no-follow-m451-tests.log` and
`target/headstart/evidence/p3g-inventory-no-follow-m451-pycompile.log`.

## M604 — Reject shared inodes across paired snapshots

The validator now tracks `(st_dev, st_ino)` from each already-open regular-file
descriptor across the baseline and candidate inventories. A hard link shared
between the two trees invalidates the pair because writing through either path
would mutate both snapshots. The regression creates a cross-snapshot hard link
and confirms validation rejects it. This check does not reject separate files
with identical bytes. The focused suite passed 12/12, `py_compile` passed, and
scoped diff checks passed. Evidence:
`target/headstart/evidence/p3g-snapshot-inode-isolation-m604-tests.log` and
`target/headstart/evidence/p3g-snapshot-inode-isolation-m604-pycompile.log`.

## M611 — Bound manifest input before JSON parsing

The validator reads at most 32 MiB plus one byte from the descriptor it already
opened with no-follow checks. It rejects a larger manifest before decoding or
parsing it. The bound leaves headroom over existing local candidate manifests
of roughly 17.6 MB. A focused regression sets the bound to 16 bytes, verifies
the oversize error, and confirms the read request is exactly limit plus one.

Validation: `python3 scripts/tests/test_validate_candidate_snapshot.py`
passed 13/13; Python compilation passed for the validator and its tests. Logs:
`target/headstart/evidence/p3g-bounded-manifest-tests-m611-20261021.log` and
`target/headstart/evidence/p3g-bounded-manifest-pycompile-m611-20261021.log`.

This establishes bounded local manifest input. Candidate execution, correctness,
build, GUI, performance, evidence aggregation, and promotion remain separate
gates.

## M616 — Bound snapshot inventory work

Each snapshot inventory is limited to 100,000 filesystem entries and 4 GiB of regular-file content. The entry count is enforced while enumerating; aggregate file size is checked before hashing, and streamed bytes are counted to catch concurrent file growth. The limits leave headroom over the largest prepared tree recorded in this task while bounding local metadata and hashing work. Boundary and simulated-growth tests passed 16/16; Python compilation passed. Evidence: `target/headstart/evidence/m616-stage10-snapshot-budget-tests-20261021.log` and `target/headstart/evidence/m616-stage10-snapshot-budget-pycompile-20261021.log`.

## M1253 — Avoid descriptor growth with snapshot depth

The inventory stack stores directory component names and identities, then
reopens active paths from the pinned root with no-follow opens. It validates
each recorded ancestor identity and retains the directory before/after metadata
check without keeping one descriptor per depth or duplicating full path
prefixes in every frame. This preserves M616's entry and byte limits while
avoiding Python recursion and depth-dependent descriptor exhaustion. A
1,100-level regression lowers `RLIMIT_NOFILE` to 32 for inventory; it was added
but not run in this coding pass. Python syntax compilation passed. See
`docs/tasks/M1253_STAGE10_P3G_BOUND_INVENTORY_DESCRIPTOR_DEPTH.md`.
