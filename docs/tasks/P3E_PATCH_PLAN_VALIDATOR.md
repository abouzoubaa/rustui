# P3E: Read-only patch-plan manifest validator

**Status: bounded local validator implemented.** This is a proposed local
manifest format; the P3 design and `patches/README.md` do not define a governing
schema. Validation checks manifest structure and referenced local-file
integrity only. A `valid` report does not establish provenance, compatibility,
approval, candidate acceptance, or permission to apply or promote a patch.

## Scope and safety boundary

`scripts/validate_patch_plan.py` uses only the Python standard library and
performs offline, read-only checks. It does not apply patches, copy or fetch
source, invoke build/test/other commands, create candidate workspaces, update
vendor files, change dependency resolution, promote a candidate, or confer an
approval. The accepted snapshot and its Cargo cache remain outside its action
surface.

The candidate manifest is plan input. Execution results and approval decisions
belong in separate evidence and human-decision records as described in
`docs/tasks/P3_PATCH_VALIDATION_DESIGN.md`. Gate values in this plan are always
`not_run`; attempts to record `passed`, `failed`, or another status are rejected.

## Proposed schema version 1

The JSON object has these fields:

- `schema_version`: integer `1`.
- `candidate`: exactly `id`, `snapshot_sha256`, and `parent_manifest_sha256`.
  The ID is a safe identifier; the two digest fields are lowercase SHA-256
  strings identifying pinned metadata. The validator checks digest syntax, not
  a complete directory-tree hash.
- `source`: exactly `repository`, `revision`, and `packages`. `revision` is a
  full lowercase 40-character commit SHA. Each package has exactly `name`,
  `version`, `checksum_file`, `checksum_sha256`, `license_expression`, and
  `license_files`.
- `patches`: an ordered array of objects with exactly `path` and `sha256`.
  Paths are relative to `--root`; every patch file is hashed as raw bytes and
  checked in the array's declared order. Duplicate patch paths are rejected.
- `gates`: optional object containing only `build`, `correctness`, `gui`, and
  `performance`, each set to `not_run`. If omitted, the report supplies
  `not_run` for all four gates.

All referenced checksum, license, and patch files must resolve to regular
files beneath the supplied root. Absolute paths, Windows drive paths, empty or
dot/traversal segments, NUL bytes, missing files, and symlinks resolving
outside root are rejected. The manifest file itself must also resolve beneath
root. The checksum metadata file is verified against its declared SHA-256;
license expression and at least one existing license file are required for
each package. JSON duplicate keys and unknown or missing schema fields are
rejected.

The source revision, candidate snapshot digest and parent-manifest digest are
declarative pins. This validator does not fetch a source revision, verify a Git
checkout, compute a snapshot tree digest, establish that a license expression
is legally correct, or prove that the input metadata came from an authorized
reviewer.

## Invocation and report

```sh
python3 scripts/validate_patch_plan.py \
  target/headstart/candidates/example/manifest.json \
  --root target/headstart/candidates/example
```

The command prints one JSON report to stdout. It includes `status`, a bounded
`scope` statement, candidate ID and source revision when valid, patch count,
gate states, and an `errors` array. Exit status is `0` when the manifest and
referenced local files satisfy this format, and `1` for invalid input. The
report does not include raw metadata file contents.

## Verification

`python3 scripts/tests/test_validate_patch_plan.py` exercises valid identity
and default gates, raw-byte digest mismatch, path traversal and absolute/NUL
paths, symlink escape, missing checksum/license files, duplicate patch paths,
gate promotion, duplicate JSON keys, unknown fields, and manifest-root
containment. Eight focused tests passed; see
`target/headstart/p3e-validator-tests.log`. The validator does not run Cargo or
any candidate command.

This task implements only a local format and validator. It does not authorize
candidate intake, patch application, dependency or revision changes, GPUI Fast,
advanced rendering, candidate build/test/native/performance work, or promotion.
