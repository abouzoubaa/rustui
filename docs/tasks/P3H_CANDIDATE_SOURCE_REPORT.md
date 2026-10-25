# P3H: Read-only effective Cargo source report

**Status: bounded verifier implemented.** P3H schema version 2 runs Cargo
metadata in locked, offline mode for one already-prepared candidate, then
reports every resolved package ID, source kind, candidate-relative manifest
path, normalized identity, dependency edges, and enabled per-package features. It does not
prepare snapshots, resolve by fetching, patch sources, build, test, accept, or
promote a candidate.

## Boundary

`scripts/report_candidate_sources.py` runs `cargo metadata --format-version 1
--locked --offline` against the selected candidate workspace. It removes
inherited `CARGO_*` environment overrides, runs Cargo from a temporary working
directory outside the repository, and explicitly loads candidate-local Cargo
configs from the candidate root down to the selected workspace. For each
directory it prefers `.cargo/config.toml`, falling back to legacy
`.cargo/config`; more specific configs are applied later, matching Cargo's
ancestor-config precedence. It uses a temporary Cargo home and target
directory. It also passes `--config build.target-dir=<temporary path>` after the
candidate config chain so that a candidate `[build] target-dir` cannot redirect
even metadata-related Cargo output. This prevents ancestor repository config
and user Cargo config from selecting sources or build settings. The report
records the most-specific candidate config path and hash, plus the ordered
config chain and hashes; the candidate Cargo lockfile is checked unchanged. The
metadata must identify the requested workspace and contain a complete
`packages`/`resolve.nodes` graph. Normalized package identities use source kind,
name/version, registry URL or workspace-relative path; they exclude the
absolute candidate root. Dependency edges preserve Cargo's crate alias and
dependency kind/target metadata. Enabled feature lists are recorded per package.
The graph digest is computed from this normalized representation. Every path package manifest must remain inside
the selected candidate workspace. Every registry package manifest must be
inside that workspace's `vendor/` directory. Git and unknown Cargo source kinds,
external paths, and symlinked source paths fail closed.

This proves the observed metadata package paths for the requested feature and
platform selection are candidate-local. It does not prove upstream identity,
licenses, checksum/provenance claims, equivalence to another lock graph, build
correctness, GUI behavior, performance, patch approval, or promotion. Cargo's
offline mode prevents network fetches; this stage runs no build or test command.

Cargo metadata stdout and stderr are drained concurrently and capped at 32 MiB
combined. If they exceed this limit, the reporter terminates Cargo and returns
an invalid report. The final JSON written to stdout has a separate 32 MiB
UTF-8 byte limit, including its trailing newline. If the serialized report
exceeds that limit, the reporter emits a compact schema-versioned invalid JSON
report with an output-limit error and exits nonzero; it does not write a
truncated report. The two limits apply independently to Cargo's output and the
reporter's JSON output.

## Invocation

```sh
python3 scripts/report_candidate_sources.py \
  target/headstart/candidates/quad-stable-sort-20261018-a \
  --workspace candidate/source
```

Use `--features`, `--all-features`, `--no-default-features`, or
`--filter-platform <triple>` to describe the intended metadata selection. The
selection is included in the JSON report; a report covers only that selection.
The command prints a JSON report and exits nonzero if Cargo metadata fails or
any source falls outside the boundary.

## Schema compatibility

Schema version 1 reports remain accepted by the comparison tool as
package-inventory-only inputs. Their graph digest was based on Cargo package
IDs, which can include candidate-specific absolute paths; it is retained as an
opaque legacy digest and is never used as a graph-equivalence result. Only two
schema version 2 reports with validated `resolved_graph` data can be compared
for dependency edges and enabled features. Mixed-version comparisons remain
inventory-only.

## Evidence

The prepared Quad-sort V1 workspace was inspected using Cargo metadata with
`--locked --offline`, from an isolated temporary working directory, with its
candidate-local Cargo config explicitly selected, and with temporary Cargo home
and target directories. Metadata resolved 804 packages and 804 graph nodes: 8 path packages
and 796 registry packages. All 804 manifest paths were within
`candidate/source`; `gpui-pre 0.3.9` resolved from
`candidate/source/path-sources/gpui-pre-0.3.9`, and `gpui-pre-platform 0.3.9`
resolved from `candidate/source/vendor/gpui-pre-platform-0.3.9`. No build or
test was run for this inspection. The report records candidate config SHA-256
`aa0967039ab47bbc07e0ea1b9242d4b5c7aa1465a37d816decfab6d7cb6d9aec` and lockfile
SHA-256 `0d64fcc7a621953c1d0674669d67dbcf629adc37713ce38544ca52542b7301e3`. The
reproducible machine-readable report is
`target/headstart/evidence/p3h-quad-source-resolution-20261020.json`.

The M178 six focused source-report tests pass in
`target/headstart/evidence/p3h-source-report-tests-20261020.log`. The M181
schema 2 reporter tests and target-directory override check pass in
`target/headstart/evidence/p3j-source-report-tests-m181-20261020.log`. Fresh
locked/offline reports for the prepared Quad baseline and candidate are under
`target/headstart/evidence/p3j-quad-{baseline,candidate}-source-resolution-20261020.json`.
Candidate intake, immutable provenance, patch
application, correctness/build, GUI, performance, evidence aggregation, and
separate human promotion remain open.

## M625 — Reject dependency edges without kinds

P3H metadata validation rejects resolved dependency edges whose `dep_kinds` list is empty, matching the P3J reader contract. The focused source-report suite passed 11/11.
