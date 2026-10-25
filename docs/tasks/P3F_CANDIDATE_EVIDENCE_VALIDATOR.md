# P3F: Read-only candidate evidence validator

**Status: bounded local validator implemented.** P3F validates one supplied
candidate evidence JSON record and its referenced local files. It does not
implement the P3 candidate pipeline or establish candidate approval, source
provenance, gate acceptance, or permission to apply or promote patches.

## Scope

`scripts/validate_candidate_evidence.py` uses only the Python standard library.
It reads the supplied record and files beneath `--root`, prints a JSON report,
and exits nonzero for invalid input. It does not create or update candidate
state, fetch sources, invoke commands, run Cargo, apply patches, modify vendor
files or caches, or promote a candidate.

The validator checks only local integrity. It cannot prove that a record is
append-only, that the referenced manifest hash identifies an approved manifest,
that provenance claims are true, or that a human performed GUI acceptance.
Those facts require separately attributable pipeline and review evidence.

## Record size bound

The supplied JSON evidence record is limited to 1 MiB. The validator reads at
most one byte beyond that limit before rejecting it, so oversized input is not
fully allocated or parsed.

## Referenced artifact byte bound

The command log and all supporting evidence files share a 4 GiB aggregate
regular-file byte budget. The validator checks opened-file sizes before hashing
and also counts streamed bytes, so a file that grows after its size check is
rejected without hashing beyond the remaining budget.

## Proposed local record shape

The record contains exactly:

- `manifest_sha256`: lowercase SHA-256 digest referencing the candidate input
  manifest.
- `command`: nonempty array of command-line strings.
- `exit_status`: nonnegative observed process exit status.
- `log`: one `{ "path", "sha256" }` object for the raw command log.
- `evidence`: array of `{ "path", "sha256" }` objects for supporting evidence.
- `provenance`: object containing recorded provenance; unknown values may be
  recorded as `"unknown"`.
- `gates`: exactly `build`, `correctness`, `gui`, and `performance`, each set to
  `not_run`, `passed`, `failed`, or `inconclusive`.

This is a local serialization of fields already called for by the P3 design;
P3 itself does not define a governing evidence-record schema. Artifact paths
are slash-separated relative paths beneath `--root`. Absolute, drive, NUL,
empty, dot, and traversal paths are rejected, as are missing files, non-regular
files, all symlink path components, duplicate JSON keys, unknown fields, and
raw-byte hash mismatches. Files are opened by walking descriptor-relative
components with no-follow flags, so a path swap after validation cannot redirect
the read outside `--root`. Artifact hashes are computed incrementally. A gate
marked `passed` requires exit status zero, a valid log, and at least one valid
evidence artifact. A `valid` report checks record integrity only and is not an
acceptance decision.

## Invocation

```sh
python3 scripts/validate_candidate_evidence.py \
  target/headstart/candidates/example/evidence/build.json \
  --root target/headstart/candidates/example
```

The report contains `status`, a bounded `scope`, the manifest digest when
syntactically valid, gate states, and an `errors` array. It does not include
artifact contents.

## Verification

`python3 scripts/tests/test_validate_candidate_evidence.py` covers a valid
record, passing-gate requirements, digest mismatch, unsafe paths, symlink
rejection, record-root containment, duplicate JSON keys, unknown fields, and
invalid gate states. The descriptor-safe reader also prevents a symlink swap
between containment validation and file opening. The validator does not run
Cargo or candidate commands. Raw output is saved at
`target/headstart/evidence/p3f-descriptor-safe-m324-20261021.log`.

## M342 — Reject symlinked evidence-record paths

The CLI keeps the supplied record path lexical relative to the lexical root and
resolves only the root before descriptor-relative no-follow loading. This lets
the reader reject an in-root record symlink rather than follow it. The focused
suite passed 7/7 and `py_compile` passed. This checks local record integrity
only; no candidate ran or was approved.
Logs: `target/headstart/evidence/p3f-record-symlink-m342-20261021.log` and
`target/headstart/evidence/p3f-record-symlink-pycompile-m342-20261021.log`.


## M601 — Distinct candidate evidence artifact files

The read-only P3F validator now requires the raw command log and every supporting evidence reference to resolve to distinct regular files. It compares device and inode values from the same safely opened descriptors used to hash the artifacts. This rejects duplicate references and hard-link aliases, while separate files with identical bytes remain valid. It prevents a command log from satisfying both evidence roles without changing any gate's acceptance meaning.

The focused validator suite passed **9/9** and Python syntax validation passed. Evidence: `target/headstart/evidence/p3f-distinct-artifact-files-m601-20261021.log` and `target/headstart/evidence/p3f-distinct-artifact-pycompile-m601-20261021.log`. Synthetic local evidence validation only; no candidate was run or approved.

## M621 — Bound referenced artifact hashing

The P3F validator caps the combined command log and supporting evidence at 4
GiB. It rejects files whose descriptor-reported sizes exceed the remaining
budget and counts streamed bytes to catch growth after that check. Focused
synthetic tests cover initial oversize and post-stat growth. This extends the
local validator's input bounds; no candidate was run or approved.
