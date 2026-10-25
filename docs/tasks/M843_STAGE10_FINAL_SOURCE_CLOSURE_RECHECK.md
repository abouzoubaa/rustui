# M843 — Stage 10 / final candidate source-closure recheck

## Defect

The pipeline ran P3G before the P3H reports, P3I/J comparison, and P3K patch
reproduction, but did not recheck both complete source closures before writing
its terminal result. P3K's own final inventory check covers the patched package
trees only. A workspace source file outside that package could therefore drift
after P3G while the local validators still appeared successful.

## Change

The runner now retains the source-hash-manifest digest and re-inventories both
copied source trees after the local validator sequence. It also verifies the
source-hash manifest itself remains byte-identical. Drift is recorded as a
failed `P3G-final-source-integrity` event, sets the P3F invocation exit status
to 1, and makes `pipeline-result.json` terminal status `failed`. P3F evidence
continues to bind the original immutable-manifest digest.

## Verification

The focused source-drift regression passed 1/1. The complete candidate-pipeline
test module passed 34/34.

This check establishes source closure integrity at the end of local validation.
It does not run build, correctness, GUI, performance, or promotion gates.
