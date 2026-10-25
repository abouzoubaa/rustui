# M1111 — Stage 10 P3G reject a symlinked candidate root

## Contract

The snapshot validator must not follow a symlink supplied as the final
candidate-root path and then report that candidate as a valid snapshot.

## Change

Non-inherited CLI mode now resolves the parent directory while retaining the
candidate root as the final no-follow path component. Manifest paths are
rebased to the validated root, preserving supported parent aliases such as
macOS `/var`.

## Evidence

`python3 -m unittest -q scripts.tests.test_validate_candidate_snapshot` passed
(29 tests), including `test_rejects_symlinked_candidate_root`. A scoped
whitespace check passed.

This validates a snapshot-root boundary only; no candidate was run or
accepted.
