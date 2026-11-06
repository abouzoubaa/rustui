# P2b: Dependency reuse report

## Use

Run this after a successful Cargo build that was recorded with
`--message-format=json`:

```sh
python3 scripts/verify_dependency_reuse.py path/to/build.jsonl \
  --report target/headstart/evidence/dependency-reuse.json
```

Repeat `--package NAME` to select required packages. Without it, the verifier
requires `gpui-pre`, `gpui-pre-platform`, and `gpui-component`. The command
keeps its existing stdout and exit codes: `0` means every requested package
has at least one artifact and every artifact was reported fresh; `1` means a
requested package is missing or has a rebuilt artifact; `2` means the input
log is invalid/unreadable or the report cannot be written. The report path cannot resolve to or identify the input log, including through
an existing hard link.

## Evidence and limits

The optional JSON report has `schema_version: 1`, the resolved log path and its
SHA-256, successful-build status, requested package names, and a package entry
for each requested name. Package status is `missing`, `rebuilt`, or
`all_fresh`; each recorded artifact includes Cargo's package ID, target name
and kinds, profile, filenames, and `fresh` value. The overall reuse field is
`confirmed` only when all requested packages have artifacts and every artifact
is fresh.

This is evidence from Cargo's `compiler-artifact` freshness events for this
single log. It does not establish runtime speed, benchmark results, executable
reuse from `cargo check`, or reuse across builds. It adds no dependencies and
does not invoke Cargo.

## Recorded execution

The utility ran successfully (exit 0) against
`target/headstart/roadmap-milestone5-check-accepted.jsonl`, producing
`target/headstart/evidence/roadmap-milestone5-reuse.json`. GPUI and Components each
had two fresh artifact records, and GPUI Platform had one. The source was a Cargo
check run, so this establishes check-artifact freshness only. No tests or benchmark
runs were performed. Python syntax parsing and CLI help also completed successfully.
