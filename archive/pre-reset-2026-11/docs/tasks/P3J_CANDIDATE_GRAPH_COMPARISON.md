# P3J: Normalized baseline/candidate dependency graph comparison

**Status: bounded local graph comparison implemented.** M181 extends the
read-only P3H/P3I reports and comparator so a pair of schema version 2
effective-source reports can be compared by normalized package identities,
resolved dependency edges, and enabled per-package features.

## Normalization and comparison

P3H schema version 2 identifies path packages by their path relative to the
selected workspace and registry packages by registry source plus name/version.
It records dependency aliases, package identities, Cargo dependency kind and
target metadata, and sorted enabled feature names. The normalized graph
contains no absolute candidate paths and is hashed canonically. P3I compares
edge sets and feature sets directly after validating graph nodes against the
package inventories and recomputing the graph digest.

Both reports must have the same locked-offline Cargo feature/platform
selection. The result reports package, edge, and per-package feature changes.
It does not establish source provenance, licensing, patch approval, build or
runtime correctness, GUI or performance acceptance, or promotion.

Dependency kind and target tuples are part of each normalized edge identity.
If either field changes, comparison reports a removed edge and an added edge
instead of graph equality.

## P3H v1 compatibility

P3H schema version 1 reports remain supported for package-inventory comparison
only. Their old graph digest may contain candidate-root paths. The comparator
labels it as a legacy opaque digest and reports graph comparison as unavailable.
A mixed v1/v2 pair is also inventory-only. Graph equality is reported only
when both inputs provide valid schema version 2 normalized graph data.

## Cargo metadata isolation

Candidate `.cargo/config.toml` is loaded so the report observes effective
sources, while the reporter follows it with a Cargo CLI override for
`build.target-dir` pointing to a temporary directory. This is necessary because
the prepared candidate config itself sets `target/headstart`. The report records
the override and isolation facts. P3J invokes only locked-offline Cargo metadata;
it performs no build, tests, GUI validation, patch application, or promotion.

## Verification evidence

The P3H report suite passes 9 tests and the P3I/P3J comparison suite passes 11
tests, including path-independent graph identities, changed edges/features,
malformed graph rejection, graph-digest integrity, v1 and mixed-version
inventory-only behavior, and the explicit target-directory override. Fresh
Quad baseline and candidate reports each contain 804 packages;
their normalized graph hashes match and the comparison found no package, edge,
or feature differences. This is evidence for that selected metadata/platform
configuration only.

Raw evidence:

- `target/headstart/evidence/p3j-source-report-tests-m181-20261020.log`
- `target/headstart/evidence/p3j-source-comparison-tests-m181-20261020.log`
- `target/headstart/evidence/p3j-quad-baseline-source-resolution-20261020.json`
- `target/headstart/evidence/p3j-quad-candidate-source-resolution-20261020.json`
- `target/headstart/evidence/p3j-quad-graph-comparison-20261020.json`

## M318 — strict JSON integer validation

The comparator now requires actual JSON integers for the P3H report schema
version, normalized graph schema version, package count, and source counts.
Python otherwise treats JSON `true` as equal to integer `1`, which could let a
tampered v2 report masquerade as a legacy v1 inventory-only report or satisfy
an inventory count check. Focused regressions cover those boolean substitutions;
the combined P3I/P3J comparison suite passes 15/15, `py_compile` passes, and
the saved 804-package Quad reports still produce a valid equal inventory and
normalized graph comparison. Evidence:
`target/headstart/evidence/p3i-p3j-strict-json-integer-tests-20261020.log`,
`target/headstart/evidence/p3i-p3j-strict-json-integer-pycompile-20261020.log`,
and `target/headstart/evidence/p3i-p3j-strict-json-integer-metadata-smoke-20261020.json`.

## M327 — Require normalized package identity

Schema-v2 package rows must include the normalized `identity` emitted by the
P3H report. Missing values now fail closed rather than being reconstructed from
other package fields. The focused comparator suite passed 16/16 and
`py_compile` passed using fixtures only. Logs:
`target/headstart/evidence/p3j-v2-package-identity-tests-m327-20261021.log` and
`target/headstart/evidence/p3j-v2-package-identity-pycompile-m327-20261021.log`.

## M330 candidate — Require dependency-kind metadata on graph edges

P3J schema-v2 now rejects dependency edges whose `kinds` array is empty. A self-consistent report digest did not make an edge with missing Cargo kind/target metadata meaningful; accepting it could compare an incomplete graph as valid. The focused comparator suite passed 17 tests, including an empty-kinds mutation with a recomputed graph digest, and `py_compile` passed. This candidate does not change the roadmap or claim candidate acceptance. Logs: `target/headstart/evidence/p3j-nonempty-dependency-kinds-m330-candidate-tests-20261021.log` and `target/headstart/evidence/p3j-nonempty-dependency-kinds-m330-candidate-pycompile-20261021.log`.

## M333 candidate — Reject blank dependency-kind metadata

P3J schema-v2 rejects dependency kind or target metadata containing only whitespace, as well as empty strings. A temporary report with `kind: ""` and a recomputed graph digest was accepted before this check. The focused comparator suite passed 18 tests, including blank-kind and blank-target mutations with recomputed graph digests, and `py_compile` passed. This remains a local report-integrity candidate, not acceptance evidence. Logs: `target/headstart/evidence/p3j-nonblank-dependency-metadata-m333-candidate-tests-20261021.log` and `target/headstart/evidence/p3j-nonblank-dependency-metadata-m333-candidate-pycompile-20261021.log`.
