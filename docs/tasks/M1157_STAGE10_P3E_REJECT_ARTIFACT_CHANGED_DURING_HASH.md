# M1157 — Stage 10 / reject P3E artifacts changed during hashing

P3E previously hashed a patch or checksum file from an opened descriptor but
did not compare its metadata after the read or confirm that the artifact path
still named that file. A deterministic probe replaced a patch after its final
read; P3E returned `valid` for the old digest while the path contained the new
bytes.

The validator now compares descriptor identity and size/mtime/ctime before and
after hashing, then reopens the path relative to the pinned root and checks that
it still names the same file version. A focused regression reproduces the
replacement and requires an invalid report.

Focused validation passed:

- P3E mutation-race, bounded-hashing, and exact-byte digest tests (3 tests).
- Full `python3 -m unittest scripts.tests.test_validate_patch_plan` suite
  (14 tests).
- Python bytecode compilation and scoped `git diff --check`.

This check rejects changes observed during hashing and path replacement before
the final identity check. It does not lock artifacts after validation or
authorize applying, accepting, or promoting a candidate.
